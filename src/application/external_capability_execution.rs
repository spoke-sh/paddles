use super::{append_evidence_item, emit_execution_governance_decision, trim_for_planner};
use crate::domain::model::{
    ExecutionGovernanceOutcomeKind, ExecutionGovernanceProfile, ExecutionPermissionRequest,
    ExecutionPolicy, ExecutionPolicyEvaluationInput, ExternalCapabilityDescriptor,
    ExternalCapabilityInvocation, ExternalCapabilityResult, ExternalCapabilityResultStatus,
    ExternalCapabilitySourceRecord, TurnEvent, TurnEventSink,
};
use crate::domain::ports::{EvidenceItem, ExternalCapabilityBroker};
use crate::infrastructure::execution_governance::{
    ExecutionPolicyPermissionGate, summarize_governance_outcome,
};
use std::sync::Arc;

pub(super) struct ExternalCapabilityExecutionFrame<'a> {
    pub rationale: &'a str,
    pub evidence_limit: usize,
    pub evidence_items: &'a mut Vec<EvidenceItem>,
    pub call_id: &'a str,
    pub event_sink: &'a dyn TurnEventSink,
}

pub(super) fn execute_external_capability_action(
    broker: Arc<dyn ExternalCapabilityBroker>,
    governance_profile: &ExecutionGovernanceProfile,
    execution_policy: Option<&ExecutionPolicy>,
    invocation: &ExternalCapabilityInvocation,
    frame: ExternalCapabilityExecutionFrame<'_>,
) -> String {
    let Some(descriptor) = broker.descriptor(&invocation.capability_id) else {
        let summary = format_external_capability_outcome(
            None,
            invocation,
            ExternalCapabilityResultStatus::Unavailable,
            "External capability unavailable".to_string(),
            format!(
                "External capability `{}` is unknown to this runtime",
                invocation.capability_id
            ),
            &[],
        );
        frame.event_sink.emit(TurnEvent::ToolFinished {
            call_id: frame.call_id.to_string(),
            tool_name: "external_capability".to_string(),
            summary: summary.clone(),
        });
        append_evidence_item(
            frame.evidence_items,
            EvidenceItem {
                source: format!("external_capability:{}", invocation.capability_id),
                snippet: trim_for_planner(&summary, 1_200),
                rationale: frame.rationale.to_string(),
                rank: 0,
            },
            frame.evidence_limit,
        );
        return summary;
    };

    let governance_request = ExecutionPermissionRequest::new(
        descriptor.hand,
        descriptor.governance_requirement(format!(
            "invoke external capability `{}` for {}",
            descriptor.id, invocation.purpose
        )),
    );
    let policy_input = ExecutionPolicyEvaluationInput::tool("external_capability");
    let governance_outcome = ExecutionPolicyPermissionGate::evaluate(
        execution_policy,
        Some(governance_profile),
        &governance_request,
        &policy_input,
    );
    emit_execution_governance_decision(
        frame.event_sink,
        Some(frame.call_id),
        Some("external_capability"),
        governance_request.clone(),
        governance_outcome.clone(),
    );

    if governance_outcome.kind != ExecutionGovernanceOutcomeKind::Allowed {
        let result = ExternalCapabilityResult::denied(
            descriptor,
            invocation.clone(),
            summarize_governance_outcome(&governance_outcome),
        );
        return record_external_capability_result(&result, frame);
    }

    match broker.invoke(invocation) {
        Ok(result) => record_external_capability_result(&result, frame),
        Err(err) => {
            let detail = format!("External capability `{}` failed: {err:#}", descriptor.id);
            let result = ExternalCapabilityResult::failed(descriptor, invocation.clone(), detail);
            record_external_capability_result(&result, frame)
        }
    }
}

pub(super) fn summarize_external_capability_result(result: &ExternalCapabilityResult) -> String {
    format_external_capability_outcome(
        Some(&result.descriptor),
        &result.invocation,
        result.status,
        result.summary.clone(),
        result.detail.clone(),
        &result.sources,
    )
}

pub(super) fn format_external_capability_invocation(
    descriptor: Option<&ExternalCapabilityDescriptor>,
    invocation: &ExternalCapabilityInvocation,
) -> String {
    let (fabric, availability, auth, effects, evidence) = descriptor
        .map(|descriptor| {
            (
                descriptor.id.as_str(),
                descriptor.availability.label(),
                descriptor.auth_posture.label(),
                descriptor.side_effect_posture.label(),
                descriptor
                    .evidence_shape
                    .kinds
                    .iter()
                    .map(|kind| kind.label())
                    .collect::<Vec<_>>()
                    .join(","),
            )
        })
        .unwrap_or((
            invocation.capability_id.as_str(),
            "unknown",
            "unknown",
            "unknown",
            "unknown".to_string(),
        ));
    let mut lines = vec![
        format!(
            "fabric={fabric} availability={availability} auth={auth} effects={effects} evidence={evidence}"
        ),
        format!("purpose={}", invocation.purpose),
    ];
    if !invocation.payload.is_null() {
        lines.push(format!("payload={}", invocation.payload));
    }
    lines.join("\n")
}

fn record_external_capability_result(
    result: &ExternalCapabilityResult,
    frame: ExternalCapabilityExecutionFrame<'_>,
) -> String {
    let summary = summarize_external_capability_result(result);
    frame.event_sink.emit(TurnEvent::ToolFinished {
        call_id: frame.call_id.to_string(),
        tool_name: "external_capability".to_string(),
        summary: summary.clone(),
    });
    for item in external_capability_result_evidence_items(result, frame.rationale) {
        append_evidence_item(frame.evidence_items, item, frame.evidence_limit);
    }
    summary
}

fn external_capability_result_evidence_items(
    result: &ExternalCapabilityResult,
    rationale: &str,
) -> Vec<EvidenceItem> {
    let mut items = vec![EvidenceItem {
        source: format!("external_capability:{}", result.descriptor.id),
        snippet: trim_for_planner(&summarize_external_capability_result(result), 1_200),
        rationale: rationale.to_string(),
        rank: 0,
    }];
    items.extend(
        result
            .sources
            .iter()
            .enumerate()
            .map(|(index, source)| EvidenceItem {
                source: format!(
                    "external_capability:{}:{}",
                    result.descriptor.id, source.locator
                ),
                snippet: trim_for_planner(&format!("{}\n{}", source.label, source.snippet), 1_200),
                rationale: rationale.to_string(),
                rank: index + 1,
            }),
    );
    items
}

fn format_external_capability_outcome(
    descriptor: Option<&ExternalCapabilityDescriptor>,
    invocation: &ExternalCapabilityInvocation,
    status: ExternalCapabilityResultStatus,
    summary: String,
    detail: String,
    sources: &[ExternalCapabilitySourceRecord],
) -> String {
    let (fabric, availability, auth, effects) = descriptor
        .map(|descriptor| {
            (
                descriptor.id.as_str(),
                descriptor.availability.label(),
                descriptor.auth_posture.label(),
                descriptor.side_effect_posture.label(),
            )
        })
        .unwrap_or((
            invocation.capability_id.as_str(),
            "unavailable",
            "unknown",
            "unknown",
        ));
    let mut lines = vec![
        format!(
            "fabric={fabric} status={} availability={availability} auth={auth} effects={effects}",
            status.label()
        ),
        format!("purpose={}", invocation.purpose),
        format!("summary={summary}"),
        format!("detail={detail}"),
    ];
    if sources.is_empty() {
        lines.push("provenance=none".to_string());
    } else {
        lines.extend(
            sources
                .iter()
                .map(|source| format!("provenance={} -> {}", source.label, source.locator)),
        );
    }
    lines.join("\n")
}
