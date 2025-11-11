use tauri::ipc::Invoke;
use opcodeGen::RawInst;
use crate::project::{ProjectState, PROJECT};
use crate::error::{Result};

pub(crate) const HANDLER: fn(Invoke) -> bool = tauri::generate_handler![
    get_instruction_list,
    get_mcu_list,
    set_mcu,
    get_project_info
];

#[tauri::command]
pub fn get_instruction_list() -> Vec<RawInst> {
    Vec::from(opcodeGen::Opcode_list)
}

#[tauri::command]
pub fn get_mcu_list() -> Result<Vec<&'static String>> {
    Ok(deviceParser::get_mcu_list()?)
}

#[tauri::command]
pub fn set_mcu(mcu:String) ->Result<()>{

    PROJECT.lock()?.get_state()?.set_mcu(mcu)?;
    Ok(())
}

#[tauri::command]
pub fn get_project_info() -> Result<ProjectState> {
    Ok(PROJECT.lock()?.get_state()?.clone())
}