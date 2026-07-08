use std::{
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::models::{TaskDto, TaskStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMarkdownTask {
    pub external_id: String,
    pub title: String,
    pub status: String,
    pub line_start: usize,
    pub heading_path: Vec<String>,
    pub tags: Vec<String>,
    pub file_path: PathBuf,
    pub raw_text_hash: String,
}

pub fn parse_markdown_file(path: &Path) -> Result<Vec<ParsedMarkdownTask>, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed to read markdown file {}: {err}", path.display()))?;
    let mut headings: Vec<String> = Vec::new();
    let mut tasks = Vec::new();

    for (index, line) in content.lines().enumerate() {
        let line_number = index + 1;
        if let Some((level, heading)) = parse_heading(line) {
            headings.truncate(level.saturating_sub(1));
            headings.push(heading);
            continue;
        }

        if let Some((checked, raw_title)) = parse_checklist_line(line) {
            let tags = extract_tags(raw_title);
            let title = strip_tags(raw_title);
            let raw_text_hash = hash_text(line);
            let external_id = format!("{}:{line_number}:{raw_text_hash}", path.display());

            tasks.push(ParsedMarkdownTask {
                external_id,
                title,
                status: if checked { "done" } else { "todo" }.to_string(),
                line_start: line_number,
                heading_path: headings.clone(),
                tags,
                file_path: path.to_path_buf(),
                raw_text_hash,
            });
        }
    }

    Ok(tasks)
}

pub fn parse_vault(source_id: &str, vault_path: &Path) -> Result<Vec<TaskDto>, String> {
    if !vault_path.exists() {
        return Err(format!(
            "Obsidian vault does not exist: {}",
            vault_path.display()
        ));
    }

    let mut tasks = Vec::new();
    for entry in WalkDir::new(vault_path).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        for parsed in parse_markdown_file(path)? {
            let id = format!(
                "task-{}",
                hash_text(&format!("{source_id}:{}", parsed.external_id))
            );
            let body = Some(parsed.heading_path.join(" / ")).filter(|body| !body.is_empty());
            let source_location_json = serde_json::json!({
                "filePath": parsed.file_path.display().to_string(),
                "lineStart": parsed.line_start,
                "lineEnd": parsed.line_start,
                "headingPath": parsed.heading_path,
            })
            .to_string();
            tasks.push(TaskDto {
                id,
                source_id: source_id.to_string(),
                source_type: "obsidian".to_string(),
                external_id: parsed.external_id,
                title: parsed.title,
                body,
                status: TaskStatus::from_db(&parsed.status),
                due_at: None,
                tags: parsed.tags,
                source_location_json,
                raw_text_hash: parsed.raw_text_hash,
                sync_state: "synced".to_string(),
                source_path: Some(parsed.file_path.display().to_string()),
                line_start: Some(parsed.line_start as i64),
                estimated_minutes: Some(30),
                created_at: None,
                updated_at: None,
                completed_at: None,
            });
        }
    }

    tasks.sort_by(|left, right| {
        left.source_path
            .cmp(&right.source_path)
            .then(left.line_start.cmp(&right.line_start))
    });

    Ok(tasks)
}

fn parse_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|ch| *ch == '#').count();
    if level == 0 || level > 6 || trimmed.chars().nth(level).is_none_or(|ch| ch != ' ') {
        return None;
    }
    Some((level, trimmed[level..].trim().to_string()))
}

fn parse_checklist_line(line: &str) -> Option<(bool, &str)> {
    let trimmed = line.trim_start();
    for marker in ["- [ ] ", "* [ ] "] {
        if let Some(title) = trimmed.strip_prefix(marker) {
            return Some((false, title.trim()));
        }
    }
    for marker in ["- [x] ", "- [X] ", "* [x] ", "* [X] "] {
        if let Some(title) = trimmed.strip_prefix(marker) {
            return Some((true, title.trim()));
        }
    }
    None
}

fn extract_tags(title: &str) -> Vec<String> {
    title
        .split_whitespace()
        .filter_map(|part| part.strip_prefix('#'))
        .filter(|tag| !tag.is_empty())
        .map(|tag| tag.trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '-' && ch != '_'))
        .filter(|tag| !tag.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn strip_tags(title: &str) -> String {
    title
        .split_whitespace()
        .filter(|part| !part.starts_with('#'))
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn hash_text(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    hex::encode(digest)[..16].to_string()
}
