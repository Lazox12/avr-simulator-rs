use crate::error::Result;
use device_parser::AvrDeviceFile;
use opcode_gen::CustomOpcodes;
use crate::sim::instruction::Instruction;

#[derive(Debug)]
pub struct Memory {
    pub flash:Vec<Instruction>,
    pub data:DataMemory,
    pub eeprom: Vec<u8>,
    pub program_couter:u32
}
impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
impl Memory {
    pub fn new() -> Memory {
        Memory {flash: Vec::new(),data:DataMemory::new(),eeprom:Vec::new(),program_couter:0}
    }
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

#[derive(Debug)]
pub struct DataMemory {
    pub registers:Vec<u8>,
    pub io:Vec<u8>,
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
    pub fn new()->Self{
        DataMemory{
            
            registers: Vec::new(),
            io: Vec::new(),
            ram: Vec::new(),
        }
    }
    pub fn init(&mut self,atdf:&'static AvrDeviceFile) -> Result<()> {
        let address_space = atdf.devices.address_spaces.iter().find(|x| {x.id=="data"}).unwrap();
        let reg_size = address_space.memory_segments.iter().find(|x| {x.name=="REGISTERS"}).unwrap().size;
        let io_size = address_space.memory_segments.iter().find(|x| {x.name=="MAPPED_IO"}).unwrap().size;
        let ram_size = address_space.memory_segments.iter().find(|x| {x.name=="IRAM"}).unwrap().size;
        self.registers.resize(reg_size as usize, 0);
        self.io.resize(io_size as usize, 0);
        self.ram.resize(ram_size as usize, 0);
        Ok(())
    }

}

