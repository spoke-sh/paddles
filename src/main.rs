use anyhow::Result;
use clap::Parser;
use std::env;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};

// External Crate Modules
use paddles::application::{GathererProvider, MechSuitService, RuntimeLaneConfig};
use paddles::infrastructure::adapters::sift_registry::SiftRegistryAdapter;

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

    /// Model ID to use from the registry (e.g. qwen3.5-2b, qwen-coder-3b).
    #[arg(short, long, default_value = "qwen3.5-2b")]
    model: String,

    /// Optional model ID for a dedicated context-gathering lane.
    #[arg(long)]
    gatherer_model: Option<String>,

    /// Provider to use for the optional gatherer lane.
    #[arg(long, value_enum, default_value = "local")]
    gatherer_provider: GathererProvider,

    /// Acknowledge that the external Context-1 harness is actually available.
    #[arg(long, default_value_t = false)]
    context1_harness_ready: bool,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Hugging Face API token for gated models.
    #[arg(long, env = "HF_TOKEN", hide_env_values = true)]
    hf_token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing based on verbosity
    let log_level = match cli.verbose {
        0 => tracing::Level::ERROR,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::fmt().with_max_level(log_level).init();

    let root_path = env::current_dir()?;
    let registry = Arc::new(SiftRegistryAdapter::new());
    let service = MechSuitService::new(root_path, registry);
    service.set_verbose(cli.verbose);

    // Boot sequence
    if cli.verbose >= 1 {
        println!("[BOOT] Initializing system...");
    }
    let _boot_ctx = service.boot(
        cli.credits,
        cli.weights,
        cli.biases,
        cli.hf_token.clone(),
        cli.reality_mode,
    )?;

    if cli.verbose >= 1 {
        println!("[BOOT] Inherited Credits: {}", _boot_ctx.credits);
        println!("[BOOT] Applying Foundational Weights: {}", _boot_ctx.weight);
        println!("[BOOT] Applying Foundational Biases: {}", _boot_ctx.bias);
        if _boot_ctx.hf_token.is_some() {
            println!("[BOOT] Hugging Face Token: [MASKED]");
        }
        println!("[BOOT] Calibration Successful.");
    }

    // Registry Synchronization via Sift
    if cli.verbose >= 1 {
        println!(
            "[BOOT] Syncing synthesizer lane via SIFT for: {}...",
            cli.model
        );
        if let Some(gatherer_model) = &cli.gatherer_model {
            println!(
                "[BOOT] Syncing gatherer lane via SIFT for: {}...",
                gatherer_model
            );
        }
        if matches!(cli.gatherer_provider, GathererProvider::Context1) {
            println!(
                "[BOOT] Context-1 gatherer provider selected (harness ready: {}).",
                cli.context1_harness_ready
            );
        }
    }
    let runtime_lanes = RuntimeLaneConfig::new(cli.model.clone(), cli.gatherer_model.clone())
        .with_gatherer_provider(cli.gatherer_provider)
        .with_context1_harness_ready(cli.context1_harness_ready);
    let _prepared_lanes = service.prepare_runtime_lanes(&runtime_lanes).await?;
    if cli.verbose >= 1 {
        println!("[BOOT] Runtime lanes ready.");
    }

    if let Some(prompt) = cli.prompt {
        let response = service.process_prompt(&prompt).await?;
        println!("Chord Response: {}", response);
    } else {
        println!("--- Interactive Mode (type 'exit' or use Ctrl+C to quit) ---");
        let mut stdin_reader = BufReader::new(io::stdin()).lines();
        loop {
            print!(">> ");
            use std::io::Write;
            std::io::stdout().flush()?;

            if let Some(line) = stdin_reader.next_line().await? {
                let input = line.trim();
                if input == "exit" || input == "quit" {
                    break;
                }
                if input.is_empty() {
                    continue;
                }

                let response = service.process_prompt(input).await?;
                println!("Chord Response: {}", response);
            } else {
                break;
            }
        }
    }

    Ok(())
}
