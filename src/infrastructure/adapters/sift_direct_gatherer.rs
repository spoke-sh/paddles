use crate::domain::model::{TurnEvent, TurnEventSink};
use crate::domain::ports::{
    ContextGatherRequest, ContextGatherResult, ContextGatherer, EvidenceBundle, EvidenceItem,
    GathererCapability, PlannerDecision, PlannerTraceMetadata, PlannerTraceStep, RetainedEvidence,
};
use crate::infrastructure::adapters::sift_progress::{SiftProgressDisplay, describe_sift_progress};
use crate::infrastructure::adapters::sift_request_factory::SiftRequestFactory;
use crate::infrastructure::sift_cache::{
    default_sift_cache_dir_for_workspace, ensure_sift_process_cache_dirs,
};
use anyhow::Result;
use async_trait::async_trait;
use sift::{SearchInput, SearchProgress, SearchResponse, Sift};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct SiftDirectGathererAdapter {
    workspace_root: PathBuf,
    sift: Arc<Sift>,
    verbose: AtomicU8,
    event_sink: Mutex<Option<Arc<dyn TurnEventSink>>>,
}

impl SiftDirectGathererAdapter {
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let workspace_root = workspace_root.into();
        ensure_sift_process_cache_dirs();
        Self {
            workspace_root: workspace_root.clone(),
            sift: Arc::new(
                Sift::builder()
                    .with_cache_dir(default_sift_cache_dir_for_workspace(&workspace_root))
                    .build(),
            ),
            verbose: AtomicU8::new(0),
            event_sink: Mutex::new(None),
        }
    }

    pub fn set_verbose(&self, level: u8) {
        self.verbose.store(level, Ordering::Relaxed);
    }

    fn build_search_input(&self, request: &ContextGatherRequest) -> SearchInput {
        SiftRequestFactory::direct_search_input(
            &self.workspace_root,
            request,
            self.verbose.load(Ordering::Relaxed),
        )
    }

    fn search_workspace_with_progress<F: Fn(&SearchProgress)>(
        &self,
        request: &ContextGatherRequest,
        progress: F,
    ) -> Result<SearchResponse> {
        self.sift
            .search_with_progress(self.build_search_input(request), Some(progress))
    }
}

#[async_trait]
impl ContextGatherer for SiftDirectGathererAdapter {
    fn capability(&self) -> GathererCapability {
        GathererCapability::Available
    }

    fn set_event_sink(&self, sink: Option<Arc<dyn TurnEventSink>>) {
        *self.event_sink.lock().expect("event sink lock") = sink;
    }

    async fn gather_context(
        &self,
        request: &ContextGatherRequest,
    ) -> Result<ContextGatherResult, anyhow::Error> {
        let event_sink = self.event_sink.lock().expect("event sink lock").clone();
        let strategy = request.planning.retrieval_strategy.label().to_string();
        if let Some(sink) = event_sink.as_ref() {
            sink.emit(TurnEvent::GathererSearchProgress {
                phase: "searching".to_string(),
                elapsed_seconds: 0,
                eta_seconds: None,
                strategy: Some(strategy.clone()),
                detail: Some("initializing direct retrieval".to_string()),
            });
        }

        let done = Arc::new(AtomicBool::new(false));
        let done_flag = Arc::clone(&done);
        let (heartbeat_tx, mut heartbeat_rx) = tokio::sync::mpsc::unbounded_channel::<u64>();
        let timer_handle = tokio::spawn(async move {
            let start = tokio::time::Instant::now();
            let mut delay = tokio::time::sleep(Duration::from_secs(2));
            loop {
                delay.await;
                if done_flag.load(Ordering::Relaxed) {
                    break;
                }
                let elapsed_seconds = start.elapsed().as_secs();
                if heartbeat_tx.send(elapsed_seconds).is_err() {
                    break;
                }
                delay = tokio::time::sleep(Duration::from_secs(60));
            }
        });

        let workspace_root = self.workspace_root.clone();
        let verbose = self.verbose.load(Ordering::Relaxed);
        let request = request.clone();
        let request_for_search = request.clone();
        let event_sink_for_task = event_sink.clone();
        let strategy_for_task = strategy.clone();
        let sift = Arc::clone(&self.sift);
        let latest_progress = Arc::new(Mutex::new(None::<SiftProgressDisplay>));
        let latest_progress_for_task = Arc::clone(&latest_progress);
        let search_handle = tokio::task::spawn_blocking(move || {
            let started_at = Instant::now();
            let adapter = SiftDirectGathererAdapter {
                workspace_root,
                sift,
                verbose: AtomicU8::new(verbose),
                event_sink: Mutex::new(None),
            };
            let telemetry_sift = Arc::clone(&adapter.sift);
            let result =
                adapter.search_workspace_with_progress(&request_for_search, move |progress| {
                    let display =
                        describe_sift_progress(progress, &telemetry_sift.telemetry_snapshot());
                    *latest_progress_for_task
                        .lock()
                        .expect("latest progress lock") = Some(display.clone());
                    if let Some(sink) = event_sink_for_task.as_ref() {
                        sink.emit(TurnEvent::GathererSearchProgress {
                            phase: display.phase,
                            elapsed_seconds: started_at.elapsed().as_secs(),
                            eta_seconds: display
                                .estimated_remaining
                                .map(|duration| duration.as_secs()),
                            strategy: Some(strategy_for_task.clone()),
                            detail: Some(display.detail),
                        });
                    }
                });
            done.store(true, Ordering::Relaxed);
            result
        });

        let mut search_handle = search_handle;
        let response = loop {
            tokio::select! {
                biased;
                event = heartbeat_rx.recv() => {
                    if let Some(elapsed_seconds) = event
                        && let Some(sink) = event_sink.as_ref()
                    {
                        let latest = latest_progress.lock().expect("latest progress lock").clone();
                        sink.emit(TurnEvent::GathererSearchProgress {
                            phase: latest
                                .as_ref()
                                .map(|progress| progress.phase.clone())
                                .unwrap_or_else(|| "searching".to_string()),
                            elapsed_seconds,
                            eta_seconds: latest
                                .as_ref()
                                .and_then(|progress| progress.estimated_remaining.map(|duration| duration.as_secs())),
                            strategy: Some(strategy.clone()),
                            detail: Some(
                                latest
                                    .as_ref()
                                    .map(|progress| progress.detail.clone())
                                    .unwrap_or_else(|| "direct retrieval in progress".to_string()),
                            ),
                        });
                    }
                }
                result = &mut search_handle => {
                    timer_handle.abort();
                    break result??;
                }
            }
        };

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
    use crate::domain::model::{TurnEvent, TurnEventSink};
    use crate::domain::ports::{
        ContextGatherRequest, ContextGatherer, EvidenceBudget, GathererCapability, PlannerConfig,
        RetrievalMode, RetrievalStrategy,
    };
    use crate::infrastructure::sift_cache::TEST_SIFT_ENV_LOCK;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    #[derive(Default)]
    struct RecordingSink {
        events: Mutex<Vec<TurnEvent>>,
    }

    impl RecordingSink {
        fn recorded(&self) -> Vec<TurnEvent> {
            self.events.lock().expect("events lock").clone()
        }
    }

    impl TurnEventSink for RecordingSink {
        fn emit(&self, event: TurnEvent) {
            self.events.lock().expect("events lock").push(event);
        }
    }

    #[test]
    fn direct_gatherer_returns_direct_retrieval_metadata_and_evidence() {
        let _env_guard = TEST_SIFT_ENV_LOCK.lock().expect("env lock");
        let workspace = tempdir().expect("workspace");
        std::fs::write(
            workspace.path().join("alpha.txt"),
            "alpha runtime details for the direct gatherer adapter",
        )
        .expect("write alpha");

        let adapter = SiftDirectGathererAdapter::new(workspace.path());
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let result = runtime
            .block_on(adapter.gather_context(&ContextGatherRequest::new(
                "find alpha runtime details",
                workspace.path(),
                "repo investigation",
                EvidenceBudget::default(),
            )))
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

    #[test]
    fn direct_gatherer_respects_budget_and_requested_mode_metadata() {
        let _env_guard = TEST_SIFT_ENV_LOCK.lock().expect("env lock");
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
                .with_retrieval_strategy(RetrievalStrategy::Vector),
        )
        .with_prior_context(vec!["Prefer runtime-related files first.".to_string()]);

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let result = runtime
            .block_on(adapter.gather_context(&request))
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

    #[test]
    fn direct_gatherer_emits_concrete_progress_without_planner_labels() {
        let _env_guard = TEST_SIFT_ENV_LOCK.lock().expect("env lock");
        let workspace = tempdir().expect("workspace");
        std::fs::write(
            workspace.path().join("alpha.txt"),
            "alpha runtime details for the direct gatherer adapter",
        )
        .expect("write alpha");

        let adapter = SiftDirectGathererAdapter::new(workspace.path());
        let sink = Arc::new(RecordingSink::default());
        adapter.set_event_sink(Some(sink.clone() as Arc<dyn TurnEventSink>));
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime
            .block_on(adapter.gather_context(&ContextGatherRequest::new(
                "find alpha runtime details",
                workspace.path(),
                "repo investigation",
                EvidenceBudget::default(),
            )))
            .expect("gather result");

        let progress_details = sink
            .recorded()
            .into_iter()
            .filter_map(|event| match event {
                TurnEvent::GathererSearchProgress { detail, .. } => detail,
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(!progress_details.is_empty());
        assert!(
            progress_details
                .iter()
                .any(|detail| detail.contains("indexing"))
        );
        assert!(
            progress_details
                .iter()
                .any(|detail| detail.contains("bm25 cache"))
        );
        assert!(
            progress_details
                .iter()
                .any(|detail| detail.contains("fresh"))
        );
        assert!(
            progress_details
                .iter()
                .all(|detail| !detail.contains("Terminate"))
        );
    }
}
