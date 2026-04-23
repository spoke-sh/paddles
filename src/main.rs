use anyhow::{Context, Result, bail};
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
use paddles::infrastructure::adapters::http_provider::{HttpPlannerAdapter, HttpProviderAdapter};
use paddles::infrastructure::adapters::sift_agent::SiftAgentAdapter;
use paddles::infrastructure::adapters::sift_context_gatherer::SiftContextGathererAdapter;
use paddles::infrastructure::adapters::sift_direct_gatherer::SiftDirectGathererAdapter;
use paddles::infrastructure::adapters::sift_planner::SiftPlannerAdapter;
use paddles::infrastructure::adapters::sift_registry::SiftRegistryAdapter;
use paddles::infrastructure::adapters::trace_recorders::HostedTransitTraceRecorder;
use paddles::infrastructure::adapters::trace_recorders::InMemoryTraceRecorder;
use paddles::infrastructure::adapters::trace_recorders::default_trace_recorder_for_workspace;
use paddles::infrastructure::cli::interactive_tui::{
    InteractiveFrontend, TuiContext, run_interactive_tui, select_interactive_frontend,
};
use paddles::infrastructure::config::{
    PaddlesConfig, TraceAuthoritySelection, normalize_gatherer_provider_alias,
    normalize_provider_model_alias, resolve_runtime_verbosity, resolve_web_server_port,
};
use paddles::infrastructure::conversation_history::ConversationHistoryStore;
use paddles::infrastructure::credentials::CredentialStore;
use paddles::infrastructure::execution_hand::ExecutionHandRegistry;
use paddles::infrastructure::native_transport::{
    NativeTransportRegistry, record_binding_started, record_bound_transport,
    record_transport_failure, resolve_shared_web_bind_target,
};
use paddles::infrastructure::providers::{ApiFormat, ModelCapabilitySurface, ModelProvider};
use paddles::infrastructure::runtime_preferences::RuntimeLanePreferenceStore;
use paddles::infrastructure::transport_mediator::TransportToolMediator;

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

fn ensure_remote_provider_transport_support(provider: ModelProvider, model_id: &str) -> Result<()> {
    if let Some(message) = provider.paddles_http_transport_error(model_id) {
        bail!("{message}");
    }
    Ok(())
}

fn resolve_remote_provider_config(
    provider: ModelProvider,
    model_id: &str,
    transport_mediator: &TransportToolMediator,
    provider_url_overrides: &std::collections::BTreeMap<ModelProvider, String>,
) -> Result<(ApiFormat, String, String, ModelCapabilitySurface)> {
    ensure_remote_provider_transport_support(provider, model_id)?;
    let capability_surface = provider.capability_surface(model_id);
    let api_key = transport_mediator.resolve_provider_api_key(provider, model_id)?;

    let api_format = capability_surface.http_format.ok_or_else(|| {
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
    Ok((api_format, base_url, api_key, capability_surface))
}

fn build_synthesizer_engine(
    workspace: &Path,
    execution_hand_registry: Arc<ExecutionHandRegistry>,
    transport_mediator: Arc<TransportToolMediator>,
    lane: &PreparedModelLane,
    provider_url_overrides: &std::collections::BTreeMap<ModelProvider, String>,
) -> Result<Arc<dyn SynthesizerEngine>> {
    match lane.provider {
        ModelProvider::Sift => Ok(Arc::new(SiftAgentAdapter::new_with_runtime_mediator(
            workspace.to_path_buf(),
            execution_hand_registry,
            transport_mediator,
            &lane.model_id,
            lane.paths
                .clone()
                .ok_or_else(|| anyhow::anyhow!("local sift lane missing prepared model paths"))?,
            lane.provider
                .capability_surface(&lane.model_id)
                .render_capability,
        )?) as Arc<dyn SynthesizerEngine>),
        provider => {
            let (format, base_url, api_key, capabilities) = resolve_remote_provider_config(
                provider,
                &lane.model_id,
                transport_mediator.as_ref(),
                provider_url_overrides,
            )?;
            Ok(Arc::new(HttpProviderAdapter::new_with_runtime_mediator(
                workspace.to_path_buf(),
                execution_hand_registry,
                transport_mediator,
                provider.name(),
                lane.model_id.clone(),
                api_key,
                base_url,
                format,
                capabilities.render_capability,
            )) as Arc<dyn SynthesizerEngine>)
        }
    }
}

fn build_planner_engine(
    workspace: &Path,
    execution_hand_registry: Arc<ExecutionHandRegistry>,
    transport_mediator: Arc<TransportToolMediator>,
    lane: &PreparedModelLane,
    provider_url_overrides: &std::collections::BTreeMap<ModelProvider, String>,
) -> Result<Arc<dyn paddles::domain::ports::RecursivePlanner>> {
    match lane.provider {
        ModelProvider::Sift => {
            let engine = Arc::new(SiftAgentAdapter::new_with_runtime_mediator(
                workspace.to_path_buf(),
                execution_hand_registry,
                transport_mediator,
                &lane.model_id,
                lane.paths.clone().ok_or_else(|| {
                    anyhow::anyhow!("local sift lane missing prepared model paths")
                })?,
                lane.provider
                    .capability_surface(&lane.model_id)
                    .render_capability,
            )?);
            Ok(Arc::new(SiftPlannerAdapter::new(engine))
                as Arc<dyn paddles::domain::ports::RecursivePlanner>)
        }
        provider => {
            let (format, base_url, api_key, capabilities) = resolve_remote_provider_config(
                provider,
                &lane.model_id,
                transport_mediator.as_ref(),
                provider_url_overrides,
            )?;
            let engine = Arc::new(HttpProviderAdapter::new_with_runtime_mediator(
                workspace.to_path_buf(),
                execution_hand_registry,
                transport_mediator,
                provider.name(),
                lane.model_id.clone(),
                api_key,
                base_url,
                format,
                capabilities.render_capability,
            ));
            Ok(Arc::new(HttpPlannerAdapter::new(engine))
                as Arc<dyn paddles::domain::ports::RecursivePlanner>)
        }
    }
}

fn build_trace_recorder_from_config(
    root_path: &Path,
    selection: &TraceAuthoritySelection,
) -> Result<Arc<dyn paddles::domain::ports::TraceRecorder>> {
    match selection {
        TraceAuthoritySelection::EmbeddedLocal { explicit } => {
            if *explicit {
                eprintln!("[BOOT] Trace authority: explicit embedded local Transit recorder.");
            }
            Ok(default_trace_recorder_for_workspace(root_path))
        }
        TraceAuthoritySelection::InMemory { .. } => {
            Ok(Arc::new(InMemoryTraceRecorder::new_ephemeral(
                "configured explicit in-memory trace authority mode",
            )))
        }
        TraceAuthoritySelection::HostedTransit(hosted) => {
            let endpoint = hosted
                .endpoint
                .parse()
                .with_context(|| format!("parse hosted transit endpoint `{}`", hosted.endpoint))?;
            Ok(Arc::new(HostedTransitTraceRecorder::connect(
                endpoint,
                hosted.namespace.clone(),
                hosted.service_identity.clone(),
            )?))
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let root_path = env::current_dir()?;
    let conversation_history_store = Arc::new(ConversationHistoryStore::new());
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
    let trace_authority_selection = config
        .resolve_trace_authority_selection()
        .map_err(anyhow::Error::msg)?;
    let authored_port_configured = PaddlesConfig::authored_port_is_configured(&root_path);

    // Merge: CLI flags override config values.
    let shared_provider = cli
        .provider
        .unwrap_or_else(|| resolve_provider_from_name(&config.provider, "provider"));
    let mut shared_model = cli.model.clone().unwrap_or_else(|| config.model.clone());
    let provider = cli.provider.unwrap_or_else(|| {
        config
            .synthesizer_provider
            .as_deref()
            .map(|provider| resolve_provider_from_name(provider, "synthesizer.provider"))
            .unwrap_or(shared_provider)
    });
    let mut model = cli.model.clone().unwrap_or_else(|| {
        config
            .synthesizer_model
            .clone()
            .unwrap_or_else(|| config.model.clone())
    });
    let mut thinking_mode = if cli.provider.is_some() || cli.model.is_some() {
        None
    } else {
        config.thinking_mode.clone()
    };
    let credits = cli.credits.unwrap_or(config.credits);
    let weights = cli.weights.unwrap_or(config.weights);
    let biases = cli.biases.unwrap_or(config.biases);
    let reality_mode = cli.reality_mode || config.reality_mode;
    let requested_port = resolve_web_server_port(cli.port, config.port, authored_port_configured);
    let verbose = resolve_runtime_verbosity(cli.verbose, config.verbose);
    let hf_token = cli.hf_token.or(config.hf_token);
    let context1_harness_ready = cli.context1_harness_ready || config.context1_harness_ready;
    let explicit_planner_provider = cli.planner_provider.or_else(|| {
        config
            .planner_provider
            .as_deref()
            .map(|provider| resolve_provider_from_name(provider, "planner_provider"))
    });
    let mut planner_model = cli.planner_model.or(config.planner_model);
    let gatherer_model = cli.gatherer_model.or(config.gatherer_model);
    let provider_url = cli.provider_url.or(config.provider_url.clone());

    let provider_name = provider.name();
    let normalized_shared_model =
        normalize_provider_model_alias(shared_provider.name(), &shared_model);
    if normalized_shared_model != shared_model {
        eprintln!(
            "[WARN] Shared model `{shared_model}` is no longer valid for provider `{}`; using `{normalized_shared_model}` instead.",
            shared_provider.name()
        );
        shared_model = normalized_shared_model;
    }
    let normalized_model = normalize_provider_model_alias(provider_name, &model);
    if normalized_model != model {
        eprintln!(
            "[WARN] Model `{model}` is no longer valid for provider `{provider_name}`; using `{normalized_model}` instead."
        );
        model = normalized_model;
    }
    if let Some(requested_thinking_mode) = thinking_mode.clone()
        && !provider
            .thinking_modes(&model)
            .iter()
            .any(|mode| mode.thinking_mode == Some(requested_thinking_mode.as_str()))
    {
        eprintln!(
            "[WARN] Thinking mode `{requested_thinking_mode}` is not valid for `{}`; clearing it.",
            provider.qualified_model_label(&model)
        );
        thinking_mode = None;
    }
    let normalized_planner_provider = explicit_planner_provider.unwrap_or(shared_provider);
    let effective_planner_model = planner_model.take().unwrap_or_else(|| shared_model.clone());
    let normalized_planner_model = normalize_provider_model_alias(
        normalized_planner_provider.name(),
        &effective_planner_model,
    );
    if normalized_planner_model != effective_planner_model {
        eprintln!(
            "[WARN] Planner model `{effective_planner_model}` is no longer valid for provider `{}`; using `{normalized_planner_model}` instead.",
            normalized_planner_provider.name()
        );
    }
    let planner_provider =
        (normalized_planner_provider != provider).then_some(normalized_planner_provider);
    let planner_model = (normalized_planner_model != model).then_some(normalized_planner_model);

    let normalized_gatherer_provider = normalize_gatherer_provider_alias(&config.gatherer_provider);
    let gatherer_provider = match cli.gatherer_provider {
        Some(provider) => provider,
        None => match normalized_gatherer_provider.as_str() {
            "sift-direct" => GathererProvider::SiftDirect,
            "local" => GathererProvider::Local,
            "context1" => GathererProvider::Context1,
            _ => bail!(
                "Invalid gatherer provider `{}` in config. Expected one of `sift-direct`, `local`, or `context1`.",
                config.gatherer_provider
            ),
        },
    };

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

    let registry = match frontend {
        InteractiveFrontend::Tui => Arc::new(SiftRegistryAdapter::new()),
        InteractiveFrontend::PlainLines if verbose >= 1 => {
            Arc::new(SiftRegistryAdapter::with_preparation_reporter(|model_id| {
                println!("[SIFT] Preparing model: {model_id}");
            }))
        }
        InteractiveFrontend::PlainLines => Arc::new(SiftRegistryAdapter::new()),
    };
    let operator_memory = Arc::new(AgentMemory::load(&root_path));

    // Resolve API key: env var first, then credential store.
    let credential_store = Arc::new(CredentialStore::new());
    let mut provider_url_overrides = std::collections::BTreeMap::new();
    if let Some(base_url) = provider_url.clone() {
        provider_url_overrides.insert(provider, base_url);
    }
    let provider_url_overrides = Arc::new(provider_url_overrides);

    let synth_overrides = Arc::clone(&provider_url_overrides);
    let execution_hand_registry = Arc::new(ExecutionHandRegistry::default());
    let transport_mediator = Arc::new(TransportToolMediator::new(
        Arc::clone(&credential_store),
        Arc::clone(&execution_hand_registry),
        &config.native_transports,
    ));
    let synth_execution_hands = Arc::clone(&execution_hand_registry);
    let synth_transport_mediator = Arc::clone(&transport_mediator);
    let synthesizer_factory: Box<paddles::application::SynthesizerFactory> =
        Box::new(move |workspace: &Path, lane: &PreparedModelLane| {
            build_synthesizer_engine(
                workspace,
                Arc::clone(&synth_execution_hands),
                Arc::clone(&synth_transport_mediator),
                lane,
                synth_overrides.as_ref(),
            )
        });

    let planner_overrides = Arc::clone(&provider_url_overrides);
    let planner_execution_hands = Arc::clone(&execution_hand_registry);
    let planner_transport_mediator = Arc::clone(&transport_mediator);
    let planner_factory: Box<paddles::application::PlannerFactory> =
        Box::new(move |workspace: &Path, lane: &PreparedModelLane| {
            build_planner_engine(
                workspace,
                Arc::clone(&planner_execution_hands),
                Arc::clone(&planner_transport_mediator),
                lane,
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
    let trace_recorder = build_trace_recorder_from_config(&root_path, &trace_authority_selection)?;
    let service = Arc::new(MechSuitService::with_trace_recorder(
        root_path,
        registry,
        operator_memory,
        synthesizer_factory,
        planner_factory,
        gatherer_factory,
        trace_recorder.clone(),
    ));
    service.set_execution_hand_registry(Arc::clone(&execution_hand_registry));
    service.set_conversation_history_store(conversation_history_store);
    let native_transport_registry = Arc::new(NativeTransportRegistry::new(
        config.native_transports.clone(),
    ));
    service.set_native_transport_registry(Arc::clone(&native_transport_registry));
    service.set_verbose(verbose);

    match service.trace_recorder_capability() {
        paddles::domain::ports::TraceRecorderCapability::Persistent { .. } => {}
        paddles::domain::ports::TraceRecorderCapability::Ephemeral { backend, reason } => {
            eprintln!(
                "[BOOT] Trace recorder degraded to {backend}; session durability is bounded: {reason}"
            );
        }
        paddles::domain::ports::TraceRecorderCapability::Unsupported { reason } => {
            eprintln!("[BOOT] Trace recorder disabled: {reason}");
        }
    }

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
        .with_synthesizer_thinking_mode(thinking_mode)
        .with_planner_model_id(planner_model.clone())
        .with_planner_provider(planner_provider)
        .with_gatherer_provider(gatherer_provider)
        .with_context1_harness_ready(context1_harness_ready);
    let _prepared_lanes = service.prepare_runtime_lanes(&runtime_lanes).await?;
    if verbose >= 3 {
        println!("[BOOT] Runtime lanes ready.");
    }

    // Start HTTP API server
    let (web_router, web_observer) = paddles::infrastructure::web::router(
        Arc::clone(&service),
        trace_recorder,
        config.native_transports.clone(),
        Arc::clone(&transport_mediator),
    );
    service.register_event_observer(web_observer);
    let http_transport = &config.native_transports.http_request_response;
    let sse_transport = &config.native_transports.server_sent_events;
    let websocket_transport = &config.native_transports.websocket;
    let transit_transport = &config.native_transports.transit;
    let default_bind_target = format!("0.0.0.0:{requested_port}");
    let resolved_bind_target = match resolve_shared_web_bind_target(
        http_transport,
        sse_transport,
        websocket_transport,
        transit_transport,
        &default_bind_target,
    ) {
        Ok(bind_target) => bind_target,
        Err(error) => {
            record_transport_failure(&native_transport_registry, http_transport, error.clone());
            record_transport_failure(&native_transport_registry, sse_transport, error.clone());
            record_transport_failure(
                &native_transport_registry,
                websocket_transport,
                error.clone(),
            );
            record_transport_failure(&native_transport_registry, transit_transport, error.clone());
            return Err(anyhow::anyhow!(error));
        }
    };
    record_binding_started(&native_transport_registry, http_transport);
    record_binding_started(&native_transport_registry, sse_transport);
    record_binding_started(&native_transport_registry, websocket_transport);
    record_binding_started(&native_transport_registry, transit_transport);
    let listener = match tokio::net::TcpListener::bind(&resolved_bind_target).await {
        Ok(listener) => listener,
        Err(error) => {
            record_transport_failure(
                &native_transport_registry,
                http_transport,
                error.to_string(),
            );
            record_transport_failure(&native_transport_registry, sse_transport, error.to_string());
            record_transport_failure(
                &native_transport_registry,
                websocket_transport,
                error.to_string(),
            );
            record_transport_failure(
                &native_transport_registry,
                transit_transport,
                error.to_string(),
            );
            return Err(error.into());
        }
    };
    let web_server_addr = listener.local_addr()?;
    record_bound_transport(
        &native_transport_registry,
        http_transport,
        &web_server_addr.to_string(),
    );
    record_bound_transport(
        &native_transport_registry,
        sse_transport,
        &web_server_addr.to_string(),
    );
    record_bound_transport(
        &native_transport_registry,
        websocket_transport,
        &web_server_addr.to_string(),
    );
    record_bound_transport(
        &native_transport_registry,
        transit_transport,
        &web_server_addr.to_string(),
    );
    if verbose >= 3 {
        println!(
            "[BOOT] HTTP API server listening on {}.",
            paddles::infrastructure::web::web_server_url(web_server_addr)
        );
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
                    web_server_addr,
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

#[cfg(test)]
mod tests {
    use super::{build_trace_recorder_from_config, ensure_remote_provider_transport_support};
    use paddles::infrastructure::config::{HostedTransitAuthorityConfig, TraceAuthoritySelection};
    use paddles::infrastructure::providers::ModelProvider;
    use std::path::Path;
    use tempfile::tempdir;
    use transit_core::engine::LocalEngineConfig;
    use transit_core::membership::NodeId;
    use transit_core::server::{ServerConfig, ServerHandle};

    fn hosted_server() -> (tempfile::TempDir, ServerHandle) {
        let temp = tempdir().expect("tempdir");
        let server = ServerHandle::bind(ServerConfig::new(
            LocalEngineConfig::new(temp.path(), NodeId::new("main-hosted-transit-test-node")),
            "127.0.0.1:0".parse().expect("listen addr"),
        ))
        .expect("bind hosted transit server");
        (temp, server)
    }

    #[test]
    fn remote_provider_transport_allows_openai_responses_models() {
        ensure_remote_provider_transport_support(ModelProvider::Openai, "gpt-5.4-pro")
            .expect("responses-capable OpenAI model should be allowed");
    }

    #[test]
    fn remote_provider_transport_allows_supported_openai_chat_models() {
        ensure_remote_provider_transport_support(ModelProvider::Openai, "gpt-5.4")
            .expect("supported OpenAI chat model should remain allowed");
    }

    #[test]
    fn hosted_service_mode_does_not_require_embedded_transit_core() {
        let (_server_root, server) = hosted_server();
        let workspace = tempdir().expect("workspace");

        let recorder = build_trace_recorder_from_config(
            Path::new(workspace.path()),
            &TraceAuthoritySelection::HostedTransit(HostedTransitAuthorityConfig {
                endpoint: server.local_addr().to_string(),
                namespace: "test-hosted".to_string(),
                service_identity: "svc-main".to_string(),
            }),
        )
        .expect("hosted service-mode recorder");

        assert!(matches!(
            recorder.capability(),
            paddles::domain::ports::TraceRecorderCapability::Persistent { backend, .. }
                if backend == "hosted_transit"
        ));
    }
}
