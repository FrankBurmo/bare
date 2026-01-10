//! Bare - En markdown-nettleser med fokus på personvern
//!
//! Hovedmodul som starter Tauri-applikasjonen og registrerer commands.

mod bookmarks;
mod commands;
mod fetcher;
mod markdown;
mod settings;

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
            commands::fetch_url,
            commands::resolve_url,
            // Bokmerker
            commands::get_bookmarks,
            commands::add_bookmark,
            commands::remove_bookmark,
            commands::is_bookmarked,
            // Innstillinger
            commands::get_settings,
            commands::update_settings,
            commands::zoom_in,
            commands::zoom_out,
            commands::zoom_reset,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
