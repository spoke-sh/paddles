use crate::domain::model::BootContext;
use crate::domain::ports::{ModelPaths, ModelRegistry};
use crate::infrastructure::adapters::sift_agent::SiftAgentAdapter;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use tokio::sync::RwLock;

/// Application service for managing the mech suit lifecycle.
pub struct MechSuitService {
    workspace_root: PathBuf,
    registry: Arc<dyn ModelRegistry>,
    engine: RwLock<Option<Arc<SiftAgentAdapter>>>,
    verbose: AtomicU8,
}

impl MechSuitService {
    pub fn new(workspace_root: impl Into<PathBuf>, registry: Arc<dyn ModelRegistry>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
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

        let engine = Arc::new(SiftAgentAdapter::new(
            self.workspace_root.clone(),
            model_id,
        )?);
        engine.set_verbose(self.verbose.load(Ordering::Relaxed));
        *self.engine.write().await = Some(engine);

        Ok(paths)
    }

    /// Process a single prompt using a specific model.
    pub async fn process_prompt(&self, prompt: &str, _paths: ModelPaths) -> Result<String> {
        let engine_guard = self.engine.read().await;
        let engine = engine_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Engine not initialized"))?
            .clone();

        let prompt = prompt.to_string();
        tokio::task::spawn_blocking(move || engine.respond(&prompt))
            .await
            .map_err(|err| anyhow::anyhow!("Sift session task failed: {err}"))?
    }
}
