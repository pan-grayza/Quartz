// Modules
mod local_dir;
mod server;
mod types;

// Uses
use local_dir::{
    get_linked_paths, link_directory, open_and_read_private_path_file, select_directory,
    setup_file_watcher, unlink_directory, PRIVATE_PATHS_FILE_PATH,
};
use server::{file_server, start_file_server_command};
use std::fs::*;
use std::io::prelude::*;
use std::path::Path;
use tokio::sync::broadcast;
use types::{LinkedPath, ServerMode};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            select_directory,
            link_directory,
            unlink_directory,
            get_linked_paths,
            start_file_server_command
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            // Ensure folders and configs are created
            if !Path::new("../vault").is_dir() {
                std::fs::create_dir("../vault").expect("Failed to create vault directory");
            }
            if !Path::new(PRIVATE_PATHS_FILE_PATH).exists() {
                let mut file = File::create(PRIVATE_PATHS_FILE_PATH)
                    .expect("Failed to create private_paths file");
                let linked_paths = serde_json::to_string_pretty::<Vec<LinkedPath>>(&vec![])
                    .expect("Failed to serialize linked paths");
                file.write_all(linked_paths.as_bytes())
                    .expect("Failed to write to private_paths file");
            }
            let app_handle_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let (file_watcher_tx, _file_watcher_rx) = broadcast::channel(32);
                let (priv_paths_file_change_tx, priv_paths_file_change_rx) =
                    std::sync::mpsc::channel::<()>();
                // Initialize the file watcher
                if let Err(e) =
                    setup_file_watcher(app_handle_clone, file_watcher_tx, priv_paths_file_change_tx)
                        .await
                {
                    eprintln!("Error setting up file watcher: {}", e);
                }
                let linked_paths: Vec<LinkedPath> = open_and_read_private_path_file().unwrap();
                file_server(ServerMode::LocalHost, linked_paths).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
