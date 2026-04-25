use super::{
    AppliedEdit, CollaborationModeResult, ConversationThreadRef, PlanChecklistItem,
    StructuredClarificationResult, ThreadDecision, ThreadDecisionKind, ThreadMergeRecord,
    TraceBranchId, TraceRecordKind, TurnEvent, TurnTraceId,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnControlOperation {
    Steer,
    Interrupt,
    Resume,
}

impl TurnControlOperation {
    pub fn label(self) -> &'static str {
        match self {
            Self::Steer => "steer",
            Self::Interrupt => "interrupt",
            Self::Resume => "resume",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreadControlOperation {
    ContinueCurrent,
    OpenChild,
    MergeIntoTarget,
    Resume,
    Rollback,
    Archive,
}

impl ThreadControlOperation {
    pub fn label(self) -> &'static str {
        match self {
            Self::ContinueCurrent => "continue_current",
            Self::OpenChild => "open_child",
            Self::MergeIntoTarget => "merge_into_target",
            Self::Resume => "resume",
            Self::Rollback => "rollback",
            Self::Archive => "archive",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "scope", content = "operation", rename_all = "snake_case")]
pub enum ControlOperation {
    Turn(TurnControlOperation),
    Thread(ThreadControlOperation),
}

impl ControlOperation {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Turn(operation) => operation.label(),
            Self::Thread(operation) => operation.label(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlResultStatus {
    Accepted,
    Applied,
    Rejected,
    Stale,
    Unavailable,
}

impl ControlResultStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Applied => "applied",
            Self::Rejected => "rejected",
            Self::Stale => "stale",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ControlSubject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_id: Option<TurnTraceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<ConversationThreadRef>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlResult {
    pub operation: ControlOperation,
    pub status: ControlResultStatus,
    pub subject: ControlSubject,
    pub detail: String,
}

impl ControlResult {
    pub fn summary(&self) -> String {
        format!("{} {}", self.operation.label(), self.status.label())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlanRuntimeItem {
    pub items: Vec<PlanChecklistItem>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DiffRuntimeItem {
    pub files: Vec<String>,
    pub diff: String,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandRuntimePhase {
    Requested,
    StreamingStdout,
    StreamingStderr,
    Finished,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CommandRuntimeItem {
    pub call_id: String,
    pub tool_name: String,
    pub phase: CommandRuntimePhase,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FileRuntimeOperation {
    Updated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FileRuntimeItem {
    pub path: String,
    pub operation: FileRuntimeOperation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ControlRuntimeItem {
    pub result: ControlResult,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CollaborationRuntimeItem {
    pub result: CollaborationModeResult,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ClarificationRuntimeItem {
    pub result: StructuredClarificationResult,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum RuntimeItem {
    Plan(PlanRuntimeItem),
    Diff(DiffRuntimeItem),
    Command(CommandRuntimeItem),
    File(FileRuntimeItem),
    Control(ControlRuntimeItem),
    Collaboration(CollaborationRuntimeItem),
    Clarification(ClarificationRuntimeItem),
}

impl RuntimeItem {
    pub fn kind_label(&self) -> &'static str {
        match self {
            Self::Plan(_) => "plan",
            Self::Diff(_) => "diff",
            Self::Command(_) => "command",
            Self::File(_) => "file",
            Self::Control(_) => "control",
            Self::Collaboration(_) => "collaboration",
            Self::Clarification(_) => "clarification",
        }
    }
}

impl AppliedEdit {
    pub fn runtime_items(&self) -> Vec<RuntimeItem> {
        let mut items = vec![RuntimeItem::Diff(DiffRuntimeItem {
            files: self.files.clone(),
            diff: self.diff.clone(),
            insertions: self.insertions,
            deletions: self.deletions,
        })];
        for path in &self.files {
            items.push(RuntimeItem::File(FileRuntimeItem {
                path: path.clone(),
                operation: FileRuntimeOperation::Updated,
            }));
        }
        items
    }
}

impl TurnEvent {
    pub fn runtime_items(&self) -> Vec<RuntimeItem> {
        match self {
            Self::PlanUpdated { items } => vec![RuntimeItem::Plan(PlanRuntimeItem {
                items: items.clone(),
            })],
            Self::ToolCalled {
                call_id,
                tool_name,
                invocation,
            } => vec![RuntimeItem::Command(CommandRuntimeItem {
                call_id: call_id.clone(),
                tool_name: tool_name.clone(),
                phase: CommandRuntimePhase::Requested,
                detail: invocation.clone(),
            })],
            Self::ToolOutput {
                call_id,
                tool_name,
                stream,
                output,
            } => vec![RuntimeItem::Command(CommandRuntimeItem {
                call_id: call_id.clone(),
                tool_name: tool_name.clone(),
                phase: if stream.eq_ignore_ascii_case("stderr") {
                    CommandRuntimePhase::StreamingStderr
                } else {
                    CommandRuntimePhase::StreamingStdout
                },
                detail: output.clone(),
            })],
            Self::ToolFinished {
                call_id,
                tool_name,
                summary,
            } => vec![RuntimeItem::Command(CommandRuntimeItem {
                call_id: call_id.clone(),
                tool_name: tool_name.clone(),
                phase: CommandRuntimePhase::Finished,
                detail: summary.clone(),
            })],
            Self::WorkspaceEditApplied { edit, .. } => edit.runtime_items(),
            Self::ThreadDecisionApplied {
                decision,
                target_thread,
                rationale,
                ..
            } => thread_control_operation_from_label(decision)
                .map(|operation| {
                    RuntimeItem::Control(ControlRuntimeItem {
                        result: ControlResult {
                            operation: ControlOperation::Thread(operation),
                            status: ControlResultStatus::Accepted,
                            subject: ControlSubject {
                                turn_id: None,
                                thread: Some(thread_ref_from_stable_id(target_thread)),
                            },
                            detail: rationale.clone(),
                        },
                    })
                })
                .into_iter()
                .collect(),
            Self::ThreadMerged {
                target_thread,
                summary,
                ..
            } => vec![RuntimeItem::Control(ControlRuntimeItem {
                result: ControlResult {
                    operation: ControlOperation::Thread(ThreadControlOperation::MergeIntoTarget),
                    status: ControlResultStatus::Applied,
                    subject: ControlSubject {
                        turn_id: None,
                        thread: Some(thread_ref_from_stable_id(target_thread)),
                    },
                    detail: summary
                        .clone()
                        .unwrap_or_else(|| "thread merged".to_string()),
                },
            })],
            Self::ControlStateChanged { result } => {
                vec![RuntimeItem::Control(ControlRuntimeItem {
                    result: result.clone(),
                })]
            }
            Self::CollaborationModeChanged { result } => {
                vec![RuntimeItem::Collaboration(CollaborationRuntimeItem {
                    result: result.clone(),
                })]
            }
            Self::StructuredClarificationChanged { result } => {
                vec![RuntimeItem::Clarification(ClarificationRuntimeItem {
                    result: result.clone(),
                })]
            }
            _ => Vec::new(),
        }
    }
}

pub fn thread_control_operation(kind: ThreadDecisionKind) -> ThreadControlOperation {
    match kind {
        ThreadDecisionKind::ContinueCurrent => ThreadControlOperation::ContinueCurrent,
        ThreadDecisionKind::OpenChildThread => ThreadControlOperation::OpenChild,
        ThreadDecisionKind::MergeIntoTarget => ThreadControlOperation::MergeIntoTarget,
    }
}

pub fn thread_control_result(decision: &ThreadDecision) -> ControlResult {
    ControlResult {
        operation: ControlOperation::Thread(thread_control_operation(decision.kind)),
        status: ControlResultStatus::Accepted,
        subject: ControlSubject {
            turn_id: None,
            thread: Some(decision.target_thread.clone()),
        },
        detail: decision.rationale.clone(),
    }
}

pub fn trace_control_result(record: &TraceRecordKind) -> Option<ControlResult> {
    match record {
        TraceRecordKind::ControlResultRecorded(result) => Some(result.clone()),
        TraceRecordKind::ThreadDecisionSelected(decision) => Some(thread_control_result(decision)),
        TraceRecordKind::ThreadMerged(merge) => Some(thread_merge_control_result(merge)),
        _ => None,
    }
}

fn thread_merge_control_result(merge: &ThreadMergeRecord) -> ControlResult {
    ControlResult {
        operation: ControlOperation::Thread(ThreadControlOperation::MergeIntoTarget),
        status: ControlResultStatus::Applied,
        subject: ControlSubject {
            turn_id: None,
            thread: Some(merge.target_thread.clone()),
        },
        detail: merge
            .decision
            .merge_summary
            .clone()
            .unwrap_or_else(|| merge.decision.rationale.clone()),
    }
}

fn thread_control_operation_from_label(label: &str) -> Option<ThreadControlOperation> {
    match label {
        "continue-current-thread" => Some(ThreadControlOperation::ContinueCurrent),
        "open-child-thread" => Some(ThreadControlOperation::OpenChild),
        "merge-into-target" => Some(ThreadControlOperation::MergeIntoTarget),
        _ => None,
    }
}

fn thread_ref_from_stable_id(stable_id: &str) -> ConversationThreadRef {
    if stable_id == "mainline" {
        ConversationThreadRef::Mainline
    } else {
        ConversationThreadRef::Branch(
            TraceBranchId::new(stable_id.to_string())
                .expect("runtime control thread ids come from recorded branch ids"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ClarificationRuntimeItem, CollaborationRuntimeItem, CommandRuntimePhase, ControlOperation,
        ControlResultStatus, DiffRuntimeItem, FileRuntimeItem, FileRuntimeOperation, RuntimeItem,
        ThreadControlOperation, thread_control_operation, thread_control_result,
        trace_control_result,
    };
    use crate::domain::model::{
        AppliedEdit, CollaborationMode, CollaborationModeRequest, CollaborationModeRequestSource,
        CollaborationModeRequestTarget, ConversationThreadRef, PlanChecklistItem,
        PlanChecklistItemStatus, StructuredClarificationKind, StructuredClarificationOption,
        StructuredClarificationRequest, ThreadCandidateId, ThreadDecision, ThreadDecisionId,
        ThreadDecisionKind, ThreadMergeRecord, TraceRecordKind, TurnEvent,
    };

    #[test]
    fn thread_decisions_lower_to_surface_agnostic_control_operations() {
        let decision = ThreadDecision {
            decision_id: ThreadDecisionId::new("decision-1").expect("decision"),
            candidate_id: ThreadCandidateId::new("candidate-1").expect("candidate"),
            kind: ThreadDecisionKind::OpenChildThread,
            rationale: "branch investigation".to_string(),
            target_thread: ConversationThreadRef::Mainline,
            new_thread_label: Some("investigate".to_string()),
            merge_mode: None,
            merge_summary: None,
        };

        assert_eq!(
            thread_control_operation(decision.kind),
            ThreadControlOperation::OpenChild
        );
        assert_eq!(
            thread_control_result(&decision).operation,
            ControlOperation::Thread(ThreadControlOperation::OpenChild)
        );
        assert_eq!(
            thread_control_result(&decision).status,
            ControlResultStatus::Accepted
        );
    }

    #[test]
    fn turn_events_project_shared_runtime_item_vocabulary() {
        let plan_items = vec![PlanChecklistItem {
            id: "acquire".to_string(),
            label: "Acquire evidence".to_string(),
            status: PlanChecklistItemStatus::Pending,
        }];
        assert_eq!(
            TurnEvent::PlanUpdated {
                items: plan_items.clone(),
            }
            .runtime_items(),
            vec![RuntimeItem::Plan(super::PlanRuntimeItem {
                items: plan_items
            })]
        );

        assert_eq!(
            TurnEvent::ToolCalled {
                call_id: "call-1".to_string(),
                tool_name: "shell".to_string(),
                invocation: "rg control".to_string(),
            }
            .runtime_items(),
            vec![RuntimeItem::Command(super::CommandRuntimeItem {
                call_id: "call-1".to_string(),
                tool_name: "shell".to_string(),
                phase: CommandRuntimePhase::Requested,
                detail: "rg control".to_string(),
            })]
        );

        let edit = AppliedEdit {
            files: vec!["src/application/mod.rs".to_string()],
            diff: "@@".to_string(),
            insertions: 4,
            deletions: 2,
            evidence: Vec::new(),
        };
        assert_eq!(
            edit.runtime_items(),
            vec![
                RuntimeItem::Diff(DiffRuntimeItem {
                    files: vec!["src/application/mod.rs".to_string()],
                    diff: "@@".to_string(),
                    insertions: 4,
                    deletions: 2,
                }),
                RuntimeItem::File(FileRuntimeItem {
                    path: "src/application/mod.rs".to_string(),
                    operation: FileRuntimeOperation::Updated,
                }),
            ]
        );

        let collaboration = crate::domain::model::CollaborationModeResult::invalid(
            CollaborationModeRequest::new(
                CollaborationModeRequestTarget::Unsupported("pairing".to_string()),
                CollaborationModeRequestSource::OperatorSurface,
                Some("unsupported request".to_string()),
            ),
            CollaborationMode::Execution.state(),
            "unsupported collaboration mode `pairing`; continuing in execution mode",
        );
        assert_eq!(
            TurnEvent::CollaborationModeChanged {
                result: collaboration.clone(),
            }
            .runtime_items(),
            vec![RuntimeItem::Collaboration(CollaborationRuntimeItem {
                result: collaboration,
            })]
        );

        let clarification = StructuredClarificationRequest::new(
            "planning-mode-clarification",
            StructuredClarificationKind::Approval,
            "Planning mode is read-only, so I stopped before `write_file README.md`.",
            vec![
                StructuredClarificationOption::new(
                    "stay_in_planning",
                    "Stay in planning",
                    "Keep the turn read-only and return a plan.",
                ),
                StructuredClarificationOption::new(
                    "switch_to_execution",
                    "Switch to execution",
                    "Rerun in execution mode so Paddles can apply the change.",
                ),
            ],
            false,
        )
        .requested("planning mode blocked a mutating action");
        assert_eq!(
            TurnEvent::StructuredClarificationChanged {
                result: clarification.clone(),
            }
            .runtime_items(),
            vec![RuntimeItem::Clarification(ClarificationRuntimeItem {
                result: clarification,
            })]
        );
    }

    #[test]
    fn trace_record_kinds_expose_replayable_control_results() {
        let decision = ThreadDecision {
            decision_id: ThreadDecisionId::new("decision-2").expect("decision"),
            candidate_id: ThreadCandidateId::new("candidate-2").expect("candidate"),
            kind: ThreadDecisionKind::MergeIntoTarget,
            rationale: "return findings".to_string(),
            target_thread: ConversationThreadRef::Mainline,
            new_thread_label: None,
            merge_mode: None,
            merge_summary: Some("merged".to_string()),
        };
        let record = TraceRecordKind::ThreadDecisionSelected(decision.clone());
        assert_eq!(
            trace_control_result(&record)
                .expect("control result")
                .operation,
            ControlOperation::Thread(ThreadControlOperation::MergeIntoTarget)
        );

        let merge_record = TraceRecordKind::ThreadMerged(ThreadMergeRecord {
            decision,
            source_thread: ConversationThreadRef::Branch(
                crate::domain::model::TraceBranchId::new("task-1.thread-0001").expect("branch"),
            ),
            target_thread: ConversationThreadRef::Mainline,
            summary_artifact: None,
        });
        assert_eq!(
            trace_control_result(&merge_record)
                .expect("merge control result")
                .status,
            ControlResultStatus::Applied
        );
    }
}
