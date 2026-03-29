use anyhow::Result;
use clap::Parser;
use std::env;
use std::io::IsTerminal;
use std::sync::Arc;
use tokio::io::{self as tokio_io, AsyncBufReadExt, BufReader};

// External Crate Modules
use paddles::application::{GathererProvider, MechSuitService, RuntimeLaneConfig};
use paddles::infrastructure::adapters::sift_registry::SiftRegistryAdapter;
use paddles::infrastructure::cli::interactive_tui::{
    InteractiveFrontend, run_interactive_tui, select_interactive_frontend,
};

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

    /// Model ID to use from the registry (e.g. qwen-1.5b, qwen-coder-1.5b).
    #[arg(short, long, default_value = "qwen-1.5b")]
    model: String,

    /// Optional planner model ID. Defaults to the synthesizer model when unset.
    #[arg(long)]
    planner_model: Option<String>,

    /// Optional model ID for the legacy static local context-gathering lane.
    #[arg(long)]
    gatherer_model: Option<String>,

    /// Provider to use for the default gatherer lane.
    #[arg(long, value_enum, default_value = "sift-autonomous")]
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
    let frontend = select_interactive_frontend(
        cli.prompt.is_some(),
        std::io::stdin().is_terminal(),
        std::io::stdout().is_terminal(),
    );

    // Initialize tracing based on verbosity
    let log_level = match frontend {
        InteractiveFrontend::Tui => tracing::Level::ERROR,
        InteractiveFrontend::PlainLines => match cli.verbose {
            0 => tracing::Level::ERROR,
            1 => tracing::Level::INFO,
            2 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        },
    };

    tracing_subscriber::fmt().with_max_level(log_level).init();

    let root_path = env::current_dir()?;
    let registry = Arc::new(SiftRegistryAdapter::new());
    let service = Arc::new(MechSuitService::new(root_path, registry));
    let runtime_verbose = match frontend {
        InteractiveFrontend::Tui => 0,
        InteractiveFrontend::PlainLines => cli.verbose,
    };
    service.set_verbose(runtime_verbose);

    // Boot sequence
    if cli.verbose >= 1 {
        println!("[BOOT] Initializing system...");
    }
    let boot_ctx = service.boot(
        cli.credits,
        cli.weights,
        cli.biases,
        cli.hf_token.clone(),
        cli.reality_mode,
    )?;

    if cli.verbose >= 1 {
        println!("[BOOT] Inherited Credits: {}", boot_ctx.credits);
        println!("[BOOT] Applying Foundational Weights: {}", boot_ctx.weight);
        println!("[BOOT] Applying Foundational Biases: {}", boot_ctx.bias);
        if boot_ctx.hf_token.is_some() {
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
        if let Some(planner_model) = &cli.planner_model {
            println!(
                "[BOOT] Syncing planner lane via SIFT for: {}...",
                planner_model
            );
        }
        if let Some(gatherer_model) = &cli.gatherer_model {
            println!(
                "[BOOT] Syncing gatherer lane via SIFT for: {}...",
                gatherer_model
            );
        }
        match cli.gatherer_provider {
            GathererProvider::SiftAutonomous => {
                println!("[BOOT] Sift autonomous gatherer provider selected.");
            }
            GathererProvider::Context1 => {
                println!(
                    "[BOOT] Context-1 gatherer provider selected (harness ready: {}).",
                    cli.context1_harness_ready
                );
            }
            GathererProvider::Local => {}
        }
    }
    let runtime_lanes = RuntimeLaneConfig::new(cli.model.clone(), cli.gatherer_model.clone())
        .with_planner_model_id(cli.planner_model.clone())
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
        match frontend {
            InteractiveFrontend::Tui => run_interactive_tui(service, cli.model.clone()).await?,
            InteractiveFrontend::PlainLines => run_plain_interactive_loop(service).await?,
        }
    }

    Ok(())
}

async fn run_plain_interactive_loop(service: Arc<MechSuitService>) -> Result<()> {
    println!("--- Interactive Mode (type 'exit' or use Ctrl+C to quit) ---");
    let mut stdin_reader = BufReader::new(tokio_io::stdin()).lines();
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

    Ok(())
}
