
use std::num::ParseIntError;
use strum::ParseError;
use std::backtrace::Backtrace;
use crate::sim::constraint::Constraint::a;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug,thiserror::Error)]
pub enum Error {

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
    
}

