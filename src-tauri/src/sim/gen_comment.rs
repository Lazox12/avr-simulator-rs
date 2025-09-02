use crate::sim::instruction::Instruction;

pub fn gen_comment(i: &mut Instruction) {
    if i.opcode.name == "rcall" {
        if let Some(ops) = &i.operands {
            if ops.len() == 1 {
                if let Ok(val) = ops[0].value.read::<i32>() {
                    
                    i.comment = (i.address+ val as u32 + 2).to_string();
                }
            }
        }
    }
}
