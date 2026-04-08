use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, path::PathBuf};

#[async_trait]
pub trait EntityResolver: Send + Sync + Debug {
    async fn resolve(
        &self,
        request: &EntityResolutionRequest,
    ) -> anyhow::Result<EntityResolutionOutcome>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityResolutionRequest {
    pub workspace_root: PathBuf,
    pub raw_hint: String,
    pub hints: Vec<NormalizedEntityHint>,
    #[serde(default)]
    pub likely_targets: Vec<String>,
}

impl EntityResolutionRequest {
    pub fn new(
        workspace_root: impl Into<PathBuf>,
        raw_hint: impl Into<String>,
        hints: Vec<NormalizedEntityHint>,
    ) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            raw_hint: raw_hint.into(),
            hints,
            likely_targets: Vec::new(),
        }
    }

    pub fn with_likely_targets(mut self, likely_targets: Vec<String>) -> Self {
        self.likely_targets = likely_targets;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityLookupMode {
    ExactPath,
    Basename,
    PathFragment,
    SymbolFragment,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedEntityHint {
    pub mode: EntityLookupMode,
    pub value: String,
}

impl NormalizedEntityHint {
    pub fn new(mode: EntityLookupMode, value: impl Into<String>) -> Self {
        Self {
            mode,
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityResolutionCandidate {
    pub path: String,
    pub matched_by: EntityLookupMode,
    pub rank: usize,
}

impl EntityResolutionCandidate {
    pub fn new(path: impl Into<String>, matched_by: EntityLookupMode, rank: usize) -> Self {
        Self {
            path: path.into(),
            matched_by,
            rank,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum EntityResolutionOutcome {
    Resolved {
        target: EntityResolutionCandidate,
        #[serde(default)]
        alternatives: Vec<EntityResolutionCandidate>,
        explanation: String,
    },
    Ambiguous {
        candidates: Vec<EntityResolutionCandidate>,
        explanation: String,
    },
    Missing {
        attempted_hints: Vec<NormalizedEntityHint>,
        explanation: String,
    },
}

impl EntityResolutionOutcome {
    pub fn status_label(&self) -> &'static str {
        match self {
            Self::Resolved { .. } => "resolved",
            Self::Ambiguous { .. } => "ambiguous",
            Self::Missing { .. } => "missing",
        }
    }

    pub fn resolved_path(&self) -> Option<&str> {
        match self {
            Self::Resolved { target, .. } => Some(target.path.as_str()),
            Self::Ambiguous { .. } | Self::Missing { .. } => None,
        }
    }

    pub fn candidate_paths(&self) -> Vec<String> {
        match self {
            Self::Resolved {
                target,
                alternatives,
                ..
            } => std::iter::once(target.path.clone())
                .chain(alternatives.iter().map(|candidate| candidate.path.clone()))
                .collect(),
            Self::Ambiguous { candidates, .. } => candidates
                .iter()
                .map(|candidate| candidate.path.clone())
                .collect(),
            Self::Missing { .. } => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EntityLookupMode, EntityResolutionCandidate, EntityResolutionOutcome,
        EntityResolutionRequest, NormalizedEntityHint,
    };
    use std::path::PathBuf;

    #[test]
    fn entity_resolver_contracts_define_explicit_outcomes_and_ranked_candidates() {
        let request = EntityResolutionRequest::new(
            PathBuf::from("/workspace"),
            "ManifoldVisualization",
            vec![
                NormalizedEntityHint::new(EntityLookupMode::Basename, "manifoldvisualization"),
                NormalizedEntityHint::new(EntityLookupMode::PathFragment, "components/manifold"),
            ],
        )
        .with_likely_targets(vec!["apps/web/src/runtime-app.tsx".to_string()]);
        assert_eq!(request.raw_hint, "ManifoldVisualization");
        assert_eq!(request.hints.len(), 2);
        assert_eq!(request.likely_targets, vec!["apps/web/src/runtime-app.tsx"]);

        let resolved = EntityResolutionOutcome::Resolved {
            target: EntityResolutionCandidate::new(
                "apps/web/src/components/ManifoldVisualization.tsx",
                EntityLookupMode::Basename,
                1,
            ),
            alternatives: vec![EntityResolutionCandidate::new(
                "apps/web/src/components/Manifold.tsx",
                EntityLookupMode::PathFragment,
                2,
            )],
            explanation: "basename match won after authored-path ranking".to_string(),
        };
        assert_eq!(
            resolved.resolved_path(),
            Some("apps/web/src/components/ManifoldVisualization.tsx")
        );
        assert_eq!(
            resolved.candidate_paths(),
            vec![
                "apps/web/src/components/ManifoldVisualization.tsx".to_string(),
                "apps/web/src/components/Manifold.tsx".to_string(),
            ]
        );

        let ambiguous = EntityResolutionOutcome::Ambiguous {
            candidates: vec![
                EntityResolutionCandidate::new(
                    "apps/web/src/components/ManifoldVisualization.tsx",
                    EntityLookupMode::Basename,
                    1,
                ),
                EntityResolutionCandidate::new(
                    "apps/docs/src/components/ManifoldVisualization.tsx",
                    EntityLookupMode::Basename,
                    2,
                ),
            ],
            explanation: "two authored basename matches remained tied".to_string(),
        };
        assert_eq!(ambiguous.status_label(), "ambiguous");
        assert_eq!(ambiguous.candidate_paths().len(), 2);

        let missing = EntityResolutionOutcome::Missing {
            attempted_hints: request.hints.clone(),
            explanation: "no authored file matched the normalized hints".to_string(),
        };
        assert_eq!(missing.status_label(), "missing");
        assert_eq!(missing.candidate_paths(), Vec::<String>::new());
    }
}
