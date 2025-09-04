use serde::Serialize;
use strum::{EnumIter, EnumString};

#[derive(Debug, EnumIter, EnumString, Clone, Copy,Serialize)]
pub enum Display {
    None,
    Bin,
    Dec,
    Oct,
    Hex,
    String,
}