use crate::domain::ports::{ModelPaths, ModelRegistry};
use anyhow::{Context, Result, anyhow, bail};
use async_trait::async_trait;
use serde::Deserialize;
use sift::internal::cache::cache_dir;
use sift::internal::search::adapters::llm_utils::ensure_hf_asset;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub struct SiftRegistryAdapter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QwenModelFamily {
    Qwen2,
    Qwen3,
    Qwen3_5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QwenWeightLayout {
    Single(&'static str),
    Indexed(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QwenModelSpec {
    pub model_id: &'static str,
    pub revision: &'static str,
    pub max_length: usize,
    pub family: QwenModelFamily,
    pub weights: QwenWeightLayout,
}

impl QwenModelSpec {
    pub fn root(self) -> Result<PathBuf> {
        Ok(cache_dir("models")?
            .join(Path::new(self.model_id))
            .join(Path::new(self.revision)))
    }

    pub fn config_path(self) -> Result<PathBuf> {
        Ok(self.root()?.join("config.json"))
    }

    pub fn tokenizer_path(self) -> Result<PathBuf> {
        Ok(self.root()?.join("tokenizer.json"))
    }

    pub fn primary_weights_path(self) -> Result<PathBuf> {
        let root = self.root()?;
        Ok(match self.weights {
            QwenWeightLayout::Single(file_name) => root.join(file_name),
            QwenWeightLayout::Indexed(index_name) => root.join(index_name),
        })
    }
}

const SUPPORTED_QWEN_MODELS: &[&str] = &[
    "qwen-1.5b",
    "qwen-coder-3b",
    "qwen3.5-2b",
    "Qwen/Qwen2.5-1.5B-Instruct",
    "Qwen/Qwen2.5-Coder-3B-Instruct",
    "Qwen/Qwen3.5-2B",
];

pub fn qwen_spec_for(model_id: &str) -> Result<QwenModelSpec> {
    let spec = match model_id {
        "qwen-1.5b" | "Qwen/Qwen2.5-1.5B-Instruct" => QwenModelSpec {
            model_id: "Qwen/Qwen2.5-1.5B-Instruct",
            revision: "main",
            max_length: 512,
            family: QwenModelFamily::Qwen2,
            weights: QwenWeightLayout::Single("model.safetensors"),
        },
        "qwen-coder-3b" | "Qwen/Qwen2.5-Coder-3B-Instruct" => QwenModelSpec {
            model_id: "Qwen/Qwen2.5-Coder-3B-Instruct",
            revision: "main",
            max_length: 512,
            family: QwenModelFamily::Qwen2,
            weights: QwenWeightLayout::Indexed("model.safetensors.index.json"),
        },
        "qwen3.5-2b" | "Qwen/Qwen3.5-2B" => QwenModelSpec {
            model_id: "Qwen/Qwen3.5-2B",
            revision: "main",
            max_length: 512,
            family: QwenModelFamily::Qwen3_5,
            weights: QwenWeightLayout::Indexed("model.safetensors.index.json"),
        },
        _ => {
            bail!(
                "unsupported model id '{model_id}'. supported ids: {}",
                SUPPORTED_QWEN_MODELS.join(", ")
            )
        }
    };

    Ok(spec)
}

pub fn qwen_weight_paths(spec: QwenModelSpec) -> Result<Vec<PathBuf>> {
    let root = spec.root()?;
    match spec.weights {
        QwenWeightLayout::Single(file_name) => Ok(vec![root.join(file_name)]),
        QwenWeightLayout::Indexed(index_name) => indexed_weight_paths(&root.join(index_name))
            .with_context(|| format!("failed to resolve weight shards for {}", spec.model_id)),
    }
}

pub fn ensure_qwen_assets(spec: QwenModelSpec) -> Result<ModelPaths> {
    let root = spec.root()?;
    let config_path = root.join("config.json");
    let tokenizer_path = root.join("tokenizer.json");
    let weights_path = spec.primary_weights_path()?;

    println!("[SIFT] Ensuring assets for {}...", spec.model_id);

    ensure_hf_asset(spec.model_id, spec.revision, &config_path, "config.json")?;
    ensure_hf_asset(
        spec.model_id,
        spec.revision,
        &tokenizer_path,
        "tokenizer.json",
    )?;

    match spec.weights {
        QwenWeightLayout::Single(file_name) => {
            ensure_hf_asset(spec.model_id, spec.revision, &weights_path, file_name)?;
        }
        QwenWeightLayout::Indexed(index_name) => {
            ensure_hf_asset(spec.model_id, spec.revision, &weights_path, index_name)?;
            for shard_name in shard_names_from_index_path(&weights_path)? {
                ensure_hf_asset(
                    spec.model_id,
                    spec.revision,
                    &root.join(&shard_name),
                    &shard_name,
                )?;
            }
        }
    }

    Ok(ModelPaths {
        weights: weights_path,
        tokenizer: tokenizer_path,
        config: config_path,
    })
}

#[derive(Debug, Deserialize)]
struct SafetensorsIndex {
    weight_map: std::collections::HashMap<String, String>,
}

fn indexed_weight_paths(index_path: &Path) -> Result<Vec<PathBuf>> {
    let root = index_path.parent().ok_or_else(|| {
        anyhow!(
            "missing parent directory for index file {}",
            index_path.display()
        )
    })?;

    Ok(shard_names_from_index_path(index_path)?
        .into_iter()
        .map(|shard_name| root.join(shard_name))
        .collect())
}

fn shard_names_from_index_path(index_path: &Path) -> Result<Vec<String>> {
    let index: SafetensorsIndex = serde_json::from_str(&fs::read_to_string(index_path)?)
        .with_context(|| format!("failed to parse {}", index_path.display()))?;

    let shards = index
        .weight_map
        .into_values()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    if shards.is_empty() {
        bail!("no weight shards listed in {}", index_path.display());
    }

    Ok(shards)
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
    async fn get_model_paths(&self, model_id: &str) -> Result<ModelPaths> {
        println!("[SIFT] Resolving model: {}", model_id);
        ensure_qwen_assets(qwen_spec_for(model_id)?)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        QwenModelFamily, QwenWeightLayout, indexed_weight_paths, qwen_spec_for,
        shard_names_from_index_path,
    };
    use std::fs;

    #[test]
    fn supports_new_runtime_aliases() {
        let spec = qwen_spec_for("qwen-coder-3b").expect("coder spec");
        assert_eq!(spec.model_id, "Qwen/Qwen2.5-Coder-3B-Instruct");
        assert_eq!(spec.family, QwenModelFamily::Qwen2);
        assert_eq!(
            spec.weights,
            QwenWeightLayout::Indexed("model.safetensors.index.json")
        );

        let spec = qwen_spec_for("qwen3.5-2b").expect("qwen3.5 spec");
        assert_eq!(spec.model_id, "Qwen/Qwen3.5-2B");
        assert_eq!(spec.family, QwenModelFamily::Qwen3_5);
    }

    #[test]
    fn rejects_unknown_model_ids() {
        let err = qwen_spec_for("mystery-model").expect_err("unknown model should fail");
        assert!(err.to_string().contains("unsupported model id"));
    }

    #[test]
    fn parses_sharded_weight_indexes_in_stable_order() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let index_path = workspace.path().join("model.safetensors.index.json");
        fs::write(
            &index_path,
            r#"{
  "weight_map": {
    "layers.0.weight": "model-00002-of-00002.safetensors",
    "layers.1.weight": "model-00001-of-00002.safetensors",
    "layers.2.weight": "model-00002-of-00002.safetensors"
  }
}"#,
        )
        .expect("write index");

        let shard_names = shard_names_from_index_path(&index_path).expect("shard names");
        assert_eq!(
            shard_names,
            vec![
                "model-00001-of-00002.safetensors".to_string(),
                "model-00002-of-00002.safetensors".to_string()
            ]
        );

        let shard_paths = indexed_weight_paths(&index_path).expect("shard paths");
        assert_eq!(shard_paths.len(), 2);
        assert_eq!(
            shard_paths[0].file_name().and_then(|name| name.to_str()),
            Some("model-00001-of-00002.safetensors")
        );
        assert_eq!(
            shard_paths[1].file_name().and_then(|name| name.to_str()),
            Some("model-00002-of-00002.safetensors")
        );
    }
}
