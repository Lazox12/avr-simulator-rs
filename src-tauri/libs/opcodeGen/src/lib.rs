
pub struct ConstraintMap{
    pub map:u32,
    pub constraint:char
}
pub struct RawInst{
    pub opcode:&'static str,
    pub len:i8,
    pub name:&'static str,
    pub constraints:Option<&'static [ConstraintMap]>,
    pub bin_mask:u16,
    pub bin_opcode:u16
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
