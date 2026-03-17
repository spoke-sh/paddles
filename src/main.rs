use anyhow::Result;
use clap::Parser;
use std::env;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use wonopcode_core::Instance;

// External Crate Modules
use paddles::application::MechSuitService;
use paddles::infrastructure::adapters::candle::CandleAdapter;

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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let root_path = env::current_dir()?;
    let instance = Instance::new(root_path).await?;
    let engine = Arc::new(CandleAdapter::new());
    let service = MechSuitService::new(instance, engine);

    // Boot sequence
    println!("[BOOT] Initializing system...");
    let _boot_ctx = service.boot(cli.credits, cli.weights, cli.biases, cli.reality_mode)?;
    println!("[BOOT] Inherited Credits: {}", _boot_ctx.credits);
    println!("[BOOT] Applying Foundational Weights: {}", _boot_ctx.weight);
    println!("[BOOT] Applying Foundational Biases: {}", _boot_ctx.bias);
    println!("[BOOT] Calibration Successful.");

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
