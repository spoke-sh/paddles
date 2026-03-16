use anyhow::Result;
use clap::Parser;

/// The mech suit for the famous assistant, Paddles mate!
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The prompt to send to the assistant.
    #[arg(short, long)]
    prompt: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    if let Some(prompt) = cli.prompt {
        println!("Received prompt: {}", prompt);
        println!("Chord integration is currently being wired up...");
        // Placeholder for Chord engine execution
        // let engine = wonopcode::Engine::new();
        // engine.run(prompt).await?;
    } else {
        println!("No prompt provided. Starting interactive mode (not implemented).");
    }

    Ok(())
}
