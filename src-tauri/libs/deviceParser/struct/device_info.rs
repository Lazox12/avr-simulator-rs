use quote::{quote, ToTokens};
use quote::__private::TokenStream;
use xmltree::Element;
use super::utils::{find_child, find_childs, to_ident};
use super::device_address_space::AddressSpace;
use super::device_interface::Interface;
use super::device_interrupt::Interrupt;
use super::device_peripherals::Module;
use super::device_property_group::PropertyGroup;

#[derive(Debug,Default)]
pub struct Device{
    pub name: &'static str,
    pub architecture: &'static str, //todo should be enum
    pub family: &'static str, //todo should be enum
    pub address_spaces: &'static [AddressSpace],
    pub peripherals:&'static [Module],
    pub interrupts:&'static [Interrupt],
    pub interfaces:&'static [Interface],
    pub propery_groups:&'static [PropertyGroup]
}
impl From<&'static Element> for Device{
    fn from(x:&'static Element) -> Self{
        Device{
            name: &x.attributes["name"],
            architecture: &x.attributes["architecture"],
            family: &x.attributes["family"],
            address_spaces: Box::leak(find_childs(find_child(x,"address-spaces").unwrap(),"address-space").into_iter().map(|x| {AddressSpace::from(x)}).collect::<Vec<AddressSpace>>().into_boxed_slice()),
            peripherals: Box::leak(find_childs(find_child(x,"peripherals").unwrap(),"module").into_iter().map(|x| {Module::from(x)}).collect::<Vec<Module>>().into_boxed_slice()),
            interrupts: Box::leak(find_childs(find_child(x,"interrupts").unwrap(),"interrupt").into_iter().map(|x| {Interrupt::from(x)}).collect::<Vec<Interrupt>>().into_boxed_slice()),
            interfaces: find_childs(find_child(x,"interfaces").unwrap(),"interface").into_iter().map(|x| {Interface::from(x)}).collect(),
            propery_groups: find_childs(find_child(x,"property-groups").unwrap(),"property-group").into_iter().map(|x| {PropertyGroup::from(x)}).collect(),
        }
    }
}

#[derive(Debug)]
pub struct Variant{
    pub order_code: &'static str,
    pub temp_min:i64,
    pub temp_max:i64,
    pub max_speed:i64,
    pub pinout: Option<&'static str>,
    pub package: &'static str,
    pub vcc_min:f64,
    pub vcc_max:f64,
}
impl From<&'static Element> for Variant{
    fn from(element:&'static Element) -> Self{
        Variant{
            order_code: &element.attributes["ordercode"],
            temp_min: element.attributes["tempmin"].parse().unwrap(),
            temp_max: element.attributes["tempmax"].parse().unwrap(),
            max_speed: element.attributes["speedmax"].parse().unwrap(),
            pinout: element.attributes.get("pinout").map(|x| x.as_str()),
            package: &element.attributes["package"],
            vcc_min: element.attributes["vccmin"].parse().unwrap(),
            vcc_max: element.attributes["vccmax"].parse().unwrap(),
        }
    }
}
impl ToTokens for Device {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let architecture = &self.architecture;
        let family = &self.family;
        let address_spaces = &self.address_spaces;
        let peripherals = &self.peripherals;
        let interrupts = &self.interrupts;
        let interfaces = &self.interfaces;
        let propery_groups = &self.propery_groups;

        tokens.extend(quote! {
            crate::r#struct::device_info::Device {
                name: #name,
                architecture: #architecture,
                family: #family,
                address_spaces: [#( #address_spaces ),*],
                peripherals: [#( #peripherals ),*],
                interrupts: [#( #interrupts ),*],
                interfaces: [#( #interfaces ),*],
                propery_groups: [#( #propery_groups ),*],
            }
        });
    }
}

impl ToTokens for Variant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let order_code = &self.order_code;
        let temp_min = self.temp_min;
        let temp_max = self.temp_max;
        let max_speed = self.max_speed;
        let package = &self.package;
        let vcc_min = self.vcc_min;
        let vcc_max = self.vcc_max;

        let pinout = match &self.pinout {
            Some(p) => quote! { Some(#p) },
            None => quote! { None },
        };

        tokens.extend(quote! {
            crate::r#struct::device_info::Variant {
                order_code: #order_code,
                temp_min: #temp_min,
                temp_max: #temp_max,
                max_speed: #max_speed,
                pinout: #pinout,
                package: #package,
                vcc_min: #vcc_min,
                vcc_max: #vcc_max,
            }
        });
    }
}