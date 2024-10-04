//Uses
use crate::types::{Error, FileError, LinkedPath};
use notify::RecommendedWatcher;
use notify::Watcher;
use notify_debouncer_full::{new_debouncer, Debouncer, FileIdMap};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex as StdMutex};
use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_dialog::FilePath;
use tokio::sync::oneshot;
use tokio::sync::{broadcast, Mutex};
use tokio::time::Duration;

pub const PRIVATE_PATHS_FILE_PATH: &str = "../vault/private_paths.json";
pub const PUBLIC_PATHS_FILE_PATH: &str = "../vault/public_paths.json";

// Global variable to keep track of watched paths
lazy_static::lazy_static! {
    static ref WATCHED_PATHS: Mutex<HashSet<LinkedPath>> = Mutex::new(HashSet::new());
}

pub fn open_and_read_private_path_file() -> Result<Vec<LinkedPath>, FileError> {
    let data = fs::read_to_string(PRIVATE_PATHS_FILE_PATH).unwrap();
    let config_contents: Vec<LinkedPath> = serde_json::from_str(&data)?;
    Ok(config_contents)
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
    // Open a file and read it contents
    let mut linked_paths = open_and_read_private_path_file().unwrap();
    if name == "" {
        return Ok("Name your vault".to_string());
    };
    if linked_paths.iter().any(|x| x.name == name) {
        return Ok("Vault with this name already exists".to_string());
    };
    if path == "" {
        return Ok("Directory not selected".to_string());
    };
    // Define new path
    let new_linked_path = LinkedPath {
        name: name,
        path: PathBuf::from(path),
    };
    //Write new path
    linked_paths.push(new_linked_path);
    let data = serde_json::to_string(&linked_paths).unwrap();
    fs::write(PRIVATE_PATHS_FILE_PATH, data).unwrap();

    if let Err(e) = app.emit("linked_paths_changed", ()) {
        eprintln!("Failed to emit event to frontend: {}", e);
    }

    Ok("Directory linked successfully".to_string())
}

#[tauri::command]
pub fn unlink_directory(app: AppHandle, path_name: String) -> Result<String, FileError> {
    // Open a file and read it contents
    let mut linked_paths = open_and_read_private_path_file().unwrap();
    //Delete LinkedPath
    linked_paths.retain(|path| path.name != path_name);

    let data = serde_json::to_string(&linked_paths).unwrap();
    fs::write(PRIVATE_PATHS_FILE_PATH, data).unwrap();

    if let Err(e) = app.emit("linked_paths_changed", ()) {
        eprintln!("Failed to emit event to frontend: {}", e);
    }

    Ok("Directory removed successfully".to_string())
}

#[tauri::command]
pub fn get_linked_paths() -> Result<Vec<LinkedPath>, FileError> {
    // Open a file and read it contents
    let linked_paths = open_and_read_private_path_file().unwrap();
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
                                    == PathBuf::from(PRIVATE_PATHS_FILE_PATH)
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
        Path::new(PRIVATE_PATHS_FILE_PATH),
        notify::RecursiveMode::NonRecursive,
    ) {
        eprintln!("Failed to watch path {}: {}", PRIVATE_PATHS_FILE_PATH, e);
    } else {
        println!("Started watching path: {:?}", PRIVATE_PATHS_FILE_PATH);
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
    let linked_paths = open_and_read_private_path_file()?;
    let new_paths: HashSet<LinkedPath> = linked_paths.into_iter().collect();

    // Retrieve currently watched paths
    let mut watched_linked_paths = WATCHED_PATHS.lock().await;

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
