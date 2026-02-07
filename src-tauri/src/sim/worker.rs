use crate::emit;
use crate::project::PROJECT;
use crate::sim::controller::Action;
use crate::sim::instruction::Instruction;
use crate::sim::memory::Memory;
use crate::sim::sim::Sim;
use anyhow::anyhow;
use device_parser::{AvrDeviceFile, Register};
use opcode_gen::Opcode;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc::TryRecvError;
use std::thread::sleep;
use std::time::Duration;
use std::{thread, time};

#[derive(Default)]
pub struct Worker<'a> {
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
    tx: Option<Sender<crate::sim::controller::Response>>,
}
impl<'a> Worker<'a> {
    pub fn init(&mut self, rx: Receiver<Action>, tx: Sender<crate::sim::controller::Response>) -> Option<()> {
        self.rx = Some(rx);
        self.tx = Some(tx);

        let f = || -> crate::error::Result<()> {
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
                    tx.send(crate::sim::controller::Response::Ready).ok()?;
                }
                Some(())
            },
            Err(e) => {
                if let Some(tx) = self.tx.as_ref() {
                    tx.send(crate::sim::controller::Response::Res(Err(e))).ok()?;
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

    unsafe fn iner(&mut self) -> crate::error::Result<bool> { // true terminates
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
    pub fn thread_run(&mut self) ->bool {
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
                    tx.send(crate::sim::controller::Response::Res(Err(e))).unwrap();
                }
                false
            }
            Ok(false) => {
                if return_res {
                    if let Some(tx) = self.tx.as_ref() {
                        tx.send(crate::sim::controller::Response::Res(Ok(()))).unwrap();
                    }
                }
                false
            }
            Ok(true) => {
                if let Some(tx) = self.tx.as_ref() {
                    println!("stoping controller");
                    tx.send(crate::sim::controller::Response::Join).unwrap();
                }
                true
            }
        }
    }
}
