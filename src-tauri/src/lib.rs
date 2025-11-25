use std::cell::OnceCell;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Manager};
use crate::project::PROJECT;

mod error;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod sim;
mod project;
mod commands;

pub static APP_HANDLE: OnceLock<Mutex<AppHandle>> = OnceLock::new();
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)] // only enable instrumentation in development builds
    let devtools = tauri_plugin_devtools::init();
    deviceParser::get_tree_map().unwrap();

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(commands::HANDLER)
        .setup(|app| {
            APP_HANDLE
                .set(Mutex::new(app.app_handle().to_owned()))
                .unwrap();
            Ok(())
        });

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(devtools);
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
