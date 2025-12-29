use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct ConstraintMap{
    map:u32,
    constraint:char
}
struct Inst {
    opcode: String,
    len:i8,
    name:String,
    constraints:Option<Vec<ConstraintMap>>,
    action:String,
    description:String,
}


impl Inst {
    #[allow(dead_code)]
    /*fn print(&self)->String {
        let mut s:String = String::from("");
        s+="opcode:";
        s+=&*self.opcode;
        s+=" len:";
        s+= &*self.len.to_string();
        s+=" name:";
        s+=&*self.name;
        if self.constraints.is_some() {
            s+=" constraints:";
            s+= &*self.constraints.clone().unwrap().iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",");
        }
        s
    }*/
    fn calculate_bin_mask(&self)->u32{
        let mut bin_mask: u32 = 0;
        for i in self.opcode.chars(){
            if i=='0'||i=='1' {
                bin_mask<<=1;
                bin_mask+=1;
                continue
            }
            bin_mask<<=1;
        }
        bin_mask
    }
    fn calculate_bin_opcode(&self)->u32{
        let mut opcode: u32 = 0;
        for i in self.opcode.chars(){
            if i=='1' {
                opcode<<=1;
                opcode+=1;
                continue
            }
            opcode<<=1;
        }
        opcode
    }
    fn gen_to_array(&self) ->String {
        let mut s:String = String::from("RawInst{");
        s+="opcode:\"";
        s+=&*self.opcode;
        s+="\" ,len:";
        s+= &*self.len.to_string();
        s+=" ,name:Opcode::";
        s+=&*self.name.to_uppercase();
        s+=",constraints: ";
        if self.constraints.is_some() {
            s+="Some(&[";
            for c in self.constraints.as_ref().unwrap() {
                //s+="'";
                s+="ConstraintMap{map:";
                s+=c.map.to_string().as_str();
                s+=",constraint:'";
                s+=c.constraint.to_string().as_str();
                s+="'},";
            }
            s+="])";
            
        }
        else{
            s+="None";
        }
        s+=" ,bin_mask:";
        s+=self.calculate_bin_mask().to_string().as_str();
        s+=" ,bin_opcode:";
        s+=self.calculate_bin_opcode().to_string().as_str();
        s+=" ,action:\"";
        s+=&*self.action;
        s+="\" ,description:\"";
        s+=&*self.description;
        s+="\"},\n";
        s
    }
}
fn main(){

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("opcode.rs");
    let mut r:Vec<Inst> = vec![];
    let mut inst_list:HashSet<String> = HashSet::new();
    inst_list.insert("CUSTOM_INST(u32)".to_string());
    for line in fs::read_to_string("src/opcode.def").unwrap().lines(){
        if line.starts_with("//") {
            continue;
        }
        if line.trim().len()==0 {
            continue;
        }
        let v:Vec<&str> = line.split(";").collect();
        if v.len()<4 {
            println!("cargo::warning={}", line);

        }
        let mut chars: HashSet<char> = v[0].chars().collect();
        chars.remove(&'0');
        chars.remove(&'1');
        let mut constraints:Vec<&str> = vec![];
        if v.len()==6 {
            constraints = v[3].split(",").filter(|s| s.len() > 0).collect();
        }
        let mut c:Vec<ConstraintMap> = vec![];
        println!("{:?}", constraints);
         constraints.iter().for_each(|s| {
                let ch = s.chars().next().unwrap();
                if chars.iter().find(|x| **x == ch).is_none() {
                    c.push(ConstraintMap{ map: 0,constraint:ch});
                    return;
                }

                let mut map = 0;
                for i in v[0].to_string().chars(){
                    map = map<<1;
                    if i == ch {
                        map+=1;
                    }
                    
                }
             if v[1].parse::<i8>().unwrap()==2 {
                 map = map<<16;
                 
             }
                if map>0 {
                    chars.remove(&ch);
                }
                c.push(ConstraintMap { constraint: ch, map });
            });
        print!("{}", v[2]);
        println!("{:?}", c);


        inst_list.insert(v[2].to_string().to_uppercase());
        c.iter_mut().for_each(|map: &mut ConstraintMap| {
            if map.map >0 {
                return;
            }
            let op_ch = chars.iter().next();
            if op_ch.is_none() {
                if v[1].parse::<i8>().unwrap()==2 && map.constraint =='i' {
                    map.map =65535;
                }
                return;
            }
            let ch = op_ch.unwrap().clone();
            chars.remove(&ch);
            let mut count:u32 =0;
            for i in v[0].to_string().chars(){
                count = count<<1;
                if i == ch {
                    count+=1;
                }
            }
            if v[1].parse::<i8>().unwrap()==2 {
                count = count<<16;

            }
            map.map = count;
        });
        if c.len()==1 && (c[0].map&65535) == 0 {
            c[0].map +=65535;
        }
        
        if c.len()>0 {
            r.push(
            Inst{
                opcode:v[0].to_string(),
                len:v[1].parse::<i8>().unwrap(),
                name:v[2].to_string(),
                constraints:Some(c),
                action: v[v.len()-2].to_string(),
                description: v[v.len()-1].to_string(),
            });
        }else{
            r.push(
                Inst{
                    opcode:v[0].to_string(),
                    len:v[1].parse::<i8>().unwrap(),
                    name:v[2].to_string(),
                    constraints:None,
                    action: v[v.len()-2].to_string(),
                    description: v[v.len()-1].to_string(),
                });
        }
        
    };

    /*let mut s:String = String::from("pub struct RawInst{
    pub opcode:&'static str,
    pub len:i8,
    pub name:&'static str,
    pub constraints:Option<&'static [char]>,
    pub bin_mask:u16,
    pub bin_opcode:u16
}
");*/
    let mut s:String = String::from("");

    s+="#[derive(Debug,Serialize,Clone,PartialEq)]\n";
    s+="#[allow(non_camel_case_types)]\n";
    s+="pub enum Opcode{\n";
    inst_list.into_iter().for_each(|i| {
        s+= &*(i + ",\n");
    });
    s+="}\n\n";

    s+="pub const OPCODE_LIST:[RawInst;";
    s+= &*r.len().to_string();
    s+="]=[\n";
    for inst in r.iter() {
        s+= &*inst.gen_to_array();
    }
    s+="];";
    fs::write(&dest_path, s).unwrap();
}
