mod instruction;
mod operand;
mod constraint;

mod parser;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle();

            // my custom settings menu item

            let open = MenuItemBuilder::new("Open...")
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
                    if(file_path.is_some()){

                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
