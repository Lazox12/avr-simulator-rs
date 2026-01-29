use anyhow::anyhow;
use opcode_gen::RawInst;

use crate::error::Result;
use crate::project::ProjectState;
use crate::sim::constraint::Constraint;
use crate::sim::display::Display;
use crate::sim::instruction::Instruction;
use crate::sim::operand::OperandInfo;
use device_parser::get_register_map;
use opcode_gen::Opcode;

pub fn gen_comment(i: &mut Instruction) -> Result<()> {
    match RawInst::get_inst_from_id(i.opcode_id).unwrap().name {
        Opcode::RJMP | Opcode::RCALL => {
            if let Some(ops) = &i.operands {
                if ops.len() == 1 {
                    i.comment_display = Display::Hex;
                    i.comment = (i.address as i64 + ops[0].value + 2).to_string();
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
pub fn gen_operand_details(i: &mut Instruction, state: &ProjectState) -> Result<()> {
    match i.operands {
        Some(ref mut operands) => {
            for x in operands.into_iter() {
                match x.constraint {
                    Constraint::p | Constraint::P => {
                        let tree = get_register_map(&state.mcu).ok_or(anyhow!("invalid mcu"))?;
                        let reg_opt = tree.get(&(x.value as u64 + 0x20));
                        if reg_opt.is_none() {
                            continue;
                        }
                        let reg = reg_opt.unwrap();
                        let info = OperandInfo {
                            register_name: reg.name.parse()?,
                            register_mask: serde_json::to_string(&reg.bitfields)?,
                            description: reg.caption.clone().unwrap_or(reg.name).parse()?,
                        };
                        x.operand_info = Some(info);

                        continue;
                    }
                    _ => continue,
                }
            }
            Ok(())
        }
        None => Ok(()),
    }
}
