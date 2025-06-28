include!(concat!(env!("OUT_DIR"), "/opcode.rs"));



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        ;
        assert_eq!(Opcode_list.len(), 116);
    }
}
