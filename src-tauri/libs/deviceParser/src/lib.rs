use std::collections::HashMap;
use std::fs::DirEntry;
//pub use phf::phf_map;
#[path = "../struct/mod.rs"]
pub mod r#struct;

#[path = "../utils.rs"]
pub mod utils;

include!(concat!(env!("OUT_DIR"), "/avr/mod.rs"));


pub fn get_tree_map() ->&'static phf::Map<&'static str,&'static AvrDeviceFile>{
    &McuStruct
}
pub fn get_register_map(device_name:&String)->Option<&'static phf::Map<u64,&'static Register>>{
    match McuRegisterStruct.get(&device_name){
        None=>None,
        Some(t)=>Some(*t)
    }
}
pub const fn get_mcu_list()->&'static[&'static str]{
    McuList
}


pub type AvrDeviceFile= crate::r#struct::avr_device_file::AvrDeviceFile;
pub type Register= crate::r#struct::module::Register;
pub type Error = String;