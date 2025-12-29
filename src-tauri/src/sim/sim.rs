#![allow(unused_mut)]

use anyhow::anyhow;
use bin_expr_parser_macro::execute;
use device_parser::{get_common_registers, get_tree_map, AvrDeviceFile, CommonRegisters};
use device_parser::r#struct::common_registers::Flags;
use opcode_gen::{CustomOpcodes, Opcode, RawInst};
use crate::sim::memory::Memory;
use crate::error::Result;
use crate::project::PROJECT;
use crate::sim::instruction::Instruction;

#[derive(Debug,Clone)]
enum RamSize {
    Size16,
    Size24,
}
impl Into<u8> for RamSize {
    fn into(self) -> u8 {
        match self {
            RamSize::Size16 => 2,
            RamSize::Size24 => 3,
        }
    }
}
impl Into<u16> for RamSize {
    fn into(self) -> u16 {
        match self {
            RamSize::Size16 => 2,
            RamSize::Size24 => 3,
        }
    }
}
impl Default for RamSize {
    fn default() -> Self {
        RamSize::Size16
    }
}


#[derive(Default,Debug)]
pub struct Sim{
    pub memory: Memory,
    registers: CommonRegisters,
    ram_size: RamSize, //todo
}
impl Sim {
    pub fn init(&mut self) -> Result<()> {
        let mcu = &PROJECT.lock().unwrap().get_project()?.mcu;
        let atdf = get_tree_map().get(mcu).ok_or(anyhow!("invalid mcu"))?;
        let inst = PROJECT.lock().unwrap().get_instruction_list()?;
        let eeprom = PROJECT.lock().unwrap().get_eeprom_data()?;
        self.init_iner(atdf, inst, eeprom)?;
        Ok(())
    }
    pub fn init_iner(&mut self, atdf: &'static AvrDeviceFile, inst: Vec<Instruction>, eeprom: Vec<u8>) -> Result<()> {
        let mut inst_vec: Vec<Instruction> = Vec::new();
        inst_vec.resize((atdf.devices.address_spaces.iter().find(|x| { x.id == "prog" }).unwrap().size / 2) as usize, Instruction::decode_from_opcode(CustomOpcodes::EMPTY as u16)?);
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
        self.registers.init_regs(atdf, &mut self.memory.data.io)?;

        Ok(())
    }
    pub fn init_debug(atdf: &'static AvrDeviceFile, flash: Vec<Instruction>) -> Result<Sim> {
        let mut s = Sim::default();
        s.init_iner(atdf, flash, vec![])?;
        Ok(s)
    }
    pub fn exec_debug(&mut self) -> Result<()> {
        unsafe {
            self.execute_inst(&self.memory.flash[0].clone())
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
    unsafe fn set_flag(&mut self,flags: Flags, value: bool){ self.registers.set_flag(flags, value) }
    unsafe fn get_flag(&mut self,flags: Flags)->bool { self.registers.get_flag(flags) }

    unsafe fn push(&mut self, data:u32,len:u16){
        let mut sp: u16 = ((self.registers.spL.get_data() as u16) << 8) + (self.registers.spH.get_data() as u16);
        for i in 0..(len as u16) {
            self.memory.data.ram[(sp - i) as usize] = ((data >> (8 * i)) & 0xff) as u8;
        }
        sp -= len;
        self.registers.spL.set_data((sp & 0xff) as u8);
        self.registers.spH.set_data(((sp >> 8) & 0xff) as u8);
    }
    unsafe fn pop(&mut self,len:u16)->u32{
        let mut sp: u16 = ((self.registers.spL.get_data() as u16) << 8) + (self.registers.spH.get_data() as u16);
        let mut data: u32 = 0;
        for i in 0..len {
            data = data << 8;
            data += self.memory.data.ram[(sp + i) as usize] as u32;
        }
        sp += len;
        self.registers.spL.set_data((sp & 0xff) as u8);
        self.registers.spH.set_data(((sp >> 8) & 0xff) as u8);
        data

    }
    pub unsafe fn execute_inst(&mut self, instruction: &Instruction) -> Result<()> {
        unsafe {
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
                let ra = match (ind1 < reg.len()) {
                    true => Ok(&mut *reg_ptr.add(ind1)),
                    false => Err(anyhow!("invalid reg index")),
                };
                let rb = match (ind2 < reg.len()) {
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
                    Ok(())
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
                    Ok(())
                }
                Opcode::ADIW => {
                    let data: u16 = ((reg[ind1+1] as u16) << 8) + (reg[ind1] as u16);
                    let (res, ov) = data.overflowing_add(op2 as u16);
                    println!("data:{},res:{}",data,res);
                    let n = (res >> 15) == 1;
                    self.set_flag(Flags::N, n);
                    self.set_flag(Flags::Z, res == 0);
                    self.set_flag(Flags::C, ov);
                    let v = n&((data>>15) ==1);
                    self.set_flag(Flags::V, v);
                    self.set_flag(Flags::S, execute![n^v]?);

                    reg[ind1] = (res & 0xff) as u8;
                    reg[ind1 + 1] = ((res >> 8) & 0xff) as u8;
                    Ok(())
                }
                Opcode::AND => {
                    let res = execute![ra&rb]?;

                    let n = execute![res7]?;
                    self.set_flag(Flags::N, n);
                    self.set_flag(Flags::Z, res == 0);
                    self.set_flag(Flags::S, n);
                    self.set_flag(Flags::V, false);

                    reg[ind1] = res;
                    Ok(())
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
                    Ok(())
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
                    Ok(())
                }
                Opcode::BCLR => {
                    let f = Flags::get_flag(op1 as u8)?;
                    self.set_flag(f, false);

                    Ok(())
                }
                Opcode::BLD => {
                    reg[ind1] =match self.get_flag(Flags::T){
                        true => {(*ra? | (1<<op2))}
                        false=>{{(*ra? & 0xff-(1<<op2))}}
                    };
                    Ok(())
                }
                Opcode::BRBC => {
                    let f = self.get_flag(Flags::get_flag(op1 as u8)?);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRBS => {
                    let f = self.get_flag(Flags::get_flag(op1 as u8)?);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRCC => {
                    let f = self.get_flag(Flags::C);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRCS => {
                    let f = self.get_flag(Flags::C);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BREAK => {
                    Err(anyhow!("halt"))
                }
                Opcode::BREQ => {
                    let f = self.get_flag(Flags::Z);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRGE => {
                    let f = self.get_flag(Flags::S);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRHC => {
                    let f = self.get_flag(Flags::H);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRHS => {
                    let f = self.get_flag(Flags::H);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRID => {
                    let f = self.get_flag(Flags::I);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRIE => {
                    let f = self.get_flag(Flags::I);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRLO => {
                    let f = self.get_flag(Flags::C);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRLT => {
                    let f = self.get_flag(Flags::S);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRMI => {
                    let f = self.get_flag(Flags::N);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRNE => {
                    let f = self.get_flag(Flags::Z);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRPL => {
                    let f = self.get_flag(Flags::N);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRSH => {
                    let f = self.get_flag(Flags::C);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRTC => {
                    let f = self.get_flag(Flags::T);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRTS => {
                    let f = self.get_flag(Flags::T);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRVC => {
                    let f = self.get_flag(Flags::V);
                    if !f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BRVS => {
                    let f = self.get_flag(Flags::V);
                    if f {
                        self.memory.program_couter = (self.memory.program_couter as i64 + op2) as u32;
                    }

                    Ok(())
                }
                Opcode::BSET => {
                    self.set_flag(Flags::get_flag(op1 as u8)?, true);
                    Ok(())
                }
                Opcode::BST => {
                    let bit = (*ra? >> op2) == 1;
                    self.set_flag(Flags::T, bit);
                    Ok(())
                }
                Opcode::CALL => {
                    self.push(self.memory.program_couter + 2, self.ram_size.clone().into());
                    self.memory.program_couter = op1 as u32;
                    Ok(())
                }
                Opcode::CBI => {
                    self.memory.data.io[op1 as usize] &= 0xff - (1<<(op2 as u8));
                    Ok(())
                }
                Opcode::CBR => {
                    let res = *ra? & (0xff - (op2 as u8));
                    self.set_flag(Flags::V, false);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N, n);
                    self.set_flag(Flags::S, n);
                    self.set_flag(Flags::Z, res == 0);

                    reg[op1 as usize] = res;
                    Ok(())
                }
                Opcode::CLC => {
                    self.set_flag(Flags::C, false);
                    Ok(())
                }
                Opcode::CLH => {
                    self.set_flag(Flags::H, false);
                    Ok(())
                }
                Opcode::CLI => {
                    self.set_flag(Flags::I, false);
                    Ok(())
                }
                Opcode::CLN => {
                    self.set_flag(Flags::N, false);
                    Ok(())
                }
                Opcode::CLR => {
                    reg[op1 as usize] = 0;
                    self.set_flag(Flags::S, false);
                    self.set_flag(Flags::V, false);
                    self.set_flag(Flags::N, false);
                    self.set_flag(Flags::Z, true);
                    Ok(())
                }
                Opcode::CLS => {
                    self.set_flag(Flags::S, false);
                    Ok(())
                }
                Opcode::CLT => {
                    self.set_flag(Flags::T, false);
                    Ok(())
                }
                Opcode::CLV => {
                    self.set_flag(Flags::V, false);
                    Ok(())
                }
                Opcode::CLZ => {
                    self.set_flag(Flags::Z, false);
                    Ok(())
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
                    Ok(())
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
                    Ok(())
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
                    if (res != 0) {
                        self.set_flag(Flags::Z, false)
                    }
                    self.set_flag(Flags::C, ovr|ovr1);
                    Ok(())
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
                    self.set_flag(Flags::V, ovr);
                    Ok(())
                }
                Opcode::CPSE => {
                    if (ra? == rb?) {
                        if RawInst::get_inst_from_id(self.memory.flash[(self.memory.program_couter + 1) as usize].opcode_id)?.len == 2 {
                            self.memory.program_couter += 2
                        } else {
                            self.memory.program_couter += 1
                        }
                    }
                    Ok(())
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
                    Ok(())
                }
                Opcode::DES => {
                    Err(anyhow!("not implemented"))
                }
                Opcode::EICALL => {
                    match self.ram_size {
                        RamSize::Size16 => {
                            self.push(self.memory.program_couter + 1, 2);
                            self.memory.program_couter = (reg[30] as u32) + ((reg[31] as u32) << 8)
                        }
                        RamSize::Size24 => {
                            self.push(self.memory.program_couter + 1, 3);
                            self.memory.program_couter = (reg[30] as u32) + ((reg[31] as u32) << 8) + ((self.registers.eind.get_data() as u32) << 16)
                        }
                    }
                    Ok(())
                }
                Opcode::EIJMP => {
                    match self.ram_size {
                        RamSize::Size16 => {
                            self.memory.program_couter = (reg[30] as u32) + ((reg[31] as u32) << 8)
                        }
                        RamSize::Size24 => {
                            self.memory.program_couter = (reg[30] as u32) + ((reg[31] as u32) << 8) + ((self.registers.eind.get_data() as u32) << 16)
                        }
                    }
                    Ok(())
                }
                Opcode::ELPM => { //todo might have issues
                    let mut ptr = (reg[30] as u32) + ((reg[31] as u32) << 8) + ((self.registers.rampz.get_data() as u32) << 16) >> 1;
                    let data: u16 = (self.memory.flash[ptr as usize].raw_opcode & 0xffff) as u16;

                    reg[op1 as usize] = (data >> (8 * (ptr & 1))) as u8;
                    if op2 != 0 {
                        ptr += 1;
                        reg[30] = (ptr & 0xff) as u8;
                        reg[31] = ((ptr >> 8) & 0xff) as u8;
                        self.registers.rampz.set_data(((ptr >> 16) & 0xff) as u8);
                    }
                    Ok(())
                }
                Opcode::EOR => {
                    let res = execute![ra^rb]?;
                    let n = execute![res7]?;
                    self.set_flag(Flags::N, n);
                    self.set_flag(Flags::S, n);
                    self.set_flag(Flags::V, false);
                    self.set_flag(Flags::Z, res == 0);

                    reg[op1 as usize] = res;

                    Ok(())
                }
                Opcode::FMUL => {
                    let mut res = (*ra? as u16) * (*rb? as u16);
                    let c = (res >> 15) == 1;
                    res <<= 1;
                    reg[1] = (res >> 8) as u8;
                    reg[0] = (res & 0xff) as u8;

                    self.set_flag(Flags::C, c);
                    self.set_flag(Flags::Z, res == 0);

                    Ok(())
                }
                Opcode::FMULS => {
                    let mut res = (*ra? as i16) * (*rb? as i16);
                    let c = (res >> 15) == 1;
                    res <<= 1;
                    reg[1] = (res >> 8) as u8;
                    reg[0] = (res & 0xff) as u8;

                    self.set_flag(Flags::C, c);
                    self.set_flag(Flags::Z, res == 0);

                    Ok(())
                }
                Opcode::FMULSU => {
                    let mut res = (*ra? as i16) * (*rb? as i16); //todo
                    let c = (res >> 15) == 1;
                    res <<= 1;
                    reg[1] = (res >> 8) as u8;
                    reg[0] = (res & 0xff) as u8;

                    self.set_flag(Flags::C, c);
                    self.set_flag(Flags::Z, res == 0);

                    Ok(())
                }
                Opcode::ICALL => {
                    self.push(self.memory.program_couter + 1, self.ram_size.clone().into());
                    self.memory.program_couter = 0;
                    self.memory.program_couter = (reg[30] as u32) + ((reg[31] as u32) << 8);
                    Ok(())
                }
                Opcode::IJMP => {
                    self.memory.program_couter = 0;
                    self.memory.program_couter = (reg[30] as u32) + ((reg[31] as u32) << 8);
                    Ok(())
                }
                Opcode::IN => {
                    reg[ind1] = self.memory.data.io[ind2];
                    Ok(())
                }
                Opcode::INC => {
                    let res = *ra? + 1;
                    let v = res == 0x80;
                    self.set_flag(Flags::V, v);
                    let n = execute![res7]?;
                    self.set_flag(Flags::N, n);
                    self.set_flag(Flags::S, v ^ n);
                    self.set_flag(Flags::Z, res == 0);
                    Ok(())
                }
                Opcode::JMP => {
                    self.memory.program_couter = ind1 as u32;
                    Ok(())
                }
                Opcode::LAC => {
                    let ptr = (reg[30] as u16) + ((reg[31] as u16) << 8);
                    let tmp = self.memory.data.ram[ptr as usize];
                    self.memory.data.ram[ptr as usize] &= 0xff - *ra?;
                    reg[ind1] = tmp;
                    Ok(())
                }
                Opcode::LAS => {
                    let ptr = (reg[30] as u16) + ((reg[31] as u16) << 8);
                    let tmp = self.memory.data.ram[ptr as usize];
                    self.memory.data.ram[ptr as usize] |= *ra?;
                    reg[ind1] = tmp;
                    Ok(())
                }
                Opcode::LAT => {
                    let ptr = (reg[30] as u16) + ((reg[31] as u16) << 8);
                    let tmp = self.memory.data.ram[ptr as usize];
                    self.memory.data.ram[ptr as usize] = !self.memory.data.ram[ptr as usize] & *ra?;
                    reg[ind1] = tmp;
                    Ok(())
                }
                Opcode::LD => {
                    let ptr = match op3 {
                        3 => { //x

                            Ok((reg[26] as u32) + ((reg[27] as u32) << 8) + ((self.registers.rampx.get_data() as u32) << 16))
                        }
                        2 => { //y
                            Ok((reg[28] as u32) + ((reg[29] as u32) << 8) + ((self.registers.rampy.get_data() as u32) << 16))
                        }
                        0 => { //z
                            Ok((reg[30] as u32) + ((reg[31] as u32) << 8) + ((self.registers.rampz.get_data() as u32) << 16))
                        }
                        _ => {
                            Err(anyhow!("invalid opcode"))
                        }
                    }?;

                    todo!();
                    //reg[ind1] =self.memory.data[ptr as usize];
                    Ok(())
                }
                Opcode::LDD => { todo!() }
                Opcode::LDI => { todo!() }
                Opcode::LDS => { todo!() }
                Opcode::LPM => { todo!() }
                Opcode::LSL => { todo!() }
                Opcode::LSR => { todo!() }
                Opcode::MOV => { todo!() }
                Opcode::MOVW => { todo!() }
                Opcode::MUL => { todo!() }
                Opcode::MULS => { todo!() }
                Opcode::MULSU => { todo!() }
                Opcode::NEG => { todo!() }
                Opcode::NOP => { Ok(())}
                Opcode::OR => { todo!() }
                Opcode::ORI => { todo!() }
                Opcode::OUT => { todo!() }
                Opcode::POP => { todo!() }
                Opcode::PUSH => { todo!() }
                Opcode::RCALL => { todo!() }
                Opcode::RET => { todo!() }
                Opcode::RETI => { todo!() }
                Opcode::RJMP => { todo!() }
                Opcode::ROL => { todo!() }
                Opcode::ROR => { todo!() }
                Opcode::SBC => { todo!() }
                Opcode::SBCI => { todo!() }
                Opcode::SBI => { todo!() }
                Opcode::SBIC => { todo!() }
                Opcode::SBIS => { todo!() }
                Opcode::SBIW => { todo!() }
                Opcode::SBR => { todo!() }
                Opcode::SBRC => { todo!() }
                Opcode::SBRS => { todo!() }
                Opcode::SEC => { todo!() }
                Opcode::SEH => { todo!() }
                Opcode::SEI => { todo!() }
                Opcode::SEN => { todo!() }
                Opcode::SER => { todo!() }
                Opcode::SES => { todo!() }
                Opcode::SET => { todo!() }
                Opcode::SEV => { todo!() }
                Opcode::SEZ => { todo!() }
                Opcode::SLEEP => { todo!() }
                Opcode::SPM => { todo!() }
                Opcode::ST => { todo!() }
                Opcode::STD => { todo!() }
                Opcode::STS => { todo!() }
                Opcode::SUB => { todo!() }
                Opcode::SUBI => { todo!() }
                Opcode::SWAP => { todo!() }
                Opcode::TST => { todo!() }
                Opcode::WDR => { todo!() }
                Opcode::XCH => { todo!() }
            }?;
            self.memory.program_couter += instruction.get_raw_inst()?.len as u32;
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
    use log::warn;

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
        st: ST(0, 16, 3) { setup: |s| {}, check: |s| {} },
        std: STD(0, 16, 0) { setup: |s| {}, check: |s| {} },
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
            setup: |s| { s.memory.data.registers[16] = 10; s.memory.data.registers[17] = 9; unsafe { s.set_flag(Flags::C, true); } },
            check: |s| { unsafe { assert!(s.get_flag(Flags::Z)); } }
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