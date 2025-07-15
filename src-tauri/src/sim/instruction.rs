use std::io;
use std::num::ParseIntError;
use super::operand::Operand;
use opcodeGen::{
    RawInst,
    Opcode_list,
};
use crate::error::{Error, Result};

pub struct Instuction{
    name: String,
    opcode: RawInst,
    operands: Option<Vec<Operand>>,
    address: u32,
}
impl Instuction{
    pub fn new(name: String, opcode: RawInst, operands: Vec<Operand>) -> Instuction{
        Instuction{name,opcode,operands: Some(operands),address:0}
    }
    pub fn decode_from_opcode(opcode: u32,address:u32) -> Result<Instuction>{
        let inst: RawInst = Self::mach_instruction(opcode)?;
        
        Ok(Instuction{
            name: "".to_string(),
            opcode: inst,
            operands: None,
            address,
        })
    }
    fn mach_instruction(opcode: u32) -> Result<RawInst>{
        for i in Opcode_list{
            if opcode & i.bin_mask == i.bin_opcode{
                return Ok(i)
            }
        }
        Err(Error::OpcodeNotFound{opcode:format!("{:#x}", opcode)})
}
}
