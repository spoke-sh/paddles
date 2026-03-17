use crate::domain::model::BootContext;
use crate::domain::ports::InferenceEngine;
use anyhow::Result;
use wonopcode_core::{Instance, PromptLoop, PromptConfig};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Application service for managing the mech suit lifecycle.
pub struct MechSuitService {
    instance: Instance,
    _engine: Arc<dyn InferenceEngine>,
}

impl MechSuitService {
    pub fn new(instance: Instance, engine: Arc<dyn InferenceEngine>) -> Self {
        Self { instance, _engine: engine }
    }

    /// Execute the boot sequence.
    pub fn boot(&self, credits: u64, weight: f64, bias: f64, reality_mode: bool) -> Result<BootContext> {
        BootContext::new(credits, weight, bias, reality_mode)
    }

    /// Process a single prompt.
    pub async fn process_prompt(&self, prompt: &str) -> Result<String> {
        let session = self.instance.create_session(Some("paddles-session".to_string())).await?;
        
        let provider = Arc::new(crate::infrastructure::adapters::candle::CandleAdapter::new());
        let tools = Arc::new(wonopcode_tools::ToolRegistry::default());
        let session_repo = Arc::new(self.instance.session_repo());
        let bus = self.instance.bus().clone();
        let cancel = CancellationToken::new();
        
        let loop_engine = PromptLoop::new(
            provider,
            tools,
            session_repo,
            bus,
            cancel,
        );
        
        let result = loop_engine.run(&session, prompt, PromptConfig::default()).await?;
        Ok(result.text)
    }
}
