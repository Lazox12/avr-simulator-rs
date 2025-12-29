use bin_expr_parser_macro::execute;
use anyhow::{anyhow,Result};
#[test]
#[allow(unused)]
pub fn exec(){
    let a :Result<u8>= Ok(0xffu8);
    let b :Result<u8>= Ok(0xf0u8);
    let c :Result<u8>= Err(anyhow!("abcd"));
    let d :Result<u8>= Err(anyhow!("efgh"));
    let d =execute![a&b];
    println!("{:?}",d);
}
