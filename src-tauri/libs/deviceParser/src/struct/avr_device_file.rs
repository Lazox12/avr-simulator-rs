use xmltree::Element;
use crate::utils::{find_child, find_childs};
use super::device_info::{Device, Variant};
use super::device_package::Pinout;
use super::module::Module;

#[derive(Debug,Default)]
pub struct AvrDeviceFile {
    pub variants:Vec<Variant>,
    pub devices:Device,
    pub modules:Vec<Module>,
    pub pinouts:Option<Vec<Pinout>>,
}
impl From<&Element> for AvrDeviceFile {
    fn from(element:&Element) -> Self {
        AvrDeviceFile{
            variants: find_childs(find_child(element,"variants".to_string()).unwrap(),"variant".to_string()).into_iter().map(|x| {Variant::from(x)}).collect(),
            devices: find_child(find_child(element,"devices".to_string()).unwrap(),"device".to_string()).map(|f| Device::from(f)).unwrap(),
            modules: find_childs(find_child(element,"modules".to_string()).unwrap(),"module".to_string()).into_iter().map(|x| {Module::from(x)}).collect(),
            pinouts: find_child(element,"pinouts".to_string()).map(|x| find_childs(x,"pinout".to_string()).into_iter().map(|x| {Pinout::from(x)}).collect()),
        }
    }
}