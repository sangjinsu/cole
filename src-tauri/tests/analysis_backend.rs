use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use async_trait::async_trait;
use cole_lib::{
    analysis::{
        analyze_checklist_with_state, build_deterministic_result, build_openai_request,
        parse_openai_response, AnalysisProvider, ProviderAnalysis, ProviderError, ProviderRequest,
        RawAnalysis, RawAnalysisGroup, RawAnalysisRelation, RawAnalysisTask,
    },
    credentials::{
        delete_openai_api_key, get_openai_credential_status, set_openai_api_key, CredentialStore,
    },
    db::{AppState, Db, DEFAULT_CHECKLIST_ID},
    models::{
        AnalyzeChecklistInput, ArchiveChecklistNodeInput, ChecklistNodeKind, CommandError,
        CommandErrorCode, CreateChecklistNodeInput, RenameChecklistNodeInput, SetTaskCheckedInput,
    },
};
use rusqlite::Connection;
use serde_json::json;

#[derive(Default)]
struct FakeCredentialStore {
    value: Mutex<Option<String>>,
}

impl FakeCredentialStore {
    fn with_key() -> Self {
        Self {
            value: Mutex::new(Some("test-key".to_string())),
        }
    }
}

impl CredentialStore for FakeCredentialStore {
    fn set(&self, secret: &str) -> Result<(), String> {
        *self.value.lock().unwrap() = Some(secret.to_string());
        Ok(())
    }

    fn get(&self) -> Result<Option<String>, String> {
        Ok(self.value.lock().unwrap().clone())
    }

    fn delete(&self) -> Result<(), String> {
        *self.value.lock().unwrap() = None;
        Ok(())
    }
}

struct FailingCredentialStore;

impl CredentialStore for FailingCredentialStore {
    fn set(&self, _secret: &str) -> Result<(), String> {
        Err("keyring unavailable".to_string())
    }

    fn get(&self) -> Result<Option<String>, String> {
        Err("keyring unavailable".to_string())
    }

    fn delete(&self) -> Result<(), String> {
        Err("keyring unavailable".to_string())
    }
}

struct OperationFailingCredentialStore {
    value: Mutex<Option<String>>,
}

impl OperationFailingCredentialStore {
    fn with_key() -> Self {
        Self {
            value: Mutex::new(Some("old-key".to_string())),
        }
    }
}

impl CredentialStore for OperationFailingCredentialStore {
    fn set(&self, _secret: &str) -> Result<(), String> {
        Err("set failed".to_string())
    }

    fn get(&self) -> Result<Option<String>, String> {
        Ok(self.value.lock().unwrap().clone())
    }

    fn delete(&self) -> Result<(), String> {
        Err("delete failed".to_string())
    }
}

struct FakeProvider {
    calls: AtomicUsize,
    result: Mutex<Result<ProviderAnalysis, ProviderError>>,
}

struct BlockingProvider {
    started: Arc<tokio::sync::Notify>,
    release: Arc<tokio::sync::Notify>,
}

#[async_trait]
impl AnalysisProvider for BlockingProvider {
    async fn analyze(
        &self,
        _api_key: &str,
        request: &ProviderRequest,
    ) -> Result<ProviderAnalysis, ProviderError> {
        self.started.notify_one();
        self.release.notified().await;
        let first_task = request.tasks.first().unwrap().task_id.clone();
        Ok(ProviderAnalysis {
            resolved_model: "gpt-5.6-2026-07-01".to_string(),
            analysis: RawAnalysis {
                groups: vec![
                    RawAnalysisGroup {
                        id: "focus".to_string(),
                        reason: "Start here.".to_string(),
                        tasks: vec![RawAnalysisTask {
                            task_id: first_task,
                        }],
                    },
                    RawAnalysisGroup {
                        id: "next".to_string(),
                        reason: "Nothing else yet.".to_string(),
                        tasks: vec![],
                    },
                    RawAnalysisGroup {
                        id: "finish".to_string(),
                        reason: "Finish when ready.".to_string(),
                        tasks: vec![],
                    },
                ],
                relations: vec![],
                summary: "One task is ready.".to_string(),
            },
        })
    }

    async fn test_connection(&self, _api_key: &str) -> Result<String, ProviderError> {
        Ok("gpt-5.6-2026-07-01".to_string())
    }
}

impl FakeProvider {
    fn valid() -> Self {
        Self {
            calls: AtomicUsize::new(0),
            result: Mutex::new(Ok(ProviderAnalysis {
                resolved_model: "gpt-5.6-2026-07-01".to_string(),
                analysis: RawAnalysis {
                    groups: vec![
                        RawAnalysisGroup {
                            id: "focus".to_string(),
                            reason: "Unblocks the remaining work.".to_string(),
                            tasks: vec![RawAnalysisTask {
                                task_id: "task-a".to_string(),
                            }],
                        },
                        RawAnalysisGroup {
                            id: "next".to_string(),
                            reason: "Continue with the next leaf task.".to_string(),
                            tasks: vec![RawAnalysisTask {
                                task_id: "task-b".to_string(),
                            }],
                        },
                        RawAnalysisGroup {
                            id: "finish".to_string(),
                            reason: "Close the shortest remaining task.".to_string(),
                            tasks: vec![RawAnalysisTask {
                                task_id: "task-c".to_string(),
                            }],
                        },
                    ],
                    relations: vec![RawAnalysisRelation {
                        from_task_id: "task-a".to_string(),
                        to_task_id: "task-b".to_string(),
                        label: "before".to_string(),
                    }],
                    summary: "Start with A, continue with B, and finish C.".to_string(),
                },
            })),
        }
    }

    fn failing(error: ProviderError) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            result: Mutex::new(Err(error)),
        }
    }
}

#[async_trait]
impl AnalysisProvider for FakeProvider {
    async fn analyze(
        &self,
        _api_key: &str,
        request: &ProviderRequest,
    ) -> Result<ProviderAnalysis, ProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let mut result = self.result.lock().unwrap().clone()?;
        for (index, group) in result.analysis.groups.iter_mut().enumerate() {
            if let Some(task) = request.tasks.get(index) {
                group.tasks = vec![RawAnalysisTask {
                    task_id: task.task_id.clone(),
                }];
            } else {
                group.tasks.clear();
            }
        }
        if request.tasks.len() >= 2 {
            result.analysis.relations[0]
                .from_task_id
                .clone_from(&request.tasks[0].task_id);
            result.analysis.relations[0]
                .to_task_id
                .clone_from(&request.tasks[1].task_id);
        } else {
            result.analysis.relations.clear();
        }
        Ok(result)
    }

    async fn test_connection(&self, _api_key: &str) -> Result<String, ProviderError> {
        Ok("gpt-5.6-2026-07-01".to_string())
    }
}

fn create_task(
    state: &AppState,
    id_title: &str,
    parent_id: Option<String>,
    estimate: Option<i64>,
    expected_revision: i64,
) {
    state
        .with_db(|db| {
            db.create_checklist_node(CreateChecklistNodeInput {
                checklist_id: DEFAULT_CHECKLIST_ID.to_string(),
                parent_id,
                kind: ChecklistNodeKind::Task,
                title: id_title.to_string(),
                estimated_minutes: estimate,
                expected_revision,
            })
        })
        .unwrap();
}

fn analysis_input(revision: i64, force: bool) -> AnalyzeChecklistInput {
    AnalyzeChecklistInput {
        checklist_id: DEFAULT_CHECKLIST_ID.to_string(),
        expected_revision: revision,
        instruction: Some("  Give me a calm plan.  ".to_string()),
        force,
    }
}

#[test]
fn current_migration_adds_immutable_analysis_and_setting_storage() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cole.sqlite");
    let db = Db::open(path.clone()).unwrap();
    assert_eq!(db.schema_version().unwrap(), 4);
    drop(db);

    let conn = Connection::open(path).unwrap();
    for table in ["analysis_snapshots", "app_settings"] {
        let count = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();
        assert_eq!(count, 1, "missing {table}");
    }
    let latest_column = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('checklists') WHERE name = 'latest_analysis_snapshot_id'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap();
    assert_eq!(latest_column, 1);
}

#[test]
fn migration_v3_rolls_back_schema_and_version_together() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cole.sqlite");
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE checklists (
          id TEXT PRIMARY KEY,
          title TEXT NOT NULL,
          revision INTEGER NOT NULL DEFAULT 0,
          checklist_hash TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE VIEW app_settings AS SELECT 1 AS incompatible;
        PRAGMA user_version = 2;
        "#,
    )
    .unwrap();
    drop(conn);

    assert!(Db::open(path.clone()).is_err());
    let conn = Connection::open(path).unwrap();
    assert_eq!(
        conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('checklists') WHERE name = 'latest_analysis_snapshot_id'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'analysis_snapshots'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
}

#[test]
fn deterministic_analysis_uses_actionable_todo_leaves_and_stable_finish_ties() {
    let dir = tempfile::tempdir().unwrap();
    let state = AppState::new(dir.path().join("cole.sqlite")).unwrap();

    create_task(&state, "Parent", None, None, 0);
    let parent = state
        .with_db(|db| db.get_default_checklist())
        .unwrap()
        .nodes[0]
        .id
        .clone();
    create_task(&state, "Child A", Some(parent), Some(30), 1);
    create_task(&state, "Task B", None, None, 2);
    create_task(&state, "Task C", None, Some(10), 3);
    create_task(&state, "Task D", None, Some(10), 4);

    let tree = state.with_db(|db| db.get_default_checklist()).unwrap();
    let result = build_deterministic_result(&tree);
    let ids = result
        .groups
        .iter()
        .flat_map(|group| group.tasks.iter().map(|task| task.title.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(ids, vec!["Child A", "Task B", "Task C"]);
    assert_eq!(result.groups[0].id, "focus");
    assert_eq!(result.groups[1].id, "next");
    assert_eq!(result.groups[2].id, "finish");
    assert!(!ids.contains(&"Parent"));
    assert!(result
        .openui_response
        .as_deref()
        .unwrap()
        .contains("AnalysisCanvas"));
    assert!(!result
        .openui_response
        .as_deref()
        .unwrap()
        .contains("TaskCard"));
}

#[test]
fn openai_request_uses_responses_strict_schema_without_tools_or_persisted_fields() {
    let request = ProviderRequest {
        model: "gpt-5.6".to_string(),
        checklist_hash: "hash".to_string(),
        instruction: Some("private instruction".to_string()),
        tasks: vec![],
    };
    let payload = build_openai_request(&request);

    assert_eq!(payload["model"], "gpt-5.6");
    assert_eq!(payload["reasoning"]["effort"], "low");
    assert_eq!(payload["store"], false);
    assert_eq!(payload["tools"], json!([]));
    assert_eq!(payload["text"]["format"]["type"], "json_schema");
    assert_eq!(payload["text"]["format"]["strict"], true);
    let schema = &payload["text"]["format"]["schema"];
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema["required"],
        json!(["groups", "relations", "summary"])
    );
    assert!(payload.get("api_key").is_none());
}

#[test]
fn response_parser_reads_nested_output_text_and_rejects_refusal_or_incomplete() {
    let body = json!({
        "status": "completed",
        "model": "gpt-5.6-2026-07-01",
        "output": [{
            "type": "message",
            "content": [{
                "type": "output_text",
                "text": serde_json::to_string(&RawAnalysis {
                    groups: vec![],
                    relations: vec![],
                    summary: "empty".to_string(),
                }).unwrap()
            }]
        }]
    });
    let parsed = parse_openai_response(&body).unwrap();
    assert_eq!(parsed.resolved_model, "gpt-5.6-2026-07-01");

    let refusal = json!({
        "status": "completed",
        "output": [{"type": "message", "content": [{"type": "refusal", "refusal": "no"}]}]
    });
    assert_eq!(parse_openai_response(&refusal), Err(ProviderError::Refusal));

    let incomplete = json!({"status": "incomplete", "output": []});
    assert_eq!(
        parse_openai_response(&incomplete),
        Err(ProviderError::Incomplete)
    );
}

#[tokio::test]
async fn analysis_caches_by_private_instruction_hash_and_force_bypasses_cache() {
    let dir = tempfile::tempdir().unwrap();
    let credential = Arc::new(FakeCredentialStore::with_key());
    let provider = Arc::new(FakeProvider::valid());
    let state =
        AppState::with_services(dir.path().join("cole.sqlite"), credential, provider.clone())
            .unwrap();

    create_task(&state, "task-a", None, Some(30), 0);
    create_task(&state, "task-b", None, Some(20), 1);
    create_task(&state, "task-c", None, Some(10), 2);

    let first = analyze_checklist_with_state(&state, analysis_input(3, false))
        .await
        .unwrap();
    let cached = analyze_checklist_with_state(&state, analysis_input(3, false))
        .await
        .unwrap();
    let forced = analyze_checklist_with_state(&state, analysis_input(3, true))
        .await
        .unwrap();

    assert_eq!(provider.calls.load(Ordering::SeqCst), 2);
    assert_eq!(first.id, cached.id);
    assert_ne!(first.id, forced.id);
    assert_eq!(first.provider, "openai");
    assert_eq!(first.state, "fresh");
    assert_eq!(first.requested_model, "gpt-5.6");
    assert_eq!(first.resolved_model.as_deref(), Some("gpt-5.6-2026-07-01"));
    assert!(!first.instruction_hash.is_empty());
    assert!(!first.request_hash.is_empty());

    let db_bytes = std::fs::read(dir.path().join("cole.sqlite")).unwrap();
    assert!(!String::from_utf8_lossy(&db_bytes).contains("Give me a calm plan"));
    let conn = Connection::open(dir.path().join("cole.sqlite")).unwrap();
    assert!(conn
        .execute(
            "UPDATE analysis_snapshots SET provider = 'deterministic' WHERE id = ?1",
            [&first.id],
        )
        .is_err());
}

#[tokio::test]
async fn provider_failures_and_missing_credentials_fall_back_deterministically() {
    for failure in [
        ProviderError::Timeout,
        ProviderError::RateLimited,
        ProviderError::Server,
        ProviderError::Refusal,
        ProviderError::Incomplete,
        ProviderError::Schema,
        ProviderError::Semantic,
    ] {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState::with_services(
            dir.path().join("cole.sqlite"),
            Arc::new(FakeCredentialStore::with_key()),
            Arc::new(FakeProvider::failing(failure.clone())),
        )
        .unwrap();
        create_task(&state, "Fallback", None, Some(5), 0);

        let snapshot = analyze_checklist_with_state(&state, analysis_input(1, false))
            .await
            .unwrap();
        assert_eq!(snapshot.provider, "deterministic");
        assert!(snapshot.fallback_reason.is_some());
        assert_eq!(snapshot.result.groups[0].tasks[0].title, "Fallback");
    }

    let dir = tempfile::tempdir().unwrap();
    let state = AppState::with_services(
        dir.path().join("cole.sqlite"),
        Arc::new(FakeCredentialStore::default()),
        Arc::new(FakeProvider::valid()),
    )
    .unwrap();
    create_task(&state, "No key", None, None, 0);
    let snapshot = analyze_checklist_with_state(&state, analysis_input(1, false))
        .await
        .unwrap();
    assert_eq!(snapshot.provider, "deterministic");
    assert_eq!(
        snapshot.fallback_reason.as_deref(),
        Some("missing_credential")
    );

    let dir = tempfile::tempdir().unwrap();
    let state = AppState::with_services(
        dir.path().join("cole.sqlite"),
        Arc::new(FailingCredentialStore),
        Arc::new(FakeProvider::valid()),
    )
    .unwrap();
    create_task(&state, "Keyring failure", None, None, 0);
    let snapshot = analyze_checklist_with_state(&state, analysis_input(1, false))
        .await
        .unwrap();
    assert_eq!(snapshot.provider, "deterministic");
    assert_eq!(
        snapshot.fallback_reason.as_deref(),
        Some("credential_store_error")
    );
}

#[tokio::test]
async fn force_retries_a_cached_transient_fallback() {
    let dir = tempfile::tempdir().unwrap();
    let provider = Arc::new(FakeProvider::failing(ProviderError::Timeout));
    let state = AppState::with_services(
        dir.path().join("cole.sqlite"),
        Arc::new(FakeCredentialStore::with_key()),
        provider.clone(),
    )
    .unwrap();
    create_task(&state, "Retry task", None, Some(5), 0);

    let fallback = analyze_checklist_with_state(&state, analysis_input(1, false))
        .await
        .unwrap();
    let cached = analyze_checklist_with_state(&state, analysis_input(1, false))
        .await
        .unwrap();
    assert_eq!(fallback.id, cached.id);
    assert_eq!(provider.calls.load(Ordering::SeqCst), 1);

    *provider.result.lock().unwrap() = FakeProvider::valid().result.into_inner().unwrap();
    let retried = analyze_checklist_with_state(&state, analysis_input(1, true))
        .await
        .unwrap();
    assert_ne!(fallback.id, retried.id);
    assert_eq!(provider.calls.load(Ordering::SeqCst), 2);
    assert_eq!(retried.provider, "openai");
}

#[tokio::test]
async fn late_provider_result_is_saved_stale_without_blocking_mutation_or_latest_promotion() {
    let dir = tempfile::tempdir().unwrap();
    let started = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let state = Arc::new(
        AppState::with_services(
            dir.path().join("cole.sqlite"),
            Arc::new(FakeCredentialStore::with_key()),
            Arc::new(BlockingProvider {
                started: Arc::clone(&started),
                release: Arc::clone(&release),
            }),
        )
        .unwrap(),
    );
    create_task(&state, "Original", None, None, 0);
    let node_id = state
        .with_db(|db| db.get_default_checklist())
        .unwrap()
        .nodes[0]
        .id
        .clone();

    let analysis_state = Arc::clone(&state);
    let running = tokio::spawn(async move {
        analyze_checklist_with_state(&analysis_state, analysis_input(1, false)).await
    });
    started.notified().await;

    let changed = state
        .with_db(|db| {
            db.rename_checklist_node(RenameChecklistNodeInput {
                node_id,
                title: "Changed while analyzing".to_string(),
                expected_revision: 1,
            })
        })
        .unwrap();
    assert_eq!(changed.checklist.revision, 2);

    release.notify_one();
    let snapshot = running.await.unwrap().unwrap();
    assert_eq!(snapshot.state, "stale");
    assert_eq!(snapshot.checklist_revision, 1);
    assert!(state
        .with_db(|db| db.get_latest_analysis_snapshot(DEFAULT_CHECKLIST_ID))
        .unwrap()
        .is_none());
    assert_eq!(
        state
            .with_db(|db| db.get_analysis_snapshot(&snapshot.id))
            .unwrap()
            .state,
        "stale"
    );
}

#[test]
fn credential_commands_never_return_or_persist_the_secret_and_bump_version() {
    let dir = tempfile::tempdir().unwrap();
    let state = AppState::with_services(
        dir.path().join("cole.sqlite"),
        Arc::new(FakeCredentialStore::default()),
        Arc::new(FakeProvider::valid()),
    )
    .unwrap();

    let initial = get_openai_credential_status(&state).unwrap();
    assert!(!initial.configured);
    assert_eq!(initial.credential_version, 0);

    let saved = set_openai_api_key(&state, "  secret-value  ").unwrap();
    assert!(saved.configured);
    assert_eq!(saved.alias.as_deref(), Some("secret://openai/default"));
    assert_eq!(saved.credential_version, 1);

    let bytes = std::fs::read(dir.path().join("cole.sqlite")).unwrap();
    assert!(!String::from_utf8_lossy(&bytes).contains("secret-value"));

    let deleted = delete_openai_api_key(&state).unwrap();
    assert!(!deleted.configured);
    assert_eq!(deleted.credential_version, 2);
}

#[tokio::test]
async fn failed_credential_operations_advance_cache_namespace_before_keyring_access() {
    let dir = tempfile::tempdir().unwrap();
    let provider = Arc::new(FakeProvider::valid());
    let state = AppState::with_services(
        dir.path().join("cole.sqlite"),
        Arc::new(OperationFailingCredentialStore::with_key()),
        provider.clone(),
    )
    .unwrap();
    create_task(&state, "Cache namespace", None, Some(10), 0);

    let original = analyze_checklist_with_state(&state, analysis_input(1, false))
        .await
        .unwrap();
    let set_error = set_openai_api_key(&state, "replacement-key").unwrap_err();
    assert_eq!(set_error.code, CommandErrorCode::CredentialError);
    assert_eq!(
        state
            .with_db(|db| db.openai_credential_metadata())
            .unwrap()
            .1,
        1
    );
    let after_set_failure = analyze_checklist_with_state(&state, analysis_input(1, false))
        .await
        .unwrap();
    assert_ne!(original.id, after_set_failure.id);
    assert_eq!(provider.calls.load(Ordering::SeqCst), 2);

    let delete_error = delete_openai_api_key(&state).unwrap_err();
    assert_eq!(delete_error.code, CommandErrorCode::CredentialError);
    assert_eq!(
        state
            .with_db(|db| db.openai_credential_metadata())
            .unwrap()
            .1,
        2
    );
    let after_delete_failure = analyze_checklist_with_state(&state, analysis_input(1, false))
        .await
        .unwrap();
    assert_ne!(after_set_failure.id, after_delete_failure.id);
    assert_eq!(provider.calls.load(Ordering::SeqCst), 3);
}

#[test]
fn provider_semantics_prune_invalid_tasks_duplicates_and_relations() {
    let dir = tempfile::tempdir().unwrap();
    let state = AppState::new(dir.path().join("cole.sqlite")).unwrap();
    create_task(&state, "Task A", None, None, 0);
    create_task(&state, "Task B", None, None, 1);
    create_task(&state, "Task C", None, None, 2);
    create_task(&state, "Done task", None, None, 3);
    create_task(&state, "Archived task", None, None, 4);
    let initial = state.with_db(|db| db.get_default_checklist()).unwrap();
    let id = |title: &str| {
        initial
            .nodes
            .iter()
            .find(|node| node.title == title)
            .unwrap()
            .id
            .clone()
    };
    let task_a = id("Task A");
    let task_b = id("Task B");
    let task_c = id("Task C");
    let done_task = id("Done task");
    let archived_task = id("Archived task");
    state
        .with_db(|db| {
            db.set_task_checked(SetTaskCheckedInput {
                node_id: done_task.clone(),
                checked: true,
                expected_revision: 5,
            })
        })
        .unwrap();
    state
        .with_db(|db| {
            db.archive_checklist_node(ArchiveChecklistNodeInput {
                node_id: archived_task.clone(),
                cascade: false,
                expected_revision: 6,
            })
        })
        .unwrap();
    let tree = state.with_db(|db| db.get_default_checklist()).unwrap();
    let raw = RawAnalysis {
        groups: vec![
            RawAnalysisGroup {
                id: "focus".to_string(),
                reason: "First".to_string(),
                tasks: vec![
                    RawAnalysisTask {
                        task_id: task_a.clone(),
                    },
                    RawAnalysisTask {
                        task_id: "unknown".to_string(),
                    },
                ],
            },
            RawAnalysisGroup {
                id: "next".to_string(),
                reason: "Next".to_string(),
                tasks: vec![
                    RawAnalysisTask {
                        task_id: task_a.clone(),
                    },
                    RawAnalysisTask {
                        task_id: task_b.clone(),
                    },
                ],
            },
            RawAnalysisGroup {
                id: "finish".to_string(),
                reason: "Finish".to_string(),
                tasks: vec![
                    RawAnalysisTask {
                        task_id: done_task.clone(),
                    },
                    RawAnalysisTask {
                        task_id: archived_task.clone(),
                    },
                    RawAnalysisTask {
                        task_id: task_c.clone(),
                    },
                    RawAnalysisTask {
                        task_id: task_b.clone(),
                    },
                ],
            },
        ],
        relations: vec![
            RawAnalysisRelation {
                from_task_id: task_a.clone(),
                to_task_id: task_b.clone(),
                label: "valid".to_string(),
            },
            RawAnalysisRelation {
                from_task_id: task_a.clone(),
                to_task_id: task_a.clone(),
                label: "self".to_string(),
            },
            RawAnalysisRelation {
                from_task_id: task_a.clone(),
                to_task_id: done_task,
                label: "done".to_string(),
            },
            RawAnalysisRelation {
                from_task_id: task_a,
                to_task_id: "unknown".to_string(),
                label: "unknown".to_string(),
            },
            RawAnalysisRelation {
                from_task_id: task_c,
                to_task_id: task_b,
                label: "also valid".to_string(),
            },
        ],
        summary: "Keep valid entries".to_string(),
    };

    let result = cole_lib::analysis::validate_provider_analysis(&tree, raw).unwrap();
    assert_eq!(
        result
            .groups
            .iter()
            .flat_map(|group| group.tasks.iter().map(|task| task.title.as_str()))
            .collect::<Vec<_>>(),
        vec!["Task A", "Task B", "Task C"]
    );
    assert_eq!(result.relations.len(), 2);
    assert!(result.relations.iter().all(|relation| {
        relation.label.as_deref() == Some("valid")
            || relation.label.as_deref() == Some("also valid")
    }));
}

#[test]
fn provider_semantics_still_require_exact_focus_next_finish_groups() {
    let dir = tempfile::tempdir().unwrap();
    let state = AppState::new(dir.path().join("cole.sqlite")).unwrap();
    let tree = state.with_db(|db| db.get_default_checklist()).unwrap();
    let invalid = RawAnalysis {
        groups: vec![RawAnalysisGroup {
            id: "focus".to_string(),
            reason: "Only one group".to_string(),
            tasks: vec![],
        }],
        relations: vec![],
        summary: "Invalid group set".to_string(),
    };
    assert_eq!(
        cole_lib::analysis::validate_provider_analysis(&tree, invalid),
        Err(ProviderError::Semantic)
    );
}

fn _assert_command_error_is_send(_: Result<(), CommandError>) {}
