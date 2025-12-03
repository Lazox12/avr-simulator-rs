use anyhow::{anyhow, format_err};
use deviceParser::{AvrDeviceFile,Register as parserReg};
use crate::error::Result;

#[derive(Debug,Default)]
struct Register{
    pub register: Option<parserReg>,
    pub data:*mut u8,
}

#[derive(Debug,Default)]
pub struct CommonRegisters{
    pub sreg:Register,
    pub eind:Register,
    pub rampx:Register,
    pub rampy:Register,
    pub rampz:Register,
    pub rampd:Register,
    pub pc:Register,
    pub sp:Register,
    pub mcucr:Register,
}
impl CommonRegisters{

    pub fn init(&mut self,atdf:&AvrDeviceFile,data:&mut Vec<u8>)->Result<()>{
        let reg_list = self.get_reg_list();

        atdf
            .modules
            .iter()
            .find(|module|{module.name=="CPU"})
            .ok_or(anyhow!("no CPU module found"))?
            .register_group
            .iter()
            .find(|register|{register.name=="CPU"})
            .ok_or(anyhow!("no CPU module found"))?
            .register
            .iter()
            .filter(|register|{ reg_list.iter().find(|x| {register.name==**x}).is_some()})
            .map(|register|{
                match self
                    .into_iter()
                    .find(|(key,_)| {*key==register.name}){
                    Some((key,value)) => value.register = Some(register.clone()),
                    None => {},
                }
                Ok(())
            }).collect::<Result<()>>()?;

        self.init_regs(atdf,data)?;

        Ok(())
    }

    fn init_regs(&mut self,atdf:&AvrDeviceFile, data:&mut Vec<u8>)->Result<()>{
        self.into_iter().map(|(key,value)| unsafe {
            match &value.register{
                Some(register) => {
                    let addr = register.offset;
                    let register_count = atdf.devices.address_spaces.iter().find(|x| {x.id=="data"}).unwrap()
                        .memory_segments.iter().find(|x1| {x1.name=="REGISTERS"}).unwrap().size;
                    value.data = data.as_mut_ptr().add((addr-register_count) as usize);
                    Ok(())
                }
                None => Ok(()),
            }
        }).collect::<Result<()>>()
    }

    fn get_reg_list(&mut self)->Vec<String>{
        self.into_iter().map(|(key,_)| {
            key.to_uppercase()
        }).collect()
    }
}
impl<'a> IntoIterator for &'a mut CommonRegisters{
    type Item = (&'a str,&'a mut Register);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let CommonRegisters{
            sreg,eind, rampx, rampy, rampz,rampd,pc,sp,mcucr
        } = self;
        vec![("sreg",sreg), ("sreg",eind),("rampx",rampx),("rampy",rampy),("rampz",rampz),("rampd",rampd),("pc",pc),("sp",sp),("mcucr",mcucr)].into_iter()
    }
}