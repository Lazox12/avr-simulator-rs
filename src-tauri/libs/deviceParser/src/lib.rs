mod utils;
mod r#struct;

use std::collections::HashMap;
use std::{fs, io};
use std::fs::DirEntry;
use std::io::ErrorKind;
use std::path::Path;
use xmltree::{Element, XMLNode};
pub use crate::r#struct::module::Register;
use build_print::*;
static mut TREE_MAP:Option<HashMap<String,AvrDeviceFile>> = None;
#[allow(static_mut_refs)]
pub fn get_tree_map() ->Result<&'static HashMap<String,AvrDeviceFile>,xmltree::Error>{
    if(unsafe{ TREE_MAP.is_none()}){
        let files = std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("atdf"))?;
        let mut map = HashMap::new();
        for file in files{
            let dir_entry = file?;
            map.insert(dir_entry.file_name().to_str().unwrap().to_string().strip_suffix(".atdf").unwrap().to_lowercase(),get_tree(&dir_entry)?);
        }    
        unsafe { TREE_MAP = Some(map);}
    }
    
    unsafe{TREE_MAP.as_ref().ok_or(xmltree::Error::from(io::Error::new(ErrorKind::Other, "failed to get tree map")))
    }
}

pub fn get_tree(file:&DirEntry) -> Result<AvrDeviceFile,xmltree::Error>{
    let data = &*fs::read_to_string(file.path()).unwrap();
    let elem = Element::parse(data.as_bytes()).unwrap();
    let a = AvrDeviceFile::from(&elem);
    Ok(a)
}

pub fn get_register_map(device_name:&String)->Result<HashMap<u64,&'static Register>,xmltree::Error>{
    match get_tree_map()?.get(device_name.as_str()){
        Some(t)=>{
            let mut reg_map = HashMap::<u64,&'static Register>::new();
            t.modules.iter().for_each(|x| {
                x.register_group.iter().for_each(|x1| {
                    x1.register.iter().for_each(|x2| {
                        match x2.size{
                            1=>{
                                reg_map.insert(x2.offset,x2);
                            }   
                            2=>{
                                if(x2.size ==2){
                                    let leaked1: &'static Register = Box::leak(Box::new(Register {
                                        caption: x2.caption.clone(),
                                        name: x2.name.clone() + "(H)",
                                        offset: x2.offset,
                                        size: 1,
                                        initval: x2.initval,
                                        bitfields: x2.bitfields.clone(),
                                    }));
                                    let leaked2: &'static Register = Box::leak(Box::new(Register {
                                        caption:x2.caption.clone(),
                                        name: x2.name.clone() + "(L)",
                                        offset: x2.offset,
                                        size: 1,
                                        initval: x2.initval,
                                        bitfields: x2.bitfields.clone(),
                                    }));
                                    
                                    reg_map.insert(x2.offset+1, leaked1);
                                    reg_map.insert(x2.offset, leaked2);
                                }
                            }
                            _=>{}
                        }
                    })
                })
            });
            
            Ok(reg_map)
        }
        None=>{
            Err(xmltree::Error::from(io::Error::new(ErrorKind::NotFound,"Device not found")))
        }
    }
    
    
}

pub fn get_mcu_list()->Result<Vec<&'static String>, xmltree::Error>{
    let tree = get_tree_map()?;
    Ok(tree.keys().collect::<Vec<&String>>())
}

#[cfg(test)]
mod tests{
    use crate::utils::time_function;
    use super::*;
    #[test]
    fn tree_test(){
        info!("starting computing");
        let (res,time) = time_function(get_tree_map);
        info!("{:?}",time);
        info!("{:?}",res.unwrap().len());

    }

    #[test]
    fn register_map_test(){
        let res = get_register_map(&"Atmega328P".to_string().to_lowercase());
        info!("{:?}",res);
    }

    #[test]
    fn get_mcu_list_test(){
        let res = get_mcu_list();
        info!("{:?}",res);
    }
} 

pub type Error = xmltree::Error;
pub type AvrDeviceFile= crate::r#struct::avr_device_file::AvrDeviceFile;