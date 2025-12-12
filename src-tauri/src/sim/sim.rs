use std::ops::Deref;
use anyhow::anyhow;
use rusqlite::fallible_iterator::Iterator;
use deviceParser::{get_common_registers, get_tree_map, AvrDeviceFile, CommonRegisters, Register};
use opcodeGen::{CustomOpcodes, Opcode, RawInst};
use crate::sim::memory::Memory;
use crate::error::Result;
use crate::project::PROJECT;
use crate::sim::instruction::Instruction;
use crate::sim::sim::RamSize::Size16;

enum RamSize {
    Size16,
    Size24,
}
impl Default for RamSize {
    fn default() -> Self {
        Size16
    }
}


#[derive(Default)]
pub struct Sim{
    pub memory: Memory,
    registers: CommonRegisters,
    ram_size: RamSize,
}
impl Sim {
    pub fn init(&mut self) ->Result<()>{
        let mcu = &PROJECT.lock().unwrap().get_project()?.mcu;
        let atdf = get_tree_map().get(mcu).ok_or(anyhow!("invalid mcu"))?;
        let inst = PROJECT.lock().unwrap().get_instruction_list()?;
        let mut inst_vec: Vec<Instruction>=Vec::new();
        inst_vec.resize((atdf.devices.address_spaces.iter().find(|x| {x.id=="prog"}).unwrap().size/2) as usize,Instruction::decode_from_opcode(CustomOpcodes::EMPTY as u16)?);
        inst.into_iter().map(|x|{
            let raw_inst = RawInst::get_inst_from_id(x.opcode_id)?;
            let address = x.address.clone();
            inst_vec.insert(x.address as usize, x);
            if(raw_inst.len==2){
                inst_vec.insert((address as usize)+1,Instruction::decode_from_opcode(CustomOpcodes::REMINDER as u16)?);
            }
            Ok(())
        }).collect::<Result<()>>()?;
        self.memory.init(atdf,inst_vec,PROJECT.lock().unwrap().get_eeprom_data()?)?;
        self.registers = *(get_common_registers(mcu).ok_or(anyhow!("mcu not supported"))?);
        self.registers.init_regs(atdf,&mut self.memory.data.io)?;
        Ok(())
    }
    pub fn execute_inst(&mut self,instruction: &Instruction) -> Result<()>{
        let op1 = match &instruction.operands{
            Some(o) => match o.get(0){Some(v)=>v.value.clone(),None=>0},
            None=>0
        };
        let op2 = match &instruction.operands{
            Some(o) => match o.get(1){Some(v)=>v.value.clone(),None=>0},
            None=>0
        };
        let op3 = match &instruction.operands{
            Some(o) => match o.get(2){Some(v)=>v.value.clone(),None=>0},
            None=>0
        };
        let ind1 = op1 as usize;
        let ind2 = op2 as usize;
        let ind3 = op3 as usize;

        /*unsafe {
            match instruction.get_raw_inst()?.name {
                Opcode::ADC => {
                    self.memory.data.registers[ind1] = self.memory.data.registers[ind1]+self.memory.data.registers[ind2];
                    Ok(())
                }
                Opcode::ADD => {}
                Opcode::ADIW => {}
                Opcode::AND => {}
                Opcode::ANDI => {}
                Opcode::ASR => {}
                Opcode::BCLR => {}
                Opcode::BLD => {}
                Opcode::BRBC => {}
                Opcode::BRBS => {}
                Opcode::BRCC => {}
                Opcode::BRCS => {}
                Opcode::BREQ => {}
                Opcode::BRGE => {}
                Opcode::BRHC => {}
                Opcode::BRHS => {}
                Opcode::BRID => {}
                Opcode::BRIE => {}
                Opcode::BRLO => {}
                Opcode::BRLT => {}
                Opcode::BRMI => {}
                Opcode::BRNE => {}
                Opcode::BRPL => {}
                Opcode::BRSH => {}
                Opcode::BRTC => {}
                Opcode::BRTS => {}
                Opcode::BRVC => {}
                Opcode::BRVS => {}
                Opcode::BSET => {}
                Opcode::BST => {}
                Opcode::BREAK => {}
                Opcode::CALL => {}
                Opcode::CBI => {}
                Opcode::CBR => {}
                Opcode::CLC => {}
                Opcode::CLH => {}
                Opcode::CLI => {}
                Opcode::CLN => {}
                Opcode::CLR => {}
                Opcode::CLS => {}
                Opcode::CLT => {}
                Opcode::CLV => {}
                Opcode::CLZ => {}
                Opcode::COM => {}
                Opcode::CP => {}
                Opcode::CPC => {}
                Opcode::CPI => {}
                Opcode::CPSE => {}
                Opcode::CUSTOM_INST(_) => {}
                Opcode::DEC => {}
                Opcode::DES => {}
                Opcode::EICALL => {}
                Opcode::EIJMP => {}
                Opcode::ELPM => {}
                Opcode::EOR => {}
                Opcode::FMUL => {}
                Opcode::FMULS => {}
                Opcode::FMULSU => {}
                Opcode::ICALL => {}
                Opcode::IJMP => {}
                Opcode::IN => {}
                Opcode::INC => {}
                Opcode::JMP => {}
                Opcode::LAC => {}
                Opcode::LAS => {}
                Opcode::LAT => {}
                Opcode::LD => {}
                Opcode::LDD => {}
                Opcode::LDI => {}
                Opcode::LDS => {}
                Opcode::LPM => {}
                Opcode::LSL => {}
                Opcode::LSR => {}
                Opcode::MOV => {}
                Opcode::MOVW => {}
                Opcode::MUL => {}
                Opcode::MULS => {}
                Opcode::MULSU => {}
                Opcode::NEG => {}
                Opcode::NOP => {}
                Opcode::OR => {}
                Opcode::ORI => {}
                Opcode::OUT => {}
                Opcode::POP => {}
                Opcode::PUSH => {}
                Opcode::RCALL => {}
                Opcode::RET => {}
                Opcode::RETI => {}
                Opcode::RJMP => {}
                Opcode::ROL => {}
                Opcode::ROR => {}
                Opcode::SBC => {}
                Opcode::SBCI => {}
                Opcode::SBI => {}
                Opcode::SBIC => {}
                Opcode::SBIS => {}
                Opcode::SBIW => {}
                Opcode::SBR => {}
                Opcode::SBRC => {}
                Opcode::SBRS => {}
                Opcode::SEC => {}
                Opcode::SEH => {}
                Opcode::SEI => {}
                Opcode::SEN => {}
                Opcode::SER => {}
                Opcode::SES => {}
                Opcode::SET => {}
                Opcode::SEV => {}
                Opcode::SEZ => {}
                Opcode::SLEEP => {}
                Opcode::SPM => {}
                Opcode::ST => {}
                Opcode::STD => {}
                Opcode::STS => {}
                Opcode::SUB => {}
                Opcode::SUBI => {}
                Opcode::SWAP => {}
                Opcode::TST => {}
                Opcode::WDR => {}
                Opcode::XCH => {}
            }
        }*/
        Ok(())
    }
}