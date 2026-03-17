use async_trait::async_trait;
use wonopcode_provider::{Message, GenerateOptions, ProviderResult, StreamChunk};
use futures::stream::BoxStream;

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
