use crate::domain::ports::{
    ContextGatherRequest, ContextGatherResult, ContextGatherer, GathererCapability,
};
use async_trait::async_trait;

pub struct Context1GathererAdapter {
    harness_ready: bool,
}

impl Context1GathererAdapter {
    pub fn new(harness_ready: bool) -> Self {
        Self { harness_ready }
    }
}

#[async_trait]
impl ContextGatherer for Context1GathererAdapter {
    fn capability(&self) -> GathererCapability {
        if self.harness_ready {
            GathererCapability::Unsupported {
                reason: "Context-1 is explicitly selected, but Paddles does not yet ship the dedicated harness-backed provider implementation required to execute it honestly.".to_string(),
            }
        } else {
            GathererCapability::HarnessRequired {
                reason: "Context-1 requires a dedicated search harness. Re-run with --context1-harness-ready only when that external harness is actually available.".to_string(),
            }
        }
    }

    async fn gather_context(
        &self,
        _request: &ContextGatherRequest,
    ) -> Result<ContextGatherResult, anyhow::Error> {
        Ok(match self.capability() {
            GathererCapability::Available => ContextGatherResult::unsupported(
                "Context-1 capability probing reported available without a provider implementation.",
            ),
            GathererCapability::Warming { reason } => ContextGatherResult::unsupported(reason),
            GathererCapability::Unsupported { reason } => ContextGatherResult::unsupported(reason),
            GathererCapability::HarnessRequired { reason } => {
                ContextGatherResult::harness_required(reason)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Context1GathererAdapter;
    use crate::domain::ports::{
        ContextGatherRequest, ContextGatherer, EvidenceBudget, GathererCapability,
    };
    use std::path::PathBuf;

    #[test]
    fn context1_without_harness_reports_harness_required() {
        let adapter = Context1GathererAdapter::new(false);
        assert!(matches!(
            adapter.capability(),
            GathererCapability::HarnessRequired { .. }
        ));
    }

    #[tokio::test]
    async fn context1_with_harness_flag_but_no_provider_reports_unsupported() {
        let adapter = Context1GathererAdapter::new(true);
        let result = adapter
            .gather_context(&ContextGatherRequest::new(
                "Summarize the repo",
                PathBuf::from("."),
                "test",
                EvidenceBudget::default(),
            ))
            .await
            .expect("result");

        assert!(matches!(
            result.capability,
            GathererCapability::Unsupported { .. }
        ));
        assert!(result.evidence_bundle.is_none());
    }
}
