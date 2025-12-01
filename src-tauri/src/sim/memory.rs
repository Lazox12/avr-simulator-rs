use std::sync::Mutex;
use anyhow::anyhow;
use crate::error::Result;
use deviceParser::AvrDeviceFile;

pub struct Memory {
    flash:Vec<u8>,
    data:DataMemory,
    eeprom: Vec<u8>,
}
impl Memory {
    pub fn new() -> Memory {
        Memory {flash: Vec::new(),data:DataMemory::new(),eeprom:Vec::new()}
    }
    pub fn init(&mut self, atdf:&'static AvrDeviceFile, flash_data:Option<Vec<u8>>, eeprom_data:Option<Vec<u8>>) ->Result<()> {
        if(flash_data.is_some()) {
            self.flash = flash_data.unwrap();
        }
        if (eeprom_data.is_some()) {
            self.eeprom = eeprom_data.unwrap();
        }
        let address_space = atdf.devices.address_spaces.iter().find(|x| {x.id=="prog"}).unwrap();
        let eeprom_space = atdf.devices.address_spaces.iter().find(|x| {x.id=="eeprom"}).unwrap();
        self.flash.resize(address_space.size as usize, 0u8);
        self.eeprom.resize(eeprom_space.size as usize, 0xffu8);
        self.data.init(&atdf)?;
        Ok(())
    }
}

pub struct DataMemory {
    data:Vec<u8>,
    pub registers:*mut u8,
    pub io:*mut u8,
    pub ram:*mut u8,
}
impl DataMemory {
    pub fn new()->Self{
        DataMemory{
            data: Vec::new(),
            registers: &mut 0,
            io: &mut 0,
            ram: &mut 0,
        }
    }
    pub fn init(&mut self,atdf:&'static AvrDeviceFile) -> Result<()> {
        let address_space = atdf.devices.address_spaces.iter().find(|x| {x.id=="data"}).unwrap();
        let io_start = address_space.memory_segments.iter().find(|x| {x.name=="MAPPED_IO"}).unwrap().start;
        let ram_start = address_space.memory_segments.iter().find(|x| {x.name=="IRAM"}).unwrap().start;
        self.data.resize(address_space.size as usize,0u8);
        unsafe {
            self.registers = self.data.as_mut_ptr();
            self.io = self.data.as_mut_ptr().add(io_start as usize);
            self.ram = self.data.as_mut_ptr().add(ram_start as usize);
        }
        Ok(())
    }

}

