use crate::domain::ports::{
    ContextGatherRequest, ContextGatherResult, ContextGatherer, EvidenceBundle, EvidenceItem,
    GathererCapability,
};
use anyhow::Result;
use async_trait::async_trait;
use sift::{
    ContextAssemblyBudget, ContextAssemblyRequest, ContextAssemblyResponse, EnvironmentFactInput,
    LocalContextSource, SearchPlan, Sift,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};

const DEFAULT_RETAINED_LIMIT: usize = 5;

pub struct SiftContextGathererAdapter {
    workspace_root: PathBuf,
    model_id: String,
    sift: Sift,
    verbose: AtomicU8,
}

impl SiftContextGathererAdapter {
    pub fn new(workspace_root: impl Into<PathBuf>, model_id: impl Into<String>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            model_id: model_id.into(),
            sift: Sift::builder().build(),
            verbose: AtomicU8::new(0),
        }
    }

    pub fn set_verbose(&self, level: u8) {
        self.verbose.store(level, Ordering::Relaxed);
    }

    fn assemble_context(&self, request: &ContextGatherRequest) -> Result<ContextAssemblyResponse> {
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

        self.sift.assemble_context(
            ContextAssemblyRequest::new(&self.workspace_root, &request.user_query)
                .with_plan(SearchPlan::default_lexical())
                .with_intent(request.intent_reason.clone())
                .with_limit(request.budget.max_items)
                .with_shortlist(request.budget.max_items)
                .with_budget(ContextAssemblyBudget::new(DEFAULT_RETAINED_LIMIT))
                .with_local_context(local_context),
        )
    }
}

#[async_trait]
impl ContextGatherer for SiftContextGathererAdapter {
    fn capability(&self) -> GathererCapability {
        GathererCapability::Available
    }

    async fn gather_context(
        &self,
        request: &ContextGatherRequest,
    ) -> Result<ContextGatherResult, anyhow::Error> {
        let assembly = self.assemble_context(request)?;
        if self.verbose.load(Ordering::Relaxed) >= 1 {
            println!(
                "[LANE] Gatherer lane '{}' assembled {} hit(s) for retrieval-heavy prompt.",
                self.model_id,
                assembly.response.hits.len(),
            );
        }

        let items = assembly
            .response
            .hits
            .iter()
            .take(request.budget.max_items)
            .enumerate()
            .map(|(index, hit)| EvidenceItem {
                source: hit.path.clone(),
                snippet: trim_for_budget(&hit.snippet, request.budget.max_snippet_chars),
                rationale: format!("retrieved for `{}`", request.user_query),
                rank: index + 1,
            })
            .collect::<Vec<_>>();

        let summary = if items.is_empty() {
            format!(
                "No matching evidence found for `{}` in the current workspace.",
                request.user_query
            )
        } else {
            format!(
                "Gathered {} ranked evidence item(s) for `{}` using the `{}` gatherer lane.",
                items.len(),
                request.user_query,
                self.model_id,
            )
        };

        Ok(ContextGatherResult::available(EvidenceBundle::new(
            summary, items,
        )))
    }
}

fn trim_for_budget(input: &str, limit: usize) -> String {
    if input.chars().count() <= limit {
        return input.to_string();
    }

    let kept = input.chars().take(limit).collect::<String>();
    format!("{kept}...[truncated]")
}
