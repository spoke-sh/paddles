use crate::application::{ConversationSession, MechSuitService, RuntimeLaneConfig};
use crate::domain::model::{
    ConversationTranscript, ConversationTranscriptSpeaker, ConversationTranscriptUpdate,
    RenderBlock, RenderDocument, RuntimeEventPresentation, ThreadCandidate, TranscriptUpdateSink,
    TurnEvent, TurnEventSink, project_runtime_event,
};
use crate::infrastructure::credentials::{CredentialStore, ProviderAvailability};
use crate::infrastructure::providers::ModelProvider;
use crate::infrastructure::runtime_preferences::{
    RuntimeLanePreferenceStore, RuntimeLanePreferences,
};
use crate::infrastructure::step_timing::{Pace, StepTimingReservoir};
use anyhow::Result;
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size as terminal_size};
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap};
use ratatui::{Frame, Terminal, TerminalOptions, Viewport};
use std::cmp;
use std::collections::{HashSet, VecDeque};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// Context passed from main to the TUI for credential management.
pub struct TuiContext {
    pub credential_store: Arc<CredentialStore>,
    pub runtime_preference_store: Arc<RuntimeLanePreferenceStore>,
    pub runtime_lanes: RuntimeLaneConfig,
    pub verbose: u8,
}

const FRAME_INTERVAL: Duration = Duration::from_millis(32);
const ASSISTANT_REVEAL_STEP: usize = 24;
const EVENT_DETAIL_LINE_LIMIT: usize = 8;
const INLINE_VIEWPORT_MIN_HEIGHT: u16 = 5;
const INLINE_VIEWPORT_MAX_HEIGHT: u16 = 9;
/// Max prose width for assistant responses (accounts for the 4-char body indent).
const MAX_PROSE_WIDTH: usize = 96;
const MULTILINE_INLINE_SEPARATOR: &str = " ⏎ ";

struct SlashCommandSpec {
    insert_text: &'static str,
    usage: &'static str,
    description: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SlashSuggestion {
    insert_text: String,
    usage: String,
    description: String,
}

const SLASH_COMMANDS: &[SlashCommandSpec] = &[
    SlashCommandSpec {
        insert_text: "/login ",
        usage: "/login <provider>",
        description: "store or replace a provider API key",
    },
    SlashCommandSpec {
        insert_text: "/model",
        usage: "/model",
        description: "show the model catalog and auth state",
    },
    SlashCommandSpec {
        insert_text: "/model planner ",
        usage: "/model planner <provider> <model>",
        description: "switch the planner lane",
    },
    SlashCommandSpec {
        insert_text: "/model synthesizer ",
        usage: "/model synthesizer <provider> <model>",
        description: "switch the synthesizer lane",
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractiveFrontend {
    Tui,
    PlainLines,
}

pub fn select_interactive_frontend(
    has_prompt: bool,
    stdin_is_terminal: bool,
    stdout_is_terminal: bool,
) -> InteractiveFrontend {
    if has_prompt || !stdin_is_terminal || !stdout_is_terminal {
        InteractiveFrontend::PlainLines
    } else {
        InteractiveFrontend::Tui
    }
}

pub async fn run_interactive_tui(
    service: Arc<MechSuitService>,
    mut tui_ctx: TuiContext,
) -> Result<()> {
    let _terminal_session = TerminalSession::enter()?;
    let backend = ratatui::backend::CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(inline_viewport_height()),
        },
    )?;

    let (tx, mut rx) = unbounded_channel();
    let session = service.shared_conversation_session();
    service
        .register_transcript_observer(Arc::new(InteractiveTranscriptUpdateSink { tx: tx.clone() }));
    let provider_availability = tui_ctx.credential_store.all_provider_availability();
    let mut app = InteractiveApp::new(
        runtime_lane_summary(&tui_ctx.runtime_lanes),
        detect_palette(),
        session.clone(),
        tui_ctx
            .runtime_lanes
            .synthesizer_provider()
            .name()
            .to_string(),
        tui_ctx
            .runtime_lanes
            .synthesizer_provider()
            .supports_interactive_login()
            .then(|| {
                tui_ctx
                    .runtime_lanes
                    .synthesizer_provider()
                    .name()
                    .to_string()
            }),
        credential_status_line(
            tui_ctx.runtime_lanes.synthesizer_provider(),
            availability_for_provider(
                &provider_availability,
                tui_ctx.runtime_lanes.synthesizer_provider(),
            ),
        ),
        tui_ctx.verbose,
    );
    app.set_runtime_preference_path(tui_ctx.runtime_preference_store.path().to_path_buf());
    app.set_runtime_catalog(tui_ctx.runtime_lanes.clone(), provider_availability);
    match service.prompt_history() {
        Ok(prompt_history) => app.load_prompt_history(prompt_history),
        Err(err) => app.push_error(
            "Prompt history unavailable",
            format!("Could not load persisted prompt history: {err:#}"),
        ),
    }
    if let Ok(transcript) = service.replay_conversation_transcript(&session.task_id()) {
        app.load_transcript(&transcript);
    }

    loop {
        drain_messages(&mut app, &mut rx);
        if app.take_transcript_sync_request()
            && let Ok(transcript) = service.replay_conversation_transcript(&session.task_id())
        {
            app.sync_transcript(&transcript);
        }
        app.tick();
        if let Some(prompt) = app.dispatch_next_prompt() {
            dispatch_prompt(prompt, Arc::clone(&service), session.clone(), tx.clone());
        }

        // Handle completed /login actions.
        if let Some(login) = app.take_pending_login() {
            match tui_ctx
                .credential_store
                .save_api_key(&login.provider, &login.api_key)
            {
                Ok(()) => match service.prepare_runtime_lanes(&tui_ctx.runtime_lanes).await {
                    Ok(_) => {
                        app.set_provider_availability(
                            tui_ctx.credential_store.all_provider_availability(),
                        );
                        app.push_event(
                            "API key saved",
                            format!(
                                "Credentials stored for `{}`. Runtime reconnected.",
                                login.provider,
                            ),
                        );
                    }
                    Err(err) => {
                        app.set_provider_availability(
                            tui_ctx.credential_store.all_provider_availability(),
                        );
                        app.push_error(
                            "Login failed",
                            format!("Key saved but runtime rebuild failed: {err:#}",),
                        );
                    }
                },
                Err(err) => {
                    app.push_error(
                        "Login failed",
                        format!("Could not save credentials: {err:#}"),
                    );
                }
            }
        }

        if let Some(update) = app.take_pending_runtime_update() {
            match service.prepare_runtime_lanes(&update.runtime_lanes).await {
                Ok(_) => {
                    tui_ctx.runtime_lanes = update.runtime_lanes.clone();
                    app.set_runtime_catalog(
                        tui_ctx.runtime_lanes.clone(),
                        tui_ctx.credential_store.all_provider_availability(),
                    );
                    match tui_ctx
                        .runtime_preference_store
                        .save(&update.persisted_preferences)
                    {
                        Ok(()) => app.push_event(
                            "Model selection updated",
                            format!(
                                "{}\nSaved runtime lane preferences to `{}`.",
                                update.summary,
                                tui_ctx.runtime_preference_store.path().display()
                            ),
                        ),
                        Err(err) => app.push_error(
                            "Runtime preference save failed",
                            format!(
                                "{}\nThe lane switch is active, but `{}` could not be updated: {err:#}",
                                update.summary,
                                tui_ctx.runtime_preference_store.path().display()
                            ),
                        ),
                    }
                }
                Err(err) => {
                    app.push_error(
                        "Model selection failed",
                        format!("Could not activate requested runtime lanes: {err:#}"),
                    );
                }
            }
        }

        flush_scrollback_rows(&mut terminal, &mut app)?;
        terminal.draw(|frame| app.render(frame))?;

        if event::poll(FRAME_INTERVAL)? {
            match event::read()? {
                Event::Key(key) if key.kind != KeyEventKind::Release => {
                    if handle_key_event(&mut app, key) {
                        break;
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }

    Ok(())
}

fn drain_messages(app: &mut InteractiveApp, rx: &mut UnboundedReceiver<UiMessage>) {
    while let Ok(message) = rx.try_recv() {
        app.handle_message(message);
    }
}

fn handle_key_event(app: &mut InteractiveApp, key: KeyEvent) -> bool {
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
        if app.input.is_empty() {
            return true;
        }
        app.input.clear();
        app.cursor_pos = 0;
        return false;
    }

    match key.code {
        KeyCode::Esc => {
            if matches!(app.input_mode, InputMode::MaskedKey { .. }) {
                app.input.clear();
                app.cursor_pos = 0;
                app.input_mode = InputMode::Normal;
                app.push_event("Login cancelled", "Returned to normal input.");
                false
            } else {
                if app.input.is_empty() {
                    app.cancel_latest_queued_steering_prompt();
                }
                false
            }
        }
        KeyCode::Enter => {
            app.submit_prompt();
            false
        }
        KeyCode::Tab => {
            app.accept_selected_slash_completion();
            false
        }
        KeyCode::Backspace => {
            if app.cursor_pos > 0 {
                let byte_pos = app
                    .input
                    .char_indices()
                    .nth(app.cursor_pos - 1)
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                let end_byte = app
                    .input
                    .char_indices()
                    .nth(app.cursor_pos)
                    .map(|(i, _)| i)
                    .unwrap_or(app.input.len());
                app.input.replace_range(byte_pos..end_byte, "");
                app.cursor_pos -= 1;
                app.reset_slash_suggestion_selection();
            }
            false
        }
        KeyCode::Delete => {
            let char_count = app.input.chars().count();
            if app.cursor_pos < char_count {
                let byte_pos = app
                    .input
                    .char_indices()
                    .nth(app.cursor_pos)
                    .map(|(i, _)| i)
                    .unwrap_or(app.input.len());
                let end_byte = app
                    .input
                    .char_indices()
                    .nth(app.cursor_pos + 1)
                    .map(|(i, _)| i)
                    .unwrap_or(app.input.len());
                app.input.replace_range(byte_pos..end_byte, "");
                app.reset_slash_suggestion_selection();
            }
            false
        }
        KeyCode::Left => {
            if app.cursor_pos > 0 {
                app.cursor_pos -= 1;
            }
            false
        }
        KeyCode::Right => {
            let char_count = app.input.chars().count();
            if app.cursor_pos < char_count {
                app.cursor_pos += 1;
            }
            false
        }
        KeyCode::Home => {
            app.cursor_pos = 0;
            false
        }
        KeyCode::End => {
            app.cursor_pos = app.input.chars().count();
            false
        }
        KeyCode::Up => {
            if app.cycle_slash_suggestion(-1) {
                return false;
            }
            if !app.cursor_up() && !app.input.contains('\n') {
                app.history_back();
            }
            false
        }
        KeyCode::Down => {
            if app.cycle_slash_suggestion(1) {
                return false;
            }
            if !app.cursor_down() && !app.input.contains('\n') {
                app.history_forward();
            }
            false
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.input.clear();
            app.cursor_pos = 0;
            app.reset_slash_suggestion_selection();
            false
        }
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.cursor_pos = 0;
            false
        }
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.cursor_pos = app.input.chars().count();
            false
        }
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let byte_pos = app
                .input
                .char_indices()
                .nth(app.cursor_pos)
                .map(|(i, _)| i)
                .unwrap_or(app.input.len());
            app.input.insert(byte_pos, '\n');
            app.cursor_pos += 1;
            app.history_cursor = None;
            app.reset_slash_suggestion_selection();
            false
        }
        KeyCode::Char(ch) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                let byte_pos = app
                    .input
                    .char_indices()
                    .nth(app.cursor_pos)
                    .map(|(i, _)| i)
                    .unwrap_or(app.input.len());
                app.input.insert(byte_pos, ch);
                app.cursor_pos += 1;
                app.history_cursor = None;
                app.reset_slash_suggestion_selection();
            }
            false
        }
        _ => false,
    }
}

struct TerminalSession;

impl TerminalSession {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), cursor::Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), cursor::Show);
        let _ = writeln!(io::stdout());
    }
}

#[derive(Debug)]
enum UiMessage {
    TurnEvent {
        event: TurnEvent,
        occurred_at: Instant,
    },
    TranscriptUpdated {
        update: ConversationTranscriptUpdate,
    },
    TurnFinished {
        result: std::result::Result<String, String>,
        occurred_at: Instant,
    },
}

impl UiMessage {
    fn turn_event(event: TurnEvent) -> Self {
        Self::TurnEvent {
            event,
            occurred_at: Instant::now(),
        }
    }

    fn turn_finished(result: std::result::Result<String, String>) -> Self {
        Self::TurnFinished {
            result,
            occurred_at: Instant::now(),
        }
    }

    fn transcript_updated(update: ConversationTranscriptUpdate) -> Self {
        Self::TranscriptUpdated { update }
    }
}

fn dispatch_prompt(
    prompt: QueuedPrompt,
    service: Arc<MechSuitService>,
    session: ConversationSession,
    tx: UnboundedSender<UiMessage>,
) {
    let sink = Arc::new(InteractiveTurnEventSink { tx: tx.clone() });
    tokio::spawn(async move {
        let result = match prompt {
            QueuedPrompt::Prompt(prompt) => {
                service
                    .process_prompt_in_session_with_sink(&prompt, session, sink)
                    .await
            }
            QueuedPrompt::Steering(candidate) => {
                service
                    .process_thread_candidate_in_session_with_sink(candidate, session, sink)
                    .await
            }
        }
        .map_err(|err| format!("{err:#}"));
        let _ = tx.send(UiMessage::turn_finished(result));
    });
}

#[derive(Clone)]
struct InteractiveTurnEventSink {
    tx: UnboundedSender<UiMessage>,
}

impl TurnEventSink for InteractiveTurnEventSink {
    fn emit(&self, event: TurnEvent) {
        let _ = self.tx.send(UiMessage::turn_event(event));
    }
}

#[derive(Clone)]
struct InteractiveTranscriptUpdateSink {
    tx: UnboundedSender<UiMessage>,
}

impl TranscriptUpdateSink for InteractiveTranscriptUpdateSink {
    fn emit(&self, update: ConversationTranscriptUpdate) {
        let _ = self.tx.send(UiMessage::transcript_updated(update));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TranscriptRowKind {
    User,
    Assistant,
    Event,
    CommandNotice,
    /// A started event that only materialized because the operation took >2s.
    InFlightEvent,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TranscriptRow {
    kind: TranscriptRowKind,
    header: String,
    content: String,
    render: Option<RenderDocument>,
    timing: Option<TranscriptTiming>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HuntingTelemetrySample {
    phase: String,
    headline: Option<ProgressHeadline>,
    metrics: Vec<ProgressMetric>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProgressHeadline {
    label: String,
    current: u64,
    total: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProgressMetric {
    label: String,
    value: u64,
}

impl TranscriptRow {
    fn new(kind: TranscriptRowKind, header: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            kind,
            header: header.into(),
            content: content.into(),
            render: None,
            timing: None,
        }
    }

    fn with_render(mut self, render: RenderDocument) -> Self {
        self.render = Some(render);
        self
    }

    fn estimated_height(&self, width: usize) -> usize {
        let width = width.max(8);
        let body_width = width.saturating_sub(4).max(1);
        let mut lines = wrapped_line_count(&self.display_header(), width);
        if self.content.is_empty() {
            if let Some(render) = &self.render {
                return lines + assistant_render_line_count(render, body_width) + 1;
            }
            return lines + 1;
        }

        for line in self.display_content().lines() {
            lines += wrapped_line_count(line, body_width);
        }
        lines + 1
    }

    fn timed(mut self, timing: TranscriptTiming) -> Self {
        self.timing = Some(timing);
        self
    }

    fn display_content(&self) -> String {
        self.render
            .as_ref()
            .map(RenderDocument::to_plain_text)
            .filter(|content| !content.is_empty())
            .unwrap_or_else(|| self.content.clone())
    }

    fn display_header(&self) -> String {
        match self.timing {
            Some(timing) => format!("{} · {}", self.header, timing.label()),
            None => self.header.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TranscriptTiming {
    elapsed: Duration,
    delta: Option<Duration>,
    kind: TranscriptTimingKind,
    pace: Pace,
}

impl TranscriptTiming {
    fn elapsed_label(&self) -> String {
        let elapsed = format_duration_compact(self.elapsed);
        let suffix = match self.kind {
            TranscriptTimingKind::Step => "",
            TranscriptTimingKind::TurnTotal => " total",
        };
        format!("{elapsed}{suffix}")
    }

    fn delta_label(&self) -> Option<String> {
        self.delta
            .map(|delta| format!(" (+{})", format_duration_compact(delta)))
    }

    fn label(&self) -> String {
        let delta = self.delta_label().unwrap_or_default();
        format!("{}{delta}", self.elapsed_label())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TranscriptTimingKind {
    Step,
    TurnTotal,
}

#[derive(Clone, Copy, Debug)]
struct ActiveTurnTiming {
    started_at: Instant,
    last_step_at: Instant,
    saw_step: bool,
}

impl ActiveTurnTiming {
    fn new(started_at: Instant) -> Self {
        Self {
            started_at,
            last_step_at: started_at,
            saw_step: false,
        }
    }

    fn mark_step(&mut self, occurred_at: Instant, pace: Pace) -> TranscriptTiming {
        let timing = TranscriptTiming {
            elapsed: occurred_at.duration_since(self.started_at),
            delta: self
                .saw_step
                .then(|| occurred_at.duration_since(self.last_step_at)),
            kind: TranscriptTimingKind::Step,
            pace,
        };
        self.last_step_at = occurred_at;
        self.saw_step = true;
        timing
    }

    fn finish(mut self, occurred_at: Instant) -> TranscriptTiming {
        let timing = TranscriptTiming {
            elapsed: occurred_at.duration_since(self.started_at),
            delta: self
                .saw_step
                .then(|| occurred_at.duration_since(self.last_step_at)),
            kind: TranscriptTimingKind::TurnTotal,
            pace: Pace::Normal,
        };
        self.last_step_at = occurred_at;
        timing
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingReveal {
    row_index: usize,
    full_text: String,
    visible_chars: usize,
    render: Option<RenderDocument>,
}

impl PendingReveal {
    fn new(row_index: usize, full_text: String, render: Option<RenderDocument>) -> Self {
        Self {
            row_index,
            full_text,
            visible_chars: 0,
            render,
        }
    }

    fn advance(&mut self) -> bool {
        let total_chars = self.full_text.chars().count();
        if self.visible_chars >= total_chars {
            return true;
        }

        self.visible_chars = cmp::min(self.visible_chars + ASSISTANT_REVEAL_STEP, total_chars);
        self.visible_chars >= total_chars
    }

    fn visible_text(&self) -> String {
        self.full_text.chars().take(self.visible_chars).collect()
    }
}

/// After this duration of silence during a busy turn, an in-flight row appears.
const IN_FLIGHT_SILENCE_THRESHOLD: Duration = Duration::from_secs(2);
const HUNTING_HISTORY_MIN_INTERVAL: Duration = Duration::from_secs(1);

/// Infer what the system is currently doing based on the last completed event.
fn in_flight_label(last_event: &TurnEvent) -> &'static str {
    last_event.in_flight_label()
}

fn format_in_flight_row(last_event: &TurnEvent) -> TranscriptRow {
    match last_event {
        TurnEvent::GathererSearchProgress {
            phase,
            strategy,
            detail,
            ..
        } => {
            let strategy = strategy
                .as_deref()
                .map(|value| format!(" strategy={value}"))
                .unwrap_or_default();
            TranscriptRow::new(
                TranscriptRowKind::InFlightEvent,
                format!("• Hunting ({phase})...{strategy}"),
                detail.clone().unwrap_or_default(),
            )
        }
        TurnEvent::HarnessState { snapshot }
            if snapshot.chamber == crate::domain::model::HarnessChamber::Gathering =>
        {
            if let Some(in_flight_title) = snapshot
                .governor_policy()
                .should_show_in_flight_row(snapshot.chamber)
            {
                TranscriptRow::new(
                    TranscriptRowKind::InFlightEvent,
                    in_flight_title,
                    snapshot.detail.clone().unwrap_or_default(),
                )
            } else {
                TranscriptRow::new(
                    TranscriptRowKind::InFlightEvent,
                    format!("• {}...", in_flight_label(last_event)),
                    "".to_string(),
                )
            }
        }
        event => TranscriptRow::new(
            TranscriptRowKind::InFlightEvent,
            format!("• {}...", in_flight_label(event)),
            "",
        ),
    }
}

fn transcript_row_from_runtime_event(presentation: RuntimeEventPresentation) -> TranscriptRow {
    let content = if presentation.detail.is_empty() {
        String::new()
    } else {
        collapse_event_details(&presentation.detail, EVENT_DETAIL_LINE_LIMIT)
    };

    TranscriptRow::new(TranscriptRowKind::Event, presentation.title, content)
}

struct InteractiveApp {
    model_label: String,
    palette: Palette,
    session: ConversationSession,
    runtime_lanes: RuntimeLaneConfig,
    provider_availability: Vec<ProviderAvailability>,
    rows: Vec<TranscriptRow>,
    input: String,
    queued_prompts: VecDeque<QueuedPrompt>,
    busy: bool,
    busy_phase: BusyPhase,
    pending_reveal: Option<PendingReveal>,
    spinner_index: usize,
    input_mode: InputMode,
    pending_login: Option<PendingLogin>,
    pending_runtime_update: Option<PendingRuntimeUpdate>,
    slash_suggestion_index: usize,
    provider_name: String,
    credential_provider: Option<String>,
    runtime_preference_path: Option<PathBuf>,
    active_turn_timing: Option<ActiveTurnTiming>,
    flushed_row_count: usize,
    search_progress_row: Option<usize>,
    gathering_harness_row: Option<usize>,
    planner_progress_row: Option<usize>,
    last_hunting_sample: Option<HuntingTelemetrySample>,
    last_hunting_history_sample: Option<HuntingTelemetrySample>,
    last_hunting_history_at: Option<Instant>,
    in_flight_row: Option<usize>,
    last_event: Option<(TurnEvent, Instant)>,
    emitted_in_flight: bool,
    step_timing: StepTimingReservoir,
    step_timing_path: PathBuf,
    verbose: u8,
    cursor_pos: usize,
    prompt_history: Vec<String>,
    history_cursor: Option<usize>,
    history_draft: String,
    current_task_id: String,
    pending_transcript_sync: bool,
    seen_transcript_record_ids: HashSet<String>,
    pending_turn_total_timing: Option<TranscriptTiming>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum QueuedPrompt {
    Prompt(String),
    Steering(ThreadCandidate),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BusyPhase {
    Idle,
    Thinking,
    Rendering,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum InputMode {
    Normal,
    MaskedKey { provider: String },
}

#[derive(Clone, Debug)]
struct PendingLogin {
    provider: String,
    api_key: String,
}

#[derive(Clone, Debug)]
struct PendingRuntimeUpdate {
    runtime_lanes: RuntimeLaneConfig,
    persisted_preferences: RuntimeLanePreferences,
    summary: String,
}

fn runtime_lane_summary(runtime_lanes: &RuntimeLaneConfig) -> String {
    format!(
        "P {} · S {}",
        runtime_lanes.planner_provider().qualified_model_label(
            runtime_lanes
                .planner_model_id()
                .unwrap_or(runtime_lanes.synthesizer_model_id()),
        ),
        runtime_lanes
            .synthesizer_provider()
            .qualified_model_label(runtime_lanes.synthesizer_model_id())
    )
}

fn availability_for_provider(
    availability: &[ProviderAvailability],
    provider: ModelProvider,
) -> ProviderAvailability {
    availability
        .iter()
        .find(|entry| entry.provider == provider)
        .cloned()
        .unwrap_or(ProviderAvailability {
            provider,
            enabled: matches!(
                provider.auth_requirement(),
                crate::infrastructure::providers::ProviderAuthRequirement::None
                    | crate::infrastructure::providers::ProviderAuthRequirement::OptionalApiKey
            ),
            detail: match provider.auth_requirement() {
                crate::infrastructure::providers::ProviderAuthRequirement::None => {
                    "auth not required".to_string()
                }
                crate::infrastructure::providers::ProviderAuthRequirement::OptionalApiKey => {
                    "auth not required".to_string()
                }
                crate::infrastructure::providers::ProviderAuthRequirement::RequiredApiKey => {
                    "login required".to_string()
                }
            },
        })
}

fn credential_status_line(provider: ModelProvider, availability: ProviderAvailability) -> String {
    match provider.auth_requirement() {
        crate::infrastructure::providers::ProviderAuthRequirement::None => {
            format!(
                "Provider: `{}` (local-first). Auth: not required.",
                provider.name()
            )
        }
        _ => format!(
            "Provider: `{}`. Auth: {}.",
            provider.name(),
            availability.detail
        ),
    }
}

impl InteractiveApp {
    fn new(
        model_label: String,
        palette: Palette,
        session: ConversationSession,
        provider_name: String,
        credential_provider: Option<String>,
        credential_status: String,
        verbose: u8,
    ) -> Self {
        let ready_message = match credential_provider.as_deref() {
            Some(provider) => format!(
                "Enter to send, Ctrl+C to quit.\n\
                 {credential_status}\n\
                 Type `/login {provider}` to set or replace its API key.\n\
                 Type `/model` to inspect or switch planner and synthesizer lanes.\n\
                 Slash commands open a popup; Tab accepts the active completion.",
            ),
            None => format!(
                "Enter to send, Ctrl+C to quit.\n\
                 {credential_status}\n\
                 Type `/login <provider>` for any remote provider.\n\
                 Type `/model` to inspect or switch planner and synthesizer lanes.\n\
                 Slash commands open a popup; Tab accepts the active completion.",
            ),
        };
        let current_task_id = session.task_id().as_str().to_string();
        let runtime_lanes = RuntimeLaneConfig::new(model_label.clone(), None)
            .with_synthesizer_provider(
                ModelProvider::from_name(&provider_name).unwrap_or(ModelProvider::Sift),
            );
        Self {
            model_label,
            palette,
            session,
            runtime_lanes,
            provider_availability: Vec::new(),
            rows: vec![TranscriptRow::new(
                TranscriptRowKind::Event,
                "• Interactive mode ready",
                ready_message,
            )],
            input: String::new(),
            queued_prompts: VecDeque::new(),
            busy: false,
            busy_phase: BusyPhase::Idle,
            pending_reveal: None,
            spinner_index: 0,
            input_mode: InputMode::Normal,
            pending_login: None,
            pending_runtime_update: None,
            slash_suggestion_index: 0,
            provider_name,
            credential_provider,
            runtime_preference_path: None,
            active_turn_timing: None,
            flushed_row_count: 0,
            search_progress_row: None,
            gathering_harness_row: None,
            planner_progress_row: None,
            last_hunting_sample: None,
            last_hunting_history_sample: None,
            last_hunting_history_at: None,
            in_flight_row: None,
            last_event: None,
            emitted_in_flight: false,
            step_timing: StepTimingReservoir::load(&step_timing_cache_path()),
            step_timing_path: step_timing_cache_path(),
            verbose,
            cursor_pos: 0,
            prompt_history: Vec::new(),
            history_cursor: None,
            history_draft: String::new(),
            current_task_id,
            pending_transcript_sync: false,
            seen_transcript_record_ids: HashSet::new(),
            pending_turn_total_timing: None,
        }
    }

    fn set_runtime_catalog(
        &mut self,
        runtime_lanes: RuntimeLaneConfig,
        provider_availability: Vec<ProviderAvailability>,
    ) {
        self.runtime_lanes = runtime_lanes;
        self.model_label = runtime_lane_summary(&self.runtime_lanes);
        self.provider_name = self.runtime_lanes.synthesizer_provider().name().to_string();
        self.credential_provider = self
            .runtime_lanes
            .synthesizer_provider()
            .supports_interactive_login()
            .then(|| self.runtime_lanes.synthesizer_provider().name().to_string());
        self.provider_availability = provider_availability;
    }

    fn set_runtime_preference_path(&mut self, path: PathBuf) {
        self.runtime_preference_path = Some(path);
    }

    fn set_provider_availability(&mut self, provider_availability: Vec<ProviderAvailability>) {
        self.provider_availability = provider_availability;
    }

    fn reset_slash_suggestion_selection(&mut self) {
        self.slash_suggestion_index = 0;
    }

    fn dynamic_model_slash_suggestions(&self, query: &str) -> Option<Vec<SlashSuggestion>> {
        const PLANNER_PREFIX: &str = "/model planner ";
        const SYNTH_PREFIX: &str = "/model synthesizer ";

        let (prefix, remainder) = if let Some(remainder) = query.strip_prefix(PLANNER_PREFIX) {
            (PLANNER_PREFIX, remainder)
        } else if let Some(remainder) = query.strip_prefix(SYNTH_PREFIX) {
            (SYNTH_PREFIX, remainder)
        } else {
            return None;
        };

        if let Some((provider_name, model_query)) = remainder.split_once(' ') {
            let provider = ModelProvider::from_name(provider_name)?;
            if provider.supports_freeform_model_id() {
                return Some(Vec::new());
            }
            return Some(
                provider
                    .known_model_ids()
                    .iter()
                    .copied()
                    .filter(|model_id| model_id.starts_with(model_query))
                    .map(|model_id| SlashSuggestion {
                        insert_text: format!("{prefix}{provider_name} {model_id}"),
                        usage: format!("{prefix}{provider_name} {model_id}"),
                        description: format!(
                            "select the {} model for this lane",
                            provider.qualified_model_label(model_id)
                        ),
                    })
                    .collect(),
            );
        }

        Some(
            ModelProvider::all()
                .iter()
                .copied()
                .filter(|provider| provider.name().starts_with(remainder))
                .map(|provider| SlashSuggestion {
                    insert_text: format!("{prefix}{} ", provider.name()),
                    usage: format!("{prefix}{} ", provider.name()),
                    description: format!("choose {} for this lane", provider.display_name()),
                })
                .collect(),
        )
    }

    fn slash_command_suggestions(&self) -> Vec<SlashSuggestion> {
        if self.is_masked_input() || self.input.contains('\n') || !self.input.starts_with('/') {
            return Vec::new();
        }
        let query = self.input.to_ascii_lowercase();
        if let Some(provider_query) = query.strip_prefix("/login ") {
            return ModelProvider::all()
                .iter()
                .copied()
                .filter(|provider| provider.supports_interactive_login())
                .filter(|provider| provider.name().starts_with(provider_query))
                .map(|provider| SlashSuggestion {
                    insert_text: format!("/login {}", provider.name()),
                    usage: format!("/login {}", provider.name()),
                    description: format!("store or replace a {} API key", provider.display_name()),
                })
                .collect();
        }
        if let Some(suggestions) = self.dynamic_model_slash_suggestions(&query) {
            return suggestions;
        }
        SLASH_COMMANDS
            .iter()
            .filter(|command| {
                command.insert_text.starts_with(&query) || command.usage.starts_with(&query)
            })
            .map(|command| SlashSuggestion {
                insert_text: command.insert_text.to_string(),
                usage: command.usage.to_string(),
                description: command.description.to_string(),
            })
            .collect()
    }

    fn selected_slash_suggestion(&self) -> Option<SlashSuggestion> {
        let suggestions = self.slash_command_suggestions();
        if suggestions.is_empty() {
            None
        } else {
            Some(suggestions[self.slash_suggestion_index.min(suggestions.len() - 1)].clone())
        }
    }

    fn cycle_slash_suggestion(&mut self, delta: isize) -> bool {
        let suggestions = self.slash_command_suggestions();
        if suggestions.is_empty() || self.input.contains('\n') {
            return false;
        }
        let len = suggestions.len() as isize;
        let next = (self.slash_suggestion_index as isize + delta).rem_euclid(len);
        self.slash_suggestion_index = next as usize;
        true
    }

    fn accept_selected_slash_completion(&mut self) -> bool {
        let Some(suggestion) = self.selected_slash_suggestion() else {
            return false;
        };
        if suggestion.insert_text == self.input {
            return false;
        }
        if !suggestion.insert_text.starts_with(&self.input) {
            return false;
        }
        self.input = suggestion.insert_text.to_string();
        self.cursor_pos = self.input.chars().count();
        self.history_cursor = None;
        true
    }

    fn history_back(&mut self) {
        if self.prompt_history.is_empty() {
            return;
        }
        let index = match self.history_cursor {
            None => {
                self.history_draft = self.input.clone();
                self.prompt_history.len() - 1
            }
            Some(0) => return,
            Some(i) => i - 1,
        };
        self.history_cursor = Some(index);
        self.input = self.prompt_history[index].clone();
        self.cursor_pos = self.input.chars().count();
    }

    fn history_forward(&mut self) {
        let Some(cursor) = self.history_cursor else {
            return;
        };
        if cursor + 1 < self.prompt_history.len() {
            self.history_cursor = Some(cursor + 1);
            self.input = self.prompt_history[cursor + 1].clone();
        } else {
            self.history_cursor = None;
            self.input = self.history_draft.clone();
        }
        self.cursor_pos = self.input.chars().count();
    }

    fn load_prompt_history(&mut self, prompt_history: Vec<String>) {
        self.prompt_history = prompt_history
            .into_iter()
            .filter(|prompt| !prompt.trim().is_empty())
            .collect();
        self.history_cursor = None;
        self.history_draft.clear();
    }

    fn cancel_latest_queued_steering_prompt(&mut self) -> bool {
        let Some(index) = self
            .queued_prompts
            .iter()
            .rposition(|prompt| matches!(prompt, QueuedPrompt::Steering(_)))
        else {
            return false;
        };

        let Some(QueuedPrompt::Steering(candidate)) = self.queued_prompts.remove(index) else {
            return false;
        };

        self.rows.push(TranscriptRow::new(
            TranscriptRowKind::Event,
            "• Cancelled queued steering prompt",
            format!(
                "`{}` removed from the queue.",
                trim_for_display(&inline_multiline_text(&candidate.prompt), 96)
            ),
        ));
        true
    }

    /// Returns (line_index, column, line_start_char_offset, line_char_len) for the cursor.
    fn cursor_line_info(&self) -> (usize, usize, usize, usize) {
        let mut offset = 0;
        for (i, line) in self.input.split('\n').enumerate() {
            let line_len = line.chars().count();
            if self.cursor_pos <= offset + line_len {
                return (i, self.cursor_pos - offset, offset, line_len);
            }
            offset += line_len + 1; // +1 for the newline
        }
        // cursor is at the very end
        let lines: Vec<&str> = self.input.split('\n').collect();
        let last = lines.len().saturating_sub(1);
        let last_len = lines.last().map(|l| l.chars().count()).unwrap_or(0);
        (
            last,
            last_len,
            self.input.chars().count() - last_len,
            last_len,
        )
    }

    /// Move cursor up one line, preserving column. Returns true if it moved.
    fn cursor_up(&mut self) -> bool {
        let (line_idx, col, _, _) = self.cursor_line_info();
        if line_idx == 0 {
            return false;
        }
        // Find the previous line
        let lines: Vec<&str> = self.input.split('\n').collect();
        let prev_line_len = lines[line_idx - 1].chars().count();
        let prev_line_start: usize = lines[..line_idx - 1]
            .iter()
            .map(|l| l.chars().count() + 1)
            .sum();
        self.cursor_pos = prev_line_start + col.min(prev_line_len);
        true
    }

    /// Move cursor down one line, preserving column. Returns true if it moved.
    fn cursor_down(&mut self) -> bool {
        let (line_idx, col, _, _) = self.cursor_line_info();
        let lines: Vec<&str> = self.input.split('\n').collect();
        if line_idx + 1 >= lines.len() {
            return false;
        }
        let next_line_len = lines[line_idx + 1].chars().count();
        let next_line_start: usize = lines[..line_idx + 1]
            .iter()
            .map(|l| l.chars().count() + 1)
            .sum();
        self.cursor_pos = next_line_start + col.min(next_line_len);
        true
    }

    fn submit_prompt(&mut self) {
        let raw = self.input.trim().to_string();
        if raw.is_empty() {
            return;
        }
        let raw_display = inline_multiline_text(&raw);
        self.prompt_history.push(raw.clone());
        self.history_cursor = None;
        self.history_draft.clear();
        self.input.clear();
        self.cursor_pos = 0;
        self.reset_slash_suggestion_selection();

        // Handle masked key submission.
        if let InputMode::MaskedKey { ref provider } = self.input_mode {
            let provider = provider.clone();
            self.pending_login = Some(PendingLogin {
                provider,
                api_key: raw,
            });
            self.input_mode = InputMode::Normal;
            return;
        }

        // Handle slash commands.
        if raw.starts_with("/login") {
            self.handle_login_command(&raw);
            return;
        }

        if raw.starts_with("/model") {
            self.handle_model_command(&raw);
            return;
        }

        // Normal prompt submission.
        let was_busy = self.busy || self.pending_reveal.is_some();
        if was_busy || !self.queued_prompts.is_empty() {
            self.queued_prompts.push_back(QueuedPrompt::Steering(
                self.session.capture_candidate(raw.clone()),
            ));
        } else {
            self.queued_prompts
                .push_back(QueuedPrompt::Prompt(raw.clone()));
        }
        if was_busy || self.queued_prompts.len() > 1 {
            self.rows.push(TranscriptRow::new(
                TranscriptRowKind::Event,
                "• Queued steering prompt",
                format!(
                    "`{}` queued behind the active turn.",
                    trim_for_display(&raw_display, 96)
                ),
            ));
        }
    }

    fn take_pending_login(&mut self) -> Option<PendingLogin> {
        self.pending_login.take()
    }

    fn take_pending_runtime_update(&mut self) -> Option<PendingRuntimeUpdate> {
        self.pending_runtime_update.take()
    }

    fn handle_login_command(&mut self, raw: &str) {
        let mut parts = raw.split_whitespace();
        let _ = parts.next();
        let requested_provider = parts.next().map(str::to_ascii_lowercase);
        let provider = match requested_provider {
            Some(provider) => match ModelProvider::from_name(&provider) {
                Some(provider) => provider,
                None => {
                    self.push_error(
                        "Login unavailable",
                        format!(
                            "Unknown provider `{provider}`. Try one of: {}.",
                            ModelProvider::all()
                                .iter()
                                .map(|provider| provider.name())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    );
                    return;
                }
            },
            None => match self
                .credential_provider
                .as_deref()
                .and_then(ModelProvider::from_name)
            {
                Some(provider) => provider,
                None => {
                    self.push_error(
                        "Login unavailable",
                        format!(
                            "The current provider `{}` does not use API-key login. Use `/login <provider>` instead.",
                            self.provider_name
                        ),
                    );
                    return;
                }
            },
        };

        if !provider.supports_interactive_login() {
            self.push_error(
                "Login unavailable",
                format!("Provider `{}` does not use API-key login.", provider.name()),
            );
            return;
        }

        self.rows.push(TranscriptRow::new(
            TranscriptRowKind::Event,
            "• Login",
            format!(
                "Enter your API key for `{}`. Input is masked.\n\
                 Press Esc to cancel.",
                provider.name()
            ),
        ));
        self.input_mode = InputMode::MaskedKey {
            provider: provider.name().to_string(),
        };
    }

    fn handle_model_command(&mut self, raw: &str) {
        let parts = raw.split_whitespace().collect::<Vec<_>>();
        if parts.len() == 1 {
            self.push_event("Model catalog", self.render_model_catalog());
            return;
        }
        if parts.len() < 4 {
            self.push_error(
                "Model command invalid",
                "Use `/model`, `/model synthesizer <provider> <model>`, or `/model planner <provider> <model>`.",
            );
            return;
        }

        let lane = parts[1].to_ascii_lowercase();
        let provider_name = parts[2].to_ascii_lowercase();
        let model_id = parts[3..].join(" ");
        let provider = match ModelProvider::from_name(&provider_name) {
            Some(provider) => provider,
            None => {
                self.push_error(
                    "Model command invalid",
                    format!("Unknown provider `{provider_name}`."),
                );
                return;
            }
        };
        let availability = self.provider_availability_for(provider);
        if !availability.enabled {
            self.push_error(
                "Model unavailable",
                format!(
                    "Provider `{}` is disabled: {}.",
                    provider.name(),
                    availability.detail
                ),
            );
            return;
        }
        let normalized_model = provider.normalize_model_alias(&model_id);
        if !provider.accepts_model(&normalized_model) {
            let accepted = if provider.supports_freeform_model_id() {
                "any model id".to_string()
            } else {
                provider.known_model_ids().join(", ")
            };
            self.push_error(
                "Model command invalid",
                format!(
                    "Provider `{}` does not recognize model `{}`. Accepted: {}.",
                    provider.name(),
                    normalized_model,
                    accepted
                ),
            );
            return;
        }

        let runtime_lanes = match lane.as_str() {
            "planner" => {
                let runtime_lanes = self
                    .runtime_lanes
                    .clone()
                    .with_planner_provider(Some(provider))
                    .with_planner_model_id(Some(normalized_model.clone()));
                self.pending_runtime_update = Some(PendingRuntimeUpdate {
                    persisted_preferences: RuntimeLanePreferences::from_runtime_lanes(
                        &runtime_lanes,
                    ),
                    runtime_lanes,
                    summary: format!(
                        "Planner lane now targets `{}`.",
                        provider.qualified_model_label(&normalized_model)
                    ),
                });
                return;
            }
            "synth" | "synthesizer" => RuntimeLaneConfig::new(
                normalized_model.clone(),
                self.runtime_lanes.gatherer_model_id().map(str::to_string),
            )
            .with_synthesizer_provider(provider)
            .with_planner_model_id(self.runtime_lanes.planner_model_id().map(str::to_string))
            .with_planner_provider(self.runtime_lanes.planner_provider_override())
            .with_gatherer_provider(self.runtime_lanes.gatherer_provider())
            .with_context1_harness_ready(self.runtime_lanes.context1_harness_ready()),
            _ => {
                self.push_error(
                    "Model command invalid",
                    format!(
                        "Unknown lane `{}`. Use `planner` or `synthesizer`.",
                        parts[1]
                    ),
                );
                return;
            }
        };
        self.pending_runtime_update = Some(PendingRuntimeUpdate {
            persisted_preferences: RuntimeLanePreferences::from_runtime_lanes(&runtime_lanes),
            runtime_lanes,
            summary: format!(
                "Synthesizer lane now targets `{}`.",
                provider.qualified_model_label(&normalized_model)
            ),
        });
    }

    fn render_model_catalog(&self) -> String {
        let mut lines = vec![
            format!("Active: {}", runtime_lane_summary(&self.runtime_lanes)),
            format!(
                "Planner: {}",
                self.runtime_lanes.planner_provider().qualified_model_label(
                    self.runtime_lanes
                        .planner_model_id()
                        .unwrap_or(self.runtime_lanes.synthesizer_model_id()),
                )
            ),
            format!(
                "Synthesizer: {}",
                self.runtime_lanes
                    .synthesizer_provider()
                    .qualified_model_label(self.runtime_lanes.synthesizer_model_id())
            ),
            String::new(),
            "Providers:".to_string(),
        ];
        for provider in ModelProvider::all() {
            let availability = self.provider_availability_for(*provider);
            let models = if provider.supports_freeform_model_id() {
                "<freeform model id>".to_string()
            } else {
                provider.known_model_ids().join(", ")
            };
            let status = if availability.enabled {
                "enabled"
            } else {
                "disabled"
            };
            lines.push(format!(
                "- [{status}] {}: {} ({})",
                provider.name(),
                models,
                availability.detail
            ));
        }
        lines.push(String::new());
        if let Some(path) = &self.runtime_preference_path {
            lines.push(format!("Runtime lane state: {}", path.display()));
            lines.push(
                "Workspace `paddles.toml` overrides this machine-managed state when both are present."
                    .to_string(),
            );
            lines.push(String::new());
        }
        lines.push(
            "Use `/model synthesizer <provider> <model>` or `/model planner <provider> <model>`."
                .to_string(),
        );
        lines.join("\n")
    }

    fn provider_availability_for(&self, provider: ModelProvider) -> ProviderAvailability {
        availability_for_provider(&self.provider_availability, provider)
    }

    fn push_event(&mut self, header: impl Into<String>, content: impl Into<String>) {
        self.rows.push(TranscriptRow::new(
            TranscriptRowKind::CommandNotice,
            format!("• {}", header.into()),
            content,
        ));
    }

    fn push_error(&mut self, header: impl Into<String>, content: impl Into<String>) {
        self.rows.push(TranscriptRow::new(
            TranscriptRowKind::Error,
            format!("• {}", header.into()),
            content,
        ));
    }

    fn dispatch_next_prompt(&mut self) -> Option<QueuedPrompt> {
        self.dispatch_next_prompt_at(Instant::now())
    }

    fn take_transcript_sync_request(&mut self) -> bool {
        let pending = self.pending_transcript_sync;
        self.pending_transcript_sync = false;
        pending
    }

    fn load_transcript(&mut self, transcript: &ConversationTranscript) {
        self.sync_transcript_with_mode(transcript, false);
    }

    fn sync_transcript(&mut self, transcript: &ConversationTranscript) {
        self.sync_transcript_with_mode(transcript, true);
    }

    fn sync_transcript_with_mode(
        &mut self,
        transcript: &ConversationTranscript,
        animate_new_assistant: bool,
    ) {
        if transcript.task_id.as_str() != self.current_task_id {
            return;
        }

        for entry in &transcript.entries {
            let record_id = entry.record_id.as_str().to_string();
            if self.seen_transcript_record_ids.contains(&record_id) {
                continue;
            }

            match entry.speaker {
                ConversationTranscriptSpeaker::User => {
                    self.rows.push(TranscriptRow::new(
                        TranscriptRowKind::User,
                        "User",
                        inline_multiline_text(&entry.content),
                    ));
                }
                ConversationTranscriptSpeaker::Assistant => {
                    let wrapped = soft_wrap_prose(&entry.content, MAX_PROSE_WIDTH);
                    let render = entry.render.clone();
                    if animate_new_assistant {
                        let row_index = self.rows.len();
                        let mut row =
                            TranscriptRow::new(TranscriptRowKind::Assistant, "Paddles", "");
                        if let Some(timing) = self.pending_turn_total_timing.take() {
                            row = row.timed(timing);
                        }
                        self.rows.push(row);
                        self.pending_reveal = Some(PendingReveal::new(row_index, wrapped, render));
                        self.busy = true;
                        self.busy_phase = BusyPhase::Rendering;
                    } else {
                        let mut row =
                            TranscriptRow::new(TranscriptRowKind::Assistant, "Paddles", wrapped);
                        if let Some(render) = render {
                            row = row.with_render(render);
                        }
                        if let Some(timing) = self.pending_turn_total_timing.take() {
                            row = row.timed(timing);
                        }
                        self.rows.push(row);
                    }
                }
            }

            self.seen_transcript_record_ids.insert(record_id);
        }
    }

    fn dispatch_next_prompt_at(&mut self, started_at: Instant) -> Option<QueuedPrompt> {
        if self.busy {
            return None;
        }

        let prompt = self.queued_prompts.pop_front()?;
        self.busy = true;
        self.busy_phase = BusyPhase::Thinking;
        self.active_turn_timing = Some(ActiveTurnTiming::new(started_at));
        Some(prompt)
    }

    fn should_show_event(&self, event: &TurnEvent, pace: Pace, is_first_step: bool) -> bool {
        if let TurnEvent::HarnessState { snapshot } = event
            && !snapshot.governor_policy().should_emit_to_stream()
        {
            return false;
        }

        if is_first_step {
            return true;
        }
        let base = event.min_verbosity();
        let promotion = match pace {
            Pace::Slow => 2,
            Pace::Normal => 1,
            Pace::Fast => 0,
        };
        base.saturating_sub(promotion) <= self.verbose
    }

    fn replace_or_push_tracked_row(
        rows: &mut Vec<TranscriptRow>,
        slot: &mut Option<usize>,
        row: TranscriptRow,
    ) {
        if let Some(idx) = *slot
            && idx < rows.len()
        {
            rows[idx] = row;
            return;
        }

        *slot = Some(rows.len());
        rows.push(row);
    }

    fn remove_in_flight_row(&mut self) {
        let Some(index) = self.in_flight_row.take() else {
            return;
        };
        if index >= self.rows.len() {
            return;
        }

        self.rows.remove(index);
        if self.flushed_row_count > index {
            self.flushed_row_count -= 1;
        }
        if let Some(pending) = self.pending_reveal.as_mut()
            && pending.row_index > index
        {
            pending.row_index -= 1;
        }
        for slot in [
            &mut self.search_progress_row,
            &mut self.gathering_harness_row,
            &mut self.planner_progress_row,
        ] {
            if let Some(slot_index) = *slot {
                if slot_index == index {
                    *slot = None;
                } else if slot_index > index {
                    *slot = Some(slot_index - 1);
                }
            }
        }
    }

    fn insert_hunting_history_row(&mut self, row: TranscriptRow) {
        let insert_at = [
            self.search_progress_row,
            self.gathering_harness_row,
            self.planner_progress_row,
            self.in_flight_row,
        ]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(self.rows.len());

        self.rows.insert(insert_at, row);

        for slot in [
            &mut self.search_progress_row,
            &mut self.gathering_harness_row,
            &mut self.planner_progress_row,
            &mut self.in_flight_row,
        ] {
            if let Some(index) = slot.as_mut()
                && *index >= insert_at
            {
                *index += 1;
            }
        }

        if let Some(pending) = self.pending_reveal.as_mut()
            && pending.row_index >= insert_at
        {
            pending.row_index += 1;
        }
    }

    fn maybe_record_hunting_history(
        &mut self,
        event: &TurnEvent,
        timing: Option<TranscriptTiming>,
        occurred_at: Instant,
    ) {
        let TurnEvent::GathererSearchProgress { phase, detail, .. } = event else {
            return;
        };
        let Some(detail) = detail.as_deref() else {
            self.last_hunting_sample = None;
            self.last_hunting_history_sample = None;
            self.last_hunting_history_at = None;
            return;
        };
        let Some(sample) = parse_hunting_telemetry_sample(phase, detail) else {
            self.last_hunting_sample = None;
            self.last_hunting_history_sample = None;
            self.last_hunting_history_at = None;
            return;
        };

        let should_emit = self
            .last_hunting_history_at
            .map(|last| occurred_at.duration_since(last) >= HUNTING_HISTORY_MIN_INTERVAL)
            .unwrap_or(false);

        if should_emit
            && let Some(previous) = self.last_hunting_history_sample.as_ref()
            && let Some(history_content) = format_hunting_history_delta(previous, &sample)
        {
            let mut row = TranscriptRow::new(
                TranscriptRowKind::Event,
                format!("• Hunting sample ({})", sample.phase),
                history_content,
            );
            if let Some(timing) = timing {
                row = row.timed(timing);
            }
            self.insert_hunting_history_row(row);
            self.last_hunting_history_sample = Some(sample.clone());
            self.last_hunting_history_at = Some(occurred_at);
        } else if self.last_hunting_history_sample.is_none() {
            self.last_hunting_history_sample = Some(sample.clone());
            self.last_hunting_history_at = Some(occurred_at);
        }

        self.last_hunting_sample = Some(sample);
    }

    fn handle_message(&mut self, message: UiMessage) {
        match message {
            UiMessage::TurnEvent { event, occurred_at } => {
                self.remove_in_flight_row();
                let key = event.event_type_key();
                let is_first_step = self
                    .active_turn_timing
                    .as_ref()
                    .is_some_and(|t| !t.saw_step);

                let delta = self.active_turn_timing.as_ref().and_then(|t| {
                    t.saw_step
                        .then(|| occurred_at.duration_since(t.last_step_at))
                });
                if let Some(d) = delta {
                    self.step_timing.record(key, d);
                }
                let pace = delta
                    .map(|d| self.step_timing.classify(key, d))
                    .unwrap_or(Pace::Normal);

                let is_search_progress = event.is_search_progress();
                let is_planner_progress = event.is_planner_progress();
                let is_gathering_harness_progress = event.is_gathering_harness_progress();

                self.last_event = Some((event.clone(), occurred_at));
                self.emitted_in_flight = false;

                if self.should_show_event(&event, pace, is_first_step) {
                    let row = format_turn_event_row(event.clone(), self.verbose);
                    let row = if let Some(timing) = self.active_turn_timing.as_mut() {
                        row.timed(timing.mark_step(occurred_at, pace))
                    } else {
                        row
                    };
                    let row_timing = row.timing;
                    self.maybe_record_hunting_history(&event, row_timing, occurred_at);

                    if is_planner_progress {
                        Self::replace_or_push_tracked_row(
                            &mut self.rows,
                            &mut self.planner_progress_row,
                            row,
                        );
                    } else if is_search_progress {
                        Self::replace_or_push_tracked_row(
                            &mut self.rows,
                            &mut self.search_progress_row,
                            row,
                        );
                    } else if is_gathering_harness_progress {
                        Self::replace_or_push_tracked_row(
                            &mut self.rows,
                            &mut self.gathering_harness_row,
                            row,
                        );
                    } else {
                        self.search_progress_row = None;
                        self.gathering_harness_row = None;
                        self.last_hunting_sample = None;
                        self.last_hunting_history_sample = None;
                        self.last_hunting_history_at = None;
                        self.rows.push(row);
                    }
                } else if let Some(timing) = self.active_turn_timing.as_mut() {
                    timing.mark_step(occurred_at, pace);
                }
            }
            UiMessage::TranscriptUpdated { update } => {
                if update.task_id.as_str() == self.current_task_id {
                    self.pending_transcript_sync = true;
                }
            }
            UiMessage::TurnFinished {
                result,
                occurred_at,
            } => {
                self.remove_in_flight_row();
                self.search_progress_row = None;
                self.gathering_harness_row = None;
                self.planner_progress_row = None;
                self.last_hunting_sample = None;
                self.last_hunting_history_sample = None;
                self.last_hunting_history_at = None;
                self.last_event = None;
                self.emitted_in_flight = false;
                match result {
                    Ok(_response) => {
                        let timing = self
                            .active_turn_timing
                            .take()
                            .map(|timing| timing.finish(occurred_at));
                        self.pending_transcript_sync = true;
                        if let (Some(pending), Some(timing)) = (&self.pending_reveal, timing) {
                            if pending.row_index < self.rows.len() {
                                self.rows[pending.row_index] =
                                    self.rows[pending.row_index].clone().timed(timing);
                            }
                        } else {
                            self.pending_turn_total_timing = timing;
                        }
                        self.busy = true;
                        self.busy_phase = BusyPhase::Rendering;
                    }
                    Err(error) => {
                        self.pending_turn_total_timing = None;
                        let row = self.annotate_turn_total(
                            TranscriptRow::new(TranscriptRowKind::Error, "• Turn failed", error),
                            occurred_at,
                        );
                        self.rows.push(row);
                        self.busy = false;
                        self.busy_phase = BusyPhase::Idle;
                    }
                }
            }
        }
        let _ = self.step_timing.flush(&self.step_timing_path);
    }

    fn tick(&mut self) {
        self.spinner_index = (self.spinner_index + 1) % SPINNER_FRAMES.len();

        // After IN_FLIGHT_SILENCE_THRESHOLD of silence during a busy turn,
        // insert a muted "working" row so the transcript doesn't look stalled.
        if self.busy
            && !self.emitted_in_flight
            && let Some((ref event, last_at)) = self.last_event
        {
            let silence = Instant::now().duration_since(last_at);
            if silence >= IN_FLIGHT_SILENCE_THRESHOLD {
                let row = format_in_flight_row(event);
                Self::replace_or_push_tracked_row(&mut self.rows, &mut self.in_flight_row, row);
                self.emitted_in_flight = true;
            }
        }

        if let Some(pending) = &mut self.pending_reveal {
            let finished = pending.advance();
            if let Some(row) = self.rows.get_mut(pending.row_index) {
                row.content = pending.visible_text();
                if finished {
                    row.render = pending.render.clone();
                }
            }
            if finished {
                self.pending_reveal = None;
                self.busy = false;
                self.busy_phase = BusyPhase::Idle;
            }
        }
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let input_height = self.input_area_height(area.width);
        let activity_height = u16::from(self.busy && !self.is_masked_input());
        let fixed_bottom = input_height + activity_height + 1;
        let transcript_height = self.live_tail_height(
            usize::from(area.width.max(1)),
            usize::from(area.height.saturating_sub(fixed_bottom)),
        );
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(transcript_height),
                Constraint::Length(activity_height),
                Constraint::Length(input_height),
                Constraint::Length(1),
            ])
            .split(area);

        if transcript_height > 0 {
            frame.render_widget(self.render_transcript(layout[0]), layout[0]);
        }
        if activity_height > 0 {
            frame.render_widget(self.render_activity_indicator(), layout[1]);
        }
        frame.render_widget(self.render_input(), layout[2]);
        if let Some(popup_area) = self.command_popup_area(area, layout[2]) {
            frame.render_widget(Clear, popup_area);
            frame.render_widget(self.render_command_popup(), popup_area);
        }
        frame.set_cursor_position(self.cursor_position(layout[2]));
        frame.render_widget(self.render_status_bar(), layout[3]);
    }

    fn render_status_bar(&self) -> Paragraph<'static> {
        let active_thread = self.session.active_thread().thread_ref.stable_id();
        let status = match self.busy_phase {
            BusyPhase::Idle if self.queued_prompts.is_empty() => "idle".to_string(),
            BusyPhase::Idle => format!("idle · {} queued", self.queued_prompts.len()),
            BusyPhase::Thinking => "thinking".to_string(),
            BusyPhase::Rendering => "rendering".to_string(),
        };

        let line = Line::from(vec![
            Span::styled(
                " paddles ",
                self.palette.header_title.add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                format!("model {}", self.model_label),
                self.palette.header_meta,
            ),
            Span::raw(" "),
            Span::styled(
                format!("{status} · {active_thread}"),
                self.palette.header_status,
            ),
        ]);

        Paragraph::new(Text::from(vec![line]))
    }

    fn render_activity_indicator(&self) -> Paragraph<'static> {
        let spinner = SPINNER_FRAMES[self.spinner_index];
        let label = match self.last_event.as_ref() {
            Some((
                TurnEvent::GathererSearchProgress { .. } | TurnEvent::GathererSummary { .. },
                _,
            )) => "hunting",
            _ => match self.busy_phase {
                BusyPhase::Thinking => "thinking",
                BusyPhase::Rendering => "rendering",
                _ => "working",
            },
        };
        let elapsed = self
            .active_turn_timing
            .as_ref()
            .map(|t| Instant::now().duration_since(t.started_at).as_secs())
            .unwrap_or(0);
        let timer = if elapsed > 0 {
            format!(" {elapsed}s")
        } else {
            String::new()
        };
        let line = Line::from(vec![
            Span::styled(format!(" {spinner} {label}"), self.palette.event_body),
            Span::styled(timer, self.palette.event_body),
        ]);
        Paragraph::new(Text::from(vec![line]))
    }

    fn render_transcript(&self, area: Rect) -> Paragraph<'static> {
        let inner_width = usize::from(area.width.max(1));
        let inner_height = usize::from(area.height.max(1));
        let visible_rows = self.visible_live_rows(inner_width, inner_height);
        let mut lines = Vec::new();

        for (index, row) in visible_rows.iter().enumerate() {
            lines.extend(render_row_lines(row, &self.palette));
            if index + 1 < visible_rows.len() {
                lines.push(Line::default());
            }
        }

        Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false })
    }

    fn render_input(&self) -> Paragraph<'static> {
        let lines = self.input_render_lines();
        Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title(" Prompt ")
                    .borders(Borders::ALL)
                    .border_style(self.palette.border)
                    .style(Style::default().bg(self.palette.input_bg)),
            )
            .wrap(Wrap { trim: false })
    }

    fn input_render_lines(&self) -> Vec<Line<'static>> {
        let is_masked = self.is_masked_input();
        if !is_masked
            && !self.input.is_empty()
            && !self.input.contains('\n')
            && let Some(suggestion) = self.selected_slash_suggestion()
            && suggestion.insert_text.starts_with(&self.input)
            && suggestion.insert_text.len() > self.input.len()
        {
            let suffix = &suggestion.insert_text[self.input.len()..];
            return vec![Line::from(vec![
                Span::styled(self.input.clone(), self.palette.input_text),
                Span::styled(suffix.to_string(), self.palette.input_hint),
            ])];
        }

        let (text, style) = self.input_display_line(is_masked);
        text.split('\n')
            .map(|line| Line::from(Span::styled(line.to_string(), style)))
            .collect()
    }

    fn render_command_popup(&self) -> Paragraph<'static> {
        let suggestions = self.slash_command_suggestions();
        let selected = self
            .slash_suggestion_index
            .min(suggestions.len().saturating_sub(1));
        let lines = suggestions
            .iter()
            .enumerate()
            .map(|(index, command)| {
                let is_selected = index == selected;
                let usage_style = if is_selected {
                    self.palette.header_title.add_modifier(Modifier::BOLD)
                } else {
                    self.palette.input_text
                };
                let desc_style = if is_selected {
                    self.palette.header_meta
                } else {
                    self.palette.input_hint
                };
                Line::from(vec![
                    Span::styled(command.usage.clone(), usage_style),
                    Span::raw("  "),
                    Span::styled(command.description.clone(), desc_style),
                ])
            })
            .collect::<Vec<_>>();

        Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title(" Commands ")
                    .borders(Borders::ALL)
                    .border_style(self.palette.border)
                    .style(Style::default().bg(self.palette.input_bg)),
            )
            .wrap(Wrap { trim: false })
    }

    fn command_popup_area(&self, frame_area: Rect, input_area: Rect) -> Option<Rect> {
        let suggestions = self.slash_command_suggestions();
        if suggestions.is_empty() {
            return None;
        }
        let content_width = suggestions
            .iter()
            .map(|command| command.usage.len() + command.description.len() + 2)
            .max()
            .unwrap_or(24)
            .min(usize::from(frame_area.width.saturating_sub(4)));
        let width = ((content_width as u16).max(28) + 2).min(frame_area.width.saturating_sub(2));
        let height = (suggestions.len() as u16 + 2).min(frame_area.height.saturating_sub(1));
        let x = input_area
            .x
            .saturating_add(2)
            .min(frame_area.right().saturating_sub(width));
        let space_above = input_area.y.saturating_sub(frame_area.y);
        let y = if space_above >= height {
            input_area.y - height
        } else {
            frame_area.y
        };
        Some(Rect::new(x, y, width, height))
    }

    fn is_masked_input(&self) -> bool {
        matches!(self.input_mode, InputMode::MaskedKey { .. })
    }

    fn input_display_line(&self, is_masked: bool) -> (String, Style) {
        // Active input — show the typed text.
        if !self.input.is_empty() {
            let text = if is_masked {
                "\u{2022}".repeat(self.input.chars().count())
            } else {
                self.input.clone()
            };
            return (text, self.palette.input_text);
        }

        // Empty input — show a contextual placeholder.
        let placeholder = if is_masked {
            "Paste or type your API key · Enter to save · Esc to cancel".to_string()
        } else {
            self.input_placeholder()
        };
        (placeholder, self.palette.input_hint)
    }

    fn input_placeholder(&self) -> String {
        let queue_hint = if self.queued_prompts.is_empty() {
            None
        } else {
            Some(format!("{} queued", self.queued_prompts.len()))
        };
        let turn_hint = if self.busy {
            Some("Turn in progress")
        } else {
            None
        };
        match (turn_hint, queue_hint) {
            (Some(turn), Some(queue)) => format!("{turn} · {queue}"),
            (Some(turn), None) => turn.to_string(),
            (None, Some(queue)) => format!("Type a prompt... · {queue}"),
            (None, None) => "Type a prompt...".to_string(),
        }
    }

    fn cursor_position(&self, area: Rect) -> (u16, u16) {
        let inner_width = area.width.saturating_sub(2).max(1) as usize;
        // Take only the text up to the cursor position.
        let text_to_cursor: String = self.input.chars().take(self.cursor_pos).collect();

        let mut row = 0u16;
        let mut col = 0usize;
        for (i, line) in text_to_cursor.split('\n').enumerate() {
            if i > 0 {
                row += 1;
            }
            let char_count = line.chars().count();
            row += (char_count / inner_width) as u16;
            col = char_count % inner_width;
        }

        let x = area.x.saturating_add(1 + col as u16);
        let y = area.y.saturating_add(1 + row);
        (x.min(area.right().saturating_sub(1)), y)
    }

    fn input_area_height(&self, width: u16) -> u16 {
        let inner_width = width.saturating_sub(2).max(1) as usize;
        let content_lines = if self.input.is_empty() {
            1
        } else {
            self.input
                .split('\n')
                .map(|line| wrapped_line_count(line, inner_width).max(1))
                .sum()
        };
        (content_lines as u16) + 2 // content + top/bottom border
    }

    fn live_tail_height(&self, width: usize, max_height: usize) -> u16 {
        if max_height == 0 {
            return 0;
        }

        let visible_rows = self.visible_live_rows(width, max_height);
        rendered_rows_height(&visible_rows, width).min(max_height) as u16
    }

    fn visible_live_rows(&self, width: usize, height: usize) -> Vec<TranscriptRow> {
        let mut visible = Vec::new();
        let mut used = 0;
        let live_rows = &self.rows[self.flushed_row_count..];

        for row in live_rows.iter().rev() {
            let row_height = row.estimated_height(width);
            let rendered_height = row_height + usize::from(!visible.is_empty());
            if !visible.is_empty() && used + rendered_height > height {
                break;
            }
            used += rendered_height;
            visible.push(row.clone());
        }

        visible.reverse();
        visible
    }

    fn take_scrollback_rows(&mut self) -> Vec<TranscriptRow> {
        let flush_cutoff = self.flush_cutoff_index();
        let rows = self.rows[self.flushed_row_count..flush_cutoff].to_vec();
        self.flushed_row_count = flush_cutoff;
        rows
    }

    fn flush_cutoff_index(&self) -> usize {
        if self.pending_reveal.is_some() {
            self.rows.len().saturating_sub(1)
        } else {
            self.rows.len()
        }
    }

    fn annotate_turn_total(&mut self, row: TranscriptRow, occurred_at: Instant) -> TranscriptRow {
        match self.active_turn_timing.take() {
            Some(timing) => row.timed(timing.finish(occurred_at)),
            None => row,
        }
    }
}

fn inline_viewport_height() -> u16 {
    terminal_size()
        .map(|(_, height)| inline_viewport_height_for_terminal(height))
        .unwrap_or(INLINE_VIEWPORT_MAX_HEIGHT)
}

fn inline_viewport_height_for_terminal(terminal_height: u16) -> u16 {
    terminal_height
        .saturating_sub(2)
        .clamp(INLINE_VIEWPORT_MIN_HEIGHT, INLINE_VIEWPORT_MAX_HEIGHT)
}

fn step_timing_cache_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".cache")
        .join("paddles")
        .join("step_timing.json")
}

fn flush_scrollback_rows<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut InteractiveApp,
) -> io::Result<()> {
    terminal.autoresize()?;
    let width = usize::from(terminal.size()?.width.max(1));
    let palette = app.palette;

    for row in app.take_scrollback_rows() {
        let height = row.estimated_height(width) as u16;
        terminal.insert_before(height, |buffer| {
            let mut lines = render_row_lines(&row, &palette);
            lines.push(Line::default());
            Paragraph::new(Text::from(lines))
                .wrap(Wrap { trim: false })
                .render(buffer.area, buffer);
        })?;
    }

    Ok(())
}

/// Soft-wrap prose at word boundaries to fit within `max_width`.
/// Preserves existing newlines and does not break inside code blocks.
fn soft_wrap_prose(text: &str, max_width: usize) -> String {
    let max_width = max_width.max(20);
    let mut result = String::with_capacity(text.len());
    let mut in_code_block = false;

    for line in text.split('\n') {
        if !result.is_empty() {
            result.push('\n');
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
        }
        // Don't rewrap code blocks or short lines.
        if in_code_block || line.len() <= max_width {
            result.push_str(line);
            continue;
        }
        // Word-wrap long prose lines.
        let mut col = 0;
        for word in line.split_whitespace() {
            if col > 0 && col + 1 + word.len() > max_width {
                result.push('\n');
                col = 0;
            }
            if col > 0 {
                result.push(' ');
                col += 1;
            }
            result.push_str(word);
            col += word.len();
        }
    }

    result
}

fn trim_for_display(input: &str, limit: usize) -> String {
    if input.chars().count() <= limit {
        return input.to_string();
    }

    let kept = input.chars().take(limit).collect::<String>();
    format!("{}...", kept.trim_end())
}

fn inline_multiline_text(input: &str) -> String {
    let mut iter = input.lines();
    let Some(first_line) = iter.next() else {
        return String::new();
    };
    iter.fold(first_line.to_string(), |mut acc, line| {
        acc.push_str(MULTILINE_INLINE_SEPARATOR);
        acc.push_str(line);
        acc
    })
}

fn rendered_rows_height(rows: &[TranscriptRow], width: usize) -> usize {
    rows.iter()
        .enumerate()
        .map(|(index, row)| row.estimated_height(width) + usize::from(index > 0))
        .sum()
}

fn render_row_lines(row: &TranscriptRow, palette: &Palette) -> Vec<Line<'static>> {
    let (header_style, body_style) = match row.kind {
        TranscriptRowKind::User => (palette.user_header, palette.user_body),
        TranscriptRowKind::Assistant => (palette.assistant_header, palette.assistant_body),
        TranscriptRowKind::Event => (palette.event_header, palette.event_body),
        TranscriptRowKind::CommandNotice => {
            (palette.command_notice_header, palette.command_notice_body)
        }
        TranscriptRowKind::InFlightEvent => (palette.in_flight_header, palette.in_flight_body),
        TranscriptRowKind::Error => (palette.error_header, palette.error_body),
    };

    let mut header_spans = vec![Span::styled(
        row.header.clone(),
        header_style.add_modifier(Modifier::BOLD),
    )];
    if let Some(timing) = row.timing {
        header_spans.push(Span::styled(
            format!(" · {}", timing.elapsed_label()),
            body_style,
        ));
        if let Some(delta_label) = timing.delta_label() {
            let delta_style = match timing.pace {
                Pace::Fast => palette.pace_fast,
                Pace::Normal => body_style,
                Pace::Slow => palette.pace_slow,
            };
            header_spans.push(Span::styled(delta_label, delta_style));
        }
    }

    let mut lines = vec![Line::from(header_spans)];

    if row.content.is_empty() {
        return lines;
    }

    if let Some(tool_name) = row.header.strip_prefix("• Completed ").map(str::trim)
        && is_mutation_tool_name(tool_name)
        && let Some(diff_content) = mutation_tool_payload(tool_name, &row.content)
    {
        let mutation_lines = render_mutation_diff_lines(&diff_content, palette);
        lines.extend(mutation_lines);
        return lines;
    }

    if row.kind == TranscriptRowKind::Assistant
        && let Some(render) = &row.render
    {
        lines.extend(render_assistant_document_lines(
            render,
            body_style,
            palette.code,
            palette.citation,
        ));
        return lines;
    }

    let mut in_code_block = false;
    for (index, line) in row.content.lines().enumerate() {
        let prefix = if index == 0 { "  └ " } else { "    " };
        let rendered = match row.kind {
            TranscriptRowKind::Assistant => render_assistant_line(
                prefix,
                line,
                body_style,
                palette.code,
                palette.citation,
                &mut in_code_block,
            ),
            _ => Line::from(vec![
                Span::styled(prefix.to_string(), body_style),
                Span::styled(line.to_string(), body_style),
            ]),
        };
        lines.push(rendered);
    }

    lines
}

fn assistant_render_line_count(render: &RenderDocument, width: usize) -> usize {
    assistant_render_line_specs(render)
        .iter()
        .map(|spec| wrapped_line_count(&spec.text, width))
        .sum()
}

#[derive(Clone, Copy)]
enum AssistantRenderLineKind {
    Heading,
    Paragraph,
    Code,
    Citations,
}

struct AssistantRenderLine {
    kind: AssistantRenderLineKind,
    text: String,
}

fn assistant_render_line_specs(render: &RenderDocument) -> Vec<AssistantRenderLine> {
    let mut lines = Vec::new();
    for (index, block) in render.blocks.iter().enumerate() {
        if index > 0 {
            lines.push(AssistantRenderLine {
                kind: AssistantRenderLineKind::Paragraph,
                text: String::new(),
            });
        }

        match block {
            RenderBlock::Heading { text } => lines.push(AssistantRenderLine {
                kind: AssistantRenderLineKind::Heading,
                text: text.clone(),
            }),
            RenderBlock::Paragraph { text } => {
                for line in text.lines() {
                    lines.push(AssistantRenderLine {
                        kind: AssistantRenderLineKind::Paragraph,
                        text: line.to_string(),
                    });
                }
            }
            RenderBlock::BulletList { items } => {
                for item in items {
                    lines.push(AssistantRenderLine {
                        kind: AssistantRenderLineKind::Paragraph,
                        text: format!("- {item}"),
                    });
                }
            }
            RenderBlock::CodeBlock { language, code } => {
                lines.push(AssistantRenderLine {
                    kind: AssistantRenderLineKind::Code,
                    text: match language.as_deref().filter(|value| !value.is_empty()) {
                        Some(language) => format!("```{language}"),
                        None => "```".to_string(),
                    },
                });
                for line in code.lines() {
                    lines.push(AssistantRenderLine {
                        kind: AssistantRenderLineKind::Code,
                        text: line.to_string(),
                    });
                }
                lines.push(AssistantRenderLine {
                    kind: AssistantRenderLineKind::Code,
                    text: "```".to_string(),
                });
            }
            RenderBlock::Citations { sources } => lines.push(AssistantRenderLine {
                kind: AssistantRenderLineKind::Citations,
                text: format!("Sources: {}", sources.join(", ")),
            }),
        }
    }

    if lines.is_empty() {
        lines.push(AssistantRenderLine {
            kind: AssistantRenderLineKind::Paragraph,
            text: String::new(),
        });
    }

    lines
}

fn render_assistant_document_lines(
    render: &RenderDocument,
    base_style: Style,
    code_style: Style,
    citation_style: Style,
) -> Vec<Line<'static>> {
    assistant_render_line_specs(render)
        .into_iter()
        .enumerate()
        .map(|(index, spec)| {
            let prefix = if index == 0 { "  └ " } else { "    " };
            let style = match spec.kind {
                AssistantRenderLineKind::Heading => base_style.add_modifier(Modifier::BOLD),
                AssistantRenderLineKind::Paragraph => base_style,
                AssistantRenderLineKind::Code => code_style,
                AssistantRenderLineKind::Citations => citation_style,
            };
            Line::from(vec![
                Span::styled(prefix.to_string(), base_style),
                Span::styled(spec.text, style),
            ])
        })
        .collect()
}

fn render_assistant_line(
    prefix: &str,
    line: &str,
    base_style: Style,
    code_style: Style,
    citation_style: Style,
    in_code_block: &mut bool,
) -> Line<'static> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
        *in_code_block = !*in_code_block;
        return Line::from(vec![
            Span::styled(prefix.to_string(), base_style),
            Span::styled(line.to_string(), code_style),
        ]);
    }

    if *in_code_block {
        return Line::from(vec![
            Span::styled(prefix.to_string(), base_style),
            Span::styled(line.to_string(), code_style),
        ]);
    }

    if trimmed.starts_with("Sources:") {
        return Line::from(vec![
            Span::styled(prefix.to_string(), base_style),
            Span::styled(line.to_string(), citation_style),
        ]);
    }

    let mut spans = vec![Span::styled(prefix.to_string(), base_style)];
    let mut code_segment = false;
    for segment in line.split('`') {
        let style = if code_segment { code_style } else { base_style };
        spans.push(Span::styled(segment.to_string(), style));
        code_segment = !code_segment;
    }

    Line::from(spans)
}

fn format_turn_event_row(event: TurnEvent, verbose: u8) -> TranscriptRow {
    let prefer_custom_render = matches!(&event, TurnEvent::InterpretationContext { .. })
        || (verbose >= 1 && matches!(&event, TurnEvent::PlannerStepProgress { .. }))
        || (verbose >= 2 && matches!(&event, TurnEvent::PlannerSummary { .. }))
        || matches!(&event, TurnEvent::ToolFinished { .. });
    if !prefer_custom_render {
        return transcript_row_from_runtime_event(project_runtime_event(&event));
    }

    match event {
        TurnEvent::IntentClassified { intent } => {
            TranscriptRow::new(TranscriptRowKind::Event, "• Classified", intent.label())
        }
        TurnEvent::InterpretationContext { context } => {
            let content = if verbose >= 2 {
                context.render()
            } else if verbose == 1 {
                let mut sections = vec![collapse_event_details(
                    &context.summary,
                    EVENT_DETAIL_LINE_LIMIT,
                )];
                if !context.documents.is_empty() {
                    sections.push(format!("Sources: {}", context.sources().join(", ")));
                    for doc in &context.documents {
                        sections.push(format!(
                            "--- {} [{:?}] ---\n{}",
                            doc.source,
                            doc.category,
                            collapse_event_details(&doc.excerpt, 2)
                        ));
                    }
                }
                if !context.tool_hints.is_empty() {
                    sections.push("--- Tool Hints ---".to_string());
                    sections.extend(context.tool_hints.iter().map(|hint| {
                        format!(
                            "- {} ({}) — {}",
                            hint.action.summary(),
                            hint.source,
                            hint.note
                        )
                    }));
                }
                sections.join("\n\n")
            } else {
                let mut content = collapse_event_details(&context.summary, EVENT_DETAIL_LINE_LIMIT);
                let sources = context.sources();
                if !sources.is_empty() {
                    if !content.is_empty() {
                        content.push('\n');
                    }
                    content.push_str("Sources: ");
                    content.push_str(&sources.join(", "));
                }
                content
            };

            TranscriptRow::new(
                TranscriptRowKind::Event,
                format!(
                    "• Assembled interpretation context [{} docs, {} hints, {} procedures]",
                    context.documents.len(),
                    context.tool_hints.len(),
                    context.decision_framework.procedures.len()
                ),
                content,
            )
        }
        TurnEvent::GuidanceGraphExpanded {
            depth,
            document_count,
            sources,
        } => TranscriptRow::new(
            TranscriptRowKind::Event,
            "• Expanded guidance graph",
            format!(
                "depth {depth}: found {document_count} docs ({})",
                sources.join(", ")
            ),
        ),
        TurnEvent::RouteSelected { summary } => TranscriptRow::new(
            TranscriptRowKind::Event,
            "• Routed",
            collapse_event_details(&summary, EVENT_DETAIL_LINE_LIMIT),
        ),
        TurnEvent::PlannerCapability {
            provider,
            capability,
        } => TranscriptRow::new(
            TranscriptRowKind::Event,
            "• Checked planner capability",
            format!("{provider}: {capability}"),
        ),
        TurnEvent::GathererCapability {
            provider,
            capability,
        } => TranscriptRow::new(
            TranscriptRowKind::Event,
            "• Checked gatherer capability",
            format!("{provider}: {capability}"),
        ),
        TurnEvent::PlannerActionSelected {
            sequence,
            action,
            rationale,
        } => {
            let title = format!(
                "• Planner step {sequence}: {}",
                collapse_event_details(&action, 1)
            );
            let content = format!("Rationale: {}", collapse_event_details(&rationale, 2));
            TranscriptRow::new(TranscriptRowKind::Event, title, content)
        }
        TurnEvent::ThreadCandidateCaptured {
            candidate_id,
            active_thread,
            prompt,
        } => TranscriptRow::new(
            TranscriptRowKind::Event,
            "• Captured steering prompt",
            format!(
                "{} on {}\n{}",
                candidate_id,
                active_thread,
                collapse_event_details(&prompt, EVENT_DETAIL_LINE_LIMIT)
            ),
        ),
        TurnEvent::ThreadDecisionApplied {
            candidate_id,
            decision,
            target_thread,
            rationale,
        } => TranscriptRow::new(
            TranscriptRowKind::Event,
            "• Applied thread decision",
            format!(
                "{}: {} -> {}\nRationale: {}",
                candidate_id,
                decision,
                target_thread,
                collapse_event_details(&rationale, EVENT_DETAIL_LINE_LIMIT)
            ),
        ),
        TurnEvent::ThreadMerged {
            source_thread,
            target_thread,
            mode,
            summary,
        } => TranscriptRow::new(
            TranscriptRowKind::Event,
            "• Merged thread",
            format!(
                "{} -> {} via {}\n{}",
                source_thread,
                target_thread,
                mode,
                collapse_event_details(
                    summary.as_deref().unwrap_or("No merge summary recorded."),
                    EVENT_DETAIL_LINE_LIMIT
                )
            ),
        ),
        TurnEvent::PlannerStepProgress {
            step_number,
            step_limit,
            action,
            query,
            evidence_count,
        } => {
            let q = query
                .map(|q| format!(" — {}", collapse_event_details(&q, 1)))
                .unwrap_or_default();
            let title = format!("• Step {step_number}/{step_limit}: {action}{q}");
            let content = if verbose >= 1 {
                format!("{evidence_count} evidence items")
            } else {
                String::new()
            };
            TranscriptRow::new(TranscriptRowKind::Event, title, content)
        }
        TurnEvent::GathererSearchProgress {
            phase,
            elapsed_seconds,
            eta_seconds,
            strategy,
            detail,
        } => {
            let elapsed = format_duration_compact(Duration::from_secs(elapsed_seconds));
            let eta = eta_seconds
                .map(|eta| format_duration_compact(Duration::from_secs(eta)))
                .unwrap_or_else(|| "unknown".to_string());
            let strategy = strategy
                .as_deref()
                .map(|value| format!(" strategy={value}"))
                .unwrap_or_default();
            let content = detail
                .as_deref()
                .map(std::string::ToString::to_string)
                .unwrap_or_default();
            TranscriptRow::new(
                TranscriptRowKind::Event,
                format!("• Hunting ({phase}) — {elapsed} (eta {eta}){strategy}"),
                content,
            )
        }
        TurnEvent::GathererSummary {
            provider,
            summary,
            sources,
        } => {
            let mut content = collapse_event_details(&summary, EVENT_DETAIL_LINE_LIMIT);
            if !sources.is_empty() {
                if !content.is_empty() {
                    content.push('\n');
                }
                content.push_str("Sources: ");
                content.push_str(&sources.join(", "));
            }
            TranscriptRow::new(
                TranscriptRowKind::Event,
                format!("• Gathered context with {provider}"),
                content,
            )
        }
        TurnEvent::HarnessState { snapshot } => {
            TranscriptRow::new(
                TranscriptRowKind::Event,
                snapshot.governor_header(),
                snapshot.governor_summary(true),
            )
        }
        TurnEvent::PlannerSummary {
            strategy,
            mode,
            turns,
            steps,
            stop_reason,
            active_branch_id,
            branch_count,
            frontier_count,
            node_count,
            edge_count,
            retained_artifact_count,
        } => {
            let opt = |v: Option<usize>| {
                v.map(|n| n.to_string())
                    .unwrap_or_else(|| "n/a".to_string())
            };
            let mut content = format!(
                "strategy={strategy}, mode={mode}, turns={turns}, steps={steps}, stop={}",
                stop_reason.as_deref().unwrap_or("none"),
            );
            if verbose >= 2 {
                content.push_str(&format!(
                    "\nGraph: nodes={}, edges={}, branches={}, frontier={}, active={}, retained={}",
                    opt(node_count),
                    opt(edge_count),
                    opt(branch_count),
                    opt(frontier_count),
                    active_branch_id.as_deref().unwrap_or("none"),
                    opt(retained_artifact_count),
                ));
            }
            TranscriptRow::new(
                TranscriptRowKind::Event,
                "• Reviewed planner trace",
                content,
            )
        }
        TurnEvent::ContextAssembly {
            label,
            hits,
            retained_artifacts,
            pruned_artifacts,
        } => TranscriptRow::new(
            TranscriptRowKind::Event,
            format!("• Assembled workspace context ({label})"),
            format!("{hits} hit(s), retained {retained_artifacts}, pruned {pruned_artifacts}"),
        ),
        TurnEvent::RefinementApplied {
            reason,
            before_summary,
            after_summary,
        } => TranscriptRow::new(
            TranscriptRowKind::Event,
            "• Applied interpretation refinement",
            format!(
                "{reason}\nbefore: {}\nafter: {}",
                collapse_event_details(&before_summary, 2),
                collapse_event_details(&after_summary, 2)
            ),
        ),
        TurnEvent::ToolCalled {
            tool_name,
            invocation,
            ..
        } => TranscriptRow::new(
            TranscriptRowKind::Event,
            if tool_name == "shell" {
                "• Ran shell".to_string()
            } else {
                format!("• Ran {tool_name}")
            },
            collapse_event_details(&invocation, EVENT_DETAIL_LINE_LIMIT),
        ),
        TurnEvent::ToolFinished {
            tool_name, summary, ..
        } => {
            let content = mutation_tool_payload(&tool_name, &summary)
                .unwrap_or_else(|| collapse_event_details(&summary, EVENT_DETAIL_LINE_LIMIT));
            TranscriptRow::new(
                TranscriptRowKind::Event,
                format!("• Completed {tool_name}"),
                content,
            )
        }
        TurnEvent::Fallback { stage, reason } => TranscriptRow::new(
            TranscriptRowKind::Event,
            "• Fell back",
            format!("{stage}: {reason}"),
        ),
        TurnEvent::ContextStrain { strain } => {
            let factors: Vec<_> = strain.factors.iter().map(|f| f.label()).collect();
            TranscriptRow::new(
                TranscriptRowKind::Event,
                format!("• Context strain: {}", strain.level.label()),
                format!(
                    "{} truncation(s), factors: [{}]",
                    strain.truncation_count,
                    factors.join(", ")
                ),
            )
        }
        TurnEvent::SynthesisReady {
            grounded,
            citations,
            insufficient_evidence,
        } => {
            if insufficient_evidence {
                TranscriptRow::new(
                    TranscriptRowKind::Event,
                    "• Reported insufficient evidence",
                    "No cited repository sources were available.",
                )
            } else if grounded {
                TranscriptRow::new(
                    TranscriptRowKind::Event,
                    "• Synthesized grounded answer",
                    if citations.is_empty() {
                        "Sources: none".to_string()
                    } else {
                        format!("Sources: {}", citations.join(", "))
                    },
                )
            } else {
                TranscriptRow::new(
                    TranscriptRowKind::Event,
                    "• Synthesized direct answer",
                    "No repository citations required for this turn.",
                )
            }
        }
    }
}

fn parse_hunting_telemetry_sample(phase: &str, detail: &str) -> Option<HuntingTelemetrySample> {
    let mut segments = detail.split(" · ");
    let first = segments.next()?.trim();
    let headline = parse_progress_headline(first);
    let mut metrics = Vec::new();

    for segment in segments {
        metrics.extend(parse_progress_metrics(segment.trim()));
    }

    Some(HuntingTelemetrySample {
        phase: phase.to_string(),
        headline,
        metrics,
    })
}

fn parse_progress_headline(segment: &str) -> Option<ProgressHeadline> {
    if let Some(rest) = segment.strip_prefix("indexing ")
        && let Some(rest) = rest.strip_suffix(" files")
    {
        let (current, total) = parse_fraction(rest)?;
        return Some(ProgressHeadline {
            label: "files".to_string(),
            current,
            total: Some(total),
        });
    }
    if let Some(rest) = segment.strip_prefix("embedding ")
        && let Some(rest) = rest.strip_suffix(" chunks")
    {
        let (current, total) = parse_fraction(rest)?;
        return Some(ProgressHeadline {
            label: "chunks".to_string(),
            current,
            total: Some(total),
        });
    }
    if let Some(rest) = segment.strip_prefix("retrieving turn ") {
        let (current, total) = parse_fraction(rest)?;
        return Some(ProgressHeadline {
            label: "turns".to_string(),
            current,
            total: Some(total),
        });
    }
    if let Some(rest) = segment.strip_prefix("ranking ")
        && let Some(rest) = rest.strip_suffix(" hits")
    {
        let (current, total) = parse_fraction(rest)?;
        return Some(ProgressHeadline {
            label: "hits".to_string(),
            current,
            total: Some(total),
        });
    }
    None
}

fn parse_progress_metrics(segment: &str) -> Vec<ProgressMetric> {
    if let Some(rest) = segment.strip_prefix("bm25 cache ") {
        let mut metrics = Vec::new();
        let mut parts = rest.split_whitespace();
        if let Some(value) = parts.next().and_then(|value| value.parse::<u64>().ok()) {
            metrics.push(ProgressMetric {
                label: "bm25 cache".to_string(),
                value,
            });
        }
        if let Some("build") = parts.next()
            && let Some(value) = parts.next().and_then(|value| value.parse::<u64>().ok())
        {
            metrics.push(ProgressMetric {
                label: "build".to_string(),
                value,
            });
        }
        return metrics;
    }

    if let Some((label, value)) = segment.rsplit_once(' ')
        && let Ok(value) = value.parse::<u64>()
    {
        return vec![ProgressMetric {
            label: label.to_string(),
            value,
        }];
    }

    Vec::new()
}

fn parse_fraction(value: &str) -> Option<(u64, u64)> {
    let (current, total) = value.split_once('/')?;
    Some((current.parse().ok()?, total.parse().ok()?))
}

fn format_hunting_history_delta(
    previous: &HuntingTelemetrySample,
    current: &HuntingTelemetrySample,
) -> Option<String> {
    if previous.phase != current.phase {
        return Some(format_hunting_history_snapshot(current));
    }

    let mut parts = Vec::new();
    if let Some(current_headline) = current.headline.as_ref() {
        let delta = previous
            .headline
            .as_ref()
            .filter(|headline| headline.label == current_headline.label)
            .map(|headline| current_headline.current.saturating_sub(headline.current))
            .unwrap_or(0);
        let total = current_headline
            .total
            .map(|total| format!("/{}", total))
            .unwrap_or_default();
        parts.push(format!(
            "{} {}{} (+{})",
            current_headline.label, current_headline.current, total, delta
        ));
    }

    for metric in &current.metrics {
        let previous_value = previous
            .metrics
            .iter()
            .find(|candidate| candidate.label == metric.label)
            .map(|candidate| candidate.value);
        if let Some(previous_value) = previous_value {
            let delta = metric.value as i64 - previous_value as i64;
            if delta != 0 {
                parts.push(format!("{} {} ({:+})", metric.label, metric.value, delta));
            }
        } else {
            parts.push(format!("{} {}", metric.label, metric.value));
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}

fn format_hunting_history_snapshot(sample: &HuntingTelemetrySample) -> String {
    let mut parts = Vec::new();
    if let Some(headline) = sample.headline.as_ref() {
        let total = headline
            .total
            .map(|total| format!("/{}", total))
            .unwrap_or_default();
        parts.push(format!("{} {}{}", headline.label, headline.current, total));
    }
    parts.extend(
        sample
            .metrics
            .iter()
            .map(|metric| format!("{} {}", metric.label, metric.value)),
    );
    parts.join(" · ")
}

fn is_mutation_tool_name(tool_name: &str) -> bool {
    matches!(tool_name, "diff" | "apply_patch")
}

fn mutation_tool_payload(tool_name: &str, summary: &str) -> Option<String> {
    if !is_mutation_tool_name(tool_name) {
        return None;
    }

    match tool_name {
        "diff" if summary == "No diff output." => Some(summary.to_string()),
        "diff" => summary
            .strip_prefix("Diff output:\n")
            .map(|diff| diff.to_string())
            .or_else(|| {
                summary
                    .strip_prefix("Diff output:\r\n")
                    .map(|diff| diff.to_string())
            }),
        "apply_patch" => summary.strip_prefix("Applied patch:\n").map(|patch_block| {
            patch_block
                .split_once("\n\nExit status:")
                .map_or(patch_block, |split| split.0)
                .split_once("\nExit status:")
                .map_or_else(|| patch_block, |split| split.0)
                .trim()
                .to_string()
        }),
        _ => None,
    }
}

fn mutation_line_style(line: &str, palette: &Palette) -> Style {
    if line.starts_with("+++")
        || line.starts_with("---")
        || line.starts_with("diff ")
        || line.starts_with("index ")
    {
        return Style::default().fg(Color::Rgb(110, 118, 129));
    }

    let Some(first_char) = line.chars().next() else {
        return palette.code;
    };

    match first_char {
        '+' => Style::default().fg(Color::Rgb(63, 185, 80)),
        '-' => Style::default().fg(Color::Rgb(248, 81, 73)),
        '@' => Style::default().fg(Color::Rgb(88, 166, 255)),
        '\\' => Style::default().fg(Color::Rgb(128, 128, 128)),
        _ => palette.code,
    }
}

fn render_mutation_diff_lines(content: &str, palette: &Palette) -> Vec<Line<'static>> {
    content
        .lines()
        .map(|line| {
            let style = mutation_line_style(line, palette);
            Line::from(vec![Span::styled(format!("  {line}"), style)])
        })
        .collect()
}

fn collapse_event_details(input: &str, max_lines: usize) -> String {
    let lines = input.lines().collect::<Vec<_>>();
    if lines.len() <= max_lines {
        return input.trim().to_string();
    }

    let mut rendered = lines
        .iter()
        .take(max_lines)
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    rendered.push_str(&format!("\n… +{} lines", lines.len() - max_lines));
    rendered
}

fn wrapped_line_count(line: &str, width: usize) -> usize {
    let width = width.max(1);
    let chars = cmp::max(1, line.chars().count());
    chars.div_ceil(width)
}

fn format_duration_compact(duration: Duration) -> String {
    if duration < Duration::from_secs(1) {
        return format!("{}ms", duration.as_millis());
    }

    if duration < Duration::from_secs(60) {
        return format!("{:.1}s", duration.as_secs_f64());
    }

    if duration < Duration::from_secs(3600) {
        let total_seconds = duration.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        return format!("{minutes}m {seconds:02}s");
    }

    let total_minutes = duration.as_secs() / 60;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{hours}h {minutes:02}m")
}

const SPINNER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

#[derive(Clone, Copy)]
struct Palette {
    header_title: Style,
    header_meta: Style,
    header_status: Style,
    border: Style,
    user_header: Style,
    user_body: Style,
    assistant_header: Style,
    assistant_body: Style,
    event_header: Style,
    event_body: Style,
    command_notice_header: Style,
    command_notice_body: Style,
    in_flight_header: Style,
    in_flight_body: Style,
    error_header: Style,
    error_body: Style,
    input_text: Style,
    input_hint: Style,
    input_bg: Color,
    code: Style,
    citation: Style,
    pace_fast: Style,
    pace_slow: Style,
}

fn detect_palette() -> Palette {
    if terminal_uses_light_background() {
        let prompt_bg = Color::Rgb(234, 237, 243);
        Palette {
            header_title: Style::default().fg(Color::Rgb(24, 63, 115)),
            header_meta: Style::default().fg(Color::Rgb(78, 87, 103)),
            header_status: Style::default().fg(Color::Rgb(19, 120, 95)),
            border: Style::default().fg(Color::Rgb(132, 145, 165)),
            user_header: Style::default().fg(Color::Rgb(18, 74, 140)).bg(prompt_bg),
            user_body: Style::default().fg(Color::Rgb(35, 43, 54)).bg(prompt_bg),
            assistant_header: Style::default().fg(Color::Rgb(0, 120, 102)),
            assistant_body: Style::default().fg(Color::Rgb(24, 33, 45)),
            event_header: Style::default().fg(Color::Rgb(138, 87, 0)),
            event_body: Style::default().fg(Color::Rgb(72, 77, 84)),
            command_notice_header: Style::default().fg(Color::Rgb(0, 92, 184)),
            command_notice_body: Style::default().fg(Color::Rgb(24, 41, 68)),
            in_flight_header: Style::default().fg(Color::Rgb(168, 126, 40)),
            in_flight_body: Style::default().fg(Color::Rgb(120, 124, 130)),
            error_header: Style::default().fg(Color::Rgb(173, 38, 45)),
            error_body: Style::default().fg(Color::Rgb(99, 39, 44)),
            input_text: Style::default().fg(Color::Rgb(35, 43, 54)),
            input_hint: Style::default().fg(Color::Rgb(109, 117, 129)),
            input_bg: prompt_bg,
            code: Style::default().fg(Color::Rgb(87, 56, 130)),
            citation: Style::default().fg(Color::Rgb(94, 66, 0)),
            pace_fast: Style::default().fg(Color::Rgb(165, 171, 180)),
            pace_slow: Style::default().fg(Color::Rgb(191, 105, 25)),
        }
    } else {
        let prompt_bg = Color::Rgb(30, 33, 39);
        Palette {
            header_title: Style::default().fg(Color::Rgb(125, 194, 255)),
            header_meta: Style::default().fg(Color::Rgb(155, 169, 187)),
            header_status: Style::default().fg(Color::Rgb(116, 225, 175)),
            border: Style::default().fg(Color::Rgb(84, 95, 114)),
            user_header: Style::default().fg(Color::Rgb(115, 197, 255)).bg(prompt_bg),
            user_body: Style::default().fg(Color::Rgb(224, 229, 236)).bg(prompt_bg),
            assistant_header: Style::default().fg(Color::Rgb(111, 231, 183)),
            assistant_body: Style::default().fg(Color::Rgb(234, 240, 247)),
            event_header: Style::default().fg(Color::Rgb(255, 202, 92)),
            event_body: Style::default().fg(Color::Rgb(182, 191, 204)),
            command_notice_header: Style::default().fg(Color::Rgb(126, 204, 255)),
            command_notice_body: Style::default().fg(Color::Rgb(229, 241, 255)),
            in_flight_header: Style::default().fg(Color::Rgb(200, 170, 80)),
            in_flight_body: Style::default().fg(Color::Rgb(130, 140, 155)),
            error_header: Style::default().fg(Color::Rgb(255, 122, 132)),
            error_body: Style::default().fg(Color::Rgb(238, 183, 190)),
            input_text: Style::default().fg(Color::Rgb(236, 242, 250)),
            input_hint: Style::default().fg(Color::Rgb(145, 154, 168)),
            input_bg: prompt_bg,
            code: Style::default().fg(Color::Rgb(204, 171, 255)),
            citation: Style::default().fg(Color::Rgb(255, 216, 130)),
            pace_fast: Style::default().fg(Color::Rgb(90, 98, 112)),
            pace_slow: Style::default().fg(Color::Rgb(255, 180, 80)),
        }
    }
}

fn terminal_uses_light_background() -> bool {
    let Ok(value) = std::env::var("COLORFGBG") else {
        return false;
    };

    value
        .split(';')
        .next_back()
        .and_then(|bg| bg.parse::<u8>().ok())
        .is_some_and(|bg| bg >= 8)
}

#[cfg(test)]
mod tests {
    use super::{
        BusyPhase, InputMode, InteractiveApp, InteractiveFrontend, PendingReveal, QueuedPrompt,
        TranscriptRow, TranscriptRowKind, TranscriptTiming, TranscriptTimingKind, UiMessage,
        collapse_event_details, detect_palette, format_duration_compact, format_turn_event_row,
        inline_multiline_text, inline_viewport_height_for_terminal, render_row_lines,
        runtime_lane_summary, select_interactive_frontend,
    };
    use crate::application::{ConversationSession, RuntimeLaneConfig};
    use crate::domain::model::{
        ConversationTranscript, ConversationTranscriptEntry, ConversationTranscriptSpeaker,
        ConversationTranscriptUpdate, RenderBlock, RenderDocument, TaskTraceId, TraceRecordId,
        TurnEvent, TurnTraceId,
    };
    use crate::infrastructure::credentials::ProviderAvailability;
    use crate::infrastructure::providers::ModelProvider;
    use crate::infrastructure::step_timing::Pace;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::style::Modifier;
    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, prelude::Rect};
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    fn session() -> ConversationSession {
        ConversationSession::new(TaskTraceId::new("task-1").expect("task"))
    }

    fn transcript(entries: &[(ConversationTranscriptSpeaker, &str)]) -> ConversationTranscript {
        ConversationTranscript {
            task_id: TaskTraceId::new("task-1").expect("task"),
            entries: entries
                .iter()
                .enumerate()
                .map(|(index, (speaker, content))| ConversationTranscriptEntry {
                    record_id: TraceRecordId::new(format!("record-{}", index + 1))
                        .expect("record id"),
                    turn_id: TurnTraceId::new(format!("task-1.turn-{:04}", index + 1))
                        .expect("turn id"),
                    speaker: *speaker,
                    content: (*content).to_string(),
                    response_mode: None,
                    render: None,
                })
                .collect(),
        }
    }

    fn render_buffer(app: &InteractiveApp, width: u16, height: u16) -> Buffer {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| app.render(frame))
            .expect("frame")
            .buffer
            .clone()
    }

    fn buffer_line(buffer: &Buffer, y: u16) -> String {
        (0..buffer.area.width)
            .map(|x| buffer[(x, y)].symbol())
            .collect::<Vec<_>>()
            .join("")
    }

    fn buffer_text(buffer: &Buffer) -> String {
        (0..buffer.area.height)
            .map(|y| buffer_line(buffer, y))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn provider_availability(
        provider: ModelProvider,
        enabled: bool,
        detail: &str,
    ) -> ProviderAvailability {
        ProviderAvailability {
            provider,
            enabled,
            detail: detail.to_string(),
        }
    }

    #[test]
    fn tty_interactive_mode_uses_tui_but_prompt_keeps_plain_path() {
        assert_eq!(
            select_interactive_frontend(false, true, true),
            InteractiveFrontend::Tui
        );
        assert_eq!(
            select_interactive_frontend(true, true, true),
            InteractiveFrontend::PlainLines
        );
        assert_eq!(
            select_interactive_frontend(false, false, true),
            InteractiveFrontend::PlainLines
        );
    }

    #[test]
    fn assistant_reveal_finishes_with_full_text() {
        let mut reveal = PendingReveal::new(0, "hello world".to_string(), None);
        while !reveal.advance() {}

        assert_eq!(reveal.visible_text(), "hello world");
    }

    #[test]
    fn assistant_rows_render_heading_blocks_without_literal_markers() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        let transcript = ConversationTranscript {
            task_id: TaskTraceId::new("task-1").expect("task"),
            entries: vec![ConversationTranscriptEntry {
                record_id: TraceRecordId::new("record-1").expect("record"),
                turn_id: TurnTraceId::new("task-1.turn-0001").expect("turn"),
                speaker: ConversationTranscriptSpeaker::Assistant,
                content: "**Summary**\n\nBody".to_string(),
                response_mode: None,
                render: Some(RenderDocument {
                    blocks: vec![
                        RenderBlock::Heading {
                            text: "Summary".to_string(),
                        },
                        RenderBlock::Paragraph {
                            text: "Body".to_string(),
                        },
                    ],
                }),
            }],
        };

        app.load_transcript(&transcript);
        let buffer = render_buffer(&app, 80, 12);
        let rendered = buffer_text(&buffer);
        assert!(rendered.contains("Summary"));
        assert!(!rendered.contains("**Summary**"));
    }

    #[test]
    fn event_details_are_elided_for_long_outputs() {
        let collapsed = collapse_event_details("a\nb\nc\nd", 2);
        assert_eq!(collapsed, "a\nb\n… +2 lines");
    }

    #[test]
    fn assistant_rows_style_sources_and_inline_code() {
        let palette = detect_palette();
        let row = TranscriptRow::new(
            TranscriptRowKind::Assistant,
            "Paddles",
            "Use `src/main.rs`\nSources: src/main.rs",
        );
        let lines = render_row_lines(&row, &palette);

        assert_eq!(lines.len(), 3);
        assert!(
            lines[2]
                .spans
                .iter()
                .any(|span| span.content.contains("Sources:"))
        );
    }

    #[test]
    fn turn_events_render_as_codex_like_action_rows() {
        let row = format_turn_event_row(
            TurnEvent::ToolCalled {
                call_id: "tool-1".to_string(),
                tool_name: "shell".to_string(),
                invocation: "git status --short".to_string(),
            },
            0,
        );

        assert_eq!(row.kind, TranscriptRowKind::Event);
        assert_eq!(row.header, "• Ran shell");
        assert_eq!(row.content, "git status --short");
    }

    #[test]
    fn gatherer_progress_rows_use_hunting_language() {
        let row = format_turn_event_row(
            TurnEvent::GathererSearchProgress {
                phase: "Indexing".to_string(),
                elapsed_seconds: 110,
                eta_seconds: Some(0),
                strategy: Some("bm25".to_string()),
                detail: Some("indexing 75914/75934 files".to_string()),
            },
            0,
        );

        assert_eq!(row.kind, TranscriptRowKind::Event);
        assert_eq!(
            row.header,
            "• Hunting (Indexing) — 1m 50s (eta 0ms) strategy=bm25"
        );
        assert_eq!(row.content, "indexing 75914/75934 files");
    }

    #[test]
    fn harness_state_events_render_governor_and_timeout_summary() {
        let row = format_turn_event_row(
            TurnEvent::HarnessState {
                snapshot: crate::domain::model::HarnessSnapshot {
                    chamber: crate::domain::model::HarnessChamber::Gathering,
                    governor: crate::domain::model::GovernorState {
                        status: crate::domain::model::HarnessStatus::Active,
                        timeout: crate::domain::model::TimeoutState {
                            phase: crate::domain::model::TimeoutPhase::Slow,
                            elapsed_seconds: Some(9),
                            deadline_seconds: Some(30),
                        },
                        intervention: None,
                    },
                    detail: Some("indexing 4/10 files".to_string()),
                },
            },
            0,
        );

        assert_eq!(row.kind, TranscriptRowKind::Event);
        assert_eq!(row.header, "• Governor: gathering");
        assert!(row.content.contains("status=active"));
        assert!(row.content.contains("timeout=slow"));
        assert!(row.content.contains("indexing 4/10 files"));
    }

    #[test]
    fn alternating_hunting_and_gathering_updates_collapse_in_place() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            0,
        );
        let rows_before = app.rows.len();
        let started = Instant::now();

        let hunt_event = |count: usize, fresh: usize, segments: usize, occurred_at: Instant| {
            super::UiMessage::TurnEvent {
                event: TurnEvent::GathererSearchProgress {
                    phase: "Indexing".to_string(),
                    elapsed_seconds: 110,
                    eta_seconds: Some(0),
                    strategy: Some("bm25".to_string()),
                    detail: Some(format!(
                        "indexing {count}/75934 files · blobs 1365 · fresh {fresh} · skipped 36239 · segments {segments} · bm25 cache 0 build 0"
                    )),
                },
                occurred_at,
            }
        };
        let governor_event = |count: usize, fresh: usize, segments: usize, occurred_at: Instant| {
            super::UiMessage::TurnEvent {
                event: TurnEvent::HarnessState {
                    snapshot: crate::domain::model::HarnessSnapshot {
                        chamber: crate::domain::model::HarnessChamber::Gathering,
                        governor: crate::domain::model::GovernorState {
                            status: crate::domain::model::HarnessStatus::Intervening,
                            timeout: crate::domain::model::TimeoutState {
                                phase: crate::domain::model::TimeoutPhase::Stalled,
                                elapsed_seconds: Some(110),
                                deadline_seconds: Some(110),
                            },
                            intervention: Some("search Indexing is stalled".to_string()),
                        },
                        detail: Some(format!(
                            "indexing {count}/75934 files · blobs 1365 · fresh {fresh} · skipped 36239 · segments {segments} · bm25 cache 0 build 0"
                        )),
                    },
                },
                occurred_at,
            }
        };

        app.handle_message(hunt_event(75921, 38318, 332391, started));
        app.handle_message(governor_event(75921, 38318, 332391, started));
        assert_eq!(app.rows.len(), rows_before + 2);

        app.handle_message(hunt_event(
            75922,
            38319,
            332405,
            started + Duration::from_millis(250),
        ));
        app.handle_message(governor_event(
            75922,
            38319,
            332405,
            started + Duration::from_millis(250),
        ));

        assert_eq!(app.rows.len(), rows_before + 2);

        app.handle_message(hunt_event(
            75925,
            38321,
            332410,
            started + Duration::from_millis(1500),
        ));
        app.handle_message(governor_event(
            75925,
            38321,
            332410,
            started + Duration::from_millis(1500),
        ));

        assert_eq!(app.rows.len(), rows_before + 3);
        assert_eq!(app.rows[rows_before].header, "• Hunting sample (Indexing)");
        assert!(
            app.rows[rows_before]
                .content
                .contains("files 75925/75934 (+4)")
        );
        assert!(app.rows[rows_before].content.contains("fresh 38321 (+3)"));
        assert!(
            app.rows[rows_before]
                .content
                .contains("segments 332410 (+19)")
        );
        assert_eq!(
            app.rows[rows_before + 1].header,
            "• Hunting (Indexing) — 1m 50s (eta 0ms) strategy=bm25"
        );
        assert!(app.rows[rows_before + 1].content.contains("75925/75934"));
        assert_eq!(app.rows[rows_before + 2].header, "• Governor: gathering");
        assert!(app.rows[rows_before + 2].content.contains("75925/75934"));
        assert!(
            app.rows[rows_before + 2]
                .content
                .contains("timeout=stalled")
        );
    }

    #[test]
    fn command_notice_rows_use_brighter_notice_styles() {
        let palette = detect_palette();
        let row = TranscriptRow::new(
            TranscriptRowKind::CommandNotice,
            "• Model catalog",
            "Current lanes...",
        );

        let lines = render_row_lines(&row, &palette);

        assert_eq!(
            lines[0].spans[0].style,
            palette.command_notice_header.add_modifier(Modifier::BOLD)
        );
        assert_eq!(lines[1].spans[0].style, palette.command_notice_body);
        assert_eq!(lines[1].spans[1].style, palette.command_notice_body);
    }

    #[test]
    fn duration_formatting_stays_compact() {
        assert_eq!(format_duration_compact(Duration::from_millis(125)), "125ms");
        assert_eq!(format_duration_compact(Duration::from_millis(1530)), "1.5s");
        assert_eq!(format_duration_compact(Duration::from_secs(65)), "1m 05s");
    }

    #[test]
    fn inline_viewport_height_remains_compact() {
        assert_eq!(inline_viewport_height_for_terminal(0), 5);
        assert_eq!(inline_viewport_height_for_terminal(7), 5);
        assert_eq!(inline_viewport_height_for_terminal(11), 9);
        assert_eq!(inline_viewport_height_for_terminal(48), 9);
    }

    #[test]
    fn idle_viewport_keeps_prompt_attached_to_scrollback() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        let _ = app.take_scrollback_rows();

        let buffer = render_buffer(&app, 80, 9);
        // transcript (5 empty) + prompt box (3) + status bar (1) = 9
        // Prompt box border with title starts at line 5.
        assert!(buffer_line(&buffer, 5).contains("Prompt"));
    }

    #[test]
    fn transcript_rows_render_timing_suffixes() {
        let palette = detect_palette();
        let row = TranscriptRow::new(TranscriptRowKind::Event, "• Routed", "direct").timed(
            TranscriptTiming {
                elapsed: Duration::from_millis(1530),
                delta: Some(Duration::from_millis(430)),
                kind: TranscriptTimingKind::TurnTotal,
                pace: Pace::Normal,
            },
        );

        let lines = render_row_lines(&row, &palette);

        let header_text: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(header_text.contains("1.5s total"));
        assert!(header_text.contains("(+430ms)"));
    }

    #[test]
    fn app_tracks_busy_state_through_reveal_completion() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "hello".to_string();

        app.submit_prompt();
        let prompt = app.dispatch_next_prompt();
        assert_eq!(prompt, Some(QueuedPrompt::Prompt("hello".to_string())));
        assert!(app.busy);
        assert_eq!(app.busy_phase, BusyPhase::Thinking);

        app.handle_message(super::UiMessage::TurnFinished {
            result: Ok("hi there".to_string()),
            occurred_at: Instant::now(),
        });
        app.sync_transcript(&transcript(&[(
            ConversationTranscriptSpeaker::Assistant,
            "hi there",
        )]));
        assert!(app.busy);
        assert_eq!(app.busy_phase, BusyPhase::Rendering);

        while app.busy {
            app.tick();
        }

        assert_eq!(
            app.rows.last().map(|row| row.content.as_str()),
            Some("hi there")
        );
        assert_eq!(app.busy_phase, BusyPhase::Idle);
    }

    #[test]
    fn app_accepts_steering_prompts_while_busy_and_queues_them() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );

        app.input = "first".to_string();
        app.submit_prompt();
        assert_eq!(
            app.dispatch_next_prompt(),
            Some(QueuedPrompt::Prompt("first".to_string()))
        );
        assert!(app.busy);

        app.input = "steer harder".to_string();
        app.submit_prompt();

        assert_eq!(app.input, "");
        assert_eq!(app.queued_prompts.len(), 1);
        assert!(matches!(
            app.queued_prompts.front(),
            Some(QueuedPrompt::Steering(candidate)) if candidate.prompt == "steer harder"
        ));
        assert!(
            app.rows
                .iter()
                .any(|row| row.header == "• Queued steering prompt")
        );
    }

    #[test]
    fn queued_prompt_dispatches_after_current_turn_finishes() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );

        app.input = "first".to_string();
        app.submit_prompt();
        assert_eq!(
            app.dispatch_next_prompt(),
            Some(QueuedPrompt::Prompt("first".to_string()))
        );

        app.input = "second".to_string();
        app.submit_prompt();

        app.handle_message(super::UiMessage::TurnFinished {
            result: Ok("done".to_string()),
            occurred_at: Instant::now(),
        });
        app.sync_transcript(&transcript(&[(
            ConversationTranscriptSpeaker::Assistant,
            "done",
        )]));
        while app.busy {
            app.tick();
        }

        assert!(matches!(
            app.dispatch_next_prompt(),
            Some(QueuedPrompt::Steering(candidate)) if candidate.prompt == "second"
        ));
        assert!(app.busy);
        assert_eq!(app.busy_phase, BusyPhase::Thinking);
    }

    #[test]
    fn completed_rows_become_scrollback_while_live_reveal_stays_inline() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );

        let initial = app.take_scrollback_rows();
        assert_eq!(initial.len(), 1);
        assert_eq!(initial[0].header, "• Interactive mode ready");

        app.input = "hello".to_string();
        app.submit_prompt();
        app.sync_transcript(&transcript(&[(
            ConversationTranscriptSpeaker::User,
            "hello",
        )]));
        let user_rows = app.take_scrollback_rows();
        assert_eq!(user_rows.len(), 1);
        assert_eq!(user_rows[0].header, "User");

        app.dispatch_next_prompt();
        app.handle_message(super::UiMessage::TurnFinished {
            result: Ok("hi there".to_string()),
            occurred_at: Instant::now(),
        });
        app.sync_transcript(&transcript(&[
            (ConversationTranscriptSpeaker::User, "hello"),
            (ConversationTranscriptSpeaker::Assistant, "hi there"),
        ]));

        assert!(app.take_scrollback_rows().is_empty());
        assert_eq!(app.visible_live_rows(80, 20).len(), 1);
        assert_eq!(app.visible_live_rows(80, 20)[0].header, "Paddles");

        while app.busy {
            app.tick();
        }

        let assistant_rows = app.take_scrollback_rows();
        assert_eq!(assistant_rows.len(), 1);
        assert_eq!(assistant_rows[0].header, "Paddles");
        assert_eq!(assistant_rows[0].content, "hi there");
        assert!(app.visible_live_rows(80, 20).is_empty());
        assert_eq!(app.live_tail_height(80, 20), 0);
    }

    #[test]
    fn idle_prompt_line_has_no_redundant_send_hint() {
        let palette = detect_palette();
        let app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );

        let placeholder = app.input_placeholder();
        assert_eq!(placeholder, "Type a prompt...");
    }

    #[test]
    fn login_command_enters_masked_mode_for_remote_provider() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "moonshot-v1".to_string(),
            palette,
            session(),
            "moonshot".to_string(),
            Some("moonshot".to_string()),
            "Provider: `moonshot`. Auth: loaded from the local credential store.".to_string(),
            2,
        );
        app.input = "/login".to_string();

        app.submit_prompt();

        assert!(matches!(
            app.input_mode,
            InputMode::MaskedKey { ref provider } if provider == "moonshot"
        ));
        assert!(
            app.rows
                .iter()
                .any(|row| row.header == "• Login" && row.content.contains("Input is masked"))
        );
    }

    #[test]
    fn login_command_accepts_explicit_provider_when_current_lane_is_local() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "/login openai".to_string();

        app.submit_prompt();

        assert!(matches!(
            app.input_mode,
            InputMode::MaskedKey { ref provider } if provider == "openai"
        ));
    }

    #[test]
    fn login_command_accepts_inception_provider_when_current_lane_is_local() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "/login inception".to_string();

        app.submit_prompt();

        assert!(matches!(
            app.input_mode,
            InputMode::MaskedKey { ref provider } if provider == "inception"
        ));
    }

    #[test]
    fn login_submission_queues_pending_key_and_masks_display_text() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "moonshot-v1".to_string(),
            palette,
            session(),
            "moonshot".to_string(),
            Some("moonshot".to_string()),
            "Provider: `moonshot`. Auth: loaded from the local credential store.".to_string(),
            2,
        );
        app.input_mode = InputMode::MaskedKey {
            provider: "moonshot".to_string(),
        };
        app.input = "sk-secret-123".to_string();

        let (text, _style) = app.input_display_line(true);
        assert_eq!(text, "\u{2022}".repeat("sk-secret-123".chars().count()));

        app.submit_prompt();

        let pending = app.take_pending_login().expect("pending login");
        assert_eq!(pending.provider, "moonshot");
        assert_eq!(pending.api_key, "sk-secret-123");
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn login_command_is_rejected_for_local_provider() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "/login".to_string();

        app.submit_prompt();

        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(
            app.rows
                .iter()
                .any(|row| row.kind == TranscriptRowKind::Error
                    && row.header == "• Login unavailable"
                    && row.content.contains("does not use API-key login"))
        );
    }

    #[test]
    fn model_command_lists_enabled_and_disabled_provider_catalog_entries() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        let runtime_lanes = RuntimeLaneConfig::new("gpt-4o".to_string(), None)
            .with_synthesizer_provider(ModelProvider::Openai)
            .with_planner_provider(Some(ModelProvider::Anthropic))
            .with_planner_model_id(Some("claude-sonnet-4-20250514".to_string()));
        app.set_runtime_catalog(
            runtime_lanes.clone(),
            vec![
                provider_availability(ModelProvider::Sift, true, "auth not required"),
                provider_availability(ModelProvider::Openai, true, "using local credential store"),
                provider_availability(ModelProvider::Anthropic, false, "login required"),
                provider_availability(ModelProvider::Google, false, "login required"),
                provider_availability(ModelProvider::Inception, false, "login required"),
                provider_availability(ModelProvider::Moonshot, false, "login required"),
                provider_availability(ModelProvider::Ollama, true, "auth not required"),
            ],
        );
        app.input = "/model".to_string();

        app.submit_prompt();

        let row = app.rows.last().expect("catalog row");
        assert_eq!(row.kind, TranscriptRowKind::CommandNotice);
        assert_eq!(row.header, "• Model catalog");
        assert!(row.content.contains(&runtime_lane_summary(&runtime_lanes)));
        assert!(row.content.contains("[enabled] openai"));
        assert!(row.content.contains("[disabled] anthropic"));
        assert!(
            row.content
                .contains("[disabled] inception: mercury-2 (login required)")
        );
    }

    #[test]
    fn model_command_queues_runtime_update_for_enabled_provider() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.set_runtime_catalog(
            RuntimeLaneConfig::new("qwen-1.5b".to_string(), None)
                .with_synthesizer_provider(ModelProvider::Sift),
            vec![
                provider_availability(ModelProvider::Sift, true, "auth not required"),
                provider_availability(ModelProvider::Openai, true, "using local credential store"),
            ],
        );
        app.input = "/model synthesizer openai gpt-4o".to_string();

        app.submit_prompt();

        let update = app
            .take_pending_runtime_update()
            .expect("pending runtime update");
        assert_eq!(
            update.runtime_lanes.synthesizer_provider(),
            ModelProvider::Openai
        );
        assert_eq!(update.runtime_lanes.synthesizer_model_id(), "gpt-4o");
        assert_eq!(
            update.persisted_preferences.provider.as_deref(),
            Some("openai")
        );
        assert_eq!(
            update.persisted_preferences.model.as_deref(),
            Some("gpt-4o")
        );
        assert!(update.persisted_preferences.planner_provider.is_none());
        assert!(update.persisted_preferences.planner_model.is_none());
    }

    #[test]
    fn model_command_rejects_disabled_provider_selection() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.set_runtime_catalog(
            RuntimeLaneConfig::new("qwen-1.5b".to_string(), None)
                .with_synthesizer_provider(ModelProvider::Sift),
            vec![
                provider_availability(ModelProvider::Sift, true, "auth not required"),
                provider_availability(ModelProvider::Anthropic, false, "login required"),
            ],
        );
        app.input = "/model planner anthropic claude-sonnet-4-20250514".to_string();

        app.submit_prompt();

        assert!(app.take_pending_runtime_update().is_none());
        assert!(
            app.rows
                .iter()
                .any(|row| row.kind == TranscriptRowKind::Error
                    && row.header == "• Model unavailable"
                    && row.content.contains("disabled"))
        );
    }

    #[test]
    fn model_catalog_mentions_runtime_lane_state_file() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.set_runtime_preference_path(PathBuf::from(
            "/home/alex/.local/state/paddles/runtime-lanes.toml",
        ));
        app.set_runtime_catalog(
            RuntimeLaneConfig::new("qwen-1.5b".to_string(), None)
                .with_synthesizer_provider(ModelProvider::Sift),
            vec![provider_availability(
                ModelProvider::Sift,
                true,
                "auth not required",
            )],
        );
        app.input = "/model".to_string();

        app.submit_prompt();

        let row = app.rows.last().expect("catalog row");
        assert!(row.content.contains("Runtime lane state"));
        assert!(row.content.contains("runtime-lanes.toml"));
        assert!(row.content.contains("Workspace `paddles.toml` overrides"));
    }

    #[test]
    fn slash_command_popup_renders_matching_commands() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "/mod".to_string();
        app.cursor_pos = app.input.chars().count();

        let buffer = render_buffer(&app, 100, 18);
        let rendered = buffer_text(&buffer);
        let suggestions = app
            .slash_command_suggestions()
            .into_iter()
            .map(|command| command.usage)
            .collect::<Vec<_>>();

        assert!(rendered.contains("Commands"));
        assert!(rendered.contains("/model"));
        assert!(rendered.contains("/model planner"));
        assert_eq!(
            suggestions,
            vec![
                "/model",
                "/model planner <provider> <model>",
                "/model synthesizer <provider> <model>",
            ]
        );
    }

    #[test]
    fn slash_command_popup_renders_login_provider_suggestions() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "/login i".to_string();
        app.cursor_pos = app.input.chars().count();

        let buffer = render_buffer(&app, 100, 18);
        let rendered = buffer_text(&buffer);
        let suggestions = app
            .slash_command_suggestions()
            .into_iter()
            .map(|command| command.usage)
            .collect::<Vec<_>>();

        assert!(rendered.contains("Commands"));
        assert!(rendered.contains("/login inception"));
        assert_eq!(suggestions, vec!["/login inception"]);
    }

    #[test]
    fn slash_command_popup_renders_model_provider_suggestions() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "/model planner inc".to_string();
        app.cursor_pos = app.input.chars().count();

        let buffer = render_buffer(&app, 120, 18);
        let rendered = buffer_text(&buffer);
        let suggestions = app
            .slash_command_suggestions()
            .into_iter()
            .map(|command| command.usage)
            .collect::<Vec<_>>();

        assert!(rendered.contains("Commands"));
        assert!(rendered.contains("/model planner inception "));
        assert_eq!(suggestions, vec!["/model planner inception "]);
    }

    #[test]
    fn slash_command_popup_renders_model_id_suggestions() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "/model synthesizer inception m".to_string();
        app.cursor_pos = app.input.chars().count();

        let buffer = render_buffer(&app, 120, 18);
        let rendered = buffer_text(&buffer);
        let suggestions = app
            .slash_command_suggestions()
            .into_iter()
            .map(|command| command.usage)
            .collect::<Vec<_>>();

        assert!(rendered.contains("Commands"));
        assert!(rendered.contains("/model synthesizer inception mercury-2"));
        assert_eq!(suggestions, vec!["/model synthesizer inception mercury-2"]);
    }

    #[test]
    fn slash_command_login_provider_suggestions_skip_non_login_providers() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "/login s".to_string();
        app.cursor_pos = app.input.chars().count();

        assert!(app.slash_command_suggestions().is_empty());
    }

    #[test]
    fn slash_command_model_suggestions_do_not_fake_freeform_model_ids() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "/model planner ollama ".to_string();
        app.cursor_pos = app.input.chars().count();

        assert!(app.slash_command_suggestions().is_empty());
    }

    #[test]
    fn escape_keeps_interactive_loop_running_in_normal_mode() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "debug the tui".to_string();
        app.cursor_pos = app.input.chars().count();

        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

        assert!(!should_exit);
        assert_eq!(app.input, "debug the tui");
        assert_eq!(app.cursor_pos, "debug the tui".chars().count());
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn escape_cancels_masked_login_input_without_exiting() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input_mode = InputMode::MaskedKey {
            provider: "inception".to_string(),
        };
        app.input = "secret-key".to_string();
        app.cursor_pos = app.input.chars().count();

        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

        assert!(!should_exit);
        assert_eq!(app.input, "");
        assert_eq!(app.cursor_pos, 0);
        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(app.rows.iter().any(|row| row.header == "• Login cancelled"
            && row.content.contains("Returned to normal input.")));
    }

    #[test]
    fn escape_removes_latest_queued_steering_prompt_without_exiting() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );

        app.input = "first".to_string();
        app.submit_prompt();
        assert_eq!(
            app.dispatch_next_prompt(),
            Some(QueuedPrompt::Prompt("first".to_string()))
        );
        assert!(app.busy);

        app.input = "Are you stuck?".to_string();
        app.submit_prompt();
        assert_eq!(app.queued_prompts.len(), 1);

        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

        assert!(!should_exit);
        assert!(app.queued_prompts.is_empty());
        assert!(
            app.rows
                .iter()
                .any(|row| row.header == "• Cancelled queued steering prompt"
                    && row.content.contains("Are you stuck?"))
        );
    }

    #[test]
    fn slash_command_popup_area_stays_within_offset_frame() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "/model".to_string();
        app.cursor_pos = app.input.chars().count();

        let frame_area = Rect::new(0, 41, 239, 9);
        let input_area = Rect::new(0, 48, 239, 2);
        let popup = app
            .command_popup_area(frame_area, input_area)
            .expect("popup area");

        assert!(popup.y >= frame_area.y);
        assert!(popup.bottom() <= frame_area.bottom());
    }

    #[test]
    fn slash_command_completion_accepts_active_suggestion() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "/mod".to_string();
        app.cursor_pos = app.input.chars().count();

        assert!(app.accept_selected_slash_completion());
        assert_eq!(app.input, "/model");
        assert_eq!(app.cursor_pos, "/model".chars().count());
    }

    #[test]
    fn slash_command_completion_can_target_subcommands() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "/model p".to_string();
        app.cursor_pos = app.input.chars().count();

        assert!(app.accept_selected_slash_completion());
        assert_eq!(app.input, "/model planner ");
    }

    #[test]
    fn slash_command_completion_can_target_login_provider() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "/login inc".to_string();
        app.cursor_pos = app.input.chars().count();

        assert!(app.accept_selected_slash_completion());
        assert_eq!(app.input, "/login inception");
        assert_eq!(app.cursor_pos, "/login inception".chars().count());
    }

    #[test]
    fn slash_command_completion_can_target_model_provider() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "/model planner inc".to_string();
        app.cursor_pos = app.input.chars().count();

        assert!(app.accept_selected_slash_completion());
        assert_eq!(app.input, "/model planner inception ");
        assert_eq!(app.cursor_pos, "/model planner inception ".chars().count());
    }

    #[test]
    fn slash_command_completion_can_target_model_id() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        app.input = "/model synthesizer inception mer".to_string();
        app.cursor_pos = app.input.chars().count();

        assert!(app.accept_selected_slash_completion());
        assert_eq!(app.input, "/model synthesizer inception mercury-2");
        assert_eq!(
            app.cursor_pos,
            "/model synthesizer inception mercury-2".chars().count()
        );
    }

    #[test]
    fn app_attaches_elapsed_and_delta_timings_to_turn_rows() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );
        let start = Instant::now();

        app.input = "hello".to_string();
        app.submit_prompt();
        assert_eq!(
            app.dispatch_next_prompt_at(start),
            Some(QueuedPrompt::Prompt("hello".to_string()))
        );

        app.handle_message(super::UiMessage::TurnEvent {
            event: TurnEvent::RouteSelected {
                summary: "direct".to_string(),
            },
            occurred_at: start + Duration::from_millis(120),
        });
        app.handle_message(super::UiMessage::TurnFinished {
            result: Ok("done".to_string()),
            occurred_at: start + Duration::from_millis(350),
        });
        app.sync_transcript(&transcript(&[(
            ConversationTranscriptSpeaker::Assistant,
            "done",
        )]));

        let event_row = app
            .rows
            .iter()
            .find(|row| row.header == "• Routed")
            .unwrap();
        assert_eq!(
            event_row.timing,
            Some(TranscriptTiming {
                elapsed: Duration::from_millis(120),
                delta: None,
                kind: TranscriptTimingKind::Step,
                pace: Pace::Normal,
            })
        );

        let assistant_row = app.rows.last().expect("assistant row");
        assert_eq!(assistant_row.header, "Paddles");
        assert_eq!(
            assistant_row.timing,
            Some(TranscriptTiming {
                elapsed: Duration::from_millis(350),
                delta: Some(Duration::from_millis(230)),
                kind: TranscriptTimingKind::TurnTotal,
                pace: Pace::Normal,
            })
        );
    }

    #[test]
    fn transcript_update_for_current_task_requests_transcript_sync() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            0,
        );

        app.handle_message(super::UiMessage::transcript_updated(
            ConversationTranscriptUpdate {
                task_id: TaskTraceId::new("task-1").expect("task"),
            },
        ));

        assert!(app.take_transcript_sync_request());
        assert!(!app.take_transcript_sync_request());
    }

    #[test]
    fn transcript_update_for_other_task_is_ignored() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            0,
        );

        app.handle_message(super::UiMessage::transcript_updated(
            ConversationTranscriptUpdate {
                task_id: TaskTraceId::new("task-2").expect("task"),
            },
        ));

        assert!(!app.take_transcript_sync_request());
    }

    #[test]
    fn in_flight_row_shows_contextual_label() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            0,
        );
        let rows_before = app.rows.len();

        // Simulate: planner capability just completed, now silence for >2s.
        app.busy = true;
        app.busy_phase = BusyPhase::Thinking;
        app.last_event = Some((
            TurnEvent::PlannerCapability {
                provider: "kimi".to_string(),
                capability: "available".to_string(),
            },
            Instant::now() - Duration::from_secs(3),
        ));

        app.tick();
        assert_eq!(app.rows.len(), rows_before + 1);
        let row = app.rows.last().expect("in-flight row");
        assert_eq!(row.kind, TranscriptRowKind::InFlightEvent);
        assert_eq!(row.header, "• Planning...");
        assert!(app.emitted_in_flight);

        // Further ticks should NOT insert more rows.
        app.tick();
        assert_eq!(app.rows.len(), rows_before + 1);
    }

    #[test]
    fn in_flight_row_says_synthesizing_after_planner_summary() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            0,
        );
        app.busy = true;
        app.busy_phase = BusyPhase::Thinking;
        app.last_event = Some((
            TurnEvent::PlannerSummary {
                strategy: "direct".to_string(),
                mode: "single".to_string(),
                turns: 1,
                steps: 1,
                stop_reason: None,
                active_branch_id: None,
                branch_count: None,
                frontier_count: None,
                node_count: None,
                edge_count: None,
                retained_artifact_count: None,
            },
            Instant::now() - Duration::from_secs(3),
        ));

        app.tick();
        let row = app.rows.last().expect("in-flight row");
        assert_eq!(row.header, "• Synthesizing...");
    }

    #[test]
    fn in_flight_row_says_hunting_for_gatherer_progress() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            0,
        );
        app.busy = true;
        app.busy_phase = BusyPhase::Thinking;
        app.last_event = Some((
            TurnEvent::GathererSearchProgress {
                phase: "Indexing".to_string(),
                elapsed_seconds: 3,
                eta_seconds: None,
                strategy: Some("bm25".to_string()),
                detail: Some("indexing 4/10 files".to_string()),
            },
            Instant::now() - Duration::from_secs(3),
        ));

        app.tick();
        let row = app.rows.last().expect("in-flight row");
        assert_eq!(row.header, "• Hunting (Indexing)... strategy=bm25");
        assert_eq!(row.content, "indexing 4/10 files");
    }

    #[test]
    fn hunting_in_flight_row_is_reused_instead_of_accumulating() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            0,
        );
        app.busy = true;
        app.busy_phase = BusyPhase::Thinking;

        let first_at = Instant::now() - Duration::from_secs(3);
        app.handle_message(UiMessage::TurnEvent {
            event: TurnEvent::GathererSearchProgress {
                phase: "Indexing".to_string(),
                elapsed_seconds: 3,
                eta_seconds: Some(12),
                strategy: Some("bm25".to_string()),
                detail: Some("indexing 4/10 files".to_string()),
            },
            occurred_at: first_at,
        });
        let rows_after_progress = app.rows.len();

        app.last_event = Some((
            TurnEvent::GathererSearchProgress {
                phase: "Indexing".to_string(),
                elapsed_seconds: 3,
                eta_seconds: Some(12),
                strategy: Some("bm25".to_string()),
                detail: Some("indexing 4/10 files".to_string()),
            },
            Instant::now() - Duration::from_secs(3),
        ));
        app.tick();
        assert_eq!(app.rows.len(), rows_after_progress + 1);
        assert_eq!(
            app.rows.last().expect("in-flight row").header,
            "• Hunting (Indexing)... strategy=bm25"
        );

        app.handle_message(UiMessage::TurnEvent {
            event: TurnEvent::GathererSearchProgress {
                phase: "Indexing".to_string(),
                elapsed_seconds: 5,
                eta_seconds: Some(10),
                strategy: Some("bm25".to_string()),
                detail: Some("indexing 5/10 files".to_string()),
            },
            occurred_at: Instant::now(),
        });
        assert_eq!(
            app.rows
                .iter()
                .filter(|row| row.kind == TranscriptRowKind::InFlightEvent)
                .count(),
            0,
            "new gather progress should clear the stale in-flight fallback before rendering the next live update"
        );
        assert_eq!(
            app.rows
                .iter()
                .filter(|row| row.header.starts_with("• Hunting sample"))
                .count(),
            1,
            "aggregated hunting history may be recorded, but stale in-flight rows should not accumulate"
        );

        app.last_event = Some((
            TurnEvent::GathererSearchProgress {
                phase: "Indexing".to_string(),
                elapsed_seconds: 5,
                eta_seconds: Some(10),
                strategy: Some("bm25".to_string()),
                detail: Some("indexing 5/10 files".to_string()),
            },
            Instant::now() - Duration::from_secs(3),
        ));
        app.tick();
        assert_eq!(
            app.rows
                .iter()
                .filter(|row| row.kind == TranscriptRowKind::InFlightEvent)
                .count(),
            1,
            "only one live hunting in-flight row should exist after silence resumes"
        );
        let row = app.rows.last().expect("updated in-flight row");
        assert_eq!(row.header, "• Hunting (Indexing)... strategy=bm25");
        assert_eq!(row.content, "indexing 5/10 files");
    }

    #[test]
    fn in_flight_row_not_emitted_for_quick_events() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            0,
        );
        let rows_before = app.rows.len();

        app.busy = true;
        app.busy_phase = BusyPhase::Thinking;
        app.last_event = Some((
            TurnEvent::RouteSelected {
                summary: "direct".to_string(),
            },
            Instant::now(),
        ));

        app.tick();
        assert_eq!(app.rows.len(), rows_before);
        assert!(!app.emitted_in_flight);
    }

    #[test]
    fn new_event_resets_in_flight_flag() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            0,
        );

        app.busy = true;
        app.busy_phase = BusyPhase::Thinking;
        app.emitted_in_flight = true;

        app.handle_message(super::UiMessage::TurnEvent {
            event: TurnEvent::RouteSelected {
                summary: "direct".to_string(),
            },
            occurred_at: Instant::now(),
        });
        assert!(!app.emitted_in_flight);
        assert!(app.last_event.is_some());
    }

    #[test]
    fn app_submits_multiline_prompt_as_inline_display_row() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );

        app.input = "line one\nline two\nline three".to_string();
        app.submit_prompt();
        app.sync_transcript(&transcript(&[(
            ConversationTranscriptSpeaker::User,
            "line one\nline two\nline three",
        )]));

        let last_row = app.rows.last().expect("user row exists");
        assert_eq!(last_row.kind, TranscriptRowKind::User);
        assert_eq!(last_row.content, "line one ⏎ line two ⏎ line three");
    }

    #[test]
    fn app_recalls_loaded_prompt_history_across_instances() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            2,
        );

        app.load_prompt_history(vec![
            "first prompt".to_string(),
            "second prompt".to_string(),
        ]);
        app.history_back();
        assert_eq!(app.input, "second prompt");

        app.history_back();
        assert_eq!(app.input, "first prompt");

        app.history_forward();
        assert_eq!(app.input, "second prompt");
    }

    #[test]
    fn inline_multiline_text_preserves_single_lines() {
        assert_eq!(inline_multiline_text("single line"), "single line");
    }
}
