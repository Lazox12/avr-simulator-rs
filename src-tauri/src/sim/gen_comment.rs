use opcodeGen::RawInst;

use crate::sim::instruction::Instruction;
use opcodeGen::Opcode;
use crate::sim::constraint::Constraint;
use crate::sim::display::Display;
use crate::sim::operand::{Operand, OperandInfo};
use deviceParser::get_tree_map;
use crate::project::{Project, ProjectState, PROJECT};
use crate::error::{Error, Result};

pub fn gen_comment(i: &mut Instruction)->Result<()> {
    match RawInst::get_inst_from_id(i.opcode_id).unwrap().name {

        Opcode::RJMP|Opcode::RCALL => {
            if let Some(ops) = &i.operands {
                if ops.len() == 1 {
                    i.comment_display = Display::Hex;
                    i.comment = (i.address as i64 + ops[0].value + 2).to_string();
                }
            }
            Ok(())
        }
        _=>{
            Ok(())
        }
    }
}
pub fn gen_operand_details(i: &mut Instruction,state:&ProjectState)->Result<()>{
    match i.operands {
        Some(ref mut operands) => {
            for x in operands.into_iter(){
                match x.constraint {
                    Constraint::p|Constraint::P=>{
                        let mcu = state.mcu.clone();
                        if(mcu.is_none()){
                            return Err(Error::InvalidMcu("select mcu".to_string()));
                        }
                        let tree = &get_tree_map()?[&mcu.unwrap()];
                        let info = OperandInfo{
                            register_name: "test1".to_string(),
                            register_mask: "test2".to_string(),
                            description: "test3".to_string(),
                        };
                        x.operand_info = Some(info);
                        
                        return Ok(())
                    }
                    _=>{
                        return Ok(())
                    }
                }
            };
            Ok(())
        }
        None => {
            Ok(())
        }
    }

}