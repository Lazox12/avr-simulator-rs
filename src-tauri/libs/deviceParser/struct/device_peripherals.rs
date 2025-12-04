use xmltree::Element;
use super::utils::{find_child, find_childs, to_ident};
#[derive(Debug)]
pub struct Module{
    pub name: &'static str,
    pub instances:Vec<Instance>
    
}
impl From<&'static Element> for Module {
    fn from(x:&'static Element) -> Self {
        Module{ 
            name: &x.attributes["name"],
            instances: find_childs(x,"memory-segment".to_string()).into_iter().map(|x| {Instance::from(x)}).collect(), 
        }
    }
}

#[derive(Debug)]
pub struct Instance{
    pub name: &'static str,
    pub caption: &'static str,
    pub register_group: RegisterGroup,
    pub signals:Option<Vec<Signal>>,
    pub parameters:Option<Vec<Param>>
}
impl From<&'static Element> for Instance {
    fn from(x:&'static Element) -> Self {
        Instance{
            name: &x.attributes["name"],
            caption: &x.attributes["caption"],
            register_group: RegisterGroup::from(find_child(x, "register-group".to_string()).unwrap()),
            signals: Some(find_childs(x,"memory-segment".to_string()).into_iter().map(|x| {Signal::from(x)}).collect()),
            parameters: Some(find_childs(x,"memory-segment".to_string()).into_iter().map(|x| {Param::from(x)}).collect()),
        }
    }
}
#[derive(Debug)]
pub struct RegisterGroup{
    pub name: &'static str,
    pub name_in_module: &'static str,
    pub offset: u64,
    pub address_space:&'static str,
    pub caption: &'static str,
}
impl From<&'static Element> for RegisterGroup {
    fn from(x:&'static Element) -> Self {
        RegisterGroup{
            name: &x.attributes["name"],
            name_in_module: &x.attributes["name-in-module"],
            offset: u64::from_str_radix(x.attributes["offset"].to_string().strip_prefix("0x").unwrap(), 16).unwrap(),
            address_space: &x.attributes["address-space"],
            caption: &x.attributes["caption"],
        }
    }
}

#[derive(Debug)]
pub struct Signal{
    pub group: &'static str,
    pub function: &'static str, //todo should be enum
    pub pad:&'static str,
    pub index:Option<i64>,
}
impl From<&'static Element> for Signal {
    fn from(x:&'static Element) -> Self {
        Signal{
            group: &x.attributes["group"],
            function: &x.attributes["function"],
            pad: &x.attributes["pad"],
            index: Some(x.attributes["index"].to_string().parse().unwrap()),
        }
    }
}

#[derive(Debug)]
pub struct Param{
    pub name: &'static str,
    pub value: &'static str,
}
impl From<&'static Element> for Param {
    fn from(x:&'static Element) -> Self {
        Param{ 
            name: &x.attributes["name"],
            value: &x.attributes["value"]
        }
    }
}
use quote::{quote, ToTokens};
use quote::__private::{Span, TokenStream};

impl ToTokens for Module {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let instances = &self.instances;

        // fully qualified path: crate::r#struct::device_peripherals::Module
        tokens.extend(quote! {
            crate::r#struct::device_peripherals::Module {
                name: #name.to_string(),
                instances: vec![#( #instances ),*],
            }
        });
    }
}

impl ToTokens for Instance {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let caption = &self.caption;
        let register_group = &self.register_group;

        let signals = match &self.signals {
            Some(s) => quote! { Some(vec![#( #s ),*]) },
            None => quote! { None },
        };
        let parameters = match &self.parameters {
            Some(p) => quote! { Some(vec![#( #p ),*]) },
            None => quote! { None },
        };

        tokens.extend(quote! {
            crate::r#struct::device_peripherals::Instance {
                name: #name.to_string(),
                caption: #caption.to_string(),
                register_group: #register_group,
                signals: #signals,
                parameters: #parameters,
            }
        });
    }
}

impl ToTokens for RegisterGroup {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let name_in_module = &self.name_in_module;
        let offset = self.offset;
        let address_space = &self.address_space;
        let caption = &self.caption;

        tokens.extend(quote! {
            crate::r#struct::device_peripherals::RegisterGroup {
                name: #name.to_string(),
                name_in_module: #name_in_module.to_string(),
                offset: #offset,
                address_space: #address_space.to_string(),
                caption: #caption.to_string(),
            }
        });
    }
}

impl ToTokens for Signal {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let group = &self.group;
        let function = &self.function;
        let pad = &self.pad;
        let index = match self.index {
            Some(i) => quote!{ Some(#i) },
            None => quote!{ None },
        };

        tokens.extend(quote!{
             crate::r#struct::device_peripherals::Signal {
                 group: #group.to_string(),
                 function: #function.to_string(),
                 pad: #pad.to_string(),
                 index: #index,
             }
         });
    }
}

impl ToTokens for Param {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let value = &self.value;
        tokens.extend(quote!{
            crate::r#struct::device_peripherals::Param {
                name: #name.to_string(),
                value: #value.to_string(),
            }
        });
    }
}