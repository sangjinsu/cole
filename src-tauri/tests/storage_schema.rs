use std::collections::HashSet;

use cole_lib::db::AppState;
use rusqlite::Connection;

#[test]
fn task_schema_contains_minimum_local_task_fields() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("cole.sqlite");
    let state = AppState::new(db_path.clone()).unwrap();
    drop(state);

    let conn = Connection::open(db_path).unwrap();
    let mut stmt = conn.prepare("PRAGMA table_info(tasks)").unwrap();
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<Result<HashSet<_>, _>>()
        .unwrap();

    for field in [
        "id",
        "source_id",
        "source_type",
        "external_id",
        "title",
        "body",
        "status",
        "due_at",
        "tags_json",
        "source_location_json",
        "raw_text_hash",
        "sync_state",
        "estimated_minutes",
        "created_at",
        "updated_at",
        "completed_at",
    ] {
        assert!(columns.contains(field), "missing task column: {field}");
    }
}
