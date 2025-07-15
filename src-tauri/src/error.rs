use std::num::ParseIntError;
use derive_more::From;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug,From)]
pub enum Error {
    OpcodeNotFound {opcode:String},
    

    #[from]
    Custom(String),

    #[from]
    Io(std::io::Error),
    #[from]
    Num(ParseIntError),
}

impl Error {
    pub fn custom(val:impl std::fmt::Display) -> Self {
        Self::Custom(val.to_string())
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
