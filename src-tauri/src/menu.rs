use std::error::Error;
use tauri::{App, menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder}, Emitter};
use crate::sim::parser::parse_hex;
use tauri_plugin_dialog::DialogExt;
use opcodeGen::RawInst;
use crate::project::PROJECT;
use crate::sim::instruction::PartialInstruction;

pub fn setup_menu(app:  &App) -> Result<(),Box<dyn Error>> {
    let import_menu = MenuItemBuilder::new("Import")
        .id("import")
        .accelerator("CmdOrCtrlo")
        .build(app)?;
    
    let open_menu = MenuItemBuilder::new("Open")
        .id("open")
        .build(app)?;
    
    let new_menu = MenuItemBuilder::new("New")
        .id("new")
        .accelerator("CmdOrCtrln")
        .build(app)?;
    let close_menu = MenuItemBuilder::new("Close")
        .id("close")
        .build(app)?;
    // my custom app submenu
    let app_submenu = SubmenuBuilder::new(app, "file")
        .item(&new_menu)
        .item(&open_menu)
        .separator()
        .item(&close_menu)
        .item(&import_menu)
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .quit()
        .build()?;

    // ... any other submenus

    let menu = MenuBuilder::new(app)
        .items(&[
            &app_submenu,
            // ... include references to any other submenus
        ])
        .build()?;

    // set the menu
    app.set_menu(menu)?;

    // listen for menu item click events
    app.on_menu_event(move |app, event| {
        match event.id() {
            val if val == import_menu.id() => {


                // emit a window event to the frontend
                let file_path = app.dialog().file()
                    .add_filter("compiled binary (*.hex, *.elf)", &["hex", "elf"])
                    .add_filter("executable linkable format (*.elf)", &["elf"])
                    .add_filter("intel hex (*.hex)", &["hex"])
                    .add_filter("all files", &["*"])
                    .set_title("select data file")
                    .blocking_pick_file();
                if file_path.is_some() {
                    let result = match parse_hex(file_path.unwrap().to_string()) {
                        Err(e) => {
                            eprint!("{}", e);
                            return;
                        }
                        Ok(i) => i
                    };

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
                    
                    match PROJECT.lock().unwrap().insert_instruction_list(&result){
                        Ok(_) => {}
                        Err(e) => {
                            eprint!("{}", e);
                            return;
                        }
                    };
                    println!("calling fe");
                    let res = app.emit("asm-update", result.into_iter().map(|x| PartialInstruction::from(x)).collect::<Vec<PartialInstruction>>());
                    if res.is_err() {
                        eprintln!("{}", format!("{}", res.err().unwrap()));
                    }
                }
            }
            val if val == open_menu.id() =>{
                match app.dialog().file()
                    .add_filter("project files (*.spro)", &["spro"])
                    .add_filter("all files", &["*"])
                    .set_title("select project file")
                    .blocking_pick_file()
                {
                    None => {}
                    Some(path) => {
                        match PROJECT.lock().unwrap().open(&*path.to_string()){
                            Ok(_) => {}
                            Err(e) => {
                                eprint!("{}", e);
                                return;
                            }
                        }
                    }
                }
                
            }
            val if val == new_menu.id() =>{
                app.dialog().file()
                    .add_filter("project files (*.spro)",&["spro"])
                    .set_title("select project directory")
                    .save_file(|file_path| {
                        match file_path {
                            None => {}
                            Some(path) => {
                                match PROJECT.lock().unwrap().create(&*path.to_string()){
                                    Ok(_) => {}
                                    Err(e) => {
                                        eprint!("{}", e);
                                        return;
                                    }
                                }
                            }
                        }
                    });
            }
            val if val == close_menu.id() =>{
                PROJECT.lock().unwrap().close().unwrap();
            }
            val =>{
                eprint!("unknown id handler {:?}",val);
            }
        }
    });

    Ok(())
}