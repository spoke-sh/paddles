use anyhow::Result;
use clap::Parser;
use std::env;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use wonopcode_core::Instance;

// External Crate Modules
use paddles::application::MechSuitService;
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

    /// Model ID to use from the registry (e.g. gemma-2b, qwen-1.5b).
    #[arg(short, long, default_value = "qwen-1.5b")]
    model: String,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Hugging Face API token for gated models.
    #[arg(long, env = "HF_TOKEN")]
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
    let instance = Instance::new(root_path).await?;

    let registry = Arc::new(SiftRegistryAdapter::new());
    let service = MechSuitService::new(instance, registry);
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
        println!("[BOOT] Syncing model assets via SIFT for: {}...", cli.model);
    }
    let paths = service.prepare_model(&cli.model).await?;
    if cli.verbose >= 1 {
        println!("[BOOT] Registry Sync Complete.");
    }

    if let Some(prompt) = cli.prompt {
        let response = service.process_prompt(&prompt, paths).await?;
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

                let response = service.process_prompt(input, paths.clone()).await?;
                println!("Chord Response: {}", response);
            } else {
                break;
            }
        }
    }

    Ok(())
}
