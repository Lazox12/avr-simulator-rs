use crate::emit;
use crate::error::Result;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::sync::{
    mpsc, mpsc::{Receiver, Sender,TryRecvError},
    Mutex,
};
use std::{thread};

use crate::sim::worker;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    Run,
    #[default]
    Pause,
    Stop,              //after setting stop use thread.join()
    Break(u32),        //breakpoint
    Next,              //next inst
    Skip,              //only on call runs until fn returns
    Watch([u8; 8]),    //we accept this as string
    WatchUpdate(bool), // update watchlist variables when running
}
#[derive(Debug)]
pub enum Response {
    Res(Result<()>),
    Join,
    Ready,
}


static CONTROLLER: Mutex<Controller> = Mutex::new(Controller::new());

#[derive(Debug,Default,PartialEq, Serialize, Clone)]
pub enum WorkerState{
    #[default]
    NotInitialized,
    Running,
    Error(String),
    Initializing,

}
impl WorkerState {
    pub fn set(&mut self, state: WorkerState)->Result<()> {
        println!("WorkerState::set::{:?}",state);
        emit!("sim_state", &state);
        *self = state;
        Ok(())

    }
}

#[derive(Debug, Default)]
pub struct Controller {
    tx: Option<Sender<Action>>,
    rx: Option<Receiver<Response>>,
    handle: Option<thread::JoinHandle<()>>,
    worker_state: WorkerState
}
impl Controller {
    pub const fn new() -> Controller {
        Controller {
            tx: None,
            rx: None,
            handle: None,
            worker_state: WorkerState::NotInitialized,
        }
    }
    fn init(&mut self) -> Result<()> {
        println!("contoller::init");
        let (tx, rx_t) = mpsc::channel::<Action>();
        self.tx = Some(tx);
        let (tx_t, rx) = mpsc::channel::<Response>();
        self.rx = Some(rx);
        self.handle = Some(thread::spawn(|| {
            println!("thread id : {:?}",thread::current().id());
            let mut worker = worker::Worker::default();
            worker.init(rx_t, tx_t);
            loop {
                if worker.thread_run() {
                    break;
                };
            }
            println!("thread finished");

        }));
        self.worker_state.set(WorkerState::Initializing)?;
        Ok(())
    }
    pub fn start()->Result<()>{
        CONTROLLER
            .lock()
            .map_err(|e| anyhow!("Poison Error:{}", e))?
            .init()
    }
    fn deinit(&mut self) -> Result<()> {
        println!("contoller::deinit");
        if  self.worker_state ==WorkerState::NotInitialized{
            return Ok(());
        }
        self.tx.take().ok_or(anyhow!("no tx available"))?.send(Action::Stop)?;
        ||->Result<()>{
            if let Some(rx) = self.rx.take(){
                loop {
                    match rx.try_recv(){
                        Ok(Response::Join)=>{
                            return Ok(());
                        }
                        Ok(d) => {
                            println!("contoller::deinit::recvData:{:?}",d);
                        }
                        Err(TryRecvError::Disconnected) => {
                            return Err(anyhow!("thread disconnected"));
                        }
                        Err(TryRecvError::Empty)=>{}
                    }
                }
            }
            Err(anyhow!("invalid case"))
        }()?;

        self.handle
            .take()
            .ok_or(anyhow!("no handle"))?
            .join()
            .map_err(|e| anyhow!("JoinError:{:?}", e))?;
        self.worker_state.set(WorkerState::NotInitialized)?;
        self.rx = None;
        Ok(())
    }
    pub fn stop()-> Result<()> {
        CONTROLLER
            .lock()
            .map_err(|e| anyhow!("Poison Error:{}", e))?
            .deinit()
    }

    #[allow(dead_code)]
    pub fn do_action(action: Action) -> Result<()> {
        CONTROLLER
            .lock()
            .map_err(|e| anyhow!("Poison Error:{}", e))?
            .do_action_iner(action)
    }
    fn do_action_iner(&mut self, action: Action) -> Result<()> {
        if let Some(tx) = self.tx.as_ref() {
            tx.send(action).map_err(|e| anyhow!("Send Error:{}", e))?;
        } else {
            self.init()?;
            self.do_action_iner(action)?;
        }
        Ok(())
    }
    pub async fn do_action_and_wait(action: Action) -> Result<()> {
        let mut control = CONTROLLER.lock().map_err(|e| anyhow!("Poison Error:{}", e))?;

        control.do_action_iner(action)?;

        if let Some(rx) = control.rx.as_mut() {
            let data = rx.recv().map_err(|_| TryRecvError::Disconnected);
            control.eval_update(data)
        } else {
            Err(anyhow!("Controller not initialized"))
        }
    }
    pub fn update() -> Result<()> {
        CONTROLLER
            .lock()
            .map_err(|e| anyhow!("Poison Error:{}", e))?
            .update_iner()
    }
    fn update_iner(&mut self) -> Result<()> {
        match &self.worker_state{
            WorkerState::NotInitialized => { return Ok(())}
            WorkerState::Running => {}
            WorkerState::Error(e) => {
                emit!("error",e);
                self.deinit()?;
                return Ok(())
            }
            WorkerState::Initializing => {}
        }
        if let Some(rx) = self.rx.as_mut() {
            let data = rx.try_recv();
            self.eval_update(data)
        } else {
            Ok(())
        }

    }
    fn eval_update(&mut self, data:std::result::Result<Response, TryRecvError>) -> Result<()> {
        match  data{
            Ok(Response::Join)=> {Err(anyhow!("Invalid state")) },
            Ok(Response::Res(Ok(_))) => Ok(()),
            Ok(Response::Res(Err(e))) => {
                self.worker_state.set(WorkerState::Error(e.to_string()))?;
                Ok(())
            },
            Ok(Response::Ready)=>{self.worker_state.set(WorkerState::Running)?;Ok(())}
            Err(TryRecvError::Disconnected) => {
                Err(anyhow!("Worker thread disconnected"))
            }
            Err(TryRecvError::Empty)=>{Ok(())}
        }
    }
}
