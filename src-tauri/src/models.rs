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
#[serde(rename_all = "snake_case")]
pub enum ChecklistNodeKind {
    Task,
    Group,
}

impl ChecklistNodeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Group => "group",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "group" => Self::Group,
            _ => Self::Task,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistDto {
    pub id: String,
    pub title: String,
    pub revision: i64,
    pub checklist_hash: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistNodeDto {
    pub id: String,
    pub checklist_id: String,
    pub parent_id: Option<String>,
    pub kind: ChecklistNodeKind,
    pub title: String,
    pub status: Option<TaskStatus>,
    pub sort_key: i64,
    pub estimated_minutes: Option<i64>,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistTreeDto {
    pub checklist: ChecklistDto,
    pub nodes: Vec<ChecklistNodeDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateChecklistNodeInput {
    pub checklist_id: String,
    pub parent_id: Option<String>,
    pub kind: ChecklistNodeKind,
    pub title: String,
    pub estimated_minutes: Option<i64>,
    pub expected_revision: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RenameChecklistNodeInput {
    pub node_id: String,
    pub title: String,
    pub expected_revision: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetTaskCheckedInput {
    pub node_id: String,
    pub checked: bool,
    pub expected_revision: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetTaskEstimateInput {
    pub node_id: String,
    pub estimated_minutes: Option<i64>,
    pub expected_revision: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveChecklistNodeInput {
    pub node_id: String,
    pub cascade: bool,
    pub expected_revision: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommandErrorCode {
    ValidationError,
    StaleRevision,
    NotFound,
    InvalidNodeKind,
    ArchivedNode,
    CompletedAncestor,
    IncompleteDescendants,
    NonEmptyNode,
    AnalysisError,
    CredentialError,
    DatabaseError,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: CommandErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_revision: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_checklist_hash: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descendant_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ancestor_node_id: Option<Box<str>>,
}

impl CommandError {
    pub fn new(code: CommandErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            latest_revision: None,
            latest_checklist_hash: None,
            descendant_count: None,
            remaining_count: None,
            ancestor_node_id: None,
        }
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self::new(CommandErrorCode::DatabaseError, message)
    }
}

impl From<String> for CommandError {
    fn from(message: String) -> Self {
        Self::database(message)
    }
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
pub struct TaskRelationDto {
    pub from_task_id: String,
    pub to_task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationFlowDto {
    pub groups: Vec<RecommendationGroupDto>,
    pub summary: String,
    #[serde(default)]
    pub relations: Vec<TaskRelationDto>,
    pub openui_response: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeChecklistInput {
    pub checklist_id: String,
    pub expected_revision: i64,
    pub instruction: Option<String>,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisSnapshotDto {
    pub id: String,
    pub checklist_id: String,
    pub checklist_revision: i64,
    pub checklist_hash: String,
    pub task_ids: Vec<String>,
    pub instruction_hash: String,
    pub request_hash: String,
    pub provider: String,
    pub requested_model: String,
    pub resolved_model: Option<String>,
    pub fallback_reason: Option<String>,
    pub result: RecommendationFlowDto,
    pub openui_response: String,
    pub generated_at: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiCredentialStatusDto {
    pub configured: bool,
    pub alias: Option<String>,
    pub credential_version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiConnectionResultDto {
    pub ok: bool,
    pub message: String,
}
