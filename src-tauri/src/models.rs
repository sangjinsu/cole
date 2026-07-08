use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    Done,
    Blocked,
    Archived,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::Done => "done",
            Self::Blocked => "blocked",
            Self::Archived => "archived",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "done" => Self::Done,
            "blocked" => Self::Blocked,
            "archived" => Self::Archived,
            _ => Self::Todo,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SourceDto {
    pub id: String,
    pub name: String,
    pub source_type: String,
    pub vault_path: Option<String>,
    pub sync_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateObsidianSourceInput {
    pub name: String,
    pub vault_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaskDto {
    pub id: String,
    pub source_id: String,
    pub source_type: String,
    pub external_id: String,
    pub title: String,
    pub body: Option<String>,
    pub status: TaskStatus,
    pub due_at: Option<String>,
    pub tags: Vec<String>,
    pub source_location_json: String,
    pub raw_text_hash: String,
    pub sync_state: String,
    pub source_path: Option<String>,
    pub line_start: Option<i64>,
    pub estimated_minutes: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationTaskDto {
    pub task_id: String,
    pub title: String,
    pub source_type: String,
    pub estimated_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecommendationGroupDto {
    pub id: String,
    pub title: String,
    pub reason: String,
    pub tasks: Vec<RecommendationTaskDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationFlowDto {
    pub groups: Vec<RecommendationGroupDto>,
    pub summary: String,
    pub openui_response: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncResultDto {
    pub source_id: String,
    pub upserts: usize,
    pub warnings: Vec<String>,
}
