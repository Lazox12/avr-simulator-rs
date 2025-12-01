use anyhow::anyhow;
use deviceParser::{get_tree_map, AvrDeviceFile};
use opcodeGen::RawInst;
use crate::sim::memory::Memory;
use crate::error::Result;
use crate::project::PROJECT;
use crate::sim::instruction::Instruction;

struct Sim{
    memory: Memory,
}
impl Sim {
    pub fn new() -> Sim {
        Sim{memory: Memory::new()}
    }
    pub fn init(&mut self) ->Result<()>{
        let atdf = get_tree_map()?.get(&PROJECT.lock().unwrap().get_project()?.mcu);
        if(atdf.is_none()){
            return Err(anyhow!("AtDF is not initialized"))
        }
        let atdf = atdf.unwrap();
        let inst = PROJECT.lock().unwrap().get_instruction_list()?;
        let mut inst_vec: Vec<Instruction>=Vec::new();
        inst_vec.resize((atdf.devices.address_spaces.iter().find(|x| {x.id=="prog"}).unwrap().size/2) as usize,0u16);
        inst.iter().map(|x|{
            let raw_inst = RawInst::get_inst_from_id(x.opcode_id);
            if raw_inst.is_none() {
                return Err(anyhow!("unknown instruction {}", x.opcode_id));
            }
            let raw_inst = raw_inst.unwrap();
            inst_vec.insert(x.address as usize, *x);
            if(raw_inst.len==2){
                inst_vec.insert((x.address as usize)+1,Instruction::new("reminder from previous instruction",));
            }
            Ok(())
        }).collect::<Result<()>>()?;
        self.memory.init(atdf,inst_vec,PROJECT.lock().unwrap().get_eeprom_data()?)?;

        Ok(())
    }
}