use crate::domain::ports::{ModelPaths, ModelRegistry};
use crate::infrastructure::adapters::hf_hub::{build_hf_api_from_env, download_hf_model_paths};
use anyhow::{Result, bail};
use async_trait::async_trait;
use sift::{ModelRuntimeContract, ModelSource, prepare_model};

pub struct SiftRegistryAdapter;

const BONSAI_UNPACKED_REPO: &str = "prism-ml/Bonsai-8B-unpacked";

const SUPPORTED_SIFT_MODELS: &[&str] = &[
    "qwen-1.5b",
    "qwen-coder-0.5b",
    "qwen-coder-1.5b",
    "qwen-coder-3b",
    "qwen3.5-2b",
    "bonsai-8b",
    "Qwen/Qwen2.5-1.5B-Instruct",
    "Qwen/Qwen2.5-Coder-0.5B-Instruct",
    "Qwen/Qwen2.5-Coder-1.5B-Instruct",
    "Qwen/Qwen2.5-Coder-3B-Instruct",
    "Qwen/Qwen3.5-2B",
    "prism-ml/Bonsai-8B-unpacked",
    "prism-ml/Bonsai-8B-gguf",
];

pub fn supported_model_ids() -> &'static [&'static str] {
    SUPPORTED_SIFT_MODELS
}

pub fn model_source_for(model_id: &str) -> Result<ModelSource> {
    let source = match model_id {
        "qwen-1.5b" | "Qwen/Qwen2.5-1.5B-Instruct" => {
            ModelSource::hugging_face_revision("Qwen/Qwen2.5-1.5B-Instruct", "main")
        }
        "qwen-coder-0.5b" | "Qwen/Qwen2.5-Coder-0.5B-Instruct" => {
            ModelSource::hugging_face_revision("Qwen/Qwen2.5-Coder-0.5B-Instruct", "main")
        }
        "qwen-coder-1.5b" | "Qwen/Qwen2.5-Coder-1.5B-Instruct" => {
            ModelSource::hugging_face_revision("Qwen/Qwen2.5-Coder-1.5B-Instruct", "main")
        }
        "qwen-coder-3b" | "Qwen/Qwen2.5-Coder-3B-Instruct" => {
            ModelSource::hugging_face_revision("Qwen/Qwen2.5-Coder-3B-Instruct", "main")
        }
        "qwen3.5-2b" | "Qwen/Qwen3.5-2B" => {
            ModelSource::hugging_face_revision("Qwen/Qwen3.5-2B", "main")
        }
        "bonsai-8b" | "prism-ml/Bonsai-8B-unpacked" | "prism-ml/Bonsai-8B-gguf" => {
            ModelSource::hugging_face_revision(BONSAI_UNPACKED_REPO, "main")
        }
        _ => {
            bail!(
                "unsupported model id '{model_id}'. supported ids: {}",
                SUPPORTED_SIFT_MODELS.join(", ")
            )
        }
    };

    Ok(source)
}

fn prepared_model_to_paths(prepared: sift::PreparedModel) -> ModelPaths {
    ModelPaths {
        weights: vec![prepared.weights],
        tokenizer: prepared.tokenizer,
        config: prepared.config,
        generation_config: prepared.generation_config,
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
    async fn get_model_paths(&self, model_id: &str) -> Result<ModelPaths> {
        if matches!(
            model_id,
            "bonsai-8b" | "prism-ml/Bonsai-8B-unpacked" | "prism-ml/Bonsai-8B-gguf"
        ) {
            let api = build_hf_api_from_env()?;
            return download_hf_model_paths(&api, BONSAI_UNPACKED_REPO, Some("main")).await;
        }

        let model_id = model_id.to_string();
        tokio::task::spawn_blocking(move || {
            println!("[SIFT] Preparing model: {}", model_id);
            let source = model_source_for(&model_id)?;
            let prepared = prepare_model(source, ModelRuntimeContract::CandleSafetensorsBundle)?;
            Ok(prepared_model_to_paths(prepared))
        })
        .await?
    }
}

#[cfg(test)]
mod tests {
    use super::{BONSAI_UNPACKED_REPO, model_source_for, supported_model_ids};
    use sift::ModelSource;

    #[test]
    fn supports_runtime_aliases_and_bonsai() {
        assert!(supported_model_ids().contains(&"qwen3.5-2b"));
        assert!(supported_model_ids().contains(&"bonsai-8b"));
        assert!(supported_model_ids().contains(&"prism-ml/Bonsai-8B-unpacked"));

        assert_eq!(
            model_source_for("qwen-coder-3b").expect("coder source"),
            ModelSource::hugging_face_revision("Qwen/Qwen2.5-Coder-3B-Instruct", "main")
        );
        assert_eq!(
            model_source_for("bonsai-8b").expect("bonsai source"),
            ModelSource::hugging_face_revision(BONSAI_UNPACKED_REPO, "main")
        );
        assert_eq!(
            model_source_for("prism-ml/Bonsai-8B-gguf").expect("legacy bonsai source"),
            ModelSource::hugging_face_revision(BONSAI_UNPACKED_REPO, "main")
        );
    }

    #[test]
    fn rejects_unknown_model_ids() {
        let err = model_source_for("mystery-model").expect_err("unknown model should fail");
        assert!(err.to_string().contains("unsupported model id"));
    }
}
