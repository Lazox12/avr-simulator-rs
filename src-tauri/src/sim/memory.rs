use crate::error::Result;
use device_parser::AvrDeviceFile;
use opcode_gen::CustomOpcodes;
use crate::sim::instruction::Instruction;

#[derive(Default,Debug)]
pub struct Memory {
    pub flash:Vec<Instruction>,
    pub data:DataMemory,
    pub eeprom: Vec<u8>,
    pub program_couter:u32
}

impl Memory {
    pub fn init(&mut self, atdf:&'static AvrDeviceFile, flash_data:Vec<Instruction>, eeprom_data:Vec<u8>) ->Result<()> {
        self.flash = flash_data;
        self.eeprom = eeprom_data;
        let address_space = atdf.devices.address_spaces.iter().find(|x| {x.id=="prog"}).unwrap();
        let eeprom_space = atdf.devices.address_spaces.iter().find(|x| {x.id=="eeprom"}).unwrap();
        self.flash.resize((address_space.size/2) as usize, Instruction::decode_from_opcode(CustomOpcodes::EMPTY as u16)?);
        self.eeprom.resize(eeprom_space.size as usize, 0xffu8);
        self.data.init(&atdf)?;
        Ok(())
    }
}

#[derive(Default,Debug)]
pub struct DataMemory {
    pub registers:Vec<u8>,
    pub io:IOMemory<u8>,
    pub ram:Vec<u8>,
}
impl std::ops::Index<usize> for DataMemory {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.registers.len() {
            &self.registers[index]
        }else if index-self.registers.len() < self.io.len(){
            &self.io[index - self.registers.len()]
        }else{
            &self.ram[index - self.registers.len() - self.io.len()]
        }
    }

}
impl std::ops::IndexMut<usize> for DataMemory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < self.registers.len() {
            &mut self.registers[index]
        } else if index - self.registers.len() < self.io.len() {
            &mut self.io[index - self.registers.len()]
        } else {
            &mut self.ram[index - self.registers.len() - self.io.len()]
        }
    }
}


impl DataMemory {
    pub fn init(&mut self,atdf:&'static AvrDeviceFile) -> Result<()> {
        let address_space = atdf.devices.address_spaces.iter().find(|x| {x.id=="data"}).unwrap();
        let reg_size = address_space.memory_segments.iter().find(|x| {x.name=="REGISTERS"}).unwrap().size;
        let io_size = address_space.memory_segments.iter().find(|x| {x.name=="MAPPED_IO"}).unwrap().size;
        let ram_size = address_space.memory_segments.iter().find(|x| {x.name=="IRAM"}).unwrap().size;
        self.registers.resize(reg_size as usize, 0);
        self.io.resize(io_size as usize, 0);
        self.io.reg_size = reg_size as usize;
        self.ram.resize(ram_size as usize, 0);

        Ok(())
    }
    pub fn len(&self) -> usize {
        self.registers.len()+self.io.len()+self.ram.len()
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut u8> {
        if index < self.len() {
            Some(&mut self[index])
        }else{
            None
        }
    }
    pub fn get(&mut self, index: usize) -> Option<&u8> {
        if index < self.len() {
            Some(&mut self[index])
        }else{
            None
        }
    }
}

#[derive(Default,Debug)]
pub struct IOMemory<T> {
    pub inner:Vec<T>,
    pub watchlist:Vec<u32>,
    pub write_status:bool,
    pub reg_size:usize,
}
impl<T: Clone> IOMemory<T> {
    pub fn len(&self)->usize{
        self.inner.len()
    }
    pub fn resize(&mut self,size:usize,data:T) {
        self.inner.resize(size,data);
    }
}
impl<T> std::ops::Index<usize> for IOMemory<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        self.inner.index(index)
    }
}
impl<T> std::ops::IndexMut<usize> for IOMemory<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if self.watchlist.iter().find(|&&x| {x as usize==(index + self.reg_size)}).is_some() {
            self.write_status = true;
        }
        self.inner.index_mut(index)
    }
}