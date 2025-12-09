use std::cell::OnceCell;
use std::sync::{Mutex, MutexGuard, OnceLock};
use anyhow::anyhow;
use tauri::{AppHandle, Manager};
use crate::project::PROJECT;
use error::Result;

mod error;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod sim;
mod project;
mod commands;
mod macros;

static APP_HANDLE: OnceLock<Mutex<AppHandle>> = OnceLock::new();

pub fn get_app_handle() -> Result<MutexGuard<'static, AppHandle>> {
    APP_HANDLE.get().unwrap().lock().map_err(|e| anyhow!("Poison Error:{}",e))
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

    #[cfg(debug_assertions)] // only enable instrumentation in development builds
    let devtools = tauri_plugin_devtools::init();

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