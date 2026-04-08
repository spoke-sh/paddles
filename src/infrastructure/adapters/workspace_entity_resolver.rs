use crate::domain::ports::{
    EntityLookupMode, EntityResolutionCandidate, EntityResolutionOutcome, EntityResolutionRequest,
    EntityResolver, NormalizedEntityHint,
};
use crate::infrastructure::workspace_entity_index::{WorkspaceEntityEntry, WorkspaceEntityIndex};
use async_trait::async_trait;

#[derive(Debug, Default)]
pub struct WorkspaceEntityResolver;

#[async_trait]
impl EntityResolver for WorkspaceEntityResolver {
    async fn resolve(
        &self,
        request: &EntityResolutionRequest,
    ) -> anyhow::Result<EntityResolutionOutcome> {
        let inventory = WorkspaceEntityIndex::new(&request.workspace_root).load_or_build()?;
        let hints = effective_hints(request);
        if hints.is_empty() {
            return Ok(EntityResolutionOutcome::Missing {
                attempted_hints: Vec::new(),
                explanation: "no deterministic entity hints were available".to_string(),
            });
        }

        let mut ranked_candidates = inventory
            .entries
            .iter()
            .filter_map(|entry| score_entry(entry, &hints, &request.likely_targets))
            .collect::<Vec<_>>();
        ranked_candidates.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.path.cmp(&right.path))
        });

        if ranked_candidates.is_empty() {
            return Ok(EntityResolutionOutcome::Missing {
                attempted_hints: hints.clone(),
                explanation: format!(
                    "no authored workspace file matched {} normalized hint(s)",
                    hints.len()
                ),
            });
        }

        let top_score = ranked_candidates[0].score;
        let ambiguous_candidates = ranked_candidates
            .iter()
            .take_while(|candidate| candidate.score == top_score)
            .cloned()
            .collect::<Vec<_>>();
        if ambiguous_candidates.len() > 1 {
            return Ok(EntityResolutionOutcome::Ambiguous {
                candidates: ambiguous_candidates
                    .into_iter()
                    .enumerate()
                    .map(|(index, candidate)| candidate.into_resolution_candidate(index + 1))
                    .collect(),
                explanation: format!(
                    "{} authored files remained tied after deterministic ranking",
                    ranked_candidates
                        .iter()
                        .take_while(|candidate| candidate.score == top_score)
                        .count()
                ),
            });
        }

        Ok(EntityResolutionOutcome::Resolved {
            target: ranked_candidates[0].clone().into_resolution_candidate(1),
            alternatives: ranked_candidates
                .into_iter()
                .skip(1)
                .enumerate()
                .map(|(index, candidate)| candidate.into_resolution_candidate(index + 2))
                .collect(),
            explanation: "deterministic ranking selected a single authored target".to_string(),
        })
    }
}

impl WorkspaceEntityResolver {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Clone, Debug)]
struct RankedEntityCandidate {
    path: String,
    matched_by: EntityLookupMode,
    score: usize,
}

impl RankedEntityCandidate {
    fn into_resolution_candidate(self, rank: usize) -> EntityResolutionCandidate {
        EntityResolutionCandidate::new(self.path, self.matched_by, rank)
    }
}

fn effective_hints(request: &EntityResolutionRequest) -> Vec<NormalizedEntityHint> {
    if !request.hints.is_empty() {
        return request.hints.clone();
    }

    let raw = request.raw_hint.trim();
    if raw.is_empty() {
        return Vec::new();
    }

    let mut hints = Vec::new();
    if raw.contains('/') || raw.contains('\\') {
        hints.push(NormalizedEntityHint::new(
            EntityLookupMode::ExactPath,
            normalize_path(raw),
        ));
    } else {
        hints.push(NormalizedEntityHint::new(
            EntityLookupMode::Basename,
            raw.to_ascii_lowercase(),
        ));
        hints.push(NormalizedEntityHint::new(
            EntityLookupMode::SymbolFragment,
            raw.to_ascii_lowercase(),
        ));
    }
    hints
}

fn score_entry(
    entry: &WorkspaceEntityEntry,
    hints: &[NormalizedEntityHint],
    likely_targets: &[String],
) -> Option<RankedEntityCandidate> {
    let mut best_match = None;
    for hint in hints {
        let hint_value = match hint.mode {
            EntityLookupMode::ExactPath | EntityLookupMode::PathFragment => {
                normalize_path(&hint.value)
            }
            EntityLookupMode::Basename | EntityLookupMode::SymbolFragment => {
                hint.value.to_ascii_lowercase()
            }
        };
        let base_score = score_hint_against_entry(entry, hint.mode, &hint_value)?;
        let likely_target_bonus = likely_targets
            .iter()
            .position(|path| path == &entry.path)
            .map(|index| 12usize.saturating_sub(index * 3))
            .unwrap_or(0);
        let candidate = RankedEntityCandidate {
            path: entry.path.clone(),
            matched_by: hint.mode,
            score: base_score + likely_target_bonus,
        };
        if best_match
            .as_ref()
            .is_none_or(|current: &RankedEntityCandidate| candidate.score > current.score)
        {
            best_match = Some(candidate);
        }
    }
    best_match
}

fn score_hint_against_entry(
    entry: &WorkspaceEntityEntry,
    mode: EntityLookupMode,
    hint: &str,
) -> Option<usize> {
    let normalized_path = normalize_path(&entry.path);
    let basename = entry.basename.to_ascii_lowercase();
    let stem = entry.stem.to_ascii_lowercase();

    match mode {
        EntityLookupMode::ExactPath => (normalized_path == hint).then_some(400),
        EntityLookupMode::Basename => {
            if basename == hint || stem == hint {
                Some(320)
            } else if entry
                .components
                .iter()
                .any(|component| component.eq_ignore_ascii_case(hint))
            {
                Some(280)
            } else {
                None
            }
        }
        EntityLookupMode::PathFragment => normalized_path.contains(hint).then_some(260),
        EntityLookupMode::SymbolFragment => {
            let hint_tokens = tokenize_identifier(hint);
            if hint_tokens.is_empty() {
                return None;
            }

            let stem_tokens = tokenize_identifier(&stem);
            let path_tokens = entry
                .components
                .iter()
                .flat_map(|component| tokenize_identifier(component))
                .collect::<Vec<_>>();
            if tokens_match_in_order(&stem_tokens, &hint_tokens) {
                Some(300)
            } else if tokens_match_in_order(&path_tokens, &hint_tokens) {
                Some(240)
            } else {
                None
            }
        }
    }
}

fn normalize_path(value: &str) -> String {
    value.replace('\\', "/").to_ascii_lowercase()
}

fn tokenize_identifier(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut previous_was_lower = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            let is_upper = ch.is_ascii_uppercase();
            if is_upper && previous_was_lower && !current.is_empty() {
                tokens.push(current.to_ascii_lowercase());
                current.clear();
            }
            current.push(ch.to_ascii_lowercase());
            previous_was_lower = ch.is_ascii_lowercase();
        } else if !current.is_empty() {
            tokens.push(current.to_ascii_lowercase());
            current.clear();
            previous_was_lower = false;
        } else {
            previous_was_lower = false;
        }
    }
    if !current.is_empty() {
        tokens.push(current.to_ascii_lowercase());
    }
    tokens
}

fn tokens_match_in_order(haystack: &[String], needle: &[String]) -> bool {
    if needle.is_empty() {
        return false;
    }

    let mut position = 0usize;
    for token in needle {
        let Some(offset) = haystack[position..]
            .iter()
            .position(|candidate| candidate == token)
        else {
            return false;
        };
        position += offset + 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::WorkspaceEntityResolver;
    use crate::domain::ports::{
        EntityLookupMode, EntityResolutionOutcome, EntityResolutionRequest, EntityResolver,
        NormalizedEntityHint,
    };
    use std::fs;

    #[tokio::test]
    async fn resolver_supports_exact_path_basename_and_symbol_hints() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("apps/web/src/components"))
            .expect("create web components");
        fs::create_dir_all(workspace.path().join("src/domain/model")).expect("create model dir");
        fs::create_dir_all(workspace.path().join("apps/docs/.docusaurus"))
            .expect("create generated dir");
        fs::write(
            workspace.path().join(".gitignore"),
            "/apps/docs/.docusaurus/\n",
        )
        .expect("write gitignore");
        fs::write(
            workspace
                .path()
                .join("apps/web/src/components/ManifoldVisualization.tsx"),
            "export function ManifoldVisualization() { return null; }\n",
        )
        .expect("write manifold component");
        fs::write(
            workspace.path().join("src/domain/model/turns.rs"),
            "pub struct Turn;\n",
        )
        .expect("write turns model");
        fs::write(
            workspace
                .path()
                .join("apps/docs/.docusaurus/ManifoldVisualization.tsx"),
            "export default {};",
        )
        .expect("write generated component");

        let resolver = WorkspaceEntityResolver::new();

        let exact = resolver
            .resolve(&EntityResolutionRequest::new(
                workspace.path(),
                "apps/web/src/components/ManifoldVisualization.tsx",
                vec![NormalizedEntityHint::new(
                    EntityLookupMode::ExactPath,
                    "apps/web/src/components/ManifoldVisualization.tsx",
                )],
            ))
            .await
            .expect("resolve exact path");
        assert_eq!(
            exact.resolved_path(),
            Some("apps/web/src/components/ManifoldVisualization.tsx")
        );

        let basename = resolver
            .resolve(&EntityResolutionRequest::new(
                workspace.path(),
                "turns",
                vec![NormalizedEntityHint::new(
                    EntityLookupMode::Basename,
                    "turns",
                )],
            ))
            .await
            .expect("resolve basename");
        assert_eq!(basename.resolved_path(), Some("src/domain/model/turns.rs"));

        let symbol = resolver
            .resolve(&EntityResolutionRequest::new(
                workspace.path(),
                "ManifoldVisualization",
                vec![NormalizedEntityHint::new(
                    EntityLookupMode::SymbolFragment,
                    "ManifoldVisualization",
                )],
            ))
            .await
            .expect("resolve symbol fragment");
        assert!(matches!(symbol, EntityResolutionOutcome::Resolved { .. }));
        assert_eq!(
            symbol.resolved_path(),
            Some("apps/web/src/components/ManifoldVisualization.tsx")
        );
    }
}
