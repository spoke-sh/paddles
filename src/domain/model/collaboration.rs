use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationMode {
    Planning,
    Execution,
    Review,
}

impl CollaborationMode {
    pub const ALL: [Self; 3] = [Self::Planning, Self::Execution, Self::Review];

    pub fn label(self) -> &'static str {
        match self {
            Self::Planning => "planning",
            Self::Execution => "execution",
            Self::Review => "review",
        }
    }

    pub fn state(self) -> CollaborationModeState {
        match self {
            Self::Planning => CollaborationModeState::new(
                self,
                CollaborationMutationPosture::FailClosedReadOnly,
                CollaborationOutputContract::Plan,
                CollaborationClarificationPolicy::BoundedStructured,
            ),
            Self::Execution => CollaborationModeState::new(
                self,
                CollaborationMutationPosture::DefaultExecution,
                CollaborationOutputContract::Execute,
                CollaborationClarificationPolicy::Disabled,
            ),
            Self::Review => CollaborationModeState::new(
                self,
                CollaborationMutationPosture::FailClosedReadOnly,
                CollaborationOutputContract::FindingsFirstReview,
                CollaborationClarificationPolicy::Disabled,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationMutationPosture {
    FailClosedReadOnly,
    DefaultExecution,
}

impl CollaborationMutationPosture {
    pub fn label(self) -> &'static str {
        match self {
            Self::FailClosedReadOnly => "fail_closed_read_only",
            Self::DefaultExecution => "default_execution",
        }
    }

    pub fn allows_mutation(self) -> bool {
        matches!(self, Self::DefaultExecution)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationOutputContract {
    Plan,
    Execute,
    FindingsFirstReview,
}

impl CollaborationOutputContract {
    pub fn label(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Execute => "execute",
            Self::FindingsFirstReview => "findings_first_review",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationClarificationPolicy {
    Disabled,
    BoundedStructured,
}

impl CollaborationClarificationPolicy {
    pub fn label(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::BoundedStructured => "bounded_structured",
        }
    }

    pub fn allows_clarification(self) -> bool {
        matches!(self, Self::BoundedStructured)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollaborationModeState {
    pub mode: CollaborationMode,
    pub mutation_posture: CollaborationMutationPosture,
    pub output_contract: CollaborationOutputContract,
    pub clarification_policy: CollaborationClarificationPolicy,
}

impl CollaborationModeState {
    pub fn new(
        mode: CollaborationMode,
        mutation_posture: CollaborationMutationPosture,
        output_contract: CollaborationOutputContract,
        clarification_policy: CollaborationClarificationPolicy,
    ) -> Self {
        Self {
            mode,
            mutation_posture,
            output_contract,
            clarification_policy,
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "{} / {} / {}",
            self.mode.label(),
            self.mutation_posture.label(),
            self.output_contract.label()
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum CollaborationModeRequestTarget {
    Known(CollaborationMode),
    Unsupported(String),
}

impl CollaborationModeRequestTarget {
    pub fn label(&self) -> &str {
        match self {
            Self::Known(mode) => mode.label(),
            Self::Unsupported(value) => value.as_str(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationModeRequestSource {
    Prompt,
    OperatorSurface,
    Transport,
    RuntimeDefault,
}

impl CollaborationModeRequestSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Prompt => "prompt",
            Self::OperatorSurface => "operator_surface",
            Self::Transport => "transport",
            Self::RuntimeDefault => "runtime_default",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollaborationModeRequest {
    pub target: CollaborationModeRequestTarget,
    pub source: CollaborationModeRequestSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl CollaborationModeRequest {
    pub fn new(
        target: CollaborationModeRequestTarget,
        source: CollaborationModeRequestSource,
        detail: Option<String>,
    ) -> Self {
        Self {
            target,
            source,
            detail,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationModeResultStatus {
    Applied,
    Defaulted,
    Invalid,
    Unavailable,
}

impl CollaborationModeResultStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Defaulted => "defaulted",
            Self::Invalid => "invalid",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollaborationModeResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<CollaborationModeRequest>,
    pub active: CollaborationModeState,
    pub status: CollaborationModeResultStatus,
    pub detail: String,
}

impl CollaborationModeResult {
    pub fn applied(request: CollaborationModeRequest, active: CollaborationModeState) -> Self {
        Self {
            request: Some(request),
            active,
            status: CollaborationModeResultStatus::Applied,
            detail: "collaboration mode request applied".to_string(),
        }
    }

    pub fn defaulted(active: CollaborationModeState, detail: impl Into<String>) -> Self {
        Self {
            request: None,
            active,
            status: CollaborationModeResultStatus::Defaulted,
            detail: detail.into(),
        }
    }

    pub fn invalid(
        request: CollaborationModeRequest,
        active: CollaborationModeState,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            request: Some(request),
            active,
            status: CollaborationModeResultStatus::Invalid,
            detail: detail.into(),
        }
    }

    pub fn unavailable(
        request: CollaborationModeRequest,
        active: CollaborationModeState,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            request: Some(request),
            active,
            status: CollaborationModeResultStatus::Unavailable,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuredClarificationKind {
    Plan,
    Approval,
    EnvironmentSelection,
}

impl StructuredClarificationKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Approval => "approval",
            Self::EnvironmentSelection => "environment_selection",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredClarificationOption {
    pub option_id: String,
    pub label: String,
    pub description: String,
}

impl StructuredClarificationOption {
    pub fn new(
        option_id: impl Into<String>,
        label: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            option_id: option_id.into(),
            label: label.into(),
            description: description.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredClarificationRequest {
    pub clarification_id: String,
    pub kind: StructuredClarificationKind,
    pub prompt: String,
    pub options: Vec<StructuredClarificationOption>,
    pub allow_free_form: bool,
}

impl StructuredClarificationRequest {
    pub fn new(
        clarification_id: impl Into<String>,
        kind: StructuredClarificationKind,
        prompt: impl Into<String>,
        options: Vec<StructuredClarificationOption>,
        allow_free_form: bool,
    ) -> Self {
        Self {
            clarification_id: clarification_id.into(),
            kind,
            prompt: prompt.into(),
            options,
            allow_free_form,
        }
    }

    pub fn answered(
        &self,
        answer: StructuredClarificationAnswer,
        detail: impl Into<String>,
    ) -> StructuredClarificationResult {
        StructuredClarificationResult {
            request: self.clone(),
            response: Some(answer),
            status: StructuredClarificationStatus::Answered,
            detail: detail.into(),
        }
    }

    pub fn unavailable(&self, detail: impl Into<String>) -> StructuredClarificationResult {
        StructuredClarificationResult {
            request: self.clone(),
            response: None,
            status: StructuredClarificationStatus::Unavailable,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StructuredClarificationAnswerPayload {
    SelectedOption { option_id: String },
    FreeForm { text: String },
}

impl StructuredClarificationAnswerPayload {
    pub fn summary(&self) -> &'static str {
        match self {
            Self::SelectedOption { .. } => "selected_option",
            Self::FreeForm { .. } => "free_form",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredClarificationAnswer {
    pub clarification_id: String,
    pub payload: StructuredClarificationAnswerPayload,
}

impl StructuredClarificationAnswer {
    pub fn new(
        clarification_id: impl Into<String>,
        payload: StructuredClarificationAnswerPayload,
    ) -> Self {
        Self {
            clarification_id: clarification_id.into(),
            payload,
        }
    }

    pub fn summary(&self) -> &'static str {
        self.payload.summary()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuredClarificationStatus {
    Requested,
    Answered,
    Rejected,
    Unavailable,
}

impl StructuredClarificationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Answered => "answered",
            Self::Rejected => "rejected",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredClarificationResult {
    pub request: StructuredClarificationRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<StructuredClarificationAnswer>,
    pub status: StructuredClarificationStatus,
    pub detail: String,
}

#[cfg(test)]
mod tests {
    use super::{
        CollaborationClarificationPolicy, CollaborationMode, CollaborationModeRequest,
        CollaborationModeRequestSource, CollaborationModeRequestTarget, CollaborationModeResult,
        CollaborationModeResultStatus, CollaborationMutationPosture, CollaborationOutputContract,
        StructuredClarificationAnswer, StructuredClarificationAnswerPayload,
        StructuredClarificationKind, StructuredClarificationOption, StructuredClarificationRequest,
        StructuredClarificationStatus,
    };

    #[test]
    fn collaboration_mode_states_stay_concise_and_fail_closed() {
        let planning = CollaborationMode::Planning.state();
        let execution = CollaborationMode::Execution.state();
        let review = CollaborationMode::Review.state();

        assert_eq!(planning.mode.label(), "planning");
        assert_eq!(
            planning.mutation_posture,
            CollaborationMutationPosture::FailClosedReadOnly
        );
        assert_eq!(
            planning.clarification_policy,
            CollaborationClarificationPolicy::BoundedStructured
        );
        assert_eq!(
            review.output_contract,
            CollaborationOutputContract::FindingsFirstReview
        );
        assert_eq!(
            execution.mutation_posture,
            CollaborationMutationPosture::DefaultExecution
        );
        assert_eq!(
            execution.summary(),
            "execution / default_execution / execute"
        );
    }

    #[test]
    fn collaboration_mode_results_preserve_invalid_and_unavailable_requests() {
        let invalid = CollaborationModeResult::invalid(
            CollaborationModeRequest::new(
                CollaborationModeRequestTarget::Unsupported("pairing".to_string()),
                CollaborationModeRequestSource::OperatorSurface,
                Some("unsupported request".to_string()),
            ),
            CollaborationMode::Execution.state(),
            "pairing mode is not available".to_string(),
        );
        let unavailable = CollaborationModeResult::unavailable(
            CollaborationModeRequest::new(
                CollaborationModeRequestTarget::Known(CollaborationMode::Review),
                CollaborationModeRequestSource::Transport,
                Some("transport does not expose review mode".to_string()),
            ),
            CollaborationMode::Execution.state(),
            "review mode is currently unavailable on this surface".to_string(),
        );

        assert_eq!(invalid.status, CollaborationModeResultStatus::Invalid);
        assert_eq!(
            invalid.request.as_ref().expect("request").target.label(),
            "pairing"
        );
        assert_eq!(invalid.active.mode, CollaborationMode::Execution);
        assert!(invalid.detail.contains("not available"));

        assert_eq!(
            unavailable.status,
            CollaborationModeResultStatus::Unavailable
        );
        assert_eq!(
            unavailable
                .request
                .as_ref()
                .expect("request")
                .target
                .label(),
            "review"
        );
        assert_eq!(unavailable.active.mode, CollaborationMode::Execution);
        assert!(unavailable.detail.contains("currently unavailable"));
    }

    #[test]
    fn clarification_contract_supports_bounded_options_and_free_form_answers() {
        let request = StructuredClarificationRequest::new(
            "clarification-1",
            StructuredClarificationKind::Plan,
            "Which implementation direction should Paddles take first?",
            vec![
                StructuredClarificationOption::new(
                    "direction-a",
                    "Direction A",
                    "Prefer the contract-first slice.",
                ),
                StructuredClarificationOption::new(
                    "direction-b",
                    "Direction B",
                    "Prefer the projection-first slice.",
                ),
            ],
            true,
        );
        let answer = StructuredClarificationAnswer::new(
            "clarification-1",
            StructuredClarificationAnswerPayload::FreeForm {
                text: "Start with the contract-first slice.".to_string(),
            },
        );
        let result = request.answered(answer.clone(), "operator answered".to_string());

        assert_eq!(request.kind, StructuredClarificationKind::Plan);
        assert_eq!(request.options.len(), 2);
        assert!(request.allow_free_form);
        assert_eq!(answer.summary(), "free_form");
        assert_eq!(result.status, StructuredClarificationStatus::Answered);
        assert_eq!(result.request.clarification_id, "clarification-1");
        assert_eq!(result.response, Some(answer));
    }
}
