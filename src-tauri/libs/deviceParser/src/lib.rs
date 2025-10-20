mod utils;
mod r#struct;

use std::error::Error;
use std::fs;
use std::str::FromStr;
use xmltree::{Element, XMLNode};
use r#struct::avr_device_file::AvrDeviceFile;

pub fn run() ->Result<(),xmltree::Error>{
    let files = std::fs::read_dir("atdf").unwrap();
    for file in files{
        let path = &*fs::read_to_string(file.as_ref().unwrap().path()).unwrap();
        let elem = Element::parse(path.as_bytes()).unwrap();
        let a = AvrDeviceFile::from(&elem);
        println!("{:#?}", a);
    }
    Ok(())
}
trait rename{
    fn rename(&self)->String;
}
impl rename for String{
    fn rename(&self)->String{
        self.replace("-", "_")
    }
}


