use crate::sim::controller::Controller;
use anyhow::anyhow;
use error::Result;
use std::sync::{Mutex, MutexGuard, OnceLock};
use tauri::{AppHandle, Manager};

mod error;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod macros;
mod project;
mod sim;

static APP_HANDLE: OnceLock<Mutex<AppHandle>> = OnceLock::new();

pub fn get_app_handle() -> Result<MutexGuard<'static, AppHandle>> {
    APP_HANDLE
        .get()
        .unwrap()
        .lock()
        .map_err(|e| anyhow!("Poison Error:{}", e))
}
pub fn set_app_title(app_title: &str) ->Result<()> {
    let title = format!("{} - avr simulator", app_title);
    println!("Setting app title to {}", title);
    get_app_handle()?
        .get_webview_window("main")
        .ok_or(anyhow!("Couldn't get window"))?
        .set_title(title.as_str())?;
    Ok(())
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)] // only enable instrumentation in development builds
    let devtools = tauri_plugin_devtools::init();
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(commands::HANDLER)
        .setup(|app| {
            APP_HANDLE
                .set(Mutex::new(app.app_handle().to_owned()))
                .unwrap();
            tauri::async_runtime::spawn(async move {
                loop {
                    Controller::update().unwrap();
                }
            });
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
