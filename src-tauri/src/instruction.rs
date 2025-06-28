use crate::operand::Operand;
use opcodeGen::{
    RawInst,
    Opcode_list,
};
use crate::constraint::Constraint::N;

pub struct Instuction{
    name: String,
    opcode: RawInst,
    operands: Option<Vec<Operand>>,
}
impl Instuction{
    pub fn new(name: String, opcode: RawInst, operands: Vec<Operand>) -> Instuction{
        Instuction{name,opcode,operands: Some(operands)}
    }
    pub fn decode_from_opcode(opcode: u32) -> Option<Instuction>{
        let a:Vec<i32>;
        let isnt: Option<RawInst> = Self::mach_instruction(opcode);
        if isnt.is_none() {
            return None;
        }
        None
    }
    fn mach_instruction(opcode: u32) -> Option<RawInst>{
        for i in Opcode_list{
            if opcode & i.bin_mask == i.bin_opcode{
                return Some(i)
            }
        }
        None
}
}
