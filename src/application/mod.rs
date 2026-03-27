use crate::domain::model::BootContext;
use crate::domain::ports::{InferenceEngine, ModelPaths, ModelRegistry};
use crate::infrastructure::adapters::candle::SiftInferenceAdapter;
use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use wonopcode_core::{Instance, PromptConfig, PromptLoop};

/// Application service for managing the mech suit lifecycle.
pub struct MechSuitService {
    instance: Instance,
    registry: Arc<dyn ModelRegistry>,
    engine: RwLock<Option<Arc<SiftInferenceAdapter>>>,
    verbose: AtomicU8,
}

impl MechSuitService {
    pub fn new(instance: Instance, registry: Arc<dyn ModelRegistry>) -> Self {
        Self {
            instance,
            registry,
            engine: RwLock::new(None),
            verbose: AtomicU8::new(0),
        }
    }

    pub fn set_verbose(&self, level: u8) {
        self.verbose.store(level, Ordering::Relaxed);
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
        let paths = self.registry.get_model_paths(model_id).await?;

        // Initialize the engine once
        let mut provider =
            crate::infrastructure::adapters::candle::SiftInferenceAdapter::new(model_id)?;
        provider.set_verbose(self.verbose.load(Ordering::Relaxed));

        let provider_arc = Arc::new(provider);
        *self.engine.write().await = Some(provider_arc);

        Ok(paths)
    }

    /// Process a single prompt using a specific model.
    pub async fn process_prompt(&self, prompt: &str, _paths: ModelPaths) -> Result<String> {
        let verbose = self.verbose.load(Ordering::Relaxed);

        let session = self
            .instance
            .create_session(Some("paddles-session".to_string()))
            .await?;

        if verbose >= 2 {
            println!("[TRACE] Session created: {}", session.id);
        }

        let engine_guard = self.engine.read().await;
        let provider = engine_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Engine not initialized"))?
            .clone();

        if verbose >= 2 {
            println!("[TRACE] Provider resolved: {}", provider.id());
        }

        let tools = Arc::new(wonopcode_tools::ToolRegistry::default());
        let session_repo = Arc::new(self.instance.session_repo());
        let bus = self.instance.bus().clone();
        let cancel = CancellationToken::new();

        let loop_engine = PromptLoop::new(provider, tools, session_repo, bus, cancel);

        if verbose >= 1 {
            println!("[INFO] Running prompt loop for: '{}'", prompt);
        }

        let result = loop_engine
            .run(&session, prompt, PromptConfig::default())
            .await?;

        if verbose >= 1 {
            println!("[INFO] Prompt loop complete. Output received.");
        }

        Ok(result.text)
    }
}
