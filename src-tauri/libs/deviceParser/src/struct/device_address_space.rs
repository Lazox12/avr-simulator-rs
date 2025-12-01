use std::str::FromStr;
use xmltree::Element;
use crate::utils::{find_child, find_childs};

#[derive(Debug)]
pub struct AddressSpace{
    pub memory_segments:Vec<MemorySegment>,
    pub endianess: Endianess,
    pub name:String,
    pub id:String, //todo should be enum
    pub start:u64,
    pub size:u64,
}
impl From<&Element> for AddressSpace {
    fn from(x:&Element) -> Self {
        AddressSpace{
            memory_segments: find_childs(x,"memory-segment".to_string()).into_iter().map(|x| {MemorySegment::from(x)}).collect(),
            endianess: Endianess::from_str(&*x.attributes["endianness"].to_string()).expect(&*x.attributes["endianness"]),
            name: x.attributes["name"].to_string(),
            id: x.attributes["id"].to_string(),
            start: match x.attributes["start"].to_string().starts_with("0x"){true=>{u64::from_str_radix(x.attributes["start"].to_string().strip_prefix("0x").unwrap(),16).unwrap()},false=>{x.attributes["start"].parse::<u64>().unwrap()},},
            size: match x.attributes["size"].to_string().starts_with("0x"){true=>{u64::from_str_radix(x.attributes["size"].to_string().strip_prefix("0x").unwrap(),16).unwrap()},false=>{x.attributes["size"].parse::<u64>().unwrap()},},
        }
    }
}

#[derive(Debug)]
pub enum Endianess{
    Big,
    Little,
}
impl FromStr for Endianess{
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "big" => Ok(Endianess::Big),
            "little" => Ok(Endianess::Little),
            _ => Err(()),
        }
    }
}
#[derive(Debug)]
pub struct MemorySegment{
    pub start:u64,
    pub size:u64,
    pub name:String,
    pub data_type:String, //todo should be enum
    pub access:Option<Access>,
    pub page_size:Option<u64>,
    pub exec:Option<bool>,
    pub external:Option<bool>,
}
impl From<&Element> for MemorySegment {
    fn from(x:&Element) -> Self {
        MemorySegment{
            start: match x.attributes["start"].to_string().starts_with("0x"){true=>{u64::from_str_radix(x.attributes["start"].to_string().strip_prefix("0x").unwrap(),16).unwrap()},false=>{x.attributes["start"].parse::<u64>().unwrap()},},
            size: match x.attributes["size"].to_string().starts_with("0x"){true=>{u64::from_str_radix(x.attributes["size"].to_string().strip_prefix("0x").unwrap(),16).unwrap()},false=>{x.attributes["size"].parse::<u64>().unwrap()},},
            data_type: x.attributes["type"].to_string(),
            access: Access::option_from(x.attributes.get("rw").unwrap_or(&"err".to_string())),
            page_size: x.attributes.get("pagesize").map(|t| {u64::from_str_radix(t.to_string().strip_prefix("0x").unwrap(), 16).unwrap()}),
            exec: x.attributes.get("exec").map(|t| {match t.as_str() {"1"=>{true},"0"=>{false} _ => {panic!("err1")} }}),
            external: x.attributes.get("external").map(|t| {match t.as_str() {"true"=>{true},"false"=>{false} _ => {panic!("err")} }}),
            name:x.attributes["name"].to_string(),
        }
    }
}
#[derive(Debug)]
pub enum Access{
    R,
    RW,
}
impl Access{
    fn option_from(s: &str) -> Option<Self> {
        match s { 
            "R" => Some(Access::R),
            "RW" => Some(Access::RW),
            _ => None,
        }
    }
}