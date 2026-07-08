use std::fs;

use cole_lib::sources::obsidian::parse_markdown_file;

#[test]
fn parses_markdown_checklists_with_heading_and_line_numbers() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Today.md");
    fs::write(
        &path,
        "# Work\n\n- [ ] Draft connector notes #cole\n- [x] Create migration\n\n## Later\n* [ ] Review task flow\n",
    )
    .unwrap();

    let tasks = parse_markdown_file(&path).unwrap();

    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0].title, "Draft connector notes");
    assert_eq!(tasks[0].status, "todo");
    assert_eq!(tasks[0].line_start, 3);
    assert_eq!(tasks[0].heading_path, vec!["Work"]);
    assert_eq!(tasks[0].tags, vec!["cole"]);
    assert_eq!(tasks[1].status, "done");
    assert_eq!(tasks[2].heading_path, vec!["Work", "Later"]);
}
