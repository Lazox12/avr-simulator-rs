use std::fmt;
use std::fmt::{write, Display, Formatter, LowerHex};
use crate::error::{Error, Result};
use super::constraint::Constraint;
#[derive(Debug)]
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


#[repr(C)]

pub union OperandValueType {
    u16:u16,
    u32:u32,
    i16:i16,
    i32:i32,
}

impl fmt::Debug for OperandValueType{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{:?}",self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandValueKind{
    u16,
    u32,
    i16,
    i32
}
#[derive(Debug)]
pub struct OperandValue{
    kind:OperandValueKind,
    value:OperandValueType,
}
pub trait OperandType: Sized {
    const KIND: OperandValueKind;
    fn to_union(val: Self) -> OperandValueType;
    unsafe fn read_from(value: &OperandValueType) -> Self;
}

impl OperandType for u16 {
    const KIND: OperandValueKind = OperandValueKind::u16;
    fn to_union(val: Self) -> OperandValueType {
        OperandValueType { u16: val }
    }
    unsafe fn read_from(value: &OperandValueType) -> Self {
        value.u16
    }
}

impl OperandType for u32 {
    const KIND: OperandValueKind = OperandValueKind::u32;
    fn to_union(val: Self) -> OperandValueType {
        OperandValueType { u32: val }
    }
    unsafe fn read_from(value: &OperandValueType) -> Self {
        value.u32
    }
}

impl OperandType for i16 {
    const KIND: OperandValueKind = OperandValueKind::i16;
    fn to_union(val: Self) -> OperandValueType {
        OperandValueType { i16: val }
    }
    unsafe fn read_from(value: &OperandValueType) -> Self {
        value.i16
    }
}

impl OperandType for i32 {
    const KIND: OperandValueKind = OperandValueKind::i32;
    fn to_union(val: Self) -> OperandValueType {
        OperandValueType { i32: val }
    }
    unsafe fn read_from(value: &OperandValueType) -> Self {
        value.i32
    }
}

impl OperandValue {
    pub fn new<T: OperandType>(val: T) -> Self {
        OperandValue {
            kind: T::KIND,
            value: T::to_union(val),
        }
    }
    pub fn read<T: OperandType>(&self) -> Result<T> {
        if self.kind == T::KIND {
            unsafe { Ok(T::read_from(&self.value)) }
        } else {
            Err(Error::InvalidReadError{expected:self.kind.try_into()?,current:T::KIND.try_into()?})
        }
    }
}
impl LowerHex for OperandValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            OperandValueKind::u16 => {write!(f, "{:#x}", self.read::<u16>().map_err(|_| std::fmt::Error)?)},
            OperandValueKind::u32 => {write!(f, "{:#x}", self.read::<u32>().map_err(|_| std::fmt::Error)?)},
            OperandValueKind::i16 => {write!(f, "{:#x}", self.read::<i16>().map_err(|_| std::fmt::Error)?)},
            OperandValueKind::i32 => {write!(f, "{:#x}", self.read::<i32>().map_err(|_| std::fmt::Error)?)},
        }
    }
}
impl Display for OperandValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            OperandValueKind::u16 => {write!(f, "{}", self.read::<u16>().map_err(|_| std::fmt::Error)?)},
            OperandValueKind::u32 => {write!(f, "{}", self.read::<u32>().map_err(|_| std::fmt::Error)?)},
            OperandValueKind::i16 => {write!(f, "{}", self.read::<i16>().map_err(|_| std::fmt::Error)?)},
            OperandValueKind::i32 => {write!(f, "{}", self.read::<i32>().map_err(|_| std::fmt::Error)?)},
        }
    }
}
impl TryInto<String> for OperandValueKind {
    type Error = Error;

    fn try_into(self) -> std::result::Result<String, Self::Error> {
        match self {
            OperandValueKind::u16 => {Ok(String::from("u16"))}
            OperandValueKind::u32 => {Ok(String::from("u32"))}
            OperandValueKind::i16 => {Ok(String::from("i16"))}
            OperandValueKind::i32 => {Ok(String::from("i32"))}
        }
    }
}