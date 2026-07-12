use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::Serialize;

use crate::{
    analysis::{AnalysisProvider, OpenAiResponsesProvider},
    credentials::{CredentialStore, KeyringCredentialStore, OPENAI_CREDENTIAL_ALIAS},
    models::{
        AnalysisSnapshotDto, ArchiveChecklistNodeInput, ChecklistDto, ChecklistNodeDto,
        ChecklistNodeKind, ChecklistTreeDto, CommandError, CommandErrorCode,
        CreateChecklistNodeInput, RecommendationFlowDto, RenameChecklistNodeInput,
        SetTaskCheckedInput, SetTaskEstimateInput, TaskDto, TaskStatus,
    },
};

pub const DEFAULT_CHECKLIST_ID: &str = "checklist-default-local";
const CURRENT_SCHEMA_VERSION: i64 = 4;
const SORT_KEY_STEP: i64 = 1024;
pub const MANUAL_SOURCE_ID: &str = "source-manual-local";
const MANUAL_SOURCE_NAME: &str = "Local Checklist";
const DEFAULT_CHECKLIST_TITLE: &str = "Today's Checklist";

pub struct AppState {
    db: Mutex<Db>,
    credential_store: Arc<dyn CredentialStore>,
    analysis_provider: Arc<dyn AnalysisProvider>,
}

impl AppState {
    pub fn new_default() -> Result<Self, String> {
        let data_dir = default_data_dir()?;
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("failed to create local data directory: {err}"))?;
        Self::new(data_dir.join("cole.sqlite"))
    }

    pub fn new(path: PathBuf) -> Result<Self, String> {
        Self::with_services(
            path,
            Arc::new(KeyringCredentialStore::new()),
            Arc::new(OpenAiResponsesProvider::new()?),
        )
    }

    pub fn with_services(
        path: PathBuf,
        credential_store: Arc<dyn CredentialStore>,
        analysis_provider: Arc<dyn AnalysisProvider>,
    ) -> Result<Self, String> {
        Ok(Self {
            db: Mutex::new(Db::open(path)?),
            credential_store,
            analysis_provider,
        })
    }

    pub fn credential_store(&self) -> Arc<dyn CredentialStore> {
        Arc::clone(&self.credential_store)
    }

    pub fn analysis_provider(&self) -> Arc<dyn AnalysisProvider> {
        Arc::clone(&self.analysis_provider)
    }

    pub fn with_db<T, E>(&self, f: impl FnOnce(&mut Db) -> Result<T, E>) -> Result<T, E>
    where
        E: From<String>,
    {
        let mut db = self
            .db
            .lock()
            .map_err(|_| E::from("local database lock was poisoned".to_string()))?;
        f(&mut db)
    }
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: PathBuf) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|err| format!("failed to open sqlite: {err}"))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|err| format!("failed to enable sqlite foreign keys: {err}"))?;
        let mut db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&mut self) -> Result<(), String> {
        let mut version = self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .map_err(|err| format!("failed to read sqlite schema version: {err}"))?;

        if version < 1 {
            let tx = self
                .conn
                .transaction()
                .map_err(|err| format!("failed to start base migration: {err}"))?;
            migrate_base_schema(&tx)?;
            tx.execute_batch("PRAGMA user_version = 1;")
                .map_err(|err| format!("failed to set base schema version: {err}"))?;
            tx.commit()
                .map_err(|err| format!("failed to commit base migration: {err}"))?;
            version = 1;
        }

        if version < 2 {
            let tx = self
                .conn
                .transaction()
                .map_err(|err| format!("failed to start checklist migration: {err}"))?;
            migrate_checklist_schema(&tx)?;
            tx.execute_batch("PRAGMA user_version = 2;")
                .map_err(|err| format!("failed to set checklist schema version: {err}"))?;
            tx.commit()
                .map_err(|err| format!("failed to commit checklist migration: {err}"))?;
            version = 2;
        }

        if version < 3 {
            let tx = self
                .conn
                .transaction()
                .map_err(|err| format!("failed to start analysis migration: {err}"))?;
            migrate_analysis_schema(&tx)?;
            tx.execute_batch("PRAGMA user_version = 3;")
                .map_err(|err| format!("failed to set analysis schema version: {err}"))?;
            tx.commit()
                .map_err(|err| format!("failed to commit analysis migration: {err}"))?;
            version = 3;
        }

        if version < 4 {
            let tx = self
                .conn
                .transaction()
                .map_err(|err| format!("failed to start estimate-constraint migration: {err}"))?;
            migrate_estimate_constraints(&tx)?;
            tx.execute_batch("PRAGMA user_version = 4;")
                .map_err(|err| format!("failed to set estimate schema version: {err}"))?;
            tx.commit()
                .map_err(|err| format!("failed to commit estimate constraints: {err}"))?;
            version = 4;
        }

        if version > CURRENT_SCHEMA_VERSION {
            return Err(format!(
                "sqlite schema version {version} is newer than supported version {CURRENT_SCHEMA_VERSION}"
            ));
        }
        Ok(())
    }

    pub fn schema_version(&self) -> Result<i64, String> {
        self.conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(|err| format!("failed to read sqlite schema version: {err}"))
    }

    pub fn get_default_checklist(&self) -> Result<ChecklistTreeDto, CommandError> {
        get_checklist_tree(&self.conn, DEFAULT_CHECKLIST_ID)
    }

    pub fn get_checklist_for_analysis(
        &self,
        checklist_id: &str,
        expected_revision: i64,
    ) -> Result<ChecklistTreeDto, CommandError> {
        ensure_revision(&self.conn, checklist_id, expected_revision)?;
        get_checklist_tree(&self.conn, checklist_id)
    }

    pub fn openai_credential_metadata(&self) -> Result<(String, i64), CommandError> {
        let alias = setting_value(&self.conn, "openai_credential_alias")?
            .unwrap_or_else(|| OPENAI_CREDENTIAL_ALIAS.to_string());
        let version = setting_value(&self.conn, "openai_credential_version")?
            .unwrap_or_else(|| "0".to_string())
            .parse::<i64>()
            .map_err(|_| CommandError::database("invalid OpenAI credential version"))?;
        Ok((alias, version))
    }

    pub fn bump_openai_credential_version(&mut self) -> Result<(String, i64), CommandError> {
        let tx = self
            .conn
            .transaction()
            .map_err(|err| db_error("failed to start credential metadata transaction", err))?;
        let alias = setting_value(&tx, "openai_credential_alias")?
            .unwrap_or_else(|| OPENAI_CREDENTIAL_ALIAS.to_string());
        let current = setting_value(&tx, "openai_credential_version")?
            .unwrap_or_else(|| "0".to_string())
            .parse::<i64>()
            .map_err(|_| CommandError::database("invalid OpenAI credential version"))?;
        let next = current
            .checked_add(1)
            .ok_or_else(|| CommandError::database("OpenAI credential version overflow"))?;
        let now = now_string();
        tx.execute(
            r#"
            INSERT INTO app_settings (key, value, updated_at)
            VALUES ('openai_credential_version', ?1, ?2)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
            "#,
            params![next.to_string(), now],
        )
        .map_err(|err| db_error("failed to update OpenAI credential version", err))?;
        tx.commit()
            .map_err(|err| db_error("failed to commit credential metadata", err))?;
        Ok((alias, next))
    }

    pub fn get_cached_analysis_snapshot(
        &self,
        cache_key: &str,
    ) -> Result<Option<AnalysisSnapshotDto>, CommandError> {
        let stored = query_analysis_snapshot(
            &self.conn,
            r#"
            SELECT id, checklist_id, checklist_revision, checklist_hash, task_ids_json,
                   instruction_hash, request_hash, provider, requested_model, resolved_model,
                   fallback_reason, result_json, openui_response, generated_at
            FROM analysis_snapshots
            WHERE cache_key = ?1
            ORDER BY rowid DESC
            LIMIT 1
            "#,
            cache_key,
        )?;
        match stored {
            Some(snapshot) => {
                let snapshot = hydrate_analysis_snapshot(&self.conn, snapshot)?;
                Ok((snapshot.state == "fresh").then_some(snapshot))
            }
            None => Ok(None),
        }
    }

    pub fn get_latest_analysis_snapshot(
        &self,
        checklist_id: &str,
    ) -> Result<Option<AnalysisSnapshotDto>, CommandError> {
        let snapshot_id = self
            .conn
            .query_row(
                "SELECT latest_analysis_snapshot_id FROM checklists WHERE id = ?1",
                params![checklist_id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()
            .map_err(|err| db_error("failed to read latest analysis pointer", err))?
            .ok_or_else(|| CommandError::new(CommandErrorCode::NotFound, "checklist not found"))?;
        match snapshot_id {
            Some(snapshot_id) => self.get_analysis_snapshot(&snapshot_id).map(Some),
            None => Ok(None),
        }
    }

    pub fn get_analysis_snapshot(
        &self,
        snapshot_id: &str,
    ) -> Result<AnalysisSnapshotDto, CommandError> {
        let stored = query_analysis_snapshot(
            &self.conn,
            r#"
            SELECT id, checklist_id, checklist_revision, checklist_hash, task_ids_json,
                   instruction_hash, request_hash, provider, requested_model, resolved_model,
                   fallback_reason, result_json, openui_response, generated_at
            FROM analysis_snapshots
            WHERE id = ?1
            LIMIT 1
            "#,
            snapshot_id,
        )?
        .ok_or_else(|| {
            CommandError::new(CommandErrorCode::NotFound, "analysis snapshot not found")
        })?;
        hydrate_analysis_snapshot(&self.conn, stored)
    }

    pub fn save_analysis_snapshot(
        &mut self,
        snapshot: &AnalysisSnapshotDto,
        cache_key: &str,
        credential_version: i64,
    ) -> Result<AnalysisSnapshotDto, CommandError> {
        let task_ids_json = serde_json::to_string(&snapshot.task_ids)
            .map_err(|_| CommandError::database("failed to serialize analysis task IDs"))?;
        let result_json = serde_json::to_string(&snapshot.result)
            .map_err(|_| CommandError::database("failed to serialize analysis result"))?;
        let tx = self
            .conn
            .transaction()
            .map_err(|err| db_error("failed to start analysis snapshot transaction", err))?;
        tx.execute(
            r#"
            INSERT INTO analysis_snapshots (
              id, checklist_id, checklist_revision, checklist_hash, task_ids_json,
              instruction_hash, request_hash, cache_key, provider, requested_model,
              resolved_model, credential_version, fallback_reason, result_json,
              openui_response, generated_at
            ) VALUES (
              ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16
            )
            "#,
            params![
                snapshot.id,
                snapshot.checklist_id,
                snapshot.checklist_revision,
                snapshot.checklist_hash,
                task_ids_json,
                snapshot.instruction_hash,
                snapshot.request_hash,
                cache_key,
                snapshot.provider,
                snapshot.requested_model,
                snapshot.resolved_model,
                credential_version,
                snapshot.fallback_reason,
                result_json,
                snapshot.openui_response,
                snapshot.generated_at,
            ],
        )
        .map_err(|err| db_error("failed to save immutable analysis snapshot", err))?;

        let current = get_checklist(&tx, &snapshot.checklist_id)?;
        let fresh = current.revision == snapshot.checklist_revision
            && current.checklist_hash == snapshot.checklist_hash;
        if fresh {
            tx.execute(
                "UPDATE checklists SET latest_analysis_snapshot_id = ?1 WHERE id = ?2",
                params![snapshot.id, snapshot.checklist_id],
            )
            .map_err(|err| db_error("failed to promote latest analysis snapshot", err))?;
        }
        tx.commit()
            .map_err(|err| db_error("failed to commit analysis snapshot", err))?;

        let mut saved = snapshot.clone();
        saved.state = if fresh { "fresh" } else { "stale" }.to_string();
        Ok(saved)
    }

    pub fn create_checklist_node(
        &mut self,
        input: CreateChecklistNodeInput,
    ) -> Result<ChecklistTreeDto, CommandError> {
        let title = validate_title(&input.title)?;
        validate_estimate(input.estimated_minutes)?;
        if input.kind == ChecklistNodeKind::Group && input.estimated_minutes.is_some() {
            return Err(CommandError::new(
                CommandErrorCode::InvalidNodeKind,
                "groups cannot have an estimate",
            ));
        }

        let tx = self
            .conn
            .transaction()
            .map_err(|err| db_error("failed to start create-node transaction", err))?;
        ensure_revision(&tx, &input.checklist_id, input.expected_revision)?;
        if let Some(parent_id) = input.parent_id.as_deref() {
            ensure_active_node(&tx, parent_id, Some(&input.checklist_id))?;
            if let Some(ancestor_id) = completed_ancestor(&tx, parent_id, true)? {
                return Err(completed_ancestor_error(ancestor_id));
            }
        }

        let id = new_node_id(&input.kind, &title);
        let sort_key = next_sort_key(&tx, &input.checklist_id, input.parent_id.as_deref())?;
        let now = now_string();
        tx.execute(
            r#"
            INSERT INTO checklist_nodes (
              id, checklist_id, parent_id, kind, title, status, sort_key,
              estimated_minutes, archived_at, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9, ?9)
            "#,
            params![
                id,
                input.checklist_id,
                input.parent_id,
                input.kind.as_str(),
                title,
                if input.kind == ChecklistNodeKind::Task {
                    Some("todo")
                } else {
                    None
                },
                sort_key,
                input.estimated_minutes,
                now,
            ],
        )
        .map_err(|err| db_error("failed to create checklist node", err))?;

        if input.kind == ChecklistNodeKind::Task {
            insert_task_projection(&tx, &id, &title, input.estimated_minutes, &now)?;
        }
        finish_semantic_mutation(&tx, &input.checklist_id, &now)?;
        tx.commit()
            .map_err(|err| db_error("failed to commit checklist node", err))?;
        get_checklist_tree(&self.conn, &input.checklist_id)
    }

    pub fn rename_checklist_node(
        &mut self,
        input: RenameChecklistNodeInput,
    ) -> Result<ChecklistTreeDto, CommandError> {
        let title = validate_title(&input.title)?;
        let tx = self
            .conn
            .transaction()
            .map_err(|err| db_error("failed to start rename transaction", err))?;
        let node = ensure_active_node(&tx, &input.node_id, None)?;
        ensure_revision(&tx, &node.checklist_id, input.expected_revision)?;
        if node.title == title {
            tx.commit()
                .map_err(|err| db_error("failed to close rename transaction", err))?;
            return get_checklist_tree(&self.conn, &node.checklist_id);
        }

        let now = now_string();
        tx.execute(
            "UPDATE checklist_nodes SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title, now, input.node_id],
        )
        .map_err(|err| db_error("failed to rename checklist node", err))?;
        if node.kind == ChecklistNodeKind::Task {
            tx.execute(
                r#"
                UPDATE tasks
                SET title = ?1, raw_text_hash = ?2, sync_state = 'local', updated_at = ?3
                WHERE id = ?4 AND source_type = 'manual'
                "#,
                params![title, sha256_hex(&title), now, input.node_id],
            )
            .map_err(|err| db_error("failed to rename task projection", err))?;
        }
        finish_semantic_mutation(&tx, &node.checklist_id, &now)?;
        tx.commit()
            .map_err(|err| db_error("failed to commit checklist rename", err))?;
        get_checklist_tree(&self.conn, &node.checklist_id)
    }

    pub fn set_task_checked(
        &mut self,
        input: SetTaskCheckedInput,
    ) -> Result<ChecklistTreeDto, CommandError> {
        let tx = self
            .conn
            .transaction()
            .map_err(|err| db_error("failed to start checked-state transaction", err))?;
        let node = ensure_active_node(&tx, &input.node_id, None)?;
        ensure_revision(&tx, &node.checklist_id, input.expected_revision)?;
        ensure_task_kind(&node)?;
        let is_done = node.status == Some(TaskStatus::Done);
        if is_done == input.checked {
            tx.commit()
                .map_err(|err| db_error("failed to close checked-state transaction", err))?;
            return get_checklist_tree(&self.conn, &node.checklist_id);
        }

        if input.checked {
            let remaining = incomplete_descendant_count(&tx, &input.node_id)?;
            if remaining > 0 {
                let mut error = CommandError::new(
                    CommandErrorCode::IncompleteDescendants,
                    "complete all descendant tasks before completing this task",
                );
                error.remaining_count = Some(remaining);
                return Err(error);
            }
        } else if let Some(ancestor_id) = completed_ancestor(&tx, &input.node_id, false)? {
            return Err(completed_ancestor_error(ancestor_id));
        }

        let now = now_string();
        let status = if input.checked { "done" } else { "todo" };
        tx.execute(
            "UPDATE checklist_nodes SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, now, input.node_id],
        )
        .map_err(|err| db_error("failed to update checklist task state", err))?;
        tx.execute(
            r#"
            UPDATE tasks
            SET status = ?1,
                completed_at = CASE WHEN ?1 = 'done' THEN ?2 ELSE NULL END,
                sync_state = 'local',
                updated_at = ?2
            WHERE id = ?3 AND source_type = 'manual'
            "#,
            params![status, now, input.node_id],
        )
        .map_err(|err| db_error("failed to update task projection state", err))?;
        finish_semantic_mutation(&tx, &node.checklist_id, &now)?;
        tx.commit()
            .map_err(|err| db_error("failed to commit checked state", err))?;
        get_checklist_tree(&self.conn, &node.checklist_id)
    }

    pub fn set_task_estimate(
        &mut self,
        input: SetTaskEstimateInput,
    ) -> Result<ChecklistTreeDto, CommandError> {
        validate_estimate(input.estimated_minutes)?;
        let tx = self
            .conn
            .transaction()
            .map_err(|err| db_error("failed to start estimate transaction", err))?;
        let node = ensure_active_node(&tx, &input.node_id, None)?;
        ensure_revision(&tx, &node.checklist_id, input.expected_revision)?;
        ensure_task_kind(&node)?;
        if node.estimated_minutes == input.estimated_minutes {
            tx.commit()
                .map_err(|err| db_error("failed to close estimate transaction", err))?;
            return get_checklist_tree(&self.conn, &node.checklist_id);
        }

        let now = now_string();
        tx.execute(
            "UPDATE checklist_nodes SET estimated_minutes = ?1, updated_at = ?2 WHERE id = ?3",
            params![input.estimated_minutes, now, input.node_id],
        )
        .map_err(|err| db_error("failed to update checklist estimate", err))?;
        tx.execute(
            r#"
            UPDATE tasks
            SET estimated_minutes = ?1, sync_state = 'local', updated_at = ?2
            WHERE id = ?3 AND source_type = 'manual'
            "#,
            params![input.estimated_minutes, now, input.node_id],
        )
        .map_err(|err| db_error("failed to update task projection estimate", err))?;
        finish_semantic_mutation(&tx, &node.checklist_id, &now)?;
        tx.commit()
            .map_err(|err| db_error("failed to commit task estimate", err))?;
        get_checklist_tree(&self.conn, &node.checklist_id)
    }

    pub fn archive_checklist_node(
        &mut self,
        input: ArchiveChecklistNodeInput,
    ) -> Result<ChecklistTreeDto, CommandError> {
        let tx = self
            .conn
            .transaction()
            .map_err(|err| db_error("failed to start archive transaction", err))?;
        let node = ensure_active_node(&tx, &input.node_id, None)?;
        ensure_revision(&tx, &node.checklist_id, input.expected_revision)?;
        let descendant_count = descendant_count(&tx, &input.node_id)?;
        if descendant_count > 0 && !input.cascade {
            let mut error = CommandError::new(
                CommandErrorCode::NonEmptyNode,
                "this node contains descendants; confirm cascade archive",
            );
            error.descendant_count = Some(descendant_count);
            return Err(error);
        }

        let now = now_string();
        tx.execute(
            r#"
            WITH RECURSIVE subtree(id) AS (
              SELECT id FROM checklist_nodes WHERE id = ?1 AND archived_at IS NULL
              UNION ALL
              SELECT child.id
              FROM checklist_nodes child
              JOIN subtree parent ON child.parent_id = parent.id
              WHERE child.archived_at IS NULL
            )
            UPDATE checklist_nodes
            SET status = CASE WHEN kind = 'task' THEN 'archived' ELSE NULL END,
                archived_at = ?2,
                updated_at = ?2
            WHERE id IN (SELECT id FROM subtree)
            "#,
            params![input.node_id, now],
        )
        .map_err(|err| db_error("failed to archive checklist subtree", err))?;
        tx.execute(
            r#"
            WITH RECURSIVE subtree(id) AS (
              SELECT id FROM checklist_nodes WHERE id = ?1
              UNION ALL
              SELECT child.id
              FROM checklist_nodes child
              JOIN subtree parent ON child.parent_id = parent.id
            )
            UPDATE tasks
            SET status = 'archived', sync_state = 'local', updated_at = ?2
            WHERE source_type = 'manual'
              AND id IN (SELECT id FROM subtree)
            "#,
            params![input.node_id, now],
        )
        .map_err(|err| db_error("failed to archive task projections", err))?;
        finish_semantic_mutation(&tx, &node.checklist_id, &now)?;
        tx.commit()
            .map_err(|err| db_error("failed to commit archive", err))?;
        get_checklist_tree(&self.conn, &node.checklist_id)
    }

    pub fn list_tasks(&self) -> Result<Vec<TaskDto>, String> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT id, source_id, source_type, external_id, title, body, status, due_at,
                       tags_json, source_location_json, raw_text_hash, sync_state,
                       source_path, line_start, estimated_minutes, created_at, updated_at, completed_at
                FROM tasks
                ORDER BY source_path ASC, line_start ASC, title ASC
                "#,
            )
            .map_err(|err| format!("failed to prepare task query: {err}"))?;
        let rows = stmt
            .query_map([], row_to_task)
            .map_err(|err| format!("failed to query tasks: {err}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|err| format!("failed to read task row: {err}"))
    }
}

fn migrate_base_schema(tx: &Transaction<'_>) -> Result<(), String> {
    tx.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS sources (
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL,
          source_type TEXT NOT NULL,
          vault_path TEXT,
          sync_enabled INTEGER NOT NULL DEFAULT 1,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tasks (
          id TEXT PRIMARY KEY,
          source_id TEXT NOT NULL,
          source_type TEXT NOT NULL,
          external_id TEXT NOT NULL,
          title TEXT NOT NULL,
          body TEXT,
          status TEXT NOT NULL,
          due_at TEXT,
          tags_json TEXT NOT NULL DEFAULT '[]',
          source_location_json TEXT NOT NULL DEFAULT '{}',
          raw_text_hash TEXT NOT NULL DEFAULT '',
          sync_state TEXT NOT NULL DEFAULT 'synced',
          source_path TEXT,
          line_start INTEGER,
          estimated_minutes INTEGER,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          completed_at TEXT,
          UNIQUE(source_id, external_id)
        );

        CREATE TABLE IF NOT EXISTS recommendation_cache (
          id TEXT PRIMARY KEY,
          response_json TEXT NOT NULL,
          created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sync_events (
          id TEXT PRIMARY KEY,
          source_id TEXT NOT NULL,
          event_type TEXT NOT NULL,
          status TEXT NOT NULL,
          message TEXT,
          started_at TEXT NOT NULL,
          finished_at TEXT
        );
        "#,
    )
    .map_err(|err| format!("failed to create base sqlite schema: {err}"))?;

    ensure_task_column(tx, "source_location_json", "TEXT NOT NULL DEFAULT '{}'")?;
    ensure_task_column(tx, "raw_text_hash", "TEXT NOT NULL DEFAULT ''")?;
    ensure_task_column(tx, "sync_state", "TEXT NOT NULL DEFAULT 'synced'")?;
    ensure_task_column(tx, "created_at", "TEXT NOT NULL DEFAULT '0'")?;
    Ok(())
}

fn ensure_task_column(tx: &Transaction<'_>, name: &str, definition: &str) -> Result<(), String> {
    let mut stmt = tx
        .prepare("PRAGMA table_info(tasks)")
        .map_err(|err| format!("failed to inspect task schema: {err}"))?;
    let existing = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|err| format!("failed to query task schema: {err}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("failed to read task schema: {err}"))?;
    if existing.iter().any(|column| column == name) {
        return Ok(());
    }
    tx.execute(
        &format!("ALTER TABLE tasks ADD COLUMN {name} {definition}"),
        [],
    )
    .map_err(|err| format!("failed to add task column {name}: {err}"))?;
    Ok(())
}

fn migrate_checklist_schema(tx: &Transaction<'_>) -> Result<(), String> {
    tx.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS checklists (
          id TEXT PRIMARY KEY,
          title TEXT NOT NULL,
          revision INTEGER NOT NULL DEFAULT 0,
          checklist_hash TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS checklist_nodes (
          id TEXT PRIMARY KEY,
          checklist_id TEXT NOT NULL REFERENCES checklists(id),
          parent_id TEXT REFERENCES checklist_nodes(id),
          kind TEXT NOT NULL CHECK (kind IN ('task', 'group')),
          title TEXT NOT NULL,
          status TEXT CHECK (status IN ('todo', 'done', 'archived')),
          sort_key INTEGER NOT NULL,
          estimated_minutes INTEGER,
          archived_at TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          CHECK (
            (kind = 'task' AND status IS NOT NULL)
            OR (kind = 'group' AND status IS NULL AND estimated_minutes IS NULL)
          )
        );

        CREATE INDEX IF NOT EXISTS checklist_nodes_parent_sort_idx
          ON checklist_nodes(checklist_id, parent_id, sort_key);
        "#,
    )
    .map_err(|err| format!("failed to create checklist schema: {err}"))?;

    let now = now_string();
    tx.execute(
        r#"
        INSERT INTO sources (id, name, source_type, vault_path, sync_enabled, created_at, updated_at)
        VALUES (?1, ?2, 'manual', NULL, 1, ?3, ?3)
        ON CONFLICT(id) DO UPDATE SET name = excluded.name
        "#,
        params![MANUAL_SOURCE_ID, MANUAL_SOURCE_NAME, now],
    )
    .map_err(|err| format!("failed to create manual source during migration: {err}"))?;
    tx.execute(
        r#"
        INSERT OR IGNORE INTO checklists
          (id, title, revision, checklist_hash, created_at, updated_at)
        VALUES (?1, ?2, 0, ?3, ?4, ?4)
        "#,
        params![
            DEFAULT_CHECKLIST_ID,
            DEFAULT_CHECKLIST_TITLE,
            sha256_hex("[]"),
            now
        ],
    )
    .map_err(|err| format!("failed to create default checklist: {err}"))?;

    let manual_tasks = {
        let mut stmt = tx
            .prepare(
                r#"
                SELECT id, title, status,
                       CASE
                         WHEN typeof(estimated_minutes) = 'integer'
                           AND estimated_minutes BETWEEN 1 AND 1440
                         THEN estimated_minutes
                         ELSE NULL
                       END,
                       created_at, updated_at
                FROM tasks
                WHERE source_type = 'manual'
                ORDER BY CAST(created_at AS INTEGER) ASC, id ASC
                "#,
            )
            .map_err(|err| format!("failed to prepare legacy manual task migration: {err}"))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })
            .map_err(|err| format!("failed to query legacy manual tasks: {err}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|err| format!("failed to read legacy manual task: {err}"))?
    };

    for (index, (id, title, legacy_status, estimate, created_at, updated_at)) in
        manual_tasks.into_iter().enumerate()
    {
        let status = if legacy_status == "blocked" {
            "todo"
        } else {
            legacy_status.as_str()
        };
        let archived_at = (status == "archived").then_some(updated_at.as_str());
        let source_location = manual_source_location(&id);
        tx.execute(
            r#"
            UPDATE tasks
            SET source_id = ?1,
                external_id = ?2,
                status = ?3,
                source_location_json = ?4,
                raw_text_hash = ?5,
                sync_state = 'local',
                completed_at = CASE WHEN ?3 = 'todo' THEN NULL ELSE completed_at END
            WHERE id = ?6 AND source_type = 'manual'
            "#,
            params![
                MANUAL_SOURCE_ID,
                format!("manual:{id}"),
                status,
                source_location,
                sha256_hex(&title),
                id,
            ],
        )
        .map_err(|err| format!("failed to normalize legacy manual task {id}: {err}"))?;
        tx.execute(
            r#"
            INSERT INTO checklist_nodes (
              id, checklist_id, parent_id, kind, title, status, sort_key,
              estimated_minutes, archived_at, created_at, updated_at
            )
            VALUES (?1, ?2, NULL, 'task', ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                id,
                DEFAULT_CHECKLIST_ID,
                title,
                status,
                (index as i64 + 1) * SORT_KEY_STEP,
                estimate,
                archived_at,
                created_at,
                updated_at,
            ],
        )
        .map_err(|err| format!("failed to project legacy manual task {id}: {err}"))?;
    }

    let checklist_hash =
        compute_checklist_hash(tx, DEFAULT_CHECKLIST_ID).map_err(|error| error.message)?;
    tx.execute(
        "UPDATE checklists SET checklist_hash = ?1 WHERE id = ?2",
        params![checklist_hash, DEFAULT_CHECKLIST_ID],
    )
    .map_err(|err| format!("failed to hash migrated checklist: {err}"))?;
    Ok(())
}

fn migrate_analysis_schema(tx: &Transaction<'_>) -> Result<(), String> {
    tx.execute_batch(
        r#"
        ALTER TABLE checklists ADD COLUMN latest_analysis_snapshot_id TEXT;

        CREATE TABLE analysis_snapshots (
          id TEXT PRIMARY KEY,
          checklist_id TEXT NOT NULL REFERENCES checklists(id),
          checklist_revision INTEGER NOT NULL,
          checklist_hash TEXT NOT NULL,
          task_ids_json TEXT NOT NULL,
          instruction_hash TEXT NOT NULL,
          request_hash TEXT NOT NULL,
          cache_key TEXT NOT NULL,
          provider TEXT NOT NULL CHECK (provider IN ('openai', 'deterministic')),
          requested_model TEXT NOT NULL,
          resolved_model TEXT,
          credential_version INTEGER NOT NULL,
          fallback_reason TEXT,
          result_json TEXT NOT NULL,
          openui_response TEXT NOT NULL,
          generated_at TEXT NOT NULL
        );

        CREATE INDEX analysis_snapshots_cache_idx
          ON analysis_snapshots(cache_key, generated_at);
        CREATE INDEX analysis_snapshots_checklist_idx
          ON analysis_snapshots(checklist_id, generated_at);

        CREATE TRIGGER analysis_snapshots_no_update
        BEFORE UPDATE ON analysis_snapshots
        BEGIN
          SELECT RAISE(ABORT, 'analysis snapshots are immutable');
        END;

        CREATE TRIGGER analysis_snapshots_no_delete
        BEFORE DELETE ON analysis_snapshots
        BEGIN
          SELECT RAISE(ABORT, 'analysis snapshots are immutable');
        END;

        CREATE TABLE app_settings (
          key TEXT PRIMARY KEY,
          value TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        "#,
    )
    .map_err(|err| format!("failed to create analysis schema: {err}"))?;

    let now = now_string();
    tx.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
        params!["openai_credential_alias", OPENAI_CREDENTIAL_ALIAS, now],
    )
    .map_err(|err| format!("failed to initialize OpenAI credential alias: {err}"))?;
    tx.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, '0', ?2)",
        params!["openai_credential_version", now],
    )
    .map_err(|err| format!("failed to initialize OpenAI credential version: {err}"))?;
    Ok(())
}

fn migrate_estimate_constraints(tx: &Transaction<'_>) -> Result<(), String> {
    tx.execute_batch(
        r#"
        UPDATE tasks
        SET estimated_minutes = NULL
        WHERE estimated_minutes IS NOT NULL
          AND (
            typeof(estimated_minutes) != 'integer'
            OR estimated_minutes NOT BETWEEN 1 AND 1440
          );

        UPDATE checklist_nodes
        SET estimated_minutes = NULL
        WHERE estimated_minutes IS NOT NULL
          AND (
            typeof(estimated_minutes) != 'integer'
            OR estimated_minutes NOT BETWEEN 1 AND 1440
          );

        CREATE TRIGGER checklist_nodes_estimate_insert_guard
        BEFORE INSERT ON checklist_nodes
        WHEN NEW.estimated_minutes IS NOT NULL
          AND (
            typeof(NEW.estimated_minutes) != 'integer'
            OR NEW.estimated_minutes NOT BETWEEN 1 AND 1440
          )
        BEGIN
          SELECT RAISE(ABORT, 'checklist node estimate must be between 1 and 1440');
        END;

        CREATE TRIGGER checklist_nodes_estimate_update_guard
        BEFORE UPDATE OF estimated_minutes ON checklist_nodes
        WHEN NEW.estimated_minutes IS NOT NULL
          AND (
            typeof(NEW.estimated_minutes) != 'integer'
            OR NEW.estimated_minutes NOT BETWEEN 1 AND 1440
          )
        BEGIN
          SELECT RAISE(ABORT, 'checklist node estimate must be between 1 and 1440');
        END;

        CREATE TRIGGER tasks_estimate_insert_guard
        BEFORE INSERT ON tasks
        WHEN NEW.estimated_minutes IS NOT NULL
          AND (
            typeof(NEW.estimated_minutes) != 'integer'
            OR NEW.estimated_minutes NOT BETWEEN 1 AND 1440
          )
        BEGIN
          SELECT RAISE(ABORT, 'task estimate must be between 1 and 1440');
        END;

        CREATE TRIGGER tasks_estimate_update_guard
        BEFORE UPDATE OF estimated_minutes ON tasks
        WHEN NEW.estimated_minutes IS NOT NULL
          AND (
            typeof(NEW.estimated_minutes) != 'integer'
            OR NEW.estimated_minutes NOT BETWEEN 1 AND 1440
          )
        BEGIN
          SELECT RAISE(ABORT, 'task estimate must be between 1 and 1440');
        END;
        "#,
    )
    .map_err(|err| format!("failed to normalize and constrain task estimates: {err}"))?;
    Ok(())
}

#[derive(Debug)]
struct StoredAnalysisSnapshot {
    id: String,
    checklist_id: String,
    checklist_revision: i64,
    checklist_hash: String,
    task_ids_json: String,
    instruction_hash: String,
    request_hash: String,
    provider: String,
    requested_model: String,
    resolved_model: Option<String>,
    fallback_reason: Option<String>,
    result_json: String,
    openui_response: String,
    generated_at: String,
}

fn query_analysis_snapshot(
    conn: &Connection,
    sql: &str,
    key: &str,
) -> Result<Option<StoredAnalysisSnapshot>, CommandError> {
    conn.query_row(sql, params![key], |row| {
        Ok(StoredAnalysisSnapshot {
            id: row.get(0)?,
            checklist_id: row.get(1)?,
            checklist_revision: row.get(2)?,
            checklist_hash: row.get(3)?,
            task_ids_json: row.get(4)?,
            instruction_hash: row.get(5)?,
            request_hash: row.get(6)?,
            provider: row.get(7)?,
            requested_model: row.get(8)?,
            resolved_model: row.get(9)?,
            fallback_reason: row.get(10)?,
            result_json: row.get(11)?,
            openui_response: row.get(12)?,
            generated_at: row.get(13)?,
        })
    })
    .optional()
    .map_err(|err| db_error("failed to load analysis snapshot", err))
}

fn hydrate_analysis_snapshot(
    conn: &Connection,
    stored: StoredAnalysisSnapshot,
) -> Result<AnalysisSnapshotDto, CommandError> {
    let task_ids = serde_json::from_str::<Vec<String>>(&stored.task_ids_json)
        .map_err(|_| CommandError::database("invalid task IDs in analysis snapshot"))?;
    let result = serde_json::from_str::<RecommendationFlowDto>(&stored.result_json)
        .map_err(|_| CommandError::database("invalid result in analysis snapshot"))?;
    let current = get_checklist(conn, &stored.checklist_id)?;
    let state = if current.revision == stored.checklist_revision
        && current.checklist_hash == stored.checklist_hash
    {
        "fresh"
    } else {
        "stale"
    };
    Ok(AnalysisSnapshotDto {
        id: stored.id,
        checklist_id: stored.checklist_id,
        checklist_revision: stored.checklist_revision,
        checklist_hash: stored.checklist_hash,
        task_ids,
        instruction_hash: stored.instruction_hash,
        request_hash: stored.request_hash,
        provider: stored.provider,
        requested_model: stored.requested_model,
        resolved_model: stored.resolved_model,
        fallback_reason: stored.fallback_reason,
        result,
        openui_response: stored.openui_response,
        generated_at: stored.generated_at,
        state: state.to_string(),
    })
}

fn setting_value(conn: &Connection, key: &str) -> Result<Option<String>, CommandError> {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
    .map_err(|err| db_error("failed to read app setting", err))
}

fn get_checklist_tree(
    conn: &Connection,
    checklist_id: &str,
) -> Result<ChecklistTreeDto, CommandError> {
    let checklist = get_checklist(conn, checklist_id)?;
    let mut stmt = conn
        .prepare(
            r#"
            WITH RECURSIVE tree(
              id, checklist_id, parent_id, kind, title, status, sort_key,
              estimated_minutes, archived_at, created_at, updated_at, path
            ) AS (
              SELECT id, checklist_id, parent_id, kind, title, status, sort_key,
                     estimated_minutes, archived_at, created_at, updated_at,
                     printf('%020d:%s', sort_key, id)
              FROM checklist_nodes
              WHERE checklist_id = ?1 AND parent_id IS NULL AND archived_at IS NULL
              UNION ALL
              SELECT child.id, child.checklist_id, child.parent_id, child.kind, child.title,
                     child.status, child.sort_key, child.estimated_minutes, child.archived_at,
                     child.created_at, child.updated_at,
                     parent.path || '/' || printf('%020d:%s', child.sort_key, child.id)
              FROM checklist_nodes child
              JOIN tree parent ON child.parent_id = parent.id
              WHERE child.archived_at IS NULL
            )
            SELECT id, checklist_id, parent_id, kind, title, status, sort_key,
                   estimated_minutes, archived_at, created_at, updated_at
            FROM tree
            ORDER BY path
            "#,
        )
        .map_err(|err| db_error("failed to prepare checklist tree query", err))?;
    let nodes = stmt
        .query_map(params![checklist_id], row_to_checklist_node)
        .map_err(|err| db_error("failed to query checklist tree", err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| db_error("failed to read checklist node", err))?;
    Ok(ChecklistTreeDto { checklist, nodes })
}

fn get_checklist(conn: &Connection, checklist_id: &str) -> Result<ChecklistDto, CommandError> {
    conn.query_row(
        r#"
        SELECT id, title, revision, checklist_hash, created_at, updated_at
        FROM checklists WHERE id = ?1
        "#,
        params![checklist_id],
        |row| {
            Ok(ChecklistDto {
                id: row.get(0)?,
                title: row.get(1)?,
                revision: row.get(2)?,
                checklist_hash: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        },
    )
    .optional()
    .map_err(|err| db_error("failed to load checklist", err))?
    .ok_or_else(|| CommandError::new(CommandErrorCode::NotFound, "checklist not found"))
}

fn row_to_checklist_node(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChecklistNodeDto> {
    let kind = ChecklistNodeKind::from_db(&row.get::<_, String>(3)?);
    let status = row
        .get::<_, Option<String>>(5)?
        .map(|value| TaskStatus::from_db(&value));
    Ok(ChecklistNodeDto {
        id: row.get(0)?,
        checklist_id: row.get(1)?,
        parent_id: row.get(2)?,
        kind,
        title: row.get(4)?,
        status,
        sort_key: row.get(6)?,
        estimated_minutes: row.get(7)?,
        archived_at: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn ensure_revision(
    conn: &Connection,
    checklist_id: &str,
    expected_revision: i64,
) -> Result<ChecklistDto, CommandError> {
    let checklist = get_checklist(conn, checklist_id)?;
    if checklist.revision != expected_revision {
        let mut error = CommandError::new(
            CommandErrorCode::StaleRevision,
            "the checklist changed before this update was applied",
        );
        error.latest_revision = Some(checklist.revision);
        error.latest_checklist_hash = Some(checklist.checklist_hash.clone().into_boxed_str());
        return Err(error);
    }
    Ok(checklist)
}

fn ensure_active_node(
    conn: &Connection,
    node_id: &str,
    checklist_id: Option<&str>,
) -> Result<ChecklistNodeDto, CommandError> {
    let node = conn
        .query_row(
            r#"
            SELECT id, checklist_id, parent_id, kind, title, status, sort_key,
                   estimated_minutes, archived_at, created_at, updated_at
            FROM checklist_nodes WHERE id = ?1
            "#,
            params![node_id],
            row_to_checklist_node,
        )
        .optional()
        .map_err(|err| db_error("failed to load checklist node", err))?
        .ok_or_else(|| CommandError::new(CommandErrorCode::NotFound, "checklist node not found"))?;
    if checklist_id.is_some_and(|expected| expected != node.checklist_id) {
        return Err(CommandError::new(
            CommandErrorCode::NotFound,
            "checklist node not found in this checklist",
        ));
    }
    if node.archived_at.is_some() || node.status == Some(TaskStatus::Archived) {
        return Err(CommandError::new(
            CommandErrorCode::ArchivedNode,
            "archived checklist nodes cannot be changed",
        ));
    }
    Ok(node)
}

fn ensure_task_kind(node: &ChecklistNodeDto) -> Result<(), CommandError> {
    if node.kind != ChecklistNodeKind::Task {
        return Err(CommandError::new(
            CommandErrorCode::InvalidNodeKind,
            "this operation is available only for task nodes",
        ));
    }
    Ok(())
}

fn next_sort_key(
    conn: &Connection,
    checklist_id: &str,
    parent_id: Option<&str>,
) -> Result<i64, CommandError> {
    conn.query_row(
        r#"
        SELECT COALESCE(MAX(sort_key), 0) + ?3
        FROM checklist_nodes
        WHERE checklist_id = ?1 AND parent_id IS ?2
        "#,
        params![checklist_id, parent_id, SORT_KEY_STEP],
        |row| row.get(0),
    )
    .map_err(|err| db_error("failed to allocate checklist sort key", err))
}

fn insert_task_projection(
    conn: &Connection,
    node_id: &str,
    title: &str,
    estimated_minutes: Option<i64>,
    now: &str,
) -> Result<(), CommandError> {
    conn.execute(
        r#"
        INSERT INTO tasks (
          id, source_id, source_type, external_id, title, body, status, due_at,
          tags_json, source_location_json, raw_text_hash, sync_state,
          source_path, line_start, estimated_minutes, created_at, updated_at, completed_at
        )
        VALUES (?1, ?2, 'manual', ?3, ?4, NULL, 'todo', NULL,
                '[]', ?5, ?6, 'local', NULL, NULL, ?7, ?8, ?8, NULL)
        "#,
        params![
            node_id,
            MANUAL_SOURCE_ID,
            format!("manual:{node_id}"),
            title,
            manual_source_location(node_id),
            sha256_hex(title),
            estimated_minutes,
            now,
        ],
    )
    .map_err(|err| db_error("failed to create task projection", err))?;
    Ok(())
}

fn completed_ancestor(
    conn: &Connection,
    node_id: &str,
    include_start: bool,
) -> Result<Option<String>, CommandError> {
    conn.query_row(
        r#"
        WITH RECURSIVE ancestors(id, parent_id, kind, status, archived_at, depth) AS (
          SELECT id, parent_id, kind, status, archived_at, 0
          FROM checklist_nodes WHERE id = ?1
          UNION ALL
          SELECT parent.id, parent.parent_id, parent.kind, parent.status, parent.archived_at,
                 child.depth + 1
          FROM checklist_nodes parent
          JOIN ancestors child ON parent.id = child.parent_id
        )
        SELECT id FROM ancestors
        WHERE kind = 'task' AND status = 'done' AND archived_at IS NULL
          AND (?2 = 1 OR depth > 0)
        ORDER BY depth ASC
        LIMIT 1
        "#,
        params![node_id, i64::from(include_start)],
        |row| row.get(0),
    )
    .optional()
    .map_err(|err| db_error("failed to inspect completed ancestors", err))
}

fn completed_ancestor_error(ancestor_id: String) -> CommandError {
    let mut error = CommandError::new(
        CommandErrorCode::CompletedAncestor,
        "a completed ancestor prevents this change",
    );
    error.ancestor_node_id = Some(ancestor_id.into_boxed_str());
    error
}

fn incomplete_descendant_count(conn: &Connection, node_id: &str) -> Result<i64, CommandError> {
    conn.query_row(
        r#"
        WITH RECURSIVE descendants(id, kind, status, archived_at) AS (
          SELECT id, kind, status, archived_at
          FROM checklist_nodes WHERE parent_id = ?1
          UNION ALL
          SELECT child.id, child.kind, child.status, child.archived_at
          FROM checklist_nodes child
          JOIN descendants parent ON child.parent_id = parent.id
        )
        SELECT COUNT(*) FROM descendants
        WHERE kind = 'task' AND status != 'done' AND archived_at IS NULL
        "#,
        params![node_id],
        |row| row.get(0),
    )
    .map_err(|err| db_error("failed to count incomplete descendants", err))
}

fn descendant_count(conn: &Connection, node_id: &str) -> Result<i64, CommandError> {
    conn.query_row(
        r#"
        WITH RECURSIVE descendants(id, archived_at) AS (
          SELECT id, archived_at FROM checklist_nodes WHERE parent_id = ?1
          UNION ALL
          SELECT child.id, child.archived_at
          FROM checklist_nodes child
          JOIN descendants parent ON child.parent_id = parent.id
        )
        SELECT COUNT(*) FROM descendants WHERE archived_at IS NULL
        "#,
        params![node_id],
        |row| row.get(0),
    )
    .map_err(|err| db_error("failed to count checklist descendants", err))
}

fn finish_semantic_mutation(
    conn: &Connection,
    checklist_id: &str,
    now: &str,
) -> Result<(), CommandError> {
    let checklist_hash = compute_checklist_hash(conn, checklist_id)?;
    let changed = conn
        .execute(
            r#"
            UPDATE checklists
            SET revision = revision + 1, checklist_hash = ?1, updated_at = ?2
            WHERE id = ?3
            "#,
            params![checklist_hash, now, checklist_id],
        )
        .map_err(|err| db_error("failed to advance checklist revision", err))?;
    if changed != 1 {
        return Err(CommandError::new(
            CommandErrorCode::NotFound,
            "checklist not found",
        ));
    }
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalChecklistNode {
    id: String,
    parent_id: Option<String>,
    sort_key: i64,
    kind: String,
    title: String,
    status: Option<String>,
    estimated_minutes: Option<i64>,
}

fn compute_checklist_hash(conn: &Connection, checklist_id: &str) -> Result<String, CommandError> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, parent_id, sort_key, kind, title, status, estimated_minutes
            FROM checklist_nodes
            WHERE checklist_id = ?1 AND archived_at IS NULL
            ORDER BY id ASC
            "#,
        )
        .map_err(|err| db_error("failed to prepare checklist hash query", err))?;
    let nodes = stmt
        .query_map(params![checklist_id], |row| {
            Ok(CanonicalChecklistNode {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                sort_key: row.get(2)?,
                kind: row.get(3)?,
                title: row.get(4)?,
                status: row.get(5)?,
                estimated_minutes: row.get(6)?,
            })
        })
        .map_err(|err| db_error("failed to query checklist hash nodes", err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| db_error("failed to read checklist hash node", err))?;
    let canonical = serde_json::to_vec(&nodes).map_err(|err| {
        CommandError::database(format!("failed to serialize checklist hash input: {err}"))
    })?;
    Ok(sha256_bytes(&canonical))
}

fn validate_title(value: &str) -> Result<String, CommandError> {
    let title = value.trim();
    if title.is_empty() {
        return Err(CommandError::new(
            CommandErrorCode::ValidationError,
            "title is required",
        ));
    }
    if title.chars().count() > 500 {
        return Err(CommandError::new(
            CommandErrorCode::ValidationError,
            "title must contain at most 500 characters",
        ));
    }
    Ok(title.to_string())
}

fn validate_estimate(value: Option<i64>) -> Result<(), CommandError> {
    if value.is_some_and(|minutes| !(1..=1440).contains(&minutes)) {
        return Err(CommandError::new(
            CommandErrorCode::ValidationError,
            "estimatedMinutes must be null or between 1 and 1440",
        ));
    }
    Ok(())
}

fn db_error(context: &str, error: impl std::fmt::Display) -> CommandError {
    CommandError::database(format!("{context}: {error}"))
}

fn new_node_id(kind: &ChecklistNodeKind, title: &str) -> String {
    format!(
        "node-{}-{}",
        kind.as_str(),
        stable_id(&format!("{}:{title}:{}", kind.as_str(), now_id_string()))
    )
}

fn manual_source_location(node_id: &str) -> String {
    serde_json::json!({
        "type": "manual",
        "checklistId": DEFAULT_CHECKLIST_ID,
        "nodeId": node_id,
    })
    .to_string()
}

fn sha256_hex(value: &str) -> String {
    sha256_bytes(value.as_bytes())
}

fn sha256_bytes(value: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(value))
}

fn row_to_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskDto> {
    let tags_json: String = row.get(8)?;
    let tags = serde_json::from_str(&tags_json).unwrap_or_default();
    let status: String = row.get(6)?;
    Ok(TaskDto {
        id: row.get(0)?,
        source_id: row.get(1)?,
        source_type: row.get(2)?,
        external_id: row.get(3)?,
        title: row.get(4)?,
        body: row.get(5)?,
        status: TaskStatus::from_db(&status),
        due_at: row.get(7)?,
        tags,
        source_location_json: row.get(9)?,
        raw_text_hash: row.get(10)?,
        sync_state: row.get(11)?,
        source_path: row.get(12)?,
        line_start: row.get(13)?,
        estimated_minutes: row.get(14)?,
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
        completed_at: row.get(17)?,
    })
}

fn stable_id(value: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(value.as_bytes());
    hex::encode(digest)[..16].to_string()
}

fn default_data_dir() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("COLE_DATA_DIR") {
        return Ok(PathBuf::from(path));
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")
            .ok_or_else(|| "HOME is not set; cannot resolve Cole data directory".to_string())?;
        Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Cole"))
    }

    #[cfg(target_os = "windows")]
    {
        let base = std::env::var_os("APPDATA")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .ok_or_else(|| {
                "APPDATA/USERPROFILE is not set; cannot resolve Cole data directory".to_string()
            })?;
        Ok(PathBuf::from(base).join("Cole"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        if let Some(path) = std::env::var_os("XDG_DATA_HOME") {
            return Ok(PathBuf::from(path).join("cole"));
        }
        let home = std::env::var_os("HOME")
            .ok_or_else(|| "HOME is not set; cannot resolve Cole data directory".to_string())?;
        Ok(PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("cole"))
    }
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn now_id_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
