use anyhow::anyhow;
use quote::{quote, ToTokens};
use crate::AvrDeviceFile;
use crate::r#struct::module::Register;

#[derive(Debug,Clone,Copy)]
pub struct CommonReg {
    pub register: &'static Register,
    pub data:*mut u8
}
static mut ZERO : u8 = 0;
impl Default for CommonReg {
    fn default() -> Self {
        let d ;
        unsafe {
            d= &raw mut ZERO;
        }
        CommonReg {
            register: Box::leak(Box::new(Register::default())),
            data: d
        }
    }
}




#[derive(Default,Debug,Clone, Copy)]
pub struct CommonRegisters{
    pub sreg:CommonReg,
    pub eind:CommonReg,
    pub rampx:CommonReg,
    pub rampy:CommonReg,
    pub rampz:CommonReg,
    pub rampd:CommonReg,
    pub pc:CommonReg,
    pub sp:CommonReg,
    pub mcucr:CommonReg,
}
impl CommonRegisters{
    pub fn init(atdf:&AvrDeviceFile,data:&mut Vec<u8>)->Result<Self,anyhow::Error>{
        let mut s = Self::default();
        let reg_list = s.get_reg_list();
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
            .map(|register| {

                let (_, v) = s.iter_mut().find(|(key,_)| {key.to_uppercase()==register.name.to_uppercase()}).ok_or_else(|| anyhow!("invalid register:{0}", register.name))?;
                v.register = register;
                Ok(())
            }).collect::<Result<(),anyhow::Error>>()?;

        s.init_regs(atdf,data)?;

        Ok(s)
    }
    fn get_reg_list(&mut self)->Vec<String>{
        self.iter_mut().map(|(key,_)| {
            key.to_uppercase()
        }).collect()
    }
    pub fn init_regs(&mut self,atdf:&AvrDeviceFile, data:&mut Vec<u8>)->Result<(),anyhow::Error>{
        for (_,value)in self.iter_mut(){
            if value.register.name ==""{
                continue;
            }
            let addr = value.register.offset;
            let register_count = atdf.devices.address_spaces.iter().find(|x| { x.id == "data" }).unwrap()
                .memory_segments.iter().find(|x1| { x1.name == "REGISTERS" }).unwrap().size;
            //warn!("a:{},B:{},c:{:?}",addr,register_count,value);
            value.data = data.get_mut((addr - register_count) as usize).ok_or(anyhow!(format!("invalid reg addr:{}",addr)))?;

        }
        Ok(())
    }
    pub fn iter_mut(&mut self)
                    -> impl Iterator<Item = (&'static str, &mut CommonReg)>
    {
        let sreg = ("sreg", &mut self.sreg);
        let eind = ("eind", &mut self.eind);
        let pc = ("pc", &mut self.pc);
        let sp = ("sp", &mut self.sp);
        let rampx = ("rampx", &mut self.rampx);
        let rampy = ("rampy", &mut self.rampy);
        let rampz = ("rampz", &mut self.rampz);
        let rampd = ("rampd", &mut self.rampd);
        let mcucr = ("mcucr", &mut self.mcucr);

        vec![sreg,eind,pc,sp,rampx,rampy, rampz, rampd,mcucr].into_iter()
    }
}

impl ToTokens for CommonReg{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let register = &self.register;
        tokens.extend(quote!{
            crate::r#struct::common_registers::CommonReg{
                register:&#register,
                data:super::null::<u8>() as *mut u8
            }
        })
    }
}
impl ToTokens for CommonRegisters {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let sreg = &self.sreg;
        let eind = &self.eind;
        let rampx = &self.rampx;
        let rampy = &self.rampy;
        let rampz = &self.rampz;
        let rampd = &self.rampd;
        let pc = &self.pc;
        let sp = &self.sp;
        let mcucr = &self.mcucr;

        tokens.extend(quote! {
            crate::r#struct::common_registers::CommonRegisters{
                sreg: #sreg,
                eind: #eind,
                rampx: #rampx,
                rampy: #rampy,
                rampz: #rampz,
                rampd: #rampd,
                pc: #pc,
                sp: #sp,
                mcucr: #mcucr,
            }
        })
    }
}