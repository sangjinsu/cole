use cole_lib::{
    db::{AppState, Db},
    models::{
        ArchiveChecklistNodeInput, ChecklistNodeKind, CommandErrorCode, CreateChecklistNodeInput,
        RenameChecklistNodeInput, SetTaskCheckedInput, SetTaskEstimateInput, TaskStatus,
    },
};
use rusqlite::{params, Connection};

const DEFAULT_CHECKLIST_ID: &str = "checklist-default-local";

fn new_state() -> (tempfile::TempDir, AppState) {
    let dir = tempfile::tempdir().unwrap();
    let state = AppState::new(dir.path().join("cole.sqlite")).unwrap();
    (dir, state)
}

fn create_node(
    state: &AppState,
    parent_id: Option<String>,
    kind: ChecklistNodeKind,
    title: &str,
    estimated_minutes: Option<i64>,
    expected_revision: i64,
) -> cole_lib::models::ChecklistTreeDto {
    state
        .with_db(|db| {
            db.create_checklist_node(CreateChecklistNodeInput {
                checklist_id: DEFAULT_CHECKLIST_ID.to_string(),
                parent_id,
                kind,
                title: title.to_string(),
                estimated_minutes,
                expected_revision,
            })
        })
        .unwrap()
}

#[test]
fn versioned_migration_creates_default_checklist_and_projects_only_manual_tasks() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("cole.sqlite");
    let conn = Connection::open(&db_path).unwrap();
    create_legacy_schema(&conn);
    insert_legacy_task(&conn, "manual-todo", "manual", "todo", "Manual todo", "1");
    insert_legacy_task(
        &conn,
        "manual-blocked",
        "manual",
        "blocked",
        "Manual blocked",
        "2",
    );
    insert_legacy_task(
        &conn,
        "manual-archived",
        "manual",
        "archived",
        "Manual archived",
        "3",
    );
    insert_legacy_task(
        &conn,
        "manual-real-estimate",
        "manual",
        "todo",
        "Manual real estimate",
        "4",
    );
    insert_legacy_task(
        &conn,
        "obsidian-todo",
        "obsidian",
        "todo",
        "Obsidian todo",
        "5",
    );
    conn.execute(
        "UPDATE tasks SET estimated_minutes = 0 WHERE id = 'manual-todo'",
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE tasks SET estimated_minutes = -5 WHERE id = 'manual-blocked'",
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE tasks SET estimated_minutes = 1441 WHERE id = 'manual-archived'",
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE tasks SET estimated_minutes = 1.5 WHERE id = 'manual-real-estimate'",
        [],
    )
    .unwrap();
    drop(conn);

    let state = AppState::new(db_path.clone()).unwrap();
    let tree = state.with_db(|db| db.get_default_checklist()).unwrap();

    assert_eq!(tree.checklist.id, DEFAULT_CHECKLIST_ID);
    assert_eq!(tree.checklist.revision, 0);
    assert_eq!(
        tree.nodes.len(),
        3,
        "archived nodes stay hidden from active tree"
    );
    assert!(tree
        .nodes
        .iter()
        .all(|node| node.kind == ChecklistNodeKind::Task));
    assert!(tree
        .nodes
        .iter()
        .all(|node| node.estimated_minutes.is_none()));
    assert!(tree
        .nodes
        .iter()
        .any(|node| { node.id == "manual-blocked" && node.status == Some(TaskStatus::Todo) }));
    assert!(!tree.nodes.iter().any(|node| node.id == "obsidian-todo"));
    drop(state);

    let conn = Connection::open(db_path).unwrap();
    assert_eq!(
        conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        4
    );
    assert_eq!(
        conn.query_row(
            "SELECT status FROM tasks WHERE id = 'manual-blocked'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap(),
        "todo"
    );
    assert_eq!(
        conn.query_row(
            "SELECT status FROM tasks WHERE id = 'obsidian-todo'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap(),
        "todo"
    );
    assert_eq!(
        conn.query_row(
            r#"
            SELECT COUNT(*)
            FROM checklist_nodes node
            JOIN tasks task ON task.id = node.id
            WHERE node.id LIKE 'manual-%'
            "#,
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        4
    );
    assert_eq!(
        conn.query_row(
            "SELECT status FROM checklist_nodes WHERE id = 'manual-archived'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap(),
        "archived"
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE estimated_minutes IS NOT NULL",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM checklist_nodes WHERE estimated_minutes IS NOT NULL",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
}

#[test]
fn database_rejects_raw_invalid_estimates_for_nodes_and_task_projections() {
    let (dir, state) = new_state();
    let tree = create_node(
        &state,
        None,
        ChecklistNodeKind::Task,
        "Guarded estimate",
        Some(30),
        0,
    );
    let node_id = tree.nodes[0].id.clone();
    drop(state);

    let conn = Connection::open(dir.path().join("cole.sqlite")).unwrap();
    for invalid in [-1, 0, 1441] {
        assert!(conn
            .execute(
                "UPDATE checklist_nodes SET estimated_minutes = ?1 WHERE id = ?2",
                params![invalid, node_id],
            )
            .is_err());
        assert!(conn
            .execute(
                "UPDATE tasks SET estimated_minutes = ?1 WHERE id = ?2",
                params![invalid, node_id],
            )
            .is_err());
    }

    assert!(conn
        .execute(
            "UPDATE checklist_nodes SET estimated_minutes = ?1 WHERE id = ?2",
            params![1.5_f64, node_id],
        )
        .is_err());
    assert!(conn
        .execute(
            "UPDATE tasks SET estimated_minutes = ?1 WHERE id = ?2",
            params![1.5_f64, node_id],
        )
        .is_err());

    assert!(conn
        .execute(
            r#"
            INSERT INTO checklist_nodes (
              id, checklist_id, parent_id, kind, title, status, sort_key,
              estimated_minutes, archived_at, created_at, updated_at
            )
            SELECT 'raw-real-node', checklist_id, NULL, 'task', 'Raw real node', 'todo',
                   sort_key + 1, ?1, NULL, created_at, updated_at
            FROM checklist_nodes WHERE id = ?2
            "#,
            params![1.5_f64, node_id],
        )
        .is_err());
    assert!(conn
        .execute(
            r#"
            INSERT INTO tasks (
              id, source_id, source_type, external_id, title, body, status, due_at,
              tags_json, source_location_json, raw_text_hash, sync_state,
              source_path, line_start, estimated_minutes, created_at, updated_at, completed_at
            )
            SELECT 'raw-real-task', source_id, source_type, 'manual:raw-real-task',
                   'Raw real task', body, 'todo', due_at, tags_json, source_location_json,
                   raw_text_hash, sync_state, source_path, line_start, ?1,
                   created_at, updated_at, NULL
            FROM tasks WHERE id = ?2
            "#,
            params![1.5_f64, node_id],
        )
        .is_err());
}

#[test]
fn migration_rolls_back_schema_and_version_together() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("cole.sqlite");
    let conn = Connection::open(&db_path).unwrap();
    create_legacy_schema(&conn);
    conn.execute_batch(
        "PRAGMA user_version = 1; CREATE VIEW checklists AS SELECT 1 AS incompatible;",
    )
    .unwrap();
    drop(conn);

    assert!(Db::open(db_path.clone()).is_err());

    let conn = Connection::open(db_path).unwrap();
    assert_eq!(
        conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'checklist_nodes'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
}

#[test]
fn creates_arbitrarily_nested_nodes_with_append_only_sort_keys_and_task_projection() {
    let (_dir, state) = new_state();

    let first = create_node(
        &state,
        None,
        ChecklistNodeKind::Group,
        "  Project  ",
        None,
        0,
    );
    let group = first
        .nodes
        .iter()
        .find(|node| node.title == "Project")
        .unwrap()
        .clone();
    assert_eq!(first.checklist.revision, 1);

    let second = create_node(
        &state,
        Some(group.id.clone()),
        ChecklistNodeKind::Task,
        "First task",
        Some(30),
        1,
    );
    let first_task = second
        .nodes
        .iter()
        .find(|node| node.title == "First task")
        .unwrap()
        .clone();
    let third = create_node(
        &state,
        Some(group.id.clone()),
        ChecklistNodeKind::Task,
        "Second task",
        None,
        2,
    );
    let second_task = third
        .nodes
        .iter()
        .find(|node| node.title == "Second task")
        .unwrap();

    assert_eq!(third.checklist.revision, 3);
    assert_eq!(first_task.parent_id.as_deref(), Some(group.id.as_str()));
    assert!(second_task.sort_key > first_task.sort_key);
    assert_ne!(
        second.checklist.checklist_hash,
        third.checklist.checklist_hash
    );

    let projected = state
        .with_db(|db| db.list_tasks())
        .unwrap()
        .into_iter()
        .find(|task| task.id == first_task.id)
        .unwrap();
    assert_eq!(projected.source_id, "source-manual-local");
    assert_eq!(projected.source_type, "manual");
    assert_eq!(projected.external_id, format!("manual:{}", first_task.id));
    assert_eq!(projected.sync_state, "local");
    assert_eq!(projected.raw_text_hash.len(), 64);
    let location: serde_json::Value =
        serde_json::from_str(&projected.source_location_json).unwrap();
    assert_eq!(location["type"], "manual");
    assert_eq!(location["checklistId"], DEFAULT_CHECKLIST_ID);
    assert_eq!(location["nodeId"], first_task.id);
}

#[test]
fn supports_a_tree_deeper_than_four_levels() {
    let (_dir, state) = new_state();
    let mut parent_id = None;
    let mut revision = 0;
    let mut expected_chain = Vec::new();

    for level in 1..=4 {
        let tree = create_node(
            &state,
            parent_id.clone(),
            ChecklistNodeKind::Group,
            &format!("Level {level}"),
            None,
            revision,
        );
        let node = tree
            .nodes
            .iter()
            .find(|node| node.title == format!("Level {level}"))
            .unwrap()
            .clone();
        expected_chain.push(node.id.clone());
        parent_id = Some(node.id);
        revision = tree.checklist.revision;
    }

    let tree = create_node(
        &state,
        parent_id,
        ChecklistNodeKind::Task,
        "Depth five task",
        Some(45),
        revision,
    );
    let task = tree
        .nodes
        .iter()
        .find(|node| node.title == "Depth five task")
        .unwrap();

    assert_eq!(tree.checklist.revision, 5);
    assert_eq!(tree.nodes.len(), 5);
    assert_eq!(
        task.parent_id.as_deref(),
        expected_chain.last().map(String::as_str)
    );
    for (index, node_id) in expected_chain.iter().enumerate().skip(1) {
        assert_eq!(
            tree.nodes
                .iter()
                .find(|node| &node.id == node_id)
                .unwrap()
                .parent_id
                .as_deref(),
            Some(expected_chain[index - 1].as_str())
        );
    }
}

#[test]
fn rejects_invalid_values_and_reports_latest_revision_on_conflict() {
    let (_dir, state) = new_state();

    let title_error = state
        .with_db(|db| {
            db.create_checklist_node(CreateChecklistNodeInput {
                checklist_id: DEFAULT_CHECKLIST_ID.to_string(),
                parent_id: None,
                kind: ChecklistNodeKind::Task,
                title: "x".repeat(501),
                estimated_minutes: None,
                expected_revision: 0,
            })
        })
        .unwrap_err();
    assert_eq!(title_error.code, CommandErrorCode::ValidationError);

    let estimate_error = state
        .with_db(|db| {
            db.create_checklist_node(CreateChecklistNodeInput {
                checklist_id: DEFAULT_CHECKLIST_ID.to_string(),
                parent_id: None,
                kind: ChecklistNodeKind::Task,
                title: "Task".to_string(),
                estimated_minutes: Some(0),
                expected_revision: 0,
            })
        })
        .unwrap_err();
    assert_eq!(estimate_error.code, CommandErrorCode::ValidationError);

    let current = create_node(&state, None, ChecklistNodeKind::Task, "Task", None, 0);
    let stale = state
        .with_db(|db| {
            db.rename_checklist_node(RenameChecklistNodeInput {
                node_id: current.nodes[0].id.clone(),
                title: "Renamed".to_string(),
                expected_revision: 0,
            })
        })
        .unwrap_err();
    assert_eq!(stale.code, CommandErrorCode::StaleRevision);
    assert_eq!(stale.latest_revision, Some(1));
    assert_eq!(
        stale.latest_checklist_hash.as_deref(),
        Some(current.checklist.checklist_hash.as_str())
    );
}

#[test]
fn enforces_task_kind_and_completed_ancestor_invariants() {
    let (_dir, state) = new_state();
    let parent_tree = create_node(&state, None, ChecklistNodeKind::Task, "Parent", None, 0);
    let parent = parent_tree.nodes[0].clone();
    let child_tree = create_node(
        &state,
        Some(parent.id.clone()),
        ChecklistNodeKind::Task,
        "Child",
        None,
        1,
    );
    let child = child_tree
        .nodes
        .iter()
        .find(|node| node.title == "Child")
        .unwrap()
        .clone();

    let blocked = state
        .with_db(|db| {
            db.set_task_checked(SetTaskCheckedInput {
                node_id: parent.id.clone(),
                checked: true,
                expected_revision: 2,
            })
        })
        .unwrap_err();
    assert_eq!(blocked.code, CommandErrorCode::IncompleteDescendants);
    assert_eq!(blocked.remaining_count, Some(1));

    let child_done = state
        .with_db(|db| {
            db.set_task_checked(SetTaskCheckedInput {
                node_id: child.id.clone(),
                checked: true,
                expected_revision: 2,
            })
        })
        .unwrap();
    let parent_done = state
        .with_db(|db| {
            db.set_task_checked(SetTaskCheckedInput {
                node_id: parent.id.clone(),
                checked: true,
                expected_revision: child_done.checklist.revision,
            })
        })
        .unwrap();

    let create_below_done = state
        .with_db(|db| {
            db.create_checklist_node(CreateChecklistNodeInput {
                checklist_id: DEFAULT_CHECKLIST_ID.to_string(),
                parent_id: Some(parent.id.clone()),
                kind: ChecklistNodeKind::Task,
                title: "Too late".to_string(),
                estimated_minutes: None,
                expected_revision: parent_done.checklist.revision,
            })
        })
        .unwrap_err();
    assert_eq!(create_below_done.code, CommandErrorCode::CompletedAncestor);
    assert_eq!(
        create_below_done.ancestor_node_id.as_deref(),
        Some(parent.id.as_str())
    );

    let uncheck_below_done = state
        .with_db(|db| {
            db.set_task_checked(SetTaskCheckedInput {
                node_id: child.id.clone(),
                checked: false,
                expected_revision: parent_done.checklist.revision,
            })
        })
        .unwrap_err();
    assert_eq!(uncheck_below_done.code, CommandErrorCode::CompletedAncestor);
    assert_eq!(
        uncheck_below_done.ancestor_node_id.as_deref(),
        Some(parent.id.as_str())
    );

    let kind_error = state
        .with_db(|db| {
            db.set_task_estimate(SetTaskEstimateInput {
                node_id: parent.id.replace("task", "missing"),
                estimated_minutes: Some(20),
                expected_revision: parent_done.checklist.revision,
            })
        })
        .unwrap_err();
    assert_eq!(kind_error.code, CommandErrorCode::NotFound);
}

#[test]
fn rejects_check_and_estimate_mutations_for_group_nodes() {
    let (_dir, state) = new_state();
    let tree = create_node(&state, None, ChecklistNodeKind::Group, "Group", None, 0);
    let group = tree.nodes[0].clone();

    let check_error = state
        .with_db(|db| {
            db.set_task_checked(SetTaskCheckedInput {
                node_id: group.id.clone(),
                checked: true,
                expected_revision: 1,
            })
        })
        .unwrap_err();
    assert_eq!(check_error.code, CommandErrorCode::InvalidNodeKind);

    let estimate_error = state
        .with_db(|db| {
            db.set_task_estimate(SetTaskEstimateInput {
                node_id: group.id,
                estimated_minutes: Some(20),
                expected_revision: 1,
            })
        })
        .unwrap_err();
    assert_eq!(estimate_error.code, CommandErrorCode::InvalidNodeKind);
    assert_eq!(
        state
            .with_db(|db| db.get_default_checklist())
            .unwrap()
            .checklist
            .revision,
        1
    );
}

#[test]
fn archives_non_empty_subtrees_only_after_cascade_confirmation() {
    let (_dir, state) = new_state();
    let group_tree = create_node(&state, None, ChecklistNodeKind::Group, "Group", None, 0);
    let group = group_tree.nodes[0].clone();
    let task_tree = create_node(
        &state,
        Some(group.id.clone()),
        ChecklistNodeKind::Task,
        "Task",
        Some(10),
        1,
    );
    let task = task_tree
        .nodes
        .iter()
        .find(|node| node.kind == ChecklistNodeKind::Task)
        .unwrap()
        .clone();

    let needs_confirmation = state
        .with_db(|db| {
            db.archive_checklist_node(ArchiveChecklistNodeInput {
                node_id: group.id.clone(),
                cascade: false,
                expected_revision: 2,
            })
        })
        .unwrap_err();
    assert_eq!(needs_confirmation.code, CommandErrorCode::NonEmptyNode);
    assert_eq!(needs_confirmation.descendant_count, Some(1));
    assert_eq!(
        state
            .with_db(|db| db.get_default_checklist())
            .unwrap()
            .checklist
            .revision,
        2
    );

    let archived = state
        .with_db(|db| {
            db.archive_checklist_node(ArchiveChecklistNodeInput {
                node_id: group.id,
                cascade: true,
                expected_revision: 2,
            })
        })
        .unwrap();
    assert_eq!(archived.checklist.revision, 3);
    assert!(archived.nodes.is_empty());
    let projected = state
        .with_db(|db| db.list_tasks())
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.id == task.id)
        .unwrap();
    assert_eq!(projected.status, TaskStatus::Archived);
}

#[test]
fn rename_and_estimate_mutations_update_projection_and_skip_noop_revisions() {
    let (_dir, state) = new_state();
    let created = create_node(&state, None, ChecklistNodeKind::Task, "Task", Some(15), 0);
    let node = created.nodes[0].clone();

    let no_op = state
        .with_db(|db| {
            db.rename_checklist_node(RenameChecklistNodeInput {
                node_id: node.id.clone(),
                title: " Task ".to_string(),
                expected_revision: 1,
            })
        })
        .unwrap();
    assert_eq!(no_op.checklist.revision, 1);

    let renamed = state
        .with_db(|db| {
            db.rename_checklist_node(RenameChecklistNodeInput {
                node_id: node.id.clone(),
                title: "Renamed".to_string(),
                expected_revision: 1,
            })
        })
        .unwrap();
    let estimated = state
        .with_db(|db| {
            db.set_task_estimate(SetTaskEstimateInput {
                node_id: node.id.clone(),
                estimated_minutes: None,
                expected_revision: renamed.checklist.revision,
            })
        })
        .unwrap();
    assert_eq!(estimated.checklist.revision, 3);
    let projected = state
        .with_db(|db| db.list_tasks())
        .unwrap()
        .into_iter()
        .find(|task| task.id == node.id)
        .unwrap();
    assert_eq!(projected.title, "Renamed");
    assert_eq!(projected.estimated_minutes, None);
    assert_eq!(projected.sync_state, "local");
}

fn create_legacy_schema(conn: &Connection) {
    conn.execute_batch(
        r#"
        CREATE TABLE sources (
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL,
          source_type TEXT NOT NULL,
          vault_path TEXT,
          sync_enabled INTEGER NOT NULL DEFAULT 1,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE tasks (
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
        CREATE TABLE recommendation_cache (
          id TEXT PRIMARY KEY,
          response_json TEXT NOT NULL,
          created_at TEXT NOT NULL
        );
        CREATE TABLE sync_events (
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
    .unwrap();
}

fn insert_legacy_task(
    conn: &Connection,
    id: &str,
    source_type: &str,
    status: &str,
    title: &str,
    created_at: &str,
) {
    let source_id = if source_type == "manual" {
        "source-manual-local"
    } else {
        "source-obsidian"
    };
    conn.execute(
        r#"
        INSERT OR IGNORE INTO sources
          (id, name, source_type, vault_path, sync_enabled, created_at, updated_at)
        VALUES (?1, ?2, ?3, NULL, 1, '0', '0')
        "#,
        params![source_id, source_type, source_type],
    )
    .unwrap();
    conn.execute(
        r#"
        INSERT INTO tasks (
          id, source_id, source_type, external_id, title, body, status, due_at,
          tags_json, source_location_json, raw_text_hash, sync_state,
          source_path, line_start, estimated_minutes, created_at, updated_at, completed_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, NULL,
                '[]', '{}', 'legacy', 'synced', NULL, NULL, NULL, ?7, ?7, NULL)
        "#,
        params![
            id,
            source_id,
            source_type,
            format!("external:{id}"),
            title,
            status,
            created_at
        ],
    )
    .unwrap();
}
