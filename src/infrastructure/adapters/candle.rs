use crate::domain::ports::InferenceEngine;
use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use sift::internal::search::adapters::qwen::{QwenModelSpec, QwenReranker};
use sift::internal::search::domain::GenerativeModel;
use std::sync::Arc;
use wonopcode_provider::{
    ContentPart, GenerateOptions, LanguageModel, Message, ModelInfo, ProviderResult, StreamChunk,
};

pub struct SiftInferenceAdapter {
    info: ModelInfo,
    inner: Arc<QwenReranker>,
}

impl SiftInferenceAdapter {
    pub fn new(_weights: std::path::PathBuf) -> Result<Self, anyhow::Error> {
        let spec = QwenModelSpec::default();
        let inner = QwenReranker::load(spec)?;

        Ok(Self {
            info: ModelInfo {
                id: "sift-qwen".to_string(),
                name: "Sift Qwen".to_string(),
                ..Default::default()
            },
            inner: Arc::new(inner),
        })
    }
}

#[async_trait]
impl InferenceEngine for SiftInferenceAdapter {
    async fn generate(
        &self,
        messages: Vec<Message>,
        options: GenerateOptions,
    ) -> ProviderResult<BoxStream<'static, ProviderResult<StreamChunk>>> {
        LanguageModel::generate(self, messages, options).await
    }

    fn id(&self) -> &str {
        "sift-qwen"
    }
}

#[async_trait]
impl LanguageModel for SiftInferenceAdapter {
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

        let inner = self.inner.clone();
        let response = tokio::task::spawn_blocking(move || inner.generate(&prompt_text, 512))
            .await
            .map_err(|e| wonopcode_provider::error::ProviderError::Internal {
                message: e.to_string(),
            })?
            .map_err(|e| wonopcode_provider::error::ProviderError::Internal {
                message: e.to_string(),
            })?;

        let chunks = vec![
            Ok(StreamChunk::TextStart),
            Ok(StreamChunk::TextDelta(response)),
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
        "sift"
    }
}
