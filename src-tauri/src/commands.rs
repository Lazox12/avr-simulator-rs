use tauri::ipc::Invoke;
use opcodeGen::RawInst;
use crate::project::{Project, ProjectState, PROJECT};
use crate::error::{Result};
use crate::sim::parser::parse_hex;

pub(crate) const HANDLER: fn(Invoke) -> bool = tauri::generate_handler![
    get_instruction_list,
    get_mcu_list,
    set_project_data,
    get_project_info,
    menu_new,
    menu_open,
    menu_import,
    menu_close,
    menu_save
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
pub fn set_project_data(project:ProjectState) ->Result<()>{

    PROJECT.lock()?.state = Some(project);
    Ok(())
}

#[tauri::command]
pub fn get_project_info() -> Result<ProjectState> {
    Ok(PROJECT.lock()?.get_state()?.clone())
}


#[tauri::command]
pub fn menu_new(file:String)->Result<()>{
    PROJECT.lock()?.create(&*file.to_string())
}
#[tauri::command]
pub fn menu_open(file:String)->Result<()>{
    PROJECT.lock()?.open(&*file.to_string())
}
#[tauri::command]
pub fn menu_import(file:String)->Result<()>{
    let result = parse_hex(file)?;

    result.iter().for_each(|i| {
        match &i.operands {
            Some(operands) => {
                print!("{:#x}:{1}, opcode: {2:#x} ,", i.address,RawInst::get_inst_from_id(i.opcode_id).unwrap().name, i.raw_opcode);
                operands.iter().for_each(|x| {
                    print!("{},", x);
                });
                println!();
            }
            None => println!("{:#x}:{1}, opcode: {2:#x} ,", i.address, RawInst::get_inst_from_id(i.opcode_id).unwrap().name, i.raw_opcode)
        }
    });
    PROJECT.lock()?.insert_instruction_list(&result)
}
#[tauri::command]
pub fn menu_close()->Result<()>{
    PROJECT.lock()?.close()
}
#[tauri::command]
pub fn menu_save()->Result<()>{
    PROJECT.lock()?.save()
}