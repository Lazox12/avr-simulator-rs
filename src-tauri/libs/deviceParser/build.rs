use std::collections::HashMap;
use std::{env, fs, io};
use std::fs::DirEntry;
use std::io::ErrorKind;
use std::path::Path;
use anyhow::anyhow;
use xmltree::Element;
pub use crate::r#struct::module::Register;
pub use crate::r#struct::common_registers::CommonRegisters;
use build_print::{error, info, warn};
use quote::quote;
pub mod r#struct;
pub mod utils;


fn main() {

    println!("cargo:rerun-if-changed=atdf");
    println!("cargo:rerun-if-changed=build.rs");
    let mut c1:u16=0;
    let mut c2:u16=0;
    let mut success:Vec<String> = vec!();
    let out_dir = env::var_os("OUT_DIR").unwrap();
    if(!fs::exists(Path::new(&out_dir).join("avr")).unwrap()){
        fs::create_dir(Path::new(&out_dir).join("avr")).unwrap();
    }
    let mut mods:Vec<String> = vec!();
    get_tree_map().unwrap().iter().for_each(|(name,data)| {
        c1 += 1;
    //get_tree_map().unwrap().iter().next().into_iter().for_each(|(name,data)|{
        let dest_path = Path::new(&out_dir).join(format!("avr/{}.rs",name));
        let mut generated = quote!{
            #data
        }.to_string();
        let reg_map = get_register_map(&name);
        generated+=format!("\npub const REGISTERMAP:phf::Map<u64,&'static super::Register> = super::phf_map!{{{}}};",reg_map.iter().map(|(u1,x1)| {format!("{}=>&{}",u1,(quote! {#x1}.to_string()))}).collect::<Vec<String>>().join(",")).as_str();
        match get_common_registers(&name){
            Some(regs) => {
                c2 += 1;
                generated+=format!("\n pub const COMMONREGISTERS:super::CommonRegisters = {};", quote! {#regs}.to_string()).as_str();
                success.push(name.clone());
            }
            None => {}
        };
        fs::write(&dest_path, generated).unwrap();
        mods.push(name.clone());
    });
    info!("{:?}",out_dir);
    let mut toMod:String = "".to_string();
    toMod="pub use phf::phf_map;\n".to_string();
    toMod+="pub use std::ptr::null;\n";
    toMod+=  mods.iter().map(|x| {
        return format!("pub mod {};",x).to_string();
    }).collect::<Vec<String>>().join("\n").as_str();
    toMod+="\n";
    toMod+=format!("pub const McuList:&'static[&'static str] =&[{}];",mods.iter().map(|f|{format!("\"{}\"",f)}).collect::<Vec<String>>().join(",")).as_str();
    toMod+=format!("\npub const McuStruct: phf::Map<&'static str,&'static AvrDeviceFile>= phf_map!{{{}}};",mods.iter().map(|f| format!("\"{0}\"=>&{0}::{1}",f,f.to_uppercase())).collect::<Vec<String>>().join(",")).as_str();
    toMod+=format!("\npub const McuRegisterStruct: phf::Map<&'static str,&'static phf::Map<u64,&'static Register>>= phf_map!{{{}}};",mods.iter().map(|f| format!("\"{0}\"=>&{0}::REGISTERMAP",f)).collect::<Vec<String>>().join(",")).as_str();
    toMod+=format!("\npub const McuCommonRegisterStruct: phf::Map<&'static str,&'static CommonRegisters>= phf_map!{{{}}};",success.iter().map(|f| format!("\"{0}\"=>&{0}::COMMONREGISTERS",f)).collect::<Vec<String>>().join(",")).as_str();
    fs::write(Path::new(&out_dir).join("avr/mod.rs"), toMod);
    info!("sucess:{} from:{}",c2,c1);
}


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
    let b :&'static Element=Box::leak(Box::from(elem));
    let a = AvrDeviceFile::from(b);
    Ok(a)
}

pub fn get_register_map(device_name:&String)->HashMap<u64,&'static Register>{
    match get_tree_map().unwrap().get(device_name.as_str()){
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
                                        name: &*(Box::leak(Box::new(x2.name.clone().to_owned() + "(H)"))),
                                        offset: x2.offset,
                                        size: 1,
                                        initval: x2.initval,
                                        bitfields: x2.bitfields.clone(),
                                    }));
                                    let leaked2: &'static Register = Box::leak(Box::new(Register {
                                        caption:x2.caption.clone(),
                                        name: &*(Box::leak(Box::new(x2.name.clone().to_owned() + "(L)"))),
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

            reg_map
        }
        None=>{
            panic!("device not found");
        }
    }


}

pub fn get_mcu_list()->Vec<&'static String>{
    let tree = get_tree_map().unwrap();
    tree.keys().collect::<Vec<&String>>()
}

pub fn get_common_registers(device_name:&String) ->Option<CommonRegisters>{
    let tree = get_tree_map().unwrap().get(device_name.as_str()).unwrap();
    let mut a:Vec<u8> =Vec::new();
    a.resize((tree.devices.address_spaces.iter().find(|x| {x.id == "data" })?
        .memory_segments.iter().find(|x1| {  "MAPPED_IO".to_string().eq(x1.name) })?.size + 20) as usize,0);
    match CommonRegisters::init(tree,&mut a){
        Ok(t)=>{
            Some(t)
        }
        Err(e)=>{
            warn!("{}",e);
            None
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;


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
