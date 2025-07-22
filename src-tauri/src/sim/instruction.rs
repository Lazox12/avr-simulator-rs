use std::ptr::write;
use std::str::FromStr;
use super::operand::{Operand, OperandValue};
use opcodeGen::{
    RawInst,
    Opcode_list,
};
use crate::error::{Error, Result};
use crate::sim::constraint::Constraint;



pub struct Instuction{
    pub(crate) name: String,
    pub(crate) opcode: RawInst,
    pub(crate) operands: Option<Vec<Operand>>,
    pub(crate) address: u32,
    pub(crate) raw_opcode: u32,
}
impl Instuction{
    pub fn new(name: String, opcode: RawInst, operands: Vec<Operand>) -> Instuction{
        Instuction{name,opcode,operands: Some(operands),address:0,raw_opcode:0}
    }
    pub fn decode_from_opcode(opcode: u16,address:u32) -> Result<Instuction>{
        let inst: RawInst = Self::mach_instruction(opcode)?;
        
        Ok(Instuction{
            name: "".to_string(),
            opcode: inst,
            operands: None,
            address,
            raw_opcode:opcode as u32
        })
    }
    fn mach_instruction(opcode: u16) -> Result<RawInst>{
        for i in Opcode_list{
            if opcode & i.bin_mask == i.bin_opcode{
                return Ok(i)
            }
        }
        Err(Error::OpcodeNotFound{opcode: opcode as u32 })
    }
    pub(crate) fn mach_registers(&mut self) ->Result<()>{
        if self.opcode.constraints.is_none(){
            return Ok(());
        }
        let r:Result<Vec<Operand>> = self.opcode.constraints
            .unwrap()
            .iter()
            .map(|x| {
                let constraint = Constraint::from_str(String::from(x.constraint).as_str())?;
                return Ok(Operand{
                    name: "".parse().unwrap(),
                    constraint,
                    value:Instuction::map_value(Instuction::decode_val(x.map, self.raw_opcode),constraint)?,
                    }
                );
            })
            .collect();
        
        self.operands = Some(r?);
        Ok(())
    }
    fn decode_val(mask: u32, opcode: u32) -> u32 {
        let mut result = 0;
        let mut bit_pos = 0;

        for i in 0..32 {
            if (mask >> i) & 1 == 1 {
                let bit = (opcode >> i) & 1;
                result |= bit << bit_pos;
                bit_pos += 1;
            }
        }

        result
    }
    fn map_value(value:u32,constraint: Constraint)->Result<OperandValue>{
        match constraint {
            Constraint::r => {if(value<32){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("r: regiter number can`t be higher than 31, got {}", value)})}}
            Constraint::d => {if(value<32 && value >15){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("d: ldi regiter number can`t be higher than 31 or lower than 16, got {}", value)})}}
            Constraint::v => {if(value<32 && value%2 ==0){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("v: movw regiter number must be even and less than 32, got {}", value)})}}
            Constraint::a => {if(value<24 && value >15){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("a: fmul regiter number can`t be higher than 23 or lower than 16, got {}", value)})}}
            Constraint::w => {if(value==24||value ==26||value ==28||value ==30){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("w: adiw regiter number must be : 24,26,28 or 30, got {}", value)})}}
            Constraint::e => {if(value<3){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("e: point regiter must be x or Y or Z, got {}", value)})}}
            Constraint::b => {if(value<2){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("b: base pointer register must be 0 or 1, got {}", value)})}}
            Constraint::z => {if(value<2){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("z: Z pointer register increment must be 0 or 1, got {}", value)})}}
            Constraint::M => {if(value<226){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("M: immediate Value that must be from 0 to 255, got {}", value)})}}
            Constraint::n => {if(value<226){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("n: immediate Value that must be from 0 to 255 ( n = ~M ). Relocation impossible, got {}", value)})}}
            Constraint::s => {if(value<8){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("s: immediate Value that must be from 0 to 7, got {}", value)})}}
            Constraint::P => {if(value<64){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("P: Port address Value that must be from 0 to 63, got {}", value)})}}
            Constraint::p => {if(value<32){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("p: Port address Value that must be from 0 to 31, got {}", value)})}}
            Constraint::K => {if(value<64){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("K: immediate Value that must be from 0 to 63 {}", value)})}}
            Constraint::i => {if(value<256){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("i: immediate Value that must be from 0 to 255, got {}", value)})}}
            Constraint::j => {if(value<0xC0 &&value > 0x3F){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("j: 7 bit immediate Value that must be from 0x40 to 0xBF, got {}", value)})}}
            Constraint::l => {if(value<128){Ok(OperandValue::new(unsigned_to_signed(value,7)))}else{Err(Error::InvalidConstraintValue {err:format!("l: signed pc relative offset that must be from -64 to 63, got {}", value)})}} // represented by last 7 bits of the u32
            Constraint::L => {if(value<4096){Ok(OperandValue::new(unsigned_to_signed(value,12)))}else{Err(Error::InvalidConstraintValue {err:format!("L: signed pc relative offset that must be from -2048 to 2047, got {}", value)})}} // represented by last 12 bits of the u32
            Constraint::h => {if(value >0 ||true){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("h: absolute code address (call, jmp), got {}", value)})}} //todo
            Constraint::S => {if(value<8){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("S: immediate Value from 0 to 7 (S = s << 4), got {}", value)})}}
            Constraint::E => {if(value<16){Ok(OperandValue::new(value))}else{Err(Error::InvalidConstraintValue {err:format!("E: immediate Value from 0 to 15, shifted left by 4 (des), got {}", value)})}}
        }
    }

}
fn unsigned_to_signed(val:u32,len:u32)->i32{ //signed len in bits
    if(val>>len-1)==0{ // positive number
        val as i32
    }else{ //negative number
        ((1<<len) *-1) +(val as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_unsigned_to_signed(){
        assert_eq!(unsigned_to_signed(0b010101101110,12),1390);
        assert_eq!(unsigned_to_signed(0b1001010111101110,16),-27154);
        assert_eq!(unsigned_to_signed(0b0,14),0);
        assert_eq!(unsigned_to_signed(0b1111,4),-1);

    }
}