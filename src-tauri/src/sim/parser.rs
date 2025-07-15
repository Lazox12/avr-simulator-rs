use super::instruction::Instuction;
use crate::error::Result;
use std::fs;
use std::io;
use std::io::ErrorKind;

const MAX_BYTE_COUNT: usize =10;//todo replace by something meaningful
struct RawData {
    data:[u8;MAX_BYTE_COUNT],
    len:u32,
    address:u32
}

pub(crate) fn parse_hex(path:String) ->Result<Vec<Instuction>>{
    let contents = fs::read_to_string(path);
    let mut i =0;
    let mut address_mul:u32 = 0;
    let mut parsed_data: Vec<RawData> = Vec::new();
    for line in contents?.lines() {
        i +=1;
        if !line.starts_with(":") {
            return Err(io::Error::new(ErrorKind::InvalidData, "file should start with \":\" as in intel hex").into());
        }
        if !calculate_checksum(&line)? {
            return Err(io::Error::new(ErrorKind::InvalidData, format!("file checksum does not match on line: {}", i)).into());
        }
        let byte_count = u32::from_str_radix(&line[1..2], 16)?;
        let address = u32::from_str_radix(&line[3..6], 16)?;
        let rec_type = u32::from_str_radix(&line[7..8], 16)?;
        let mut data:[u8;MAX_BYTE_COUNT] = [0;MAX_BYTE_COUNT];
        for n in 0..byte_count as usize {
            data[n] = u8::from_str_radix(&line[(9 + 2*n).. 11 + (2 * n)], 16)?;
        }
        match rec_type {
            0=>{parsed_data.push(RawData{data, len: byte_count, address: address+address_mul});}
            1=>{break}
            2=>{address_mul = (u128::from_str_radix(&line[9..9 + (2 * byte_count) as usize], 16)? * 16) as u32 }
            3=>{return Err(io::Error::new(ErrorKind::InvalidData,"record type 3 not implemented").into());}
            4=>{return Err(io::Error::new(ErrorKind::InvalidData,"record type 4 not implemented").into());}
            5=>{return Err(io::Error::new(ErrorKind::InvalidData,"record type 5 not implemented").into());}
            _=>{return Err(io::Error::new(ErrorKind::InvalidData,"record type should be between 0 and 2").into());}
        }

    }
    let mut inst :Vec<Instuction> = vec![];
     for data in parsed_data.iter(){
         let i:usize =0;
         while i<(data.len-2) as usize {
             let i:Instuction = Instuction::decode_from_opcode(((data.data[i]as u32) <<8) +(data.data[i+1]) as u32, data.address+i as u32)?;
             inst.push(i);
         }
     }
    Ok(inst)
}
fn calculate_checksum(line:&str) -> Result<bool>{
    let checksum = u32::from_str_radix(&line[line.len()-2 ..],16)?;
    
    let parsed = (1..line.len() - 2)
        .step_by(2)
        .map(|i| u32::from_str_radix(&line[i..i + 2], 16));
    
    let mut count = 0;
    for i in parsed{
        count += i?;
    }
        
    if count.wrapping_neg() as u8 ==checksum as u8{
        return Ok(true);
    }
    Ok(false)
}