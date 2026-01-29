use super::instruction::{Instruction};
use crate::error::Result;
use std::fs;
use std::io;
use std::io::ErrorKind;
use anyhow::anyhow;
use opcode_gen::RawInst;
use crate::error::Error::{InvalidRecordType, NotImplemented};

const MAX_BYTE_COUNT: usize =16;//todo replace by something meaningful
struct RawData {
    data:[u8;MAX_BYTE_COUNT],
    len:u32,
    address:u32
}

pub(crate) fn parse_hex(path:String) ->Result<Vec<Instruction>>{
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
        let byte_count = u32::from_str_radix(&line[1..3], 16)?;
        let address = u32::from_str_radix(&line[3..7], 16)?;
        let rec_type = u32::from_str_radix(&line[7..9], 16)?;
        let mut data:[u8;MAX_BYTE_COUNT] = [0;MAX_BYTE_COUNT];
        for n in 0..byte_count as usize {
            data[n] = u8::from_str_radix(&line[(9 + 2*n).. 11 + (2 * n)], 16)?;
        }
        match rec_type {
            0=>{parsed_data.push(RawData{data, len: byte_count, address: address+address_mul});}
            1=>{break}
            2=>{address_mul = (u128::from_str_radix(&line[9..9 + (2 * byte_count) as usize], 16)? * 16) as u32 }
            3=>{return Err(anyhow!(NotImplemented {err: "record type 3 not implemented".parse()? }));}
            4=>{return Err(anyhow!(NotImplemented {err: "record type 4 not implemented".parse()? }));}
            5=>{return Err(anyhow!(NotImplemented {err: "record type 5 not implemented".parse()? }));}
            _=>{return Err(anyhow!(InvalidRecordType {err:rec_type.to_string()}));}
        }

    }
    let mut continue_prev = false;
    let mut inst_list:Vec<Instruction> = vec![];
     for data in parsed_data.iter(){
         let mut i:usize =0;
         if continue_prev{
             continue_prev = false;
             let mut inst = inst_list.pop().unwrap();
             inst.raw_opcode+=((data.data[i+1] as u32)<<8)+data.data[i] as u32;
             inst_list.push(inst);
             i+=2;
         }
         while i<=(data.len-2) as usize {
             let mut inst:Instruction = Instruction::decode_from_opcode(((data.data[i+1] as u16)<<8)+data.data[i] as u16)?;
             inst.address = data.address+i as u32;
             i+=2;
             if RawInst::get_inst_from_id(inst.opcode_id).unwrap().len ==2{
                 inst.raw_opcode= inst.raw_opcode<<16;
                 if data.data.get(i).is_none() {
                     continue_prev = true;

                 }else {
                     inst.raw_opcode+=((data.data[i+1] as u32)<<8)+data.data[i] as u32;
                     i+=2;
                 }
             }
             inst_list.push(inst);
         }
     }
    
    for inst in &mut inst_list{
        inst.mach_registers()?;
    }
    Ok(inst_list)
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



#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_parse_hex(){ //todo
        let ignored_opcodes = vec!["ld","ldd","st","std","lac","las","lat","xch","lpm","elpm"];
       let out = parse_hex("/opt/projects/avr-simulator-rs/tests/disassembly/main.hex".to_string()).unwrap();
        let f = fs::read_to_string("/opt/projects/avr-simulator-rs/tests/disassembly/out.txt").unwrap();
        let input = f.split("\n").collect::<Vec<&str>>();
        let mut iter = 6;
        for i in &out{
            iter+=1;
            let data:Vec<_> = input[iter].split(":").collect::<Vec<_>>()[1].split(";").collect::<Vec<_>>()[0].trim().split("\t").collect::<Vec<_>>();
            let opcode = data[1];
            let mut op1=None;
            let mut op2=None;
            let mut op3=None;
            let mut operands = None;
            if data.len() >=3{
                operands = Some(data[2].split(",").collect::<Vec<_>>());
                op1 = operands.as_ref().unwrap().get(0).clone();
                op2 = operands.as_ref().unwrap().get(1).clone();
                op3 = operands.as_ref().unwrap().get(2).clone();
            }

            assert_eq!(opcode,i.get_raw_inst().unwrap().name.to_string().to_lowercase().as_str());
            println!("{:#x}",i.address);
            if ignored_opcodes.contains(&opcode){
                continue;
            }
            if let Some(op1) = op1 {
                let op =i.operands.as_ref().unwrap();
                println!("{:?}", op);
                assert_eq!(*op1.trim().to_lowercase(),op[0].map_string_from_value().unwrap().to_lowercase());
                if let Some(op2) = op2 {

                    assert_eq!(*op2.trim().to_lowercase(),op[1].map_string_from_value().unwrap().to_lowercase());
                }
                if let Some(op3) = op3 {
                    assert_eq!(*op3.trim().to_lowercase(),op[2].map_string_from_value().unwrap().to_lowercase());
                }
            }
        }
    }
}