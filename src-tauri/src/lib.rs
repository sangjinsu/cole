pub mod commands;
pub mod db;
pub mod models;
pub mod recommendations;
pub mod sources;

use commands::{
    create_obsidian_source, get_recommendation_flow, list_sources, list_tasks,
    mark_task_done_local, sync_obsidian_source,
};
use db::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new_default().expect("failed to initialize local database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            create_obsidian_source,
            sync_obsidian_source,
            list_sources,
            list_tasks,
            get_recommendation_flow,
            mark_task_done_local
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
