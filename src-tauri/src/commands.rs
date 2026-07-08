use std::path::PathBuf;

use tauri::State;

use crate::{
    db::AppState,
    models::{CreateObsidianSourceInput, RecommendationFlowDto, SourceDto, SyncResultDto, TaskDto},
    recommendations::build_recommendation_flow,
    sources::obsidian::parse_vault,
};

#[tauri::command]
pub fn create_obsidian_source(
    state: State<'_, AppState>,
    input: CreateObsidianSourceInput,
) -> Result<SourceDto, String> {
    state.with_db(|db| db.create_obsidian_source(input))
}

#[tauri::command]
pub fn list_sources(state: State<'_, AppState>) -> Result<Vec<SourceDto>, String> {
    state.with_db(|db| db.list_sources())
}

#[tauri::command]
pub fn sync_obsidian_source(
    state: State<'_, AppState>,
    source_id: String,
) -> Result<SyncResultDto, String> {
    state.with_db(|db| {
        let source = db.get_source(&source_id)?;
        let vault_path = source
            .vault_path
            .clone()
            .ok_or_else(|| "source does not have a vault path".to_string())?;
        let tasks = parse_vault(&source.id, &PathBuf::from(vault_path))?;
        let upserts = db.upsert_tasks(&tasks)?;
        Ok(SyncResultDto {
            source_id,
            upserts,
            warnings: vec![],
        })
    })
}

#[tauri::command]
pub fn list_tasks(state: State<'_, AppState>) -> Result<Vec<TaskDto>, String> {
    state.with_db(|db| db.list_tasks())
}

#[tauri::command]
pub fn get_recommendation_flow(
    state: State<'_, AppState>,
) -> Result<RecommendationFlowDto, String> {
    state.with_db(|db| {
        let tasks = db.list_tasks()?;
        Ok(build_recommendation_flow(&tasks))
    })
}

#[tauri::command]
pub fn mark_task_done_local(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<TaskDto, String> {
    state.with_db(|db| db.mark_task_done_local(&task_id))
}
