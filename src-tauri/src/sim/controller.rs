use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{mpsc, mpsc::{Sender, Receiver, TryRecvError}, Mutex, MutexGuard};
use std::{thread, time};
use std::thread::sleep;
use std::time::Duration;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use phf::phf_map;
use device_parser::{AvrDeviceFile, Register};
use opcode_gen::Opcode;
use crate::project::{Project, PROJECT};
use crate::sim::sim::Sim;
use crate::error::Result;
use crate::{emit};
use crate::sim::instruction::Instruction;
use crate::sim::memory::Memory;

#[derive(Debug,Default,Serialize,Deserialize,PartialEq,Clone,Copy)]
#[serde(rename_all = "camelCase")]
pub enum Action{
    Run,
    #[default]
    Pause,
    Stop, //after setting stop use thread.join()
    Break(u32),//breakpoint
    Next, //next inst
    Skip, //only on call runs until fn returns
    Watch([u8;8]), //we accept this as string
    WatchUpdate(bool), // update watchlist variables when running
}
enum Response{
    Res(Result<()>),
    Join
}


#[derive(Default)]
struct Worker<'a>{
    atdf:&'static AvrDeviceFile,
    reg_map:Option<&'static phf::Map<u64,&'static Register>>,
    memory:Memory,
    sim:Sim<'a>,
    action: Action,
    action_prev:Action,
    action_executed:bool,
    breakpoints:Vec<u32>,
    watch_list:HashMap<String,u32>,
    update_watch_list:bool,
    rx:Option<Receiver<Action>>,
    tx:Option<Sender<Response>>,
}
impl<'a> Worker<'a> {

    pub fn init(&mut self,rx:Receiver<Action>,tx:Sender<Response>)->Option<()>{
        let f = ||->Result<()> {
            let mut project_lock = PROJECT.lock().map_err(|e| anyhow!("Poison Error:{}",e))?;
            let mcu = project_lock.state.clone().ok_or(anyhow!("invalid project state"))?.mcu;
            self.atdf = device_parser::get_tree_map().get(&*mcu).ok_or(anyhow!("invalid mcu"))?;

            self.rx = Some(rx);
            self.tx = Some(tx);

            self.sim.init(&mut *project_lock, self.atdf, &mut self.memory)?;
            Ok(())
        };
        match f() {
            Ok(())=>{Some(())}
            Err(e)=>{
                if let Some(tx) = self.tx.as_ref() {
                    tx.send(Response::Res(Err(e))).ok()?;
                }
                None
            }
        }
    }
    fn check_brekpoint(&mut self)->bool{
        self.breakpoints.iter().find(|x|{self.memory.program_couter==**x}).is_some()
    }

    unsafe fn iner(&mut self) ->Result<()>{
        match self.action {
            Action::Run => {
                if (self.action_prev !=Action::Run){
                    self.action_prev=Action::Run;
                    emit!("sim-status",Action::Run);
                    unsafe{self.sim.execute_inst()?};
                }
                if self.check_brekpoint() {
                    self.action = Action::Pause;
                }else{
                    unsafe{self.sim.execute_inst()?}
                }
                if self.update_watch_list && self.memory.data.io.write_status{
                    self.memory.data.io.write_status = false;
                    emit!("sim-watch-list-update",self.watch_list.iter().map(|(key,val)| (key.clone(),match self.memory.data.get(*val as usize){Some(t)=>{t.clone()},None=>0u8})).collect::<HashMap<_,_>>());
                    thread::sleep(time::Duration::from_millis(10));
                }
                Ok(())

            }
            Action::Pause => {
                if !self.action_executed {
                    self.action_executed = true;
                    emit!("sim-status",Action::Pause);
                    emit!("sim-location",self.memory.program_couter);
                    emit!("sim-register-status",self.memory.data.registers.clone());
                    println!("{:?}",self.watch_list);
                    emit!("sim-watch-list-update",self.watch_list.iter().map(|(key,val)| (key.clone(),match self.memory.data.get(*val as usize){Some(t)=>{t.clone()},None=>0u8})).collect::<HashMap<_,_>>());
                }
                sleep(Duration::from_millis(100)); //to not hold up the core
                Ok(())
            }
            Action::Stop => {
                Err(anyhow!("halt"))//caught in handler
            }
            Action::Break(address) => {
                if let Some(index)=self.breakpoints.iter().position(|x| *x==address){
                    self.breakpoints.remove(index);
                }else{
                    self.breakpoints.push(address);
                }
                self.action= self.action_prev;
                emit!("breakpoints-update",self.breakpoints.clone());
                Ok(())
            }
            Action::Next => {
                unsafe{self.sim.execute_inst()?}
                self.action = Action::Pause;
                Ok(())
            }
            Action::Skip => {
                loop{
                    let i:&Instruction = self.memory.flash.get(self.memory.program_couter as usize).ok_or(anyhow!("invalid address:{}",self.memory.program_couter))?;
                    unsafe{self.sim.execute_inst()?};
                    match i.get_raw_inst()?.name {
                        Opcode::CALL|
                        Opcode::ICALL|
                        Opcode::EICALL|
                        Opcode::RCALL=>{
                            unsafe{ self.iner()?};
                        }
                        Opcode::RET=>{
                            self.action = Action::Pause;
                            break
                        }
                        _=>{}
                    }
                }
                Ok(())
            }
            Action::Watch(data) => {
                if self.reg_map.is_none(){
                    self.reg_map =device_parser::get_register_map(&PROJECT.lock().map_err(|e| anyhow!("poison error:{}",e))?.get_state()?.mcu);
                }
                let mut name:String = String::new();
                data.iter().for_each(|x1| {if *x1 >0{name+= &*(x1.clone() as char).to_string() }});
                name = name.to_uppercase();
                let address:u32 =
                    *self.reg_map.ok_or(anyhow!("could not get common regs"))?
                    .into_iter()
                    .find(|(_,reg)| {reg.name ==name})
                    .ok_or(anyhow!("invalid register"))?.0 as u32;
                if let Some(index)=self.watch_list.iter().position(|(key,_)| *key==name){
                    self.watch_list.remove(&name.to_string());
                }else{
                    self.watch_list.insert(name, address);
                }
                self.action= self.action_prev;
                self.memory.data.io.watchlist = self.watch_list.iter().map(|(key,val)| val.clone()).collect();
                emit!("sim-watch-list-update",self.watch_list.iter().map(|(key,val)| (key.clone(),match self.memory.data.get(*val as usize){Some(t)=>{t.clone()},None=>0u8})).collect::<HashMap<_,_>>());
                Ok(())
            }
            Action::WatchUpdate(data) => {
                self.action = self.action_prev;
                self.update_watch_list=data;
                Ok(())
            }
        }
    }
    fn thread_run(&mut self){
        
        loop{
            if let Some(rx) = self.rx.as_ref() {
                match rx.try_recv(){
                    Ok(action)=>{
                        self.action_prev = self.action;
                        self.action = action;
                        self.action_executed = false;
                    }
                    Err(TryRecvError::Empty)=>{}
                    Err(TryRecvError::Disconnected)=>{
                        self.action = Action::Stop;
                    }
                };

            }
            if let Err(e) = unsafe{self.iner()}{
                self.action = Action::Pause;
                if let Some(tx) = self.tx.as_ref() {
                    if e.to_string() == "halt"{
                        tx.send(Response::Join).ok();
                    }else{
                        tx.send(Response::Res(Err(e))).ok();
                    }
                }
            };
        }
    }
}

static CONTROLLER: Mutex<Controller> = Mutex::new(Controller::new());

#[derive(Debug,Default)]
pub struct Controller {
    tx: Option<Sender<Action>>,
    rx: Option<Receiver<Response>>,
    handle: Option<std::thread::JoinHandle<()>>
}
impl Controller {
    pub const fn new()->Controller{
        Controller{tx:None,rx:None,handle:None}
    }
    pub fn init(&mut self) -> Result<()> {
        let (tx,rx) = mpsc::channel::<Action>();
        self.tx=Some(tx);
        let (tx2,rx2) = mpsc::channel::<Response>();
        self.rx=Some(rx2);
        self.handle = Some(std::thread::spawn(|| {
            let tx = tx2;
            let rx = rx;
            let mut worker = Worker::default();
            let res = worker.init(rx,tx);
            if res.is_none(){
                return
            }
            loop{
                worker.thread_run();
            }

        }));
        Ok(())
    }
    pub fn deinit(&mut self) -> Result<()> {
        if let Some(tx) = self.tx.as_ref() {
            tx.send(Action::Stop).map_err(|e| anyhow!("Send Error:{}",e))?;
        }
        self.handle.take().ok_or(anyhow!("no handle"))?.join().map_err(|e| anyhow!("JoinError:{:?}",e))?;
        self.tx = None;
        self.rx = None;
        Ok(())
    }
    pub fn do_action(action: Action) -> Result<()>{
        CONTROLLER.lock().map_err(|e| anyhow!("Poison Error:{}",e))?.do_action_iner(action)
    }
    fn do_action_iner(&mut self, action: Action) -> Result<()>{

        if let Some(tx) = self.tx.as_ref() {
            tx.send(action).map_err(|e| anyhow!("Send Error:{}",e))?;
        }else {
            self.init()?;
            self.do_action_iner(action)?;
        }
        Ok(())
    }
    pub fn update()-> Result<()>{
        CONTROLLER.lock().map_err(|e| anyhow!("Poison Error:{}",e))?.update_iner()    
    }
    fn update_iner(&mut self)-> Result<()>{
        if let Some(rx) = self.rx.as_mut() {
            match rx.try_recv(){
                Ok(resp)=>{
                    match resp {
                        Response::Res(r) => {
                            r
                        }
                        Response::Join=>{
                            self.deinit()
                        }
                    }
                }
                Err(TryRecvError::Empty)=>{Ok(())}
                Err(TryRecvError::Disconnected)=>{
                    self.deinit()?;
                    Ok(())
                }
            }
        }else { 
            Ok(())
        }
    }
}