use std::fmt;
use std::fmt::{write, Display, Formatter, LowerHex};
use std::str::FromStr;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use crate::error::{Error, Result};
use super::constraint::Constraint;

#[derive(Debug,Serialize,Deserialize,Clone)]
#[serde(rename_all = "camelCase")]
pub struct Operand{
    pub(crate) name: String,
    pub(crate) constraint:Constraint,
    pub(crate) value: OperandValue,
    pub(crate) operand_info: Option<OperandInfo>,
}
impl Operand{
    pub(crate) fn map_value(mut value:u32, constraint: Constraint) ->Result<OperandValue>{ // todo std does not work
        match constraint {
            Constraint::r => {
                if(value<32){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("r: regiter number can`t be higher than 31, got {}", value),address:0}))
                }
            }
            Constraint::d => {

                if(value >15 && value<32){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("d: ldi regiter number must be between 16 and 31, got {}", value),address:0}))
                }
            }
            Constraint::v => {
                if(value<32){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("v: movw regiter number must be even and less 32, got {}", value),address:0}))
                }
            }
            Constraint::a => {
                if(value >15 && value<24){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("a: fmul regiter number must be between 16 and 23, got {}", value),address:0}))
                }
            }
            Constraint::w => {
                if(value>23 && value<31){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("w: adiw regiter number must 24, 26, 28 or 30, got {}", value),address:0}))
                }
            }
            Constraint::e=>{
                if(value<4){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("e: pointer register must be less than 4, got {}", value),address:0}))
                }
            }
            Constraint::b => {
                if(value<2){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("b: base pointer register and displacement must be less that 2, got {}", value),address:0}))
                }
            }
            Constraint::z =>{
                if(value<2){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("z: Z pointer register must be less than 2, got {}", value),address:0}))
                }
            }
            Constraint::M =>{
                if(value<256){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("M: immediate Value must be between from 0 to 255, got {}", value),address:0}))
                }
            }
            Constraint::n =>{
                if(value<256){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("n: immediate Value must be between from 0 to 255, got {}", value),address:0}))
                }
            }
            Constraint::s=>{
                if(value<8){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("s: immediate Value must be between from 0 to 7, got {}", value),address:0}))
                }
            }
            Constraint::P=>{
                if(value<64){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("P: Port address Value must be between from 0 to 63., got {}", value),address:0}))
                }
            }
            Constraint::p =>{
                if(value<32){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("p: Port address Value must be between from 0 to 31, got {}", value),address:0}))
                }
            }
            Constraint::K=>{
                if(value<64){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("K: immediate Value must be between from 0 to 63, got {}", value),address:0}))
                }
            }
            Constraint::i=>{
                if(value>=0){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("M: immediate Value, got {}", value),address:0}))
                }
            }
            Constraint::j=>{
                value += 0x40;
                if(value<0xbf){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("j: 7 bit immediate Value that must be between from 0x40 to 0xBF, got {}", value),address:0}))
                }
            }
            Constraint::l =>{
                let mut t = Operand::unsigned_to_signed(value, 7);
                t*=2; // 16 bit wide addresses
                if(t>=-64 && t<64){
                    Ok(t as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("l: signed pc relative offset must be between  -64 to 63, got {}", value),address:0}))
                }
            }
            Constraint::L =>{
                let mut t = Operand::unsigned_to_signed(value, 12);
                t*=2; // 16 bit wide addresses
                if(t>=-2048 && t<2048){
                    Ok(t as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("l: signed pc relative offset must be between  -2048 to 2047, got {}", value),address:0}))
                }
            }
            Constraint::h=>{
                if(value>=0){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("h: absolute code address, got {}", value),address:0}))
                }
            }
            Constraint::S=>{
                if(value<8){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("S: immediate Value must be between from 0 to 7, got {}", value),address:0}))
                }
            }
            Constraint::E=>{
                if(value<16){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("E: immediate Value must be between from 0 to 15, got {}", value),address:0}))
                }
            }
            Constraint::o=>{
                if(value<64){
                    Ok(value as OperandValue)
                }
                else{
                    Err(anyhow!(Error::InvalidConstraintValue {err:format!("o: Displacement value must be between 0 and 63, got {}", value),address:0}))
                }
            }
        }
    }
    pub(crate) fn map_value_from_string(value:&String,constraint: Constraint) -> Result<OperandValue> {
        let mut value =value.clone();
        match constraint {
            Constraint::r|
            Constraint::d|
            Constraint::v|
            Constraint::a|
            Constraint::w => {
                value.remove(0);
                Ok(OperandValue::from_str(&*value)?)
            }
            Constraint::e => {pointer_into_constraint_e(&*value)}
            Constraint::b => {pointer_into_constraint_b(&*value)}
            Constraint::z => {pointer_into_constraint_z(&*value)}
            Constraint::M| 
            Constraint::n|
            Constraint::s|
            Constraint::P|
            Constraint::p|
            Constraint::K|
            Constraint::i|
            Constraint::h|
            Constraint::S|
            Constraint::E|
            Constraint::j=> {
                Ok(OperandValue::from_str_radix(value.trim_start_matches("0X"), 16)?)
            }
            Constraint::l|
            Constraint::L => {
                value.remove(0);
                Ok(OperandValue::from_str(&*value)?)
            }
            Constraint::o => {
                value.remove(0);
                Ok(OperandValue::from_str(&*value)?)
            }
        }
    }

    pub(crate) fn map_string_from_value(&self) -> Result<String> {
        let value = self.value.clone();
        match self.constraint {
            Constraint::r =>{Ok(format!("r{}",value))}
            Constraint::d =>{Ok(format!("r{}",value))}
            Constraint::v =>{Ok(format!("r{}",value))}
            Constraint::a =>{Ok(format!("r{}",value))}
            Constraint::w =>{Ok(format!("r{}",value))}
            Constraint::e =>{Ok(format!("{}",constraint_e_into_pointer(self.value)?))}
            Constraint::b =>{Ok(format!("{}",constraint_b_into_pointer(self.value)?))}
            Constraint::z =>{Ok(format!("{}",constraint_z_into_pointer(self.value)?))}
            Constraint::M =>{Ok(format!("{:#x}",value))}
            Constraint::n =>{Ok(format!("{:#x}",value))}
            Constraint::s =>{Ok(format!("{:#x}",value))}
            Constraint::P =>{Ok(format!("{:#x}",value))}
            Constraint::p =>{Ok(format!("{:#x}",value))}
            Constraint::K =>{Ok(format!("{:#x}",value))}
            Constraint::i =>{Ok(format!("{:#x}",value))}
            Constraint::j =>{Ok(format!("{:#x}",value))}
            Constraint::l =>{Ok(format!(".{}",value))}
            Constraint::L =>{Ok(format!(".{}",value))}
            Constraint::h =>{Ok(format!("{:#x}",value))}
            Constraint::S =>{Ok(format!("{:#x}",value))}
            Constraint::E =>{Ok(format!("{:#x}",value))}
            Constraint::o =>{Ok(format!("+{}",value))}
        }
    }
    fn unsigned_to_signed(val:u32,len:u32)->i32{ //signed len in bits
        if(val>>len-1)==0{ // positive number
            val as i32
        }else{ //negative number
            ((1<<len) *-1) +(val as i32)
        }
    }
}

impl Display for Operand{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        //write!(f,"name:{},constraint:{:?},val:{:#x}",self.name,self.constraint,self.value);
        if(self.name.len()>0){
            write!(f,"{}",self.name)?;
        }
        match self.constraint {
            Constraint::r =>{write!(f,"r{}",self.value)}
            Constraint::d =>{write!(f,"r{}",self.value)}
            Constraint::v =>{write!(f,"r{}",self.value)}
            Constraint::a =>{write!(f,"r{}",self.value)}
            Constraint::w =>{write!(f,"r{}",self.value)}
            Constraint::e =>{write!(f,"{}",constraint_e_into_pointer(self.value).map_err(|_| {fmt::Error})?)}
            Constraint::b =>{write!(f,"{}",constraint_b_into_pointer(self.value).map_err(|_| {fmt::Error})?)}
            Constraint::z =>{write!(f,"{}",constraint_z_into_pointer(self.value).map_err(|_| {fmt::Error})?)}
            Constraint::M =>{write!(f,"{:#x}",self.value)}
            Constraint::n =>{write!(f,"{:#x}",self.value)}
            Constraint::s =>{write!(f,"{:#x}",self.value)}
            Constraint::P =>{write!(f,"{:#x}",self.value)}
            Constraint::p =>{write!(f,"{:#x}",self.value)}
            Constraint::K =>{write!(f,"{:#x}",self.value)}
            Constraint::i =>{write!(f,"{:#x}",self.value)}
            Constraint::j =>{write!(f,"{:#x}",self.value)}
            Constraint::l =>{write!(f,".{}",self.value)}
            Constraint::L =>{write!(f,".{}",self.value)}
            Constraint::h =>{write!(f,"{:#x}",self.value)}
            Constraint::S =>{write!(f,"{:#x}",self.value)}
            Constraint::E =>{write!(f,"{:#x}",self.value)}
            Constraint::o =>{write!(f,"+{}",self.value)}
        }
    }
}


fn constraint_e_into_pointer(val:OperandValue) ->Result<String>{
    match val {
        3 => Ok(String::from('X')),
        2 => Ok(String::from('Y')),
        0 => Ok(String::from('Z')),
        _ => Ok(String::from("Invalid Value")),
    }
}
fn constraint_b_into_pointer(val:OperandValue) ->Result<String>{
    match val {
        0 => Ok(String::from('Z')),
        1 => Ok(String::from('Y')),
        _ => Ok(String::from("Invalid Value")),
    }
}
fn constraint_z_into_pointer(val:OperandValue) ->Result<String>{
    match val {
        0 => Ok(String::new()),
        1 => Ok(String::from("Z+")),
        _ => Ok(String::from("Invalid Value")),
    }
}

fn pointer_into_constraint_e(val:&str) ->Result<OperandValue>{
    if(val.len()==0){
        return Err(anyhow!(Error::InvalidValue))
    }

    match val.to_uppercase().chars().next().unwrap() {
        'X' => Ok(3),
        'Y' => Ok(2),
        'Z' => Ok(0),
        _ => Err(anyhow!(Error::InvalidValue)),
    }
}
fn pointer_into_constraint_b(val:&str) ->Result<OperandValue>{
    if(val.len()==0){
        return Err(anyhow!(Error::InvalidValue))
    }

    match val.to_uppercase().as_str() {
        "" => Ok(0),
        "Z+" => Ok(1),
        _ => Err(anyhow!(Error::InvalidValue)),
    }
}
fn pointer_into_constraint_z(val:&str) ->Result<OperandValue>{
    if(val.len()==0){
        return Err(anyhow!(Error::InvalidValue))
    }

    match val.to_uppercase().chars().next().unwrap() {
        'X' => Ok(3),
        'Y' => Ok(2),
        'Z' => Ok(0),
        _ => Err(anyhow!(Error::InvalidValue)),
    }
}

pub type OperandValue = i64;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_unsigned_to_signed(){
        assert_eq!(Operand::unsigned_to_signed(0b010101101110,12),1390);
        assert_eq!(Operand::unsigned_to_signed(0b1001010111101110,16),-27154);
        assert_eq!(Operand::unsigned_to_signed(0b0,14),0);
        assert_eq!(Operand::unsigned_to_signed(0b1111,4),-1);

    }
}

#[derive(Debug, Default, Clone, PartialEq,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperandInfo {
    pub register_name:String,
    pub register_mask:String,
    pub description:String,
}