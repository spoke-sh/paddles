use crate::domain::ports::{
    ContextGatherRequest, ContextGatherResult, RetrievalCapability, RetrievalProvider,
};
use async_trait::async_trait;

pub struct Context1RetrievalAdapter {
    harness_ready: bool,
}

impl Context1RetrievalAdapter {
    pub fn new(harness_ready: bool) -> Self {
        Self { harness_ready }
    }
}

#[async_trait]
impl RetrievalProvider for Context1RetrievalAdapter {
    fn capability(&self) -> RetrievalCapability {
        if self.harness_ready {
            RetrievalCapability::Unsupported {
                reason: "Context-1 is explicitly selected, but Paddles does not yet ship the dedicated harness-backed provider implementation required to execute it honestly.".to_string(),
            }
        } else {
            RetrievalCapability::HarnessRequired {
                reason: "Context-1 requires a dedicated search harness. Re-run with --context1-harness-ready only when that external harness is actually available.".to_string(),
            }
        }
    }

    async fn gather_context(
        &self,
        _request: &ContextGatherRequest,
    ) -> Result<ContextGatherResult, anyhow::Error> {
        Ok(match self.capability() {
            RetrievalCapability::Available => ContextGatherResult::unsupported(
                "Context-1 capability probing reported available without a provider implementation.",
            ),
            RetrievalCapability::Warming { reason } => ContextGatherResult::unsupported(reason),
            RetrievalCapability::Unsupported { reason } => ContextGatherResult::unsupported(reason),
            RetrievalCapability::HarnessRequired { reason } => {
                ContextGatherResult::harness_required(reason)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Context1RetrievalAdapter;
    use crate::domain::ports::{
        ContextGatherRequest, EvidenceBudget, RetrievalCapability, RetrievalProvider,
    };
    use std::path::PathBuf;

    #[test]
    fn context1_without_harness_reports_harness_required() {
        let adapter = Context1RetrievalAdapter::new(false);
        assert!(matches!(
            adapter.capability(),
            RetrievalCapability::HarnessRequired { .. }
        ));
    }

    #[tokio::test]
    async fn context1_with_harness_flag_but_no_provider_reports_unsupported() {
        let adapter = Context1RetrievalAdapter::new(true);
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
            RetrievalCapability::Unsupported { .. }
        ));
        assert!(result.evidence_bundle.is_none());
    }
}
