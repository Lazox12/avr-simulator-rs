use opcodeGen::RawInst;
#[tauri::command]
pub fn get_instruction_list() -> Option<Vec<RawInst>> {
    Some(Vec::from(opcodeGen::Opcode_list))
}