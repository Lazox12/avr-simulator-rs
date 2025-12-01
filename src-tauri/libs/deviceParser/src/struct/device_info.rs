use xmltree::Element;
use crate::utils::{find_child, find_childs};
use super::device_address_space::AddressSpace;
use super::device_interface::Interface;
use super::device_interrupt::Interrupt;
use super::device_peripherals::Module;
use super::device_property_group::PropertyGroup;

#[derive(Debug,Default)]
pub struct Device{
    pub name: String,
    pub architecture: String, //todo should be enum
    pub family: String, //todo should be enum
    pub address_spaces: Vec<AddressSpace>,
    pub peripherals:Vec<Module>,
    pub interrupts:Vec<Interrupt>,
    pub interfaces:Vec<Interface>,
    pub propery_groups:Vec<PropertyGroup>
}
impl From<&Element> for Device{
    fn from(x:&Element) -> Self{
        Device{
            name: x.attributes["name"].to_string(),
            architecture: x.attributes["architecture"].to_string(),
            family: x.attributes["family"].to_string(),
            address_spaces: find_childs(find_child(x,"address-spaces".to_string()).unwrap(),"address-space".to_string()).into_iter().map(|x| {AddressSpace::from(x)}).collect(),
            peripherals: find_childs(find_child(x,"peripherals".to_string()).unwrap(),"module".to_string()).into_iter().map(|x| {Module::from(x)}).collect(),
            interrupts: find_childs(find_child(x,"interrupts".to_string()).unwrap(),"interrupt".to_string()).into_iter().map(|x| {Interrupt::from(x)}).collect(),
            interfaces: find_childs(find_child(x,"interfaces".to_string()).unwrap(),"interface".to_string()).into_iter().map(|x| {Interface::from(x)}).collect(),
            propery_groups: find_childs(find_child(x,"property-groups".to_string()).unwrap(),"property-group".to_string()).into_iter().map(|x| {PropertyGroup::from(x)}).collect(),
        }
    }
}

#[derive(Debug)]
pub struct Variant{
    pub order_code: String,
    pub temp_min:i64,
    pub temp_max:i64,
    pub max_speed:i64,
    pub pinout: Option<String>,
    pub package: String,
    pub vcc_min:f64,
    pub vcc_max:f64,
}
impl From<&Element> for Variant{
    fn from(element:&Element) -> Self{
        Variant{
            order_code: element.attributes["ordercode"].to_string(),
            temp_min: element.attributes["tempmin"].to_string().parse().unwrap(),
            temp_max: element.attributes["tempmax"].to_string().parse().unwrap(),
            max_speed: element.attributes["speedmax"].to_string().parse().unwrap(),
            pinout: element.attributes.get("pinout").map(|x| x.to_string()),
            package: element.attributes["package"].to_string(),
            vcc_min: element.attributes["vccmin"].to_string().parse().unwrap(),
            vcc_max: element.attributes["vccmax"].to_string().parse().unwrap(),
        }
    }
}