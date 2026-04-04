use crate::domain::ports::{ModelPaths, ModelRegistry};
use anyhow::{Result, bail};
use async_trait::async_trait;
use sift::{ModelRuntimeContract, ModelSource, prepare_model};

pub struct SiftRegistryAdapter;

const BONSAI_GGUF_REPO: &str = "prism-ml/Bonsai-8B-gguf";

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
            ModelSource::hugging_face_revision(BONSAI_GGUF_REPO, "main")
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
    use super::{BONSAI_GGUF_REPO, model_source_for, supported_model_ids};
    use crate::infrastructure::sift_cache::TEST_SIFT_ENV_LOCK;
    use candle_core::Tensor;
    use candle_core::quantized::gguf_file;
    use candle_core::quantized::{GgmlDType, QTensor};
    use sift::{ModelPreparationMode, ModelRuntimeContract, ModelSource, prepare_model};
    use std::fs::{self, File};
    use std::io::{BufWriter, Write};
    use std::path::Path;
    use tempfile::tempdir;

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
            ModelSource::hugging_face_revision(BONSAI_GGUF_REPO, "main")
        );
        assert_eq!(
            model_source_for("prism-ml/Bonsai-8B-gguf").expect("legacy bonsai source"),
            ModelSource::hugging_face_revision(BONSAI_GGUF_REPO, "main")
        );
        assert_eq!(
            model_source_for("prism-ml/Bonsai-8B-unpacked").expect("legacy unpacked source"),
            ModelSource::hugging_face_revision(BONSAI_GGUF_REPO, "main")
        );
    }

    #[test]
    fn rejects_unknown_model_ids() {
        let err = model_source_for("mystery-model").expect_err("unknown model should fail");
        assert!(err.to_string().contains("unsupported model id"));
    }

    #[test]
    fn prepares_bonsai_alias_through_sift_gguf_compatibility_path() {
        let _env_guard = TEST_SIFT_ENV_LOCK.lock().expect("env lock");
        let temp = tempdir().expect("tempdir");
        let sift_cache = temp.path().join("sift-cache");
        let metamorph_cache = temp.path().join("metamorph-cache");
        let mock_root = temp.path().join("mock");

        write_mock_remote_gguf_repo(
            &mock_root,
            "prism-ml/Bonsai-8B-gguf",
            "main",
            "Bonsai-8B-Q4.gguf",
            Some("sha-main-001"),
        );

        unsafe {
            std::env::set_var("SIFT_CACHE", &sift_cache);
            std::env::set_var("METAMORPH_CACHE_DIR", &metamorph_cache);
            std::env::set_var("METAMORPH_HF_MOCK_ROOT", &mock_root);
        }

        let prepared = prepare_model(
            model_source_for("bonsai-8b").expect("bonsai source"),
            ModelRuntimeContract::CandleSafetensorsBundle,
        )
        .expect("prepare bonsai gguf source");

        unsafe {
            std::env::remove_var("SIFT_CACHE");
            std::env::remove_var("METAMORPH_CACHE_DIR");
            std::env::remove_var("METAMORPH_HF_MOCK_ROOT");
        }

        assert_eq!(
            prepared.source,
            ModelSource::hugging_face_revision("prism-ml/Bonsai-8B-gguf", "main")
        );
        assert_eq!(prepared.preparation_mode, ModelPreparationMode::Converted);
        assert!(prepared.lossy);
        assert!(
            prepared
                .notes
                .iter()
                .any(|note| note.contains("compatibility path"))
        );
    }

    fn write_mock_remote_gguf_repo(
        root: &Path,
        repo: &str,
        revision: &str,
        artifact_name: &str,
        resolved_revision: Option<&str>,
    ) {
        let repo_root = root.join(repo).join(revision);
        fs::create_dir_all(&repo_root).expect("create repo root");
        write_fixture_gguf(&repo_root.join(artifact_name));

        if let Some(resolved_revision) = resolved_revision {
            fs::write(
                repo_root.join(".metamorph-hf.json"),
                serde_json::to_vec_pretty(&serde_json::json!({
                    "resolved_revision": resolved_revision
                }))
                .expect("serialize mock hf config"),
            )
            .expect("write mock hf config");
        }
    }

    fn write_fixture_gguf(path: &Path) {
        let device = candle_core::Device::Cpu;
        let tensor = Tensor::from_vec(vec![0f32, 1.0, 2.0, 3.0], (2, 2), &device).expect("tensor");
        let qtensor = QTensor::quantize(&tensor, GgmlDType::F32).expect("qtensor");

        let metadata = vec![
            (
                "general.architecture",
                gguf_file::Value::String("llama".to_owned()),
            ),
            ("llama.context_length", gguf_file::Value::U32(64)),
            ("llama.embedding_length", gguf_file::Value::U32(32)),
            ("llama.block_count", gguf_file::Value::U32(1)),
            ("llama.feed_forward_length", gguf_file::Value::U32(64)),
            ("llama.attention.head_count", gguf_file::Value::U32(2)),
            ("llama.attention.head_count_kv", gguf_file::Value::U32(2)),
            ("llama.rope.freq_base", gguf_file::Value::F32(10000.0)),
            (
                "llama.attention.layer_norm_rms_epsilon",
                gguf_file::Value::F32(0.00001),
            ),
            (
                "tokenizer.ggml.model",
                gguf_file::Value::String("gpt2".to_owned()),
            ),
            (
                "tokenizer.ggml.pre",
                gguf_file::Value::String("gpt2".to_owned()),
            ),
            (
                "tokenizer.ggml.tokens",
                gguf_file::Value::Array(vec![
                    gguf_file::Value::String("<unk>".to_owned()),
                    gguf_file::Value::String("a".to_owned()),
                    gguf_file::Value::String("b".to_owned()),
                    gguf_file::Value::String("ab".to_owned()),
                ]),
            ),
            (
                "tokenizer.ggml.merges",
                gguf_file::Value::Array(vec![gguf_file::Value::String("a b".to_owned())]),
            ),
            ("tokenizer.ggml.unk_token_id", gguf_file::Value::U32(0)),
            ("tokenizer.ggml.bos_token_id", gguf_file::Value::U32(1)),
            ("tokenizer.ggml.eos_token_id", gguf_file::Value::U32(2)),
            (
                "tokenizer.ggml.add_bos_token",
                gguf_file::Value::Bool(false),
            ),
            (
                "tokenizer.ggml.add_eos_token",
                gguf_file::Value::Bool(false),
            ),
        ];
        let metadata_refs = metadata
            .iter()
            .map(|(name, value)| (*name, value))
            .collect::<Vec<_>>();

        let tensors = [("tok_embeddings.weight", qtensor)];
        let tensor_refs = tensors
            .iter()
            .map(|(name, tensor)| (*name, tensor))
            .collect::<Vec<_>>();

        let mut writer = BufWriter::new(File::create(path).expect("create gguf"));
        gguf_file::write(&mut writer, &metadata_refs, &tensor_refs).expect("write gguf");
        writer.flush().expect("flush gguf");
    }
}
