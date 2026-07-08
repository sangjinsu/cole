use crate::models::{
    RecommendationFlowDto, RecommendationGroupDto, RecommendationTaskDto, TaskDto, TaskStatus,
};

pub fn build_recommendation_flow(tasks: &[TaskDto]) -> RecommendationFlowDto {
    let visible: Vec<&TaskDto> = tasks
        .iter()
        .filter(|task| task.status == TaskStatus::Todo)
        .take(7)
        .collect();

    let focus = visible.iter().take(2).copied().collect::<Vec<_>>();
    let next = visible.iter().skip(2).take(3).copied().collect::<Vec<_>>();
    let finish = visible.iter().skip(5).take(2).copied().collect::<Vec<_>>();

    let groups = vec![
        group(
            "focus",
            "Focus",
            "Start here. These tasks create the clearest momentum.",
            focus,
        ),
        group(
            "next",
            "Next",
            "Continue with the next actionable work.",
            next,
        ),
        group(
            "finish",
            "Finish",
            "Close small tasks to clear the board.",
            finish,
        ),
    ];

    let summary = if visible.is_empty() {
        "Cole has no open checklist items yet.".to_string()
    } else {
        format!(
            "Cole arranged {} task{} for today.",
            visible.len(),
            if visible.len() == 1 { "" } else { "s" }
        )
    };

    let openui_response = Some(to_openui_lang(&groups, &summary));

    RecommendationFlowDto {
        groups,
        summary,
        openui_response,
    }
}

fn group(id: &str, title: &str, reason: &str, tasks: Vec<&TaskDto>) -> RecommendationGroupDto {
    RecommendationGroupDto {
        id: id.to_string(),
        title: title.to_string(),
        reason: reason.to_string(),
        tasks: tasks
            .into_iter()
            .map(|task| RecommendationTaskDto {
                task_id: task.id.clone(),
                title: task.title.clone(),
                source_type: task.source_type.clone(),
                estimated_minutes: task.estimated_minutes,
            })
            .collect(),
    }
}

fn to_openui_lang(groups: &[RecommendationGroupDto], summary: &str) -> String {
    let mut lines = Vec::new();
    let group_refs = groups
        .iter()
        .enumerate()
        .map(|(group_index, group)| {
            let group_ref = format!("group_{group_index}");
            let task_refs = group
                .tasks
                .iter()
                .enumerate()
                .map(|(task_index, task)| {
                    let task_ref = format!("task_{group_index}_{task_index}");
                    lines.push(format!(
                        "{task_ref} = TaskCard(\"{}\", \"{}\", \"{}\", {})",
                        escape(&task.task_id),
                        escape(&task.title),
                        escape(&task.source_type),
                        task.estimated_minutes.unwrap_or(0)
                    ));
                    task_ref
                })
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!(
                "{group_ref} = TaskGroup(\"{}\", \"{}\", \"{}\", [{}])",
                escape(&group.id),
                escape(&group.title),
                escape(&group.reason),
                task_refs
            ));
            group_ref
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "root = TaskFlow(\"Today\", \"{}\", [{}])\n{}",
        escape(summary),
        group_refs,
        lines.join("\n")
    )
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
