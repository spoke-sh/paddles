use crate::domain::ports::{ModelPaths, ModelRegistry};
use async_trait::async_trait;
use sift::internal::cache::cache_dir;
use sift::internal::search::adapters::llm_utils::ensure_hf_asset;
use sift::internal::search::adapters::qwen::QwenModelSpec;
use std::path::Path;

pub struct SiftRegistryAdapter;

pub fn qwen_spec_for(model_id: &str) -> QwenModelSpec {
    match model_id {
        "qwen-1.5b" => QwenModelSpec {
            model_id: "Qwen/Qwen2.5-1.5B-Instruct".to_string(),
            revision: "main".to_string(),
            max_length: 512,
        },
        _ => QwenModelSpec::default(),
    }
}

impl SiftRegistryAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SiftRegistryAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModelRegistry for SiftRegistryAdapter {
    async fn get_model_paths(&self, model_id: &str) -> Result<ModelPaths, anyhow::Error> {
        println!("[SIFT] Resolving model: {}", model_id);

        let spec = qwen_spec_for(model_id);

        let cache_root = cache_dir("models")?;

        let root = cache_root
            .join(Path::new(&spec.model_id))
            .join(Path::new(&spec.revision));

        let config_path = root.join("config.json");
        let tokenizer_path = root.join("tokenizer.json");
        let weights_path = root.join("model.safetensors");

        println!("[SIFT] Ensuring assets for {}...", spec.model_id);

        // Use sift's utility
        ensure_hf_asset(&spec.model_id, &spec.revision, &config_path, "config.json")?;
        ensure_hf_asset(
            &spec.model_id,
            &spec.revision,
            &tokenizer_path,
            "tokenizer.json",
        )?;
        ensure_hf_asset(
            &spec.model_id,
            &spec.revision,
            &weights_path,
            "model.safetensors",
        )?;

        Ok(ModelPaths {
            weights: weights_path,
            tokenizer: tokenizer_path,
            config: config_path,
        })
    }
}
