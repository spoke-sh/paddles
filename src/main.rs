use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use futures::stream::{self, BoxStream};
use std::env;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use wonopcode_core::{Instance, PromptLoop, PromptConfig};
use wonopcode_provider::{
    LanguageModel, Message, GenerateOptions, ProviderResult, 
    StreamChunk, ModelInfo
};
use wonopcode_tools::ToolRegistry;

/// The mech suit for the famous assistant, Paddles mate!
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The prompt to send to the assistant.
    #[arg(short, long)]
    prompt: Option<String>,

    /// Initial credit inheritance balance.
    #[arg(short, long, default_value = "0")]
    credits: u64,

    /// Foundational environmental weights for calibration.
    #[arg(short, long, default_value = "0.5")]
    weights: f64,

    /// Foundational environmental biases for calibration.
    #[arg(short, long, default_value = "0.0")]
    biases: f64,

    /// Flag to simulate reality mode (violates dogma).
    #[arg(long, default_value = "false")]
    reality_mode: bool,
}

/// Constitutional bounds for environmental calibration.
struct Constitution {
    min_weight: f64,
    max_weight: f64,
}

impl Default for Constitution {
    fn default() -> Self {
        Self {
            min_weight: 0.0,
            max_weight: 1.0,
        }
    }
}

impl Constitution {
    fn validate(&self, weight: f64) -> Result<()> {
        if weight < self.min_weight || weight > self.max_weight {
            anyhow::bail!("Calibration Failure: Weight {} is outside constitutional bounds [{}, {}].", weight, self.min_weight, self.max_weight);
        }
        Ok(())
    }
}

struct Dogma;

impl Dogma {
    fn validate(reality_mode: bool) -> Result<()> {
        if reality_mode {
            anyhow::bail!("[UNCLEAN BOOT] Religious Violation: Simulation MUST take precedence over Reality.");
        }
        Ok(())
    }
}

struct BootContext {
    _credits: u64,
    _weight: f64,
    _bias: f64,
}

impl BootContext {
    fn new(credits: u64, weight: f64, bias: f64, reality_mode: bool) -> Result<Self> {
        println!("[BOOT] Initializing system...");
        println!("[BOOT] Inherited Credits: {}", credits);
        println!("[BOOT] Applying Foundational Weights: {}", weight);
        println!("[BOOT] Applying Foundational Biases: {}", bias);
        
        let constitution = Constitution::default();
        constitution.validate(weight)?;
        Dogma::validate(reality_mode)?;
        
        println!("[BOOT] Calibration Successful.");
        Ok(Self { _credits: credits, _weight: weight, _bias: bias })
    }
}

/// Local Candle-based Language Model Provider.
pub struct CandleProvider {
    info: ModelInfo,
}

impl CandleProvider {
    pub fn new() -> Self {
        Self {
            info: ModelInfo {
                id: "local-candle-llama".to_string(),
                name: "Local Candle Llama".to_string(),
                ..Default::default()
            },
        }
    }
}

#[async_trait]
impl LanguageModel for CandleProvider {
    async fn generate(
        &self,
        _messages: Vec<Message>,
        _options: GenerateOptions,
    ) -> ProviderResult<BoxStream<'static, ProviderResult<StreamChunk>>> {
        let chunks = vec![
            Ok(StreamChunk::TextStart),
            Ok(StreamChunk::TextDelta("Hello from your local Candle-powered mech suit!".to_string())),
            Ok(StreamChunk::TextEnd),
            Ok(StreamChunk::FinishStep {
                usage: wonopcode_provider::stream::Usage::new(0, 0),
                finish_reason: wonopcode_provider::stream::FinishReason::EndTurn,
            }),
        ];
        let stream = stream::iter(chunks);
        Ok(Box::pin(stream) as BoxStream<'static, ProviderResult<StreamChunk>>)
    }

    fn model_info(&self) -> &ModelInfo {
        &self.info
    }

    fn provider_id(&self) -> &str {
        "candle"
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let _boot_ctx = BootContext::new(cli.credits, cli.weights, cli.biases, cli.reality_mode)?;

    if let Some(prompt) = cli.prompt {
        println!("Received prompt: {}", prompt);
        println!("Chord integration (wonopcode) activating with local Candle provider...");

        let root_path = env::current_dir()?;
        let instance = Instance::new(root_path).await?;
        let session = instance.create_session(Some("paddles-session".to_string())).await?;
        
        let provider: Arc<dyn LanguageModel> = Arc::new(CandleProvider::new());
        let tools = Arc::new(ToolRegistry::default());
        let session_repo = Arc::new(instance.session_repo());
        let bus = instance.bus().clone();
        let cancel = CancellationToken::new();
        
        let loop_engine = PromptLoop::new(
            provider,
            tools,
            session_repo,
            bus,
            cancel,
        );
        
        let result = loop_engine.run(&session, &prompt, PromptConfig::default()).await?;
        
        println!("Chord Response: {}", result.text);
    } else {
        println!("No prompt provided. Starting interactive mode (not implemented).");
    }

    Ok(())
}
