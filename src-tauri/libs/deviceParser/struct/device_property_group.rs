use std::str::FromStr;
use proc_macro2::{ Span, TokenStream};
use quote::{quote, ToTokens};
use xmltree::Element;
use crate::utils::find_childs;
use syn::Ident;
#[derive(Debug)]
pub struct PropertyGroup{
    pub name: String,
    pub properties: Vec<Property>,
}
impl From<&Element> for PropertyGroup{
    fn from(element: &Element) -> PropertyGroup{
        PropertyGroup{
            name: element.attributes["name"].to_string(),
            properties: find_childs(element,"property".to_string()).into_iter().map(|x| {Property::from(x)}).collect(),
        }
    }
}
#[derive(Debug)]
pub struct Property {
    name: String,
    value: PropertyValue,
}
impl From<&Element> for Property{
    fn from(x: &Element) -> Self{
        Property{
            name: x.attributes["name"].to_string(),
            value: PropertyValue::from(&x.attributes["value"]),
        }
    }
}

#[derive(Debug)]
pub enum PropertyValue {
    Number(u64),
    Vec(Vec<u64>),
    String(String),
}
impl From<&String> for PropertyValue{
    fn from(x: &String) -> Self{
        match x.strip_prefix("0x") { 
            Some(v) => {match u64::from_str_radix(v,16) {
                Ok(v) => PropertyValue::Number(v),
                Err(_)=> PropertyValue::Vec(x.split(" ").into_iter().map(|x| u64::from_str_radix(x.strip_prefix("0x").unwrap(),16).unwrap()).collect())
            }},
            None => match u64::from_str(x){
                Ok(v) => PropertyValue::Number(v),
                Err(_)=> PropertyValue::String(x.to_string())
            },
        }
    }
}
impl ToTokens for PropertyGroup {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let properties = &self.properties;

        tokens.extend(quote! {
            crate::r#struct::device_property_group::PropertyGroup {
                name: #name.to_string(),
                properties: vec![#( #properties ),*],
            }
        });
    }
}

impl ToTokens for Property {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let value = &self.value;

        tokens.extend(quote! {
            crate::r#struct::device_property_group::Property {
                name: #name.to_string(),
                value: #value,
            }
        });
    }
}

impl ToTokens for PropertyValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            PropertyValue::Number(n) => tokens.extend(quote! { crate::r#struct::device_property_group::PropertyValue::Number(#n) }),
            PropertyValue::Vec(v) => tokens.extend(quote! { crate::r#struct::device_property_group::PropertyValue::Vec(vec![#( #v ),*]) }),
            PropertyValue::String(s) => tokens.extend(quote! { crate::r#struct::device_property_group::PropertyValue::String(#s.to_string()) }),
        }
    }
}