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
    pub len:i8,
    pub name:&'static str,
    pub constraints:Option<&'static [ConstraintMap]>,
    pub bin_mask:u16,
    pub bin_opcode:u16,
    pub action:&'static str,
    pub description:&'static str,
}


include!(concat!(env!("OUT_DIR"), "/opcode.rs"));



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        
        assert_eq!(Opcode_list.len(), 124);
    }
}
