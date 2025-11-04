use tauri::ipc::Invoke;
use opcodeGen::RawInst;
use crate::project::{ProjectState, PROJECT};

pub(crate) const HANDLER: fn(Invoke) -> bool = tauri::generate_handler![
    get_instruction_list,
    get_mcu_list,
    set_mcu,
    get_project_info
];

#[tauri::command]
pub fn get_instruction_list() -> Option<Vec<RawInst>> {
    println!("test");
    Some(Vec::from(opcodeGen::Opcode_list))
}

#[tauri::command]
pub fn get_mcu_list() -> Option<Vec<&'static String>> {
    match deviceParser::get_mcu_list() {
        Ok(list) => Some(list),
        Err(_) => None,
    }
}

#[tauri::command]
pub fn set_mcu(mcu:String) {
    PROJECT.lock().unwrap().state.set_mcu(mcu).expect("TODO: panic message");
}

#[tauri::command]
pub fn get_project_info() -> Option<ProjectState> {
    Some(PROJECT.lock().unwrap().state.clone())
}