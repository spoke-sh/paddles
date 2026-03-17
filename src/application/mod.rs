use crate::domain::model::BootContext;
use crate::domain::ports::{InferenceEngine, ModelRegistry, ModelPaths};
use anyhow::Result;
use wonopcode_core::{Instance, PromptLoop, PromptConfig};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Application service for managing the mech suit lifecycle.
pub struct MechSuitService {
    instance: Instance,
    _engine: Arc<dyn InferenceEngine>,
    registry: Arc<dyn ModelRegistry>,
}

impl MechSuitService {
    pub fn new(instance: Instance, engine: Arc<dyn InferenceEngine>, registry: Arc<dyn ModelRegistry>) -> Self {
        Self { instance, _engine: engine, registry }
    }

    /// Execute the boot sequence.
    pub fn boot(&self, credits: u64, weight: f64, bias: f64, reality_mode: bool) -> Result<BootContext> {
        BootContext::new(credits, weight, bias, reality_mode)
    }

    /// Prepare the model for inference.
    pub async fn prepare_model(&self, model_id: &str) -> Result<ModelPaths> {
        self.registry.get_model_paths(model_id).await
    }

    /// Process a single prompt.
    pub async fn process_prompt(&self, prompt: &str) -> Result<String> {
        let session = self.instance.create_session(Some("paddles-session".to_string())).await?;
        
        // In a real implementation, we would use the paths from prepare_model
        // to initialize the engine. For now, we still use the CandleAdapter.
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
