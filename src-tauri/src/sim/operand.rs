use std::fmt;
use std::fmt::{write, Display, Formatter, LowerHex};
use serde::{Serialize, Serializer};
use strum::Display;
use crate::error::{Error, Result};
use super::constraint::Constraint;
#[derive(Debug,Serialize,Clone)]
#[serde(rename_all = "camelCase")]
pub struct Operand{
    pub(crate) name: String,
    pub(crate) constraint:Constraint,
    pub(crate) value: OperandValue,
}

impl Display for Operand{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        //write!(f,"name:{},constraint:{:?},val:{:#x}",self.name,self.constraint,self.value);
        match self.constraint {
            Constraint::r =>{write!(f,"r{}",self.value)}
            Constraint::d =>{write!(f,"r{}",self.value)}
            Constraint::v =>{write!(f,"r{}",self.value)}
            Constraint::a =>{write!(f,"r{}",self.value)}
            Constraint::w =>{write!(f,"r{}",self.value)}
            Constraint::e =>{write!(f,"{}",constraint_e_into_pointer(&self.value).map_err(|_| {fmt::Error})?)}
            Constraint::b =>{write!(f,"{}",constraint_b_into_pointer(&self.value).map_err(|_| {fmt::Error})?)}
            Constraint::z =>{write!(f,"{}",constraint_z_into_pointer(&self.value).map_err(|_| {fmt::Error})?)}
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

fn constraint_e_into_pointer(val:&OperandValue) ->Result<String>{
    match val.read::<u32>()? {
        3 => Ok(String::from('X')),
        2 => Ok(String::from('Y')),
        0 => Ok(String::from('Z')),
        _ => Ok(String::from("Invalid Value")),
    }
}
fn constraint_b_into_pointer(val:&OperandValue) ->Result<String>{
    match val.read::<u32>()? {
        0 => Ok(String::from('Z')),
        1 => Ok(String::from('Y')),
        _ => Ok(String::from("Invalid Value")),
    }
}
fn constraint_z_into_pointer(val:&OperandValue) ->Result<String>{
    match val.read::<u32>()? {
        0 => Ok(String::new()),
        1 => Ok(String::from("Z+")),
        _ => Ok(String::from("Invalid Value")),
    }
}

#[derive(Debug, Clone)]
pub struct  OperandValue{
    inner:IntValue,
}

#[derive(Debug,Clone,Copy)]
pub enum IntValue{
    Unsigned(u32),
    Signed(i32),
}
impl Serialize for OperandValue{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        match self.inner { 
            IntValue::Unsigned(v) => serializer.serialize_u32(v),
            IntValue::Signed(v) => serializer.serialize_i32 (v),
        }
    }
}
impl Display for OperandValue{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.inner {
            IntValue::Unsigned(_) => {
                write!(f,"{}",self.read::<u32>().map_err(|_| std::fmt::Error)?)
            }
            IntValue::Signed(_) => {
                write!(f,"{}",self.read::<i32>().map_err(|_| std::fmt::Error)?)
            }
        }
    }
}
impl LowerHex for OperandValue{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.inner {
            IntValue::Unsigned(_) => {
                write!(f,"{:#x}",self.read::<u32>().map_err(|_| std::fmt::Error)?)
            }
            IntValue::Signed(_) => {
                write!(f,"{:#x}",self.read::<i32>().map_err(|_| std::fmt::Error)?)
            }
        }
    }
}

impl OperandValue {
    pub fn new<T>(val: T) -> Self
    where
        T: Into<IntValue>,
    {
        OperandValue { inner: val.into() }
    }

    pub fn read<T>(&self) -> Result<T>
    where
        IntValue: TryInto<T>,
    {
        match self.inner.clone().try_into() {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::InvalidReadError { current: "".to_string(), expected: "".to_string() }),
        }
    }
}

impl From<u32> for IntValue {
    fn from(v: u32) -> Self {
        IntValue::Unsigned(v)
    }
}

impl From<i32> for IntValue {
    fn from(v: i32) -> Self {
        IntValue::Signed(v)
    }
}

impl TryFrom<IntValue> for u32 {
    type Error = ();
    fn try_from(v: IntValue) -> std::result::Result<u32, ()> {
        match v {
            IntValue::Unsigned(x) => Ok(x),
            _ => Err(()),
        }
    }
}

impl TryFrom<IntValue> for i32 {
    type Error = ();
    fn try_from(v: IntValue) -> std::result::Result<i32, ()> {
        match v {
            IntValue::Signed(x) => Ok(x),
            _ => Err(()),
        }
    }
}