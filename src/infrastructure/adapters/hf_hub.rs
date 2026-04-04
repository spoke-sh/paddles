use crate::domain::ports::{ModelPaths, ModelRegistry};
use crate::infrastructure::adapters::sift_registry::model_source_for;
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use hf_hub::api::tokio::{Api, ApiBuilder, ApiRepo};
use hf_hub::{Repo, RepoType};
use serde::Deserialize;
use sift::ModelSource;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub struct HFHubAdapter {
    api: Api,
}

#[derive(Debug, Deserialize)]
struct SafetensorsIndex {
    weight_map: BTreeMap<String, String>,
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

pub(crate) async fn download_hf_model_paths(
    api: &Api,
    repo_id: &str,
    revision: Option<&str>,
) -> Result<ModelPaths> {
    let repo = api.repo(match revision {
        Some(revision) => {
            Repo::with_revision(repo_id.to_string(), RepoType::Model, revision.to_string())
        }
        None => Repo::new(repo_id.to_string(), RepoType::Model),
    });

    println!("[REGISTRY] Downloading config.json...");
    let config = repo.get("config.json").await?;

    println!("[REGISTRY] Downloading tokenizer.json...");
    let tokenizer = repo.get("tokenizer.json").await?;

    let weights = download_hf_weight_files(&repo).await?;
    let generation_config = repo.get("generation_config.json").await.ok();

    Ok(ModelPaths {
        weights,
        tokenizer,
        config,
        generation_config,
    })
}

async fn download_hf_weight_files(repo: &ApiRepo) -> Result<Vec<PathBuf>> {
    if let Ok(weights) = repo.get("model.safetensors").await {
        return Ok(vec![weights]);
    }

    println!("[REGISTRY] Downloading model.safetensors.index.json...");
    let index_path = repo.get("model.safetensors.index.json").await?;
    let shard_files = shard_files_from_index_path(&index_path)?;
    let mut weights = Vec::with_capacity(shard_files.len());
    for shard in shard_files {
        println!("[REGISTRY] Downloading {shard}...");
        weights.push(repo.get(&shard).await?);
    }
    Ok(weights)
}

fn shard_files_from_index_path(index_path: &Path) -> Result<Vec<String>> {
    let contents = fs::read_to_string(index_path)
        .with_context(|| format!("read safetensor index {}", index_path.display()))?;
    shard_files_from_index_contents(&contents)
}

fn shard_files_from_index_contents(contents: &str) -> Result<Vec<String>> {
    let index: SafetensorsIndex =
        serde_json::from_str(contents).context("parse safetensor index json")?;
    let shard_files: BTreeSet<String> = index.weight_map.into_values().collect();
    if shard_files.is_empty() {
        bail!("safetensor index does not reference any shard files");
    }
    Ok(shard_files.into_iter().collect())
}

#[async_trait]
impl ModelRegistry for HFHubAdapter {
    async fn get_model_paths(&self, model_id: &str) -> Result<ModelPaths, anyhow::Error> {
        println!("[REGISTRY] Resolving model: {}", model_id);

        let resolved = model_source_for(model_id)
            .ok()
            .unwrap_or_else(|| ModelSource::hugging_face(model_id.to_string()));
        let (repo_id, revision) = match resolved {
            ModelSource::HuggingFace { repo, revision } => (repo, revision),
            ModelSource::LocalPath(_) => {
                bail!("HFHubAdapter only supports Hugging Face model sources")
            }
        };

        download_hf_model_paths(&self.api, &repo_id, revision.as_deref()).await
    }
}

#[cfg(test)]
mod tests {
    use super::shard_files_from_index_contents;

    #[test]
    fn parses_unique_shard_files_from_safetensor_index() {
        let shards = shard_files_from_index_contents(
            r#"{
              "metadata": {"total_size": 42},
              "weight_map": {
                "a": "model-00002-of-00004.safetensors",
                "b": "model-00001-of-00004.safetensors",
                "c": "model-00002-of-00004.safetensors"
              }
            }"#,
        )
        .expect("parse shard index");

        assert_eq!(
            shards,
            vec![
                "model-00001-of-00004.safetensors".to_string(),
                "model-00002-of-00004.safetensors".to_string(),
            ]
        );
    }

    #[test]
    fn rejects_empty_safetensor_index() {
        let err = shard_files_from_index_contents(r#"{"weight_map": {}}"#)
            .expect_err("empty index should fail");
        assert!(
            err.to_string()
                .contains("safetensor index does not reference any shard files")
        );
    }
}
