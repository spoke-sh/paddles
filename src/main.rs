use anyhow::{Result, bail};
use clap::Parser;
use std::env;
use std::io::IsTerminal;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{self as tokio_io, AsyncBufReadExt, BufReader};

use paddles::application::{
    GathererProvider, MechSuitService, PreparedGathererLane, PreparedModelLane, RuntimeLaneConfig,
};
use paddles::domain::ports::{ContextGatherer, SynthesizerEngine};
use paddles::infrastructure::adapters::agent_memory::AgentMemory;
use paddles::infrastructure::adapters::context1_gatherer::Context1GathererAdapter;
use paddles::infrastructure::adapters::http_provider::{
    ApiFormat, HttpPlannerAdapter, HttpProviderAdapter,
};
use paddles::infrastructure::adapters::sift_agent::SiftAgentAdapter;
use paddles::infrastructure::adapters::sift_context_gatherer::SiftContextGathererAdapter;
use paddles::infrastructure::adapters::sift_direct_gatherer::SiftDirectGathererAdapter;
use paddles::infrastructure::adapters::sift_planner::SiftPlannerAdapter;
use paddles::infrastructure::adapters::sift_registry::SiftRegistryAdapter;
use paddles::infrastructure::cli::interactive_tui::{
    InteractiveFrontend, TuiContext, run_interactive_tui, select_interactive_frontend,
};
use paddles::infrastructure::config::{
    PaddlesConfig, normalize_gatherer_provider_alias, normalize_provider_model_alias,
};
use paddles::infrastructure::credentials::CredentialStore;
use paddles::infrastructure::providers::ModelProvider;
use paddles::infrastructure::rendering::RenderCapability;
use paddles::infrastructure::runtime_preferences::RuntimeLanePreferenceStore;

/// The mech suit for the famous assistant, Paddles mate!
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The prompt to send to the assistant.
    #[arg(short, long)]
    prompt: Option<String>,

    /// Initial credit inheritance balance.
    #[arg(short, long)]
    credits: Option<u64>,

    /// Foundational environmental weights for calibration.
    #[arg(short, long)]
    weights: Option<f64>,

    /// Foundational environmental biases for calibration.
    #[arg(short, long)]
    biases: Option<f64>,

    /// Flag to simulate reality mode (violates dogma).
    #[arg(long)]
    reality_mode: bool,

    /// Model provider backend.
    #[arg(long, value_enum)]
    provider: Option<ModelProvider>,

    /// Custom API base URL (overrides provider default).
    #[arg(long)]
    provider_url: Option<String>,

    /// Model ID to use (e.g. qwen-1.5b, gpt-4o, claude-sonnet-4-20250514).
    #[arg(short, long)]
    model: Option<String>,

    /// Optional planner model ID. Defaults to the synthesizer model when unset.
    #[arg(long)]
    planner_model: Option<String>,

    /// Optional planner provider. Defaults to the synthesizer provider when unset.
    #[arg(long, value_enum)]
    planner_provider: Option<ModelProvider>,

    /// Optional model ID for the legacy static local context-gathering lane.
    #[arg(long)]
    gatherer_model: Option<String>,

    /// Provider to use for the default gatherer lane.
    #[arg(long, value_enum)]
    gatherer_provider: Option<GathererProvider>,

    /// Acknowledge that the external Context-1 harness is actually available.
    #[arg(long)]
    context1_harness_ready: bool,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Port for the HTTP API server.
    #[arg(long)]
    port: Option<u16>,

    /// Hugging Face API token for gated models.
    #[arg(long, env = "HF_TOKEN", hide_env_values = true)]
    hf_token: Option<String>,
}

fn resolve_provider_from_name(configured: &str, field_name: &str) -> ModelProvider {
    match ModelProvider::from_name(configured) {
        Some(provider) => provider,
        None => {
            eprintln!("[WARN] Unsupported {field_name} `{configured}`; using `sift` instead.");
            ModelProvider::Sift
        }
    }
}

fn provider_api_format(provider: ModelProvider) -> Option<ApiFormat> {
    match provider {
        ModelProvider::Openai
        | ModelProvider::Inception
        | ModelProvider::Moonshot
        | ModelProvider::Ollama => Some(ApiFormat::OpenAi),
        ModelProvider::Anthropic => Some(ApiFormat::Anthropic),
        ModelProvider::Google => Some(ApiFormat::Gemini),
        ModelProvider::Sift => None,
    }
}

fn resolve_remote_provider_config(
    provider: ModelProvider,
    model_id: &str,
    credential_store: &CredentialStore,
    provider_url_overrides: &std::collections::BTreeMap<ModelProvider, String>,
) -> Result<(ApiFormat, String, String, RenderCapability)> {
    let resolved_api_key = credential_store.resolve_provider_api_key(provider);
    if provider.auth_requirement()
        == paddles::infrastructure::providers::ProviderAuthRequirement::RequiredApiKey
        && resolved_api_key.value.is_empty()
    {
        let env_var = provider.credential_env_var().unwrap_or("PROVIDER_API_KEY");
        bail!(
            "provider `{}` is not authenticated; set `{}` or use `/login {}` before selecting `{}`",
            provider.name(),
            env_var,
            provider.name(),
            provider.qualified_model_label(model_id)
        );
    }

    let api_format = provider_api_format(provider).ok_or_else(|| {
        anyhow::anyhow!(
            "provider `{}` does not use the HTTP adapter",
            provider.name()
        )
    })?;
    let base_url = provider_url_overrides
        .get(&provider)
        .cloned()
        .or_else(|| provider.default_base_url().map(str::to_string))
        .ok_or_else(|| {
            anyhow::anyhow!("provider `{}` does not define a base URL", provider.name())
        })?;
    let render_capability = RenderCapability::resolve(provider.name(), model_id);
    Ok((
        api_format,
        base_url,
        resolved_api_key.value,
        render_capability,
    ))
}

fn build_synthesizer_engine(
    workspace: &Path,
    lane: &PreparedModelLane,
    credential_store: &CredentialStore,
    provider_url_overrides: &std::collections::BTreeMap<ModelProvider, String>,
) -> Result<Arc<dyn SynthesizerEngine>> {
    match lane.provider {
        ModelProvider::Sift => Ok(Arc::new(SiftAgentAdapter::new(
            workspace.to_path_buf(),
            &lane.model_id,
            RenderCapability::resolve(lane.provider.name(), &lane.model_id),
        )?) as Arc<dyn SynthesizerEngine>),
        provider => {
            let (format, base_url, api_key, render_capability) = resolve_remote_provider_config(
                provider,
                &lane.model_id,
                credential_store,
                provider_url_overrides,
            )?;
            Ok(Arc::new(HttpProviderAdapter::new(
                workspace.to_path_buf(),
                provider.name(),
                lane.model_id.clone(),
                api_key,
                base_url,
                format,
                render_capability,
            )) as Arc<dyn SynthesizerEngine>)
        }
    }
}

fn build_planner_engine(
    workspace: &Path,
    lane: &PreparedModelLane,
    credential_store: &CredentialStore,
    provider_url_overrides: &std::collections::BTreeMap<ModelProvider, String>,
) -> Result<Arc<dyn paddles::domain::ports::RecursivePlanner>> {
    match lane.provider {
        ModelProvider::Sift => {
            let engine = Arc::new(SiftAgentAdapter::new(
                workspace.to_path_buf(),
                &lane.model_id,
                RenderCapability::resolve(lane.provider.name(), &lane.model_id),
            )?);
            Ok(Arc::new(SiftPlannerAdapter::new(engine))
                as Arc<dyn paddles::domain::ports::RecursivePlanner>)
        }
        provider => {
            let (format, base_url, api_key, render_capability) = resolve_remote_provider_config(
                provider,
                &lane.model_id,
                credential_store,
                provider_url_overrides,
            )?;
            let engine = Arc::new(HttpProviderAdapter::new(
                workspace.to_path_buf(),
                provider.name(),
                lane.model_id.clone(),
                api_key,
                base_url,
                format,
                render_capability,
            ));
            Ok(Arc::new(HttpPlannerAdapter::new(engine))
                as Arc<dyn paddles::domain::ports::RecursivePlanner>)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let root_path = env::current_dir()?;
    let runtime_preference_store = Arc::new(RuntimeLanePreferenceStore::new());
    let runtime_preferences = match runtime_preference_store.load() {
        Ok(preferences) => preferences,
        Err(err) => {
            eprintln!("[WARN] Failed to load runtime lane preferences: {err:#}");
            None
        }
    };

    // Load layered config: system -> user -> workspace -> runtime lane state.
    let config =
        PaddlesConfig::load_with_runtime_preferences(&root_path, runtime_preferences.as_ref());

    // Merge: CLI flags override config values
    let mut model = cli.model.unwrap_or(config.model);
    let credits = cli.credits.unwrap_or(config.credits);
    let weights = cli.weights.unwrap_or(config.weights);
    let biases = cli.biases.unwrap_or(config.biases);
    let reality_mode = cli.reality_mode || config.reality_mode;
    let port = cli.port.unwrap_or(config.port);
    let verbose = if cli.verbose > 0 {
        cli.verbose
    } else {
        config.verbose
    };
    let hf_token = cli.hf_token.or(config.hf_token);
    let context1_harness_ready = cli.context1_harness_ready || config.context1_harness_ready;
    let mut planner_model = cli.planner_model.or(config.planner_model);
    let planner_provider = cli.planner_provider.or_else(|| {
        config
            .planner_provider
            .as_deref()
            .map(|provider| resolve_provider_from_name(provider, "planner_provider"))
    });
    let gatherer_model = cli.gatherer_model.or(config.gatherer_model);
    let provider_url = cli.provider_url.or(config.provider_url);

    let provider = cli
        .provider
        .unwrap_or_else(|| resolve_provider_from_name(&config.provider, "provider"));

    let provider_name = provider.name();
    let normalized_model = normalize_provider_model_alias(provider_name, &model);
    if normalized_model != model {
        eprintln!(
            "[WARN] Model `{model}` is no longer valid for provider `{provider_name}`; using `{normalized_model}` instead."
        );
        model = normalized_model;
    }
    let normalized_planner_provider = planner_provider.unwrap_or(provider);
    planner_model = planner_model.map(|planner| {
        let normalized =
            normalize_provider_model_alias(normalized_planner_provider.name(), &planner);
        if normalized != planner {
            eprintln!(
                "[WARN] Planner model `{planner}` is no longer valid for provider `{}`; using `{normalized}` instead.",
                normalized_planner_provider.name()
            );
        }
        normalized
    });

    let normalized_gatherer_provider = normalize_gatherer_provider_alias(&config.gatherer_provider);
    if normalized_gatherer_provider != config.gatherer_provider {
        eprintln!(
            "[WARN] Gatherer provider `{}` is deprecated; using `{normalized_gatherer_provider}` instead.",
            config.gatherer_provider
        );
    }
    let gatherer_provider =
        cli.gatherer_provider
            .unwrap_or(match normalized_gatherer_provider.as_str() {
                "local" => GathererProvider::Local,
                "context1" => GathererProvider::Context1,
                _ => GathererProvider::SiftDirect,
            });

    let frontend = select_interactive_frontend(
        cli.prompt.is_some(),
        std::io::stdin().is_terminal(),
        std::io::stdout().is_terminal(),
    );

    // Initialize tracing based on verbosity
    let log_level = match frontend {
        InteractiveFrontend::Tui => tracing::Level::ERROR,
        InteractiveFrontend::PlainLines => match verbose {
            0 => tracing::Level::ERROR,
            1 => tracing::Level::INFO,
            2 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        },
    };

    tracing_subscriber::fmt().with_max_level(log_level).init();

    let registry = Arc::new(SiftRegistryAdapter::new());
    let operator_memory = Arc::new(AgentMemory::load(&root_path));

    // Resolve API key: env var first, then credential store.
    let credential_store = Arc::new(CredentialStore::new());
    let mut provider_url_overrides = std::collections::BTreeMap::new();
    if let Some(base_url) = provider_url.clone() {
        provider_url_overrides.insert(provider, base_url);
    }
    let provider_url_overrides = Arc::new(provider_url_overrides);

    let synth_credentials = Arc::clone(&credential_store);
    let synth_overrides = Arc::clone(&provider_url_overrides);
    let synthesizer_factory: Box<paddles::application::SynthesizerFactory> =
        Box::new(move |workspace: &Path, lane: &PreparedModelLane| {
            build_synthesizer_engine(
                workspace,
                lane,
                synth_credentials.as_ref(),
                synth_overrides.as_ref(),
            )
        });

    let planner_credentials = Arc::clone(&credential_store);
    let planner_overrides = Arc::clone(&provider_url_overrides);
    let planner_factory: Box<paddles::application::PlannerFactory> =
        Box::new(move |workspace: &Path, lane: &PreparedModelLane| {
            build_planner_engine(
                workspace,
                lane,
                planner_credentials.as_ref(),
                planner_overrides.as_ref(),
            )
        });
    let gatherer_factory: Box<paddles::application::GathererFactory> = Box::new(
        |config: &RuntimeLaneConfig,
         workspace: &Path,
         verbose: u8,
         gatherer_model_paths: Option<paddles::domain::ports::ModelPaths>|
         -> Result<Option<(PreparedGathererLane, Arc<dyn ContextGatherer>)>> {
            match config.gatherer_provider() {
                GathererProvider::Local => match config.gatherer_model_id() {
                    Some(model_id) => {
                        let lane = PreparedGathererLane {
                            provider: GathererProvider::Local,
                            label: model_id.to_string(),
                            model_id: Some(model_id.to_string()),
                            paths: gatherer_model_paths,
                        };
                        let adapter =
                            SiftContextGathererAdapter::new(workspace.to_path_buf(), model_id);
                        adapter.set_verbose(verbose);
                        Ok(Some((lane, Arc::new(adapter) as Arc<dyn ContextGatherer>)))
                    }
                    None => Ok(None),
                },
                GathererProvider::SiftDirect => {
                    let lane = PreparedGathererLane {
                        provider: GathererProvider::SiftDirect,
                        label: "sift-direct".to_string(),
                        model_id: None,
                        paths: None,
                    };
                    let adapter = SiftDirectGathererAdapter::new(workspace.to_path_buf());
                    adapter.set_verbose(verbose);
                    Ok(Some((lane, Arc::new(adapter) as Arc<dyn ContextGatherer>)))
                }
                GathererProvider::Context1 => {
                    let lane = PreparedGathererLane {
                        provider: GathererProvider::Context1,
                        label: "context-1".to_string(),
                        model_id: None,
                        paths: None,
                    };
                    let adapter = Context1GathererAdapter::new(config.context1_harness_ready());
                    Ok(Some((lane, Arc::new(adapter) as Arc<dyn ContextGatherer>)))
                }
            }
        },
    );
    let trace_recorder = Arc::new(
        paddles::infrastructure::adapters::trace_recorders::InMemoryTraceRecorder::default(),
    );
    let service = Arc::new(MechSuitService::with_trace_recorder(
        root_path,
        registry,
        operator_memory,
        synthesizer_factory,
        planner_factory,
        gatherer_factory,
        trace_recorder.clone(),
    ));
    let runtime_verbose = match frontend {
        InteractiveFrontend::Tui => 0,
        InteractiveFrontend::PlainLines => verbose,
    };
    service.set_verbose(runtime_verbose);

    // Boot sequence
    if verbose >= 3 {
        println!("[BOOT] Initializing system...");
    }
    let boot_ctx = service.boot(credits, weights, biases, hf_token.clone(), reality_mode)?;

    if verbose >= 3 {
        println!("[BOOT] Inherited Credits: {}", boot_ctx.credits);
        println!("[BOOT] Applying Foundational Weights: {}", boot_ctx.weight);
        println!("[BOOT] Applying Foundational Biases: {}", boot_ctx.bias);
        if boot_ctx.hf_token.is_some() {
            println!("[BOOT] Hugging Face Token: [MASKED]");
        }
        println!("[BOOT] Calibration Successful.");
    }

    // Registry Synchronization via Sift
    if verbose >= 3 {
        println!("[BOOT] Syncing synthesizer lane via SIFT for: {model}...");
        if let Some(pm) = &planner_model {
            println!("[BOOT] Syncing planner lane via SIFT for: {pm}...");
        }
        if let Some(gm) = &gatherer_model {
            println!("[BOOT] Syncing gatherer lane via SIFT for: {gm}...");
        }
        match gatherer_provider {
            GathererProvider::SiftDirect => {
                println!("[BOOT] Sift direct retrieval gatherer provider selected.");
            }
            GathererProvider::Context1 => {
                println!(
                    "[BOOT] Context-1 gatherer provider selected (harness ready: {context1_harness_ready})."
                );
            }
            GathererProvider::Local => {}
        }
    }
    let runtime_lanes = RuntimeLaneConfig::new(model.clone(), gatherer_model.clone())
        .with_synthesizer_provider(provider)
        .with_planner_model_id(planner_model.clone())
        .with_planner_provider(planner_provider)
        .with_gatherer_provider(gatherer_provider)
        .with_context1_harness_ready(context1_harness_ready);
    let _prepared_lanes = service.prepare_runtime_lanes(&runtime_lanes).await?;
    if verbose >= 3 {
        println!("[BOOT] Runtime lanes ready.");
    }

    // Start HTTP API server
    let (web_router, web_observer) =
        paddles::infrastructure::web::router(Arc::clone(&service), trace_recorder);
    service.register_event_observer(web_observer);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    if verbose >= 3 {
        println!("[BOOT] HTTP API server listening on port {port}.");
    }
    tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, web_router).await {
            eprintln!("[ERROR] HTTP server failed: {err}");
        }
    });

    if let Some(prompt) = cli.prompt {
        let response = service.process_prompt(&prompt).await?;
        println!("Chord Response: {}", response);
    } else {
        match frontend {
            InteractiveFrontend::Tui => {
                let tui_ctx = TuiContext {
                    credential_store: Arc::clone(&credential_store),
                    runtime_preference_store: Arc::clone(&runtime_preference_store),
                    runtime_lanes: runtime_lanes.clone(),
                    verbose,
                };
                run_interactive_tui(service, tui_ctx).await?
            }
            InteractiveFrontend::PlainLines => run_plain_interactive_loop(service).await?,
        }
    }

    Ok(())
}

async fn run_plain_interactive_loop(service: Arc<MechSuitService>) -> Result<()> {
    println!("--- Interactive Mode (type 'exit' or use Ctrl+C to quit) ---");
    let mut stdin_reader = BufReader::new(tokio_io::stdin()).lines();
    let session = service.shared_conversation_session();
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

            let response = service
                .process_prompt_in_session_with_sink(
                    input,
                    session.clone(),
                    Arc::new(paddles::domain::model::NullTurnEventSink),
                )
                .await?;
            println!("Chord Response: {}", response);
        } else {
            break;
        }
    }

    Ok(())
}
