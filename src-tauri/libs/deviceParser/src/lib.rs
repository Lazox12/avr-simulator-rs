mod utils;
mod r#struct;

use std::collections::HashMap;
use std::error::Error;
use std::{fs, io};
use std::fs::DirEntry;
use std::io::ErrorKind;
use std::str::FromStr;
use xmltree::{Element, XMLNode};
use r#struct::avr_device_file::AvrDeviceFile;


static mut TREE_MAP:Option<HashMap<String,AvrDeviceFile>> = None;
#[allow(static_mut_refs)]
pub fn get_tree_map() ->Result<&'static HashMap<String,AvrDeviceFile>,xmltree::Error>{
    if(unsafe{ TREE_MAP.is_none()}){
        let files = std::fs::read_dir("atdf").unwrap();
        let mut map = HashMap::new();
        for file in files{
            let dir_entry = file?;
            map.insert(dir_entry.file_name().to_str().unwrap().to_string(),get_tree(&dir_entry)?);
        }    
        unsafe { TREE_MAP = Some(map);}
    }
    
    unsafe{match TREE_MAP.as_ref() {
        Some(map) => {
            Ok(&map)
        }
        None => {
            Err(xmltree::Error::from(io::Error::new(ErrorKind::Other, "failed to get tree map")))
        }
    }
    }
}

pub fn get_tree(file:&DirEntry) -> Result<AvrDeviceFile,xmltree::Error>{
    let data = &*fs::read_to_string(file.path()).unwrap();
    let elem = Element::parse(data.as_bytes()).unwrap();
    let a = AvrDeviceFile::from(&elem);
    Ok(a)
}

pub fn get_register_map(device_name:String)->Result<HashMap<u32,String>,xmltree::Error>{
    match get_tree_map()?.get(device_name.as_str()){
        Some(t)=>{
            
        }
        None=>{
            Err(xmltree::Error::from(io::Error::new(ErrorKind::NotFound,"Device not found")))
        }
    }
    
    
}


#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn tree_test(){
        assert!(get_tree_map().is_ok())
    }
}


