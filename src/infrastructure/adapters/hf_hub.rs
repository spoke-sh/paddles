use crate::domain::ports::{ModelPaths, ModelRegistry};
use async_trait::async_trait;
use hf_hub::api::tokio::Api;
use hf_hub::{Repo, RepoType};
use std::path::PathBuf;

pub struct HFHubAdapter {
    api: Api,
}

impl HFHubAdapter {
    pub fn new() -> Result<Self, anyhow::Error> {
        let api = Api::new()?;
        Ok(Self { api })
    }
}

#[async_trait]
impl ModelRegistry for HFHubAdapter {
    async fn get_model_paths(&self, model_id: &str) -> Result<ModelPaths, anyhow::Error> {
        println!("[REGISTRY] Resolving model: {}", model_id);
        
        // Map common aliases to real HF repos
        let (repo_id, weights_filename) = match model_id {
            "gemma-2b" => ("google/gemma-2b-it", "model.safetensors"),
            "qwen-1.5b" => ("Qwen/Qwen2.5-1.5B-Instruct", "model.safetensors"),
            _ => {
                // Default to assuming model_id is the repo_id
                (model_id, "model.safetensors")
            }
        };

        let repo = self.api.repo(Repo::new(repo_id.to_string(), RepoType::Model));
        
        println!("[REGISTRY] Downloading config.json...");
        let config = repo.get("config.json").await?;
        
        println!("[REGISTRY] Downloading tokenizer.json...");
        let tokenizer = repo.get("tokenizer.json").await?;
        
        println!("[REGISTRY] Downloading {}...", weights_filename);
        let weights = repo.get(weights_filename).await?;

        Ok(ModelPaths {
            weights,
            tokenizer,
            config,
        })
    }
}
