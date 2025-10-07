use std::cell::OnceCell;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Manager};
use crate::project::PROJECT;

mod error;
mod menu;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod sim;
mod project;
mod commands;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

pub static APP_HANDLE: OnceLock<Mutex<AppHandle>> = OnceLock::new();
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)] // only enable instrumentation in development builds
    let devtools = tauri_plugin_devtools::init();


    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![commands::get_instruction_list])
        .setup(|app| {
            APP_HANDLE
                .set(Mutex::new(app.app_handle().to_owned()))
                .unwrap();
            menu::setup_menu(app)
        });

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(devtools);
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
