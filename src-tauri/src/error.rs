
use std::num::ParseIntError;
use std::sync::PoisonError;
use anyhow::anyhow;
use serde::{Serialize, Serializer};
use strum::ParseError;
use crate::error::Error::ProjectError;

pub type Result<T> = anyhow::Result<T>;

#[derive(Debug,thiserror::Error)]
pub enum Error{

    #[error("OpcodeNotFound found: {opcode:?}")]
    OpcodeNotFound {opcode:u32},

    #[error("Invalid input into Conversion expected: {err_val} , got:{val}",val = expected_val.join(","))]
    InvalidConversion { err_val:String, expected_val:Vec<String> },

    #[error("invalid read: trying to read {current} from {expected}")]
    InvalidReadError{current:String, expected:String},
    
    #[error("constraint Requirements not met: {err} at address {:#x}",address)]
    InvalidConstraintValue{err:String, address:u32},

    #[error("function not implemented: {err}")]
    NotImplemented{err:String},
    
    #[error("invalid record type must be between 0 and 5 instead got: {err}")]
    InvalidRecordType{err:String},

    #[error("Failed to parse int:{0}")]
    ParseInt(#[from] ParseIntError),
    
    #[error("Failed to parse str:{0}")]
    ParseError(#[from] ParseError),
    
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    
    
    #[error("SQLite error: {0}")]
    SQLError(#[from] rusqlite::Error),
    
    #[error("FileExists: {0}")]
    FileExists(String),
    
    #[error("ProjectAlreadyOpened")]
    ProjectAlreadyOpened,

    #[error("ProjectNotOpened")]
    ProjectNotOpened,
    
    #[error("SerdeError {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Invalid Instruction Name {0}")]
    InvalidInstructionName(u32),
    
    #[error("Invaild operand count expected:{expected}, instead got:{got}")]
    InvalidOperandCount{expected:usize, got:usize},
    
    #[error("Invalid Value")]
    InvalidValue,
    
    #[error("Invalid mcu: {0}")]
    InvalidMcu(String),

    #[error("Lock poisoned: {0}")]
    Poison(String),
    
    #[error("Tauri error: {0}")]
    TauriError(#[from] tauri::Error),
    
    #[error("xml tree error: {0}")]
    XmlTreeError(#[from] deviceParser::Error),
    
    #[error("Project Handler Error: {0}")]
    ProjectError(&'static str)
}

impl<T> From<PoisonError<T>> for Error {
    fn from(err: PoisonError<T>) -> Self {
        Error::Poison(err.to_string())
    }
}
impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}
