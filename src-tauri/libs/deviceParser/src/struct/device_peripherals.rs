use xmltree::Element;
use crate::r#struct::device_address_space::MemorySegment;
use crate::utils::{find_child, find_childs};

#[derive(Debug)]
pub struct Module{
    pub name: String,
    pub instances:Vec<Instance>
    
}
impl From<&Element> for Module {
    fn from(x:&Element) -> Self {
        Module{ 
            name: x.attributes["name"].to_string(),
            instances: find_childs(x,"memory-segment".to_string()).into_iter().map(|x| {Instance::from(x)}).collect(), 
        }
    }
}

#[derive(Debug)]
pub struct Instance{
    pub name: String,
    pub caption: String,
    pub register_group: RegisterGroup,
    pub signals:Option<Vec<Signal>>,
    pub parameters:Option<Vec<Param>>
}
impl From<&Element> for Instance {
    fn from(x:&Element) -> Self {
        Instance{
            name: x.attributes["name"].to_string(),
            caption: x.attributes["caption"].to_string(),
            register_group: RegisterGroup::from(find_child(x, "register-group".to_string()).unwrap()),
            signals: Some(find_childs(x,"memory-segment".to_string()).into_iter().map(|x| {Signal::from(x)}).collect()),
            parameters: Some(find_childs(x,"memory-segment".to_string()).into_iter().map(|x| {Param::from(x)}).collect()),
        }
    }
}
#[derive(Debug)]
pub struct RegisterGroup{
    pub name: String,
    pub name_in_module: String,
    pub offset: u64,
    pub address_space:String,
    pub caption: String,
}
impl From<&Element> for RegisterGroup {
    fn from(x:&Element) -> Self {
        RegisterGroup{
            name: x.attributes["name"].to_string(),
            name_in_module: x.attributes["name-in-module"].to_string(),
            offset: u64::from_str_radix(x.attributes["offset"].to_string().strip_prefix("0x").unwrap(), 16).unwrap(),
            address_space: x.attributes["address-space"].to_string(),
            caption: x.attributes["caption"].to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Signal{
    pub group: String,
    pub function: String, //todo should be enum
    pub pad:String,
    pub index:Option<i64>,
}
impl From<&Element> for Signal {
    fn from(x:&Element) -> Self {
        Signal{
            group: x.attributes["group"].to_string(),
            function: x.attributes["function"].to_string(),
            pad: x.attributes["pad"].to_string(),
            index: Some(x.attributes["index"].to_string().parse().unwrap()),
        }
    }
}

#[derive(Debug)]
pub struct Param{
    pub name: String,
    pub value: String,
}
impl From<&Element> for Param {
    fn from(x:&Element) -> Self {
        Param{ 
            name: x.attributes["name"].to_string(),
            value: x.attributes["value"].to_string()
        }
    }
}