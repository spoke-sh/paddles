use crate::domain::ports::{
    ContextGatherRequest, ContextGatherResult, ContextGatherer, EvidenceBundle, EvidenceItem,
    GathererCapability, PlannerDecision, PlannerStrategyKind, PlannerTraceMetadata,
    PlannerTraceStep, RetainedEvidence,
};
use anyhow::Result;
use async_trait::async_trait;
use sift::{
    AutonomousPlannerAction, AutonomousPlannerStopReason, AutonomousPlannerStrategy,
    AutonomousPlannerStrategyKind, AutonomousSearchRequest, AutonomousSearchResponse,
    EnvironmentFactInput, LocalContextSource, SearchEmission, SearchPlan, Sift,
};
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};

pub struct SiftAutonomousGathererAdapter {
    workspace_root: PathBuf,
    sift: Sift,
    verbose: AtomicU8,
    planner_profile: Option<String>,
}

impl SiftAutonomousGathererAdapter {
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            sift: Sift::builder().build(),
            verbose: AtomicU8::new(0),
            planner_profile: None,
        }
    }

    pub fn set_verbose(&self, level: u8) {
        self.verbose.store(level, Ordering::Relaxed);
    }

    pub fn with_model_driven_profile(mut self, profile: impl Into<String>) -> Self {
        self.planner_profile = Some(profile.into());
        self
    }

    fn planner_strategy(&self, request: &ContextGatherRequest) -> AutonomousPlannerStrategy {
        let profile = request
            .planning
            .profile
            .as_ref()
            .or(self.planner_profile.as_ref());
        match request.planning.strategy {
            PlannerStrategyKind::Heuristic => AutonomousPlannerStrategy::heuristic(),
            PlannerStrategyKind::ModelDriven => match profile {
                Some(profile) => AutonomousPlannerStrategy::model_driven().with_profile(profile),
                None => AutonomousPlannerStrategy::model_driven(),
            },
        }
    }

    fn search_autonomous(
        &self,
        request: &ContextGatherRequest,
    ) -> Result<AutonomousSearchResponse> {
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

        self.sift.search_autonomous(
            AutonomousSearchRequest::new(&self.workspace_root, &request.user_query)
                .with_plan(SearchPlan::default_lexical())
                .with_intent(request.intent_reason.clone())
                .with_planner_strategy(self.planner_strategy(request))
                .with_step_limit(request.planning.step_limit)
                .with_limit(request.budget.max_items)
                .with_shortlist(request.budget.max_items)
                .with_retained_artifact_limit(request.planning.retained_artifact_limit)
                .with_local_context(local_context)
                .with_verbose(self.verbose.load(Ordering::Relaxed)),
        )
    }
}

#[async_trait]
impl ContextGatherer for SiftAutonomousGathererAdapter {
    fn capability(&self) -> GathererCapability {
        GathererCapability::Available
    }

    async fn gather_context(
        &self,
        request: &ContextGatherRequest,
    ) -> Result<ContextGatherResult, anyhow::Error> {
        let response = self.search_autonomous(request)?;
        if self.verbose.load(Ordering::Relaxed) >= 1 {
            println!(
                "[LANE] Autonomous gatherer executed {} planner turn(s) for `{}`.",
                response.turns.len(),
                request.user_query,
            );
        }

        let (items, mut warnings) = collect_evidence_items(&response, request);
        if items.is_empty() {
            warnings.push(format!(
                "Autonomous gatherer returned no retained evidence for `{}`.",
                request.user_query
            ));
        }

        let planner = planner_metadata_from_response(&response);
        let summary = format_autonomous_summary(request, &planner, items.len());
        let mut bundle = EvidenceBundle::new(summary, items).with_planner(planner);
        if !warnings.is_empty() {
            bundle = bundle.with_warnings(warnings);
        }

        Ok(ContextGatherResult::available(bundle))
    }
}

fn collect_evidence_items(
    response: &AutonomousSearchResponse,
    request: &ContextGatherRequest,
) -> (Vec<EvidenceItem>, Vec<String>) {
    let retained = response
        .state
        .retained_artifacts
        .iter()
        .take(request.budget.max_items)
        .enumerate()
        .map(|(index, artifact)| EvidenceItem {
            source: artifact.path.clone(),
            snippet: artifact
                .snippet
                .as_deref()
                .map(|value| trim_for_budget(value, request.budget.max_snippet_chars))
                .unwrap_or_else(|| {
                    artifact
                        .location
                        .clone()
                        .unwrap_or_else(|| "No retained snippet available.".to_string())
                }),
            rationale: artifact
                .rationale
                .clone()
                .unwrap_or_else(|| "retained by the autonomous planner".to_string()),
            rank: index + 1,
        })
        .collect::<Vec<_>>();
    if !retained.is_empty() {
        return (prioritize_evidence_items(retained), Vec::new());
    }

    let Some(view) = response
        .turns
        .iter()
        .rev()
        .find_map(|turn| match &turn.emission {
            SearchEmission::View(view) => Some(view),
            _ => None,
        })
    else {
        return (
            Vec::new(),
            vec!["No autonomous view emission was available.".to_string()],
        );
    };

    let mut seen = HashSet::new();
    let items = view
        .hits
        .iter()
        .filter(|hit| seen.insert((hit.path.as_str(), hit.rank)))
        .take(request.budget.max_items)
        .enumerate()
        .map(|(index, hit)| EvidenceItem {
            source: hit.path.clone(),
            snippet: trim_for_budget(&hit.snippet, request.budget.max_snippet_chars),
            rationale: format!("retrieved during autonomous step {}", index + 1),
            rank: index + 1,
        })
        .collect::<Vec<_>>();

    (
        prioritize_evidence_items(items),
        vec!["Autonomous gatherer fell back to last-turn hits because no retained artifacts were available.".to_string()],
    )
}

fn planner_metadata_from_response(response: &AutonomousSearchResponse) -> PlannerTraceMetadata {
    PlannerTraceMetadata {
        strategy: map_planner_strategy_kind(response.planner_strategy.kind),
        profile: response.planner_strategy.profile.clone(),
        completed: response.state.completed,
        stop_reason: response.planner_trace.stop_reason.map(format_stop_reason),
        turn_count: response.turns.len(),
        steps: response
            .planner_trace
            .steps
            .iter()
            .map(|step| PlannerTraceStep {
                step_id: step.step.step_id.clone(),
                sequence: step.step.sequence,
                parent_step_id: step.step.parent_step_id.clone(),
                decisions: step
                    .decisions
                    .iter()
                    .map(|decision| PlannerDecision {
                        action: format_action(decision.action),
                        query: decision.query.clone(),
                        rationale: decision.rationale.clone(),
                        next_step_id: decision.next_step.as_ref().map(|step| step.step_id.clone()),
                        turn_id: decision.turn_id.clone(),
                        stop_reason: decision.stop_reason.map(format_stop_reason),
                    })
                    .collect(),
            })
            .collect(),
        retained_artifacts: response
            .state
            .retained_artifacts
            .iter()
            .map(|artifact| RetainedEvidence {
                source: artifact.path.clone(),
                snippet: artifact.snippet.clone(),
                rationale: artifact.rationale.clone(),
            })
            .collect(),
    }
}

fn format_autonomous_summary(
    request: &ContextGatherRequest,
    planner: &PlannerTraceMetadata,
    item_count: usize,
) -> String {
    let strategy = match planner.strategy {
        PlannerStrategyKind::Heuristic => "heuristic",
        PlannerStrategyKind::ModelDriven => "model-driven",
    };
    let stop_reason = planner
        .stop_reason
        .as_deref()
        .unwrap_or("planner still in progress");
    format!(
        "Autonomous `{strategy}` gatherer collected {item_count} evidence item(s) for `{}` across {} turn(s); stop reason: {stop_reason}.",
        request.user_query, planner.turn_count,
    )
}

fn map_planner_strategy_kind(kind: AutonomousPlannerStrategyKind) -> PlannerStrategyKind {
    match kind {
        AutonomousPlannerStrategyKind::Heuristic => PlannerStrategyKind::Heuristic,
        AutonomousPlannerStrategyKind::ModelDriven => PlannerStrategyKind::ModelDriven,
    }
}

fn prioritize_evidence_items(items: Vec<EvidenceItem>) -> Vec<EvidenceItem> {
    let has_non_keel = items.iter().any(|item| !is_keel_path(&item.source));
    let has_non_test_snippet = items.iter().any(|item| !is_test_snippet(&item.snippet));
    let mut prioritized = items
        .into_iter()
        .filter(|item| !has_non_keel || !is_keel_path(&item.source))
        .filter(|item| !has_non_test_snippet || !is_test_snippet(&item.snippet))
        .collect::<Vec<_>>();
    prioritized.sort_by_key(|item| {
        (
            evidence_priority(&item.source),
            snippet_noise_priority(&item.snippet),
            item.rank,
        )
    });
    for (index, item) in prioritized.iter_mut().enumerate() {
        item.rank = index + 1;
    }
    prioritized
}

fn evidence_priority(source: &str) -> usize {
    if is_src_path(source) {
        0
    } else if is_top_level_doc(source) {
        1
    } else if is_keel_path(source) {
        3
    } else {
        2
    }
}

fn is_src_path(source: &str) -> bool {
    source.starts_with("src/") || source.contains("/src/")
}

fn is_top_level_doc(source: &str) -> bool {
    Path::new(source)
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                "README.md"
                    | "ARCHITECTURE.md"
                    | "CONFIGURATION.md"
                    | "POLICY.md"
                    | "AGENTS.md"
                    | "INSTRUCTIONS.md"
                    | "PROTOCOL.md"
            )
        })
}

fn is_keel_path(source: &str) -> bool {
    source.starts_with(".keel/") || source.contains("/.keel/")
}

fn snippet_noise_priority(snippet: &str) -> usize {
    if is_test_snippet(snippet) { 1 } else { 0 }
}

fn is_test_snippet(snippet: &str) -> bool {
    snippet.contains("#[cfg(test)]") || snippet.contains("mod tests")
}

fn format_action(action: AutonomousPlannerAction) -> String {
    match action {
        AutonomousPlannerAction::Search => "search".to_string(),
        AutonomousPlannerAction::Continue => "continue".to_string(),
        AutonomousPlannerAction::Terminate => "terminate".to_string(),
    }
}

fn format_stop_reason(reason: AutonomousPlannerStopReason) -> String {
    match reason {
        AutonomousPlannerStopReason::GoalSatisfied => "goal-satisfied".to_string(),
        AutonomousPlannerStopReason::StepLimitReached => "step-limit-reached".to_string(),
        AutonomousPlannerStopReason::NoFurtherQueries => "no-further-queries".to_string(),
        AutonomousPlannerStopReason::NoAdditionalEvidence => "no-additional-evidence".to_string(),
    }
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
    use super::SiftAutonomousGathererAdapter;
    use crate::domain::ports::{
        ContextGatherRequest, ContextGatherer, EvidenceBudget, GathererCapability,
        PlannerStrategyKind,
    };
    use crate::infrastructure::adapters::sift_context_gatherer::SiftContextGathererAdapter;
    use tempfile::tempdir;

    #[test]
    fn autonomous_gatherer_reports_available_capability() {
        let adapter = SiftAutonomousGathererAdapter::new(".");
        let request = ContextGatherRequest::new(
            "Summarize the runtime path",
            ".",
            "test",
            EvidenceBudget::default(),
        );

        assert_eq!(adapter.capability(), GathererCapability::Available);
        assert_eq!(
            adapter.planner_strategy(&request).kind,
            sift::AutonomousPlannerStrategyKind::Heuristic
        );
    }

    #[test]
    fn autonomous_gatherer_supports_optional_model_driven_profile_configuration() {
        let adapter =
            SiftAutonomousGathererAdapter::new(".").with_model_driven_profile("local-planner-v1");
        let request = ContextGatherRequest::new(
            "Summarize the runtime path",
            ".",
            "test",
            EvidenceBudget::default(),
        )
        .with_planning(
            crate::domain::ports::PlannerConfig::default()
                .with_strategy(PlannerStrategyKind::ModelDriven),
        );

        let planner_strategy = adapter.planner_strategy(&request);
        assert_eq!(
            planner_strategy.kind,
            sift::AutonomousPlannerStrategyKind::ModelDriven
        );
        assert_eq!(
            planner_strategy.profile.as_deref(),
            Some("local-planner-v1")
        );
    }

    #[tokio::test]
    async fn autonomous_gatherer_returns_planner_metadata_and_evidence() {
        let workspace = tempdir().expect("workspace");
        std::fs::write(
            workspace.path().join("alpha.txt"),
            "alpha runtime details for the autonomous gatherer adapter",
        )
        .expect("write alpha");

        let adapter = SiftAutonomousGathererAdapter::new(workspace.path());
        let result = adapter
            .gather_context(&ContextGatherRequest::new(
                "find alpha runtime details",
                workspace.path(),
                "repo investigation",
                EvidenceBudget::default(),
            ))
            .await
            .expect("gather result");

        let bundle = result.evidence_bundle.expect("bundle");
        let planner = bundle.planner.expect("planner metadata");
        assert_eq!(planner.strategy, PlannerStrategyKind::Heuristic);
        assert!(planner.completed);
        assert!(!bundle.items.is_empty());
        assert!(bundle.summary.contains("Autonomous `heuristic` gatherer"));
    }

    #[tokio::test]
    async fn autonomous_and_static_gatherers_produce_distinct_evidence_shapes() {
        let workspace = tempdir().expect("workspace");
        std::fs::write(
            workspace.path().join("alpha.txt"),
            "alpha runtime details for the autonomous gatherer adapter",
        )
        .expect("write alpha");

        let request = ContextGatherRequest::new(
            "find alpha runtime details",
            workspace.path(),
            "repo investigation",
            EvidenceBudget::default(),
        );
        let autonomous = SiftAutonomousGathererAdapter::new(workspace.path());
        let static_gatherer = SiftContextGathererAdapter::new(workspace.path(), "qwen-1.5b");

        let autonomous_bundle = autonomous
            .gather_context(&request)
            .await
            .expect("autonomous result")
            .evidence_bundle
            .expect("autonomous bundle");
        let static_bundle = static_gatherer
            .gather_context(&request)
            .await
            .expect("static result")
            .evidence_bundle
            .expect("static bundle");

        assert!(autonomous_bundle.planner.is_some());
        assert!(static_bundle.planner.is_none());
        assert!(autonomous_bundle.summary.contains("Autonomous"));
        assert!(static_bundle.summary.contains("Gathered"));
    }
}
