use crate::domain::model::{
    AppliedEdit, CollaborationModeResult, PlanChecklistItem, StructuredClarificationResult,
};
use std::time::Duration;

pub(super) fn format_applied_edit_detail(edit: &AppliedEdit) -> String {
    let files = if edit.files.is_empty() {
        "(unknown file)".to_string()
    } else {
        edit.files.join(", ")
    };
    let mut detail = if edit.diff.trim().is_empty() {
        format!(
            "Files: {files}\nChange: +{} -{}",
            edit.insertions, edit.deletions
        )
    } else {
        format!(
            "Files: {files}\nChange: +{} -{}\n\n{}",
            edit.insertions, edit.deletions, edit.diff
        )
    };
    if !edit.evidence.is_empty() {
        detail.push_str("\n\nEvidence:");
        for evidence in &edit.evidence {
            detail.push_str(&format!(
                "\n- {}: {} - {}",
                evidence.kind.label(),
                evidence.status.label(),
                evidence.summary
            ));
        }
    }
    detail
}

pub(super) fn format_collaboration_mode_detail(result: &CollaborationModeResult) -> String {
    let mut lines = Vec::new();
    if let Some(request) = &result.request {
        lines.push(format!("requested={}", request.target.label()));
        lines.push(format!("source={}", request.source.label()));
        if let Some(detail) = request
            .detail
            .as_deref()
            .filter(|detail| !detail.trim().is_empty())
        {
            lines.push(format!("request_detail={}", detail.trim()));
        }
    }
    lines.push(format!("active={}", result.active.mode.label()));
    lines.push(format!(
        "mutation_posture={}",
        result.active.mutation_posture.label()
    ));
    lines.push(format!(
        "output_contract={}",
        result.active.output_contract.label()
    ));
    lines.push(format!(
        "clarification_policy={}",
        result.active.clarification_policy.label()
    ));
    if !result.detail.trim().is_empty() {
        lines.push(format!("detail={}", result.detail.trim()));
    }
    lines.join("\n")
}

pub(super) fn format_plan_checklist_detail(items: &[PlanChecklistItem]) -> String {
    if items.is_empty() {
        return "No checklist items recorded.".to_string();
    }

    items
        .iter()
        .map(|item| format!("{} {}", item.status.marker(), item.label))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn format_structured_clarification_detail(
    result: &StructuredClarificationResult,
) -> String {
    let mut lines = vec![
        format!("id={}", result.request.clarification_id),
        format!("status={}", result.status.label()),
        result.request.prompt.clone(),
    ];
    if !result.request.options.is_empty() {
        lines.push("Options:".to_string());
        lines.extend(
            result
                .request
                .options
                .iter()
                .map(|option| format!("- {}: {}", option.option_id, option.description.trim())),
        );
    }
    if let Some(response) = &result.response {
        lines.push(format!("response={}", response.summary()));
    }
    lines.push(format!(
        "allow_free_form={}",
        result.request.allow_free_form
    ));
    if !result.detail.trim().is_empty() {
        lines.push(format!("detail={}", result.detail.trim()));
    }
    lines.join("\n")
}

pub(super) fn format_applied_edit_text(tool_name: &str, edit: &AppliedEdit) -> String {
    let files = if edit.files.is_empty() {
        "unknown file".to_string()
    } else {
        edit.files.join(", ")
    };
    format!(
        "{tool_name}: {files} (+{} -{})",
        edit.insertions, edit.deletions
    )
}

pub(super) fn format_duration_compact(duration: Duration) -> String {
    if duration < Duration::from_secs(1) {
        return format!("{}ms", duration.as_millis());
    }

    if duration < Duration::from_secs(60) {
        return format!("{:.1}s", duration.as_secs_f64());
    }

    if duration < Duration::from_secs(3600) {
        let total_seconds = duration.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        return format!("{minutes}m {seconds:02}s");
    }

    let total_minutes = duration.as_secs() / 60;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{hours}h {minutes:02}m")
}
