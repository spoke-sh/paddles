use crate::domain::model::BootContext;
use crate::domain::ports::{ModelPaths, ModelRegistry};
use anyhow::Result;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use wonopcode_core::{Instance, PromptConfig, PromptLoop};

/// Application service for managing the mech suit lifecycle.
pub struct MechSuitService {
    instance: Instance,
    registry: Arc<dyn ModelRegistry>,
}

impl MechSuitService {
    pub fn new(instance: Instance, registry: Arc<dyn ModelRegistry>) -> Self {
        Self { instance, registry }
    }

    /// Execute the boot sequence.
    pub fn boot(
        &self,
        credits: u64,
        weight: f64,
        bias: f64,
        hf_token: Option<String>,
        reality_mode: bool,
    ) -> Result<BootContext> {
        BootContext::new(credits, weight, bias, hf_token, reality_mode)
    }

    /// Prepare the model for inference.
    pub async fn prepare_model(&self, model_id: &str) -> Result<ModelPaths> {
        self.registry.get_model_paths(model_id).await
    }

    /// Process a single prompt using a specific model.
    pub async fn process_prompt(&self, prompt: &str, paths: ModelPaths) -> Result<String> {
        let session = self
            .instance
            .create_session(Some("paddles-session".to_string()))
            .await?;

        // Use the Sift-backed inference adapter
        let provider = Arc::new(
            crate::infrastructure::adapters::candle::SiftInferenceAdapter::new(paths.weights)?,
        );
        let tools = Arc::new(wonopcode_tools::ToolRegistry::default());
        let session_repo = Arc::new(self.instance.session_repo());
        let bus = self.instance.bus().clone();
        let cancel = CancellationToken::new();

        let loop_engine = PromptLoop::new(provider, tools, session_repo, bus, cancel);

        let result = loop_engine
            .run(&session, prompt, PromptConfig::default())
            .await?;
        Ok(result.text)
    }
}
