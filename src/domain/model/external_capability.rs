use super::{ExecutionHandKind, ExecutionPermission, ExecutionPermissionRequirement};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalCapabilityKind {
    WebSearch,
    McpTool,
    ConnectorApp,
}

impl ExternalCapabilityKind {
    pub const ALL: [Self; 3] = [Self::WebSearch, Self::McpTool, Self::ConnectorApp];

    pub fn label(self) -> &'static str {
        match self {
            Self::WebSearch => "web_search",
            Self::McpTool => "mcp_tool",
            Self::ConnectorApp => "connector_app",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalCapabilityAvailability {
    Available,
    Disabled,
    Unauthenticated,
    Unavailable,
    Stale,
}

impl ExternalCapabilityAvailability {
    pub fn label(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Disabled => "disabled",
            Self::Unauthenticated => "unauthenticated",
            Self::Unavailable => "unavailable",
            Self::Stale => "stale",
        }
    }

    pub fn is_usable(self) -> bool {
        matches!(self, Self::Available)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalCapabilityAuthPosture {
    NoneRequired,
    Optional,
    Required,
}

impl ExternalCapabilityAuthPosture {
    pub fn label(self) -> &'static str {
        match self {
            Self::NoneRequired => "none_required",
            Self::Optional => "optional",
            Self::Required => "required",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalCapabilitySideEffectPosture {
    ReadOnly,
    PotentiallyMutating,
    Mutating,
}

impl ExternalCapabilitySideEffectPosture {
    pub fn label(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::PotentiallyMutating => "potentially_mutating",
            Self::Mutating => "mutating",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalCapabilityEvidenceKind {
    Citation,
    SourceLineage,
    StructuredRecord,
    RuntimeSummary,
}

impl ExternalCapabilityEvidenceKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Citation => "citation",
            Self::SourceLineage => "source_lineage",
            Self::StructuredRecord => "structured_record",
            Self::RuntimeSummary => "runtime_summary",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalCapabilityEvidenceShape {
    pub summary: String,
    pub kinds: Vec<ExternalCapabilityEvidenceKind>,
}

impl ExternalCapabilityEvidenceShape {
    pub fn new(summary: impl Into<String>, kinds: Vec<ExternalCapabilityEvidenceKind>) -> Self {
        let mut kinds = kinds;
        kinds.sort();
        kinds.dedup();
        Self {
            summary: summary.into(),
            kinds,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalCapabilityDescriptor {
    pub id: String,
    pub kind: ExternalCapabilityKind,
    pub label: String,
    pub summary: String,
    pub availability: ExternalCapabilityAvailability,
    pub auth_posture: ExternalCapabilityAuthPosture,
    pub side_effect_posture: ExternalCapabilitySideEffectPosture,
    pub hand: ExecutionHandKind,
    pub required_permissions: Vec<ExecutionPermission>,
    pub evidence_shape: ExternalCapabilityEvidenceShape,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalCapabilityDescriptorMetadata {
    pub availability: ExternalCapabilityAvailability,
    pub auth_posture: ExternalCapabilityAuthPosture,
    pub side_effect_posture: ExternalCapabilitySideEffectPosture,
    pub hand: ExecutionHandKind,
    pub required_permissions: Vec<ExecutionPermission>,
    pub evidence_shape: ExternalCapabilityEvidenceShape,
}

impl ExternalCapabilityDescriptorMetadata {
    pub fn new(
        availability: ExternalCapabilityAvailability,
        auth_posture: ExternalCapabilityAuthPosture,
        side_effect_posture: ExternalCapabilitySideEffectPosture,
        hand: ExecutionHandKind,
        required_permissions: Vec<ExecutionPermission>,
        evidence_shape: ExternalCapabilityEvidenceShape,
    ) -> Self {
        let mut required_permissions = required_permissions;
        required_permissions.sort();
        required_permissions.dedup();
        Self {
            availability,
            auth_posture,
            side_effect_posture,
            hand,
            required_permissions,
            evidence_shape,
        }
    }
}

impl ExternalCapabilityDescriptor {
    pub fn new(
        id: impl Into<String>,
        kind: ExternalCapabilityKind,
        label: impl Into<String>,
        summary: impl Into<String>,
        metadata: ExternalCapabilityDescriptorMetadata,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            label: label.into(),
            summary: summary.into(),
            availability: metadata.availability,
            auth_posture: metadata.auth_posture,
            side_effect_posture: metadata.side_effect_posture,
            hand: metadata.hand,
            required_permissions: metadata.required_permissions,
            evidence_shape: metadata.evidence_shape,
        }
    }

    pub fn governance_requirement(
        &self,
        purpose: impl Into<String>,
    ) -> ExecutionPermissionRequirement {
        ExecutionPermissionRequirement::new(purpose, self.required_permissions.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalCapabilityInvocation {
    pub capability_id: String,
    pub purpose: String,
    pub payload: Value,
}

impl ExternalCapabilityInvocation {
    pub fn new(
        capability_id: impl Into<String>,
        purpose: impl Into<String>,
        payload: Value,
    ) -> Self {
        Self {
            capability_id: capability_id.into(),
            purpose: purpose.into(),
            payload,
        }
    }

    pub fn summary(&self) -> String {
        format!("{} ({})", self.capability_id, self.purpose)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalCapabilitySourceRecord {
    pub label: String,
    pub locator: String,
    pub snippet: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalCapabilityResultStatus {
    Succeeded,
    Degraded,
    Unavailable,
    Denied,
    Failed,
}

impl ExternalCapabilityResultStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Degraded => "degraded",
            Self::Unavailable => "unavailable",
            Self::Denied => "denied",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalCapabilityResult {
    pub descriptor: ExternalCapabilityDescriptor,
    pub invocation: ExternalCapabilityInvocation,
    pub status: ExternalCapabilityResultStatus,
    pub summary: String,
    pub detail: String,
    pub sources: Vec<ExternalCapabilitySourceRecord>,
}

impl ExternalCapabilityResult {
    pub fn unavailable(
        descriptor: ExternalCapabilityDescriptor,
        invocation: ExternalCapabilityInvocation,
        detail: impl Into<String>,
    ) -> Self {
        let detail = detail.into();
        Self {
            summary: format!(
                "{} {}",
                descriptor.label,
                ExternalCapabilityResultStatus::Unavailable.label()
            ),
            descriptor,
            invocation,
            status: ExternalCapabilityResultStatus::Unavailable,
            detail,
            sources: Vec::new(),
        }
    }
}

pub fn default_external_capability_descriptors() -> Vec<ExternalCapabilityDescriptor> {
    vec![
        ExternalCapabilityDescriptor::new(
            "web.search",
            ExternalCapabilityKind::WebSearch,
            "Web Search",
            "Search current public web sources and return citation-backed evidence.",
            ExternalCapabilityDescriptorMetadata::new(
                ExternalCapabilityAvailability::Unavailable,
                ExternalCapabilityAuthPosture::NoneRequired,
                ExternalCapabilitySideEffectPosture::ReadOnly,
                ExecutionHandKind::TransportMediator,
                vec![ExecutionPermission::AccessNetwork],
                ExternalCapabilityEvidenceShape::new(
                    "public web search should yield source-backed citations and runtime summaries",
                    vec![
                        ExternalCapabilityEvidenceKind::Citation,
                        ExternalCapabilityEvidenceKind::RuntimeSummary,
                        ExternalCapabilityEvidenceKind::SourceLineage,
                    ],
                ),
            ),
        ),
        ExternalCapabilityDescriptor::new(
            "mcp.tool",
            ExternalCapabilityKind::McpTool,
            "MCP Tool",
            "Invoke an MCP-backed tool through the typed transport mediation boundary.",
            ExternalCapabilityDescriptorMetadata::new(
                ExternalCapabilityAvailability::Unavailable,
                ExternalCapabilityAuthPosture::Optional,
                ExternalCapabilitySideEffectPosture::PotentiallyMutating,
                ExecutionHandKind::TransportMediator,
                vec![
                    ExecutionPermission::AccessCredentials,
                    ExecutionPermission::AccessNetwork,
                ],
                ExternalCapabilityEvidenceShape::new(
                    "MCP tools should yield structured records plus runtime summaries and lineage",
                    vec![
                        ExternalCapabilityEvidenceKind::RuntimeSummary,
                        ExternalCapabilityEvidenceKind::SourceLineage,
                        ExternalCapabilityEvidenceKind::StructuredRecord,
                    ],
                ),
            ),
        ),
        ExternalCapabilityDescriptor::new(
            "connector.app_action",
            ExternalCapabilityKind::ConnectorApp,
            "Connector App Action",
            "Invoke a connector-backed application action through the credential-mediated transport boundary.",
            ExternalCapabilityDescriptorMetadata::new(
                ExternalCapabilityAvailability::Unavailable,
                ExternalCapabilityAuthPosture::Required,
                ExternalCapabilitySideEffectPosture::PotentiallyMutating,
                ExecutionHandKind::TransportMediator,
                vec![
                    ExecutionPermission::AccessCredentials,
                    ExecutionPermission::AccessNetwork,
                ],
                ExternalCapabilityEvidenceShape::new(
                    "connector actions should yield structured records, lineage, and operator-visible summaries",
                    vec![
                        ExternalCapabilityEvidenceKind::RuntimeSummary,
                        ExternalCapabilityEvidenceKind::SourceLineage,
                        ExternalCapabilityEvidenceKind::StructuredRecord,
                    ],
                ),
            ),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        ExternalCapabilityAuthPosture, ExternalCapabilityAvailability, ExternalCapabilityKind,
        ExternalCapabilitySideEffectPosture, default_external_capability_descriptors,
    };
    use crate::domain::model::{ExecutionHandKind, ExecutionPermission};

    #[test]
    fn default_descriptors_cover_web_mcp_and_connector_fabrics() {
        let descriptors = default_external_capability_descriptors();

        assert_eq!(descriptors.len(), 3);
        assert_eq!(
            descriptors
                .iter()
                .map(|descriptor| descriptor.kind)
                .collect::<Vec<_>>(),
            vec![
                ExternalCapabilityKind::WebSearch,
                ExternalCapabilityKind::McpTool,
                ExternalCapabilityKind::ConnectorApp,
            ]
        );
        assert!(descriptors.iter().all(|descriptor| {
            descriptor.availability == ExternalCapabilityAvailability::Unavailable
                && descriptor.hand == ExecutionHandKind::TransportMediator
        }));
    }

    #[test]
    fn descriptors_carry_governance_and_evidence_metadata_without_surface_specific_paths() {
        let descriptors = default_external_capability_descriptors();
        let web = &descriptors[0];
        let connector = &descriptors[2];

        assert_eq!(
            web.auth_posture,
            ExternalCapabilityAuthPosture::NoneRequired
        );
        assert_eq!(
            web.side_effect_posture,
            ExternalCapabilitySideEffectPosture::ReadOnly
        );
        assert!(
            web.required_permissions
                .contains(&ExecutionPermission::AccessNetwork)
        );

        assert_eq!(
            connector.auth_posture,
            ExternalCapabilityAuthPosture::Required
        );
        assert_eq!(
            connector.side_effect_posture,
            ExternalCapabilitySideEffectPosture::PotentiallyMutating
        );
        assert!(
            connector
                .required_permissions
                .contains(&ExecutionPermission::AccessCredentials)
        );
        assert!(!connector.evidence_shape.kinds.is_empty());
    }
}
