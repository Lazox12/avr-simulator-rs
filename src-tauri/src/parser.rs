use crate::instruction::Instuction;
use std::fs;
use std::num::ParseIntError;
use std::io::{ErrorKind};
use std::io;
use std::error::Error;
use std::fmt::format;

struct RawData {
    data:u128,
    address:u32
}

fn parse_hex(path:String) ->Result<Vec<Instuction>,Box<dyn Error>>{
    let contents = fs::read_to_string(path);
    let mut i =0;
    let mut address_mul:u32 = 0;
    let mut parsed_data: Vec<RawData> = Vec::new();
    for line in contents?.lines() {
        i +=1;
        if(!line.starts_with(":")){
            return Err(io::Error::new(ErrorKind::InvalidData, "file should start with \":\" as in intel hex").into());
        }
        if !(calculate_checksum(&line)?){
            return Err(io::Error::new(ErrorKind::InvalidData, format!("file checksum does not match on line: {}", i)).into());
        }
        let byte_count = u32::from_str_radix(&line[1..2], 16)?;
        let address = u32::from_str_radix(&line[3..6], 16)?;
        let rec_type = u32::from_str_radix(&line[7..8], 16)?;
        let data = u128::from_str_radix(&line[9..9 + (2 * byte_count) as usize], 16)?;
        match rec_type {
            0=>{parsed_data.push(RawData{data, address: address+address_mul});}
            1=>{break}
            2=>{address_mul = (data * 16) as u32 }
            3=>{return Err(io::Error::new(ErrorKind::InvalidData,"record type 3 not implemented").into());}
            4=>{return Err(io::Error::new(ErrorKind::InvalidData,"record type 4 not implemented").into());}
            5=>{return Err(io::Error::new(ErrorKind::InvalidData,"record type 5 not implemented").into());}
            _=>{return Err(io::Error::new(ErrorKind::InvalidData,"record type should be between 0 and 2").into());}
        }

    }

    Ok(vec![])
}
fn calculate_checksum(line:&str) -> Result<bool,ParseIntError>{
    let checksum = u32::from_str_radix(&line[line.len()-2 ..],16)?;

    let count : u32 = (1..line.len()-2)
        .step_by(2)
        .map(|i| u32::from_str_radix(&line[i..i + 2], 16))
        .sum::<Result<u32, ParseIntError>>()?;
    if count.wrapping_neg() ==checksum{
        return Ok(true);
    }
    Ok(false)
}