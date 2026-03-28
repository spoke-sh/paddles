use crate::domain::ports::{ModelPaths, ModelRegistry};
use crate::infrastructure::adapters::sift_registry::{QwenWeightLayout, qwen_spec_for};
use anyhow::Context;
use async_trait::async_trait;
use hf_hub::api::tokio::{Api, ApiBuilder};
use hf_hub::{Repo, RepoType};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub struct HFHubAdapter {
    api: Api,
}

impl HFHubAdapter {
    pub fn new(token: Option<String>) -> Result<Self, anyhow::Error> {
        let mut builder = ApiBuilder::new();
        if let Some(t) = token {
            builder = builder.with_token(Some(t));
        }
        let api = builder.build()?;
        Ok(Self { api })
    }
}

#[async_trait]
impl ModelRegistry for HFHubAdapter {
    async fn get_model_paths(&self, model_id: &str) -> Result<ModelPaths, anyhow::Error> {
        println!("[REGISTRY] Resolving model: {}", model_id);

        if let Ok(spec) = qwen_spec_for(model_id) {
            let repo = self
                .api
                .repo(Repo::new(spec.model_id.to_string(), RepoType::Model));

            println!("[REGISTRY] Downloading config.json...");
            let config = repo.get("config.json").await?;

            println!("[REGISTRY] Downloading tokenizer.json...");
            let tokenizer = repo.get("tokenizer.json").await?;

            let weights = match spec.weights {
                QwenWeightLayout::Single(file_name) => {
                    println!("[REGISTRY] Downloading {}...", file_name);
                    repo.get(file_name).await?
                }
                QwenWeightLayout::Indexed(index_name) => {
                    println!("[REGISTRY] Downloading {}...", index_name);
                    let index = repo.get(index_name).await?;
                    for shard_name in shard_names_from_index_path(&index)? {
                        println!("[REGISTRY] Downloading {}...", shard_name);
                        let _ = repo.get(&shard_name).await?;
                    }
                    index
                }
            };

            return Ok(ModelPaths {
                weights,
                tokenizer,
                config,
            });
        }

        let repo = self
            .api
            .repo(Repo::new(model_id.to_string(), RepoType::Model));

        println!("[REGISTRY] Downloading config.json...");
        let config = repo.get("config.json").await?;

        println!("[REGISTRY] Downloading tokenizer.json...");
        let tokenizer = repo.get("tokenizer.json").await?;

        println!("[REGISTRY] Downloading model.safetensors...");
        let weights = repo.get("model.safetensors").await?;

        Ok(ModelPaths {
            weights,
            tokenizer,
            config,
        })
    }
}

#[derive(Debug, Deserialize)]
struct SafetensorsIndex {
    weight_map: std::collections::HashMap<String, String>,
}

fn shard_names_from_index_path(index_path: &Path) -> Result<Vec<String>, anyhow::Error> {
    let index: SafetensorsIndex = serde_json::from_str(&fs::read_to_string(index_path)?)
        .with_context(|| format!("failed to parse {}", index_path.display()))?;

    let shards = index
        .weight_map
        .into_values()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    if shards.is_empty() {
        anyhow::bail!("no weight shards listed in {}", index_path.display());
    }

    Ok(shards)
}
