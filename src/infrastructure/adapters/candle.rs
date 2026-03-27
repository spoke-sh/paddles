use crate::domain::ports::InferenceEngine;
use crate::infrastructure::adapters::sift_registry::qwen_spec_for;
use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use sift::internal::search::adapters::qwen::QwenReranker;
use sift::internal::search::domain::{Conversation, GenerativeModel};
use std::sync::Arc;
use wonopcode_provider::{
    ContentPart, GenerateOptions, LanguageModel, Message, ModelInfo, ProviderResult, StreamChunk,
};

pub struct SiftInferenceAdapter {
    info: ModelInfo,
    conversation: Arc<std::sync::Mutex<Box<dyn Conversation>>>,
    verbose: std::sync::atomic::AtomicU8,
}

impl SiftInferenceAdapter {
    pub fn new(model_id: &str) -> Result<Self, anyhow::Error> {
        let spec = qwen_spec_for(model_id);
        let inner = QwenReranker::load(spec)?;
        let conversation = inner.start_conversation()?;

        Ok(Self {
            info: ModelInfo {
                id: "sift-qwen".to_string(),
                name: "Sift Qwen".to_string(),
                ..Default::default()
            },
            conversation: Arc::new(std::sync::Mutex::new(conversation)),
            verbose: std::sync::atomic::AtomicU8::new(0),
        })
    }

    pub fn set_verbose(&mut self, level: u8) {
        self.verbose
            .store(level, std::sync::atomic::Ordering::Relaxed);
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

        let conv = self.conversation.clone();
        let prompt_clone = prompt_text.clone();
        let verbose = self.verbose.load(std::sync::atomic::Ordering::Relaxed);

        if verbose >= 1 {
            println!("[INFO] SiftInferenceAdapter starting generation...");
        }

        let response = tokio::task::spawn_blocking(move || {
            let mut conv = conv.lock().unwrap();

            if verbose >= 3 {
                println!("[TRACE] Current Conversation History:");
                for (i, turn) in conv.history().iter().enumerate() {
                    println!("[TRACE]   Turn {}: {}", i, turn);
                }
            }

            if verbose >= 2 {
                println!("[DEBUG] Sending prompt to Sift model: '{}'", prompt_clone);
            }

            let result = conv.send(&prompt_clone, 512);

            #[allow(clippy::collapsible_if)]
            if verbose >= 2 {
                if let Ok(ref res) = result {
                    println!("[DEBUG] Sift model responded with: '{}'", res);
                }
            }

            result
        })
        .await
        .map_err(|e| wonopcode_provider::error::ProviderError::Internal {
            message: e.to_string(),
        })?
        .map_err(|e| wonopcode_provider::error::ProviderError::Internal {
            message: e.to_string(),
        })?;

        if verbose >= 1 {
            println!(
                "[INFO] SiftInferenceAdapter generation complete. Response length: {}",
                response.len()
            );
        }

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
