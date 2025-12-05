use proc_macro2::Ident;
use quote::__private::{Span, TokenStream};
use quote::{quote, ToTokens};
use xmltree::Element;
use serde::Serialize;
use crate::r#struct::device_property_group::PropertyValue;
use super::utils::{find_childs, to_ident};

#[derive(Debug)]
pub struct Module{
    pub caption:Option<&'static str>,
    pub name: &'static str,
    pub register_group: Vec<ModuleRegisterGroup>,
    pub value_grop: Vec<ValueGroup>
}
impl From<&'static Element> for Module{
    fn from(x:&'static Element) -> Self{
        Module{
            caption: x.attributes.get("caption").map(|t| t.as_str()),
            name: &x.attributes["name"],
            register_group: find_childs(x,"register-group").into_iter().map(|x1| {ModuleRegisterGroup::from(x1)}).collect(),
            value_grop: find_childs(x,"value-group").into_iter().map(|x1| {ValueGroup::from(x1)}).collect(),
        }
    }
}
#[derive(Debug)]
pub struct ModuleRegisterGroup{
    pub caption:Option<&'static str>,
    pub name: &'static str,
    pub register: Vec<Register>
}
impl From<&'static Element> for ModuleRegisterGroup{
    fn from(x:&'static Element) -> Self{
        ModuleRegisterGroup{
            caption: x.attributes.get("caption").map(|x1| x1.as_str()),
            name: &x.attributes["name"],
            register: find_childs(x,"register").into_iter().map(|x1| {Register::from(x1)}).collect(),
        }
    }
}
#[derive(Debug,Default,Serialize,Clone)]
pub struct Register{
    pub caption:Option<&'static str>,
    pub name: &'static str,
    pub offset: u64,
    pub size: u64,
    pub initval:u64,
    pub bitfields:Option<Vec<BitField>>,
}
impl From<&'static Element> for Register{
    fn from(x:&'static Element) -> Self{
        Register{
            caption:x.attributes.get("caption").map(|x1| x1.as_str()),
            name: &x.attributes["name"],
            offset: u64::from_str_radix(x.attributes["offset"].strip_prefix("0x").unwrap(), 16).unwrap(),
            size: x.attributes["size"].parse().unwrap(),
            initval: u64::from_str_radix(x.attributes["offset"].strip_prefix("0x").unwrap(), 16).unwrap(),
            bitfields: Some(find_childs(x,"bitfield").into_iter().map(|x1| {BitField::from(x1)}).collect()),
        }
    }
}
#[derive(Debug,Serialize,Clone,Default)]
#[serde(rename_all = "camelCase")]
pub struct BitField{
    pub caption: Option<&'static str>,
    pub mask: u64,
    pub name: &'static str,
    pub values:Option<&'static str>,
}
impl From<&'static Element> for BitField{
    fn from(x:&'static Element) -> Self{
        BitField{
            caption: x.attributes.get("caption").map(|x1| x1.as_str()),
            mask: u64::from_str_radix(x.attributes["mask"].strip_prefix("0x").unwrap(), 16).unwrap(),
            name: &x.attributes["name"],
            values: x.attributes.get("name").map(|x1| x1.as_str()),
        }
    }
}
#[derive(Debug)]
pub struct ValueGroup{
    pub name: &'static str,
    pub values: Vec<Value>
}
impl From<&'static Element> for ValueGroup{
    fn from(x:&'static Element) -> Self{
        ValueGroup{ 
            name: &x.attributes["name"],
            values: find_childs(x,"value").into_iter().map(|x1| {Value::from(x1)}).collect()
        }
    }
}
#[derive(Debug)]
pub struct Value{
    caption: &'static str,
    name: &'static str,
    value: PropertyValue,
}
impl From<&'static Element> for Value{
    fn from(x:&'static Element) -> Self{
        Value{
            caption: &x.attributes["caption"],
            name: &x.attributes["name"],
            value: PropertyValue::from(&x.attributes["value"]),
        }
    }
}
impl ToTokens for Module {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let register_group = &self.register_group;
        let value_group = &self.value_grop;

        let caption = match &self.caption {
            Some(c) => quote! { Some(#c) },
            None => quote! { None },
        };

        // fully qualified path: crate::r#struct::module::Module
        tokens.extend(quote! {
            crate::r#struct::module::Module {
                caption: #caption,
                name: #name,
                register_group: vec![#( #register_group ),*],
                value_grop: vec![#( #value_group ),*],
            }
        });
    }
}

impl ToTokens for ModuleRegisterGroup {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let register = &self.register;
        let caption = match &self.caption {
            Some(c) => quote! { Some(#c) },
            None => quote! { None },
        };

        tokens.extend(quote! {
            crate::r#struct::module::ModuleRegisterGroup {
                caption: #caption,
                name: #name,
                register: vec![#( #register ),*],
            }
        });
    }
}

impl ToTokens for Register {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let offset = self.offset;
        let size = self.size;
        let initval = self.initval;

        let caption = match &self.caption {
            Some(c) => quote! { Some(#c) },
            None => quote! { None },
        };
        let bitfields = match &self.bitfields {
            Some(b) => quote! { Some(vec![#( #b ),*]) },
            None => quote! { None },
        };

        tokens.extend(quote! {
            crate::r#struct::module::Register {
                caption: #caption,
                name: #name,
                offset: #offset,
                size: #size,
                initval: #initval,
                bitfields: #bitfields,
            }
        });
    }
}

impl ToTokens for BitField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mask = self.mask;
        let name = &self.name;
        let caption = match &self.caption {
            Some(c) => quote! { Some(#c) },
            None => quote! { None },
        };
        let values = match &self.values {
            Some(v) => quote! { Some(#v) },
            None => quote! { None },
        };

        tokens.extend(quote! {
            crate::r#struct::module::BitField {
                caption: #caption,
                mask: #mask,
                name: #name,
                values: #values,
            }
        });
    }
}

impl ToTokens for ValueGroup {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let values = &self.values;
        tokens.extend(quote!{
            crate::r#struct::module::ValueGroup {
                name: #name,
                values: vec![#( #values ),*]
            }
        });
    }
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let caption = &self.caption;
        let name = &self.name;
        let value = &self.value;
        tokens.extend(quote!{
             crate::r#struct::module::Value {
                caption: #caption,
                name: #name,
                value: #value,
             }
        });
    }
}