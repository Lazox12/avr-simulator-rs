use crate::project::PROJECT;

mod error;
mod menu;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod sim;
mod project;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)] // only enable instrumentation in development builds
    let devtools = tauri_plugin_devtools::init();


    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| menu::setup_menu(app));

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(devtools);
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
