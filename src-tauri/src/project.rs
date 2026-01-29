use crate::emit;
use crate::error::{Error, Result};
use crate::sim::instruction::{Instruction, PartialInstruction};
use anyhow::anyhow;
use rusqlite::Connection;
use rusqlite::Error as SqlError;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::{Mutex, MutexGuard};
use strum::{EnumIter, IntoEnumIterator};
use tauri::Emitter;

pub static PROJECT: Mutex<Project> = Mutex::new(Project::new());
pub fn get_project() -> Result<MutexGuard<'static, Project>> {
    PROJECT.lock().map_err(|e| anyhow!("Poison Error :{}", e))
}

#[derive(Debug, EnumIter, Clone)]
#[allow(non_camel_case_types)]
pub enum Tables {
    instruction,
    project,
}
impl Display for Tables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self))
    }
}

pub struct Project {
    connection: Option<Connection>,
    pub state: Option<ProjectState>,
}

//db
impl Project {
    pub const fn new() -> Project {
        Project {
            connection: None,
            state: None,
        }
    }
    pub fn create(&mut self, path: &str) -> Result<()> {
        if std::fs::exists(path)? {
            return Err(anyhow!(Error::FileExists(path.to_string())));
        }
        self.open_conn(path)?;

        Tables::iter()
            .map(|t| self.create_table(t))
            .collect::<Result<()>>()
    }
    pub fn get_state(&mut self) -> Result<&mut ProjectState> {
        self.state.as_mut().ok_or(anyhow!(Error::ProjectNotOpened))
    }
    pub fn open(&mut self, path: &str) -> Result<()> {
        match self.open_db(path) {
            Ok(t) => Ok(t),
            Err(e) => {
                self.close()?;
                Err(e)
            }
        }
    }
    fn open_db(&mut self, path: &str) -> Result<()> {
        if self.connection.is_some() {
            return Err(anyhow!(Error::ProjectAlreadyOpened));
        }
        self.open_conn(path)?;
        Tables::iter()
            .map(|t| {
                if self.table_exists(t.clone()).is_err() {
                    self.create_table(t)?;
                }
                Ok(())
            })
            .collect::<Result<()>>()?;
        self.state = Some(*self.get_project()?);
        emit!(
            "asm-update",
            self.get_instruction_list()?
                .into_iter()
                .map(|x| PartialInstruction::from(x))
                .collect::<Vec<PartialInstruction>>()
        );

        emit!("project-update", self.get_project()?);
        Ok(())
    }
    fn open_conn(&mut self, path: &str) -> Result<()> {
        self.connection = Some(Connection::open(path)?);
        Ok(())
    }
    pub fn close(&mut self) -> Result<()> {
        self.connection = None;

        emit!("asm-update", ());

        emit!("project-update", ProjectState::default());
        Ok(())
    }
    pub fn save(&mut self) -> Result<()> {
        self.is_open()?;
        self.insert_project()?;
        Ok(())
    }
    pub fn is_open(&self) -> Result<()> {
        if self.connection.is_some() {
            return Ok(());
        }
        Err(anyhow!(Error::ProjectNotOpened))
    }
    pub fn create_table(&mut self, name: Tables) -> Result<()> {
        self.is_open()?;
        let query =
            std::fs::read_to_string(format!("{}/sql/{:?}.sql", env!("CARGO_MANIFEST_DIR"), name))?;
        self.connection.as_ref().unwrap().execute(&*query, ())?;
        Ok(())
    }
    pub fn table_exists(&self, name: Tables) -> Result<String> {
        self.is_open()?;
        let mut stmt = self
            .connection
            .as_ref()
            .unwrap()
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?")?;

        let r: String = stmt.query_one([name.to_string()], |x| x.get(0))?;
        Ok(r)
    }
    pub fn insert_instruction_list(&mut self, inst: &Vec<Instruction>) -> Result<()> {
        self.table_exists(Tables::instruction)?;
        self.connection
            .as_mut()
            .unwrap()
            .execute("DELETE FROM instruction", [])?;
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
                    serde_json::to_string(&i.comment_display)?,
                ))?;
            }
        }
        tx.commit()?;

        emit!(
            "asm-update",
            self.get_instruction_list()?
                .into_iter()
                .map(|x| PartialInstruction::from(x))
                .collect::<Vec<PartialInstruction>>()
        );

        Ok(())
    }

    pub fn get_instruction_list(&mut self) -> Result<Vec<Instruction>> {
        self.table_exists(Tables::instruction)?;
        let mut stmt = self
            .connection
            .as_ref()
            .unwrap()
            .prepare("SELECT * FROM instruction")?;
        let instructions = stmt
            .query_map([], |row| {
                let operands_json: String = row.get(3)?;
                let operands = serde_json::from_str(&operands_json)
                    .map_err(|e| SqlError::UserFunctionError(Box::new(e)))?;

                let comment_json: String = row.get(5)?;
                let comment_display = serde_json::from_str(&comment_json)
                    .map_err(|e| SqlError::UserFunctionError(Box::new(e)))?;

                let i = Instruction {
                    address: row.get(0)?,
                    opcode_id: row.get(1)?,
                    raw_opcode: row.get(2)?,
                    operands,
                    break_point: false,
                    comment: row.get(4)?,
                    comment_display,
                };

                Ok(i)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        match self.state {
            Some(ref state) => {
                if state.mcu.is_empty() {
                    return Ok(instructions);
                }
                instructions
                    .into_iter()
                    .map(|mut x| {
                        x.gen_comment(&state)?;
                        Ok(x)
                    })
                    .collect::<Result<Vec<Instruction>>>()
            }
            None => Err(anyhow!(Error::ProjectNotOpened)),
        }
    }

    fn get_project(&mut self) -> Result<Box<ProjectState>> {
        //                self.connection.as_ref().unwrap().prepare("INSERT INTO project (text) VALUES (?)")?
        self.table_exists(Tables::project)?;
        let mut stmt = self
            .connection
            .as_ref()
            .unwrap()
            .prepare("SELECT * FROM project")?;
        let proj = match stmt.query_one([], |row| Ok(ProjectState::from(row.get(0)?))) {
            Ok(project_state) => Ok(project_state),
            Err(SqlError::QueryReturnedNoRows) => {
                let mut instert_stmt = self
                    .connection
                    .as_ref()
                    .unwrap()
                    .prepare("INSERT INTO project (text) VALUES (?)")?;
                let proj = ProjectState::default();
                let r = instert_stmt.execute([serde_json::to_string(&proj)?])?;
                if r != 1 {
                    return Err(anyhow!(Error::ProjectError(
                        "querry returned more than 1 row"
                    )));
                }
                Ok(proj)
            }
            Err(e) => Err(e),
        }?;
        Ok(Box::from(proj))
    }
    pub fn insert_project(&mut self) -> Result<()> {
        self.table_exists(Tables::project)?;
        let mut stmt = self
            .connection
            .as_ref()
            .unwrap()
            .prepare("UPDATE project SET text =?")?;
        stmt.execute([serde_json::to_string(&self.state.clone())?])?;
        Ok(())
    }
    pub fn get_eeprom_data(&mut self) -> Result<Vec<u8>> {
        Ok(vec![])
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProjectState {
    pub name: String,
    pub mcu: String,
    pub freq: u32,
}

impl FromSql for ProjectState {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        serde_json::from_str::<ProjectState>(value.as_str()?)
            .map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}
