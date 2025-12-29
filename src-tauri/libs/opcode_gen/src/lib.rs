use std::fmt;
use anyhow::anyhow;
use serde::Serialize;


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
    pub len:u8,
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
pub enum CustomOpcodes{
    WORD=999,
    REMINDER=998,
    EMPTY=1000

}
impl TryFrom<usize> for CustomOpcodes{
    type Error = anyhow::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value{
            999 => Ok(CustomOpcodes::WORD),
            998 => Ok(CustomOpcodes::REMINDER),
            1000 => Ok(CustomOpcodes::EMPTY),
            _ => Err(anyhow!("Opcode {} not found", value))
        }
    }
}
impl RawInst{
    pub fn get_inst_id_from_opcode_num(opcode:u16) ->Option<usize>{
        OPCODE_LIST.iter().position(|i| {
            opcode & i.bin_mask == i.bin_opcode
        })
    }
    pub fn get_inst_id_from_opcode(opcode:Opcode) ->Option<usize>{
        OPCODE_LIST.iter().position(|i| {
            i.name ==opcode
        })
    }
    pub fn get_inst_from_id(id:usize)->Result<&'static RawInst,anyhow::Error>{
        let opcode = OPCODE_LIST.get(id);
        if opcode.is_some(){
            return Ok(opcode.unwrap())
        }
        let mut i = CUSTOM_INST.clone();
        i.name = Opcode::CUSTOM_INST(id as u32);
        match CustomOpcodes::try_from(id)? {
            CustomOpcodes::WORD=>{
                i.opcode = ".word"
            }
            CustomOpcodes::REMINDER=>{
                i.opcode = ".reminder"
            }
            CustomOpcodes::EMPTY=>{
                i.opcode = ".empty"
            }
        }
        Ok(Box::leak(Box::new(i)))
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
        
        assert_eq!(OPCODE_LIST.len(), 124);
    }
}
