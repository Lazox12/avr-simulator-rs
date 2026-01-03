use anyhow::anyhow;
use opcode_gen::Opcode;
use crate::sim::core::Core;
use crate::sim::instruction::Instruction;
use crate::Result;
use crate::sim::sim::Sim;
use crate::sim::sim::RamSize;

fn get_time(core: Core, inst: Instruction, sim: Sim) -> Result<u8> {
    let err = Err(anyhow!("not supperted on this core"));
    match inst.get_raw_inst()?.name{
        Opcode::ADD|
        Opcode::ADC|
        Opcode::SUB|
        Opcode::SUBI|
        Opcode::SBC|
        Opcode::SBCI=>{Ok(1)}

        Opcode::ADIW|
        Opcode::SBIW=>{match core {
        Core::AVRrc=>err,
        _ => Ok(2)
        }}
        Opcode::AND|
        Opcode::ANDI|
        Opcode::OR|
        Opcode::ORI|
        Opcode::EOR|

        Opcode::COM|
        Opcode::NEG|
        Opcode::SBR|
        Opcode::CBR|
        Opcode::INC|
        Opcode::DEC|
        Opcode::TST|
        Opcode::CLR|
        Opcode::SER=>{Ok(1)}
        Opcode::MUL|
        Opcode::MULS|
        Opcode::MULSU|
        Opcode::FMUL|
        Opcode::FMULS|
        Opcode::FMULSU=> {match core {
        Core::AVRrc=>err,
        _ => Ok(2)
        }}
        Opcode::DES =>{match core {
            Core::AVRxm=>{
                match sim.memory.flash.get((inst.address-1) as usize) {
                    None => { Ok(2)}
                    Some(i) => {
                        if i.get_raw_inst()?.name ==Opcode::DES{
                            Ok(1)
                        }else{
                            Ok(2)
                        }
                    }
                }
            },
            _ => err
        }}
        Opcode::RJMP|
        Opcode::IJMP=>{Ok(2)}
        Opcode::EIJMP => {match core {
            Core::AVRrc=>err,
            _ => Ok(2)
        }}
        Opcode::JMP =>{Ok(3)}
        Opcode::RCALL|
        Opcode::ICALL=>{
            match sim.ram_size {
                RamSize::Size16 => {
                    match core {
                        Core::AVR|
                        Core::AVRe|
                        Core::AVRep => {
                            Ok(3)
                        }
                        Core::AVRxm => {
                            Ok(2)
                        }
                        Core::AVRxt => {
                            Ok(2)
                        }
                        Core::AVRrc => {
                            Ok(3)
                        }
                    }
                }
                RamSize::Size24 => {
                    match core {
                        Core::AVR|
                        Core::AVRe|
                        Core::AVRep => {
                            Ok(4)
                        }
                        Core::AVRxm => {
                            Ok(3)
                        }
                        Core::AVRxt => {
                            Ok(3)
                        }
                        Core::AVRrc => {
                            err
                        }
                    }
                }
            }
        }
        Opcode::EICALL=> match core {
            Core::AVR|
            Core::AVRe|
            Core::AVRep => {Ok(4)}
            Core::AVRxm => {Ok(3)}
            Core::AVRxt => {Ok(3)}
            Core::AVRrc => {err}
        }
        Opcode::CALL =>{
            match sim.ram_size {
                RamSize::Size16 => {
                    match core {
                        Core::AVR|
                        Core::AVRe|
                        Core::AVRep => {
                            Ok(4)
                        }
                        Core::AVRxm => {
                            Ok(3)
                        }
                        Core::AVRxt => {
                            Ok(3)
                        }
                        Core::AVRrc => {
                            err
                        }
                    }
                }
                RamSize::Size24 => {
                    match core {
                        Core::AVR|
                        Core::AVRe|
                        Core::AVRep => {
                            Ok(5)
                        }
                        Core::AVRxm => {
                            Ok(4)
                        }
                        Core::AVRxt => {
                            Ok(4)
                        }
                        Core::AVRrc => {
                            err
                        }
                    }
                }
            }
        }
        Opcode::RET|
        Opcode::RETI=>{
            match sim.ram_size {
                RamSize::Size16 => {
                    match core {
                        Core::AVR|
                        Core::AVRe|
                        Core::AVRep => {
                            Ok(4)
                        }
                        Core::AVRxm => {
                            Ok(4)
                        }
                        Core::AVRxt => {
                            Ok(4)
                        }
                        Core::AVRrc => {
                            Ok(6)
                        }
                    }
                }
                RamSize::Size24 => {
                    match core {
                        Core::AVR|
                        Core::AVRe|
                        Core::AVRep => {
                            Ok(5)
                        }
                        Core::AVRxm => {
                            Ok(5)
                        }
                        Core::AVRxt => {
                            Ok(5)
                        }
                        Core::AVRrc => {
                            err
                        }
                    }
                }
            }
        }

        Opcode::CPSE|
        Opcode::SBRC|
        Opcode::SBRS|
        Opcode::SBIC|
        Opcode::SBIS=>{
            match(sim.memory.program_couter - inst.address){
                1=>{Ok(1)}
                2=>{Ok(2)}
                3=>{match core {
                    Core::AVRrc=>err,
                    _ => Ok(3)
                }}
                _=>err
            }
        }
        Opcode::CP|
        Opcode::CPC|
        Opcode::CPI =>{Ok(1)}
        Opcode::BRBS|
        Opcode::BRBC|
        Opcode::BREQ|
        Opcode::BRNE|
        Opcode::BRCS|
        Opcode::BRCC|
        Opcode::BRSH|
        Opcode::BRLO|
        Opcode::BRMI|
        Opcode::BRPL|
        Opcode::BRGE|
        Opcode::BRLT|
        Opcode::BRHS|
        Opcode::BRHC|
        Opcode::BRTS|
        Opcode::BRTC|
        Opcode::BRVS|
        Opcode::BRVC|
        Opcode::BRIE|
        Opcode::BRID=>{
            if sim.memory.program_couter - inst.address==1{ // todo k=0
                Ok(1)
            }else{
                Ok(2)
            }
        }
        Opcode::MOV|
        Opcode::LDI=> {Ok(1)}
        Opcode::MOVW=>{
            match core {
                Core::AVRrc=>err,
                _ => Ok(1)
            }
        }
        Opcode::LDS=>{
            match core {
                Core::AVR|
                Core::AVRe|
                Core::AVRep => {Ok(2)}
                Core::AVRxm => {Ok(3)}
                Core::AVRxt => {Ok(3)} //todo
                Core::AVRrc => {Ok(2)}
            }
        }
        Opcode::LD=>{
            match inst.operands.unwrap()[2].value {
                0=>{
                    match core {
                        Core::AVR|
                        Core::AVRe|
                        Core::AVRep => {Ok(2)}
                        Core::AVRxm => {Ok(2)}
                        Core::AVRxt => {Ok(2)} //todo
                        Core::AVRrc => {Ok(2)}//todo 1/2
                    }
                }
                1=>{//X+
                    match core {
                        Core::AVR|
                        Core::AVRe|
                        Core::AVRep => {Ok(2)}
                        Core::AVRxm => {Ok(2)}
                        Core::AVRxt => {Ok(2)} //todo
                        Core::AVRrc => {Ok(2)} //todo 2/3
                    }
                }
                2=>{ //-X
                    match core {
                        Core::AVR|
                        Core::AVRe|
                        Core::AVRep => {Ok(2)}
                        Core::AVRxm => {Ok(3)}
                        Core::AVRxt => {Ok(2)} //todo
                        Core::AVRrc => {Ok(2)}//todo 2/3
                    }
                }
                _=>err
            }
        }
        Opcode::LDD =>{
            match core {
                Core::AVR|
                Core::AVRe|
                Core::AVRep => {Ok(2)}
                Core::AVRxm => {Ok(3)} //todo
                Core::AVRxt => {Ok(2)}
                Core::AVRrc => err
            }
        }
        Opcode::STS=>{
            match core {
                Core::AVR|
                Core::AVRe|
                Core::AVRep => {Ok(2)}
                Core::AVRxm => {Ok(2)}
                Core::AVRxt => {Ok(2)}
                Core::AVRrc => {Ok(1)}

            }
        }
        Opcode::ST =>{
            match core {
                Core::AVR|
                Core::AVRe|
                Core::AVRep => {Ok(2)}
                Core::AVRxt => {Ok(1)}
                Core::AVRxm|
                Core::AVRrc =>{match inst.operands.unwrap()[2].value {
                    2=>{Ok(2)}
                    _=>{Ok(1)}
                }}

            }
        }
        Opcode::STD =>{
            match core {
                Core::AVR|
                Core::AVRe|
                Core::AVRep => {Ok(2)}
                Core::AVRxm => {Ok(2)}
                Core::AVRxt => {Ok(1)}
                Core::AVRrc => err
            }
        }
        Opcode::LPM|
        Opcode::ELPM=>{
            match core {
                Core::AVRrc => {err}
                _=>Ok(3)
            }
        }
        Opcode::SPM=>{Ok(1)} //todo


        Opcode::IN|
        Opcode::OUT =>{Ok(1)}

        Opcode::PUSH=>{
            match core{
                Core::AVR|
                Core::AVRe|
                Core::AVRep => {Ok(2)}
                _=>Ok(1)
            }
        }
        Opcode::POP=>{
            match core{
                Core::AVRrc => {Ok(3)}
                _=>Ok(2)
            }
        }
        Opcode::XCH|
        Opcode::LAS|
        Opcode::LAC|
        Opcode::LAT=>{
            match core{
                Core::AVRxm => {Ok(2)}
                _=>err
            }
        }
        Opcode::LSL|
        Opcode::LSR|
        Opcode::ROL|
        Opcode::ROR|
        Opcode::ASR|
        Opcode::SWAP=>{Ok(1)}
        Opcode::SBI|
        Opcode::CBI=>{
            match core{
                Core::AVR|
                Core::AVRe|
                Core::AVRep => {Ok(2)}
                _=>Ok(1)
            }
        }
        Opcode::BST|
        Opcode::BLD|
        Opcode::BSET|
        Opcode::BCLR|
        Opcode::SEC|
        Opcode::CLC|
        Opcode::SEN|
        Opcode::CLN|
        Opcode::SEZ|
        Opcode::CLZ|
        Opcode::SEI|
        Opcode::CLI|
        Opcode::SES|
        Opcode::CLS|
        Opcode::SEV|
        Opcode::CLV|
        Opcode::SET|
        Opcode::CLT|
        Opcode::SEH|
        Opcode::CLH=>{Ok(1)}

        Opcode::BREAK|
        Opcode::NOP|
        Opcode::SLEEP|
        Opcode::WDR=> {Ok(1)}

        Opcode::CUSTOM_INST(_)=> {err}

    }
}
