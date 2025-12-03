use proc_macro2::Ident;
use quote::__private::{Span, TokenStream};
use quote::{quote, ToTokens};
use xmltree::Element;
use serde::Serialize;
use crate::r#struct::device_property_group::PropertyValue;
use crate::utils::{find_childs, to_ident};

#[derive(Debug)]
pub struct Module{
    pub caption:Option<String>,
    pub name: String,
    pub register_group: Vec<ModuleRegisterGroup>,
    pub value_grop: Vec<ValueGroup>
}
impl From<&Element> for Module{
    fn from(x:&Element) -> Self{
        Module{
            caption: x.attributes.get("caption").map(|t| t.to_string()),
            name: x.attributes["name"].to_string(),
            register_group: find_childs(x,"register-group".to_string()).into_iter().map(|x1| {ModuleRegisterGroup::from(x1)}).collect(),
            value_grop: find_childs(x,"value-group".to_string()).into_iter().map(|x1| {ValueGroup::from(x1)}).collect(),
        }
    }
}
#[derive(Debug)]
pub struct ModuleRegisterGroup{
    pub caption:Option<String>,
    pub name: String,
    pub register: Vec<Register>
}
impl From<&Element> for ModuleRegisterGroup{
    fn from(x:&Element) -> Self{
        ModuleRegisterGroup{
            caption: x.attributes.get("caption").map(|x1| x1.to_string()),
            name: x.attributes["name"].to_string(),
            register: find_childs(x,"register".to_string()).into_iter().map(|x1| {Register::from(x1)}).collect(),
        }
    }
}
#[derive(Debug,Default,Serialize,Clone)]
pub struct Register{
    pub caption:Option<String>,
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub initval:u64,
    pub bitfields:Option<Vec<BitField>>,
}
impl From<&Element> for Register{
    fn from(x:&Element) -> Self{
        Register{
            caption:x.attributes.get("caption").map(|x1| x1.to_string()),
            name: x.attributes["name"].to_string(),
            offset: u64::from_str_radix(x.attributes["offset"].to_string().strip_prefix("0x").unwrap(), 16).unwrap(),
            size: x.attributes["size"].to_string().parse().unwrap(),
            initval: u64::from_str_radix(x.attributes["offset"].to_string().strip_prefix("0x").unwrap(), 16).unwrap(),
            bitfields: Some(find_childs(x,"bitfield".to_string()).into_iter().map(|x1| {BitField::from(x1)}).collect()),
        }
    }
}
#[derive(Debug,Serialize,Clone,Default)]
#[serde(rename_all = "camelCase")]
pub struct BitField{
    pub caption: Option<String>,
    pub mask: u64,
    pub name: String,
    pub values:Option<String>,
}
impl From<&Element> for BitField{
    fn from(x:&Element) -> Self{
        BitField{
            caption: x.attributes.get("caption").map(|x1| x1.to_string()),
            mask: u64::from_str_radix(x.attributes["mask"].to_string().strip_prefix("0x").unwrap(), 16).unwrap(),
            name: x.attributes["name"].to_string(),
            values: x.attributes.get("name").map(|x1| x1.to_string()),
        }
    }
}
#[derive(Debug)]
pub struct ValueGroup{
    pub name: String,
    pub values: Vec<Value>
}
impl From<&Element> for ValueGroup{
    fn from(x:&Element) -> Self{
        ValueGroup{ 
            name: x.attributes["name"].to_string(),
            values: find_childs(x,"value".to_string()).into_iter().map(|x1| {Value::from(x1)}).collect()
        }
    }
}
#[derive(Debug)]
pub struct Value{
    caption: String,
    name: String,
    value: PropertyValue,
}
impl From<&Element> for Value{
    fn from(x:&Element) -> Self{
        Value{
            caption: x.attributes["caption"].to_string(),
            name: x.attributes["name"].to_string(),
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
            Some(c) => quote! { Some(#c.to_string()) },
            None => quote! { None },
        };

        // fully qualified path: crate::r#struct::module::Module
        tokens.extend(quote! {
            crate::r#struct::module::Module {
                caption: #caption,
                name: #name.to_string(),
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
            Some(c) => quote! { Some(#c.to_string()) },
            None => quote! { None },
        };

        tokens.extend(quote! {
            crate::r#struct::module::ModuleRegisterGroup {
                caption: #caption,
                name: #name.to_string(),
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
            Some(c) => quote! { Some(#c.to_string()) },
            None => quote! { None },
        };
        let bitfields = match &self.bitfields {
            Some(b) => quote! { Some(vec![#( #b ),*]) },
            None => quote! { None },
        };

        tokens.extend(quote! {
            crate::r#struct::module::Register {
                caption: #caption,
                name: #name.to_string(),
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
            Some(c) => quote! { Some(#c.to_string()) },
            None => quote! { None },
        };
        let values = match &self.values {
            Some(v) => quote! { Some(#v.to_string()) },
            None => quote! { None },
        };

        tokens.extend(quote! {
            crate::r#struct::module::BitField {
                caption: #caption,
                mask: #mask,
                name: #name.to_string(),
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
                name: #name.to_string(),
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
                caption: #caption.to_string(),
                name: #name.to_string(),
                value: #value,
             }
        });
    }
}