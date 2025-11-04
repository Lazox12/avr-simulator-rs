use std::fmt;
use serde::Serialize;
use serde::Deserialize;


#[derive(Debug,Clone,Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintMap{
    pub map:u32,
    pub constraint:char
}
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RawInst{
    pub opcode:&'static str,
    pub len:i8,
    pub name:Opcode,
    pub constraints:Option<&'static [ConstraintMap]>,
    pub bin_mask:u16,
    pub bin_opcode:u16,
    pub action:&'static str,
    pub description:&'static str,
}
pub const CUSTOM_INST:RawInst=RawInst{
    opcode: "",
    len: 0,
    name: Opcode::CUSTOM_INST(0),
    constraints: None,
    bin_mask: 0xff,
    bin_opcode: 0x00,
    action: "nothing",
    description: "custom instruction (not executable)",
};
impl RawInst{
    pub fn get_inst_id_from_opcode(opcode:u16) ->Option<usize>{
        Opcode_list.iter().position(|i| {
            opcode & i.bin_mask == i.bin_opcode
        })
    }
    pub fn get_inst_from_id(id:usize)->Option<&'static RawInst>{
        Opcode_list.get(id).or({
            let mut i = CUSTOM_INST.clone();
            i.name = Opcode::CUSTOM_INST(id as u32);
            match id {
                999=>{
                    i.opcode= ".word";
                }
                _=>{}
            }
            Some(Box::leak(Box::new(i)))
        })

    }
}


include!(concat!(env!("OUT_DIR"), "/opcode.rs"));

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        
        assert_eq!(Opcode_list.len(), 124);
    }
}
