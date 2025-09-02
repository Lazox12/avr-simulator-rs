use std::cmp::PartialEq;
use std::ptr::write;
use std::str::FromStr;
use serde::Serialize;
use super::operand::{Operand, OperandValue};
use opcodeGen::{RawInst, Opcode_list, ConstraintMap};
use crate::error::{Error, Result};
use crate::sim::constraint::Constraint;
use crate::sim::gen_comment::gen_comment;

#[derive(Debug,Serialize,Clone)]
#[serde(rename_all = "camelCase")]
pub struct Instruction{
    pub(crate) comment: String,
    pub(crate) opcode: RawInst,
    pub(crate) operands: Option<Vec<Operand>>,
    pub(crate) address: u32,
    pub(crate) raw_opcode: u32,
}


impl Instruction{
    pub fn new(comment: String, opcode: RawInst, operands: Vec<Operand>) -> Instruction{
        Instruction{comment,opcode,operands: Some(operands),address:0,raw_opcode:0}
    }
    pub fn decode_from_opcode(opcode: u16,address:u32) -> Result<Instruction>{
        let inst: RawInst = Self::mach_instruction(opcode)?;
        
        Ok(Instruction{
            comment: "".to_string(),
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
                let Rresult = Instruction::map_value(Instruction::map_register_number(Instruction::decode_val(x.map, self.raw_opcode),constraint),constraint);
                let result: OperandValue;
                let mut name:String = "".to_string();
                if( Rresult.is_err()){
                    result = OperandValue::new(1);
                    name = "opcode:".to_string();
                    name+= self.raw_opcode.to_string().as_str();
                    name += &*"error: ".to_string();
                    name += Rresult.err().unwrap().to_string().as_str();
                }else{
                    result = Rresult.unwrap();
                }
                return Ok(Operand{
                    name,
                    constraint,
                    value:result,
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
    fn map_register_number(mut value:u32,constraint: Constraint)->u32{
        match constraint {
            Constraint::d =>{
                value +16
            }
            Constraint::v =>{
                value*2
            }
            Constraint::a =>{
                value +16
            }
            Constraint::w =>{
                value *=2;
                value +24
            }
            Constraint::h =>{
                value*2
            }
            _ =>{
                value
            }
        }
    }
    fn map_value(mut value:u32, constraint: Constraint) ->Result<OperandValue>{ // todo std does not work
        match constraint {
            Constraint::r => {
                if(value<32){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("r: regiter number can`t be higher than 31, got {}", value),address:0})
                }
            }
            Constraint::d => {
                
                if(value >15 && value<32){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("d: ldi regiter number must be between 16 and 31, got {}", value),address:0})
                }
            }
            Constraint::v => {
                if(value<32){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("v: movw regiter number must be even and less 32, got {}", value),address:0})
                }
            }
            Constraint::a => {
                if(value >15 && value<24){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("a: fmul regiter number must be between 16 and 23, got {}", value),address:0})
                }
            }
            Constraint::w => {
                if(value>23 && value<31){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("w: adiw regiter number must 24, 26, 28 or 30, got {}", value),address:0})
                }
            }
            Constraint::e=>{
                if(value<3){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("e: pointer register must be less than 4, got {}", value),address:0})
                }
            }
            Constraint::b => {
                if(value<2){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("b: base pointer register and displacement must be less that 2, got {}", value),address:0})
                }
            }
            Constraint::z =>{
                if(value<2){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("z: Z pointer register must be less than 2, got {}", value),address:0})
                }
            }
            Constraint::M =>{
                if(value<256){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("M: immediate Value must be between from 0 to 255, got {}", value),address:0})
                }
            }
            Constraint::n =>{
                if(value<256){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("n: immediate Value must be between from 0 to 255, got {}", value),address:0})
                }
            }
            Constraint::s=>{
                if(value<8){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("s: immediate Value must be between from 0 to 7, got {}", value),address:0})
                }
            }
            Constraint::P=>{
                if(value<64){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("P: Port address Value must be between from 0 to 63., got {}", value),address:0})
                }
            }
            Constraint::p =>{
                if(value<32){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("p: Port address Value must be between from 0 to 31, got {}", value),address:0})
                }
            }
            Constraint::K=>{
                if(value<64){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("K: immediate Value must be between from 0 to 63, got {}", value),address:0})
                }
            }
            Constraint::i=>{
                if(value>=0){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("M: immediate Value, got {}", value),address:0})
                }
            }
            Constraint::j=>{
                value += 0x40;
                if(value<0xbf){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("j: 7 bit immediate Value that must be between from 0x40 to 0xBF, got {}", value),address:0})
                }
            }
            Constraint::l =>{
                let mut t = unsigned_to_signed(value,7);
                t*=2; // 16 bit wide addresses
                if(t>=-64 && t<64){
                    Ok(OperandValue::new(t))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("l: signed pc relative offset must be between  -64 to 63, got {}", value),address:0})
                }
            }
            Constraint::L =>{
                let mut t = unsigned_to_signed(value,12);
                t*=2; // 16 bit wide addresses
                if(t>=-2048 && t<2048){
                    Ok(OperandValue::new(t))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("l: signed pc relative offset must be between  -2048 to 2047, got {}", value),address:0})
                }
            }
            Constraint::h=>{
                if(value>=0){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("h: absolute code address, got {}", value),address:0})
                }
            }
            Constraint::S=>{
                if(value<8){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("S: immediate Value must be between from 0 to 7, got {}", value),address:0})
                }
            }
            Constraint::E=>{
                if(value<16){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("E: immediate Value must be between from 0 to 15, got {}", value),address:0})
                }
            }
            Constraint::o=>{
                if(value<64){
                    Ok(OperandValue::new(value))
                }
                else{
                    Err(Error::InvalidConstraintValue {err:format!("o: Displacement value must be between 0 and 63, got {}", value),address:0})
                }
            }
        }
    }

    pub(crate) fn gen_comment(&mut self){
        super::gen_comment::gen_comment(self);
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