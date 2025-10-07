use std::sync::Mutex;
use crate::error::{Error, Result};
use rusqlite::{Connection};
use rusqlite::Error as SqlError;
use serde::Serialize;
use tauri::AppHandle;
use crate::APP_HANDLE;
use crate::sim::instruction::{Instruction, PartialInstruction};
use tauri::{App,Emitter};
pub static PROJECT: Mutex<Project> = Mutex::new(Project::new());


pub struct Project {
    connection: Option<Connection>,
}
impl Project {
    pub const fn new() -> Project {
        Project {connection: None}
    }
    pub fn create(&mut self,path:&str) -> Result<bool>{
        if(std::fs::exists(path)?){
            std::fs::remove_file(path)?;
        }
        self.open(path)?;
        self.create_table("instruction")
        
    }
    pub fn open(&mut self, path:&str) -> Result<bool>{
        if(self.connection.is_some()){
            return Err(Error::ProjectAlreadyOpened);
        }
        self.connection = Some(Connection::open(path)?);
        

        APP_HANDLE.get().unwrap().lock()?
            .emit("asm-update", self.get_instruction_list()?.into_iter().map(|x| PartialInstruction::from(x)).collect::<Vec<PartialInstruction>>())?;

        Ok(true)
    }
    pub fn close(&mut self) -> Result<bool>{
        self.connection = None;
        
        APP_HANDLE.get().unwrap().lock()?
            .emit("asm-update", ())?;
        
        Ok(true)
    }
    
    pub fn is_open(&self) -> Result<()>{
        if(self.connection.is_some()){
            return Ok(());
        }
        Err(Error::ProjectNotOpened)
    }
    pub fn create_table(&mut self,name:&str) -> Result<bool>{
        self.is_open()?;
        let query = std::fs::read_to_string("sql/".to_owned() +name+".sql")?;
        self.connection.as_ref().unwrap().execute(&*query, ())?;
        Ok(true)
    }
    pub fn table_exists(&mut self,name:&str) -> Result<String>{
        self.is_open()?;
        let mut stmt = self.connection.as_ref().unwrap().prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?")?;

        let r:String =stmt.query_one(&[name], |x|  x.get(0))?;
        Ok(r)
    }
    pub fn insert_instruction_list(&mut self,inst:&Vec<Instruction>) -> Result<bool>{
        self.table_exists("instruction")?;
        let tx = self.connection.as_mut().unwrap().transaction()?;
        {
        let mut stmt = tx.prepare("INSERT INTO instruction (address,opcode,RawOpcode,operands,comment,commentDisplay) VALUES (?,?,?,?,?,?)")?;
        for i in inst {
            stmt.execute((
                i.address,
                i.opcode_id,
                i.raw_opcode,
                serde_json::to_string(&i.operands)?,
                &i.comment,
                serde_json::to_string(&i.comment_display)?
            ))?;
        }
        }
        tx.commit()?;
        Ok(true)
    }
    pub fn get_instruction_list(&mut self) -> Result<Vec<Instruction>>{
        self.table_exists("instruction")?;
        let mut stmt = self.connection.as_ref().unwrap().prepare("SELECT * FROM instruction")?;
        let instructions = stmt.query_map([], |row| {
            let operands_json: String = row.get(3)?;
            let operands = serde_json::from_str(&operands_json)
                .map_err(|e| SqlError::UserFunctionError(Box::new(e)))?;

            let comment_json: String = row.get(5)?;
            let comment_display = serde_json::from_str(&comment_json)
                .map_err(|e| SqlError::UserFunctionError(Box::new(e)))?;

            Ok(Instruction {
                address: row.get(0)?,
                opcode_id: row.get(1)?,
                raw_opcode: row.get(2)?,
                operands,
                comment: row.get(4)?,
                comment_display,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;
        
        Ok(instructions)
        
    }
}


