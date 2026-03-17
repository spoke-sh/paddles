use crate::domain::ports::{ModelPaths, ModelRegistry};
use async_trait::async_trait;
use sift::internal::search::adapters::qwen::QwenModelSpec;
use sift::internal::search::adapters::llm_utils::ensure_hf_asset;
use std::path::{Path, PathBuf};
use anyhow::anyhow;

pub struct SiftRegistryAdapter;

impl SiftRegistryAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ModelRegistry for SiftRegistryAdapter {
    async fn get_model_paths(&self, model_id: &str) -> Result<ModelPaths, anyhow::Error> {
        println!("[SIFT] Resolving model: {}", model_id);
        
        // We'll use sift's internal directory helpers or define our own if needed.
        // For this refactor, we'll map to what QwenModelSpec expects.
        let spec = match model_id {
            "qwen-1.5b" => QwenModelSpec {
                model_id: "Qwen/Qwen2.5-1.5B-Instruct".to_string(),
                revision: "main".to_string(),
                max_length: 512,
            },
            _ => QwenModelSpec::default(),
        };

        // Sift's ensure_hf_asset is synchronous and uses ureq.
        // We'll call it in spawn_blocking if needed, but since we are in a boot sequence
        // and it's IO bound, it's acceptable for now.
        
        let cache_root = directories::ProjectDirs::from("io", "wonop", "paddles")
            .ok_or_else(|| anyhow!("could not determine cache directory"))?
            .cache_dir()
            .join("models");

        let root = cache_root
            .join(Path::new(&spec.model_id))
            .join(Path::new(&spec.revision));

        let config_path = root.join("config.json");
        let tokenizer_path = root.join("tokenizer.json");
        let weights_path = root.join("model.safetensors");

        println!("[SIFT] Ensuring assets for {}...", spec.model_id);
        
        // Use sift's utility
        ensure_hf_asset(&spec.model_id, &spec.revision, &config_path, "config.json")?;
        ensure_hf_asset(&spec.model_id, &spec.revision, &tokenizer_path, "tokenizer.json")?;
        ensure_hf_asset(&spec.model_id, &spec.revision, &weights_path, "model.safetensors")?;

        Ok(ModelPaths {
            weights: weights_path,
            tokenizer: tokenizer_path,
            config: config_path,
        })
    }
}
