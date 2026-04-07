use crate::domain::ports::{
    ContextGatherRequest, RetrievalMode, RetrievalStrategy, RetrieverOption,
};
use sift::{
    AutonomousPlannerStrategy, AutonomousSearchMode, AutonomousSearchRequest,
    ContextAssemblyBudget, ContextAssemblyRequest, EnvironmentFactInput, FusionPolicy,
    LocalContextSource, QueryExpansionPolicy, RerankingPolicy, RetrieverPolicy, SearchInput,
    SearchOptions, SearchPlan,
};
use std::path::Path;

pub(crate) struct SiftRequestFactory;

impl SiftRequestFactory {
    pub(crate) fn local_context_sources(request: &ContextGatherRequest) -> Vec<LocalContextSource> {
        request
            .prior_context
            .iter()
            .enumerate()
            .map(|(index, value)| {
                LocalContextSource::EnvironmentFact(EnvironmentFactInput::new(
                    format!("prior_context_{index}"),
                    value.clone(),
                ))
            })
            .collect()
    }

    pub(crate) fn direct_search_input(
        workspace_root: &Path,
        request: &ContextGatherRequest,
        verbose: u8,
    ) -> SearchInput {
        let plan = Self::search_plan_for(
            request.planning.retrieval_strategy,
            &request.planning.retrievers,
        );
        SearchInput::new(workspace_root, request.user_query.clone())
            .with_intent(request.intent_reason.clone())
            .with_options(
                SearchOptions::default()
                    .with_strategy(plan.name)
                    .with_limit(request.budget.max_items)
                    .with_shortlist(request.budget.max_items)
                    .with_verbose(verbose)
                    .with_local_context(Self::local_context_sources(request)),
            )
    }

    pub(crate) fn context_assembly_request(
        workspace_root: &Path,
        request: &ContextGatherRequest,
        retained_limit: usize,
    ) -> ContextAssemblyRequest {
        let plan = Self::search_plan_for(
            request.planning.retrieval_strategy,
            &request.planning.retrievers,
        );
        ContextAssemblyRequest::new(workspace_root, &request.user_query)
            .with_strategy(plan.name.clone())
            .with_plan(plan)
            .with_intent(request.intent_reason.clone())
            .with_limit(request.budget.max_items)
            .with_shortlist(request.budget.max_items)
            .with_budget(ContextAssemblyBudget::new(retained_limit))
            .with_local_context(Self::local_context_sources(request))
    }

    pub(crate) fn autonomous_search_request(
        workspace_root: &Path,
        request: &ContextGatherRequest,
        verbose: u8,
        planner_strategy: AutonomousPlannerStrategy,
    ) -> AutonomousSearchRequest {
        let plan = Self::search_plan_for(
            request.planning.retrieval_strategy,
            &request.planning.retrievers,
        );
        AutonomousSearchRequest::new(workspace_root, &request.user_query)
            .with_strategy(plan.name.clone())
            .with_plan(plan)
            .with_mode(Self::autonomous_search_mode_for(request.planning.mode))
            .with_intent(request.intent_reason.clone())
            .with_planner_strategy(planner_strategy)
            .with_step_limit(request.planning.step_limit)
            .with_limit(request.budget.max_items)
            .with_shortlist(request.budget.max_items)
            .with_retained_artifact_limit(request.planning.retained_artifact_limit)
            .with_local_context(Self::local_context_sources(request))
            .with_verbose(verbose)
    }

    pub(crate) fn autonomous_search_mode_for(mode: RetrievalMode) -> AutonomousSearchMode {
        match mode {
            RetrievalMode::Linear => AutonomousSearchMode::Linear,
            RetrievalMode::Graph => AutonomousSearchMode::Graph,
        }
    }

    pub(crate) fn search_plan_for(
        strategy: RetrievalStrategy,
        retrievers: &[RetrieverOption],
    ) -> SearchPlan {
        match effective_structural_plan(strategy, retrievers) {
            Some(StructuralPlanPreset::PathHybrid) => SearchPlan::default_path_hybrid(),
            Some(StructuralPlanPreset::PageIndexHybrid) => SearchPlan::default_page_index_hybrid(),
            None => match strategy {
                RetrievalStrategy::Lexical => SearchPlan::default_lexical(),
                RetrievalStrategy::Vector => SearchPlan {
                    name: "vector".to_string(),
                    query_expansion: QueryExpansionPolicy::None,
                    retrievers: vec![RetrieverPolicy::Vector],
                    fusion: FusionPolicy::Rrf,
                    reranking: RerankingPolicy::None,
                },
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StructuralPlanPreset {
    PathHybrid,
    PageIndexHybrid,
}

fn effective_structural_plan(
    strategy: RetrievalStrategy,
    retrievers: &[RetrieverOption],
) -> Option<StructuralPlanPreset> {
    let has_path_fuzzy = retrievers.contains(&RetrieverOption::PathFuzzy);
    let has_segment_fuzzy = retrievers.contains(&RetrieverOption::SegmentFuzzy);

    if has_segment_fuzzy || (has_path_fuzzy && matches!(strategy, RetrievalStrategy::Vector)) {
        Some(StructuralPlanPreset::PageIndexHybrid)
    } else if has_path_fuzzy {
        Some(StructuralPlanPreset::PathHybrid)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::SiftRequestFactory;
    use crate::domain::ports::{
        ContextGatherRequest, EvidenceBudget, PlannerConfig, RetrievalMode, RetrievalStrategy,
        RetrieverOption,
    };
    use sift::{LocalContextSource, RetrieverPolicy};

    #[test]
    fn request_factory_turns_prior_context_into_environment_facts() {
        let request = ContextGatherRequest::new(
            "runtime shell host",
            "/workspace",
            "repo investigation",
            EvidenceBudget::default(),
        )
        .with_prior_context(vec![
            "Prefer runtime-related files first.".to_string(),
            "The shell host lives in CSS.".to_string(),
        ]);

        let local_context = SiftRequestFactory::local_context_sources(&request);

        assert_eq!(local_context.len(), 2);
        assert!(matches!(
            &local_context[0],
            LocalContextSource::EnvironmentFact(fact)
                if fact.key == "prior_context_0"
                    && fact.value == "Prefer runtime-related files first."
        ));
        assert!(matches!(
            &local_context[1],
            LocalContextSource::EnvironmentFact(fact)
                if fact.key == "prior_context_1" && fact.value == "The shell host lives in CSS."
        ));
    }

    #[test]
    fn request_factory_reuses_search_plan_for_vector_retrieval() {
        let request = ContextGatherRequest::new(
            "runtime shell host",
            "/workspace",
            "repo investigation",
            EvidenceBudget::default(),
        )
        .with_planning(
            PlannerConfig::default()
                .with_mode(RetrievalMode::Graph)
                .with_retrieval_strategy(RetrievalStrategy::Vector),
        );

        let plan = SiftRequestFactory::search_plan_for(
            request.planning.retrieval_strategy,
            &request.planning.retrievers,
        );

        assert_eq!(plan.name, "vector");
    }

    #[test]
    fn request_factory_uses_path_hybrid_when_structural_path_fuzzy_is_requested() {
        let request = ContextGatherRequest::new(
            "runtime app shell path",
            "/workspace",
            "repo investigation",
            EvidenceBudget::default(),
        )
        .with_planning(
            PlannerConfig::default()
                .with_retrieval_strategy(RetrievalStrategy::Lexical)
                .with_retrievers(vec![RetrieverOption::PathFuzzy]),
        );

        let plan = SiftRequestFactory::search_plan_for(
            request.planning.retrieval_strategy,
            &request.planning.retrievers,
        );

        assert_eq!(plan.name, "path-hybrid");
        assert_eq!(
            plan.retrievers,
            vec![RetrieverPolicy::Bm25, RetrieverPolicy::PathFuzzy]
        );
    }

    #[test]
    fn request_factory_uses_page_index_hybrid_when_segment_fuzzy_is_requested() {
        let request = ContextGatherRequest::new(
            "fn search_plan_for",
            "/workspace",
            "repo investigation",
            EvidenceBudget::default(),
        )
        .with_planning(
            PlannerConfig::default()
                .with_retrieval_strategy(RetrievalStrategy::Vector)
                .with_retrievers(vec![
                    RetrieverOption::PathFuzzy,
                    RetrieverOption::SegmentFuzzy,
                ]),
        );

        let plan = SiftRequestFactory::search_plan_for(
            request.planning.retrieval_strategy,
            &request.planning.retrievers,
        );

        assert_eq!(plan.name, "page-index-hybrid");
        assert!(plan.retrievers.contains(&RetrieverPolicy::PathFuzzy));
        assert!(plan.retrievers.contains(&RetrieverPolicy::SegmentFuzzy));
    }
}
