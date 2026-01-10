//! Bare - En markdown-nettleser med fokus på personvern
//!
//! Hovedmodul som starter Tauri-applikasjonen og registrerer commands.

mod commands;
mod markdown;

use log::info;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    info!("Starting Bare browser");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::render_markdown,
            commands::open_file,
            commands::get_welcome_content,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
