use super::{ExecutionHandKind, ExecutionPermission, ExecutionPermissionRequirement};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExternalCapabilityCatalogConfig {
    enabled_capabilities: BTreeSet<String>,
}

impl ExternalCapabilityCatalogConfig {
    pub fn enable(mut self, capability_id: impl Into<String>) -> Self {
        self.enabled_capabilities.insert(capability_id.into());
        self
    }

    pub fn is_enabled(&self, capability_id: &str) -> bool {
        self.enabled_capabilities.contains(capability_id)
    }

    pub fn enabled_capability_ids(&self) -> impl Iterator<Item = &str> {
        self.enabled_capabilities.iter().map(String::as_str)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalCapabilityCatalog {
    descriptors: Vec<ExternalCapabilityDescriptor>,
}

impl Default for ExternalCapabilityCatalog {
    fn default() -> Self {
        Self::default_unavailable()
    }
}

impl ExternalCapabilityCatalog {
    pub fn new(descriptors: Vec<ExternalCapabilityDescriptor>) -> Self {
        Self { descriptors }
    }

    pub fn default_unavailable() -> Self {
        Self::new(default_external_capability_descriptors())
    }

    pub fn from_local_configuration(config: &ExternalCapabilityCatalogConfig) -> Self {
        let mut catalog = Self::default_unavailable();
        for capability_id in config.enabled_capability_ids() {
            catalog =
                catalog.with_availability(capability_id, ExternalCapabilityAvailability::Available);
        }
        catalog
    }

    pub fn descriptors(&self) -> Vec<ExternalCapabilityDescriptor> {
        self.descriptors.clone()
    }

    pub fn descriptor(&self, capability_id: &str) -> Option<ExternalCapabilityDescriptor> {
        self.descriptors
            .iter()
            .find(|descriptor| descriptor.id == capability_id)
            .cloned()
    }

    pub fn with_availability(
        mut self,
        capability_id: &str,
        availability: ExternalCapabilityAvailability,
    ) -> Self {
        if let Some(descriptor) = self
            .descriptors
            .iter_mut()
            .find(|descriptor| descriptor.id == capability_id)
        {
            descriptor.availability = availability;
        }
        self
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
    fn new_with_status(
        descriptor: ExternalCapabilityDescriptor,
        invocation: ExternalCapabilityInvocation,
        detail: impl Into<String>,
        sources: Vec<ExternalCapabilitySourceRecord>,
        status: ExternalCapabilityResultStatus,
    ) -> Self {
        Self {
            summary: format!("{} {}", descriptor.label, status.label()),
            descriptor,
            invocation,
            status,
            detail: detail.into(),
            sources,
        }
    }

    pub fn unavailable(
        descriptor: ExternalCapabilityDescriptor,
        invocation: ExternalCapabilityInvocation,
        detail: impl Into<String>,
    ) -> Self {
        Self::new_with_status(
            descriptor,
            invocation,
            detail,
            Vec::new(),
            ExternalCapabilityResultStatus::Unavailable,
        )
    }

    pub fn denied(
        descriptor: ExternalCapabilityDescriptor,
        invocation: ExternalCapabilityInvocation,
        detail: impl Into<String>,
    ) -> Self {
        Self::new_with_status(
            descriptor,
            invocation,
            detail,
            Vec::new(),
            ExternalCapabilityResultStatus::Denied,
        )
    }

    pub fn degraded(
        descriptor: ExternalCapabilityDescriptor,
        invocation: ExternalCapabilityInvocation,
        detail: impl Into<String>,
        sources: Vec<ExternalCapabilitySourceRecord>,
    ) -> Self {
        Self::new_with_status(
            descriptor,
            invocation,
            detail,
            sources,
            ExternalCapabilityResultStatus::Degraded,
        )
    }

    pub fn failed(
        descriptor: ExternalCapabilityDescriptor,
        invocation: ExternalCapabilityInvocation,
        detail: impl Into<String>,
    ) -> Self {
        Self::new_with_status(
            descriptor,
            invocation,
            detail,
            Vec::new(),
            ExternalCapabilityResultStatus::Failed,
        )
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
        ExternalCapabilityAuthPosture, ExternalCapabilityAvailability,
        ExternalCapabilityInvocation, ExternalCapabilityKind, ExternalCapabilityResult,
        ExternalCapabilityResultStatus, ExternalCapabilitySideEffectPosture,
        ExternalCapabilitySourceRecord, default_external_capability_descriptors,
    };
    use crate::domain::model::{ExecutionHandKind, ExecutionPermission};
    use serde_json::json;

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

    #[test]
    fn external_capability_result_states_construct_typed_denied_degraded_and_failed_evidence() {
        let descriptor = default_external_capability_descriptors()
            .into_iter()
            .next()
            .expect("web descriptor");
        let invocation = ExternalCapabilityInvocation::new(
            "web.search",
            "check current docs",
            json!({ "query": "paddles" }),
        );

        let denied = ExternalCapabilityResult::denied(
            descriptor.clone(),
            invocation.clone(),
            "network access requires approval",
        );
        let degraded = ExternalCapabilityResult::degraded(
            descriptor.clone(),
            invocation.clone(),
            "provider timed out after partial source discovery",
            vec![ExternalCapabilitySourceRecord {
                label: "Partial source".to_string(),
                locator: "https://example.com/partial".to_string(),
                snippet: "partial result".to_string(),
            }],
        );
        let failed = ExternalCapabilityResult::failed(
            descriptor,
            invocation,
            "provider returned malformed payload",
        );

        assert_eq!(denied.status, ExternalCapabilityResultStatus::Denied);
        assert!(denied.detail.contains("requires approval"));
        assert_eq!(degraded.status, ExternalCapabilityResultStatus::Degraded);
        assert_eq!(degraded.sources.len(), 1);
        assert_eq!(failed.status, ExternalCapabilityResultStatus::Failed);
        assert!(failed.detail.contains("malformed payload"));
    }
}
