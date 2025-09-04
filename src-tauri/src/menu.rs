use std::error::Error;
use tauri::{App, menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder}, Emitter};
use crate::sim::parser::parse_hex;
use tauri_plugin_dialog::DialogExt;

pub fn setup_menu(app:  &App) -> Result<(),Box<dyn Error>> {
    let open = MenuItemBuilder::new("Open")
        .id("open")
        .accelerator("CmdOrCtrlo")
        .build(app)?;

    // my custom app submenu
    let app_submenu = SubmenuBuilder::new(app, "file")
        .item(&open)
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
        if event.id() == open.id() {
            // emit a window event to the frontend
            let file_path = app.dialog().file()
                .add_filter("compiled binary (*.hex, *.elf)",&["hex","elf"])
                .add_filter("executable linkable format (*.elf)",&["elf"])
                .add_filter("intel hex (*.hex)",&["hex"])
                .add_filter("all files", &["*"])
                .blocking_pick_file();
            if file_path.is_some() {
                let result = parse_hex(file_path.unwrap().to_string());
                if result.is_err() {
                    eprint!("{}", format!("{}", result.err().unwrap()));
                    return;
                
                }
                let result = result.unwrap();
                
                
                for i in result.clone() { //print to console
                    if(i.operands.is_some()){
                        print!("{:#x}:{1}, opcode: {2:#x} ,",i.address,i.opcode.name,i.raw_opcode);
                        i.operands.unwrap().iter().for_each(|x| {
                            if(i.address==0x1bc){
                                print!("test");
                            }
                            print!("{},",x);
                        });
                        println!();
                    }else{
                        println!("{:#x}:{1}, opcode: {2:#x} ,",i.address,i.opcode.name,i.raw_opcode);
                    }
                }
                println!("calling fe");
                let res = app.emit("asm-update",result);
                if res.is_err() {
                    eprintln!("{}", format!("{}", res.err().unwrap()));
                }
                
            }
        }
    });

    Ok(())
}