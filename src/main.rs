use anyhow::{Context, Result, bail};
use clap::Parser;
use serde::Serialize;
use std::env;
use std::io::IsTerminal;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{self as tokio_io, AsyncBufReadExt, BufReader};

use paddles::application::{
    AgentRuntime, ExternalCapabilityBrokerRegistry, GathererProvider,
    HarnessCapabilityRuntimeStatus, PreparedModelClient, PreparedRetrievalProvider,
    RuntimeHarnessCapabilityPostureService, TurnRuntimeConfig,
    ensure_supported_model_inference_provider,
};
use paddles::domain::ports::{FinalRenderingEngine, ModelPaths, ModelRegistry, RetrievalProvider};
use paddles::infrastructure::adapters::agent_memory::AgentMemory;
use paddles::infrastructure::adapters::context1_retrieval::Context1RetrievalAdapter;
use paddles::infrastructure::adapters::http_provider::{
    HttpActionSelectionAdapter, HttpProviderAdapter,
};
use paddles::infrastructure::adapters::sift_context_retrieval::SiftContextRetrievalAdapter;
use paddles::infrastructure::adapters::sift_direct_retrieval::SiftDirectRetrievalAdapter;
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
use paddles::infrastructure::runtime_preferences::TurnRuntimePreferenceStore;
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

    /// Optional model ID for the legacy static local retrieval provider.
    #[arg(long)]
    gatherer_model: Option<String>,

    /// Provider to use for retrieval.
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

#[derive(Debug, Default)]
struct UnsupportedModelRegistry;

#[async_trait::async_trait]
impl ModelRegistry for UnsupportedModelRegistry {
    async fn get_model_paths(&self, model_id: &str) -> Result<ModelPaths> {
        bail!(
            "paddles no longer prepares local inference model paths for `{model_id}`; run a local HTTP model service and select `ollama:<model>` instead"
        );
    }
}

fn resolve_provider_from_name(configured: &str, field_name: &str) -> Result<ModelProvider> {
    let provider = ModelProvider::from_name(configured.trim()).ok_or_else(|| {
        anyhow::anyhow!(
            "Unsupported {field_name} `{configured}`. Expected one of `openai`, `inception`, `anthropic`, `google`, `moonshot`, or `ollama`."
        )
    })?;
    ensure_supported_model_inference_provider(provider, field_name)?;
    Ok(provider)
}

fn resolve_cli_model_provider(provider: ModelProvider, field_name: &str) -> Result<ModelProvider> {
    ensure_supported_model_inference_provider(provider, field_name)?;
    Ok(provider)
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

fn build_final_renderer(
    workspace: &Path,
    execution_hand_registry: Arc<ExecutionHandRegistry>,
    transport_mediator: Arc<TransportToolMediator>,
    lane: &PreparedModelClient,
    provider_url_overrides: &std::collections::BTreeMap<ModelProvider, String>,
) -> Result<Arc<dyn FinalRenderingEngine>> {
    ensure_supported_model_inference_provider(lane.provider, "synthesizer.provider")?;
    if lane.paths.is_some() {
        bail!("final-rendering HTTP model client must not receive local ModelPaths");
    }
    let (format, base_url, api_key, capabilities) = resolve_remote_provider_config(
        lane.provider,
        &lane.model_id,
        transport_mediator.as_ref(),
        provider_url_overrides,
    )?;
    Ok(Arc::new(HttpProviderAdapter::new_with_runtime_mediator(
        workspace.to_path_buf(),
        execution_hand_registry,
        transport_mediator,
        lane.provider.name(),
        lane.model_id.clone(),
        api_key,
        base_url,
        format,
        capabilities.render_capability,
    )) as Arc<dyn FinalRenderingEngine>)
}

fn build_action_selector(
    workspace: &Path,
    execution_hand_registry: Arc<ExecutionHandRegistry>,
    transport_mediator: Arc<TransportToolMediator>,
    lane: &PreparedModelClient,
    provider_url_overrides: &std::collections::BTreeMap<ModelProvider, String>,
) -> Result<Arc<dyn paddles::domain::ports::ActionSelectionEngine>> {
    ensure_supported_model_inference_provider(lane.provider, "planner_provider")?;
    if lane.paths.is_some() {
        bail!("action-selection HTTP model client must not receive local ModelPaths");
    }
    let (format, base_url, api_key, capabilities) = resolve_remote_provider_config(
        lane.provider,
        &lane.model_id,
        transport_mediator.as_ref(),
        provider_url_overrides,
    )?;
    let engine = Arc::new(HttpProviderAdapter::new_with_runtime_mediator(
        workspace.to_path_buf(),
        execution_hand_registry,
        transport_mediator,
        lane.provider.name(),
        lane.model_id.clone(),
        api_key,
        base_url,
        format,
        capabilities.render_capability,
    ));
    Ok(Arc::new(HttpActionSelectionAdapter::new(engine))
        as Arc<dyn paddles::domain::ports::ActionSelectionEngine>)
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ServiceRuntimeState {
    Ready,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum OperatorSurfaceRuntimeStatus {
    Disabled,
    Listening { bind_target: String },
    Degraded { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ServiceRuntimeStatus {
    mode: String,
    state: ServiceRuntimeState,
    authority_backend: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    authority_location: Option<String>,
    operator_surfaces: OperatorSurfaceRuntimeStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    harness_posture: Option<HarnessCapabilityRuntimeStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperatorSurfaceBootstrap {
    Start,
    Skip,
}

struct OperatorSurfaceBootOutcome {
    runtime_status: OperatorSurfaceRuntimeStatus,
    web_server_addr: Option<SocketAddr>,
}

fn resolve_operator_surface_bootstrap(
    service_mode_enabled: bool,
    operator_surfaces_enabled: bool,
) -> OperatorSurfaceBootstrap {
    if service_mode_enabled && !operator_surfaces_enabled {
        OperatorSurfaceBootstrap::Skip
    } else {
        OperatorSurfaceBootstrap::Start
    }
}

fn service_runtime_ready_status(
    capability: &paddles::domain::ports::TraceRecorderCapability,
    operator_surfaces: OperatorSurfaceRuntimeStatus,
    harness_posture: Option<HarnessCapabilityRuntimeStatus>,
) -> ServiceRuntimeStatus {
    let (authority_backend, authority_location) = match capability {
        paddles::domain::ports::TraceRecorderCapability::Persistent { backend, location } => {
            (backend.clone(), Some(location.clone()))
        }
        paddles::domain::ports::TraceRecorderCapability::Ephemeral { backend, reason } => {
            (backend.clone(), Some(reason.clone()))
        }
        paddles::domain::ports::TraceRecorderCapability::Unsupported { reason } => {
            ("unsupported".to_string(), Some(reason.clone()))
        }
    };

    ServiceRuntimeStatus {
        mode: "service".to_string(),
        state: ServiceRuntimeState::Ready,
        authority_backend,
        authority_location,
        operator_surfaces,
        harness_posture,
        failure: None,
    }
}

fn service_runtime_failure_status(
    selection: &TraceAuthoritySelection,
    failure: impl Into<String>,
) -> ServiceRuntimeStatus {
    let (authority_backend, authority_location) = match selection {
        TraceAuthoritySelection::EmbeddedLocal { .. } => ("embedded_local".to_string(), None),
        TraceAuthoritySelection::InMemory { .. } => ("in_memory".to_string(), None),
        TraceAuthoritySelection::HostedTransit(hosted) => (
            "hosted_transit".to_string(),
            Some(format!(
                "{}#namespace={};service={}",
                hosted.endpoint, hosted.namespace, hosted.service_identity
            )),
        ),
    };

    ServiceRuntimeStatus {
        mode: "service".to_string(),
        state: ServiceRuntimeState::Failed,
        authority_backend,
        authority_location,
        operator_surfaces: OperatorSurfaceRuntimeStatus::Disabled,
        harness_posture: None,
        failure: Some(failure.into()),
    }
}

fn emit_service_runtime_status(status: &ServiceRuntimeStatus) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string(status).context("serialize service runtime status")?
    );
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let root_path = env::current_dir()?;
    let conversation_history_store = Arc::new(ConversationHistoryStore::new());
    let runtime_preference_store = Arc::new(TurnRuntimePreferenceStore::new());
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
    let service_mode_enabled = config.service_mode.enabled;
    let service_operator_surfaces_enabled = config.service_mode.operator_surfaces_enabled;

    let run_result: Result<()> = async {
    // Merge: CLI flags override config values.
    let shared_provider = match cli.provider {
        Some(provider) => resolve_cli_model_provider(provider, "provider")?,
        None => resolve_provider_from_name(&config.provider, "provider")?,
    };
    let mut shared_model = cli.model.clone().unwrap_or_else(|| config.model.clone());
    let provider = match cli.provider {
        Some(provider) => resolve_cli_model_provider(provider, "provider")?,
        None => match config.synthesizer_provider.as_deref() {
            Some(provider) => resolve_provider_from_name(provider, "synthesizer.provider")?,
            None => shared_provider,
        },
    };
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
    let explicit_planner_provider = match cli.planner_provider {
        Some(provider) => Some(resolve_cli_model_provider(provider, "planner_provider")?),
        None => config
            .planner_provider
            .as_deref()
            .map(|provider| resolve_provider_from_name(provider, "planner_provider"))
            .transpose()?,
    };
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

    let registry = Arc::new(UnsupportedModelRegistry);
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
    let final_renderer_factory: Box<paddles::application::FinalRendererFactory> =
        Box::new(move |workspace: &Path, lane: &PreparedModelClient| {
            build_final_renderer(
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
    let action_selector_factory: Box<paddles::application::ActionSelectorFactory> =
        Box::new(move |workspace: &Path, lane: &PreparedModelClient| {
            build_action_selector(
                workspace,
                Arc::clone(&planner_execution_hands),
                Arc::clone(&planner_transport_mediator),
                lane,
                planner_overrides.as_ref(),
            )
        });
    let retrieval_provider_factory: Box<paddles::application::RetrievalProviderFactory> = Box::new(
        |config: &TurnRuntimeConfig,
         workspace: &Path,
         verbose: u8,
         gatherer_model_paths: Option<paddles::domain::ports::ModelPaths>|
         -> Result<Option<(PreparedRetrievalProvider, Arc<dyn RetrievalProvider>)>> {
            match config.gatherer_provider() {
                GathererProvider::Local => match config.gatherer_model_id() {
                    Some(model_id) => {
                        let lane = PreparedRetrievalProvider {
                            provider: GathererProvider::Local,
                            label: model_id.to_string(),
                            model_id: Some(model_id.to_string()),
                            paths: gatherer_model_paths,
                        };
                        let adapter =
                            SiftContextRetrievalAdapter::new(workspace.to_path_buf(), model_id);
                        adapter.set_verbose(verbose);
                        Ok(Some((lane, Arc::new(adapter) as Arc<dyn RetrievalProvider>)))
                    }
                    None => Ok(None),
                },
                GathererProvider::SiftDirect => {
                    let lane = PreparedRetrievalProvider {
                        provider: GathererProvider::SiftDirect,
                        label: "sift-direct".to_string(),
                        model_id: None,
                        paths: None,
                    };
                    let adapter = SiftDirectRetrievalAdapter::new(workspace.to_path_buf());
                    adapter.set_verbose(verbose);
                    Ok(Some((lane, Arc::new(adapter) as Arc<dyn RetrievalProvider>)))
                }
                GathererProvider::Context1 => {
                    let lane = PreparedRetrievalProvider {
                        provider: GathererProvider::Context1,
                        label: "context-1".to_string(),
                        model_id: None,
                        paths: None,
                    };
                    let adapter = Context1RetrievalAdapter::new(config.context1_harness_ready());
                    Ok(Some((lane, Arc::new(adapter) as Arc<dyn RetrievalProvider>)))
                }
            }
        },
    );
    let trace_recorder = build_trace_recorder_from_config(&root_path, &trace_authority_selection)?;
    let service = Arc::new(AgentRuntime::with_trace_recorder(
        root_path,
        registry,
        operator_memory,
        final_renderer_factory,
        action_selector_factory,
        retrieval_provider_factory,
        trace_recorder.clone(),
    ));
    service.set_execution_hand_registry(Arc::clone(&execution_hand_registry));
    service.set_external_capability_broker(Arc::new(
        ExternalCapabilityBrokerRegistry::from_local_configuration(
            config.external_capabilities.clone(),
        ),
    ));
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

    if verbose >= 3 {
        println!("[BOOT] Model clients use HTTP providers; no local model preparation is run.");
        println!(
            "[BOOT] Final rendering model client: {}",
            provider.qualified_model_label(&model)
        );
        if let Some(pm) = &planner_model {
            println!(
                "[BOOT] Action selection model client: {}",
                planner_provider
                    .unwrap_or(provider)
                    .qualified_model_label(pm)
            );
        }
        if let Some(gm) = &gatherer_model {
            println!("[BOOT] Retrieval model hint: {gm}.");
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
    let turn_runtime_config = TurnRuntimeConfig::new(model.clone(), gatherer_model.clone())
        .with_synthesizer_provider(provider)
        .with_synthesizer_thinking_mode(thinking_mode)
        .with_planner_model_id(planner_model.clone())
        .with_planner_provider(planner_provider)
        .with_gatherer_provider(gatherer_provider)
        .with_context1_harness_ready(context1_harness_ready);
    let prepared_turn_runtime = service.prepare_turn_runtime(&turn_runtime_config).await?;
    if verbose >= 3 {
        println!("[BOOT] Turn runtime ready.");
    }

    let http_transport = &config.native_transports.http_request_response;
    let sse_transport = &config.native_transports.server_sent_events;
    let websocket_transport = &config.native_transports.websocket;
    let transit_transport = &config.native_transports.transit;
    let operator_surface_boot = match resolve_operator_surface_bootstrap(
        service_mode_enabled,
        service_operator_surfaces_enabled,
    ) {
        OperatorSurfaceBootstrap::Skip => OperatorSurfaceBootOutcome {
            runtime_status: OperatorSurfaceRuntimeStatus::Disabled,
            web_server_addr: None,
        },
        OperatorSurfaceBootstrap::Start => 'operator_surface: {
            let (web_router, web_observer) = paddles::infrastructure::web::router(
                Arc::clone(&service),
                trace_recorder,
                config.native_transports.clone(),
                Arc::clone(&transport_mediator),
            );
            service.register_event_observer(web_observer);
            let default_bind_target = format!("0.0.0.0:{requested_port}");
            let resolved_bind_target = match resolve_shared_web_bind_target(
                http_transport,
                sse_transport,
                websocket_transport,
                transit_transport,
                &default_bind_target,
            ) {
                Ok(bind_target) => bind_target,
                Err(error) if service_mode_enabled => {
                    record_transport_failure(
                        &native_transport_registry,
                        http_transport,
                        error.clone(),
                    );
                    record_transport_failure(
                        &native_transport_registry,
                        sse_transport,
                        error.clone(),
                    );
                    record_transport_failure(
                        &native_transport_registry,
                        websocket_transport,
                        error.clone(),
                    );
                    record_transport_failure(
                        &native_transport_registry,
                        transit_transport,
                        error.clone(),
                    );
                    break 'operator_surface OperatorSurfaceBootOutcome {
                        runtime_status: OperatorSurfaceRuntimeStatus::Degraded { reason: error },
                        web_server_addr: None,
                    };
                }
                Err(error) => return Err(anyhow::anyhow!(error)),
            };
            record_binding_started(&native_transport_registry, http_transport);
            record_binding_started(&native_transport_registry, sse_transport);
            record_binding_started(&native_transport_registry, websocket_transport);
            record_binding_started(&native_transport_registry, transit_transport);
            let listener = match tokio::net::TcpListener::bind(&resolved_bind_target).await {
                Ok(listener) => listener,
                Err(error) if service_mode_enabled => {
                    record_transport_failure(
                        &native_transport_registry,
                        http_transport,
                        error.to_string(),
                    );
                    record_transport_failure(
                        &native_transport_registry,
                        sse_transport,
                        error.to_string(),
                    );
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
                    break 'operator_surface OperatorSurfaceBootOutcome {
                        runtime_status: OperatorSurfaceRuntimeStatus::Degraded {
                            reason: error.to_string(),
                        },
                        web_server_addr: None,
                    };
                }
                Err(error) => return Err(error.into()),
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
            break 'operator_surface OperatorSurfaceBootOutcome {
                runtime_status: OperatorSurfaceRuntimeStatus::Listening {
                    bind_target: paddles::infrastructure::web::web_server_url(web_server_addr),
                },
                web_server_addr: Some(web_server_addr),
            };
        },
    };

    if service_mode_enabled {
        let harness_posture = RuntimeHarnessCapabilityPostureService::project(
            &prepared_turn_runtime,
            &service.external_capability_descriptors(),
        );
        let status = service_runtime_ready_status(
            &service.trace_recorder_capability(),
            operator_surface_boot.runtime_status.clone(),
            Some(harness_posture),
        );
        emit_service_runtime_status(&status)?;
        tokio::signal::ctrl_c()
            .await
            .context("wait for service shutdown signal")?;
        return Ok(());
    }

    if let Some(prompt) = cli.prompt {
        let response = service.process_prompt(&prompt).await?;
        println!("Chord Response: {}", response);
    } else {
        match frontend {
            InteractiveFrontend::Tui => {
                let web_server_addr = operator_surface_boot
                    .web_server_addr
                    .expect("interactive TUI requires operator web surfaces");
                let tui_ctx = TuiContext {
                    credential_store: Arc::clone(&credential_store),
                    runtime_preference_store: Arc::clone(&runtime_preference_store),
                    turn_runtime_config: turn_runtime_config.clone(),
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
    .await;

    if let Err(error) = run_result {
        if service_mode_enabled {
            let status =
                service_runtime_failure_status(&trace_authority_selection, format!("{error:#}"));
            let _ = emit_service_runtime_status(&status);
        }
        return Err(error);
    }

    Ok(())
}

async fn run_plain_interactive_loop(service: Arc<AgentRuntime>) -> Result<()> {
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
    use super::{
        OperatorSurfaceBootstrap, OperatorSurfaceRuntimeStatus, ServiceRuntimeState,
        build_action_selector, build_final_renderer, build_trace_recorder_from_config,
        ensure_remote_provider_transport_support, resolve_operator_surface_bootstrap,
        resolve_provider_from_name, service_runtime_failure_status, service_runtime_ready_status,
    };
    use paddles::application::{
        ModelClientRole, PreparedModelClient, PreparedTurnRuntime,
        RuntimeHarnessCapabilityPostureService,
    };
    use paddles::domain::model::{
        ExecutionHandKind, ExecutionHandPhase, ExternalCapabilityCatalog,
        ExternalCapabilityCatalogConfig, NativeTransportConfigurations,
    };
    use paddles::domain::ports::ModelPaths;
    use paddles::infrastructure::config::{HostedTransitAuthorityConfig, TraceAuthoritySelection};
    use paddles::infrastructure::credentials::CredentialStore;
    use paddles::infrastructure::execution_hand::ExecutionHandRegistry;
    use paddles::infrastructure::providers::ModelProvider;
    use paddles::infrastructure::runtime_preferences::TurnRuntimePreferenceStore;
    use paddles::infrastructure::transport_mediator::TransportToolMediator;
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
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

    fn sample_model_paths(prefix: &str) -> ModelPaths {
        ModelPaths {
            weights: vec![PathBuf::from(format!("{prefix}-weights.safetensors"))],
            tokenizer: PathBuf::from(format!("{prefix}-tokenizer.json")),
            config: PathBuf::from(format!("{prefix}-config.json")),
            generation_config: Some(PathBuf::from(format!("{prefix}-generation-config.json"))),
        }
    }

    fn transport_mediator_for_test() -> (Arc<ExecutionHandRegistry>, Arc<TransportToolMediator>) {
        let execution_hand_registry = Arc::new(ExecutionHandRegistry::default());
        let transport_mediator = Arc::new(TransportToolMediator::with_execution_hand_registry(
            Arc::clone(&execution_hand_registry),
        ));
        (execution_hand_registry, transport_mediator)
    }

    fn transport_mediator_with_empty_credentials_for_test(
        credential_root: &Path,
    ) -> (Arc<ExecutionHandRegistry>, Arc<TransportToolMediator>) {
        let execution_hand_registry = Arc::new(ExecutionHandRegistry::default());
        let transport_mediator = Arc::new(TransportToolMediator::new(
            Arc::new(CredentialStore::with_base_dir_and_env(
                credential_root,
                BTreeMap::new(),
            )),
            Arc::clone(&execution_hand_registry),
            &NativeTransportConfigurations::default(),
        ));
        (execution_hand_registry, transport_mediator)
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
    fn provider_config_rejects_legacy_sift_model_provider_with_migration_hint() {
        let error = resolve_provider_from_name("sift", "provider")
            .expect_err("legacy sift provider config should fail");
        let message = format!("{error:#}");

        assert!(
            message.contains("provider `sift` no longer performs model inference"),
            "{message}"
        );
        assert!(message.contains("ollama:<model>"), "{message}");
    }

    #[test]
    fn planner_provider_config_rejects_legacy_sift_model_provider_with_migration_hint() {
        let error = resolve_provider_from_name("sift", "planner_provider")
            .expect_err("legacy sift planner provider config should fail");
        let message = format!("{error:#}");

        assert!(
            message.contains("provider `sift` no longer performs model inference"),
            "{message}"
        );
        assert!(message.contains("ollama:<model>"), "{message}");
    }

    #[test]
    fn action_selection_http_client_rejects_local_model_paths() {
        let workspace = tempdir().expect("workspace");
        let (execution_hand_registry, transport_mediator) = transport_mediator_for_test();
        let lane = PreparedModelClient {
            role: ModelClientRole::ActionSelection,
            provider: ModelProvider::Ollama,
            model_id: "qwen3".to_string(),
            paths: Some(sample_model_paths("planner")),
        };

        let error = match build_action_selector(
            workspace.path(),
            execution_hand_registry,
            transport_mediator,
            &lane,
            &BTreeMap::new(),
        ) {
            Ok(_) => panic!("action-selection HTTP model client should reject local ModelPaths"),
            Err(error) => error,
        };
        let message = format!("{error:#}");

        assert!(
            message
                .contains("action-selection HTTP model client must not receive local ModelPaths"),
            "{message}"
        );
    }

    #[test]
    fn action_selection_client_builds_from_http_provider_configuration() {
        let workspace = tempdir().expect("workspace");
        let (execution_hand_registry, transport_mediator) = transport_mediator_for_test();
        let lane = PreparedModelClient {
            role: ModelClientRole::ActionSelection,
            provider: ModelProvider::Ollama,
            model_id: "qwen3".to_string(),
            paths: None,
        };

        build_action_selector(
            workspace.path(),
            execution_hand_registry,
            transport_mediator,
            &lane,
            &BTreeMap::new(),
        )
        .expect("Ollama planner should build through the HTTP provider adapter");
    }

    #[test]
    fn action_selection_client_rejects_legacy_sift_provider_with_migration_hint() {
        let workspace = tempdir().expect("workspace");
        let (execution_hand_registry, transport_mediator) = transport_mediator_for_test();
        let lane = PreparedModelClient {
            role: ModelClientRole::ActionSelection,
            provider: ModelProvider::Sift,
            model_id: "qwen-1.5b".to_string(),
            paths: None,
        };

        let error = match build_action_selector(
            workspace.path(),
            execution_hand_registry,
            transport_mediator,
            &lane,
            &BTreeMap::new(),
        ) {
            Ok(_) => panic!("legacy sift action-selection provider should fail"),
            Err(error) => error,
        };
        let message = format!("{error:#}");

        assert!(
            message.contains("provider `sift` no longer performs model inference"),
            "{message}"
        );
        assert!(message.contains("ollama:<model>"), "{message}");
    }

    #[test]
    fn final_rendering_http_client_rejects_local_model_paths() {
        let workspace = tempdir().expect("workspace");
        let (execution_hand_registry, transport_mediator) = transport_mediator_for_test();
        let lane = PreparedModelClient {
            role: ModelClientRole::FinalRendering,
            provider: ModelProvider::Ollama,
            model_id: "qwen3".to_string(),
            paths: Some(sample_model_paths("synthesizer")),
        };

        let error = match build_final_renderer(
            workspace.path(),
            execution_hand_registry,
            transport_mediator,
            &lane,
            &BTreeMap::new(),
        ) {
            Ok(_) => panic!("final-rendering HTTP model client should reject local ModelPaths"),
            Err(error) => error,
        };
        let message = format!("{error:#}");

        assert!(
            message.contains("final-rendering HTTP model client must not receive local ModelPaths"),
            "{message}"
        );
    }

    #[test]
    fn final_rendering_client_builds_from_http_provider_configuration() {
        let workspace = tempdir().expect("workspace");
        let (execution_hand_registry, transport_mediator) = transport_mediator_for_test();
        let lane = PreparedModelClient {
            role: ModelClientRole::FinalRendering,
            provider: ModelProvider::Ollama,
            model_id: "qwen3".to_string(),
            paths: None,
        };

        build_final_renderer(
            workspace.path(),
            execution_hand_registry,
            transport_mediator,
            &lane,
            &BTreeMap::new(),
        )
        .expect("Ollama synthesizer should build through the HTTP provider adapter");
    }

    #[test]
    fn ollama_model_clients_build_without_credentials() {
        let workspace = tempdir().expect("workspace");
        let credentials = tempdir().expect("credentials");
        let (execution_hand_registry, transport_mediator) =
            transport_mediator_with_empty_credentials_for_test(credentials.path());
        let planner_lane = PreparedModelClient {
            role: ModelClientRole::ActionSelection,
            provider: ModelProvider::Ollama,
            model_id: "qwen3".to_string(),
            paths: None,
        };

        build_action_selector(
            workspace.path(),
            Arc::clone(&execution_hand_registry),
            Arc::clone(&transport_mediator),
            &planner_lane,
            &BTreeMap::new(),
        )
        .expect("Ollama planner should build without credentials");

        let planner_diagnostic = execution_hand_registry
            .diagnostic(ExecutionHandKind::TransportMediator)
            .expect("transport mediator diagnostic");
        assert_eq!(planner_diagnostic.phase, ExecutionHandPhase::Ready);
        assert!(
            planner_diagnostic
                .summary
                .contains("resolved provider credential for `ollama`"),
            "{}",
            planner_diagnostic.summary
        );

        let synthesizer_lane = PreparedModelClient {
            role: ModelClientRole::FinalRendering,
            provider: ModelProvider::Ollama,
            model_id: "qwen3".to_string(),
            paths: None,
        };

        build_final_renderer(
            workspace.path(),
            execution_hand_registry,
            transport_mediator,
            &synthesizer_lane,
            &BTreeMap::new(),
        )
        .expect("Ollama synthesizer should build without credentials");
    }

    #[test]
    fn migrated_turn_runtime_preferences_do_not_bypass_transport_mediator_credentials() {
        let workspace = tempdir().expect("workspace");
        let state = tempdir().expect("state");
        let credentials = tempdir().expect("credentials");
        let turn_runtime_path = state.path().join("turn-runtime.toml");
        let legacy_path = state.path().join("runtime-lanes.toml");
        std::fs::write(
            &legacy_path,
            r#"provider = "openai"
model = "gpt-5.4"
"#,
        )
        .expect("write legacy runtime lane preferences");
        let store = TurnRuntimePreferenceStore::with_migration_paths(
            &turn_runtime_path,
            Some(&legacy_path),
        );
        let preferences = store
            .load()
            .expect("load migrated preferences")
            .expect("migrated preferences");
        let final_rendering = preferences.final_rendering();
        let provider = final_rendering
            .provider
            .as_deref()
            .and_then(ModelProvider::from_name)
            .expect("migrated provider");
        let model_id = final_rendering
            .model
            .clone()
            .expect("migrated final rendering model");
        let lane = PreparedModelClient {
            role: ModelClientRole::FinalRendering,
            provider,
            model_id,
            paths: None,
        };
        let (execution_hand_registry, transport_mediator) =
            transport_mediator_with_empty_credentials_for_test(credentials.path());

        let error = match build_final_renderer(
            workspace.path(),
            Arc::clone(&execution_hand_registry),
            transport_mediator,
            &lane,
            &BTreeMap::new(),
        ) {
            Ok(_) => panic!("migrated OpenAI preferences should still require credentials"),
            Err(error) => error,
        };
        let message = format!("{error:#}");

        assert!(turn_runtime_path.exists());
        assert!(message.contains("provider `openai` is not authenticated"));
        assert!(message.contains("OPENAI_API_KEY"));
        assert!(message.contains("/login openai"));
        assert!(message.contains("openai:gpt-5.4"));
        let diagnostic = execution_hand_registry
            .diagnostic(ExecutionHandKind::TransportMediator)
            .expect("transport mediator diagnostic");
        assert_eq!(diagnostic.phase, ExecutionHandPhase::Failed);
        assert!(
            diagnostic
                .last_error
                .as_deref()
                .is_some_and(|error| error.contains("OPENAI_API_KEY"))
        );
    }

    #[test]
    fn final_rendering_client_rejects_legacy_sift_provider_with_migration_hint() {
        let workspace = tempdir().expect("workspace");
        let (execution_hand_registry, transport_mediator) = transport_mediator_for_test();
        let lane = PreparedModelClient {
            role: ModelClientRole::FinalRendering,
            provider: ModelProvider::Sift,
            model_id: "qwen-1.5b".to_string(),
            paths: None,
        };

        let error = match build_final_renderer(
            workspace.path(),
            execution_hand_registry,
            transport_mediator,
            &lane,
            &BTreeMap::new(),
        ) {
            Ok(_) => panic!("legacy sift final-rendering provider should fail"),
            Err(error) => error,
        };
        let message = format!("{error:#}");

        assert!(
            message.contains("provider `sift` no longer performs model inference"),
            "{message}"
        );
        assert!(message.contains("ollama:<model>"), "{message}");
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

    #[test]
    fn hosted_service_runtime_reports_readiness_and_failure_state() {
        let ready = service_runtime_ready_status(
            &paddles::domain::ports::TraceRecorderCapability::Persistent {
                backend: "hosted_transit".to_string(),
                location: "127.0.0.1:7171#namespace=test;service=svc-main".to_string(),
            },
            OperatorSurfaceRuntimeStatus::Disabled,
            None,
        );

        assert_eq!(ready.state, ServiceRuntimeState::Ready);
        assert_eq!(ready.authority_backend, "hosted_transit".to_string());
        assert_eq!(
            ready.operator_surfaces,
            OperatorSurfaceRuntimeStatus::Disabled
        );
        assert_eq!(ready.failure, None);

        let failed = service_runtime_failure_status(
            &TraceAuthoritySelection::HostedTransit(HostedTransitAuthorityConfig {
                endpoint: "127.0.0.1:7171".to_string(),
                namespace: "test".to_string(),
                service_identity: "svc-main".to_string(),
            }),
            "hosted transit unavailable",
        );

        assert_eq!(failed.state, ServiceRuntimeState::Failed);
        assert_eq!(
            failed.failure.as_deref(),
            Some("hosted transit unavailable")
        );
    }

    #[test]
    fn hosted_service_mode_keeps_operator_surfaces_optional() {
        assert_eq!(
            resolve_operator_surface_bootstrap(false, true),
            OperatorSurfaceBootstrap::Start
        );
        assert_eq!(
            resolve_operator_surface_bootstrap(true, true),
            OperatorSurfaceBootstrap::Start
        );
        assert_eq!(
            resolve_operator_surface_bootstrap(true, false),
            OperatorSurfaceBootstrap::Skip
        );
    }

    #[test]
    fn runtime_entrypoint_smoke_exposes_harness_capability_configuration_posture() {
        let prepared_turn_runtime = PreparedTurnRuntime {
            planner: PreparedModelClient {
                role: ModelClientRole::ActionSelection,
                provider: ModelProvider::Google,
                model_id: "gemini-2.5-flash".to_string(),
                paths: None,
            },
            synthesizer: PreparedModelClient {
                role: ModelClientRole::FinalRendering,
                provider: ModelProvider::Openai,
                model_id: "gpt-5.4".to_string(),
                paths: None,
            },
            retrieval_provider: None,
        };
        let external_capabilities = ExternalCapabilityCatalog::from_local_configuration(
            &ExternalCapabilityCatalogConfig::default().enable("web.search"),
        )
        .descriptors();

        let status = RuntimeHarnessCapabilityPostureService::project(
            &prepared_turn_runtime,
            &external_capabilities,
        );

        assert!(status.external_capabilities.iter().any(|capability| {
            capability.id == "web.search"
                && capability.availability == "available"
                && capability.effects == "read_only"
        }));
        assert!(status.external_capabilities.iter().any(|capability| {
            capability.id == "mcp.tool"
                && capability.availability == "unavailable"
                && capability.auth == "optional"
        }));
        assert_eq!(
            status.execution_policy.profile_id,
            "recursive-structured-v1"
        );
        assert_eq!(status.execution_policy.sandbox_mode, "workspace_write");
        assert!(
            status
                .execution_policy
                .supported_reuse_scopes
                .contains(&"command_prefix".to_string())
        );
        assert!(status.execution_policy.rules.iter().any(|rule| {
            rule.id == "allow-external-capability-through-governance" && rule.decision == "allow"
        }));
        assert!(status.evals.offline);
        assert_eq!(status.evals.failed_reports, 0);
        assert!(
            status
                .evals
                .scenario_ids
                .contains(&"replay-local".to_string())
        );
        assert!(status.provider_registry.offline_safe);
        assert!(!status.provider_registry.network_discovery_required);
        assert!(status.provider_registry.entries.iter().any(|entry| {
            entry.provider == "google"
                && entry.model_id == "gemini-2.5-flash"
                && entry.status == "configured"
        }));
        assert!(status.provider_registry.entries.iter().any(|entry| {
            entry.provider == "openai"
                && entry.model_id == "gpt-5.4"
                && entry.status == "configured"
        }));
    }
}
