use crate::local_dir::PUBLIC_PATHS_FILE_PATH;
use crate::types::{LinkedPath, ServerMode};
use std::fs::File;
use std::io::prelude::*;
use warp::Filter;

#[tauri::command]
pub async fn start_file_server_command(
    server_mode: ServerMode,
    linked_paths: Vec<LinkedPath>,
) -> Result<String, String> {
    // Load the initial paths
    tauri::async_runtime::spawn(async {
        file_server(server_mode, linked_paths).await;
    });
    Ok("Server started!".into())
}
pub async fn file_server(server_mode: ServerMode, linked_paths: Vec<LinkedPath>) {
    let mut file =
        File::create(PUBLIC_PATHS_FILE_PATH).expect("Failed to create private_paths file");
    {
        let linked_paths = serde_json::to_string_pretty::<Vec<LinkedPath>>(&linked_paths)
            .expect("Failed to serialize linked paths");
        file.write_all(linked_paths.as_bytes())
            .expect("Failed to write to private_paths file");
    }

    let default_route = warp::path::end()
        .and(warp::get())
        .and(warp::fs::file(PUBLIC_PATHS_FILE_PATH));
    let mut combined_fs_routes = default_route.boxed();
    for linked_path in linked_paths {
        // Create a route for each LinkedPath
        let route = warp::path(linked_path.name.clone())
            .and(warp::fs::dir(linked_path.path.clone()))
            .boxed();

        // Combine all routes using the `or` combinator
        combined_fs_routes = route.or(combined_fs_routes).unify().boxed();
    }

    let routes = combined_fs_routes;
    match server_mode {
        ServerMode::LocalHost => {
            warp::serve(routes).bind(([127, 0, 0, 1], 3030)).await;
        }
        ServerMode::Internet => {}
        ServerMode::DarkWeb => {}
    }
}
