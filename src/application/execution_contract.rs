use crate::domain::model::{
    CollaborationMode, CollaborationModeResult, ExecutionGovernanceProfile,
    ExecutionHandDiagnostic, ExternalCapabilityDescriptor, InstructionFrame,
};
use crate::domain::ports::{
    ContextGatherer, GathererCapability, GroundingDomain, GroundingRequirement, PlannerConfig,
    PlannerExecutionContract, RetrievalStrategy, WorkspaceActionCapability,
    WorkspaceCapabilitySurface, WorkspaceToolCapability,
};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ExecutionContractService;

impl ExecutionContractService {
    pub(super) const fn new() -> Self {
        Self
    }

    pub(super) fn build(&self, context: ExecutionContractContext<'_>) -> PlannerExecutionContract {
        let ExecutionContractContext {
            workspace_capability_surface,
            execution_hands,
            governance_profile,
            external_capabilities,
            gatherer,
            collaboration,
            instruction_frame,
            grounding,
        } = context;
        let mut capability_manifest = workspace_capability_surface
            .actions
            .iter()
            .map(|capability| format_workspace_action_capability(capability, collaboration))
            .collect::<Vec<_>>();
        capability_manifest.extend(
            workspace_capability_surface
                .tools
                .iter()
                .map(format_workspace_tool_capability),
        );
        capability_manifest.extend(
            workspace_capability_surface
                .notes
                .iter()
                .map(|note| format!("workspace note: {note}")),
        );
        capability_manifest.extend(
            execution_hands
                .iter()
                .map(format_execution_hand_capability_line),
        );
        capability_manifest.extend(retrieval_capability_lines(gatherer));
        capability_manifest.extend(external_capabilities.iter().map(|descriptor| {
            format!(
                "external capability {}: {}",
                descriptor.id,
                format_external_capability_catalog_entry(descriptor)
            )
        }));
        capability_manifest.push(match governance_profile {
            Some(profile) => {
                format!(
                    "execution governance: {} {}",
                    profile.summary(),
                    profile.detail()
                )
            }
            None => {
                "execution governance: unavailable; mutating or networked actions may fail closed"
                    .to_string()
            }
        });

        let mut completion_contract = vec![
            "Choose only actions supported by the capability manifest. If a capability is blocked or unavailable, choose a different bounded action."
                .to_string(),
            "When a task depends on a local program that is not already observed in the capability manifest, choose a bounded single-step probe such as `inspect` `command -v <tool>` before depending on it."
                .to_string(),
            "`inspect` is only for single read-only probes. Do not chain commands or use redirection; use `shell` for broader governed workspace command execution."
                .to_string(),
        ];

        match collaboration.active.mode {
            CollaborationMode::Planning => completion_contract.push(
                "Planning mode is read-only. Do not choose mutating workspace actions or shell commands that could change the repository."
                    .to_string(),
            ),
            CollaborationMode::Review => completion_contract.push(
                "Review mode is read-only. Inspect local evidence and stop at findings; do not choose mutating workspace actions."
                    .to_string(),
            ),
            CollaborationMode::Execution => completion_contract.push(
                "Execution mode allows mutating workspace actions, but they still run through execution governance and may be denied or downgraded."
                    .to_string(),
            ),
        }

        if let Some(frame) = instruction_frame {
            if frame.requires_applied_edit() {
                let mut line = "The turn is not complete until an applied workspace edit succeeds."
                    .to_string();
                if let Some(candidates) = frame.candidate_summary() {
                    line.push_str(&format!(" Current candidate files: {candidates}."));
                }
                completion_contract.push(line);
            }
            if frame.requires_applied_commit() {
                completion_contract.push(
                    "The turn is not complete until the requested git commit has been recorded in the workspace."
                        .to_string(),
                );
            }
        }

        if let Some(grounding) = grounding {
            let mut line = format!(
                "Do not stop with a final answer until {} evidence has been assembled.",
                grounding_domain_label(grounding.domain)
            );
            if let Some(reason) = grounding
                .reason
                .as_deref()
                .filter(|reason| !reason.trim().is_empty())
            {
                line.push_str(&format!(" Reason: {}.", reason.trim()));
            }
            completion_contract.push(line);
        }

        PlannerExecutionContract {
            capability_manifest,
            completion_contract,
        }
    }
}

pub(super) struct ExecutionContractContext<'a> {
    pub(super) workspace_capability_surface: &'a WorkspaceCapabilitySurface,
    pub(super) execution_hands: &'a [ExecutionHandDiagnostic],
    pub(super) governance_profile: Option<&'a ExecutionGovernanceProfile>,
    pub(super) external_capabilities: &'a [ExternalCapabilityDescriptor],
    pub(super) gatherer: Option<&'a dyn ContextGatherer>,
    pub(super) collaboration: &'a CollaborationModeResult,
    pub(super) instruction_frame: Option<&'a InstructionFrame>,
    pub(super) grounding: Option<&'a GroundingRequirement>,
}

pub(super) fn format_gatherer_capability(capability: &GathererCapability) -> String {
    match capability {
        GathererCapability::Available => "available".to_string(),
        GathererCapability::Warming { reason } => format!("warming: {reason}"),
        GathererCapability::Unsupported { reason } => format!("unsupported: {reason}"),
        GathererCapability::HarnessRequired { reason } => {
            format!("harness-required: {reason}")
        }
    }
}

pub(super) fn gatherer_readiness_label(capability: &GathererCapability) -> &'static str {
    match capability {
        GathererCapability::Available => "available",
        GathererCapability::Warming { .. } => "warming",
        GathererCapability::Unsupported { .. } => "unsupported",
        GathererCapability::HarnessRequired { .. } => "harness-required",
    }
}

pub(super) fn format_external_capability_catalog_entry(
    descriptor: &ExternalCapabilityDescriptor,
) -> String {
    let evidence = descriptor
        .evidence_shape
        .kinds
        .iter()
        .map(|kind| kind.label())
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "fabric={} availability={} auth={} effects={} evidence={evidence}",
        descriptor.id,
        descriptor.availability.label(),
        descriptor.auth_posture.label(),
        descriptor.side_effect_posture.label(),
    )
}

fn format_workspace_action_capability(
    capability: &WorkspaceActionCapability,
    collaboration: &CollaborationModeResult,
) -> String {
    if capability.mutating && !collaboration.active.mutation_posture.allows_mutation() {
        return format!(
            "workspace action {}: blocked by {} mode read-only boundary \u{2014} {}",
            capability.action,
            collaboration.active.mode.label(),
            capability.summary
        );
    }

    let posture = if capability.mutating {
        "mutating"
    } else {
        "read-only"
    };
    format!(
        "workspace action {}: available ({posture}) \u{2014} {}",
        capability.action, capability.summary
    )
}

fn format_workspace_tool_capability(tool: &WorkspaceToolCapability) -> String {
    match tool.suggested_probe.as_ref() {
        Some(action) => format!(
            "workspace tool observation {}: {} \u{2014} re-probe via {}",
            tool.tool,
            tool.summary,
            action.summary()
        ),
        None => format!("workspace tool observation {}: {}", tool.tool, tool.summary),
    }
}

fn format_execution_hand_capability_line(diagnostic: &ExecutionHandDiagnostic) -> String {
    let operations = diagnostic
        .supported_operations
        .iter()
        .map(|operation| operation.label())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "execution hand {}: {}, authority={}, operations=[{}] \u{2014} {}",
        diagnostic.hand.label(),
        diagnostic.phase.label(),
        diagnostic.authority.label(),
        operations,
        diagnostic.summary
    )
}

fn retrieval_capability_lines(gatherer: Option<&dyn ContextGatherer>) -> Vec<String> {
    let Some(gatherer) = gatherer else {
        return vec![
            "search/refine via bm25: unavailable \u{2014} no gatherer is configured".to_string(),
            "search/refine via vector: unavailable \u{2014} no gatherer is configured".to_string(),
        ];
    };

    let lexical = gatherer.capability_for_planning(
        &PlannerConfig::default().with_retrieval_strategy(RetrievalStrategy::Lexical),
    );
    let vector = gatherer.capability_for_planning(
        &PlannerConfig::default().with_retrieval_strategy(RetrievalStrategy::Vector),
    );

    vec![
        format!(
            "search/refine via bm25: {}",
            format_gatherer_capability(&lexical)
        ),
        format!(
            "search/refine via vector: {}",
            format_gatherer_capability(&vector)
        ),
    ]
}

fn grounding_domain_label(domain: GroundingDomain) -> &'static str {
    match domain {
        GroundingDomain::Repository => "repository",
        GroundingDomain::External => "external",
        GroundingDomain::Mixed => "mixed repository and external",
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::model::{CollaborationMode, InstructionFrame};
    use crate::domain::ports::{
        GroundingDomain, GroundingRequirement, WorkspaceActionCapability,
        WorkspaceCapabilitySurface, WorkspaceToolCapability,
    };

    #[test]
    fn execution_contract_discloses_planner_capabilities_and_constraints() {
        let service = super::ExecutionContractService::new();
        let collaboration = crate::domain::model::CollaborationModeResult::defaulted(
            CollaborationMode::Planning.state(),
            "planning test",
        );
        let instruction_frame = InstructionFrame::for_edit(vec!["src/lib.rs".to_string()]);
        let grounding = GroundingRequirement {
            domain: GroundingDomain::External,
            reason: Some("latest upstream behavior is required".to_string()),
        };
        let surface = WorkspaceCapabilitySurface {
            actions: vec![
                WorkspaceActionCapability::new(
                    "inspect",
                    "run a single read-only shell probe through the terminal hand",
                    false,
                ),
                WorkspaceActionCapability::new(
                    "shell",
                    "run a governed workspace command when a command should execute now",
                    true,
                ),
            ],
            tools: vec![WorkspaceToolCapability::new(
                "just",
                "observed available from prior shell command `just --version`",
                None,
            )],
            notes: vec!["local workspace only".to_string()],
        };
        let external_capabilities = crate::domain::model::default_external_capability_descriptors();

        let contract = service.build(super::ExecutionContractContext {
            workspace_capability_surface: &surface,
            execution_hands: &[],
            governance_profile: None,
            external_capabilities: &external_capabilities,
            gatherer: None,
            collaboration: &collaboration,
            instruction_frame: Some(&instruction_frame),
            grounding: Some(&grounding),
        });

        assert!(contract.capability_manifest.iter().any(|line| {
            line.contains("workspace action inspect: available (read-only)")
                && line.contains("run a single read-only shell probe through the terminal hand")
        }));
        assert!(contract.capability_manifest.iter().any(|line| {
            line.contains("workspace action shell: blocked by planning mode read-only boundary")
        }));
        assert!(
            contract.capability_manifest.iter().any(|line| {
                line.contains("workspace tool observation just: observed available")
            })
        );
        assert!(
            contract
                .capability_manifest
                .contains(&"workspace note: local workspace only".to_string())
        );
        assert!(contract.capability_manifest.iter().any(|line| {
            line.contains("search/refine via bm25: unavailable")
                && line.contains("no gatherer is configured")
        }));
        assert!(
            contract
                .capability_manifest
                .iter()
                .any(|line| { line.contains("external capability web.search: fabric=web.search") })
        );
        assert!(contract.capability_manifest.iter().any(|line| {
            line.contains(
                "execution governance: unavailable; mutating or networked actions may fail closed",
            )
        }));

        assert!(
            contract
                .completion_contract
                .iter()
                .any(|line| { line.contains("Planning mode is read-only") })
        );
        assert!(contract.completion_contract.iter().any(|line| {
            line.contains("bounded single-step probe such as `inspect` `command -v <tool>`")
        }));
        assert!(
            contract
                .completion_contract
                .iter()
                .any(|line| { line.contains("Current candidate files: src/lib.rs") })
        );
        assert!(contract.completion_contract.iter().any(|line| {
            line.contains("external evidence") && line.contains("latest upstream behavior")
        }));
    }
}
