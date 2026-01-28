use std::collections::HashMap;
use anyhow::anyhow;
use quote::{quote, ToTokens};
use crate::AvrDeviceFile;
use crate::r#struct::module::Register;

static mut ZERO : u8 = 0;

#[derive(Debug,Clone,Copy)]
pub struct CommonReg {
    pub register: &'static Register,
    data:Option<*mut u8>
}


impl Default for CommonReg {
    fn default() -> Self {
        unsafe {
            CommonReg {
                register: Box::leak(Box::new(Register::default())),
                data: None
            }
        }
    }
}
impl CommonReg {
    pub const fn new(reg :&'static Register)->Self{
        unsafe{
            Self{ register: reg, data: None }
        }
    }
    pub unsafe fn get_data(&self) -> u8 { unsafe {
        match self.data {
            Some(data) => data.read(),
            None => {panic!("read on uninitilized CommonReg")}
        }
    }}
    pub unsafe fn set_data(&mut self, val: u8) { unsafe {
        match self.data {
            Some(data) => {
                data.write(val);
            }
            None => {panic!("write to uninitilized CommonReg")}
        }
    }}
    pub unsafe fn try_get(&self) -> Option<u8> { unsafe {
        match self.data {
            Some(data) => Some(data.read()),
            None => None
        }
    }}
    pub unsafe fn try_set(&self,value:u8) -> Option<()> { unsafe {
        match self.data {
            Some(data) => {
                data.write(value);
                Some(())
            }
            None => {None}
        }
    }}
}


#[allow(non_snake_case)]
#[derive(Default,Debug,Clone, Copy)]
pub struct CommonRegisters{
    pub sreg:CommonReg,
    pub eind:CommonReg,
    pub rampx:CommonReg,
    pub rampy:CommonReg,
    pub rampz:CommonReg,
    pub rampd:CommonReg,
    pub spL:CommonReg,
    pub spH:CommonReg,
    pub mcucr:CommonReg,
}
impl CommonRegisters{
    pub fn init(atdf:&AvrDeviceFile,reg_map:&HashMap<u64,&'static Register>,data:&mut Vec<u8>)->Result<Self,anyhow::Error>{
        let mut s = Self::default();
        let reg_list = s.get_reg_list();
        reg_map
            .iter()
            .filter(|(_,value)|{ reg_list.iter().find(|x| {value.name.to_lowercase()==*x.to_lowercase()}).is_some()})
            .map(|(_,register)| { //todo does not work
                //warn!("{}",register.name);
                let (_, v) = s.iter_mut().find(|(key,_)| {key.to_lowercase()==register.name.to_lowercase()}).ok_or_else(|| anyhow!("invalid register:{0}", register.name))?;
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
            value.data = Some(data.get_mut((addr - register_count) as usize).ok_or(anyhow!(format!("invalid reg addr:{}",addr)))?);

        }
        Ok(())
    }
    pub fn iter_mut(&mut self)
                    -> impl Iterator<Item = (&'static str, &mut CommonReg)>
    {
        let sreg = ("sreg", &mut self.sreg);
        let eind = ("eind", &mut self.eind);
        let spl = ("spl", &mut self.spL);
        let sph = ("sph", &mut self.spH);
        let rampx = ("rampx", &mut self.rampx);
        let rampy = ("rampy", &mut self.rampy);
        let rampz = ("rampz", &mut self.rampz);
        let rampd = ("rampd", &mut self.rampd);
        let mcucr = ("mcucr", &mut self.mcucr);

        vec![sreg,eind,spl,sph,rampx,rampy, rampz, rampd,mcucr].into_iter()
    }
    pub unsafe fn get_flag(&self,flag:Flags)->bool{ unsafe {
        flag.get_value(self.sreg.get_data())
    }}
    pub unsafe fn set_flag(&mut self, flag:Flags,value: bool) { unsafe {
        self.sreg.set_data(flag.set_value(self.sreg.get_data(),value))
    }}
}
type FlagDataType=bool;
pub enum Flags{
    I,//interrupt enable
    T,//Bit Copy Storage
    H,//Half Carry
    S,//Sign flag S = N xor V
    V,//Two’s Complement Overflow Flag
    N,//Negative Flag
    Z,//Zero Flag
    C,//Carry Flag
}
impl Flags {
    pub fn get_value(self, data:u8) -> bool{
        (data>>self.get_bit())&1==1
    }
    pub fn set_value(self, data:u8,value:bool)->u8{
        let idx = self.get_bit();
        let mask = !(1 << idx);
        let flag = (value as u8) << idx;
        data & mask | flag
    }
    pub const fn get_bit(self)->u8{
        match self {
            Flags::I => {7}
            Flags::T => {6}
            Flags::H => {5}
            Flags::S => {4}
            Flags::V => {3}
            Flags::N => {2}
            Flags::Z => {1}
            Flags::C => {0}
        }
    }
    pub fn get_flag(data:u8)->Result<Flags,anyhow::Error>{
        match data {
            7 => Ok(Flags::I),
            6 => Ok(Flags::T),
            5 => Ok(Flags::H),
            4 => Ok(Flags::S),
            3 => Ok(Flags::V),
            2 => Ok(Flags::N),
            1 => Ok(Flags::Z),
            0 => Ok(Flags::C),
            _ => Err(anyhow!("invalid flag value"))
        }
    }
}

impl ToTokens for CommonReg{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let register = &self.register;
        tokens.extend(quote!{
            crate::r#struct::common_registers::CommonReg::new(&#register)
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
        let spl = &self.spL;
        let sph = &self.spH;
        let mcucr = &self.mcucr;

        tokens.extend(quote! {
            crate::r#struct::common_registers::CommonRegisters{
                sreg: #sreg,
                eind: #eind,
                rampx: #rampx,
                rampy: #rampy,
                rampz: #rampz,
                rampd: #rampd,
                spL: #spl,
                spH: #sph,
                mcucr: #mcucr,
            }
        })
    }
}