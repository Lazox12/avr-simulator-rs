use tauri::ipc::Invoke;
use opcode_gen::RawInst;
use crate::project::{get_project, ProjectState};
use crate::sim::parser::parse_hex;
use crate::wrap_anyhow;
use crate::sim::controller::{Controller,Action};


pub(crate) const HANDLER: fn(Invoke) -> bool = tauri::generate_handler![
    get_instruction_list,
    get_mcu_list,
    set_project_data,
    get_project_info,
    menu_new,
    menu_open,
    menu_import,
    menu_close,
    menu_save,
    sim_action
];


wrap_anyhow!(get_instruction_list() -> Vec<RawInst> {
    Ok(Vec::from(opcode_gen::OPCODE_LIST))
});


wrap_anyhow!(get_mcu_list() -> &'static [&'static str] {
    Ok(device_parser::get_mcu_list())
});


wrap_anyhow!(set_project_data(project:ProjectState) ->(){

    get_project()?.state = Some(project);
    Ok(())
});


wrap_anyhow!(get_project_info() -> ProjectState {
    get_project()?.get_state().cloned()
});



wrap_anyhow!(menu_new(file:String)->(){
    get_project()?.create(&*file.to_string())
});

wrap_anyhow!(menu_open(file:String)->(){
    get_project()?.open(&*file.to_string())
});

wrap_anyhow!(menu_import(file:String)->(){
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
    get_project()?.insert_instruction_list(&result)
});

wrap_anyhow!(menu_close()->(){
    get_project()?.close()
});

wrap_anyhow!(menu_save()->(){
    get_project()?.save()
});

wrap_anyhow!(sim_action(action:Action)->(){
   Controller::do_action(action)
});
