use super::operand::{Operand, OperandValue};
use crate::error::{Error, Result};
use crate::sim::constraint::Constraint;
use crate::sim::display::Display;
use opcodeGen::{Opcode, RawInst};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use anyhow::anyhow;
use crate::project::ProjectState;

#[derive(Debug,Serialize,Clone)]
#[serde(rename_all = "camelCase")]
pub struct Instruction{
    pub(crate) comment: String,
    pub(crate) comment_display:Display,
    pub(crate) opcode_id:usize,
    pub(crate) operands: Option<Vec<Operand>>,
    pub(crate) address: u32,
    pub(crate) raw_opcode: u32,
}



impl Instruction{
    pub fn new(comment: String, opcode_id: usize, operands: Vec<Operand>) -> Instruction{
        Instruction{comment, comment_display: Display::None, opcode_id,operands: Some(operands),address:0,raw_opcode:0}
    }
    pub fn get_raw_inst(&self)->Result<&RawInst>{
        RawInst::get_inst_from_id(self.opcode_id)
    }
    pub fn decode_from_opcode(opcode: u16) -> Result<Instruction>{
        let inst = Self::match_raw_instruction_from_opcode(opcode).unwrap_or_else(|x|{
            999
        });
        
        Ok(Instruction{
            comment: "".to_string(),
            comment_display: Display::None,
            opcode_id: inst,
            operands: None,
            address:0,
            raw_opcode:opcode as u32
        })
    }

    fn match_raw_instruction_from_opcode(opcode: u16) -> Result<usize>{
        match RawInst::get_inst_id_from_opcode(opcode){
            None => {
                Err(anyhow!(Error::OpcodeNotFound{opcode: opcode as u32 }))
            }
            Some(i) => {
                Ok(i)
            }
        }
    }
    pub(crate) fn mach_registers(&mut self) ->Result<()>{

        match self.get_raw_inst()? {
            RawInst{name:Opcode::CUSTOM_INST(_),..  } => {
                self.operands = Some(vec![Operand{
                    name: "".to_string(),
                    constraint: Constraint::h,
                    value: self.raw_opcode.clone() as OperandValue,
                    operand_info: None,
                }]);
                Ok(())
            }
            i=>{
                if self.get_raw_inst()?.constraints.is_none(){
                    return Ok(());
                }
                match i.constraints.unwrap().iter()
                    .map(|x| {
                        let constraint = Constraint::from_str(String::from(x.constraint).as_str())?;
                        let rresult = Operand::map_value(Instruction::map_register_number(Instruction::decode_val(x.map, self.raw_opcode), constraint), constraint);
                        let result: OperandValue;
                        let mut name:String = "".to_string();
                        if( rresult.is_err()){
                            result = 1;
                            name = "opcode:".to_string();
                            name+= self.raw_opcode.to_string().as_str();
                            name += &*"error: ".to_string();
                            name += rresult.err().unwrap().to_string().as_str();
                        }else{
                            result = rresult.unwrap();
                        }
                        return Ok(Operand{
                            name,
                            constraint,
                            value:result,
                            operand_info: None,
                        }
                        );
                    })
                    .collect::<Result<Vec<Operand>>>()
                {
                    Ok(i)=>{
                        self.operands = Some(i);
                        Ok(())
                    }
                    Err(e)=>{
                        Err(e)
                    }
                }
            }
        }
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
    

    pub(crate) fn gen_comment(&mut self,state:&ProjectState)->Result<()>{
        super::gen_comment::gen_comment(self)?;
        super::gen_comment::gen_operand_details(self,state)?;
        Ok(())
    }
}

impl TryFrom<PartialInstruction> for Instruction {
    type Error = anyhow::Error;

    fn try_from(value:PartialInstruction) -> std::result::Result<Instruction, anyhow::Error> { //todo

        let opcode =RawInst::get_inst_from_id(value.opcode_id)?;
        let  comment_display = Display::decode(&*value.comment);

        Ok(Instruction{
            comment:value.comment,
            comment_display,
            opcode_id:value.opcode_id,
            operands:value.operands,
            address:value.address,
            raw_opcode:0
        })
    }
}

#[derive(Debug,Deserialize,Serialize,Clone)]
#[serde(rename_all = "camelCase")]
pub struct PartialInstruction{
    pub(crate) comment: String,
    pub(crate) comment_display: Display,
    pub(crate) operands: Option<Vec<Operand>>,
    pub(crate) address: u32,
    pub(crate) opcode_id:usize,
}

impl From<Instruction> for PartialInstruction {
    fn from(value:Instruction) -> PartialInstruction {
        PartialInstruction{
            comment: value.comment,
            comment_display:value.comment_display,
            operands: value.operands,          
            address: value.address,
            opcode_id: value.opcode_id,
        }
    }
}




