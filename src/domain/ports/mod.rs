use async_trait::async_trait;
use wonopcode_provider::{Message, GenerateOptions, ProviderResult, StreamChunk};
use futures::stream::BoxStream;
use std::path::PathBuf;

/// Port for the agentic inference engine.
#[async_trait]
pub trait InferenceEngine: Send + Sync {
    async fn generate(
        &self,
        messages: Vec<Message>,
        options: GenerateOptions,
    ) -> ProviderResult<BoxStream<'static, ProviderResult<StreamChunk>>>;
    
    fn id(&self) -> &str;
}

/// Port for model discovery and acquisition.
#[async_trait]
pub trait ModelRegistry: Send + Sync {
    /// Get the local paths for a model by its ID.
    async fn get_model_paths(&self, model_id: &str) -> Result<ModelPaths, anyhow::Error>;
}

/// Paths to local model assets.
pub struct ModelPaths {
    pub weights: PathBuf,
    pub tokenizer: PathBuf,
    pub config: PathBuf,
}
