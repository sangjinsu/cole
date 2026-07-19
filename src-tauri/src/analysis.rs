use std::{
    collections::{HashMap, HashSet},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    db::AppState,
    models::{
        AnalysisSnapshotDto, AnalyzeChecklistInput, ChecklistNodeDto, ChecklistNodeKind,
        ChecklistTreeDto, CommandError, CommandErrorCode, RecommendationFlowDto,
        RecommendationGroupDto, RecommendationTaskDto, TaskRelationDto, TaskStatus,
    },
};

pub const DEFAULT_OPENAI_MODEL: &str = "gpt-5.6";
const OPENAI_RESPONSES_URL: &str = "https://api.openai.com/v1/responses";
const MAX_INSTRUCTION_CHARS: usize = 2_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderTask {
    pub task_id: String,
    pub title: String,
    pub parent_id: Option<String>,
    pub estimated_minutes: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRequest {
    pub model: String,
    pub checklist_hash: String,
    pub instruction: Option<String>,
    pub tasks: Vec<ProviderTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RawAnalysisTask {
    pub task_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawAnalysisGroup {
    pub id: String,
    pub reason: String,
    pub tasks: Vec<RawAnalysisTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RawAnalysisRelation {
    pub from_task_id: String,
    pub to_task_id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawAnalysis {
    pub groups: Vec<RawAnalysisGroup>,
    pub relations: Vec<RawAnalysisRelation>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderAnalysis {
    pub resolved_model: String,
    pub analysis: RawAnalysis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderError {
    Timeout,
    RateLimited,
    Server,
    Refusal,
    Incomplete,
    Schema,
    Semantic,
    Network,
    Unauthorized,
    Provider,
}

impl ProviderError {
    fn fallback_reason(&self) -> &'static str {
        match self {
            Self::Timeout => "openai_timeout",
            Self::RateLimited => "openai_rate_limited",
            Self::Server => "openai_server_error",
            Self::Refusal => "openai_refusal",
            Self::Incomplete => "openai_incomplete",
            Self::Schema => "openai_schema_error",
            Self::Semantic => "openai_semantic_error",
            Self::Network => "openai_network_error",
            Self::Unauthorized => "openai_unauthorized",
            Self::Provider => "openai_provider_error",
        }
    }
}

#[async_trait]
pub trait AnalysisProvider: Send + Sync {
    async fn analyze(
        &self,
        api_key: &str,
        request: &ProviderRequest,
    ) -> Result<ProviderAnalysis, ProviderError>;

    async fn test_connection(&self, api_key: &str) -> Result<String, ProviderError>;
}

pub struct OpenAiResponsesProvider {
    client: reqwest::Client,
    endpoint: String,
}

impl OpenAiResponsesProvider {
    pub fn new() -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|_| "failed to initialize the OpenAI HTTP client".to_string())?;
        Ok(Self {
            client,
            endpoint: OPENAI_RESPONSES_URL.to_string(),
        })
    }

    async fn send(
        &self,
        api_key: &str,
        request: &ProviderRequest,
    ) -> Result<ProviderAnalysis, ProviderError> {
        let response = self
            .client
            .post(&self.endpoint)
            .bearer_auth(api_key)
            .json(&build_openai_request(request))
            .send()
            .await
            .map_err(|error| {
                if error.is_timeout() {
                    ProviderError::Timeout
                } else {
                    ProviderError::Network
                }
            })?;

        let status = response.status();
        if status.as_u16() == 429 {
            return Err(ProviderError::RateLimited);
        }
        if status.is_server_error() {
            return Err(ProviderError::Server);
        }
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(ProviderError::Unauthorized);
        }
        if !status.is_success() {
            return Err(ProviderError::Provider);
        }

        let body = response
            .json::<Value>()
            .await
            .map_err(|_| ProviderError::Schema)?;
        parse_openai_response(&body)
    }
}

#[async_trait]
impl AnalysisProvider for OpenAiResponsesProvider {
    async fn analyze(
        &self,
        api_key: &str,
        request: &ProviderRequest,
    ) -> Result<ProviderAnalysis, ProviderError> {
        self.send(api_key, request).await
    }

    async fn test_connection(&self, api_key: &str) -> Result<String, ProviderError> {
        let request = ProviderRequest {
            model: DEFAULT_OPENAI_MODEL.to_string(),
            checklist_hash: sha256_hex(b"connection-test"),
            instruction: Some("Return an empty valid Cole analysis.".to_string()),
            tasks: Vec::new(),
        };
        self.send(api_key, &request)
            .await
            .map(|response| response.resolved_model)
    }
}

pub async fn analyze_checklist_with_state(
    state: &AppState,
    input: AnalyzeChecklistInput,
) -> Result<AnalysisSnapshotDto, CommandError> {
    let instruction = normalize_instruction(input.instruction)?;
    let tree = state.with_db(|db| {
        db.get_checklist_for_analysis(&input.checklist_id, input.expected_revision)
    })?;
    let credential_version = state.with_db(|db| db.openai_credential_metadata())?.1;
    let instruction_hash = sha256_hex(instruction.as_deref().unwrap_or("").as_bytes());
    let request_hash = cache_key(
        &tree.checklist.checklist_hash,
        &instruction_hash,
        DEFAULT_OPENAI_MODEL,
        credential_version,
    );

    if !input.force {
        if let Some(cached) = state.with_db(|db| db.get_cached_analysis_snapshot(&request_hash))? {
            return Ok(cached);
        }
    }

    let actionable = actionable_tasks(&tree);
    let task_ids = actionable
        .iter()
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    let provider_request = ProviderRequest {
        model: DEFAULT_OPENAI_MODEL.to_string(),
        checklist_hash: tree.checklist.checklist_hash.clone(),
        instruction,
        tasks: actionable
            .iter()
            .map(|node| ProviderTask {
                task_id: node.id.clone(),
                title: node.title.clone(),
                parent_id: node.parent_id.clone(),
                estimated_minutes: node.estimated_minutes,
            })
            .collect(),
    };

    let credential_store = state.credential_store();
    let analysis_provider = state.analysis_provider();
    let (provider, resolved_model, fallback_reason, mut result) = match credential_store.get() {
        Ok(Some(api_key)) => match analysis_provider.analyze(&api_key, &provider_request).await {
            Ok(response) => match validate_provider_analysis(&tree, response.analysis) {
                Ok(flow) => (
                    "openai".to_string(),
                    nonempty_model(response.resolved_model),
                    None,
                    flow,
                ),
                Err(error) => (
                    "deterministic".to_string(),
                    None,
                    Some(error.fallback_reason().to_string()),
                    build_deterministic_result(&tree),
                ),
            },
            Err(error) => (
                "deterministic".to_string(),
                None,
                Some(error.fallback_reason().to_string()),
                build_deterministic_result(&tree),
            ),
        },
        Ok(None) => (
            "deterministic".to_string(),
            None,
            Some("missing_credential".to_string()),
            build_deterministic_result(&tree),
        ),
        Err(_) => (
            "deterministic".to_string(),
            None,
            Some("credential_store_error".to_string()),
            build_deterministic_result(&tree),
        ),
    };

    let openui_response = to_openui_lang(&result);
    result.openui_response = Some(openui_response.clone());
    let snapshot = AnalysisSnapshotDto {
        id: new_snapshot_id(&request_hash),
        checklist_id: tree.checklist.id,
        checklist_revision: tree.checklist.revision,
        checklist_hash: tree.checklist.checklist_hash,
        task_ids,
        instruction_hash,
        request_hash: request_hash.clone(),
        provider,
        requested_model: DEFAULT_OPENAI_MODEL.to_string(),
        resolved_model,
        fallback_reason,
        result,
        openui_response,
        generated_at: now_string(),
        state: "fresh".to_string(),
    };

    state.with_db(|db| db.save_analysis_snapshot(&snapshot, &request_hash, credential_version))
}

pub fn build_deterministic_result(tree: &ChecklistTreeDto) -> RecommendationFlowDto {
    let actionable = actionable_tasks(tree);
    let focus = actionable.first().copied();
    let next = actionable.get(1).copied();
    let finish = actionable
        .iter()
        .enumerate()
        .skip(2)
        .min_by_key(|(index, task)| (task.estimated_minutes.unwrap_or(i64::MAX), *index))
        .map(|(_, task)| *task);

    let mut result = RecommendationFlowDto {
        groups: vec![
            deterministic_group(
                "focus",
                "Focus",
                "Start with the first actionable task in checklist order.",
                focus,
            ),
            deterministic_group(
                "next",
                "Next",
                "Continue with the next actionable task.",
                next,
            ),
            deterministic_group(
                "finish",
                "Finish",
                "Use the shortest known remaining task to close the flow.",
                finish,
            ),
        ],
        summary: if actionable.is_empty() {
            "There are no actionable incomplete tasks in this checklist.".to_string()
        } else {
            "Cole arranged up to three actionable tasks without changing checklist order."
                .to_string()
        },
        relations: Vec::new(),
        openui_response: None,
    };
    result.openui_response = Some(to_openui_lang(&result));
    result
}

fn deterministic_group(
    id: &str,
    title: &str,
    reason: &str,
    task: Option<&ChecklistNodeDto>,
) -> RecommendationGroupDto {
    RecommendationGroupDto {
        id: id.to_string(),
        title: title.to_string(),
        reason: reason.to_string(),
        tasks: task.into_iter().map(recommendation_task).collect(),
    }
}

fn recommendation_task(node: &ChecklistNodeDto) -> RecommendationTaskDto {
    RecommendationTaskDto {
        task_id: node.id.clone(),
        title: node.title.clone(),
        source_type: "manual".to_string(),
        estimated_minutes: node.estimated_minutes,
    }
}

fn actionable_tasks(tree: &ChecklistTreeDto) -> Vec<&ChecklistNodeDto> {
    let active_todo_ids = tree
        .nodes
        .iter()
        .filter(|node| {
            node.kind == ChecklistNodeKind::Task && node.status == Some(TaskStatus::Todo)
        })
        .map(|node| node.id.as_str())
        .collect::<HashSet<_>>();
    let parent_by_id = tree
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node.parent_id.as_deref()))
        .collect::<HashMap<_, _>>();
    let mut todo_ancestors = HashSet::new();

    for task_id in &active_todo_ids {
        let mut parent_id = parent_by_id.get(task_id).copied().flatten();
        while let Some(parent) = parent_id {
            if active_todo_ids.contains(parent) {
                todo_ancestors.insert(parent);
            }
            parent_id = parent_by_id.get(parent).copied().flatten();
        }
    }

    tree.nodes
        .iter()
        .filter(|node| {
            active_todo_ids.contains(node.id.as_str()) && !todo_ancestors.contains(node.id.as_str())
        })
        .collect()
}

pub fn validate_provider_analysis(
    tree: &ChecklistTreeDto,
    raw: RawAnalysis,
) -> Result<RecommendationFlowDto, ProviderError> {
    if raw.groups.len() != 3 || !valid_text(&raw.summary, 1_000) {
        return Err(ProviderError::Semantic);
    }

    let actionable = actionable_tasks(tree);
    let by_id = actionable
        .iter()
        .map(|node| (node.id.as_str(), *node))
        .collect::<HashMap<_, _>>();
    let mut group_ids = HashSet::new();
    let mut task_ids = HashSet::new();
    let mut groups = Vec::with_capacity(3);

    for raw_group in raw.groups {
        if !matches!(raw_group.id.as_str(), "focus" | "next" | "finish")
            || !group_ids.insert(raw_group.id.clone())
            || !valid_text(&raw_group.reason, 500)
        {
            return Err(ProviderError::Semantic);
        }
        let mut tasks = Vec::with_capacity(1);
        for raw_task in raw_group.tasks {
            let Some(node) = by_id.get(raw_task.task_id.as_str()) else {
                continue;
            };
            if !task_ids.insert(raw_task.task_id) {
                continue;
            }
            tasks.push(recommendation_task(node));
            break;
        }
        let title = group_title(&raw_group.id).ok_or(ProviderError::Semantic)?;
        groups.push(RecommendationGroupDto {
            id: raw_group.id,
            title: title.to_string(),
            reason: raw_group.reason.trim().to_string(),
            tasks,
        });
    }
    if group_ids.len() != 3 || task_ids.len() > 3 {
        return Err(ProviderError::Semantic);
    }
    groups.sort_by_key(|group| group_order(&group.id));

    let relations = raw
        .relations
        .into_iter()
        .filter_map(|relation| {
            if !task_ids.contains(&relation.from_task_id)
                || !task_ids.contains(&relation.to_task_id)
                || relation.from_task_id == relation.to_task_id
                || !valid_optional_text(&relation.label, 200)
            {
                return None;
            }
            Some(TaskRelationDto {
                from_task_id: relation.from_task_id,
                to_task_id: relation.to_task_id,
                label: (!relation.label.trim().is_empty())
                    .then(|| relation.label.trim().to_string()),
            })
        })
        .collect::<Vec<_>>();

    let mut result = RecommendationFlowDto {
        groups,
        summary: raw.summary.trim().to_string(),
        relations,
        openui_response: None,
    };
    result.openui_response = Some(to_openui_lang(&result));
    Ok(result)
}

pub fn build_openai_request(request: &ProviderRequest) -> Value {
    let schema = json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["groups", "relations", "summary"],
        "properties": {
            "groups": {
                "type": "array",
                "minItems": 3,
                "maxItems": 3,
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["id", "reason", "tasks"],
                    "properties": {
                        "id": { "type": "string", "enum": ["focus", "next", "finish"] },
                        "reason": { "type": "string" },
                        "tasks": {
                            "type": "array",
                            "maxItems": 1,
                            "items": {
                                "type": "object",
                                "additionalProperties": false,
                                "required": ["taskId"],
                                "properties": { "taskId": { "type": "string" } }
                            }
                        }
                    }
                }
            },
            "relations": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["fromTaskId", "toTaskId", "label"],
                    "properties": {
                        "fromTaskId": { "type": "string" },
                        "toTaskId": { "type": "string" },
                        "label": { "type": "string" }
                    }
                }
            },
            "summary": { "type": "string" }
        }
    });
    let user_input = json!({
        "checklistHash": request.checklist_hash,
        "instruction": request.instruction,
        "tasks": request.tasks,
    });

    json!({
        "model": request.model,
        "reasoning": { "effort": "low" },
        "store": false,
        "tools": [],
        "input": [
            {
                "role": "system",
                "content": [{
                    "type": "input_text",
                    "text": "Arrange only the supplied task IDs into Focus, Next, and Finish. Return exactly the requested schema. Do not invent tasks or actions."
                }]
            },
            {
                "role": "user",
                "content": [{ "type": "input_text", "text": user_input.to_string() }]
            }
        ],
        "text": {
            "format": {
                "type": "json_schema",
                "name": "cole_checklist_analysis",
                "description": "A read-only recommendation flow for Cole.",
                "strict": true,
                "schema": schema
            }
        }
    })
}

pub fn parse_openai_response(body: &Value) -> Result<ProviderAnalysis, ProviderError> {
    match body.get("status").and_then(Value::as_str) {
        Some("completed") => {}
        Some("incomplete") => return Err(ProviderError::Incomplete),
        _ => return Err(ProviderError::Provider),
    }

    let mut output_text = String::new();
    let output = body
        .get("output")
        .and_then(Value::as_array)
        .ok_or(ProviderError::Schema)?;
    for item in output {
        let Some(content) = item.get("content").and_then(Value::as_array) else {
            continue;
        };
        for part in content {
            match part.get("type").and_then(Value::as_str) {
                Some("refusal") => return Err(ProviderError::Refusal),
                Some("output_text") => {
                    output_text.push_str(
                        part.get("text")
                            .and_then(Value::as_str)
                            .ok_or(ProviderError::Schema)?,
                    );
                }
                _ => {}
            }
        }
    }
    if output_text.is_empty() {
        return Err(ProviderError::Schema);
    }
    let analysis =
        serde_json::from_str::<RawAnalysis>(&output_text).map_err(|_| ProviderError::Schema)?;
    let resolved_model = body
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_OPENAI_MODEL)
        .to_string();
    Ok(ProviderAnalysis {
        resolved_model,
        analysis,
    })
}

pub fn to_openui_lang(result: &RecommendationFlowDto) -> String {
    let mut definitions = Vec::new();
    let mut children = Vec::new();
    let mut task_titles = HashMap::new();

    for (group_index, group) in result.groups.iter().enumerate() {
        for (task_index, task) in group.tasks.iter().enumerate() {
            let reference = format!("priority_{group_index}_{task_index}");
            task_titles.insert(task.task_id.as_str(), task.title.as_str());
            let estimate = task
                .estimated_minutes
                .map(|minutes| format!(", {minutes}"))
                .unwrap_or_default();
            definitions.push(format!(
                "{reference} = PriorityTask({}, {}, {}, {}{estimate})",
                literal(&task.task_id),
                literal(&task.title),
                literal(&group.title),
                literal(&group.reason),
            ));
            children.push(reference);
        }
        let reason_ref = format!("reason_{group_index}");
        definitions.push(format!(
            "{reason_ref} = RecommendationReason({}, {})",
            literal(&group.title),
            literal(&group.reason)
        ));
        children.push(reason_ref);
    }

    for (index, relation) in result.relations.iter().enumerate() {
        let reference = format!("relation_{index}");
        let from = task_titles
            .get(relation.from_task_id.as_str())
            .copied()
            .unwrap_or(relation.from_task_id.as_str());
        let to = task_titles
            .get(relation.to_task_id.as_str())
            .copied()
            .unwrap_or(relation.to_task_id.as_str());
        let label = relation
            .label
            .as_deref()
            .map(|value| format!(", {}", literal(value)))
            .unwrap_or_default();
        definitions.push(format!(
            "{reference} = TaskRelation({}, {}{label})",
            literal(from),
            literal(to)
        ));
        children.push(reference);
    }

    definitions.push(format!(
        "source = SourceReference({}, {})",
        literal("Local checklist"),
        literal("Read-only analysis snapshot")
    ));
    children.push("source".to_string());
    definitions.push(format!(
        "summary = AnalysisSummary({})",
        literal(&result.summary)
    ));
    children.push("summary".to_string());

    format!(
        "root = AnalysisCanvas({}, {}, [{}])\n{}",
        literal("Today's work flow"),
        literal(&result.summary),
        children.join(", "),
        definitions.join("\n")
    )
}

fn normalize_instruction(value: Option<String>) -> Result<Option<String>, CommandError> {
    let instruction = value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty());
    if instruction
        .as_ref()
        .is_some_and(|item| item.chars().count() > MAX_INSTRUCTION_CHARS)
    {
        return Err(CommandError::new(
            CommandErrorCode::ValidationError,
            "instruction must contain at most 2000 characters",
        ));
    }
    Ok(instruction)
}

fn cache_key(
    checklist_hash: &str,
    instruction_hash: &str,
    model: &str,
    credential_version: i64,
) -> String {
    let value = json!({
        "checklistHash": checklist_hash,
        "instructionHash": instruction_hash,
        "model": model,
        "credentialVersion": credential_version,
    });
    sha256_hex(value.to_string().as_bytes())
}

fn group_title(id: &str) -> Option<&'static str> {
    match id {
        "focus" => Some("Focus"),
        "next" => Some("Next"),
        "finish" => Some("Finish"),
        _ => None,
    }
}

fn group_order(id: &str) -> usize {
    match id {
        "focus" => 0,
        "next" => 1,
        "finish" => 2,
        _ => usize::MAX,
    }
}

fn valid_text(value: &str, max: usize) -> bool {
    let length = value.trim().chars().count();
    (1..=max).contains(&length)
}

fn valid_optional_text(value: &str, max: usize) -> bool {
    value.trim().chars().count() <= max
}

fn nonempty_model(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

fn literal(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

fn new_snapshot_id(request_hash: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!(
        "analysis-{}",
        &sha256_hex(format!("{request_hash}:{now}").as_bytes())[..20]
    )
}

fn now_string() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn sha256_hex(value: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(value))
}
