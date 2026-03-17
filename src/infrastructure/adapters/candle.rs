use crate::domain::ports::InferenceEngine;
use async_trait::async_trait;
use candle_core::Device;
use wonopcode_provider::{
    LanguageModel, Message, GenerateOptions, ProviderResult, 
    StreamChunk, ModelInfo, ContentPart
};
use futures::stream::{self, BoxStream};

pub struct CandleAdapter {
    info: ModelInfo,
    device: Device,
}

impl Default for CandleAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl CandleAdapter {
    pub fn new() -> Self {
        let device = Device::Cpu; 
        Self {
            info: ModelInfo {
                id: "local-candle-llama".to_string(),
                name: "Local Candle Llama".to_string(),
                ..Default::default()
            },
            device,
        }
    }
}

#[async_trait]
impl InferenceEngine for CandleAdapter {
    async fn generate(
        &self,
        messages: Vec<Message>,
        options: GenerateOptions,
    ) -> ProviderResult<BoxStream<'static, ProviderResult<StreamChunk>>> {
        // Delegate to LanguageModel implementation
        LanguageModel::generate(self, messages, options).await
    }

    fn id(&self) -> &str {
        "candle"
    }
}

#[async_trait]
impl LanguageModel for CandleAdapter {
    async fn generate(
        &self,
        messages: Vec<Message>,
        _options: GenerateOptions,
    ) -> ProviderResult<BoxStream<'static, ProviderResult<StreamChunk>>> {
        let mut prompt_text = String::new();
        if let Some(last_msg) = messages.last() {
            for part in &last_msg.content {
                if let ContentPart::Text { text } = part {
                    prompt_text.push_str(text);
                }
            }
        }
        
        let chunks = vec![
            Ok(StreamChunk::TextStart),
            Ok(StreamChunk::TextDelta(format!("(Infrastructure::Candle) I heard: '{}' on device: {:?}", prompt_text, self.device))),
            Ok(StreamChunk::TextEnd),
            Ok(StreamChunk::FinishStep {
                usage: wonopcode_provider::stream::Usage::new(0, 0),
                finish_reason: wonopcode_provider::stream::FinishReason::EndTurn,
            }),
        ];
        Ok(Box::pin(stream::iter(chunks)) as BoxStream<'static, ProviderResult<StreamChunk>>)
    }

    fn model_info(&self) -> &ModelInfo {
        &self.info
    }

    fn provider_id(&self) -> &str {
        "candle"
    }
}
