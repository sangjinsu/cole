use cole_lib::models::{TaskDto, TaskStatus};
use cole_lib::recommendations::build_recommendation_flow;

fn task(id: &str, title: &str, estimated_minutes: Option<i64>) -> TaskDto {
    TaskDto {
        id: id.to_string(),
        source_id: "source-1".to_string(),
        source_type: "obsidian".to_string(),
        external_id: id.to_string(),
        title: title.to_string(),
        body: None,
        status: TaskStatus::Todo,
        due_at: None,
        tags: vec![],
        source_location_json: "{}".to_string(),
        raw_text_hash: id.to_string(),
        sync_state: "synced".to_string(),
        source_path: Some("vault/today.md".to_string()),
        line_start: Some(1),
        estimated_minutes,
        created_at: None,
        updated_at: None,
        completed_at: None,
    }
}

#[test]
fn builds_focus_next_finish_groups_and_limits_visible_tasks() {
    let tasks = vec![
        task("1", "Urgent project follow-up", Some(45)),
        task("2", "Review notes", Some(20)),
        task("3", "Quick reply", Some(10)),
        task("4", "Plan next draft", Some(30)),
        task("5", "Update checklist", Some(15)),
        task("6", "Read source docs", Some(60)),
        task("7", "Clean inbox", Some(10)),
        task("8", "Extra task should be hidden", Some(10)),
    ];

    let flow = build_recommendation_flow(&tasks);

    assert_eq!(flow.groups.len(), 3);
    assert_eq!(flow.groups[0].id, "focus");
    assert_eq!(flow.groups[1].id, "next");
    assert_eq!(flow.groups[2].id, "finish");
    assert_eq!(
        flow.groups
            .iter()
            .map(|group| group.tasks.len())
            .sum::<usize>(),
        7
    );
    assert!(flow
        .openui_response
        .as_deref()
        .unwrap()
        .starts_with("root = TaskFlow"));
}
