use tauri::State;

use crate::{
    analysis::analyze_checklist_with_state,
    credentials,
    db::AppState,
    models::{
        AnalysisSnapshotDto, AnalyzeChecklistInput, ArchiveChecklistNodeInput, ChecklistTreeDto,
        CommandError, CreateChecklistNodeInput, OpenAiConnectionResultDto,
        OpenAiCredentialStatusDto, RenameChecklistNodeInput, SetTaskCheckedInput,
        SetTaskEstimateInput,
    },
};

#[tauri::command]
pub fn get_default_checklist(state: State<'_, AppState>) -> Result<ChecklistTreeDto, CommandError> {
    state.with_db(|db| db.get_default_checklist())
}

#[tauri::command]
pub fn create_checklist_node(
    state: State<'_, AppState>,
    input: CreateChecklistNodeInput,
) -> Result<ChecklistTreeDto, CommandError> {
    state.with_db(|db| db.create_checklist_node(input))
}

#[tauri::command]
pub fn rename_checklist_node(
    state: State<'_, AppState>,
    input: RenameChecklistNodeInput,
) -> Result<ChecklistTreeDto, CommandError> {
    state.with_db(|db| db.rename_checklist_node(input))
}

#[tauri::command]
pub fn set_task_checked(
    state: State<'_, AppState>,
    input: SetTaskCheckedInput,
) -> Result<ChecklistTreeDto, CommandError> {
    state.with_db(|db| db.set_task_checked(input))
}

#[tauri::command]
pub fn set_task_estimate(
    state: State<'_, AppState>,
    input: SetTaskEstimateInput,
) -> Result<ChecklistTreeDto, CommandError> {
    state.with_db(|db| db.set_task_estimate(input))
}

#[tauri::command]
pub fn archive_checklist_node(
    state: State<'_, AppState>,
    input: ArchiveChecklistNodeInput,
) -> Result<ChecklistTreeDto, CommandError> {
    state.with_db(|db| db.archive_checklist_node(input))
}

#[tauri::command]
pub async fn analyze_checklist(
    state: State<'_, AppState>,
    input: AnalyzeChecklistInput,
) -> Result<AnalysisSnapshotDto, CommandError> {
    analyze_checklist_with_state(state.inner(), input).await
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_latest_analysis_snapshot(
    state: State<'_, AppState>,
    checklist_id: String,
) -> Result<Option<AnalysisSnapshotDto>, CommandError> {
    state.with_db(|db| db.get_latest_analysis_snapshot(&checklist_id))
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_analysis_snapshot(
    state: State<'_, AppState>,
    snapshot_id: String,
) -> Result<AnalysisSnapshotDto, CommandError> {
    state.with_db(|db| db.get_analysis_snapshot(&snapshot_id))
}

#[tauri::command(rename_all = "camelCase")]
pub fn set_openai_api_key(
    state: State<'_, AppState>,
    api_key: String,
) -> Result<OpenAiCredentialStatusDto, CommandError> {
    credentials::set_openai_api_key(state.inner(), &api_key)
}

#[tauri::command]
pub fn get_openai_credential_status(
    state: State<'_, AppState>,
) -> Result<OpenAiCredentialStatusDto, CommandError> {
    credentials::get_openai_credential_status(state.inner())
}

#[tauri::command]
pub fn delete_openai_api_key(
    state: State<'_, AppState>,
) -> Result<OpenAiCredentialStatusDto, CommandError> {
    credentials::delete_openai_api_key(state.inner())
}

#[tauri::command]
pub async fn test_openai_connection(
    state: State<'_, AppState>,
) -> Result<OpenAiConnectionResultDto, CommandError> {
    credentials::test_openai_connection(state.inner()).await
}
