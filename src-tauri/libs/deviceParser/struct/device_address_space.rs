use std::str::FromStr;
use proc_macro2::{ Span, TokenStream};
use quote::{quote, ToTokens};
use xmltree::Element;
use super::utils::{find_child, find_childs};
use syn::Ident;
#[derive(Debug)]
pub struct AddressSpace{
    pub memory_segments:&'static [MemorySegment],
    pub endianess: Endianess,
    pub name:&'static str,
    pub id:&'static str, //todo should be enum
    pub start:u64,
    pub size:u64,
}
impl From<&'static Element> for AddressSpace {
    fn from(x:&'static Element) -> Self {
        AddressSpace{
            memory_segments: Box::leak(find_childs(x,"memory-segment").into_iter().map(|x| {MemorySegment::from(x)}).collect::<Vec<MemorySegment>>().into_boxed_slice()),
            endianess: Endianess::from_str(&*x.attributes["endianness"]).expect(&*x.attributes["endianness"]),
            name: &x.attributes["name"],
            id: &x.attributes["id"],
            start: match x.attributes["start"].starts_with("0x"){true=>{u64::from_str_radix(x.attributes["start"].strip_prefix("0x").unwrap(),16).unwrap()},false=>{x.attributes["start"].parse::<u64>().unwrap()},},
            size: match x.attributes["size"].starts_with("0x"){true=>{u64::from_str_radix(x.attributes["size"].strip_prefix("0x").unwrap(),16).unwrap()},false=>{x.attributes["size"].parse::<u64>().unwrap()},},
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
    pub name:&'static str,
    pub data_type:&'static str, //todo should be enum
    pub access:Option<Access>,
    pub page_size:Option<u64>,
    pub exec:Option<bool>,
    pub external:Option<bool>,
}
impl From<&'static Element> for MemorySegment {
    fn from(x:&'static Element) -> Self {
        MemorySegment{
            start: match x.attributes["start"].starts_with("0x"){true=>{u64::from_str_radix(x.attributes["start"].strip_prefix("0x").unwrap(),16).unwrap()},false=>{x.attributes["start"].parse::<u64>().unwrap()},},
            size: match x.attributes["size"].starts_with("0x"){true=>{u64::from_str_radix(x.attributes["size"].strip_prefix("0x").unwrap(),16).unwrap()},false=>{x.attributes["size"].parse::<u64>().unwrap()},},
            data_type: &x.attributes["type"],
            access: Access::option_from(x.attributes.get("rw").unwrap_or(&"err".to_string())),
            page_size: x.attributes.get("pagesize").map(|t| {u64::from_str_radix(t.strip_prefix("0x").unwrap(), 16).unwrap()}),
            exec: x.attributes.get("exec").map(|t| {match t.as_str() {"1"=>{true},"0"=>{false} _ => {panic!("err1")} }}),
            external: x.attributes.get("external").map(|t| {match t.as_str() {"true"=>{true},"false"=>{false} _ => {panic!("err")} }}),
            name:&x.attributes["name"],
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
impl ToTokens for AddressSpace {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let memory_segments = &self.memory_segments;
        let endianess = &self.endianess;
        let name = &self.name;
        let id = &self.id;
        let start = self.start;
        let size = self.size;

        tokens.extend(quote! {
            crate::r#struct::device_address_space::AddressSpace {
                memory_segments: &[#( #memory_segments ),*],
                endianess: #endianess,
                name: #name,
                id: #id,
                start: #start,
                size: #size,
            }
        });
    }
}

impl ToTokens for Endianess {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Endianess::Big => tokens.extend(quote! { crate::r#struct::device_address_space::Endianess::Big }),
            Endianess::Little => tokens.extend(quote! { crate::r#struct::device_address_space::Endianess::Little }),
        }
    }
}

impl ToTokens for MemorySegment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let start = self.start;
        let size = self.size;
        let name = &self.name;
        let data_type = &self.data_type;
        let access = match &self.access {
            Some(a) => quote! { Some(#a) },
            None => quote! { None },
        };
        let page_size = match self.page_size {
            Some(p) => quote! { Some(#p) },
            None => quote! { None },
        };
        let exec = match self.exec {
            Some(e) => quote! { Some(#e) },
            None => quote! { None },
        };
        let external = match self.external {
            Some(e) => quote! { Some(#e) },
            None => quote! { None },
        };

        tokens.extend(quote! {
            crate::r#struct::device_address_space::MemorySegment {
                start: #start,
                size: #size,
                name: #name,
                data_type: #data_type,
                access: #access,
                page_size: #page_size,
                exec: #exec,
                external: #external,
            }
        });
    }
}

impl ToTokens for Access {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Access::R => tokens.extend(quote! { crate::r#struct::device_address_space::Access::R }),
            Access::RW => tokens.extend(quote! { crate::r#struct::device_address_space::Access::RW }),
        }
    }
}