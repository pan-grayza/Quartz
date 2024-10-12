// Modules
mod local_dir;
mod server;
mod types;

// Uses
use local_dir::{
    create_local_network, get_linked_paths, link_directory, read_private_networks, remove_network,
    select_directory, setup_file_watcher, unlink_directory, PRIVATE_CONFIG_FILE_PATH,
};
use serde_json::json;
use server::{start_file_server_command, stop_file_server_command};
use std::fs::*;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use types::ServerState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(Arc::new(Mutex::new(ServerState::default())))
        .invoke_handler(tauri::generate_handler![
            select_directory,
            link_directory,
            unlink_directory,
            get_linked_paths,
            start_file_server_command,
            stop_file_server_command,
            read_private_networks,
            create_local_network,
            remove_network
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            // Ensure folders and configs are created
            if !Path::new("../configs").is_dir() {
                std::fs::create_dir("../configs").expect("Failed to create configs directory");
            }
            if !Path::new(PRIVATE_CONFIG_FILE_PATH).exists() {
                let mut file = File::create(PRIVATE_CONFIG_FILE_PATH)
                    .expect("Failed to create private_config file");
                let data = json!({
                    "linked_paths": [],
                    "networks" : [],
                });
                let json_content = serde_json::to_string_pretty(&data)
                    .expect("Failed to serialize local networks");
                file.write_all(json_content.as_bytes())
                    .expect("Failed to write to private_config file");
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
                loop {
                    match priv_paths_file_change_rx.try_recv() {
                        Ok(msg) => println!("Received: {:?}", msg),
                        Err(std::sync::mpsc::TryRecvError::Empty) => {
                            println!("No more messages yet");
                            break;
                        }
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            println!("Channel closed");
                            break;
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
