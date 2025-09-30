use std::cmp::PartialEq;
use std::ptr::write;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use super::operand::{Operand, OperandValue};
use opcodeGen::{RawInst, Opcode_list, ConstraintMap};
use crate::error::{Error, Result};
use crate::sim::constraint::Constraint;
use crate::sim::display::Display;
use crate::sim::gen_comment::gen_comment;

#[derive(Debug,Serialize,Clone)]
#[serde(rename_all = "camelCase")]
pub struct Instruction{
    pub(crate) comment: String,
    pub(crate) comment_display:Display,
    pub(crate) opcode: RawInst,
    pub(crate) operands: Option<Vec<Operand>>,
    pub(crate) address: u32,
    pub(crate) raw_opcode: u32,
}



impl Instruction{
    pub fn new(comment: String, opcode: RawInst, operands: Vec<Operand>) -> Instruction{
        Instruction{comment, comment_display: Display::None, opcode,operands: Some(operands),address:0,raw_opcode:0}
    }
    pub fn decode_from_opcode(opcode: u16) -> Result<Instruction>{
        let inst: RawInst = Self::match_raw_instruction_from_opcode(opcode)?;
        
        Ok(Instruction{
            comment: "".to_string(),
            comment_display: Display::None,
            opcode: inst,
            operands: None,
            address:0,
            raw_opcode:opcode as u32
        })
    }

    fn match_raw_instruction_from_opcode(opcode: u16) -> Result<RawInst>{
        for i in Opcode_list{
            if opcode & i.bin_mask == i.bin_opcode{
                return Ok(i)
            }
        }
        Err(Error::OpcodeNotFound{opcode: opcode as u32 })
    }
    pub(crate) fn mach_registers(&mut self) ->Result<()>{
        if self.opcode.constraints.is_none(){
            return Ok(());
        }
        let r:Result<Vec<Operand>> = self.opcode.constraints
            .unwrap()
            .iter()
            .map(|x| {
                let constraint = Constraint::from_str(String::from(x.constraint).as_str())?;
                let Rresult = Operand::map_value(Instruction::map_register_number(Instruction::decode_val(x.map, self.raw_opcode),constraint),constraint);
                let result: OperandValue;
                let mut name:String = "".to_string();
                if( Rresult.is_err()){
                    result = OperandValue::new(1);
                    name = "opcode:".to_string();
                    name+= self.raw_opcode.to_string().as_str();
                    name += &*"error: ".to_string();
                    name += Rresult.err().unwrap().to_string().as_str();
                }else{
                    result = Rresult.unwrap();
                }
                return Ok(Operand{
                    name,
                    constraint,
                    value:result,
                }
                );
            })
            .collect();

        self.operands = Some(r?);
        Ok(())
    }
    fn decode_val(mask: u32, opcode: u32) -> u32 {
        let mut result = 0;
        let mut bit_pos = 0;

        for i in 0..32 {
            if (mask >> i) & 1 == 1 {
                let bit = (opcode >> i) & 1;
                result |= bit << bit_pos;
                bit_pos += 1;
            }
        }

        result
    }
    fn map_register_number(mut value:u32,constraint: Constraint)->u32{
        match constraint {
            Constraint::d =>{
                value +16
            }
            Constraint::v =>{
                value*2
            }
            Constraint::a =>{
                value +16
            }
            Constraint::w =>{
                value *=2;
                value +24
            }
            Constraint::h =>{
                value*2
            }
            _ =>{
                value
            }
        }
    }
    

    pub(crate) fn gen_comment(&mut self){
        super::gen_comment::gen_comment(self);
    }
}



#[derive(Debug,Deserialize,Serialize,Clone)]
#[serde(rename_all = "camelCase")]
pub struct PartialInstruction{
    pub(crate) comment: String,
    pub(crate) operands: Option<Vec<String>>,
    pub(crate) address: u32,
    pub(crate) name:String,
}
impl TryInto<Instruction> for PartialInstruction {
    type Error = Error;

    fn try_into(self) -> std::result::Result<Instruction, Error> { //todo
        
        let r: Vec<&RawInst> = Opcode_list.iter().filter(|x| {x.name == self.name}).collect();
        if r.len() != 1{
            return Err(Error::InvalidInstructionName(self.name.clone()))
        }
        let opcode = r[0];
        let  comment_display = Display::decode(&*self.comment);
        let mut operands:Option<Vec<Operand>>;
        if self.operands.is_none(){
           operands = None;
        }
        else {
            if(self.operands.as_ref().unwrap().len() != opcode.constraints.unwrap().len()){
                return Err(Error::InvalidOperandCount{expected:opcode.constraints.unwrap().len(), got:self.operands.unwrap().len()})
            }
            let mut iter = opcode.constraints.unwrap().iter();
            let a = self.operands
                .unwrap()
                .iter()
                .map(|x| {
                    let constraint = Constraint::from_str(&*String::from(iter.next().unwrap().constraint))?;
                    Ok(Operand{name:"".to_string(),constraint,value:Operand::map_value_from_string(x, constraint)? }) 
                })
                .collect();
            
        }
        Ok(Instruction{
            comment:self.comment,
            comment_display,
            opcode:opcode.clone(),
            operands:self.operands,
            address:self.address,
            raw_opcode:0
        })
    }
}




