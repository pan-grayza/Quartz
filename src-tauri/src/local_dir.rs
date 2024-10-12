use crate::types::LocalNetwork;
//Uses
use crate::types::{Error, FileError, LinkedPath, Network};
use notify::RecommendedWatcher;
use notify::Watcher;
use notify_debouncer_full::{new_debouncer, Debouncer, FileIdMap};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex as StdMutex};
use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_dialog::FilePath;
use tokio::sync::oneshot;
use tokio::sync::{broadcast, Mutex};
use tokio::time::Duration;

pub const PRIVATE_CONFIG_FILE_PATH: &str = "../configs/private_config.json";
pub const PUBLIC_LINKEDPATH_FILE_PATH: &str = "../configs/public_paths.json";

// Global variable to keep track of watched paths
lazy_static::lazy_static! {
    static ref WATCHED_LINKEDPATHS: Mutex<HashSet<LinkedPath>> = Mutex::new(HashSet::new());
}

pub fn read_private_config() -> Result<Value, FileError> {
    let data = fs::read_to_string(PRIVATE_CONFIG_FILE_PATH).unwrap();
    let config_contents: Value = serde_json::from_str(&data)?;

    Ok(config_contents)
}
pub fn read_private_linked_paths() -> Result<Vec<LinkedPath>, FileError> {
    let config_contents = read_private_config().unwrap();
    if let Some(linked_paths_value) = config_contents.get("linked_paths") {
        let linked_paths: Vec<LinkedPath> = serde_json::from_value(linked_paths_value.clone())
            .expect("Failed to deserialize linked_paths");
        Ok(linked_paths)
    } else {
        Err(FileError::MissingLinkedPathsError)
    }
}
#[tauri::command]
pub fn read_private_networks() -> Result<Vec<Network>, FileError> {
    let config_contents = read_private_config().unwrap();
    if let Some(networks_value) = config_contents.get("networks") {
        let networks: Vec<Network> =
            serde_json::from_value(networks_value.clone()).expect("Failed to deserialize networks");
        Ok(networks)
    } else {
        Err(FileError::MissingLinkedPathsError)
    }
}

fn write_json_to_file(json_value: &Value) -> Result<(), FileError> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true) // Truncate the file to overwrite it
        .open(PRIVATE_CONFIG_FILE_PATH)?;

    let updated_json_str = serde_json::to_string_pretty(json_value)?;
    file.write_all(updated_json_str.as_bytes())?;

    Ok(())
}

#[tauri::command]
pub async fn select_directory(app: AppHandle) -> Result<Option<PathBuf>, Error> {
    let (tx, rx) = oneshot::channel::<Option<PathBuf>>();

    // Use pick_folder to allow the user to select a folder
    app.dialog().file().pick_folder(move |folder_path| {
        // Capture the selected folder path inside the closure
        let selected_dir = match folder_path {
            Some(FilePath::Path(path)) => Some(path),
            Some(FilePath::Url(url)) => url.to_file_path().ok(), // Convert Url to PathBuf
            None => None,
        };

        let _ = tx.send(selected_dir);
    });

    let selected_dir = rx.await?;

    Ok(selected_dir)
}

#[tauri::command]
pub fn link_directory(app: AppHandle, path: String, name: String) -> Result<String, FileError> {
    if name == "" {
        return Ok("Name your linked path".to_string());
    };
    if path == "" {
        return Ok("Directory not selected".to_string());
    };

    let mut json_value = read_private_config()?;

    // Modify the `linked_paths` field without altering other fields
    if let Some(linked_paths_value) = json_value.get_mut("linked_paths") {
        // Deserialize `linked_paths` into a Vec<LinkedPath>
        let mut linked_paths: Vec<LinkedPath> = serde_json::from_value(linked_paths_value.clone())?;
        if linked_paths.iter().any(|x| x.name == name) {
            return Ok("Linked path with this name already exists".to_string());
        };
        let new_linked_path = LinkedPath {
            name: name,
            path: PathBuf::from(path),
        };
        // Add the new path
        linked_paths.push(new_linked_path);

        // Serialize the updated `linked_paths` back into the JSON value
        *linked_paths_value = serde_json::to_value(&linked_paths)?;
    } else {
        return Err(FileError::MissingLinkedPathsError);
    }

    // Write the updated JSON back to the file
    write_json_to_file(&json_value)?;

    if let Err(e) = app.emit("linked_paths_changed", ()) {
        eprintln!("Failed to emit event to frontend: {}", e);
    }

    Ok("Directory linked successfully".to_string())
}

#[tauri::command]
pub fn unlink_directory(app: AppHandle, path_name: String) -> Result<String, FileError> {
    let mut json_value = read_private_config()?;

    // Modify the `linked_paths` field without altering other fields
    if let Some(linked_paths_value) = json_value.get_mut("linked_paths") {
        // Deserialize `linked_paths` into a Vec<LinkedPath>
        let mut linked_paths: Vec<LinkedPath> = serde_json::from_value(linked_paths_value.clone())?;

        // Filter out the target item based on `name`
        linked_paths.retain(|path| path.name != path_name);

        // Serialize the updated `linked_paths` back into the JSON value
        *linked_paths_value = serde_json::to_value(&linked_paths)?;
    } else {
        return Err(FileError::MissingLinkedPathsError);
    }

    // Write the updated JSON back to the file
    write_json_to_file(&json_value)?;

    if let Err(e) = app.emit("linked_paths_changed", ()) {
        eprintln!("Failed to emit event to frontend: {}", e);
    }

    Ok("Directory removed successfully".to_string())
}
#[tauri::command]
pub fn create_local_network(
    app: AppHandle,
    name: String,
    linked_paths: Vec<LinkedPath>,
) -> Result<String, FileError> {
    if name == "" {
        return Ok("Name your network".to_string());
    };
    if linked_paths.is_empty() {
        return Ok("Paths not selected".to_string());
    };

    let mut json_value = read_private_config()?;

    // Modify the `networks` field without altering other fields
    if let Some(networks_value) = json_value.get_mut("networks") {
        let mut networks: Vec<Network> = serde_json::from_value(networks_value.clone())?;
        let new_network = Network::LocalNetwork(LocalNetwork {
            name,
            port: 3030,
            linked_paths,
        });
        networks.push(new_network);
        *networks_value = serde_json::to_value(&networks)?;
    } else {
        return Err(FileError::MissingLinkedPathsError);
    }

    // Write the updated JSON back to the file
    write_json_to_file(&json_value)?;

    if let Err(e) = app.emit("linked_paths_changed", ()) {
        eprintln!("Failed to emit event to frontend: {}", e);
    }

    Ok("Network created successfully".to_string())
}

#[tauri::command]
pub fn remove_network(app: AppHandle, network_to_remove: Network) -> Result<String, FileError> {
    let mut json_value = read_private_config()?;

    // Modify the `networks` field without altering other fields
    if let Some(networks_value) = json_value.get_mut("linked_paths") {
        let mut networks: Vec<LinkedPath> = serde_json::from_value(networks_value.clone())?;
        match network_to_remove {
            Network::LocalNetwork(remove_network) => {
                networks.retain(|network| network.name != remove_network.name);
            }
            Network::InternetNetwork(remove_network) => {
                networks.retain(|network| network.name != remove_network.name);
            }
            Network::DarkWebNetwork(remove_network) => {
                networks.retain(|network| network.name != remove_network.name);
            }
        }

        *networks_value = serde_json::to_value(&networks)?;
    } else {
        return Err(FileError::MissingLinkedPathsError);
    }

    // Write the updated JSON back to the file
    write_json_to_file(&json_value)?;

    if let Err(e) = app.emit("linked_paths_changed", ()) {
        eprintln!("Failed to emit event to frontend: {}", e);
    }

    Ok("Directory removed successfully".to_string())
}

#[tauri::command]
pub fn get_linked_paths() -> Result<Vec<LinkedPath>, FileError> {
    // Open a file and read it contents
    let linked_paths = read_private_linked_paths().unwrap();
    // println!("{}", linked_paths[0].name);

    Ok(linked_paths)
}

// Function to set up the file watcher
pub async fn setup_file_watcher(
    app_handle: AppHandle,
    tx: broadcast::Sender<LinkedPath>,
    priv_paths_file_change_tx: std::sync::mpsc::Sender<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a channel to receive file events
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();

    // Specify the generic parameters for Debouncer
    let debouncer = new_debouncer(Duration::from_secs(1), None, notify_tx)?;
    println!("Debouncer initialized");

    // Wrap channels in Arc and Mutex for concurrent access
    let tx = Arc::new(Mutex::new(tx));
    let notify_rx = Arc::new(StdMutex::new(notify_rx));
    let debouncer = Arc::new(StdMutex::new(debouncer));

    // Spawn blocking task to handle file events
    let tx_clone = Arc::clone(&tx);
    let notify_rx_clone = Arc::clone(&notify_rx);
    let debouncer_clone = Arc::clone(&debouncer);
    let app_handle_clone_1 = app_handle.clone();
    let app_handle_clone_2 = app_handle.clone();

    tokio::task::spawn_blocking(move || {
        println!("Starting blocking task to handle file events");
        let notify_rx = notify_rx_clone.lock().unwrap();
        println!("Acquired lock on notify_rx");

        while let Ok(events) = notify_rx.recv() {
            match events {
                Ok(debounced_events) => {
                    println!("Received debounced events");
                    for debounced_event in debounced_events {
                        for path in &debounced_event.paths {
                            if let Ok(canonical_path) = path.canonicalize() {
                                if canonical_path
                                    == PathBuf::from(PRIVATE_CONFIG_FILE_PATH)
                                        .canonicalize()
                                        .unwrap()
                                {
                                    // File was changed, reload linked paths
                                    let tx_clone = Arc::clone(&tx_clone);
                                    let debouncer_clone = Arc::clone(&debouncer_clone);
                                    println!("private_paths.json changed");
                                    if let Err(e) = priv_paths_file_change_tx.send(()) {
                                        eprintln!("Error sending file change signal: {}", e);
                                    }
                                    tokio::runtime::Runtime::new().unwrap().block_on(async {
                                        if let Err(e) = handle_file_change(
                                            app_handle_clone_1.clone(),
                                            &debouncer_clone,
                                            &tx_clone,
                                        )
                                        .await
                                        {
                                            eprintln!("Error handling file change: {}", e);
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
                Err(errors) => {
                    eprintln!("File watch error: {:?}", errors);
                }
            }
        }
    });

    // Watch the VAULT_PATH initially
    if let Err(e) = debouncer.lock().unwrap().watcher().watch(
        Path::new(PRIVATE_CONFIG_FILE_PATH),
        notify::RecursiveMode::NonRecursive,
    ) {
        eprintln!("Failed to watch path {}: {}", PRIVATE_CONFIG_FILE_PATH, e);
    } else {
        println!("Started watching path: {:?}", PRIVATE_CONFIG_FILE_PATH);
    }

    // Initial load of paths and start watching them
    if let Err(e) = handle_file_change(app_handle_clone_2.clone(), &debouncer, &tx).await {
        eprintln!("Error setting up file watcher: {}", e);
    }

    Ok(())
}

// Handle changes to the files
async fn handle_file_change(
    app_handle: AppHandle,
    debouncer: &Arc<StdMutex<Debouncer<RecommendedWatcher, FileIdMap>>>,
    tx: &Arc<Mutex<broadcast::Sender<LinkedPath>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let linked_paths = read_private_linked_paths()?;
    let new_paths: HashSet<LinkedPath> = linked_paths.into_iter().collect();

    // Retrieve currently watched paths
    let mut watched_linked_paths = WATCHED_LINKEDPATHS.lock().await;

    // Identify paths to add and remove
    let paths_to_add: HashSet<_> = new_paths
        .difference(&watched_linked_paths)
        .cloned()
        .collect();
    let paths_to_remove: HashSet<_> = watched_linked_paths
        .difference(&new_paths)
        .cloned()
        .collect();

    // Add new paths to the watcher
    for linked_path in &paths_to_add {
        if let Err(e) = debouncer
            .lock()
            .unwrap()
            .watcher()
            .watch(&linked_path.path, notify::RecursiveMode::Recursive)
        {
            eprintln!("Failed to watch path {}: {}", linked_path.path.display(), e);
        } else {
            println!("Started watching path: {:?}", linked_path.path);
        }
        watched_linked_paths.insert(linked_path.clone());
    }

    // Remove paths from the watcher
    for linked_path in &paths_to_remove {
        if let Err(e) = debouncer
            .lock()
            .unwrap()
            .watcher()
            .unwatch(&linked_path.path)
        {
            eprintln!(
                "Failed to unwatch path {}: {}",
                linked_path.path.display(),
                e
            );
        } else {
            println!("Stopped watching path: {:?}", linked_path.path);
        }
        watched_linked_paths.remove(linked_path);
    }

    // Notify about the changes
    for linked_path in &paths_to_add {
        let tx = tx.lock().await;
        if let Err(e) = tx.send(linked_path.clone()) {
            eprintln!("Failed to send file change event: {}", e);
        }
    }

    // Emit event with updated paths
    if let Err(e) = app_handle.emit("linked_paths_changed", ()) {
        eprintln!("Failed to emit event to frontend: {}", e);
    } else {
        println!("Emitted event to frontend with updated paths");
    }

    Ok(())
}
