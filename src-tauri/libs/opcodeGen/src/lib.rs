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
impl RawInst{
    pub fn get_inst_id_from_opcode(opcode:u16) ->Option<usize>{
        Opcode_list.iter().position(|i| {
            opcode & i.bin_mask == i.bin_opcode
        })
    }
    pub fn get_inst_from_id(id:usize)->Option<&'static RawInst>{
        Opcode_list.get(id)
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
