use crate::emit;
use crate::error::Result;
use crate::project::PROJECT;
use crate::sim::instruction::Instruction;
use crate::sim::memory::Memory;
use crate::sim::sim::Sim;
use anyhow::anyhow;
use device_parser::{AvrDeviceFile, Register};
use opcode_gen::Opcode;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::{
    mpsc, mpsc::{Receiver, Sender,TryRecvError},
    Mutex,
};
use std::thread::sleep;
use std::time::Duration;
use std::{thread, time};
use tauri::Emitter;

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
enum Response {
    Res(Result<()>),
    Join,
    Ready,
}

#[derive(Default)]
struct Worker<'a> {
    atdf: &'static AvrDeviceFile,
    reg_map: Option<&'static phf::Map<u64, &'static Register>>,
    memory: Memory,
    sim: Sim<'a>,
    action: Action,
    action_prev: Action,
    action_executed: bool,
    breakpoints: Vec<u32>,
    watch_list: HashMap<String, u32>,
    update_watch_list: bool,
    rx: Option<Receiver<Action>>,
    tx: Option<Sender<Response>>,
}
impl<'a> Worker<'a> {
    pub fn init(&mut self, rx: Receiver<Action>, tx: Sender<Response>) -> Option<()> {
        self.rx = Some(rx);
        self.tx = Some(tx);

        let f = || -> Result<()> {
            let mut project_lock = PROJECT.lock().map_err(|e| anyhow!("Poison Error:{}", e))?;
            let mcu = project_lock.get_state()?.mcu.clone();
            self.atdf = device_parser::get_tree_map()
                .get(&*mcu)
                .ok_or(anyhow!("invalid mcu"))?;


            self.sim
                .init(&mut *project_lock, self.atdf, &mut self.memory)?;
            Ok(())
        }();
        match f {
            Ok(()) => {
                if let Some(tx) = self.tx.as_ref() {
                    tx.send(Response::Ready).ok()?;
                }
                Some(())
            },
            Err(e) => {
                if let Some(tx) = self.tx.as_ref() {
                    tx.send(Response::Res(Err(e))).ok()?;
                }
                self.action = Action::Pause;
                Some(())
            }
        }
    }
    fn check_brekpoint(&mut self) -> bool {
        self.breakpoints
            .iter()
            .find(|x| self.memory.program_couter == **x)
            .is_some()
    }

    unsafe fn iner(&mut self) -> Result<bool> { // true terminates
        match self.action {
            Action::Run => {
                if self.action_prev != Action::Run {
                    self.action_prev = Action::Run;
                    emit!("sim-status", Action::Run);
                    unsafe { self.sim.execute_inst()? };
                }
                if self.check_brekpoint() {
                    self.action = Action::Pause;
                } else {
                    unsafe { self.sim.execute_inst()? }
                }
                if self.update_watch_list && self.memory.data.io.write_status {
                    self.memory.data.io.write_status = false;
                    emit!(
                        "sim-watch-list-update",
                        self.watch_list
                            .iter()
                            .map(|(key, val)| (
                                key.clone(),
                                match self.memory.data.get(*val as usize) {
                                    Some(t) => {
                                        t.clone()
                                    }
                                    None => 0u8,
                                }
                            ))
                            .collect::<HashMap<_, _>>()
                    );
                    thread::sleep(time::Duration::from_millis(10));
                }
                Ok(false)
            }
            Action::Pause => {
                if !self.action_executed {
                    self.action_executed = true;
                    emit!("sim-status", Action::Pause);
                    emit!("sim-location", self.memory.program_couter);
                    emit!("sim-register-status", self.memory.data.registers.clone());
                    println!("{:?}", self.watch_list);
                    emit!(
                        "sim-watch-list-update",
                        self.watch_list
                            .iter()
                            .map(|(key, val)| (
                                key.clone(),
                                match self.memory.data.get(*val as usize) {
                                    Some(t) => {
                                        t.clone()
                                    }
                                    None => 0u8,
                                }
                            ))
                            .collect::<HashMap<_, _>>()
                    );
                }
                sleep(Duration::from_millis(100)); //to not hold up the core
                Ok(false)
            }
            Action::Stop => {
                Ok(true)
            }
            Action::Break(address) => {
                if let Some(index) = self.breakpoints.iter().position(|x| *x == address) {
                    self.breakpoints.remove(index);
                } else {
                    self.breakpoints.push(address);
                }
                self.action = self.action_prev;
                emit!("breakpoints-update", self.breakpoints.clone());
                Ok(false)
            }
            Action::Next => {
                unsafe { self.sim.execute_inst()? }
                self.action = Action::Pause;
                Ok(false)
            }
            Action::Skip => {
                loop {
                    let i: &Instruction = self
                        .memory
                        .flash
                        .get(self.memory.program_couter as usize)
                        .ok_or(anyhow!("invalid address:{}", self.memory.program_couter))?;
                    unsafe { self.sim.execute_inst()? };
                    match i.get_raw_inst()?.name {
                        Opcode::CALL | Opcode::ICALL | Opcode::EICALL | Opcode::RCALL => {
                            unsafe { self.iner()? };
                        }
                        Opcode::RET => {
                            self.action = Action::Pause;
                            break;
                        }
                        _ => {}
                    }
                }
                Ok(false)
            }
            Action::Watch(data) => {
                if self.reg_map.is_none() {
                    self.reg_map = device_parser::get_register_map(
                        &PROJECT
                            .lock()
                            .map_err(|e| anyhow!("poison error:{}", e))?
                            .get_state()?
                            .mcu,
                    );
                }
                let mut name: String = String::new();
                data.iter().for_each(|x1| {
                    if *x1 > 0 {
                        name += &*(x1.clone() as char).to_string()
                    }
                });
                name = name.to_uppercase();
                let address: u32 = *self
                    .reg_map
                    .ok_or(anyhow!("could not get common regs"))?
                    .into_iter()
                    .find(|(_, reg)| reg.name == name)
                    .ok_or(anyhow!("invalid register"))?
                    .0 as u32;
                if let Some(_index) = self.watch_list.iter().position(|(key, _)| *key == name) {
                    self.watch_list.remove(&name.to_string());
                } else {
                    self.watch_list.insert(name, address);
                }
                self.action = self.action_prev;
                self.memory.data.io.watchlist = self
                    .watch_list
                    .iter()
                    .map(|(_key, val)| val.clone())
                    .collect();
                emit!(
                    "sim-watch-list-update",
                    self.watch_list
                        .iter()
                        .map(|(key, val)| (
                            key.clone(),
                            match self.memory.data.get(*val as usize) {
                                Some(t) => {
                                    t.clone()
                                }
                                None => 0u8,
                            }
                        ))
                        .collect::<HashMap<_, _>>()
                );
                Ok(false)
            }
            Action::WatchUpdate(data) => {
                self.action = self.action_prev;
                self.update_watch_list = data;
                emit!("auto_update_status", self.update_watch_list);
                Ok(false)
            }
        }
    }
    fn thread_run(&mut self) ->bool {
        let mut return_res=false;
        if let Some(rx) = self.rx.as_ref() {
            match rx.try_recv() {
                Ok(action) => {
                    self.action_prev = self.action;
                    self.action = action;
                    self.action_executed = false;
                    return_res = true
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    println!("Worker::disconnect");
                    self.action = Action::Stop;
                }
            };
        }

        match unsafe{self.iner()} {
            Err(e) => {
                self.action = Action::Pause;
                if let Some(tx) = self.tx.as_ref() {
                    tx.send(Response::Res(Err(e))).unwrap();
                }
                false
            }
            Ok(false) => {
                if return_res {
                    if let Some(tx) = self.tx.as_ref() {
                        tx.send(Response::Res(Ok(()))).unwrap();
                    }
                }
                false
            }
            Ok(true) => {
                if let Some(tx) = self.tx.as_ref() {
                    println!("stoping controller");
                    tx.send(Response::Join).unwrap();
                }
                true
            }
        }
    }
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
            let mut worker = Worker::default();
            worker.init(rx_t, tx_t);
            loop {
                if worker.thread_run() {
                    break;
                };
            }
            println!("thread finished");

        }));
        self.worker_state.set(WorkerState::Initializing);
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
        self.worker_state.set(WorkerState::NotInitialized);
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
