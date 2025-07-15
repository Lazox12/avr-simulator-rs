mod error;
mod menu;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod sim;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| menu::setup_menu(app))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
