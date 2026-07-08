use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{params, Connection};

use crate::models::{CreateObsidianSourceInput, SourceDto, TaskDto, TaskStatus};

pub struct AppState {
    db: Mutex<Db>,
}

impl AppState {
    pub fn new_default() -> Result<Self, String> {
        let data_dir = default_data_dir()?;
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("failed to create local data directory: {err}"))?;
        Self::new(data_dir.join("cole.sqlite"))
    }

    pub fn new(path: PathBuf) -> Result<Self, String> {
        Ok(Self {
            db: Mutex::new(Db::open(path)?),
        })
    }

    pub fn with_db<T>(&self, f: impl FnOnce(&mut Db) -> Result<T, String>) -> Result<T, String> {
        let mut db = self
            .db
            .lock()
            .map_err(|_| "local database lock was poisoned".to_string())?;
        f(&mut db)
    }
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: PathBuf) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|err| format!("failed to open sqlite: {err}"))?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
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
            .map_err(|err| format!("failed to migrate sqlite: {err}"))?;

        self.ensure_task_column("source_location_json", "TEXT NOT NULL DEFAULT '{}'")?;
        self.ensure_task_column("raw_text_hash", "TEXT NOT NULL DEFAULT ''")?;
        self.ensure_task_column("sync_state", "TEXT NOT NULL DEFAULT 'synced'")?;
        self.ensure_task_column("created_at", "TEXT NOT NULL DEFAULT '0'")?;
        Ok(())
    }

    fn ensure_task_column(&self, name: &str, definition: &str) -> Result<(), String> {
        let mut stmt = self
            .conn
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
        self.conn
            .execute(
                &format!("ALTER TABLE tasks ADD COLUMN {name} {definition}"),
                [],
            )
            .map_err(|err| format!("failed to add task column {name}: {err}"))?;
        Ok(())
    }

    pub fn create_obsidian_source(
        &self,
        input: CreateObsidianSourceInput,
    ) -> Result<SourceDto, String> {
        if input.name.trim().is_empty() {
            return Err("source name is required".to_string());
        }
        if input.vault_path.trim().is_empty() {
            return Err("vault path is required".to_string());
        }
        let id = format!("source-{}", stable_id(&input.vault_path));
        let now = now_string();
        self.conn
            .execute(
                r#"
                INSERT INTO sources (id, name, source_type, vault_path, sync_enabled, created_at, updated_at)
                VALUES (?1, ?2, 'obsidian', ?3, 1, ?4, ?4)
                ON CONFLICT(id) DO UPDATE SET
                  name = excluded.name,
                  vault_path = excluded.vault_path,
                  updated_at = excluded.updated_at
                "#,
                params![id, input.name.trim(), input.vault_path.trim(), now],
            )
            .map_err(|err| format!("failed to save source: {err}"))?;
        self.get_source(&id)
    }

    pub fn get_source(&self, id: &str) -> Result<SourceDto, String> {
        self.conn
            .query_row(
                "SELECT id, name, source_type, vault_path, sync_enabled FROM sources WHERE id = ?1",
                params![id],
                row_to_source,
            )
            .map_err(|err| format!("source not found: {err}"))
    }

    pub fn list_sources(&self) -> Result<Vec<SourceDto>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, source_type, vault_path, sync_enabled FROM sources ORDER BY name ASC",
            )
            .map_err(|err| format!("failed to prepare source query: {err}"))?;
        let rows = stmt
            .query_map([], row_to_source)
            .map_err(|err| format!("failed to query sources: {err}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|err| format!("failed to read source row: {err}"))
    }

    pub fn upsert_tasks(&mut self, tasks: &[TaskDto]) -> Result<usize, String> {
        let tx = self
            .conn
            .transaction()
            .map_err(|err| format!("failed to open task transaction: {err}"))?;
        let now = now_string();
        for task in tasks {
            let tags_json = serde_json::to_string(&task.tags)
                .map_err(|err| format!("failed to serialize tags: {err}"))?;
            tx.execute(
                r#"
                INSERT INTO tasks (
                  id, source_id, source_type, external_id, title, body, status, due_at,
                  tags_json, source_location_json, raw_text_hash, sync_state,
                  source_path, line_start, estimated_minutes, created_at, updated_at, completed_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
                ON CONFLICT(source_id, external_id) DO UPDATE SET
                  title = excluded.title,
                  body = excluded.body,
                  status = CASE
                    WHEN tasks.status = 'done' THEN tasks.status
                    ELSE excluded.status
                  END,
                  due_at = excluded.due_at,
                  tags_json = excluded.tags_json,
                  source_location_json = excluded.source_location_json,
                  raw_text_hash = excluded.raw_text_hash,
                  sync_state = excluded.sync_state,
                  source_path = excluded.source_path,
                  line_start = excluded.line_start,
                  estimated_minutes = excluded.estimated_minutes,
                  updated_at = excluded.updated_at
                "#,
                params![
                    task.id,
                    task.source_id,
                    task.source_type,
                    task.external_id,
                    task.title,
                    task.body,
                    task.status.as_str(),
                    task.due_at,
                    tags_json,
                    task.source_location_json,
                    task.raw_text_hash,
                    task.sync_state,
                    task.source_path,
                    task.line_start,
                    task.estimated_minutes,
                    task.created_at.as_deref().unwrap_or(&now),
                    now,
                    task.completed_at,
                ],
            )
            .map_err(|err| format!("failed to upsert task: {err}"))?;
        }
        tx.commit()
            .map_err(|err| format!("failed to commit task transaction: {err}"))?;
        Ok(tasks.len())
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

    pub fn mark_task_done_local(&self, task_id: &str) -> Result<TaskDto, String> {
        let now = now_string();
        let changed = self
            .conn
            .execute(
                "UPDATE tasks SET status = 'done', completed_at = ?1, updated_at = ?1 WHERE id = ?2",
                params![now, task_id],
            )
            .map_err(|err| format!("failed to mark task done: {err}"))?;
        if changed == 0 {
            return Err("task not found".to_string());
        }
        self.conn
            .query_row(
                r#"
                SELECT id, source_id, source_type, external_id, title, body, status, due_at,
                       tags_json, source_location_json, raw_text_hash, sync_state,
                       source_path, line_start, estimated_minutes, created_at, updated_at, completed_at
                FROM tasks WHERE id = ?1
                "#,
                params![task_id],
                row_to_task,
            )
            .map_err(|err| format!("failed to load updated task: {err}"))
    }
}

fn row_to_source(row: &rusqlite::Row<'_>) -> rusqlite::Result<SourceDto> {
    Ok(SourceDto {
        id: row.get(0)?,
        name: row.get(1)?,
        source_type: row.get(2)?,
        vault_path: row.get(3)?,
        sync_enabled: row.get::<_, i64>(4)? == 1,
    })
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
