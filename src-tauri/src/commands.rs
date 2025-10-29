use opcodeGen::RawInst;
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