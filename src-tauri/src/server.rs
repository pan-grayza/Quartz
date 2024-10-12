use crate::local_dir::PUBLIC_LINKEDPATH_FILE_PATH;
use crate::types::{LinkedPath, ServerMode, ServerState};
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use tauri::State;
use tokio::sync::mpsc::{self, Receiver};
use tokio::sync::Mutex;
use warp::Filter;

#[tauri::command]
pub async fn start_file_server_command(
    server_mode: ServerMode,
    linked_paths: Vec<LinkedPath>,
    state: State<'_, Arc<Mutex<ServerState>>>,
) -> Result<String, String> {
    if linked_paths.is_empty() {
        return Ok("Choose linked paths to share".into());
    }
    let mut server_state = state.lock().await;

    // If the server is already running, return an error
    if server_state.shutdown_tx.is_some() {
        return Err("Server is already running.".into());
    }
    let (shutdown_tx, shutdown_rx) = mpsc::channel(100);
    server_state.shutdown_tx = Some(shutdown_tx);

    tauri::async_runtime::spawn(async move {
        file_server(server_mode, linked_paths, shutdown_rx).await;
    });
    Ok("Server started!".into())
}

#[tauri::command]
pub async fn stop_file_server_command(
    state: State<'_, Arc<Mutex<ServerState>>>,
) -> Result<String, String> {
    let mut server_state = state.lock().await;

    if let Some(shutdown_tx) = server_state.shutdown_tx.take() {
        shutdown_tx
            .send(())
            .await
            .expect("Failed to send shutdown signal");
        Ok("Server stopped.".into())
    } else {
        Err("Server is not running.".into())
    }
}

pub async fn file_server(
    server_mode: ServerMode,
    linked_paths: Vec<LinkedPath>,
    mut shutdown_rx: Receiver<()>,
) {
    let mut file =
        File::create(PUBLIC_LINKEDPATH_FILE_PATH).expect("Failed to create private_paths file");
    {
        let linked_paths = serde_json::to_string_pretty::<Vec<LinkedPath>>(&linked_paths)
            .expect("Failed to serialize linked paths");
        file.write_all(linked_paths.as_bytes())
            .expect("Failed to write to private_paths file");
    }

    let default_route = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file(PUBLIC_LINKEDPATH_FILE_PATH));
    let mut combined_fs_routes = default_route.boxed();
    for linked_path in linked_paths {
        // Create a route for each Node
        let route = warp::path(linked_path.name.clone())
            .and(warp::fs::dir(linked_path.path.clone()))
            .boxed();

        // Combine all routes using the `or` combinator
        combined_fs_routes = route.or(combined_fs_routes).unify().boxed();
    }

    let routes = combined_fs_routes;
    match server_mode {
        ServerMode::LocalHost => {
            warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), async move {
                shutdown_rx.recv().await;
            });
        }
        ServerMode::Internet => {}
        ServerMode::DarkWeb => {}
    }
}
