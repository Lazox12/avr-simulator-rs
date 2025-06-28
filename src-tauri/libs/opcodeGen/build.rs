

use std::env;
use std::fs;
use std::path::Path;

struct Inst {
    opcode: String,
    len:i8,
    name:String,
    constraints:Option<Vec<char>>,
}


impl Inst {
    #[allow(dead_code)]
    fn print(&self)->String {
        let mut s:String = String::from("");
        s+="opcode:";
        s+=&*self.opcode;
        s+=" len:";
        s+= &*self.len.to_string();
        s+=" name:";
        s+=&*self.name;
        if(self.constraints.is_some()) {
            s+=" constraints:";
            s+= &*self.constraints.clone().unwrap().iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",");
        }
        s
    }
    fn calculate_bin_mask(&self)->u32{
        let mut bin_mask: u32 = 0;
        for i in self.opcode.chars(){
            if(i=='0'||i=='1'){
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
            if(i=='1'){
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
        s+=" ,name:\"";
        s+=&*self.name;
        s+="\" ,constraints: ";
        if(self.constraints.is_some()) {
            s+="Some(&[";
            for c in self.constraints.as_ref().unwrap() {
                s+="'";
                s+=c.to_string().as_str();
                s+="',";
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
        s+="},\n";
        s
    }
}
fn main(){

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("opcode.rs");
    let mut r:Vec<Inst> = vec![];
    for line in fs::read_to_string("src/opcode.def").unwrap().lines(){
        if(line.starts_with("//")){
            continue;
        }
        if(line.trim().len()==0){
            continue;
        }
        let v:Vec<&str> = line.split(";").collect();
        if(v.len()<4){
            println!("cargo::warning={}", line);

        }
        let c:Vec<char> = v[3].split(",")
            .collect::<Vec<_>>()
            .iter()
            .flat_map(|s| s.chars().next())
            .collect();
        if(c.len()>0){
            r.push(
            Inst{
                    opcode:v[0].to_string(),
                    len:v[1].parse::<i8>().unwrap(),
                    name:v[2].to_string(),
                    constraints:Some(c),
            });
        }else{
            r.push(
                Inst{
                    opcode:v[0].to_string(),
                    len:v[1].parse::<i8>().unwrap(),
                    name:v[2].to_string(),
                    constraints:None,
                });
        }
        
    };

    let mut s:String = String::from("pub struct RawInst{
    pub opcode:&'static str,
    pub len:i8,
    pub name:&'static str,
    pub constraints:Option<&'static [char]>,
    pub bin_mask:u32,
    pub bin_opcode:u32
}
");
    s+="pub const Opcode_list:[RawInst;";
    s+= &*r.len().to_string();
    s+="]=[\n";
    for inst in r.iter() {
        s+= &*inst.gen_to_array();
    }
    s+="];";
    fs::write(&dest_path, s).unwrap();
}
