use std::str::FromStr;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumString};

#[derive(Debug, EnumIter, EnumString, Clone, Copy,Serialize,Deserialize,Default)]
pub enum Display {
    #[default]
    None,
    Bin,
    Dec,
    Oct,
    Hex,
    String,
}
impl Display {
    pub fn decode(s:&str)->Display{
        if s.is_empty()  { Display::None }
        else if s.starts_with("0b") || s.starts_with("0B")  {
            Display::Bin
        }
        else if u32::from_str(&*s).is_ok()  {
            Display::Dec
        }
        else if s.starts_with("0c") || s.starts_with("0C")  {
            Display::Oct
        }
        else if s.starts_with("0x") || s.starts_with("0X")  {
            Display::Hex
        }else{
            Display::String
        }
    }
}