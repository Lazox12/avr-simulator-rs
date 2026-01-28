#![allow(unused_mut)]

use std::io::Write;
use std::sync::LazyLock;
use anyhow::anyhow;
use bin_expr_parser_macro::execute;
use device_parser::{get_common_registers, AvrDeviceFile, CommonRegisters};
use device_parser::r#struct::common_registers::Flags;
use opcode_gen::{CustomOpcodes, Opcode, RawInst};
use crate::sim::memory::Memory;
use crate::error::Result;
use crate::project::Project;
use crate::sim::instruction::Instruction;


static mut MEMORY: LazyLock<Memory> = LazyLock::new(Memory::default);

#[derive(Debug)]
pub struct Sim<'a>{
    pub memory: &'a mut Memory,
    registers: CommonRegisters,
    pub pc_len: u32,
    pub pc_bytesize:u32, //used for calls
}
impl<'a> Default for Sim<'a> {
    fn default() -> Sim<'a> {
        let memory = unsafe{&mut *(MEMORY)};
        Sim{memory,registers:CommonRegisters::default(),pc_len: 0,pc_bytesize:0}
    }
}
impl<'a> Sim<'a> {

    pub fn init(&mut self, project: &mut Project, atdf:&'static AvrDeviceFile,memory: &mut Memory) -> Result<()>
    {
        
        let eeprom = project.get_eeprom_data()?;
        let inst = project.get_instruction_list()?;
        self.memory = unsafe { &mut *(memory as *mut Memory) };
        self.init_iner(atdf, inst, eeprom)?;
        Ok(())
    }
    pub fn init_iner(&mut self, atdf: &'static AvrDeviceFile, inst: Vec<Instruction>, eeprom: Vec<u8>) -> Result<()> {
        let mut inst_vec: Vec<Instruction> = Vec::new();
        inst_vec.resize((atdf.devices.address_spaces.iter().find(|x| { x.id == "prog" }).unwrap().size / 2) as usize,
                        Instruction::new("".to_string(),CustomOpcodes::EMPTY as usize,vec![],0));
        inst.into_iter().map(|x| {
            let raw_inst = RawInst::get_inst_from_id(x.opcode_id)?;
            let address = x.address.clone();
            inst_vec.insert(x.address as usize, x);
            if raw_inst.len == 2 {
                inst_vec.insert((address as usize) + 1, Instruction::decode_from_opcode(CustomOpcodes::REMINDER as u16)?);
            }
            Ok(())
        }).collect::<Result<()>>()?;
        self.memory.init(atdf, inst_vec, eeprom)?;
        self.registers = *(get_common_registers(&*atdf.devices.name.to_lowercase()).ok_or(anyhow!("mcu not supported"))?);
        self.registers.init_regs(atdf, &mut self.memory.data.io.inner)?;
        let pc_size =atdf.devices.address_spaces.iter().find(|x1| {x1.name=="prog"}).ok_or(anyhow!("invalid pc size"))?.size as u32;
        if pc_size != 0 {
            self.pc_len = pc_size.ilog2()-1;//divide by 2 because instructions are 16bit
        }else {
            Err(anyhow!("pc_size ==0"))?;
        }
        if self.pc_len>=8 && self.pc_len<=15{
            self.pc_bytesize = 2;
        }else if self.pc_len>=16 && self.pc_len<=17 {
            self.pc_bytesize = 3;
        }else{
            Err(anyhow!("pc_size =={}", self.pc_len))?;
        }

        Ok(())
    }
    pub fn init_debug(atdf: &'static AvrDeviceFile, flash: Vec<Instruction>) -> Result<Sim<'static>> {
        let mut s = Sim::default();
        s.init_iner(atdf, flash, vec![])?;
        Ok(s)
    }
    pub fn exec_debug(&mut self) -> Result<()> {
        unsafe {
            self.execute_inst()
        }
    }
    pub unsafe fn debug_init_stack(&mut self)->Result<()>{
        let ramlen = self.memory.data.ram.len();
        unsafe {
            self.registers.spL.set_data((ramlen & 0xff) as u8);
            self.registers.spH.set_data(((ramlen >> 8) & 0xff) as u8);
        }
        Ok(())
    }
    unsafe fn set_flag(&mut self,flags: Flags, value: bool){ unsafe { self.registers.set_flag(flags, value) }}
    unsafe fn get_flag(&mut self,flags: Flags)->bool { unsafe { self.registers.get_flag(flags) }}

    unsafe fn push(&mut self, data:u32,len:u32)->Result<()>{ unsafe {
        let mut sp: u16 = ((self.registers.spH.try_get().or(Some(0)).unwrap() as u16) << 8) + (self.registers.spL.get_data() as u16);
        //sp &= 2u16.pow(self.pc_len)-1;
        for i in 0..(len as u16) {
            *self.memory.data.get_mut(sp as usize - i as usize).ok_or(anyhow!("invalid ram offset: {}",sp))? = ((data >> (8 * i)) & 0xff) as u8;
        }
        sp -= len as u16;
        self.registers.spL.set_data((sp & 0xff) as u8);
        self.registers.spH.try_set(((sp >> 8) & 0xff) as u8);
        Ok(())
    }}
    unsafe fn pop(&mut self,len:u32)->Result<u32>{ unsafe {
        let mut sp: u16 = ((self.registers.spH.try_get().or(Some(0)).unwrap() as u16) << 8) + (self.registers.spL.get_data() as u16);
        //sp &= 2u16.pow(self.pc_len)-1;
        let mut data: u32 = 0;
        for i in 0..(len as u16) {
            data = data << 8;
            data += *self.memory.data.get((sp + i) as usize).ok_or(anyhow!("invalid ram offset: {}",sp))? as u32;
        }
        sp += len as u16;
        self.registers.spL.set_data((sp & 0xff) as u8);
        self.registers.spH.set_data(((sp >> 8) & 0xff) as u8);
        Ok(data)

    }}
    pub unsafe fn execute_inst(&mut self) -> Result<()> {
        unsafe {
            let instruction = self.memory.flash.get(self.memory.program_couter as usize).ok_or(anyhow!("cant access : {}",self.memory.program_couter))?.clone();
            //println!("{:?}",instruction);
            let op1 = match &instruction.operands {
                Some(o) => match o.get(0) {
                    Some(v) => v.value.clone(),
                    None => 0
                },
                None => 0
            };
            let op2 = match &instruction.operands {
                Some(o) => match o.get(1) {
                    Some(v) => v.value.clone(),
                    None => 0
                },
                None => 0
            };
            let op3 = match &instruction.operands {
                Some(o) => match o.get(2) {
                    Some(v) => v.value.clone(),
                    None => 0
                },
                None => 0
            };
            let ind1 = op1 as usize;
            let ind2 = op2 as usize;
            let ind3 = op3 as usize;
            let mut reg = self.memory.data.registers.clone();
            let reg_ptr = reg.as_mut_ptr();

            let (mut ra, mut rb) = unsafe {
                let ra = match ind1 < reg.len() {
                    true => Ok(&mut *reg_ptr.add(ind1)),
                    false => Err(anyhow!("invalid reg index")),
                };
                let rb = match ind2 < reg.len() {
                    true => Ok(&mut *reg_ptr.add(ind2)),
                    false => Err(anyhow!("invalid reg index")),
                };
                (ra, rb)
            };
            const fn get_bit(data: u8, bit: u8) -> bool { ((data >> bit) & 1) == 1 };
            let res = match instruction.get_raw_inst()?.name {
                Opcode::ADC => {
                    let val_ra: u8 = *ra?;
                    let val_rb: u8 = *rb?;
                    let (mut res, ov) = val_ra.overflowing_add(val_rb);
                    let ov1;
                    (res, ov1) = res.overflowing_add(self.get_flag(Flags::C) as u8);
                    let ra = val_ra;
                    let rb = val_rb;
                    self.set_flag(Flags::H, execute![(ra3&rb3)|(ra3&!res3)|(rb3&!res3)]?);
                    let v = execute![(ra7&rb7&!res7)|(!ra7&!rb7&res7)]?;
                    self.set_flag(Flags::V, v);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N, n);
                    self.set_flag(Flags::S, execute![v^n]?);
                    self.set_flag(Flags::Z, res == 0);
                    self.set_flag(Flags::C, ov | ov1);
                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::ADD => {
                    let ra = *ra?;
                    let rb = *rb?;
                    let (res, ov) = ra.overflowing_add(rb);

                    self.set_flag(Flags::H, execute![(ra3&rb3)|(ra3&!res3)|(rb3&!res3)]?);
                    let v = execute![(ra7&rb7&!res7)|(!ra7&!rb7&res7)]?;
                    self.set_flag(Flags::V, v);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N, n);
                    self.set_flag(Flags::S, execute![v^n]?);
                    self.set_flag(Flags::Z, res == 0);
                    self.set_flag(Flags::C, ov);
                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::ADIW => {
                    let data: u16 = ((reg[ind1+1] as u16)<<8) + (reg[ind1] as u16);
                    let (res, ov) = data.overflowing_add(op2 as u16);
                    let n = (res >> 15) == 1;
                    self.set_flag(Flags::N, n);
                    self.set_flag(Flags::Z, res == 0);
                    self.set_flag(Flags::C, ov);
                    let v = n&((data>>15) ==1);
                    self.set_flag(Flags::V, v);
                    self.set_flag(Flags::S, execute![n^v]?);

                    reg[ind1] = (res & 0xff) as u8;
                    reg[ind1 + 1] = ((res >> 8) & 0xff) as u8;
                    Ok(true)
                }
                Opcode::AND => {
                    let res = execute![ra&rb]?;

                    let n = execute![res7]?;
                    self.set_flag(Flags::N, n);
                    self.set_flag(Flags::Z, res == 0);
                    self.set_flag(Flags::S, n);
                    self.set_flag(Flags::V, false);

                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::ANDI => {
                    let data = op2 as u8;
                    let res = execute![ra&data]?;

                    let n = execute![res7]?;
                    self.set_flag(Flags::N, n);
                    self.set_flag(Flags::Z, res == 0);
                    self.set_flag(Flags::S, n);
                    self.set_flag(Flags::V, false);

                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::ASR => {
                    let ra = *ra?;
                    let mut res = ra >> 1;
                    res = res | (ra & (1<<7));

                    let n = execute![res7]?;
                    self.set_flag(Flags::N, n);
                    self.set_flag(Flags::Z, res == 0);
                    let c = (res & 1) == 1;
                    self.set_flag(Flags::C, c);
                    let v = n ^ c;
                    self.set_flag(Flags::V, v);
                    self.set_flag(Flags::S, v ^ n);
                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::BCLR => {
                    let f = Flags::get_flag(op1 as u8)?;
                    self.set_flag(f, false);

                    Ok(true)
                }
                Opcode::BLD => {
                    reg[ind1] =match self.get_flag(Flags::T){
                        true => {*ra? | (1<<op2)}
                        false=>{*ra? & 0xff-(1<<op2)}
                    };
                    Ok(true)
                }
                Opcode::BRBC => {
                    let f = self.get_flag(Flags::get_flag(op1 as u8)?);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRBS => {
                    let f = self.get_flag(Flags::get_flag(op1 as u8)?);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRCC => {
                    let f = self.get_flag(Flags::C);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRCS => {
                    let f = self.get_flag(Flags::C);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BREAK => {
                    Err(anyhow!("halt"))
                }
                Opcode::BREQ => {
                    let f = self.get_flag(Flags::Z);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRGE => {
                    let f = self.get_flag(Flags::S);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRHC => {
                    let f = self.get_flag(Flags::H);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRHS => {
                    let f = self.get_flag(Flags::H);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRID => {
                    let f = self.get_flag(Flags::I);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRIE => {
                    let f = self.get_flag(Flags::I);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRLO => {
                    let f = self.get_flag(Flags::C);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRLT => {
                    let f = self.get_flag(Flags::S);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRMI => {
                    let f = self.get_flag(Flags::N);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRNE => {
                    let f = self.get_flag(Flags::Z);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRPL => {
                    let f = self.get_flag(Flags::N);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRSH => {
                    let f = self.get_flag(Flags::C);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRTC => {
                    let f = self.get_flag(Flags::T);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRTS => {
                    let f = self.get_flag(Flags::T);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRVC => {
                    let f = self.get_flag(Flags::V);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BRVS => {
                    let f = self.get_flag(Flags::V);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op1) as u32;
                    }

                    Ok(true)
                }
                Opcode::BSET => {
                    self.set_flag(Flags::get_flag(op1 as u8)?, true);
                    Ok(true)
                }
                Opcode::BST => {
                    let bit = (*ra? >> op2) == 1;
                    self.set_flag(Flags::T, bit);
                    Ok(true)
                }
                Opcode::CALL => {
                    self.push(self.memory.program_couter + 2, self.pc_bytesize);
                    self.memory.program_couter = op1 as u32;
                    Ok(false)
                }
                Opcode::CBI => {
                    self.memory.data.io[op1 as usize] &= 0xff - (1<<(op2 as u8));
                    Ok(true)
                }
                Opcode::CBR => {
                    let res = *ra? & (0xff - (op2 as u8));
                    self.set_flag(Flags::V, false);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N, n);
                    self.set_flag(Flags::S, n);
                    self.set_flag(Flags::Z, res == 0);

                    reg[op1 as usize] = res;
                    Ok(true)
                }
                Opcode::CLC => {
                    self.set_flag(Flags::C, false);
                    Ok(true)
                }
                Opcode::CLH => {
                    self.set_flag(Flags::H, false);
                    Ok(true)
                }
                Opcode::CLI => {
                    self.set_flag(Flags::I, false);
                    Ok(true)
                }
                Opcode::CLN => {
                    self.set_flag(Flags::N, false);
                    Ok(true)
                }
                Opcode::CLR => {
                    reg[op1 as usize] = 0;
                    self.set_flag(Flags::S, false);
                    self.set_flag(Flags::V, false);
                    self.set_flag(Flags::N, false);
                    self.set_flag(Flags::Z, true);
                    Ok(true)
                }
                Opcode::CLS => {
                    self.set_flag(Flags::S, false);
                    Ok(true)
                }
                Opcode::CLT => {
                    self.set_flag(Flags::T, false);
                    Ok(true)
                }
                Opcode::CLV => {
                    self.set_flag(Flags::V, false);
                    Ok(true)
                }
                Opcode::CLZ => {
                    self.set_flag(Flags::Z, false);
                    Ok(true)
                }
                Opcode::COM => {
                    let ra = *ra?;
                    let res = 0xffu8 - ra;
                    let n = execute![ra7]?;
                    self.set_flag(Flags::N, n);
                    self.set_flag(Flags::S, n);
                    self.set_flag(Flags::V, false);
                    self.set_flag(Flags::C, true);
                    self.set_flag(Flags::Z, res == 0);

                    reg[ind1] =res;
                    Ok(true)
                }
                Opcode::CP => {
                    let ra = *ra?;
                    let rb = *rb?;
                    let (res,ovr) = ra.overflowing_sub(rb);
                    self.set_flag(Flags::H, execute![(!ra3&rb3)|(rb3&res3)|(res3&!ra3)]?);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N, n);
                    let v = execute![(ra7&!rb7&!res7)|(!ra7&rb7&res7)]?;
                    self.set_flag(Flags::V, v);
                    self.set_flag(Flags::S, execute![n^v]?);
                    self.set_flag(Flags::Z, res == 0);
                    self.set_flag(Flags::C, ovr);
                    Ok(true)
                }
                Opcode::CPC => {
                    let ra = *ra?;
                    let rb = *rb?;
                    let (res,ovr) = ra.overflowing_sub(rb) ;
                    let (res,ovr1) = res.overflowing_sub(self.get_flag(Flags::C) as u8);
                    self.set_flag(Flags::H, execute![(!ra3&rb3)|(rb3&res3)|(res3&!ra3)]?);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N, n);
                    let v = execute![(ra7&!rb7&!res7)|(!ra7&rb7&res7)]?;
                    self.set_flag(Flags::V, v);
                    self.set_flag(Flags::S, execute![n^v]?);
                    if res != 0 {
                        self.set_flag(Flags::Z, false)
                    }
                    self.set_flag(Flags::C, ovr|ovr1);
                    Ok(true)
                }
                Opcode::CPI => {
                    let ra = *ra?;
                    let val = op2 as u8;
                    let (res,ovr) = ra.overflowing_sub(val);
                    self.set_flag(Flags::H, execute![(!ra3&val3)|(val3&res3)|(res3&!ra3)]?);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N, n);
                    let v = execute![(ra7&!val7&!res7)|(!ra7&val7&res7)]?;
                    self.set_flag(Flags::V, v);
                    self.set_flag(Flags::S, execute![n^v]?);
                    self.set_flag(Flags::Z, res == 0);
                    self.set_flag(Flags::C, ovr);
                    Ok(true)
                }
                Opcode::CPSE => {
                    if ra? == rb? {
                        if RawInst::get_inst_from_id(self.memory.flash[(self.memory.program_couter + 1) as usize].opcode_id)?.len == 2 {
                            self.memory.program_couter += 2
                        } else {
                            self.memory.program_couter += 1
                        }
                    }
                    Ok(true)
                }
                Opcode::CUSTOM_INST(_) => {
                    Err(anyhow!("this should not execute"))
                }
                Opcode::DEC => {
                    let res = *ra? - 1;
                    let v = res == 0x79;
                    self.set_flag(Flags::V, v);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N, n);
                    self.set_flag(Flags::S, v ^ n);
                    self.set_flag(Flags::Z, res == 0);
                    reg[op1 as usize] = res;
                    Ok(true)
                }
                Opcode::DES => {
                    Err(anyhow!("not implemented"))
                }
                Opcode::EICALL => {
                    match self.pc_bytesize {
                        2 => {
                            self.push(self.memory.program_couter + 1, 2);
                            self.memory.program_couter = (reg[30] as u32) + ((reg[31] as u32) << 8);
                            Ok(())
                        }
                        3 => {
                            self.push(self.memory.program_couter + 1, 3);
                            self.memory.program_couter = (reg[30] as u32) + ((reg[31] as u32) << 8) + ((self.registers.eind.get_data() as u32) << 16);
                            Ok(())
                        }
                        _=>{
                            Err(anyhow!("invalid pc_bytesize: {}",self.pc_bytesize))
                        }
                    }?;
                    Ok(false)
                }
                Opcode::EIJMP => {
                    match self.pc_bytesize {
                        2 => {
                            self.memory.program_couter = (reg[30] as u32) + ((reg[31] as u32) << 8);
                            Ok(())
                        }
                        3 => {
                            self.memory.program_couter = (reg[30] as u32) + ((reg[31] as u32) << 8) + ((self.registers.eind.get_data() as u32) << 16);
                            Ok(())
                        }
                        _=>{
                            Err(anyhow!("invalid pc_bytesize: {}",self.pc_bytesize))
                        }
                    }?;
                    Ok(false)
                }
                Opcode::ELPM => { //todo might have issues
                    let mut ptr = (reg[30] as u32) + ((reg[31] as u32) << 8) + ((self.registers.rampz.get_data() as u32) << 16) >> 1;
                    let data: u16 = self.memory.flash[(ptr>>1) as usize].raw_opcode as u16;

                    reg[op1 as usize] = (data >> (8 * (ptr & 1))) as u8;
                    if op2 != 0 {
                        ptr += 1;
                        reg[30] = (ptr & 0xff) as u8;
                        reg[31] = ((ptr >> 8) & 0xff) as u8;
                        self.registers.rampz.set_data(((ptr >> 16) & 0xff) as u8);
                    }
                    Ok(true)
                }
                Opcode::EOR => {
                    let res = execute![ra^rb]?;
                    let n = execute![res7]?;
                    self.set_flag(Flags::N, n);
                    self.set_flag(Flags::S, n);
                    self.set_flag(Flags::V, false);
                    self.set_flag(Flags::Z, res == 0);

                    reg[op1 as usize] = res;

                    Ok(true)
                }
                Opcode::FMUL => {
                    let mut res = (*ra? as u16) * (*rb? as u16);
                    let c = (res >> 15) == 1;
                    res <<= 1;
                    reg[1] = (res >> 8) as u8;
                    reg[0] = (res & 0xff) as u8;

                    self.set_flag(Flags::C, c);
                    self.set_flag(Flags::Z, res == 0);

                    Ok(true)
                }
                Opcode::FMULS => {
                    let mut res = (*ra? as i16) * (*rb? as i16);
                    let c = (res >> 15) == 1;
                    res <<= 1;
                    reg[1] = (res >> 8) as u8;
                    reg[0] = (res & 0xff) as u8;

                    self.set_flag(Flags::C, c);
                    self.set_flag(Flags::Z, res == 0);

                    Ok(true)
                }
                Opcode::FMULSU => {
                    let mut res = (*ra? as i16) * (*rb? as i16); //todo
                    let c = (res >> 15) == 1;
                    res <<= 1;
                    reg[1] = (res >> 8) as u8;
                    reg[0] = (res & 0xff) as u8;

                    self.set_flag(Flags::C, c);
                    self.set_flag(Flags::Z, res == 0);

                    Ok(true)
                }
                Opcode::ICALL => {
                    self.push(self.memory.program_couter + 1, self.pc_bytesize);
                    self.memory.program_couter = 0;
                    self.memory.program_couter = (reg[30] as u32) + ((reg[31] as u32) << 8);
                    Ok(false)
                }
                Opcode::IJMP => {
                    self.memory.program_couter = 0;
                    self.memory.program_couter = (reg[30] as u32) + ((reg[31] as u32) << 8);
                    Ok(false)
                }
                Opcode::IN => {
                    reg[ind1] = self.memory.data.io[ind2];
                    Ok(true)
                }
                Opcode::INC => {
                    let res = *ra? + 1;
                    let v = res == 0x80;
                    self.set_flag(Flags::V, v);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N, n);
                    self.set_flag(Flags::S, v ^ n);
                    self.set_flag(Flags::Z, res == 0);
                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::JMP => {
                    self.memory.program_couter = ind1 as u32;
                    Ok(false)
                }
                Opcode::LAC => {
                    let ptr = (reg[30] as u16) + ((reg[31] as u16) << 8);
                    let tmp = self.memory.data.ram[ptr as usize];
                    self.memory.data.ram[ptr as usize] &= 0xff - *ra?;
                    reg[ind1] = tmp;
                    Ok(true)
                }
                Opcode::LAS => {
                    let ptr = (reg[30] as u16) + ((reg[31] as u16) << 8);
                    let tmp = self.memory.data.ram[ptr as usize];
                    self.memory.data.ram[ptr as usize] |= *ra?;
                    reg[ind1] = tmp;
                    Ok(true)
                }
                Opcode::LAT => {
                    let ptr = (reg[30] as u16) + ((reg[31] as u16) << 8);
                    let tmp = self.memory.data.ram[ptr as usize];
                    self.memory.data.ram[ptr as usize] = !self.memory.data.ram[ptr as usize] & *ra?;
                    reg[ind1] = tmp;
                    Ok(true)
                }
                Opcode::LD => {
                    let mut ptr = match op2 {
                        3 => { //x

                            Ok((reg[26] as u32) + ((reg[27] as u32) << 8)) //+ ((self.registers.rampx.get_data() as u32) << 16))
                        }
                        2 => { //y
                            Ok((reg[28] as u32) + ((reg[29] as u32) << 8)) //+ ((self.registers.rampy.get_data() as u32) << 16))
                        }
                        0 => { //z
                            Ok((reg[30] as u32) + ((reg[31] as u32) << 8)) //+ ((self.registers.rampz.get_data() as u32) << 16))
                        }
                        _ => {
                            Err(anyhow!("invalid opcode"))
                        }
                    }?;
                    if op3 ==2{
                        ptr -=1;
                    }

                    reg[ind1] =self.memory.data[ptr as usize];

                    if op3 ==1{
                        ptr +=1;
                    }
                    match op2 {
                        3 => { //x
                            reg[26] = (ptr &0xff) as u8;
                            reg[27] = ((ptr &0xff)>>8) as u8;
                            //self.registers.rampx.set_data(((ptr &0xff)>>16) as u8);
                            Ok(())
                        }
                        2 => { //y
                            reg[26] = (ptr &0xff) as u8;
                            reg[27] = ((ptr &0xff)>>8) as u8;
                            //self.registers.rampy.set_data(((ptr &0xff)>>16) as u8);
                            Ok(())
                        }
                        0 => { //z
                            reg[26] = (ptr &0xff) as u8;
                            reg[27] = ((ptr &0xff)>>8) as u8;
                            //self.registers.rampz.set_data(((ptr &0xff)>>16) as u8);
                            Ok(())
                        }
                        _ => {
                            Err(anyhow!("invalid opcode"))
                        }
                    }?;
                    Ok(true)
                }
                Opcode::LDD => {
                    let ptr = match op2 {
                        1 => { //y
                            Ok((reg[28] as u32) + ((reg[29] as u32) << 8)+ ((self.registers.rampy.try_get().or(Some(0)).unwrap() as u32) << 16)+op3 as u32)
                        }
                        0 => { //z
                            Ok((reg[30] as u32) + ((reg[31] as u32) << 8)+ ((self.registers.rampy.try_get().or(Some(0)).unwrap() as u32) << 16)+op3 as u32)
                        }
                        _ => {
                            Err(anyhow!("invalid opcode"))
                        }
                    }?;
                    reg[ind1] =self.memory.data[ptr as usize];
                    Ok(true)
                }
                Opcode::LDI => {
                    reg[ind1] = op2 as u8;
                    Ok(true)
                }
                Opcode::LDS => {
                    reg[ind1] = self.memory.data[ind2];
                    Ok(true)
                }
                Opcode::LPM => { //todo might have issues
                    let mut ptr = (reg[30] as u16) + ((reg[31] as u16) << 8);
                    let data: u16 = self.memory.flash[(ptr>>1) as usize].raw_opcode as u16;

                    reg[op1 as usize] = (data >> (8 * (ptr & 1))) as u8;
                    if op2 != 0 {
                        ptr += 1;
                        reg[30] = (ptr & 0xff) as u8;
                        reg[31] = ((ptr >> 8) & 0xff) as u8;
                    }
                    Ok(true)
                }
                Opcode::LSL => {
                    let ra = *ra?;
                    let res = ra<<1;

                    self.set_flag(Flags::H,execute![ra3]?);
                    let c = execute![ra7]?;
                    self.set_flag(Flags::C,c);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N,n);
                    let v = n^c;
                    self.set_flag(Flags::V,v);
                    self.set_flag(Flags::S,n^v);
                    self.set_flag(Flags::Z,res==0);

                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::LSR => {
                    let ra = *ra?;
                    let res = ra>>1;

                    let c = execute![ra0]?;
                    self.set_flag(Flags::C,c);
                    self.set_flag(Flags::N,false);
                    self.set_flag(Flags::V,c);
                    self.set_flag(Flags::S,c);
                    self.set_flag(Flags::Z,res==0);

                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::MOV => {
                    reg[ind1] = *rb?;
                    Ok(true)
                }
                Opcode::MOVW => {
                    reg[ind1] = *rb?;
                    reg[ind1+1] = reg[ind2+1];
                    Ok(true)
                }
                Opcode::MUL => {
                    let ra = *ra? as u16;
                    let rb = *rb? as u16;
                    let res = ra*rb;

                    self.set_flag(Flags::C,(rb>>15) ==1);
                    self.set_flag(Flags::Z,rb ==0);
                    reg[0] = (res &0xff) as u8;
                    reg[1] = ((res &0xff)>>8) as u8;
                    Ok(true)
                }
                Opcode::MULS => {
                    let ra = *ra? as i16;
                    let rb = *rb? as i16;
                    let res = ra*rb;

                    self.set_flag(Flags::C,(rb>>15) ==1);
                    self.set_flag(Flags::Z,rb ==0);
                    reg[0] = (res &0xff) as u8;
                    reg[1] = ((res &0xff)>>8) as u8;
                    Ok(true)
                }
                Opcode::MULSU => {
                    let ra = *ra? as i16;
                    let rb = *rb? as i16;
                    let res = ra*rb;

                    self.set_flag(Flags::C,(rb>>15) ==1);
                    self.set_flag(Flags::Z,rb ==0);
                    reg[0] = (res &0xff) as u8;
                    reg[1] = ((res &0xff)>>8) as u8;
                    Ok(true)
                }
                Opcode::NEG => {
                    let ra = *ra? as i8;
                    let ra_u = ra as u8;
                    let res = (0-ra) as u8;
                    self.set_flag(Flags::C,ra!=0);
                    self.set_flag(Flags::Z,ra==0);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N,n);
                    let v =res==0x80;
                    self.set_flag(Flags::V,v);
                    self.set_flag(Flags::S,v^n);
                    self.set_flag(Flags::H,execute![ra_u3 | res3]?);
                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::NOP => {
                    Ok(true)
                }
                Opcode::OR => {
                    let res = *ra? |*rb?;
                    self.set_flag(Flags::V,false);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N,n);
                    self.set_flag(Flags::S,n);
                    self.set_flag(Flags::Z,res==0);
                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::ORI => {
                    let res = *ra? |(op2 as u8);
                    self.set_flag(Flags::V,false);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N,n);
                    self.set_flag(Flags::S,n);
                    self.set_flag(Flags::Z,res==0);
                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::OUT => {
                    self.memory.data.io[ind1] = *rb?;
                    Ok(true)
                }
                Opcode::POP => {
                    reg[ind1] = self.pop(1)? as u8;
                    Ok(true)
                }
                Opcode::PUSH => {
                    self.push(*ra? as u32,1)?;
                    Ok(true)
                }
                Opcode::RCALL => {
                    self.push(self.memory.program_couter + 1, self.pc_bytesize.clone().into())?;
                    if op1 >=0{
                        self.memory.program_couter += op1 as u32 +1;
                    }else{
                        self.memory.program_couter -= op1 as u32 +1;
                    }
                    Ok(false)
                }
                Opcode::RET => {
                    self.memory.program_couter = self.pop(self.pc_bytesize)?;
                    Ok(false)
                }
                Opcode::RETI => {
                    self.memory.program_couter = self.pop(self.pc_bytesize)?;
                    self.set_flag(Flags::I,true);
                    Ok(false)
                }
                Opcode::RJMP => {
                    if op1 >=0{
                        self.memory.program_couter += op1 as u32;
                    }else{
                        self.memory.program_couter -= (-op1) as u32;
                    }
                    Ok(true)
                }
                Opcode::ROL => {
                    let ra = *ra?;
                    let mut res = ra<<1;
                    if self.get_flag(Flags::C) {
                        res+=1;
                    }
                    self.set_flag(Flags::Z,res==0);
                    self.set_flag(Flags::H,execute![ra3]?);
                    let c = execute![ra7]?;
                    self.set_flag(Flags::C,c);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N,n);
                    let v =c^n;
                    self.set_flag(Flags::V,v);
                    self.set_flag(Flags::S,n^v);

                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::ROR => {
                    let ra = *ra?;
                    let mut res = ra>>1;
                    if self.get_flag(Flags::C) {
                        res+=1<<7;
                    }
                    self.set_flag(Flags::Z,res==0);
                    let c = execute![ra0]?;
                    self.set_flag(Flags::C,c);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N,n);
                    let v =c^n;
                    self.set_flag(Flags::V,v);
                    self.set_flag(Flags::S,n^v);

                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::SBC => {
                    let ra = *ra?;
                    let rb = *rb?;
                    let mut res;
                    let ovr1;
                    let ovr2;
                    (res,ovr1) = ra.overflowing_sub(rb);
                    (res,ovr2) = res.overflowing_sub(self.get_flag(Flags::C) as u8);

                    if res !=0 {
                        self.set_flag(Flags::Z,true);
                    }
                    self.set_flag(Flags::C,ovr1|ovr2);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N,n);
                    let v =execute![ra7&!rb7&!res7 | !ra7&rb7&res7]?;
                    self.set_flag(Flags::V,v);
                    self.set_flag(Flags::S,v^n);
                    self.set_flag(Flags::H,execute![!ra3&rb3|rb3&res3|res3&!ra3]?);
                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::SBCI => {
                    let ra = *ra?;
                    let rb = op2 as u8;
                    let mut res;
                    let ovr1;
                    let ovr2;
                    (res,ovr1) = ra.overflowing_sub(rb);
                    (res,ovr2) = res.overflowing_sub(self.get_flag(Flags::C) as u8);

                    if res !=0 {
                        self.set_flag(Flags::Z,true);
                    }
                    self.set_flag(Flags::C,ovr1|ovr2);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N,n);
                    let v =execute![ra7&!rb7&!res7 | !ra7&rb7&res7]?;
                    self.set_flag(Flags::V,v);
                    self.set_flag(Flags::S,v^n);
                    self.set_flag(Flags::H,execute![!ra3&rb3|rb3&res3|res3&!ra3]?);
                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::SBI => {
                    self.memory.data.io[ind1] |=1<<op2;

                    Ok(true)
                }
                Opcode::SBIC => {
                    if((self.memory.data.io[ind1]>>op2)&1 )==0{
                        self.memory.program_couter+=self.memory.flash[(self.memory.program_couter+1) as usize].get_raw_inst()?.len as u32;

                    }
                    Ok(true)
                }
                Opcode::SBIS => {
                    if((self.memory.data.io[ind1]>>op2)&1 )==1{
                        self.memory.program_couter+=self.memory.flash[(self.memory.program_couter+1) as usize].get_raw_inst()?.len as u32;
                    }
                    Ok(true)
                }
                Opcode::SBIW => {
                    let data = ((reg[ind1+1] as u16)<<8)+reg[ind1] as u16;
                    let mut res;
                    let ovr1;
                    (res,ovr1) = data.overflowing_sub(op2 as u16);

                    self.set_flag(Flags::Z,res ==0);
                    self.set_flag(Flags::C,ovr1);
                    let n = (res>>15)==1;
                    self.set_flag(Flags::N,n);
                    let v =!ovr1;
                    self.set_flag(Flags::S,v^n);
                    self.set_flag(Flags::V,v);
                    reg[ind1+1] = (res>>8) as u8;
                    reg[ind1] = (res&0xff) as u8;
                    Ok(true)
                }
                Opcode::SBR => {
                    let res = *ra? | op2 as u8;
                    self.set_flag(Flags::Z,res ==0);
                    let n = (res>>7)==1;
                    self.set_flag(Flags::N,n);
                    self.set_flag(Flags::S,n);
                    self.set_flag(Flags::V,false);
                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::SBRC => {
                    if((*ra?>>op2)&1 )==0{
                        self.memory.program_couter+=self.memory.flash[(self.memory.program_couter+1) as usize].get_raw_inst()?.len as u32;
                    }
                    Ok(true)
                }
                Opcode::SBRS => {
                    if((*ra?>>op2)&1 )==1{
                        self.memory.program_couter+=self.memory.flash[(self.memory.program_couter+1) as usize].get_raw_inst()?.len as u32;
                    }
                    Ok(true)
                }
                Opcode::SEC => {
                    self.set_flag(Flags::C,true);
                    Ok(true)
                }
                Opcode::SEH => {
                    self.set_flag(Flags::H,true);
                    Ok(true)
                }
                Opcode::SEI => {
                    self.set_flag(Flags::I,true);
                    Ok(true)
                }
                Opcode::SEN => {
                    self.set_flag(Flags::N,true);
                    Ok(true)
                }
                Opcode::SER => {
                    reg[ind1] = 0xff;
                    Ok(true)
                }
                Opcode::SES => {
                    self.set_flag(Flags::S,true);
                    Ok(true)
                }
                Opcode::SET => {
                    self.set_flag(Flags::T,true);
                    Ok(true)
                }
                Opcode::SEV => {
                    self.set_flag(Flags::V,true);
                    Ok(true)
                }
                Opcode::SEZ => {
                    self.set_flag(Flags::Z,true);
                    Ok(true)
                }
                Opcode::SLEEP => {
                    todo!();
                    Ok(true)
                }
                Opcode::SPM => {
                    let ptr = (reg[30] as u32) + ((reg[31] as u32) << 8) + ((self.registers.rampz.get_data() as u32) << 16);
                    let data =reg[0] as u16 + (reg[1] as u16)<<8;
                    self.memory.flash[ptr as usize] = Instruction::decode_from_opcode(data)?;
                    Ok(true)
                }
                Opcode::ST => {
                    let mut ptr = match op1 {
                        3 => { //x

                            Ok((reg[26] as u32) + ((reg[27] as u32) << 8)+((self.registers.rampx.try_get().or(Some(0)).unwrap() as u32) << 16))
                        }
                        2 => { //y
                            Ok((reg[28] as u32) + ((reg[29] as u32) << 8)+((self.registers.rampy.try_get().or(Some(0)).unwrap() as u32) << 16))
                        }
                        0 => { //z
                            Ok((reg[30] as u32) + ((reg[31] as u32) << 8)+((self.registers.rampz.try_get().or(Some(0)).unwrap() as u32) << 16))
                        }
                        x => {
                            Err(anyhow!("invalid opcode:{}",x))
                        }
                    }?;
                    if op2 ==2{
                        ptr -=1;
                    }

                    self.memory.data[ptr as usize] = reg[ind3];

                    if op2 ==1{
                        ptr +=1;
                    }
                    match op1 {
                        3 => { //x
                            reg[26] = (ptr &0xff) as u8;
                            reg[27] = ((ptr &0xff)>>8) as u8;
                            self.registers.rampx.try_set(((ptr &0xff)>>16) as u8);
                            Ok(())
                        }
                        2 => { //y
                            reg[26] = (ptr &0xff) as u8;
                            reg[27] = ((ptr &0xff)>>8) as u8;
                            self.registers.rampx.try_set(((ptr &0xff)>>16) as u8);
                            Ok(())
                        }
                        0 => { //z
                            reg[26] = (ptr &0xff) as u8;
                            reg[27] = ((ptr &0xff)>>8) as u8;
                            self.registers.rampx.try_set(((ptr &0xff)>>16) as u8);
                            Ok(())
                        }
                        _ => {
                            Err(anyhow!("invalid opcode"))
                        }
                    }?;
                    Ok(true)
                }
                Opcode::STD => {
                    let ptr = match op1 {
                        1 => { //y
                            Ok((reg[28] as u32) + ((reg[29] as u32) << 8) + ((self.registers.rampy.try_get().or(Some(0)).unwrap() as u32) << 16)+op2 as u32)
                        }
                        0 => { //z
                            Ok((reg[30] as u32) + ((reg[31] as u32) << 8) + ((self.registers.rampz.try_get().or(Some(0)).unwrap() as u32) << 16)+op2 as u32)
                        }
                        x => {
                            Err(anyhow!("invalid opcode {}",x))
                        }
                    }?;
                    self.memory.data[ptr as usize] = reg[ind3];
                    Ok(true)
                }
                Opcode::STS => {
                    self.memory.data[ind1]=*rb?;
                    Ok(true)
                }
                Opcode::SUB => {
                    let ra = *ra?;
                    let rb = *rb?;
                    let mut res;
                    let ovr1;
                    (res,ovr1) = ra.overflowing_sub(rb);

                    self.set_flag(Flags::Z,res ==0);
                    self.set_flag(Flags::C,ovr1);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N,n);
                    let v =execute![ra7&!rb7&!res7 | !ra7&rb7&res7]?;
                    self.set_flag(Flags::V,v);
                    self.set_flag(Flags::S,v^n);
                    self.set_flag(Flags::H,execute![!ra3&rb3|rb3&res3|res3&!ra3]?);
                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::SUBI => {
                    let ra = *ra?;
                    let rb = op2 as u8;
                    let mut res;
                    let ovr1;
                    (res,ovr1) = ra.overflowing_sub(rb);

                    self.set_flag(Flags::Z,res ==0);
                    self.set_flag(Flags::C,ovr1);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N,n);
                    let v =execute![ra7&!rb7&!res7 | !ra7&rb7&res7]?;
                    self.set_flag(Flags::V,v);
                    self.set_flag(Flags::S,v^n);
                    self.set_flag(Flags::H,execute![!ra3&rb3|rb3&res3|res3&!ra3]?);
                    reg[ind1] = res;
                    Ok(true)
                }
                Opcode::SWAP => {
                    let ra = *ra?;
                    reg[ind1] = (ra & 0x0F)<<4 | (ra & 0xF0)>>4;
                    Ok(true)
                }
                Opcode::TST => {
                    let ra = *ra?;
                    self.set_flag(Flags::Z,ra == 0);
                    let n = execute![ra7]?;
                    self.set_flag(Flags::N,n);
                    self.set_flag(Flags::V,false);
                    self.set_flag(Flags::S,n);
                    Ok(true)
                }
                Opcode::WDR => {
                    todo!();
                    Ok(true)
                }
                Opcode::XCH => {
                    let ptr = reg[30] as u16+(reg[31] as u16)<<8;
                    let data = self.memory.data[ptr as usize];
                    self.memory.data[ptr as usize] = *ra?;
                    reg[ind1] = data;
                    Ok(true)
                }
            }?;
            if res{
                self.memory.program_couter += (instruction.get_raw_inst()?.len *2) as u32;
            }
            self.memory.data.registers = reg;
            Ok(())
        }
    }


}

#[allow(unused_mut)]
#[cfg(test)]
mod opcode_tests {
    use super::*;
    use crate::sim::operand::Operand;
    use opcode_gen::RawInst;
    use std::env;
    use device_parser::get_tree_map;

    // 1. Define the helper macro
    // 1. Define the helper macro
    macro_rules! test_opcodes {
        (
            $(
                $test_name:ident: $opcode:ident $( ( $($op:expr),* ) )? {
                    setup: $setup:expr,
                    check: $check:expr
                }
            ),* $(,)?
        ) => {
            // 1. Generate the actual Unit Tests
            $(
                #[test]
                fn $test_name() -> Result<()> {
                    unsafe { env::set_var("RUST_BACKTRACE", "1"); }
                    fn get_tree() -> &'static AvrDeviceFile {
                        get_tree_map().get("atmega2560").expect("mcu not found")
                    }

                    let mut operands = Vec::new();
                    $(
                        $(
                            let mut operand = Operand::default();
                            operand.value = $op;
                            operands.push(operand);
                        )*
                    )?

                    let inst_id = RawInst::get_inst_id_from_opcode(Opcode::$opcode)
                        .ok_or(anyhow::anyhow!("Opcode ID not found"))?;

                    let inst = Instruction::new("".to_string(), inst_id, operands,0);

                    // Safety NOP for skipping instructions
                    let nop_id = RawInst::get_inst_id_from_opcode(Opcode::NOP).unwrap();
                    let nop = Instruction::new("".to_string(), nop_id, vec![],inst.get_raw_inst()?.len as u32);

                    let mut s = Sim::init_debug(get_tree(), vec![inst, nop])?;
                    unsafe{s.debug_init_stack()?};
                    let setup_fn: fn(&mut Sim) = $setup;
                    setup_fn(&mut s);
                    s.exec_debug()?;

                    let check_fn: fn(&mut Sim) = $check;
                    check_fn(&mut s);

                    Ok(())
                }
            )*

            // 2. The Exhaustiveness Check
            #[allow(dead_code, unreachable_patterns)]
            fn verify_opcode_coverage(op: Opcode) {
                match op {
                    // Manually exempt CUSTOM_INST from the loop
                    Opcode::CUSTOM_INST(_) => {},

                    // Check everything else
                    $(
                        Opcode::$opcode => {},
                    )*
                }
            }
        }
    }

    test_opcodes! {
        // =============================================================
        // ARITHMETIC INSTRUCTIONS
        // =============================================================
        add: ADD(0, 1) {
            setup: |s| { s.memory.data.registers[0] = 10; s.memory.data.registers[1] = 20; },
            check: |s| { assert_eq!(s.memory.data.registers[0], 30); }
        },
        adc: ADC(0, 1) {
            setup: |s| {
                s.memory.data.registers[0] = 10; s.memory.data.registers[1] = 20;
                unsafe { s.set_flag(Flags::C, true); }
            },
            check: |s| { assert_eq!(s.memory.data.registers[0], 31); }
        },
        adiw: ADIW(24, 10) {
            setup: |s| { s.memory.data.registers[24] = 0xFE;s.memory.data.registers[25] = 0x40},
            check: |s| { assert_eq!(((s.memory.data.registers[25] as u16)<<8) +s.memory.data.registers[24] as u16, 0x4108);}
        },
        sub: SUB(16, 17) {
            setup: |s| { s.memory.data.registers[16] = 10; s.memory.data.registers[17] = 3; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 7); }
        },
        subi: SUBI(16, 5) {
            setup: |s| { s.memory.data.registers[16] = 10; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 5); }
        },
        sbc: SBC(16, 17) {
            setup: |s| {
                s.memory.data.registers[16] = 10; s.memory.data.registers[17] = 3;
                unsafe { s.set_flag(Flags::C, true); }
            },
            check: |s| { assert_eq!(s.memory.data.registers[16], 6); }
        },
        sbci: SBCI(16, 5) {
            setup: |s| {
                s.memory.data.registers[16] = 10;
                unsafe { s.set_flag(Flags::C, true); }
            },
            check: |s| { assert_eq!(s.memory.data.registers[16], 4); }
        },
        sbiw: SBIW(24, 1) {
            setup: |s| { s.memory.data.registers[24] = 0x00; s.memory.data.registers[25] = 0x01; }, // Value 256
            check: |s| { assert_eq!(s.memory.data.registers[24], 0xFF); assert_eq!(s.memory.data.registers[25], 0x00); }
        },
        and: AND(16, 17) {
            setup: |s| { s.memory.data.registers[16] = 0xFF; s.memory.data.registers[17] = 0x0F; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 0x0F); }
        },
        andi: ANDI(16, 0x0F) {
            setup: |s| { s.memory.data.registers[16] = 0xFF; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 0x0F); }
        },
        or: OR(16, 17) {
            setup: |s| { s.memory.data.registers[16] = 0xF0; s.memory.data.registers[17] = 0x0F; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 0xFF); }
        },
        ori: ORI(16, 0x0F) {
            setup: |s| { s.memory.data.registers[16] = 0xF0; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 0xFF); }
        },
        eor: EOR(16, 17) {
            setup: |s| { s.memory.data.registers[16] = 0xFF; s.memory.data.registers[17] = 0x0F; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 0xF0); }
        },
        com: COM(16) {
            setup: |s| { s.memory.data.registers[16] = 0x00; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 0xFF); }
        },
        neg: NEG(16) {
            setup: |s| { s.memory.data.registers[16] = 1; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 0xFF); } // -1 in two's complement
        },
        inc: INC(16) {
            setup: |s| { s.memory.data.registers[16] = 10; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 11); }
        },
        dec: DEC(16) {
            setup: |s| { s.memory.data.registers[16] = 10; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 9); }
        },
        mul: MUL(16, 17) {
            setup: |s| { s.memory.data.registers[16] = 2; s.memory.data.registers[17] = 3; },
            check: |s| { assert_eq!(s.memory.data.registers[0], 6); }
        },
        muls: MULS(16, 17) {
            setup: |s| { s.memory.data.registers[16] = 0xFF; s.memory.data.registers[17] = 2; }, // -1 * 2
            check: |s| { assert_eq!(s.memory.data.registers[0], 0xFE); } // -2
        },
        mulsu: MULSU(16, 17) {
            setup: |s| { s.memory.data.registers[16] = 0xFF; s.memory.data.registers[17] = 2; },
            check: |s| { /* Logic Check needed for Signed * Unsigned */ }
        },
        fmul: FMUL(16, 17) {
            setup: |s| { s.memory.data.registers[16] = 0x80; s.memory.data.registers[17] = 0x80; }, // 0.5 * 0.5
            check: |s| { assert_eq!(s.memory.data.registers[1], 0x80); } // 0.25 (shifted)
        },
        fmuls: FMULS(16, 17) {
            setup: |s| { s.memory.data.registers[16] = 0x80; s.memory.data.registers[17] = 0x80; },
            check: |s| {}
        },
        fmulsu: FMULSU(16, 17) {
            setup: |s| {}, check: |s| {}
        },

        // =============================================================
        // BRANCHING INSTRUCTIONS
        // =============================================================
        rjmp: RJMP(10) {
            setup: |s| {},
            check: |s| { /* PC Relative Jump Logic */ }
        },
        jmp: JMP(0x0100) {
            setup: |s| {},
            check: |s| { assert_eq!(s.memory.program_couter, 0x0100); }
        },
        ijmp: IJMP {
            setup: |s| { s.memory.data.registers[30] = 0x00; s.memory.data.registers[31] = 0x01; },
            check: |s| { assert_eq!(s.memory.program_couter, 0x0100); }
        },
        eijmp: EIJMP {
            setup: |s| {}, check: |s| {} // Size24 logic
        },
        rcall: RCALL(10) {
            setup: |s| {}, check: |s| {}
        },
        call: CALL(0x0100) {
            setup: |s| {},
            check: |s| { assert_eq!(s.memory.program_couter, 0x0100); }
        },
        icall: ICALL {
            setup: |s| { s.memory.data.registers[30] = 0x00; s.memory.data.registers[31] = 0x01; },
            check: |s| { assert_eq!(s.memory.program_couter, 0x0100); }
        },
        eicall: EICALL {
            setup: |s| {}, check: |s| {}
        },
        ret: RET {
            setup: |s| {}, check: |s| {} // Stack pop logic
        },
        reti: RETI {
            setup: |s| {}, check: |s| {} // Stack pop + SEI
        },
        cpse: CPSE(16, 17) {
            setup: |s| { s.memory.data.registers[16] = 5; s.memory.data.registers[17] = 5; },
            check: |s| { assert_eq!(s.memory.program_couter, 2); } // Skips next
        },
        sbrc: SBRC(16, 0) {
            setup: |s| { s.memory.data.registers[16] = 0x00; }, // Bit 0 is 0
            check: |s| { /* Should skip next */ }
        },
        sbrs: SBRS(16, 0) {
            setup: |s| { s.memory.data.registers[16] = 0x01; }, // Bit 0 is 1
            check: |s| { /* Should skip next */ }
        },
        sbic: SBIC(0x10, 0) {
            setup: |s| {}, check: |s| {} // IO Skip
        },
        sbis: SBIS(0x10, 0) {
            setup: |s| {}, check: |s| {} // IO Skip
        },
        brbs: BRBS(1, 10) { // Branch if Z set
            setup: |s| { unsafe { s.set_flag(Flags::Z, true); } },
            check: |s| { /* Check PC offset */ }
        },
        brbc: BRBC(1, 10) { // Branch if Z clear
            setup: |s| { unsafe { s.set_flag(Flags::Z, false); } },
            check: |s| { /* Check PC offset */ }
        },
        breq: BREQ(10) {
            setup: |s| { unsafe { s.set_flag(Flags::Z, true); } },
            check: |s| {}
        },
        brne: BRNE(10) {
            setup: |s| { unsafe { s.set_flag(Flags::Z, false); } },
            check: |s| {}
        },
        brcs: BRCS(10) {
            setup: |s| { unsafe { s.set_flag(Flags::C, true); } },
            check: |s| {}
        },
        brcc: BRCC(10) {
            setup: |s| { unsafe { s.set_flag(Flags::C, false); } },
            check: |s| {}
        },
        brsh: BRSH(10) { setup: |s| {}, check: |s| {} },
        brlo: BRLO(10) { setup: |s| {}, check: |s| {} },
        brmi: BRMI(10) { setup: |s| {}, check: |s| {} },
        brpl: BRPL(10) { setup: |s| {}, check: |s| {} },
        brge: BRGE(10) { setup: |s| {}, check: |s| {} },
        brlt: BRLT(10) { setup: |s| {}, check: |s| {} },
        brhs: BRHS(10) { setup: |s| {}, check: |s| {} },
        brhc: BRHC(10) { setup: |s| {}, check: |s| {} },
        brts: BRTS(10) { setup: |s| {}, check: |s| {} },
        brtc: BRTC(10) { setup: |s| {}, check: |s| {} },
        brvs: BRVS(10) { setup: |s| {}, check: |s| {} },
        brvc: BRVC(10) { setup: |s| {}, check: |s| {} },
        brie: BRIE(10) { setup: |s| {}, check: |s| {} },
        brid: BRID(10) { setup: |s| {}, check: |s| {} },

        // =============================================================
        // DATA TRANSFER
        // =============================================================
        mov: MOV(16, 17) {
            setup: |s| { s.memory.data.registers[17] = 0xAA; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 0xAA); }
        },
        movw: MOVW(16, 18) {
            setup: |s| { s.memory.data.registers[18] = 0xAA; s.memory.data.registers[19] = 0xBB; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 0xAA); assert_eq!(s.memory.data.registers[17], 0xBB); }
        },
        ldi: LDI(16, 0xFF) {
            setup: |s| {},
            check: |s| { assert_eq!(s.memory.data.registers[16], 0xFF); }
        },
        ld: LD(16, 0, 3) { // Reg, 0, Mode 3 (X pointer) - Args depend on implementation
            setup: |s| {}, check: |s| {}
        },
        ldd: LDD(16, 0, 0) { setup: |s| {}, check: |s| {} },
        lds: LDS(16, 0x0100) { setup: |s| {}, check: |s| {} },
        st: ST(0, 1, 16) { setup: |s| {}, check: |s| {} },
        std: STD(0, 0x80, 16) { setup: |s| {}, check: |s| {} },
        sts: STS(0x0100, 16) { setup: |s| {}, check: |s| {} },
        lpm: LPM(16, 0) { setup: |s| {}, check: |s| {} },
        elpm: ELPM(16, 0) { setup: |s| {}, check: |s| {} },
        spm: SPM { setup: |s| {}, check: |s| {} },
        in_io: IN(0, 0x3F) {
            setup: |s| { s.memory.data.io[0x3F] = 0x55; },
            check: |s| { assert_eq!(s.memory.data.registers[0], 0x55); }
        },
        out_io: OUT(0x3F, 0) {
            setup: |s| { s.memory.data.registers[0] = 0x55; },
            check: |s| { assert_eq!(s.memory.data.io[0x3F], 0x55); }
        },
        push: PUSH(0) { setup: |s| {}, check: |s| {} },
        pop: POP(0) { setup: |s| {}, check: |s| {} },
        xch: XCH(0, 16) { setup: |s| {}, check: |s| {} },
        las: LAS(0, 16) { setup: |s| {}, check: |s| {} },
        lac: LAC(0, 16) { setup: |s| {}, check: |s| {} },
        lat: LAT(0, 16) { setup: |s| {}, check: |s| {} },

        // =============================================================
        // BIT & SHIFT
        // =============================================================
        lsl: LSL(16) {
            setup: |s| { s.memory.data.registers[16] = 0x01; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 0x02); }
        },
        lsr: LSR(16) {
            setup: |s| { s.memory.data.registers[16] = 0x02; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 0x01); }
        },
        rol: ROL(16) {
            setup: |s| { s.memory.data.registers[16] = 0x80; }, // Rotate left into carry
            check: |s| { assert_eq!(s.memory.data.registers[16], 0x00); unsafe { assert!(s.get_flag(Flags::C)); } }
        },
        ror: ROR(16) {
            setup: |s| { s.memory.data.registers[16] = 0x01; }, // Rotate right into carry
            check: |s| { assert_eq!(s.memory.data.registers[16], 0x00); unsafe { assert!(s.get_flag(Flags::C)); } }
        },
        asr: ASR(16) {
            setup: |s| { s.memory.data.registers[16] = 0x80; }, // -128
            check: |s| { assert_eq!(s.memory.data.registers[16], 0xC0); } // -64 (sign extended)
        },
        swap: SWAP(16) {
            setup: |s| { s.memory.data.registers[16] = 0xF0; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 0x0F); }
        },
        bset: BSET(1) { // Set Z flag (1)
            setup: |s| {},
            check: |s| { unsafe { assert!(s.get_flag(Flags::Z)); } }
        },
        bclr: BCLR(1) { // Clear Z flag
            setup: |s| { unsafe { s.set_flag(Flags::Z, true); } },
            check: |s| { unsafe { assert!(!s.get_flag(Flags::Z)); } }
        },
        sbi: SBI(0x10, 0) { // Set bit 0 in IO port 0x10
            setup: |s| { s.memory.data.io[0x10] = 0x00; },
            check: |s| { assert_eq!(s.memory.data.io[0x10], 0x01); }
        },
        cbi: CBI(0x10, 0) { // Clear bit 0
            setup: |s| { s.memory.data.io[0x10] = 0x01; },
            check: |s| { assert_eq!(s.memory.data.io[0x10], 0x00); }
        },
        bst: BST(16, 0) { // Store bit 0 of R16 to T flag
            setup: |s| { s.memory.data.registers[16] = 0x01; },
            check: |s| { unsafe { assert!(s.get_flag(Flags::T)); } }
        },
        bld: BLD(16, 3) { // Load T flag to bit 0 of R16
            setup: |s| { unsafe { s.set_flag(Flags::T, true); } s.memory.data.registers[16] = 0xF7; },
            check: |s| { assert_eq!(s.memory.data.registers[16], 0xFF); }
        },
        sec: SEC { setup: |s| {}, check: |s| { unsafe { assert!(s.get_flag(Flags::C)); } } },
        clc: CLC { setup: |s| { unsafe { s.set_flag(Flags::C, true); } }, check: |s| { unsafe { assert!(!s.get_flag(Flags::C)); } } },
        sen: SEN { setup: |s| {}, check: |s| { unsafe { assert!(s.get_flag(Flags::N)); } } },
        cln: CLN { setup: |s| { unsafe { s.set_flag(Flags::N, true); } }, check: |s| { unsafe { assert!(!s.get_flag(Flags::N)); } } },
        sez: SEZ { setup: |s| {}, check: |s| { unsafe { assert!(s.get_flag(Flags::Z)); } } },
        clz: CLZ { setup: |s| { unsafe { s.set_flag(Flags::Z, true); } }, check: |s| { unsafe { assert!(!s.get_flag(Flags::Z)); } } },
        sei: SEI { setup: |s| {}, check: |s| { unsafe { assert!(s.get_flag(Flags::I)); } } },
        cli: CLI { setup: |s| { unsafe { s.set_flag(Flags::I, true); } }, check: |s| { unsafe { assert!(!s.get_flag(Flags::I)); } } },
        ses: SES { setup: |s| {}, check: |s| { unsafe { assert!(s.get_flag(Flags::S)); } } },
        cls: CLS { setup: |s| { unsafe { s.set_flag(Flags::S, true); } }, check: |s| { unsafe { assert!(!s.get_flag(Flags::S)); } } },
        sev: SEV { setup: |s| {}, check: |s| { unsafe { assert!(s.get_flag(Flags::V)); } } },
        clv: CLV { setup: |s| { unsafe { s.set_flag(Flags::V, true); } }, check: |s| { unsafe { assert!(!s.get_flag(Flags::V)); } } },
        set: SET { setup: |s| {}, check: |s| { unsafe { assert!(s.get_flag(Flags::T)); } } },
        clt: CLT { setup: |s| { unsafe { s.set_flag(Flags::T, true); } }, check: |s| { unsafe { assert!(!s.get_flag(Flags::T)); } } },
        seh: SEH { setup: |s| {}, check: |s| { unsafe { assert!(s.get_flag(Flags::H)); } } },
        clh: CLH { setup: |s| { unsafe { s.set_flag(Flags::H, true); } }, check: |s| { unsafe { assert!(!s.get_flag(Flags::H)); } } },

        // =============================================================
        // OTHER
        // =============================================================
        nop: NOP { setup: |s| {}, check: |s| {} },
        sleep: SLEEP { setup: |s| {}, check: |s| {} },
        wdr: WDR { setup: |s| {}, check: |s| {} },
        break_inst: BREAK { setup: |s| {}, check: |s| {} }, // Will return Error "halt" in implementation

        // =============================================================
        // BIT MANIPULATION ALIASES
        // =============================================================
        sbr: SBR(16, 0x01) { // Alias for ORI
             setup: |s| { s.memory.data.registers[16] = 0x00; },
             check: |s| { assert_eq!(s.memory.data.registers[16], 0x01); }
        },
        cbr: CBR(16, 0x01) { // Alias for ANDI with complement
             setup: |s| { s.memory.data.registers[16] = 0x01; },
             check: |s| { assert_eq!(s.memory.data.registers[16], 0x00); }
        },
        clr: CLR(16) { // Alias for EOR Rd, Rd
             setup: |s| { s.memory.data.registers[16] = 0xFF; },
             check: |s| { assert_eq!(s.memory.data.registers[16], 0x00); }
        },
        ser: SER(16) { // Alias for LDI Rd, 0xFF
             setup: |s| { s.memory.data.registers[16] = 0x00; },
             check: |s| { assert_eq!(s.memory.data.registers[16], 0xFF); }
        },
        tst: TST(16) { // Alias for AND Rd, Rd
             setup: |s| { s.memory.data.registers[16] = 0xFF; },
             check: |s| { unsafe { assert!(!s.get_flag(Flags::Z)); } }
        },

        // =============================================================
        // COMPARISON
        // =============================================================
        cp: CP(16, 17) {
            setup: |s| { s.memory.data.registers[16] = 10; s.memory.data.registers[17] = 10; },
            check: |s| { unsafe { assert!(s.get_flag(Flags::Z)); } }
        },
        cpc: CPC(16, 17) {
            setup: |s| { s.memory.data.registers[16] = 10; s.memory.data.registers[17] = 9; unsafe { s.set_flag(Flags::C, true);s.set_flag(Flags::Z, true) } },
            check: |s| { unsafe { assert!(s.get_flag(Flags::Z)); } }
        },
        cpc2: CPC(16, 17) {
            setup: |s| { s.memory.data.registers[16] = 10; s.memory.data.registers[17] = 9; unsafe { s.set_flag(Flags::C, true);s.set_flag(Flags::Z, false) } },
            check: |s| { unsafe { assert!(!s.get_flag(Flags::Z)); } }
        },
        cpi: CPI(16, 10) {
            setup: |s| { s.memory.data.registers[16] = 10; },
            check: |s| { unsafe { assert!(s.get_flag(Flags::Z)); } }
        },

        // =============================================================
        // UNIMPLEMENTED / SPECIAL / PLACEHOLDERS
        // =============================================================
        // These are required to pass the exhaustiveness check, but may fail logic
        // if not handled in execute_inst or marked as todo!()
        des: DES(0) { setup: |s| {}, check: |s| {} }
    }
}