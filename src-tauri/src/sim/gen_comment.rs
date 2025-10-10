use opcodeGen::RawInst;
use crate::sim::display::Display::{*};
use crate::sim::instruction::Instruction;
use opcodeGen::Opcode;
pub fn gen_comment(i: &mut Instruction) {
    match RawInst::get_inst_from_id(i.opcode_id).unwrap().name {

        Opcode::RJMP|Opcode::RCALL => {
            if let Some(ops) = &i.operands {
                if ops.len() == 1 {
                    i.comment_display = Hex;
                    i.comment = (i.address as i64 + ops[0].value + 2).to_string();
                }
            }
        }
        _=>{
            return;
        }
    }
}
