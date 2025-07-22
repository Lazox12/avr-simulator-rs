use std::fmt::{write, Display, Formatter, LowerHex};
use crate::error::{Error, Result};
use super::constraint::Constraint;
pub struct Operand{
    pub(crate) name: String,
    pub(crate) constraint:Constraint,
    pub(crate) value: OperandValue,
}

impl Display for Operand{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"name:{},constraint:{:?},val:{:#x}",self.name,self.constraint,self.value)
    }
}

#[repr(C)]
pub union OperandValueType {
    u16:u16,
    u32:u32,
    i16:i16,
    i32:i32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandValueKind{
    u16,
    u32,
    i16,
    i32
}
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