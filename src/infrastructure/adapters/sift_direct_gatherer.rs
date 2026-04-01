use crate::domain::ports::{
    ContextGatherRequest, ContextGatherResult, ContextGatherer, EvidenceBundle, EvidenceItem,
    GathererCapability, PlannerDecision, PlannerTraceMetadata, PlannerTraceStep, RetainedEvidence,
};
use anyhow::Result;
use async_trait::async_trait;
use sift::{EnvironmentFactInput, LocalContextSource, SearchInput, SearchOptions, Sift};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};

pub struct SiftDirectGathererAdapter {
    workspace_root: PathBuf,
    sift: Sift,
    verbose: AtomicU8,
}

impl SiftDirectGathererAdapter {
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let workspace_root = workspace_root.into();
        Self {
            workspace_root: workspace_root.clone(),
            sift: Sift::builder()
                .with_cache_dir(cache_dir_for_sift(&workspace_root))
                .build(),
            verbose: AtomicU8::new(0),
        }
    }

    pub fn set_verbose(&self, level: u8) {
        self.verbose.store(level, Ordering::Relaxed);
    }

    fn search_workspace(&self, request: &ContextGatherRequest) -> Result<sift::SearchResponse> {
        let local_context = request
            .prior_context
            .iter()
            .enumerate()
            .map(|(index, value)| {
                LocalContextSource::EnvironmentFact(EnvironmentFactInput::new(
                    format!("prior_context_{index}"),
                    value.clone(),
                ))
            })
            .collect::<Vec<_>>();
        let options = SearchOptions::default()
            .with_strategy(request.planning.retrieval_strategy.label())
            .with_limit(request.budget.max_items)
            .with_shortlist(request.budget.max_items)
            .with_local_context(local_context)
            .with_verbose(self.verbose.load(Ordering::Relaxed));
        let input = SearchInput::new(&self.workspace_root, request.user_query.clone())
            .with_intent(request.intent_reason.clone())
            .with_options(options);
        self.sift.search(input)
    }
}

#[async_trait]
impl ContextGatherer for SiftDirectGathererAdapter {
    fn capability(&self) -> GathererCapability {
        GathererCapability::Available
    }

    async fn gather_context(
        &self,
        request: &ContextGatherRequest,
    ) -> Result<ContextGatherResult, anyhow::Error> {
        let response = self.search_workspace(request)?;
        if self.verbose.load(Ordering::Relaxed) >= 1 {
            println!(
                "[LANE] Direct sift gatherer retrieved {} hit(s) for `{}`.",
                response.hits.len(),
                request.user_query,
            );
        }

        let items = response
            .hits
            .iter()
            .take(request.budget.max_items)
            .enumerate()
            .map(|(index, hit)| EvidenceItem {
                source: hit.path.clone(),
                snippet: trim_for_budget(&hit.snippet, request.budget.max_snippet_chars),
                rationale: format!("retrieved directly for `{}`", request.user_query),
                rank: index + 1,
            })
            .collect::<Vec<_>>();

        let summary = if items.is_empty() {
            format!(
                "Direct sift retrieval found no matching evidence for `{}` in the current workspace.",
                request.user_query
            )
        } else {
            format!(
                "Direct sift retrieval gathered {} evidence item(s) for `{}` using `{}` strategy.",
                items.len(),
                request.user_query,
                request.planning.retrieval_strategy.label(),
            )
        };

        let planner = PlannerTraceMetadata {
            mode: request.planning.mode,
            strategy: request.planning.planner_strategy.clone(),
            profile: None,
            session_id: None,
            completed: true,
            stop_reason: Some("direct-retrieval".to_string()),
            turn_count: 1,
            steps: vec![PlannerTraceStep {
                step_id: "direct-search-step".to_string(),
                sequence: 1,
                parent_step_id: None,
                decisions: vec![PlannerDecision {
                    action: "retrieve".to_string(),
                    query: Some(request.user_query.clone()),
                    rationale: Some(format!(
                        "executed direct sift {} retrieval without nested autonomous planning",
                        request.planning.retrieval_strategy.label(),
                    )),
                    next_step_id: None,
                    turn_id: None,
                    branch_id: None,
                    node_id: None,
                    target_branch_id: None,
                    target_node_id: None,
                    edge_id: None,
                    edge_kind: None,
                    frontier_id: None,
                    stop_reason: Some("direct-retrieval".to_string()),
                }],
            }],
            retained_artifacts: items
                .iter()
                .map(|item| RetainedEvidence {
                    source: item.source.clone(),
                    snippet: Some(item.snippet.clone()),
                    rationale: Some(item.rationale.clone()),
                    locator: None,
                })
                .collect(),
            graph_episode: None,
            trace_artifact_ref: None,
        };

        Ok(ContextGatherResult::available(
            EvidenceBundle::new(summary, items).with_planner(planner),
        ))
    }
}

fn cache_dir_for_sift(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".sift").join("cache")
}

fn trim_for_budget(input: &str, limit: usize) -> String {
    let cleaned = strip_ansi_sequences(input);
    let input = cleaned.as_str();
    if input.chars().count() <= limit {
        return input.to_string();
    }

    let kept = input.chars().take(limit).collect::<String>();
    format!("{kept}...[truncated]")
}

fn strip_ansi_sequences(input: &str) -> String {
    let mut cleaned = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && matches!(chars.peek(), Some('[')) {
            chars.next();
            for next in chars.by_ref() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
            continue;
        }
        cleaned.push(ch);
    }
    cleaned
}

#[cfg(test)]
mod tests {
    use super::SiftDirectGathererAdapter;
    use crate::domain::ports::{
        ContextGatherRequest, ContextGatherer, EvidenceBudget, GathererCapability, PlannerConfig,
        RetrievalMode, RetrievalStrategy,
    };
    use tempfile::tempdir;

    #[tokio::test]
    async fn direct_gatherer_returns_direct_retrieval_metadata_and_evidence() {
        let workspace = tempdir().expect("workspace");
        std::fs::write(
            workspace.path().join("alpha.txt"),
            "alpha runtime details for the direct gatherer adapter",
        )
        .expect("write alpha");

        let adapter = SiftDirectGathererAdapter::new(workspace.path());
        let result = adapter
            .gather_context(&ContextGatherRequest::new(
                "find alpha runtime details",
                workspace.path(),
                "repo investigation",
                EvidenceBudget::default(),
            ))
            .await
            .expect("gather result");

        assert_eq!(adapter.capability(), GathererCapability::Available);
        let bundle = result.evidence_bundle.expect("bundle");
        let planner = bundle.planner.expect("planner metadata");
        assert_eq!(planner.mode, RetrievalMode::Linear);
        assert_eq!(planner.stop_reason.as_deref(), Some("direct-retrieval"));
        assert_eq!(planner.turn_count, 1);
        assert!(!bundle.items.is_empty());
        assert!(bundle.summary.contains("Direct sift retrieval gathered"));
    }

    #[tokio::test]
    async fn direct_gatherer_respects_budget_and_requested_mode_metadata() {
        let workspace = tempdir().expect("workspace");
        std::fs::write(
            workspace.path().join("alpha.txt"),
            "alpha runtime details for the direct gatherer adapter",
        )
        .expect("write alpha");
        std::fs::write(
            workspace.path().join("beta.txt"),
            "beta runtime details for the direct gatherer adapter",
        )
        .expect("write beta");

        let adapter = SiftDirectGathererAdapter::new(workspace.path());
        let request = ContextGatherRequest::new(
            "runtime details",
            workspace.path(),
            "repo investigation",
            EvidenceBudget {
                max_items: 1,
                ..EvidenceBudget::default()
            },
        )
        .with_planning(
            PlannerConfig::default()
                .with_mode(RetrievalMode::Graph)
                .with_retrieval_strategy(RetrievalStrategy::Hybrid),
        )
        .with_prior_context(vec!["Prefer runtime-related files first.".to_string()]);

        let result = adapter
            .gather_context(&request)
            .await
            .expect("gather result");
        let bundle = result.evidence_bundle.expect("bundle");
        let planner = bundle.planner.expect("planner metadata");

        assert_eq!(bundle.items.len(), 1);
        assert_eq!(planner.mode, RetrievalMode::Graph);
        assert_eq!(
            planner.steps[0].decisions[0].query.as_deref(),
            Some("runtime details")
        );
    }
}
