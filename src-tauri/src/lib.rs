pub mod analysis;
pub mod commands;
pub mod credentials;
pub mod db;
pub mod models;

use commands::{
    analyze_checklist, archive_checklist_node, create_checklist_node, delete_openai_api_key,
    get_analysis_snapshot, get_default_checklist, get_latest_analysis_snapshot,
    get_openai_credential_status, rename_checklist_node, set_openai_api_key, set_task_checked,
    set_task_estimate, test_openai_connection,
};
use db::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new_default().expect("failed to initialize local database");

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_default_checklist,
            create_checklist_node,
            rename_checklist_node,
            set_task_checked,
            set_task_estimate,
            archive_checklist_node,
            analyze_checklist,
            get_latest_analysis_snapshot,
            get_analysis_snapshot,
            set_openai_api_key,
            get_openai_credential_status,
            delete_openai_api_key,
            test_openai_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
