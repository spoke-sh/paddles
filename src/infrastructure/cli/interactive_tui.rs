use crate::application::{
    ConversationSession, ConversationTranscript, ConversationTranscriptSpeaker,
    ConversationTranscriptUpdate, MechSuitService, ResumableConversation, RuntimeLaneConfig,
    TranscriptUpdateSink,
};
use crate::domain::model::render::uses_compact_block_separator;
use crate::domain::model::{
    RenderBlock, RenderDocument, RuntimeItem, TaskTraceId, ThreadCandidate, TurnEvent,
    TurnEventSink,
};
use crate::infrastructure::credentials::{CredentialStore, ProviderAvailability};
use crate::infrastructure::providers::ModelProvider;
use crate::infrastructure::runtime_preferences::{
    RuntimeLanePreferenceStore, RuntimeLanePreferences,
};
use crate::infrastructure::runtime_presentation::{
    RuntimeEventPresentation, project_runtime_event_for_tui,
};
use crate::infrastructure::step_timing::{Pace, StepTimingReservoir};
use anyhow::Result;
use crossterm::cursor;
use crossterm::event::{
    self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size as terminal_size};
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap};
use ratatui::{Frame, Terminal, TerminalOptions, Viewport};
use std::cmp;
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::io::{self, Write};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::task::JoinHandle;

/// Context passed from main to the TUI for credential management.
pub struct TuiContext {
    pub credential_store: Arc<CredentialStore>,
    pub runtime_preference_store: Arc<RuntimeLanePreferenceStore>,
    pub runtime_lanes: RuntimeLaneConfig,
    pub web_server_addr: SocketAddr,
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
const PASTE_PREVIEW_LIMIT: usize = 48;

fn normalize_pasted_text(text: &str) -> String {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut trimmed_prefix_len = 0usize;

    for segment in normalized.split_inclusive('\n') {
        if segment.trim().is_empty() {
            trimmed_prefix_len += segment.len();
            continue;
        }
        break;
    }

    let trimmed = &normalized[trimmed_prefix_len..];
    let had_trailing_newline = trimmed.ends_with('\n');
    let mut lines = Vec::new();
    let split_lines = trimmed.split('\n').collect::<Vec<_>>();

    for (index, line) in split_lines.iter().enumerate() {
        if had_trailing_newline && index + 1 == split_lines.len() && line.is_empty() {
            continue;
        }
        let trimmed_line = line.trim();
        if trimmed_line != "```"
            && !trimmed_line.starts_with("```")
            && trimmed_line.ends_with("```")
            && let Some(fence_index) = line.rfind("```")
        {
            let content = line[..fence_index].trim_end();
            if !content.is_empty() {
                lines.push(content.to_string());
            }
            lines.push("```".to_string());
            continue;
        }

        lines.push(line.to_string());
    }

    let mut result = lines.join("\n");
    if had_trailing_newline && !result.is_empty() {
        result.push('\n');
    }
    result
}

fn pasted_line_count(text: &str) -> usize {
    let normalized = normalize_pasted_text(text);
    let trimmed = normalized.trim_end_matches('\n');
    if trimmed.is_empty() {
        return 0;
    }
    trimmed.split('\n').count()
}

fn should_compress_pasted_text(text: &str) -> bool {
    pasted_line_count(text) > 1
}

fn pasted_preview(text: &str) -> String {
    let normalized = normalize_pasted_text(text);
    let trimmed = normalized.trim_end();
    let preview_source = trimmed
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .or_else(|| trimmed.lines().next())
        .unwrap_or("");
    trim_for_display(preview_source, PASTE_PREVIEW_LIMIT)
}

fn collapse_text_part_trailing_blank_lines_for_paste(text: &str) -> String {
    if !text.ends_with('\n') && !text.ends_with('\r') {
        return text.to_string();
    }

    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines = normalized.split('\n').collect::<Vec<_>>();
    while matches!(lines.last(), Some(line) if line.trim().is_empty()) {
        lines.pop();
    }
    if lines.is_empty() {
        return String::new();
    }

    let mut collapsed = lines.join("\n");
    collapsed.push('\n');
    collapsed
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
enum ComposerPart {
    Text(String),
    Paste {
        text: String,
        lines: usize,
        preview: String,
    },
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
        description: "choose the shared runtime model",
    },
    SlashCommandSpec {
        insert_text: "/resume",
        usage: "/resume",
        description: "list or restore persisted conversations",
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

pub async fn run_interactive_tui(service: Arc<MechSuitService>, tui_ctx: TuiContext) -> Result<()> {
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
    if let Ok(transcript) = service.replay_conversation_transcript(&app.session.task_id()) {
        app.load_transcript(&transcript);
    }
    app.rows.push(web_server_ready_row(tui_ctx.web_server_addr));

    loop {
        drain_messages(&mut app, &mut rx);
        if let Some(command) = app.take_pending_resume_command() {
            match command {
                PendingResumeCommand::List => match service.resumable_conversations() {
                    Ok(conversations) => app.show_resumable_conversations(&conversations),
                    Err(err) => app.push_error(
                        "Resume command failed",
                        format!("Could not load persisted conversations: {err:#}"),
                    ),
                },
                PendingResumeCommand::Restore { task_id } => {
                    let task_id =
                        TaskTraceId::new(task_id).expect("resume commands validate task ids");
                    match service.restore_shared_conversation_session(&task_id) {
                        Ok(session) => match service.replay_conversation_transcript(&task_id) {
                            Ok(transcript) => app.restore_resumed_session(session, &transcript),
                            Err(err) => app.push_error(
                                "Resume command failed",
                                format!(
                                    "Could not replay transcript for `{}`: {err:#}",
                                    task_id.as_str()
                                ),
                            ),
                        },
                        Err(err) => app.push_error(
                            "Resume command failed",
                            format!("Could not restore `{}`: {err:#}", task_id.as_str()),
                        ),
                    }
                }
            }
        }
        if app.take_transcript_sync_request()
            && let Ok(transcript) = service.replay_conversation_transcript(&app.session.task_id())
        {
            app.sync_transcript(&transcript);
        }
        app.tick();
        if let Some(update) = app.dispatch_pending_runtime_update() {
            let work_id = app.next_work_id();
            let handle = dispatch_runtime_update(
                update,
                Arc::clone(&service),
                Arc::clone(&tui_ctx.credential_store),
                Arc::clone(&tui_ctx.runtime_preference_store),
                tx.clone(),
                work_id,
            );
            app.set_active_work(InFlightWorkKind::RuntimeUpdate, work_id, handle);
        }
        if let Some(prompt) = app.dispatch_next_prompt() {
            let work_id = app.next_work_id();
            let handle = dispatch_prompt(
                prompt,
                Arc::clone(&service),
                app.session.clone(),
                tx.clone(),
                work_id,
            );
            app.set_active_work(InFlightWorkKind::Prompt, work_id, handle);
        }

        // Handle completed /login actions.
        if let Some(login) = app.take_pending_login() {
            match tui_ctx
                .credential_store
                .save_api_key(&login.provider, &login.api_key)
            {
                Ok(()) => match service.prepare_runtime_lanes(&app.runtime_lanes).await {
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

        flush_scrollback_rows(&mut terminal, &mut app)?;
        terminal.draw(|frame| app.render(frame))?;

        if event::poll(FRAME_INTERVAL)? {
            match event::read()? {
                Event::Key(key) if key.kind != KeyEventKind::Release => {
                    if handle_key_event(&mut app, key) {
                        break;
                    }
                }
                Event::Paste(data) => handle_paste_event(&mut app, &data),
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

fn handle_paste_event(app: &mut InteractiveApp, text: &str) {
    let normalized = normalize_pasted_text(text);
    app.clear_model_selection();
    if app.is_masked_input() || !should_compress_pasted_text(&normalized) {
        app.insert_text_at_cursor(&normalized);
        return;
    }

    app.history_cursor = None;
    app.reset_slash_suggestion_selection();
    app.push_input_as_text_part();
    app.composer_parts.push(ComposerPart::Paste {
        text: normalized.clone(),
        lines: pasted_line_count(&normalized),
        preview: pasted_preview(&normalized),
    });
    app.input.clear();
    app.cursor_pos = 0;
}

fn handle_key_event(app: &mut InteractiveApp, key: KeyEvent) -> bool {
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
        if app.input.is_empty() && !app.has_composer_parts() {
            return true;
        }
        app.clear_model_selection();
        app.composer_parts.clear();
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
            } else if app.cancel_model_selection() || app.dismiss_slash_popup() {
                false
            } else {
                if app.busy {
                    if app.session.request_turn_interrupt().is_ok() {
                        app.push_event(
                            "Requested turn interrupt",
                            "The active turn will stop at the next safe checkpoint.",
                        );
                        return false;
                    }
                    if app.abort_active_work() {
                        app.clear_queued_prompts();
                    } else if app.input.is_empty() {
                        app.cancel_latest_queued_steering_prompt();
                    }
                    return false;
                } else if app.input.is_empty() {
                    app.cancel_latest_queued_steering_prompt();
                    return false;
                }
                false
            }
        }
        KeyCode::Enter => {
            if app.submit_model_selection() {
                return false;
            }
            app.submit_prompt();
            false
        }
        KeyCode::Tab => {
            app.accept_selected_slash_completion();
            false
        }
        KeyCode::Backspace => {
            app.clear_model_selection();
            if app.input.is_empty() && app.has_composer_parts() {
                app.pop_composer_part();
                return false;
            }
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
            app.clear_model_selection();
            if app.input.is_empty() && app.has_composer_parts() {
                app.pop_composer_part();
                return false;
            }
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
            app.clear_model_selection();
            if app.cursor_pos > 0 {
                app.cursor_pos -= 1;
            }
            false
        }
        KeyCode::Right => {
            app.clear_model_selection();
            let char_count = app.input.chars().count();
            if app.cursor_pos < char_count {
                app.cursor_pos += 1;
            }
            false
        }
        KeyCode::Home => {
            app.clear_model_selection();
            app.cursor_pos = 0;
            false
        }
        KeyCode::End => {
            app.clear_model_selection();
            app.cursor_pos = app.input.chars().count();
            false
        }
        KeyCode::Up => {
            if app.cycle_model_selection(-1) {
                return false;
            }
            if app.cycle_slash_suggestion(-1) {
                return false;
            }
            if !app.cursor_up() && !app.input.contains('\n') {
                app.history_back();
            }
            false
        }
        KeyCode::Down => {
            if app.cycle_model_selection(1) {
                return false;
            }
            if app.cycle_slash_suggestion(1) {
                return false;
            }
            if !app.cursor_down() && !app.input.contains('\n') {
                app.history_forward();
            }
            false
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.clear_model_selection();
            app.input.clear();
            app.cursor_pos = 0;
            app.reset_slash_suggestion_selection();
            false
        }
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.clear_model_selection();
            app.cursor_pos = 0;
            false
        }
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.clear_model_selection();
            app.cursor_pos = app.input.chars().count();
            false
        }
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.clear_model_selection();
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
                app.clear_model_selection();
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
        execute!(io::stdout(), cursor::Hide, EnableBracketedPaste)?;
        Ok(Self)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), DisableBracketedPaste, cursor::Show);
        let _ = disable_raw_mode();
        let _ = writeln!(io::stdout());
    }
}

#[derive(Debug)]
enum UiMessage {
    TurnEvent {
        event: TurnEvent,
        occurred_at: Instant,
        work_id: Option<u64>,
    },
    TranscriptUpdated {
        update: ConversationTranscriptUpdate,
    },
    TurnFinished {
        result: std::result::Result<String, String>,
        occurred_at: Instant,
        work_id: Option<u64>,
    },
    RuntimeUpdateFinished {
        result: std::result::Result<RuntimeUpdateCompletion, String>,
        occurred_at: Instant,
        work_id: Option<u64>,
    },
}

impl UiMessage {
    fn turn_event_with_id(event: TurnEvent, work_id: u64) -> Self {
        Self::TurnEvent {
            event,
            occurred_at: Instant::now(),
            work_id: Some(work_id),
        }
    }

    fn turn_finished_with_id(result: std::result::Result<String, String>, work_id: u64) -> Self {
        Self::TurnFinished {
            result,
            occurred_at: Instant::now(),
            work_id: Some(work_id),
        }
    }

    fn runtime_update_finished_with_id(
        result: std::result::Result<RuntimeUpdateCompletion, String>,
        work_id: u64,
    ) -> Self {
        Self::RuntimeUpdateFinished {
            result,
            occurred_at: Instant::now(),
            work_id: Some(work_id),
        }
    }

    fn transcript_updated(update: ConversationTranscriptUpdate) -> Self {
        Self::TranscriptUpdated { update }
    }
}

#[derive(Debug)]
enum InFlightWorkKind {
    Prompt,
    RuntimeUpdate,
}

#[derive(Debug)]
struct InFlightWork {
    id: u64,
    kind: InFlightWorkKind,
    handle: JoinHandle<()>,
}

#[derive(Clone, Debug)]
struct RuntimeUpdateCompletion {
    runtime_lanes: RuntimeLaneConfig,
    provider_availability: Vec<ProviderAvailability>,
    summary: String,
    preference_path: PathBuf,
    preference_save_error: Option<String>,
}

fn dispatch_prompt(
    prompt: QueuedPrompt,
    service: Arc<MechSuitService>,
    session: ConversationSession,
    tx: UnboundedSender<UiMessage>,
    work_id: u64,
) -> JoinHandle<()> {
    let sink = Arc::new(InteractiveTurnEventSink {
        tx: tx.clone(),
        work_id: Some(work_id),
    });
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
        let _ = tx.send(UiMessage::turn_finished_with_id(result, work_id));
    })
}

fn dispatch_runtime_update(
    update: PendingRuntimeUpdate,
    service: Arc<MechSuitService>,
    credential_store: Arc<CredentialStore>,
    runtime_preference_store: Arc<RuntimeLanePreferenceStore>,
    tx: UnboundedSender<UiMessage>,
    work_id: u64,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let result = match service.prepare_runtime_lanes(&update.runtime_lanes).await {
            Ok(_) => {
                let preference_save_error = runtime_preference_store
                    .save(&update.persisted_preferences)
                    .err()
                    .map(|err| format!("{err:#}"));
                Ok(RuntimeUpdateCompletion {
                    runtime_lanes: update.runtime_lanes,
                    provider_availability: credential_store.all_provider_availability(),
                    summary: update.summary,
                    preference_path: runtime_preference_store.path().to_path_buf(),
                    preference_save_error,
                })
            }
            Err(err) => Err(format!("{err:#}")),
        };
        let _ = tx.send(UiMessage::runtime_update_finished_with_id(result, work_id));
    })
}

#[derive(Clone)]
struct InteractiveTurnEventSink {
    tx: UnboundedSender<UiMessage>,
    work_id: Option<u64>,
}

impl TurnEventSink for InteractiveTurnEventSink {
    fn emit(&self, event: TurnEvent) {
        if let Some(work_id) = self.work_id {
            let _ = self.tx.send(UiMessage::turn_event_with_id(event, work_id));
        }
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
    transcript_record_id: Option<String>,
    render: Option<RenderDocument>,
    timing: Option<TranscriptTiming>,
    runtime_items: Vec<RuntimeItem>,
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
            transcript_record_id: None,
            render: None,
            timing: None,
            runtime_items: Vec::new(),
        }
    }

    fn with_transcript_record_id(mut self, record_id: impl Into<String>) -> Self {
        self.transcript_record_id = Some(record_id.into());
        self
    }

    fn with_render(mut self, render: RenderDocument) -> Self {
        self.render = Some(render);
        self
    }

    fn with_runtime_items(mut self, runtime_items: Vec<RuntimeItem>) -> Self {
        self.runtime_items = runtime_items;
        self
    }

    fn timed(mut self, timing: TranscriptTiming) -> Self {
        self.timing = Some(timing);
        self
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingTurnResultFallback {
    content: String,
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

fn busy_label(busy_phase: BusyPhase, last_event: Option<&TurnEvent>) -> String {
    match busy_phase {
        BusyPhase::Idle => "working".to_string(),
        BusyPhase::Thinking => last_event
            .map(|event| in_flight_label(event).to_ascii_lowercase())
            .unwrap_or_else(|| "thinking".to_string()),
        BusyPhase::Reconfiguring => "reconfiguring".to_string(),
        BusyPhase::Rendering => "rendering".to_string(),
    }
}

fn format_in_flight_row(last_event: &TurnEvent) -> TranscriptRow {
    match last_event {
        TurnEvent::ToolCalled {
            tool_name,
            invocation,
            ..
        } => TranscriptRow::new(
            TranscriptRowKind::InFlightEvent,
            format!("• {tool_name}..."),
            collapse_event_details(invocation, EVENT_DETAIL_LINE_LIMIT),
        ),
        TurnEvent::ToolOutput {
            tool_name,
            stream,
            output,
            ..
        } => TranscriptRow::new(
            TranscriptRowKind::InFlightEvent,
            format!("• {tool_name} {stream}..."),
            collapse_event_details(output, EVENT_DETAIL_LINE_LIMIT),
        ),
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
        TurnEvent::HarnessState { snapshot }
            if snapshot.chamber == crate::domain::model::HarnessChamber::Tooling =>
        {
            format_tooling_in_flight_row(snapshot.detail.as_deref())
        }
        event => TranscriptRow::new(
            TranscriptRowKind::InFlightEvent,
            format!("• {}...", in_flight_label(event)),
            "",
        ),
    }
}

fn format_tooling_in_flight_row(detail: Option<&str>) -> TranscriptRow {
    let Some(detail) = detail else {
        return TranscriptRow::new(
            TranscriptRowKind::InFlightEvent,
            "• Running tool...".to_string(),
            "".to_string(),
        );
    };
    if let Some((label, body)) = detail.split_once(": ") {
        return TranscriptRow::new(
            TranscriptRowKind::InFlightEvent,
            format!("• {label}..."),
            collapse_event_details(body, EVENT_DETAIL_LINE_LIMIT),
        );
    }
    TranscriptRow::new(
        TranscriptRowKind::InFlightEvent,
        "• Running tool...".to_string(),
        collapse_event_details(detail, EVENT_DETAIL_LINE_LIMIT),
    )
}

fn planner_step_matches_tool_call(
    planner_event: Option<&TurnEvent>,
    tool_name: &str,
    invocation: &str,
) -> bool {
    let Some(TurnEvent::PlannerStepProgress {
        action,
        query: Some(query),
        ..
    }) = planner_event
    else {
        return false;
    };

    query == invocation && (action == tool_name || action.starts_with(&format!("{tool_name} ")))
}

fn transcript_row_from_runtime_event(
    presentation: RuntimeEventPresentation,
    runtime_items: Vec<RuntimeItem>,
) -> TranscriptRow {
    let content = if presentation.detail.is_empty() {
        String::new()
    } else {
        collapse_event_details(&presentation.detail, EVENT_DETAIL_LINE_LIMIT)
    };

    TranscriptRow::new(TranscriptRowKind::Event, presentation.title, content)
        .with_runtime_items(runtime_items)
}

fn web_server_ready_row(addr: SocketAddr) -> TranscriptRow {
    TranscriptRow::new(
        TranscriptRowKind::CommandNotice,
        "• Web UI ready",
        format!(
            "HTTP server listening on {}.",
            crate::infrastructure::web::web_server_url(addr)
        ),
    )
}

struct InteractiveApp {
    model_label: String,
    palette: Palette,
    session: ConversationSession,
    runtime_lanes: RuntimeLaneConfig,
    provider_availability: Vec<ProviderAvailability>,
    rows: Vec<TranscriptRow>,
    input: String,
    composer_parts: Vec<ComposerPart>,
    queued_prompts: VecDeque<QueuedPrompt>,
    busy: bool,
    busy_phase: BusyPhase,
    next_work_id: u64,
    active_work: Option<InFlightWork>,
    pending_reveal: Option<PendingReveal>,
    spinner_index: usize,
    input_mode: InputMode,
    pending_login: Option<PendingLogin>,
    pending_resume_command: Option<PendingResumeCommand>,
    pending_runtime_update: Option<PendingRuntimeUpdate>,
    runtime_update_started_at: Option<Instant>,
    slash_suggestion_index: usize,
    provider_name: String,
    credential_provider: Option<String>,
    runtime_preference_path: Option<PathBuf>,
    active_turn_timing: Option<ActiveTurnTiming>,
    flushed_row_count: usize,
    search_progress_row: Option<usize>,
    gathering_harness_row: Option<usize>,
    planner_progress_row: Option<usize>,
    tool_output_rows: BTreeMap<(String, String), usize>,
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
    slash_popup_dismissed: bool,
    prompt_history: Vec<String>,
    history_cursor: Option<usize>,
    history_draft: String,
    current_task_id: String,
    pending_transcript_sync: bool,
    seen_transcript_record_ids: HashSet<String>,
    pending_turn_total_timing: Option<TranscriptTiming>,
    pending_turn_result_fallback: Option<PendingTurnResultFallback>,
    fallback_assistant_row: Option<usize>,
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
    Reconfiguring,
    Rendering,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum InputMode {
    Normal,
    MaskedKey { provider: String },
    ModelSelection(ModelSelectionState),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ModelSelectionStage {
    Provider,
    Model {
        provider: ModelProvider,
    },
    ThinkingMode {
        provider: ModelProvider,
        model: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ModelSelectionState {
    stage: ModelSelectionStage,
    selected_index: usize,
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

#[derive(Clone, Debug, PartialEq, Eq)]
enum PendingResumeCommand {
    List,
    Restore { task_id: String },
}

fn runtime_lane_summary(runtime_lanes: &RuntimeLaneConfig) -> String {
    let synthesizer_label = runtime_lanes
        .synthesizer_provider()
        .qualified_model_label_with_thinking(
            runtime_lanes.synthesizer_model_id(),
            runtime_lanes.synthesizer_thinking_mode(),
        );
    let planner_thinking_mode = (runtime_lanes.planner_provider()
        == runtime_lanes.synthesizer_provider()
        && runtime_lanes
            .planner_model_id()
            .unwrap_or(runtime_lanes.synthesizer_model_id())
            == runtime_lanes.synthesizer_model_id())
    .then(|| runtime_lanes.synthesizer_thinking_mode())
    .flatten();
    format!(
        "P {} · S {}",
        runtime_lanes
            .planner_provider()
            .qualified_model_label_with_thinking(
                runtime_lanes
                    .planner_model_id()
                    .unwrap_or(runtime_lanes.synthesizer_model_id()),
                planner_thinking_mode,
            ),
        synthesizer_label
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
                 Type `/model`, then Enter, to choose the shared runtime model.\n\
                 Other slash commands open a popup; Tab accepts the active completion.",
            ),
            None => format!(
                "Enter to send, Ctrl+C to quit.\n\
                 {credential_status}\n\
                 Type `/login <provider>` for any remote provider.\n\
                 Type `/model`, then Enter, to choose the shared runtime model.\n\
                 Other slash commands open a popup; Tab accepts the active completion.",
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
            composer_parts: Vec::new(),
            queued_prompts: VecDeque::new(),
            busy: false,
            busy_phase: BusyPhase::Idle,
            next_work_id: 0,
            active_work: None,
            pending_reveal: None,
            spinner_index: 0,
            input_mode: InputMode::Normal,
            pending_login: None,
            pending_resume_command: None,
            pending_runtime_update: None,
            runtime_update_started_at: None,
            slash_suggestion_index: 0,
            provider_name,
            credential_provider,
            runtime_preference_path: None,
            active_turn_timing: None,
            flushed_row_count: 0,
            search_progress_row: None,
            gathering_harness_row: None,
            planner_progress_row: None,
            tool_output_rows: BTreeMap::new(),
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
            slash_popup_dismissed: false,
            prompt_history: Vec::new(),
            history_cursor: None,
            history_draft: String::new(),
            current_task_id,
            pending_transcript_sync: false,
            seen_transcript_record_ids: HashSet::new(),
            pending_turn_total_timing: None,
            pending_turn_result_fallback: None,
            fallback_assistant_row: None,
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
        self.slash_popup_dismissed = false;
    }

    fn dismiss_slash_popup(&mut self) -> bool {
        if self.slash_popup_dismissed || self.slash_command_suggestions().is_empty() {
            return false;
        }
        self.slash_popup_dismissed = true;
        true
    }

    fn model_selection_state(&self) -> Option<&ModelSelectionState> {
        match &self.input_mode {
            InputMode::ModelSelection(state) => Some(state),
            _ => None,
        }
    }

    fn clear_model_selection(&mut self) -> bool {
        if matches!(self.input_mode, InputMode::ModelSelection(_)) {
            self.input_mode = InputMode::Normal;
            return true;
        }
        false
    }

    fn start_model_provider_selection(&mut self) {
        let selected_index = ModelProvider::all()
            .iter()
            .position(|provider| *provider == self.runtime_lanes.synthesizer_provider())
            .unwrap_or(0);
        self.input = "/model".to_string();
        self.cursor_pos = self.input.chars().count();
        self.input_mode = InputMode::ModelSelection(ModelSelectionState {
            stage: ModelSelectionStage::Provider,
            selected_index,
        });
    }

    fn start_model_id_selection(&mut self, provider: ModelProvider) {
        let current_model = self.runtime_lanes.synthesizer_model_id();
        let selected_index = if self.runtime_lanes.synthesizer_provider() == provider {
            provider
                .selectable_model_ids()
                .iter()
                .position(|model_id| {
                    provider.selectable_model_matches_runtime_model(model_id, current_model)
                })
                .unwrap_or(0)
        } else {
            0
        };
        self.input = format!("/model {}", provider.name());
        self.cursor_pos = self.input.chars().count();
        self.input_mode = InputMode::ModelSelection(ModelSelectionState {
            stage: ModelSelectionStage::Model { provider },
            selected_index,
        });
    }

    fn start_model_thinking_mode_selection(&mut self, provider: ModelProvider, model: &str) {
        let current_model = self.runtime_lanes.synthesizer_model_id();
        let current_thinking_mode = self.runtime_lanes.synthesizer_thinking_mode();
        let selected_index = if self.runtime_lanes.synthesizer_provider() == provider {
            provider
                .thinking_mode_index_for_runtime_model(model, current_model, current_thinking_mode)
                .unwrap_or(0)
        } else {
            0
        };
        self.input = format!("/model {} {}", provider.name(), model);
        self.cursor_pos = self.input.chars().count();
        self.input_mode = InputMode::ModelSelection(ModelSelectionState {
            stage: ModelSelectionStage::ThinkingMode {
                provider,
                model: model.to_string(),
            },
            selected_index,
        });
    }

    fn model_selection_option_count(&self) -> usize {
        let Some(state) = self.model_selection_state() else {
            return 0;
        };
        match &state.stage {
            ModelSelectionStage::Provider => ModelProvider::all().len(),
            ModelSelectionStage::Model { provider } => provider.selectable_model_ids().len(),
            ModelSelectionStage::ThinkingMode { provider, model } => {
                provider.thinking_modes(model).len()
            }
        }
    }

    fn cycle_model_selection(&mut self, delta: isize) -> bool {
        let option_count = self.model_selection_option_count();
        if option_count == 0 {
            return false;
        }
        let len = option_count as isize;
        if let InputMode::ModelSelection(state) = &mut self.input_mode {
            let next = (state.selected_index as isize + delta).rem_euclid(len);
            state.selected_index = next as usize;
            return true;
        }
        false
    }

    fn cancel_model_selection(&mut self) -> bool {
        let Some(state) = self.model_selection_state().cloned() else {
            return false;
        };
        match state.stage {
            ModelSelectionStage::Provider => {
                self.input_mode = InputMode::Normal;
                self.input = "/model".to_string();
                self.cursor_pos = self.input.chars().count();
            }
            ModelSelectionStage::Model { provider } => {
                let selected_index = ModelProvider::all()
                    .iter()
                    .position(|candidate| *candidate == provider)
                    .unwrap_or(0);
                self.input = "/model".to_string();
                self.cursor_pos = self.input.chars().count();
                self.input_mode = InputMode::ModelSelection(ModelSelectionState {
                    stage: ModelSelectionStage::Provider,
                    selected_index,
                });
            }
            ModelSelectionStage::ThinkingMode { provider, model } => {
                let selected_index = provider
                    .selectable_model_ids()
                    .iter()
                    .position(|candidate| *candidate == model)
                    .unwrap_or(0);
                self.input = format!("/model {}", provider.name());
                self.cursor_pos = self.input.chars().count();
                self.input_mode = InputMode::ModelSelection(ModelSelectionState {
                    stage: ModelSelectionStage::Model { provider },
                    selected_index,
                });
            }
        }
        true
    }

    fn submit_model_selection(&mut self) -> bool {
        let Some(state) = self.model_selection_state().cloned() else {
            return false;
        };
        match state.stage {
            ModelSelectionStage::Provider => {
                let Some(provider) = ModelProvider::all().get(state.selected_index).copied() else {
                    return true;
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
                    return true;
                }
                if provider.supports_freeform_model_id() {
                    self.input_mode = InputMode::Normal;
                    self.input = format!("/model {} ", provider.name());
                    self.cursor_pos = self.input.chars().count();
                    self.push_event(
                        "Model selection",
                        format!(
                            "`{}` accepts freeform model ids. Type `/model {} <model>` and press Enter.",
                            provider.name(),
                            provider.name()
                        ),
                    );
                } else {
                    self.start_model_id_selection(provider);
                }
            }
            ModelSelectionStage::Model { provider } => {
                let Some(model_id) = provider
                    .selectable_model_ids()
                    .get(state.selected_index)
                    .map(ToString::to_string)
                else {
                    return true;
                };
                if provider.thinking_modes(&model_id).is_empty() {
                    if self.queue_model_selection(provider, &model_id, None) {
                        self.input.clear();
                        self.cursor_pos = 0;
                        self.input_mode = InputMode::Normal;
                        self.reset_slash_suggestion_selection();
                    }
                } else {
                    self.start_model_thinking_mode_selection(provider, &model_id);
                }
            }
            ModelSelectionStage::ThinkingMode { provider, model } => {
                let Some(thinking_mode) = provider
                    .thinking_modes(&model)
                    .get(state.selected_index)
                    .copied()
                else {
                    return true;
                };
                let resolved_model_id = thinking_mode.model_override.unwrap_or(model.as_str());
                if self.queue_model_selection(
                    provider,
                    resolved_model_id,
                    thinking_mode.thinking_mode,
                ) {
                    self.input.clear();
                    self.cursor_pos = 0;
                    self.input_mode = InputMode::Normal;
                    self.reset_slash_suggestion_selection();
                }
            }
        }
        true
    }

    fn model_selection_lines(&self) -> Option<Vec<Line<'static>>> {
        let state = self.model_selection_state()?;
        let mut lines = Vec::new();
        let selected_style = self.palette.header_title.add_modifier(Modifier::BOLD);
        let selected_index = state.selected_index;

        match &state.stage {
            ModelSelectionStage::Provider => {
                let providers = ModelProvider::all();
                for offset in 0..providers.len() {
                    let index = (selected_index + offset) % providers.len();
                    let provider = providers[index];
                    let is_selected = offset == 0;
                    let line_style = if is_selected {
                        selected_style
                    } else {
                        self.palette.input_text
                    };
                    let mut meta = Vec::new();
                    if provider == self.runtime_lanes.synthesizer_provider() {
                        meta.push("current");
                    }
                    if !self.provider_availability_for(provider).enabled {
                        meta.push("disabled");
                    }
                    let mut spans = vec![
                        Span::styled(
                            if is_selected { "> " } else { "  " }.to_string(),
                            line_style,
                        ),
                        Span::styled(provider.name().to_string(), line_style),
                    ];
                    if !meta.is_empty() {
                        spans.push(Span::styled(
                            format!(" · {}", meta.join(" · ")),
                            self.palette.input_hint,
                        ));
                    }
                    lines.push(Line::from(spans));
                }
            }
            ModelSelectionStage::Model { provider } => {
                let model_ids = provider.selectable_model_ids();
                let current_model = self.runtime_lanes.synthesizer_model_id();
                for offset in 0..model_ids.len() {
                    let index = (selected_index + offset) % model_ids.len();
                    let model_id = model_ids[index];
                    let is_selected = offset == 0;
                    let line_style = if is_selected {
                        selected_style
                    } else {
                        self.palette.input_text
                    };
                    let mut spans = vec![
                        Span::styled(
                            if is_selected { "> " } else { "  " }.to_string(),
                            line_style,
                        ),
                        Span::styled(model_id.to_string(), line_style),
                    ];
                    if self.runtime_lanes.synthesizer_provider() == *provider
                        && provider.selectable_model_matches_runtime_model(model_id, current_model)
                    {
                        spans.push(Span::styled(
                            " · current".to_string(),
                            self.palette.input_hint,
                        ));
                    }
                    lines.push(Line::from(spans));
                }
            }
            ModelSelectionStage::ThinkingMode { provider, model } => {
                let thinking_modes = provider.thinking_modes(model);
                let current_model = self.runtime_lanes.synthesizer_model_id();
                let current_thinking_mode = self.runtime_lanes.synthesizer_thinking_mode();
                for offset in 0..thinking_modes.len() {
                    let index = (selected_index + offset) % thinking_modes.len();
                    let thinking_mode = thinking_modes[index];
                    let is_selected = offset == 0;
                    let line_style = if is_selected {
                        selected_style
                    } else {
                        self.palette.input_text
                    };
                    let mut spans = vec![
                        Span::styled(
                            if is_selected { "> " } else { "  " }.to_string(),
                            line_style,
                        ),
                        Span::styled(thinking_mode.label.to_string(), line_style),
                    ];
                    if self.runtime_lanes.synthesizer_provider() == *provider
                        && provider.thinking_mode_index_for_runtime_model(
                            model,
                            current_model,
                            current_thinking_mode,
                        ) == Some(index)
                    {
                        spans.push(Span::styled(
                            " · current".to_string(),
                            self.palette.input_hint,
                        ));
                    }
                    lines.push(Line::from(spans));
                }
            }
        }

        Some(lines)
    }

    fn next_work_id(&mut self) -> u64 {
        let work_id = self.next_work_id;
        self.next_work_id = self.next_work_id.wrapping_add(1);
        work_id
    }

    fn set_active_work(&mut self, kind: InFlightWorkKind, work_id: u64, handle: JoinHandle<()>) {
        if let Some(work) = self.active_work.take() {
            work.handle.abort();
        }
        self.active_work = Some(InFlightWork {
            id: work_id,
            kind,
            handle,
        });
    }

    fn clear_active_work_if_current(&mut self, work_id: Option<u64>) {
        let Some(work_id) = work_id else {
            return;
        };
        if let Some(active) = &self.active_work
            && active.id == work_id
        {
            self.active_work = None;
        }
    }

    fn is_current_work_id(&self, work_id: Option<u64>) -> bool {
        match work_id {
            Some(work_id) => self
                .active_work
                .as_ref()
                .is_some_and(|active| active.id == work_id),
            None => true,
        }
    }

    fn clear_in_flight_turn_state(&mut self) {
        self.active_turn_timing = None;
        self.pending_turn_total_timing = None;
        self.pending_reveal = None;
        self.pending_turn_result_fallback = None;
        self.fallback_assistant_row = None;
        self.last_event = None;
        self.emitted_in_flight = false;
        self.remove_in_flight_row();
        self.search_progress_row = None;
        self.planner_progress_row = None;
        self.gathering_harness_row = None;
        self.last_hunting_sample = None;
        self.last_hunting_history_sample = None;
        self.last_hunting_history_at = None;
    }

    fn abort_active_work(&mut self) -> bool {
        let Some(work) = self.active_work.take() else {
            return false;
        };

        work.handle.abort();
        self.busy = false;
        self.busy_phase = BusyPhase::Idle;
        let kind = match work.kind {
            InFlightWorkKind::Prompt => "prompt",
            InFlightWorkKind::RuntimeUpdate => "runtime update",
        };
        self.runtime_update_started_at = None;
        self.clear_in_flight_turn_state();
        self.pending_transcript_sync = false;
        self.push_event(
            "Work cancelled",
            format!("Current {kind} in-flight work cancelled."),
        );
        true
    }

    fn clear_queued_prompts(&mut self) -> bool {
        if self.queued_prompts.is_empty() {
            return false;
        }
        self.queued_prompts.clear();
        true
    }

    fn slash_command_suggestions(&self) -> Vec<SlashSuggestion> {
        if self.is_masked_input()
            || matches!(self.input_mode, InputMode::ModelSelection(_))
            || self.has_composer_parts()
            || self.input.contains('\n')
            || !self.input.starts_with('/')
            || self.slash_popup_dismissed
        {
            return Vec::new();
        }
        let query = self.input.to_ascii_lowercase();
        if query == "/model" || query.starts_with("/model ") {
            return Vec::new();
        }
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
        if self.has_composer_parts() {
            return;
        }
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
        if self.has_composer_parts() {
            return;
        }
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

    fn has_composer_parts(&self) -> bool {
        !self.composer_parts.is_empty()
    }

    fn composer_prompt_text(&self) -> String {
        let mut prompt = String::new();
        for (index, part) in self.composer_parts.iter().enumerate() {
            match part {
                ComposerPart::Text(text) => {
                    let text = if matches!(
                        self.composer_parts.get(index + 1),
                        Some(ComposerPart::Paste { .. })
                    ) {
                        collapse_text_part_trailing_blank_lines_for_paste(text)
                    } else {
                        text.clone()
                    };
                    prompt.push_str(&text);
                }
                ComposerPart::Paste { text, .. } => prompt.push_str(text),
            }
        }
        prompt.push_str(&self.input);
        prompt
    }

    fn insert_text_at_cursor(&mut self, text: &str) {
        let byte_pos = self
            .input
            .char_indices()
            .nth(self.cursor_pos)
            .map(|(i, _)| i)
            .unwrap_or(self.input.len());
        self.input.insert_str(byte_pos, text);
        self.cursor_pos += text.chars().count();
        self.history_cursor = None;
        self.reset_slash_suggestion_selection();
    }

    fn push_input_as_text_part(&mut self) {
        if self.input.is_empty() {
            return;
        }
        let input = std::mem::take(&mut self.input);
        self.composer_parts.push(ComposerPart::Text(input));
        self.cursor_pos = 0;
    }

    fn pop_composer_part(&mut self) {
        let Some(part) = self.composer_parts.pop() else {
            return;
        };
        match part {
            ComposerPart::Text(text) => {
                self.input = text;
                self.cursor_pos = self.input.chars().count();
            }
            ComposerPart::Paste { .. } => {
                self.input.clear();
                self.cursor_pos = 0;
            }
        }
        self.history_cursor = None;
        self.reset_slash_suggestion_selection();
    }

    fn composer_part_summary(part: &ComposerPart) -> Option<String> {
        match part {
            ComposerPart::Text(_) => None,
            ComposerPart::Paste { lines, preview, .. } => {
                let mut summary = format!("[{lines} lines pasted]");
                if !preview.is_empty() {
                    summary.push(' ');
                    summary.push_str(preview);
                }
                Some(summary)
            }
        }
    }

    fn composer_prefix_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        for (index, part) in self.composer_parts.iter().enumerate() {
            match part {
                ComposerPart::Text(text) => {
                    let followed_by_paste = matches!(
                        self.composer_parts.get(index + 1),
                        Some(ComposerPart::Paste { .. })
                    );
                    let text = if followed_by_paste {
                        collapse_text_part_trailing_blank_lines_for_paste(text)
                    } else {
                        text.clone()
                    };
                    let mut text_lines = text.split('\n').collect::<Vec<_>>();
                    if followed_by_paste && text.ends_with('\n') && !text_lines.is_empty() {
                        text_lines.pop();
                    }
                    for line in text_lines {
                        lines.push(Line::from(Span::styled(
                            line.to_string(),
                            self.palette.input_text,
                        )));
                    }
                }
                ComposerPart::Paste {
                    lines: line_count,
                    preview,
                    ..
                } => {
                    let mut spans = vec![Span::styled(
                        format!("[{line_count} lines pasted]"),
                        self.palette.input_hint.add_modifier(Modifier::BOLD),
                    )];
                    if !preview.is_empty() {
                        spans.push(Span::raw(" "));
                        spans.push(Span::styled(preview.clone(), self.palette.input_text));
                    }
                    lines.push(Line::from(spans));
                }
            }
        }
        lines
    }

    fn composer_prefix_height(&self, width: usize) -> usize {
        self.composer_parts
            .iter()
            .enumerate()
            .map(|(index, part)| match part {
                ComposerPart::Text(text) => {
                    let followed_by_paste = matches!(
                        self.composer_parts.get(index + 1),
                        Some(ComposerPart::Paste { .. })
                    );
                    let text = if followed_by_paste {
                        collapse_text_part_trailing_blank_lines_for_paste(text)
                    } else {
                        text.clone()
                    };
                    let mut text_lines = text.split('\n').collect::<Vec<_>>();
                    if followed_by_paste && text.ends_with('\n') && !text_lines.is_empty() {
                        text_lines.pop();
                    }
                    text_lines
                        .into_iter()
                        .map(|line| wrapped_line_count(line, width).max(1))
                        .sum::<usize>()
                }
                ComposerPart::Paste { .. } => Self::composer_part_summary(part)
                    .map(|summary| wrapped_line_count(&summary, width).max(1))
                    .unwrap_or(0),
            })
            .sum()
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
        let raw = self.composer_prompt_text().trim().to_string();
        if raw.is_empty() {
            return;
        }
        let raw_display = inline_multiline_text(&raw);
        self.prompt_history.push(raw.clone());
        self.history_cursor = None;
        self.history_draft.clear();
        self.composer_parts.clear();
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

        if raw.starts_with("/resume") {
            self.handle_resume_command(&raw);
            return;
        }

        self.rows.push(TranscriptRow::new(
            TranscriptRowKind::User,
            "User",
            raw.clone(),
        ));

        // Normal prompt submission.
        let was_busy = self.busy || self.pending_reveal.is_some();
        if was_busy && self.session.request_turn_steer(raw.clone()).is_ok() {
            self.rows.push(TranscriptRow::new(
                TranscriptRowKind::Event,
                "• Requested same-turn steering",
                format!(
                    "`{}` will apply at the next safe checkpoint.",
                    trim_for_display(&raw_display, 96)
                ),
            ));
            return;
        }
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

    fn take_pending_resume_command(&mut self) -> Option<PendingResumeCommand> {
        self.pending_resume_command.take()
    }

    #[cfg(test)]
    fn take_pending_runtime_update(&mut self) -> Option<PendingRuntimeUpdate> {
        self.pending_runtime_update.take()
    }

    fn dispatch_pending_runtime_update(&mut self) -> Option<PendingRuntimeUpdate> {
        self.dispatch_pending_runtime_update_at(Instant::now())
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
            self.start_model_provider_selection();
            return;
        }
        self.push_error(
            "Model command invalid",
            "Use `/model`, then press Enter to choose from the selector.",
        );
    }

    fn queue_model_selection(
        &mut self,
        provider: ModelProvider,
        model_id: &str,
        thinking_mode: Option<&str>,
    ) -> bool {
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
            return false;
        }
        let normalized_model = provider.normalize_model_alias(model_id);
        let normalized_thinking_mode = thinking_mode
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
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
            return false;
        }
        if let Some(requested_thinking_mode) = normalized_thinking_mode.as_deref()
            && !provider
                .thinking_modes(&normalized_model)
                .iter()
                .any(|mode| mode.thinking_mode == Some(requested_thinking_mode))
        {
            let accepted = provider
                .thinking_modes(&normalized_model)
                .iter()
                .filter_map(|mode| mode.thinking_mode)
                .collect::<Vec<_>>()
                .join(", ");
            self.push_error(
                "Model command invalid",
                format!(
                    "Provider `{}` does not recognize thinking mode `{}` for model `{}`. Accepted: {}.",
                    provider.name(),
                    requested_thinking_mode,
                    normalized_model,
                    accepted
                ),
            );
            return false;
        }
        if self.pending_runtime_update.is_some()
            || matches!(self.busy_phase, BusyPhase::Reconfiguring)
        {
            self.push_error(
                "Model selection busy",
                "Another runtime lane activation is already in progress.",
            );
            return false;
        }

        let runtime_lanes = RuntimeLaneConfig::new(
            normalized_model.clone(),
            self.runtime_lanes.gatherer_model_id().map(str::to_string),
        )
        .with_synthesizer_provider(provider)
        .with_synthesizer_thinking_mode(normalized_thinking_mode.clone())
        .with_gatherer_provider(self.runtime_lanes.gatherer_provider())
        .with_context1_harness_ready(self.runtime_lanes.context1_harness_ready());
        self.pending_runtime_update = Some(PendingRuntimeUpdate {
            persisted_preferences: RuntimeLanePreferences::from_runtime_lanes(&runtime_lanes),
            runtime_lanes,
            summary: format!(
                "Runtime lanes now target `{}`.",
                provider.qualified_model_label_with_thinking(
                    &normalized_model,
                    normalized_thinking_mode.as_deref(),
                )
            ),
        });
        true
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

    fn show_resumable_conversations(&mut self, conversations: &[ResumableConversation]) {
        if conversations.is_empty() {
            self.push_event(
                "Resume",
                "No persisted conversations are available to restore.",
            );
            return;
        }

        let mut lines = conversations
            .iter()
            .take(8)
            .map(|conversation| {
                format!(
                    "{} · {} turn{} · {}",
                    conversation.task_id.as_str(),
                    conversation.turn_count,
                    if conversation.turn_count == 1 {
                        ""
                    } else {
                        "s"
                    },
                    conversation.preview
                )
            })
            .collect::<Vec<_>>();
        lines.push("Type `/resume <task-id>` to restore one.".to_string());
        self.push_event("Resumable conversations", lines.join("\n"));
    }

    fn restore_resumed_session(
        &mut self,
        session: ConversationSession,
        transcript: &ConversationTranscript,
    ) {
        self.session = session;
        self.current_task_id = self.session.task_id().as_str().to_string();
        self.busy = false;
        self.busy_phase = BusyPhase::Idle;
        self.pending_transcript_sync = false;
        self.seen_transcript_record_ids.clear();
        self.pending_turn_total_timing = None;
        self.pending_turn_result_fallback = None;
        self.queued_prompts.clear();
        self.clear_in_flight_turn_state();
        self.rows.retain(|row| {
            row.header == "• Interactive mode ready" || row.header == "• Web UI ready"
        });
        self.flushed_row_count = self.flushed_row_count.min(self.rows.len());
        self.load_transcript(transcript);
        self.push_event(
            "Conversation resumed",
            format!("Restored `{}`.", self.current_task_id),
        );
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
                    if let Some(row_index) = self.matching_pending_user_row(&entry.content) {
                        if let Some(row) = self.rows.get_mut(row_index) {
                            row.transcript_record_id = Some(record_id.clone());
                        }
                        self.seen_transcript_record_ids.insert(record_id);
                        continue;
                    }
                    self.rows.push(
                        TranscriptRow::new(TranscriptRowKind::User, "User", entry.content.clone())
                            .with_transcript_record_id(record_id.clone()),
                    );
                }
                ConversationTranscriptSpeaker::Assistant => {
                    let wrapped = soft_wrap_prose(&entry.content, MAX_PROSE_WIDTH);
                    let render = entry.render.clone();
                    self.pending_turn_result_fallback = None;
                    if let Some(row_index) = self.matching_fallback_assistant_row(&wrapped) {
                        if let Some(row) = self.rows.get_mut(row_index) {
                            row.content = wrapped;
                            if let Some(render) = render.clone() {
                                row.render = Some(render);
                            }
                            if row.timing.is_none()
                                && let Some(timing) = self.pending_turn_total_timing.take()
                            {
                                row.timing = Some(timing);
                            }
                        }
                        if let Some(pending) = self.pending_reveal.as_mut()
                            && pending.row_index == row_index
                        {
                            pending.render = render;
                        }
                        self.fallback_assistant_row = None;
                        self.seen_transcript_record_ids.insert(record_id);
                        continue;
                    }
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
                ConversationTranscriptSpeaker::System => {
                    self.rows.push(TranscriptRow::new(
                        TranscriptRowKind::Event,
                        "Policy",
                        entry.content.clone(),
                    ));
                }
            }

            self.seen_transcript_record_ids.insert(record_id);
        }
    }

    fn handle_resume_command(&mut self, raw: &str) {
        if self.busy || !self.queued_prompts.is_empty() {
            self.push_error(
                "Resume unavailable",
                "Wait for the current turn queue to drain before restoring another conversation.",
            );
            return;
        }

        let mut parts = raw.split_whitespace();
        let _ = parts.next();
        match (parts.next(), parts.next()) {
            (None, None) => {
                self.pending_resume_command = Some(PendingResumeCommand::List);
            }
            (Some(task_id), None) if TaskTraceId::new(task_id).is_ok() => {
                self.pending_resume_command = Some(PendingResumeCommand::Restore {
                    task_id: task_id.to_string(),
                });
            }
            _ => self.push_error(
                "Resume command",
                "Use `/resume` to list persisted conversations or `/resume <task-id>` to restore one.",
            ),
        }
    }

    fn dispatch_next_prompt_at(&mut self, started_at: Instant) -> Option<QueuedPrompt> {
        if self.busy {
            return None;
        }

        let prompt = self.queued_prompts.pop_front()?;
        self.busy = true;
        self.busy_phase = BusyPhase::Thinking;
        self.fallback_assistant_row = None;
        self.active_turn_timing = Some(ActiveTurnTiming::new(started_at));
        Some(prompt)
    }

    fn dispatch_pending_runtime_update_at(
        &mut self,
        started_at: Instant,
    ) -> Option<PendingRuntimeUpdate> {
        if self.busy {
            return None;
        }

        let update = self.pending_runtime_update.take()?;
        self.busy = true;
        self.busy_phase = BusyPhase::Reconfiguring;
        self.runtime_update_started_at = Some(started_at);
        self.push_event("Activating runtime lanes", update.summary.clone());
        Some(update)
    }

    fn matching_pending_user_row(&self, content: &str) -> Option<usize> {
        self.rows.iter().position(|row| {
            row.kind == TranscriptRowKind::User
                && row.content == content
                && row.transcript_record_id.is_none()
        })
    }

    fn should_show_event(&self, event: &TurnEvent, pace: Pace, is_first_step: bool) -> bool {
        if is_first_step {
            return true;
        }
        let promoted_verbose = if event.allows_pace_promotion() {
            self.verbose.saturating_add(match pace {
                Pace::Slow => 2,
                Pace::Normal => 1,
                Pace::Fast => 0,
            })
        } else {
            self.verbose
        };
        event.is_visible_at_verbosity(promoted_verbose)
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
        if let Some(fallback_index) = self.fallback_assistant_row {
            if fallback_index == index {
                self.fallback_assistant_row = None;
            } else if fallback_index > index {
                self.fallback_assistant_row = Some(fallback_index - 1);
            }
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
        let mut stale_streams = Vec::new();
        for (stream_key, stream_index) in &mut self.tool_output_rows {
            if *stream_index == index {
                stale_streams.push(stream_key.clone());
            } else if *stream_index > index {
                *stream_index -= 1;
            }
        }
        for stream_key in stale_streams {
            self.tool_output_rows.remove(&stream_key);
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
        for stream_index in self.tool_output_rows.values_mut() {
            if *stream_index >= insert_at {
                *stream_index += 1;
            }
        }

        if let Some(pending) = self.pending_reveal.as_mut()
            && pending.row_index >= insert_at
        {
            pending.row_index += 1;
        }
        if let Some(index) = self.fallback_assistant_row.as_mut()
            && *index >= insert_at
        {
            *index += 1;
        }
    }

    fn matching_fallback_assistant_row(&self, content: &str) -> Option<usize> {
        let row_index = self.fallback_assistant_row?;
        if row_index >= self.rows.len() {
            return None;
        }

        if let Some(pending) = self.pending_reveal.as_ref()
            && pending.row_index == row_index
            && pending.full_text == content
        {
            return Some(row_index);
        }

        (self.rows[row_index].kind == TranscriptRowKind::Assistant
            && self.rows[row_index].content == content)
            .then_some(row_index)
    }

    fn queue_turn_result_fallback(&mut self, response: &str) {
        let render = RenderDocument::canonicalize_assistant_response(response);
        let wrapped = soft_wrap_prose(&render.to_plain_text(), MAX_PROSE_WIDTH);
        if wrapped.trim().is_empty() {
            self.pending_turn_result_fallback = None;
            return;
        }
        self.pending_turn_result_fallback = Some(PendingTurnResultFallback {
            content: wrapped,
            render: Some(render),
        });
    }

    fn activate_pending_turn_result_fallback(&mut self) {
        let Some(PendingTurnResultFallback { content, render }) =
            self.pending_turn_result_fallback.take()
        else {
            return;
        };

        let row_index = self.rows.len();
        let mut row = TranscriptRow::new(TranscriptRowKind::Assistant, "Paddles", "");
        if let Some(timing) = self.pending_turn_total_timing.take() {
            row = row.timed(timing);
        }
        self.rows.push(row);
        self.pending_reveal = Some(PendingReveal::new(row_index, content, render));
        self.fallback_assistant_row = Some(row_index);
        self.busy = true;
        self.busy_phase = BusyPhase::Rendering;
    }

    fn append_tool_output_row(
        &mut self,
        call_id: &str,
        stream: &str,
        output: &str,
        mut row: TranscriptRow,
    ) {
        let key = (call_id.to_string(), stream.to_string());
        row.content = output.to_string();
        if let Some(index) = self.tool_output_rows.get(&key).copied()
            && index < self.rows.len()
        {
            row.content = format!("{}{}", self.rows[index].content, output);
            self.rows[index] = row;
            return;
        }

        self.tool_output_rows.insert(key, self.rows.len());
        self.rows.push(row);
    }

    fn forget_tool_output_rows_for_call(&mut self, call_id: &str) {
        self.tool_output_rows
            .retain(|(active_call_id, _), _| active_call_id != call_id);
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
            UiMessage::TurnEvent {
                event,
                occurred_at,
                work_id,
            } => {
                if !self.is_current_work_id(work_id) {
                    return;
                }
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
                let prior_event = self.last_event.as_ref().map(|(event, _)| event);
                let suppress_matching_tool_call = matches!(
                    &event,
                    TurnEvent::ToolCalled {
                        tool_name,
                        invocation,
                        ..
                    } if planner_step_matches_tool_call(prior_event, tool_name, invocation)
                );

                self.last_event = Some((event.clone(), occurred_at));
                self.emitted_in_flight = false;

                if !suppress_matching_tool_call
                    && self.should_show_event(&event, pace, is_first_step)
                {
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
                    } else if let TurnEvent::ToolOutput {
                        call_id,
                        stream,
                        output,
                        ..
                    } = &event
                    {
                        self.search_progress_row = None;
                        self.gathering_harness_row = None;
                        self.last_hunting_sample = None;
                        self.last_hunting_history_sample = None;
                        self.last_hunting_history_at = None;
                        self.append_tool_output_row(call_id, stream, output, row);
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
                if let TurnEvent::ToolFinished { call_id, .. } = &event {
                    self.forget_tool_output_rows_for_call(call_id);
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
                work_id,
            } => {
                if !self.is_current_work_id(work_id) {
                    return;
                }
                self.clear_active_work_if_current(work_id);
                self.remove_in_flight_row();
                self.search_progress_row = None;
                self.gathering_harness_row = None;
                self.planner_progress_row = None;
                self.tool_output_rows.clear();
                self.last_hunting_sample = None;
                self.last_hunting_history_sample = None;
                self.last_hunting_history_at = None;
                self.last_event = None;
                self.emitted_in_flight = false;
                match result {
                    Ok(response) => {
                        let timing = self
                            .active_turn_timing
                            .take()
                            .map(|timing| timing.finish(occurred_at));
                        self.queue_turn_result_fallback(&response);
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
            UiMessage::RuntimeUpdateFinished {
                result,
                occurred_at,
                work_id,
            } => {
                if !self.is_current_work_id(work_id) {
                    return;
                }
                self.clear_active_work_if_current(work_id);
                let started_at = self.runtime_update_started_at.take();
                self.pending_turn_total_timing = None;
                match result {
                    Ok(completion) => {
                        self.set_runtime_catalog(
                            completion.runtime_lanes,
                            completion.provider_availability,
                        );
                        match completion.preference_save_error {
                            None => self.push_event(
                                "Model selection updated",
                                format!(
                                    "{}\nSaved runtime lane preferences to `{}`.",
                                    completion.summary,
                                    completion.preference_path.display()
                                ),
                            ),
                            Some(err) => self.push_error(
                                "Runtime preference save failed",
                                format!(
                                    "{}\nThe lane switch is active, but `{}` could not be updated: {}",
                                    completion.summary,
                                    completion.preference_path.display(),
                                    err
                                ),
                            ),
                        }
                    }
                    Err(error) => {
                        let row = TranscriptRow::new(
                            TranscriptRowKind::Error,
                            "• Model selection failed",
                            format!("Could not activate requested runtime lanes: {error}"),
                        );
                        let row = if let Some(started_at) = started_at {
                            row.timed(TranscriptTiming {
                                elapsed: occurred_at.duration_since(started_at),
                                delta: None,
                                kind: TranscriptTimingKind::TurnTotal,
                                pace: Pace::Normal,
                            })
                        } else {
                            row
                        };
                        self.rows.push(row);
                    }
                }
                self.busy = false;
                self.busy_phase = BusyPhase::Idle;
            }
        }
        let _ = self.step_timing.flush(&self.step_timing_path);
    }

    fn tick(&mut self) {
        self.spinner_index = (self.spinner_index + 1) % SPINNER_FRAMES.len();

        if self.busy && self.busy_phase == BusyPhase::Rendering && self.pending_reveal.is_none() {
            self.activate_pending_turn_result_fallback();
        }

        // After IN_FLIGHT_SILENCE_THRESHOLD of silence during a busy turn,
        // insert a muted "working" row so the transcript doesn't look stalled.
        if self.busy
            && !self.emitted_in_flight
            && let Some((ref event, last_at)) = self.last_event
            && self.should_emit_in_flight_row(event)
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

    fn should_emit_in_flight_row(&self, event: &TurnEvent) -> bool {
        match event {
            TurnEvent::ToolOutput {
                call_id, stream, ..
            } => !self
                .tool_output_rows
                .contains_key(&(call_id.clone(), stream.clone())),
            TurnEvent::ToolFinished { .. } => false,
            TurnEvent::HarnessState { snapshot } => {
                snapshot.chamber == crate::domain::model::HarnessChamber::Gathering
            }
            _ => true,
        }
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let input_height = self.input_area_height(area.width);
        let activity_height = u16::from(self.busy && !self.is_masked_input());
        let status_height = u16::from(self.model_selection_state().is_none());
        let fixed_bottom = input_height + activity_height + status_height;
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
                Constraint::Length(status_height),
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
            frame.render_widget(self.render_command_popup(popup_area), popup_area);
        }
        frame.set_cursor_position(self.cursor_position(layout[2]));
        if status_height > 0 {
            frame.render_widget(self.render_status_bar(), layout[3]);
        }
    }

    fn render_status_bar(&self) -> Paragraph<'static> {
        let active_thread = self.session.active_thread().thread_ref.stable_id();
        let last_event = self.last_event.as_ref().map(|(event, _)| event);
        let status = match self.busy_phase {
            BusyPhase::Idle if self.queued_prompts.is_empty() => "idle".to_string(),
            BusyPhase::Idle => format!("idle · {} queued", self.queued_prompts.len()),
            BusyPhase::Thinking | BusyPhase::Reconfiguring | BusyPhase::Rendering => {
                busy_label(self.busy_phase, last_event)
            }
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
        let label = busy_label(
            self.busy_phase,
            self.last_event.as_ref().map(|(event, _)| event),
        );
        let elapsed = self
            .active_turn_timing
            .as_ref()
            .map(|t| t.started_at)
            .or(self.runtime_update_started_at)
            .map(|started_at| Instant::now().duration_since(started_at).as_secs())
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
            lines.extend(render_row_lines_for_width(row, &self.palette, inner_width));
            if index + 1 < visible_rows.len() {
                lines.push(Line::default());
            }
        }

        let scroll_y = rendered_lines_height(&lines, inner_width).saturating_sub(inner_height);

        Paragraph::new(Text::from(lines))
            .wrap(Wrap { trim: false })
            .scroll((scroll_y as u16, 0))
    }

    fn render_input(&self) -> Paragraph<'static> {
        let lines = self.input_render_lines();
        let title = match self.model_selection_state() {
            Some(ModelSelectionState {
                stage: ModelSelectionStage::Provider,
                ..
            }) => " Select provider ".to_string(),
            Some(ModelSelectionState {
                stage: ModelSelectionStage::Model { provider },
                ..
            }) => format!(" Select model · {} ", provider.name()),
            Some(ModelSelectionState {
                stage: ModelSelectionStage::ThinkingMode { model, .. },
                ..
            }) => format!(" Select thinking mode · {} ", model),
            None => " Prompt ".to_string(),
        };
        Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(self.palette.border)
                    .style(Style::default().bg(self.palette.input_bg)),
            )
            .wrap(Wrap { trim: false })
    }

    fn input_render_lines(&self) -> Vec<Line<'static>> {
        if let Some(lines) = self.model_selection_lines() {
            return lines;
        }

        if self.has_composer_parts() {
            let mut lines = self.composer_prefix_lines();
            if !self.input.is_empty() {
                lines.extend(self.input.split('\n').map(|line| {
                    Line::from(Span::styled(line.to_string(), self.palette.input_text))
                }));
            }
            return lines;
        }

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

    fn visible_slash_suggestion_window(&self, visible_rows: usize) -> (usize, usize) {
        let suggestions = self.slash_command_suggestions();
        if suggestions.is_empty() || visible_rows == 0 {
            return (0, 0);
        }
        let selected = self
            .slash_suggestion_index
            .min(suggestions.len().saturating_sub(1));
        let end = (selected + 1).max(visible_rows).min(suggestions.len());
        let start = end.saturating_sub(visible_rows);
        (start, end)
    }

    fn truncate_popup_text(text: &str, max_chars: usize) -> String {
        if max_chars == 0 {
            return String::new();
        }
        let char_count = text.chars().count();
        if char_count <= max_chars {
            return text.to_string();
        }
        if max_chars == 1 {
            return "…".to_string();
        }
        let mut truncated = text.chars().take(max_chars - 1).collect::<String>();
        truncated.push('…');
        truncated
    }

    fn render_command_popup(&self, area: Rect) -> Paragraph<'static> {
        let suggestions = self.slash_command_suggestions();
        let visible_rows = area.height.saturating_sub(2) as usize;
        let (start, end) = self.visible_slash_suggestion_window(visible_rows);
        let inner_width = area.width.saturating_sub(2) as usize;
        let selected = self
            .slash_suggestion_index
            .min(suggestions.len().saturating_sub(1));
        let lines = suggestions[start..end]
            .iter()
            .enumerate()
            .map(|(offset, command)| {
                let index = start + offset;
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
                let usage = Self::truncate_popup_text(&command.usage, inner_width);
                let usage_width = usage.chars().count();
                let desc_budget = inner_width.saturating_sub(usage_width + 2);
                let description = Self::truncate_popup_text(&command.description, desc_budget);
                let mut spans = vec![Span::styled(usage, usage_style)];
                if !description.is_empty() {
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(description, desc_style));
                }
                Line::from(spans)
            })
            .collect::<Vec<_>>();
        let title = if start > 0 || end < suggestions.len() {
            " Commands (scroll) "
        } else {
            " Commands "
        };

        Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title(title)
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
        let max_height_above = input_area.y.saturating_sub(frame_area.y);
        if max_height_above < 3 {
            return None;
        }
        let content_width = suggestions
            .iter()
            .map(|command| command.usage.len() + command.description.len() + 2)
            .max()
            .unwrap_or(24)
            .min(usize::from(frame_area.width.saturating_sub(4)));
        let width = ((content_width as u16).max(28) + 2).min(frame_area.width.saturating_sub(2));
        let height = (suggestions.len() as u16 + 2).min(max_height_above);
        let x = input_area.x.min(frame_area.right().saturating_sub(width));
        let y = input_area.y - height;
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
        if self.model_selection_state().is_some() {
            let x = area.x.saturating_add(3).min(area.right().saturating_sub(1));
            let y = area
                .y
                .saturating_add(1)
                .min(area.bottom().saturating_sub(1));
            return (x, y);
        }

        let inner_width = area.width.saturating_sub(2).max(1) as usize;
        let mut row = self.composer_prefix_height(inner_width) as u16;
        let mut col = 0usize;

        for ch in self.input.chars().take(self.cursor_pos) {
            if ch == '\n' {
                row += 1;
                col = 0;
                continue;
            }

            if col == inner_width {
                row += 1;
                col = 0;
            }

            col += 1;
        }

        if col == inner_width {
            // Keep the cursor on the final body cell until another character forces a wrap.
            col = inner_width.saturating_sub(1);
        }

        let x = area.x.saturating_add(1 + col as u16);
        let y = area.y.saturating_add(1 + row);
        (x.min(area.right().saturating_sub(1)), y)
    }

    fn input_area_height(&self, width: u16) -> u16 {
        let inner_width = width.saturating_sub(2).max(1) as usize;
        if self.model_selection_state().is_some() {
            let content_lines =
                self.composer_prefix_height(inner_width) + self.model_selection_option_count();
            return (content_lines as u16) + 2;
        }

        let input_lines = if self.input.is_empty() {
            1
        } else {
            self.input
                .split('\n')
                .map(|line| wrapped_line_count(line, inner_width).max(1))
                .sum()
        };
        let content_lines = self.composer_prefix_height(inner_width)
            + input_lines
            + self.model_selection_option_count();
        (content_lines as u16) + 2 // content + top/bottom border
    }

    fn live_tail_height(&self, width: usize, max_height: usize) -> u16 {
        if max_height == 0 {
            return 0;
        }

        let visible_rows = self.visible_live_rows(width, max_height);
        rendered_rows_height(&visible_rows, &self.palette, width).min(max_height) as u16
    }

    fn visible_live_rows(&self, width: usize, height: usize) -> Vec<TranscriptRow> {
        let mut visible = Vec::new();
        let mut used = 0;
        let live_rows = &self.rows[self.flushed_row_count..];

        for row in live_rows.iter().rev() {
            let row_height = row_rendered_height(row, &self.palette, width);
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
        let mut lines = render_row_lines_for_width(&row, &palette, width);
        lines.push(Line::default());
        let height = rendered_lines_height(&lines, width) as u16;
        terminal.insert_before(height, |buffer| {
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

fn rendered_rows_height(rows: &[TranscriptRow], palette: &Palette, width: usize) -> usize {
    rows.iter()
        .enumerate()
        .map(|(index, row)| row_rendered_height(row, palette, width) + usize::from(index > 0))
        .sum()
}

fn render_row_lines_for_width(
    row: &TranscriptRow,
    palette: &Palette,
    width: usize,
) -> Vec<Line<'static>> {
    if row.kind == TranscriptRowKind::User {
        return render_full_width_user_row_lines(row, palette, width);
    }
    render_row_lines(row, palette)
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

    if let Some(tool_name) = row.header.strip_prefix("• Applied ").map(str::trim)
        && is_mutation_tool_name(tool_name)
    {
        let mutation_lines = render_mutation_diff_lines(&row.content, palette);
        lines.extend(mutation_lines);
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
            AssistantTextStyles {
                base: body_style,
                heading: palette.assistant_heading,
                code: palette.code,
                citation: palette.citation,
                list_marker: palette.list_marker,
                ordered_marker: palette.ordered_marker,
            },
        ));
        return lines;
    }

    if row.content.is_empty() {
        return lines;
    }

    let mut in_code_block = false;
    for (index, line) in row.content.lines().enumerate() {
        let prefix = if index == 0 { "  └ " } else { "    " };
        let rendered = match row.kind {
            TranscriptRowKind::Assistant => render_assistant_line(
                prefix,
                line,
                AssistantTextStyles {
                    base: body_style,
                    heading: palette.assistant_heading,
                    code: palette.code,
                    citation: palette.citation,
                    list_marker: palette.list_marker,
                    ordered_marker: palette.ordered_marker,
                },
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

fn render_full_width_user_row_lines(
    row: &TranscriptRow,
    palette: &Palette,
    width: usize,
) -> Vec<Line<'static>> {
    let width = width.max(1);
    let mut header_spans = vec![Span::styled(
        row.header.clone(),
        palette.user_header.add_modifier(Modifier::BOLD),
    )];
    if let Some(timing) = row.timing {
        header_spans.push(Span::styled(
            format!(" · {}", timing.elapsed_label()),
            palette.user_body,
        ));
        if let Some(delta_label) = timing.delta_label() {
            let delta_style = match timing.pace {
                Pace::Fast => palette.pace_fast.bg(palette.input_bg),
                Pace::Normal => palette.user_body,
                Pace::Slow => palette.pace_slow.bg(palette.input_bg),
            };
            header_spans.push(Span::styled(delta_label, delta_style));
        }
    }
    pad_spans_to_width(&mut header_spans, palette.user_body, width);

    let mut lines = vec![Line::from(header_spans)];
    if row.content.is_empty() {
        return lines;
    }

    let content_width = width.saturating_sub(4).max(1);
    for (line_index, line) in row.content.lines().enumerate() {
        if line.is_empty() {
            lines.push(Line::from(Span::styled(
                "\u{00A0}".repeat(width),
                palette.user_body,
            )));
            continue;
        }
        let first_prefix = if line_index == 0 { "  └ " } else { "    " };
        for (segment_index, segment) in wrap_exact_segments(line, content_width)
            .into_iter()
            .enumerate()
        {
            let prefix = if segment_index == 0 {
                first_prefix
            } else {
                "    "
            };
            lines.push(Line::from(Span::styled(
                pad_to_width(&format!("{prefix}{segment}"), width),
                palette.user_body,
            )));
        }
    }

    lines
}

fn row_rendered_height(row: &TranscriptRow, palette: &Palette, width: usize) -> usize {
    let mut lines = render_row_lines_for_width(row, palette, width);
    lines.push(Line::default());
    rendered_lines_height(&lines, width)
}

fn rendered_lines_height(lines: &[Line<'_>], width: usize) -> usize {
    lines
        .iter()
        .map(|line| wrapped_line_count(&flatten_rendered_line(line), width))
        .sum()
}

fn flatten_rendered_line(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>()
}

fn pad_spans_to_width(spans: &mut Vec<Span<'static>>, fill_style: Style, width: usize) {
    let current_width = spans
        .iter()
        .map(|span| span.content.chars().count())
        .sum::<usize>();
    if current_width < width {
        spans.push(Span::styled(" ".repeat(width - current_width), fill_style));
    }
}

fn pad_to_width(text: &str, width: usize) -> String {
    let text_width = text.chars().count();
    if text_width >= width {
        return text.to_string();
    }
    let mut padded = String::with_capacity(text.len() + (width - text_width));
    padded.push_str(text);
    padded.push_str(&" ".repeat(width - text_width));
    padded
}

fn wrap_exact_segments(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    if text.is_empty() {
        return vec![String::new()];
    }

    let mut segments = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;
    for ch in text.chars() {
        if current_width == width {
            segments.push(current);
            current = String::new();
            current_width = 0;
        }
        current.push(ch);
        current_width += 1;
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments
}

#[derive(Clone, Copy)]
enum AssistantRenderLineKind {
    Heading,
    Paragraph,
    Code,
    Citations,
}

#[derive(Clone, Copy)]
enum AssistantListMarkerKind {
    Bullet,
    Ordered,
}

struct AssistantRenderLine {
    kind: AssistantRenderLineKind,
    text: String,
}

#[derive(Clone, Copy)]
struct AssistantTextStyles {
    base: Style,
    heading: Style,
    code: Style,
    citation: Style,
    list_marker: Style,
    ordered_marker: Style,
}

fn assistant_render_line_specs(render: &RenderDocument) -> Vec<AssistantRenderLine> {
    let mut lines = Vec::new();
    for (index, block) in render.blocks.iter().enumerate() {
        if index > 0 && assistant_block_separator(&render.blocks[index - 1], block) {
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

fn assistant_block_separator(previous: &RenderBlock, next: &RenderBlock) -> bool {
    !uses_compact_block_separator(previous, next)
}

fn render_assistant_document_lines(
    render: &RenderDocument,
    styles: AssistantTextStyles,
) -> Vec<Line<'static>> {
    assistant_render_line_specs(render)
        .into_iter()
        .enumerate()
        .map(|(index, spec)| {
            let prefix = if index == 0 { "  └ " } else { "    " };
            match spec.kind {
                AssistantRenderLineKind::Heading => Line::from(vec![
                    Span::styled(prefix.to_string(), styles.base),
                    Span::styled(spec.text, styles.heading),
                ]),
                AssistantRenderLineKind::Paragraph => {
                    render_assistant_prose_line(prefix, &spec.text, styles)
                }
                AssistantRenderLineKind::Code => Line::from(vec![
                    Span::styled(prefix.to_string(), styles.base),
                    Span::styled(spec.text, styles.code),
                ]),
                AssistantRenderLineKind::Citations => Line::from(vec![
                    Span::styled(prefix.to_string(), styles.base),
                    Span::styled(spec.text, styles.citation),
                ]),
            }
        })
        .collect()
}

fn render_assistant_line(
    prefix: &str,
    line: &str,
    styles: AssistantTextStyles,
    in_code_block: &mut bool,
) -> Line<'static> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
        *in_code_block = !*in_code_block;
        return Line::from(vec![
            Span::styled(prefix.to_string(), styles.base),
            Span::styled(line.to_string(), styles.code),
        ]);
    }

    if *in_code_block {
        return Line::from(vec![
            Span::styled(prefix.to_string(), styles.base),
            Span::styled(line.to_string(), styles.code),
        ]);
    }

    if trimmed.starts_with("Sources:") {
        return Line::from(vec![
            Span::styled(prefix.to_string(), styles.base),
            Span::styled(line.to_string(), styles.citation),
        ]);
    }

    render_assistant_prose_line(prefix, line, styles)
}

fn render_assistant_prose_line(
    prefix: &str,
    line: &str,
    styles: AssistantTextStyles,
) -> Line<'static> {
    let mut spans = vec![Span::styled(prefix.to_string(), styles.base)];

    if let Some((leading, marker_kind, marker, remainder)) = split_list_marker(line) {
        if !leading.is_empty() {
            spans.push(Span::styled(leading.to_string(), styles.base));
        }
        let marker_style = match marker_kind {
            AssistantListMarkerKind::Bullet => styles.list_marker,
            AssistantListMarkerKind::Ordered => styles.ordered_marker,
        };
        spans.push(Span::styled(marker.to_string(), marker_style));
        append_inline_code_spans(&mut spans, remainder, styles.base, styles.code);
    } else {
        append_inline_code_spans(&mut spans, line, styles.base, styles.code);
    }

    Line::from(spans)
}

fn append_inline_code_spans(
    spans: &mut Vec<Span<'static>>,
    text: &str,
    base_style: Style,
    code_style: Style,
) {
    let mut code_segment = false;
    for segment in text.split('`') {
        let style = if code_segment { code_style } else { base_style };
        spans.push(Span::styled(segment.to_string(), style));
        code_segment = !code_segment;
    }
}

fn split_list_marker(line: &str) -> Option<(&str, AssistantListMarkerKind, &str, &str)> {
    let trimmed = line.trim_start();
    let leading = &line[..line.len() - trimmed.len()];

    if let Some(remainder) = trimmed.strip_prefix("- ") {
        return Some((leading, AssistantListMarkerKind::Bullet, "- ", remainder));
    }

    let marker_end = ordered_marker_end(trimmed)?;
    Some((
        leading,
        AssistantListMarkerKind::Ordered,
        &trimmed[..marker_end],
        &trimmed[marker_end..],
    ))
}

fn ordered_marker_end(line: &str) -> Option<usize> {
    let digit_count = line
        .bytes()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digit_count == 0 {
        return None;
    }

    let marker_end = digit_count + 2;
    (line.as_bytes().get(digit_count) == Some(&b'.')
        && line.as_bytes().get(digit_count + 1) == Some(&b' '))
    .then_some(marker_end)
}

fn format_turn_event_row(event: TurnEvent, verbose: u8) -> TranscriptRow {
    let runtime_items = event.runtime_items();
    if let TurnEvent::WorkspaceEditApplied {
        tool_name, edit, ..
    } = &event
    {
        return TranscriptRow::new(
            TranscriptRowKind::Event,
            format!("• Applied {tool_name}"),
            edit.diff.clone(),
        )
        .with_runtime_items(runtime_items);
    }

    if let TurnEvent::ToolFinished {
        tool_name, summary, ..
    } = &event
        && tool_name != "external_capability"
    {
        let content = mutation_tool_payload(tool_name, summary)
            .unwrap_or_else(|| collapse_event_details(summary, EVENT_DETAIL_LINE_LIMIT));
        return TranscriptRow::new(
            TranscriptRowKind::Event,
            format!("• Completed {tool_name}"),
            content,
        )
        .with_runtime_items(runtime_items);
    }

    transcript_row_from_runtime_event(
        project_runtime_event_for_tui(&event, verbose),
        runtime_items,
    )
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
    matches!(
        tool_name,
        "diff" | "apply_patch" | "write_file" | "replace_in_file"
    )
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
    assistant_heading: Style,
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
    list_marker: Style,
    ordered_marker: Style,
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
            assistant_heading: Style::default()
                .fg(Color::Rgb(18, 74, 140))
                .add_modifier(Modifier::BOLD),
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
            list_marker: Style::default().fg(Color::Rgb(0, 120, 102)),
            ordered_marker: Style::default()
                .fg(Color::Rgb(18, 74, 140))
                .add_modifier(Modifier::BOLD),
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
            assistant_heading: Style::default()
                .fg(Color::Rgb(126, 204, 255))
                .add_modifier(Modifier::BOLD),
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
            list_marker: Style::default().fg(Color::Rgb(111, 231, 183)),
            ordered_marker: Style::default()
                .fg(Color::Rgb(115, 197, 255))
                .add_modifier(Modifier::BOLD),
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
        BusyPhase, ComposerPart, InFlightWorkKind, InputMode, InteractiveApp, InteractiveFrontend,
        PendingReveal, QueuedPrompt, RuntimeUpdateCompletion, TranscriptRow, TranscriptRowKind,
        TranscriptTiming, TranscriptTimingKind, UiMessage, collapse_event_details, detect_palette,
        format_duration_compact, format_turn_event_row, inline_multiline_text,
        inline_viewport_height_for_terminal, render_row_lines, select_interactive_frontend,
        web_server_ready_row,
    };
    use crate::application::{ConversationSession, RuntimeLaneConfig};
    use crate::domain::model::{
        ControlOperation, ControlResult, ControlResultStatus, ControlSubject,
        ConversationTranscript, ConversationTranscriptEntry, ConversationTranscriptSpeaker,
        ConversationTranscriptUpdate, RenderBlock, RenderDocument, TaskTraceId, TraceRecordId,
        TurnControlOperation, TurnEvent, TurnIntent, TurnTraceId,
    };
    use crate::infrastructure::credentials::ProviderAvailability;
    use crate::infrastructure::providers::ModelProvider;
    use crate::infrastructure::step_timing::Pace;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::style::Modifier;
    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, prelude::Rect, text::Line};
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
                    citations: Vec::new(),
                    grounded: None,
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

    fn rendered_line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>()
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
                citations: Vec::new(),
                grounded: Some(false),
            }],
        };

        app.load_transcript(&transcript);
        let buffer = render_buffer(&app, 80, 12);
        let rendered = buffer_text(&buffer);
        assert!(rendered.contains("Summary"));
        assert!(!rendered.contains("**Summary**"));
    }

    #[test]
    fn assistant_rows_keep_numbered_section_labels_tight_with_following_bullets() {
        let palette = detect_palette();
        let row = TranscriptRow::new(TranscriptRowKind::Assistant, "Paddles", "").with_render(
            RenderDocument {
                blocks: vec![
                    RenderBlock::Paragraph {
                        text: "1. Shared contract surfaces".to_string(),
                    },
                    RenderBlock::BulletList {
                        items: vec!["Define what HQ owns vs what spoke owns.".to_string()],
                    },
                ],
            },
        );

        let lines = render_row_lines(&row, &palette);
        let rendered = lines
            .iter()
            .skip(1)
            .map(rendered_line_text)
            .collect::<Vec<_>>();

        assert_eq!(
            rendered,
            vec![
                "  └ 1. Shared contract surfaces".to_string(),
                "    - Define what HQ owns vs what spoke owns.".to_string(),
            ]
        );
    }

    #[test]
    fn assistant_rows_style_ordered_and_bullet_markers_distinctly() {
        let palette = detect_palette();
        let row = TranscriptRow::new(TranscriptRowKind::Assistant, "Paddles", "").with_render(
            RenderDocument {
                blocks: vec![
                    RenderBlock::Paragraph {
                        text: "1. Shared contract surfaces".to_string(),
                    },
                    RenderBlock::BulletList {
                        items: vec!["Define what HQ owns vs what spoke owns.".to_string()],
                    },
                ],
            },
        );

        let lines = render_row_lines(&row, &palette);

        assert_eq!(lines[1].spans[1].content.as_ref(), "1. ");
        assert_eq!(lines[1].spans[1].style, palette.ordered_marker);
        assert_eq!(lines[2].spans[1].content.as_ref(), "- ");
        assert_eq!(lines[2].spans[1].style, palette.list_marker);
    }

    #[test]
    fn assistant_rows_keep_headings_tight_with_following_content() {
        let palette = detect_palette();
        let row = TranscriptRow::new(TranscriptRowKind::Assistant, "Paddles", "").with_render(
            RenderDocument {
                blocks: vec![
                    RenderBlock::Heading {
                        text: "Summary".to_string(),
                    },
                    RenderBlock::Paragraph {
                        text: "Body".to_string(),
                    },
                    RenderBlock::Heading {
                        text: "Checklist".to_string(),
                    },
                    RenderBlock::BulletList {
                        items: vec!["Ship it.".to_string()],
                    },
                ],
            },
        );

        let lines = render_row_lines(&row, &palette);
        let rendered = lines
            .iter()
            .skip(1)
            .map(rendered_line_text)
            .collect::<Vec<_>>();

        assert_eq!(
            rendered,
            vec![
                "  └ Summary".to_string(),
                "    Body".to_string(),
                "    ".to_string(),
                "    Checklist".to_string(),
                "    - Ship it.".to_string(),
            ]
        );
    }

    #[test]
    fn assistant_rows_style_headings_distinctly_from_body_bold() {
        let palette = detect_palette();
        let row = TranscriptRow::new(TranscriptRowKind::Assistant, "Paddles", "").with_render(
            RenderDocument {
                blocks: vec![RenderBlock::Heading {
                    text: "Summary".to_string(),
                }],
            },
        );

        let lines = render_row_lines(&row, &palette);

        assert_ne!(
            lines[1].spans[1].style,
            palette.assistant_body.add_modifier(Modifier::BOLD)
        );
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
        let event = TurnEvent::ToolCalled {
            call_id: "tool-1".to_string(),
            tool_name: "shell".to_string(),
            invocation: "git status --short".to_string(),
        };
        let row = format_turn_event_row(event.clone(), 0);

        assert_eq!(row.kind, TranscriptRowKind::Event);
        assert_eq!(row.header, "• Ran shell");
        assert_eq!(row.content, "git status --short");
        assert_eq!(row.runtime_items, event.runtime_items());
    }

    #[test]
    fn plan_updates_render_as_checklist_rows_in_the_tui_transcript() {
        let event = TurnEvent::PlanUpdated {
            items: vec![
                crate::domain::model::PlanChecklistItem {
                    id: "inspect".to_string(),
                    label: "Inspect `git status --short`".to_string(),
                    status: crate::domain::model::PlanChecklistItemStatus::Pending,
                },
                crate::domain::model::PlanChecklistItem {
                    id: "verify".to_string(),
                    label: "Verify the change and summarize the outcome.".to_string(),
                    status: crate::domain::model::PlanChecklistItemStatus::Completed,
                },
            ],
        };
        let row = format_turn_event_row(event.clone(), 0);

        assert_eq!(row.kind, TranscriptRowKind::Event);
        assert_eq!(row.header, "• Updated Plan");
        assert!(row.content.contains("□ Inspect `git status --short`"));
        assert!(
            row.content
                .contains("✓ Verify the change and summarize the outcome.")
        );
        assert_eq!(row.runtime_items, event.runtime_items());
    }

    #[test]
    fn terminal_output_events_render_as_terminal_rows() {
        let row = format_turn_event_row(
            TurnEvent::ToolOutput {
                call_id: "tool-1".to_string(),
                tool_name: "shell".to_string(),
                stream: "stdout".to_string(),
                output: "alpha\nbeta".to_string(),
            },
            0,
        );

        assert_eq!(row.kind, TranscriptRowKind::Event);
        assert_eq!(row.header, "• shell stdout");
        assert_eq!(row.content, "alpha\nbeta");
    }

    #[test]
    fn applied_edit_events_render_diff_lines_in_the_tui_transcript() {
        let palette = detect_palette();
        let event = TurnEvent::WorkspaceEditApplied {
            call_id: "tool-2".to_string(),
            tool_name: "write_file".to_string(),
            edit: crate::domain::model::AppliedEdit {
                files: vec!["note.txt".to_string()],
                diff: "--- a/note.txt\n+++ b/note.txt\n@@ -0,0 +1 @@\n+hello".to_string(),
                insertions: 1,
                deletions: 0,
            },
        };
        let row = format_turn_event_row(event.clone(), 0);

        assert_eq!(row.header, "• Applied write_file");
        assert!(row.content.contains("+++ b/note.txt"));
        assert_eq!(row.runtime_items, event.runtime_items());

        let rendered_lines = render_row_lines(&row, &palette);
        let rendered_text: Vec<String> = rendered_lines
            .iter()
            .skip(1)
            .map(|line| line.spans.iter().map(|span| span.content.clone()).collect())
            .collect();

        assert!(
            rendered_text
                .iter()
                .any(|line| line.contains("--- a/note.txt"))
        );
        assert!(
            rendered_text
                .iter()
                .any(|line| line.contains("+++ b/note.txt"))
        );
        assert!(rendered_text.iter().any(|line| line.contains("+hello")));
    }

    #[test]
    fn control_state_rows_keep_runtime_items_and_degraded_detail_visible() {
        let event = TurnEvent::ControlStateChanged {
            result: ControlResult {
                operation: ControlOperation::Turn(TurnControlOperation::Interrupt),
                status: ControlResultStatus::Unavailable,
                subject: ControlSubject::default(),
                detail: "planner lane is reconfiguring and cannot honor interrupt yet".to_string(),
            },
        };
        let row = format_turn_event_row(event.clone(), 0);

        assert_eq!(row.kind, TranscriptRowKind::Event);
        assert_eq!(row.header, "• Control: interrupt unavailable");
        assert_eq!(
            row.content,
            "planner lane is reconfiguring and cannot honor interrupt yet"
        );
        assert_eq!(row.runtime_items, event.runtime_items());
    }

    #[test]
    fn external_capability_results_render_with_shared_fabric_vocabulary() {
        let event = TurnEvent::ToolFinished {
            call_id: "tool-3".to_string(),
            tool_name: "external_capability".to_string(),
            summary: "fabric=web.search status=degraded availability=stale auth=none_required effects=read_only\npurpose=confirm the latest release notes\nsummary=Web Search degraded\ndetail=Capability metadata is stale; using cached release notes.\nprovenance=Release notes -> https://example.com/releases".to_string(),
        };
        let row = format_turn_event_row(event.clone(), 0);

        assert_eq!(row.kind, TranscriptRowKind::Event);
        assert_eq!(row.header, "• External fabric result");
        assert!(row.content.contains("summary=Web Search degraded"));
        assert!(
            row.content
                .contains("provenance=Release notes -> https://example.com/releases")
        );
        assert_eq!(row.runtime_items, event.runtime_items());
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
        assert!(row.content.contains("watch=slow"));
        assert!(row.content.contains("projected_total=30s"));
        assert!(!row.content.contains("timeout="));
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
                work_id: None,
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
                work_id: None,
            }
        };

        app.handle_message(hunt_event(75921, 38318, 332391, started));
        app.handle_message(governor_event(75921, 38318, 332391, started));
        assert_eq!(app.rows.len(), rows_before + 1);

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

        assert_eq!(app.rows.len(), rows_before + 1);

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

        assert_eq!(app.rows.len(), rows_before + 2);
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
    fn prompt_wrap_redraw_does_not_leave_duplicated_text_on_the_next_line() {
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
        let width = 20;
        let height = 9;
        let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("terminal");

        app.input = "abcdefghijklmnopqr".to_string();
        app.cursor_pos = app.input.chars().count();
        terminal
            .draw(|frame| app.render(frame))
            .expect("first frame");

        app.input.push('s');
        app.cursor_pos = app.input.chars().count();
        terminal
            .draw(|frame| app.render(frame))
            .expect("second frame");

        let buffer = terminal.backend().buffer().clone();
        let input_height = app.input_area_height(width);
        let input_top = height.saturating_sub(1 + input_height);
        let first_body_line = buffer_line(&buffer, input_top + 1);
        let second_body_line = buffer_line(&buffer, input_top + 2);
        let inner_width = usize::from(width.saturating_sub(2));

        let first_body = first_body_line
            .chars()
            .skip(1)
            .take(inner_width)
            .collect::<String>()
            .trim_end()
            .to_string();
        let second_body = second_body_line
            .chars()
            .skip(1)
            .take(inner_width)
            .collect::<String>()
            .trim_end()
            .to_string();

        assert_eq!(first_body, "abcdefghijklmnopqr");
        assert_eq!(second_body, "s");
    }

    #[test]
    fn cursor_stays_inside_prompt_body_when_text_exactly_fills_the_line() {
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
        let area = Rect::new(0, 5, 20, 3);

        app.input = "abcdefghijklmnopqr".to_string();
        app.cursor_pos = app.input.chars().count();

        assert_eq!(app.cursor_position(area), (18, 6));
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
            work_id: None,
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
    fn turn_finished_renders_fallback_when_transcript_sync_never_arrives() {
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
        assert_eq!(
            app.dispatch_next_prompt(),
            Some(QueuedPrompt::Prompt("hello".to_string()))
        );

        app.handle_message(super::UiMessage::TurnFinished {
            result: Ok("hi there".to_string()),
            occurred_at: Instant::now(),
            work_id: None,
        });

        for _ in 0..16 {
            if !app.busy {
                break;
            }
            app.tick();
        }

        assert!(!app.busy, "fallback rendering should close the busy state");
        assert_eq!(app.busy_phase, BusyPhase::Idle);
        assert_eq!(
            app.rows
                .last()
                .map(|row| (row.header.as_str(), row.content.as_str())),
            Some(("Paddles", "hi there"))
        );
    }

    #[test]
    fn late_transcript_sync_reuses_fallback_assistant_row_without_duplicate() {
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
        let submitted_user_rows = app
            .rows
            .iter()
            .filter(|row| row.kind == TranscriptRowKind::User)
            .collect::<Vec<_>>();
        assert_eq!(submitted_user_rows.len(), 1);
        assert_eq!(submitted_user_rows[0].content, "hello");
        app.sync_transcript(&transcript(&[(
            ConversationTranscriptSpeaker::User,
            "hello",
        )]));
        let synced_user_rows = app
            .rows
            .iter()
            .filter(|row| row.kind == TranscriptRowKind::User)
            .collect::<Vec<_>>();
        assert_eq!(synced_user_rows.len(), 1);
        assert_eq!(synced_user_rows[0].content, "hello");
        assert_eq!(
            app.dispatch_next_prompt(),
            Some(QueuedPrompt::Prompt("hello".to_string()))
        );
        app.handle_message(super::UiMessage::TurnFinished {
            result: Ok("hi there".to_string()),
            occurred_at: Instant::now(),
            work_id: None,
        });

        for _ in 0..16 {
            if !app.busy {
                break;
            }
            app.tick();
        }

        let transcript = ConversationTranscript {
            task_id: TaskTraceId::new("task-1").expect("task"),
            entries: vec![
                ConversationTranscriptEntry {
                    record_id: TraceRecordId::new("record-1").expect("record"),
                    turn_id: TurnTraceId::new("task-1.turn-0001").expect("turn"),
                    speaker: ConversationTranscriptSpeaker::User,
                    content: "hello".to_string(),
                    response_mode: None,
                    render: None,
                    citations: Vec::new(),
                    grounded: None,
                },
                ConversationTranscriptEntry {
                    record_id: TraceRecordId::new("record-2").expect("record"),
                    turn_id: TurnTraceId::new("task-1.turn-0002").expect("turn"),
                    speaker: ConversationTranscriptSpeaker::Assistant,
                    content: "hi there".to_string(),
                    response_mode: None,
                    render: Some(RenderDocument {
                        blocks: vec![RenderBlock::Paragraph {
                            text: "hi there".to_string(),
                        }],
                    }),
                    citations: Vec::new(),
                    grounded: Some(false),
                },
            ],
        };

        app.sync_transcript(&transcript);

        let assistant_rows = app
            .rows
            .iter()
            .filter(|row| row.kind == TranscriptRowKind::Assistant)
            .collect::<Vec<_>>();
        assert_eq!(assistant_rows.len(), 1);
        assert_eq!(assistant_rows[0].content, "hi there");
        assert_eq!(
            assistant_rows[0]
                .render
                .as_ref()
                .map(|render| render.blocks.len()),
            Some(1)
        );
    }

    #[test]
    fn fallback_reveal_normalizes_heading_render_without_waiting_for_transcript_sync() {
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
        assert_eq!(
            app.dispatch_next_prompt(),
            Some(QueuedPrompt::Prompt("hello".to_string()))
        );

        app.handle_message(super::UiMessage::TurnFinished {
            result: Ok("**Summary**\n\n\nBody".to_string()),
            occurred_at: Instant::now(),
            work_id: None,
        });

        for _ in 0..16 {
            if !app.busy {
                break;
            }
            app.tick();
        }

        let row = app.rows.last().expect("assistant row");
        let rendered = render_row_lines(row, &palette)
            .iter()
            .skip(1)
            .map(rendered_line_text)
            .collect::<Vec<_>>();

        assert_eq!(
            rendered,
            vec!["  └ Summary".to_string(), "    Body".to_string()]
        );
        assert!(row.render.is_some());
    }

    #[test]
    fn live_transcript_keeps_tail_of_tall_assistant_rows_visible() {
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

        app.rows.push(
            TranscriptRow::new(TranscriptRowKind::Assistant, "Paddles", "").with_render(
                RenderDocument {
                    blocks: vec![
                        RenderBlock::Paragraph {
                            text: "Best next integration opportunities for hq with spoke are:"
                                .to_string(),
                        },
                        RenderBlock::BulletList {
                            items: vec![
                                "Consume a Radar-derived social graph as an input to HQ prioritization.".to_string(),
                                "Make HQ a control-plane over Spoke social primitives.".to_string(),
                                "Feed HQ state back into Spoke.".to_string(),
                                "Start with a thin integration seam.".to_string(),
                                "Relationship radar: who around this mission matters most right now?".to_string(),
                            ],
                        },
                    ],
                },
            ),
        );

        let buffer = render_buffer(&app, 72, 9);
        let rendered = buffer_text(&buffer);

        assert!(rendered.contains("Relationship radar"));
    }

    #[test]
    fn app_requests_same_turn_steering_while_busy() {
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
        let turn_id = app.session.allocate_turn_id();
        app.session.mark_turn_active(turn_id.clone());

        app.input = "steer harder".to_string();
        app.submit_prompt();

        assert_eq!(app.input, "");
        assert!(app.queued_prompts.is_empty());
        let controls = app.session.take_turn_control_requests(&turn_id);
        assert_eq!(controls.len(), 1);
        assert_eq!(controls[0].prompt.as_deref(), Some("steer harder"));
        assert!(
            app.rows
                .iter()
                .any(|row| row.header == "• Requested same-turn steering")
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
            work_id: None,
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
            work_id: None,
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
    fn web_server_ready_row_reports_bound_socket_address() {
        let row = web_server_ready_row(std::net::SocketAddr::from(([0, 0, 0, 0], 41234)));

        assert_eq!(row.kind, TranscriptRowKind::CommandNotice);
        assert_eq!(row.header, "• Web UI ready");
        assert!(row.content.contains("http://0.0.0.0:41234"));
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
    fn model_command_lists_enabled_and_disabled_provider_selector_entries() {
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

        assert!(matches!(app.input_mode, InputMode::ModelSelection(_)));
        let rendered = app
            .model_selection_lines()
            .expect("provider selector lines")
            .iter()
            .map(rendered_line_text)
            .collect::<Vec<_>>();
        assert!(rendered.iter().any(|line| line.contains("> openai")));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("anthropic") && line.contains("disabled"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("inception") && line.contains("disabled"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("moonshot") && line.contains("disabled"))
        );
    }

    #[test]
    fn model_provider_selector_wraps_visible_order_from_active_selection() {
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
        app.submit_prompt();
        if let InputMode::ModelSelection(state) = &mut app.input_mode {
            state.selected_index = ModelProvider::all().len() - 1;
        }

        let rendered = app
            .model_selection_lines()
            .expect("provider selector lines")
            .iter()
            .map(rendered_line_text)
            .collect::<Vec<_>>();

        assert!(
            rendered
                .first()
                .is_some_and(|line| line.contains("> ollama"))
        );
        assert!(rendered.get(1).is_some_and(|line| line.contains("sift")));
    }

    #[test]
    fn model_command_requires_selector_instead_of_direct_provider_and_model_args() {
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
        app.input = "/model openai gpt-4o".to_string();

        app.submit_prompt();

        assert!(app.take_pending_runtime_update().is_none());
        assert!(app.rows.iter().any(|row| {
            row.header == "• Model command invalid"
                && row
                    .content
                    .contains("Use `/model`, then press Enter to choose from the selector.")
        }));
    }

    #[test]
    fn runtime_update_dispatch_enters_reconfiguring_phase_without_blocking() {
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
        app.input = "/model".to_string();
        app.submit_prompt();
        if let InputMode::ModelSelection(state) = &mut app.input_mode {
            state.selected_index = ModelProvider::all()
                .iter()
                .position(|provider| *provider == ModelProvider::Openai)
                .expect("openai index");
        }
        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!should_exit);
        if let InputMode::ModelSelection(state) = &mut app.input_mode {
            state.selected_index = ModelProvider::Openai
                .selectable_model_ids()
                .iter()
                .position(|model_id| *model_id == "gpt-4o")
                .expect("openai gpt-4o index");
        }
        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!should_exit);

        let started_at = Instant::now();
        let update = app
            .dispatch_pending_runtime_update_at(started_at)
            .expect("runtime update");

        assert_eq!(
            update.runtime_lanes.synthesizer_provider(),
            ModelProvider::Openai
        );
        assert!(app.busy);
        assert_eq!(app.busy_phase, BusyPhase::Reconfiguring);
        assert_eq!(app.runtime_update_started_at, Some(started_at));
        assert!(
            app.rows
                .iter()
                .any(|row| row.header == "• Activating runtime lanes"
                    && row.content.contains("openai:gpt-4o"))
        );
    }

    #[test]
    fn runtime_update_completion_updates_catalog_and_clears_busy_state() {
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
        let started_at = Instant::now();
        app.busy = true;
        app.busy_phase = BusyPhase::Reconfiguring;
        app.runtime_update_started_at = Some(started_at);

        app.handle_message(UiMessage::RuntimeUpdateFinished {
            result: Ok(RuntimeUpdateCompletion {
                runtime_lanes: RuntimeLaneConfig::new("gpt-4o".to_string(), None)
                    .with_synthesizer_provider(ModelProvider::Openai),
                provider_availability: vec![
                    provider_availability(ModelProvider::Sift, true, "auth not required"),
                    provider_availability(
                        ModelProvider::Openai,
                        true,
                        "using local credential store",
                    ),
                ],
                summary: "Runtime lanes now target `openai:gpt-4o`.".to_string(),
                preference_path: PathBuf::from("/tmp/runtime-lanes.toml"),
                preference_save_error: None,
            }),
            occurred_at: started_at + Duration::from_secs(2),
            work_id: None,
        });

        assert!(!app.busy);
        assert_eq!(app.busy_phase, BusyPhase::Idle);
        assert!(app.runtime_update_started_at.is_none());
        assert_eq!(
            app.runtime_lanes.synthesizer_provider(),
            ModelProvider::Openai
        );
        assert_eq!(app.runtime_lanes.synthesizer_model_id(), "gpt-4o");
        assert!(
            app.rows
                .iter()
                .any(|row| row.header == "• Model selection updated"
                    && row.content.contains("runtime-lanes.toml"))
        );
    }

    #[test]
    fn model_command_rejects_overlapping_runtime_reconfiguration() {
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
        app.busy = true;
        app.busy_phase = BusyPhase::Reconfiguring;

        app.input = "/model".to_string();
        app.submit_prompt();
        if let InputMode::ModelSelection(state) = &mut app.input_mode {
            state.selected_index = ModelProvider::all()
                .iter()
                .position(|provider| *provider == ModelProvider::Openai)
                .expect("openai index");
        }
        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!should_exit);
        if let InputMode::ModelSelection(state) = &mut app.input_mode {
            state.selected_index = ModelProvider::Openai
                .selectable_model_ids()
                .iter()
                .position(|model_id| *model_id == "gpt-4o")
                .expect("openai gpt-4o index");
        }
        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!should_exit);

        assert!(
            app.rows
                .iter()
                .any(|row| row.kind == TranscriptRowKind::Error
                    && row.header == "• Model selection busy")
        );
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
        app.input = "/model".to_string();

        app.submit_prompt();
        if let InputMode::ModelSelection(state) = &mut app.input_mode {
            state.selected_index = ModelProvider::all()
                .iter()
                .position(|provider| *provider == ModelProvider::Anthropic)
                .expect("anthropic index");
        }
        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!should_exit);

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
        assert_eq!(suggestions, vec!["/model"]);
    }

    #[test]
    fn slash_command_popup_renders_resume_command() {
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
        app.input = "/res".to_string();
        app.cursor_pos = app.input.chars().count();

        let buffer = render_buffer(&app, 100, 18);
        let rendered = buffer_text(&buffer);
        let suggestions = app
            .slash_command_suggestions()
            .into_iter()
            .map(|command| command.usage)
            .collect::<Vec<_>>();

        assert!(rendered.contains("Commands"));
        assert!(rendered.contains("/resume"));
        assert_eq!(suggestions, vec!["/resume"]);
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
    fn model_command_enters_provider_selection_in_prompt_box() {
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

        app.submit_prompt();

        let buffer = render_buffer(&app, 120, 18);
        let rendered = buffer_text(&buffer);

        assert!(matches!(app.input_mode, InputMode::ModelSelection(_)));
        assert_eq!(app.input, "/model");
        assert!(rendered.contains("> sift"));
        assert!(!rendered.contains("Commands"));
        assert_eq!(
            app.input_area_height(120),
            ModelProvider::all().len() as u16 + 2
        );
    }

    #[test]
    fn model_provider_selection_advances_to_model_list_in_prompt_box() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "mercury-2".to_string(),
            palette,
            session(),
            "inception".to_string(),
            None,
            "Provider: `inception`. Auth: using local credential store.".to_string(),
            2,
        );
        app.set_runtime_catalog(
            RuntimeLaneConfig::new("mercury-2".to_string(), None)
                .with_synthesizer_provider(ModelProvider::Inception),
            vec![provider_availability(
                ModelProvider::Inception,
                true,
                "using local credential store",
            )],
        );
        app.input = "/model".to_string();

        app.submit_prompt();
        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        let buffer = render_buffer(&app, 120, 18);
        let rendered = buffer_text(&buffer);

        assert!(!should_exit);
        assert!(matches!(app.input_mode, InputMode::ModelSelection(_)));
        assert_eq!(app.input, "/model inception");
        assert!(rendered.contains("> mercury-2"));
        assert!(!rendered.contains("Commands"));
    }

    #[test]
    fn moonshot_model_selection_advances_to_thinking_mode_list_when_available() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "kimi-k2.5".to_string(),
            palette,
            session(),
            "moonshot".to_string(),
            None,
            "Provider: `moonshot`. Auth: using local credential store.".to_string(),
            2,
        );
        app.set_runtime_catalog(
            RuntimeLaneConfig::new("kimi-k2.5".to_string(), None)
                .with_synthesizer_provider(ModelProvider::Moonshot),
            vec![provider_availability(
                ModelProvider::Moonshot,
                true,
                "using local credential store",
            )],
        );
        app.input = "/model".to_string();

        app.submit_prompt();
        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!should_exit);

        if let InputMode::ModelSelection(state) = &mut app.input_mode {
            state.selected_index = ModelProvider::Moonshot
                .selectable_model_ids()
                .iter()
                .position(|model_id| *model_id == "kimi-k2")
                .expect("kimi-k2 index");
        }

        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        let buffer = render_buffer(&app, 120, 18);
        let rendered = buffer_text(&buffer);

        assert!(!should_exit);
        assert_eq!(app.input, "/model moonshot kimi-k2");
        assert!(rendered.contains("Select thinking mode"));
        assert!(rendered.contains("> Default"));
        assert!(rendered.contains("Thinking"));
    }

    #[test]
    fn moonshot_thinking_mode_selection_queues_runtime_update_from_prompt_box() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "kimi-k2.5".to_string(),
            palette,
            session(),
            "moonshot".to_string(),
            None,
            "Provider: `moonshot`. Auth: using local credential store.".to_string(),
            2,
        );
        app.set_runtime_catalog(
            RuntimeLaneConfig::new("kimi-k2.5".to_string(), None)
                .with_synthesizer_provider(ModelProvider::Moonshot),
            vec![provider_availability(
                ModelProvider::Moonshot,
                true,
                "using local credential store",
            )],
        );
        app.input = "/model".to_string();

        app.submit_prompt();
        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!should_exit);

        if let InputMode::ModelSelection(state) = &mut app.input_mode {
            state.selected_index = ModelProvider::Moonshot
                .selectable_model_ids()
                .iter()
                .position(|model_id| *model_id == "kimi-k2")
                .expect("kimi-k2 index");
        }

        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!should_exit);

        if let InputMode::ModelSelection(state) = &mut app.input_mode {
            state.selected_index = ModelProvider::Moonshot
                .thinking_modes("kimi-k2")
                .iter()
                .position(|mode| mode.model_override == Some("kimi-k2-thinking"))
                .expect("thinking mode index");
        }

        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(!should_exit);
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.input, "");
        let update = app
            .take_pending_runtime_update()
            .expect("pending runtime update");
        assert_eq!(
            update.runtime_lanes.synthesizer_model_id(),
            "kimi-k2-thinking"
        );
        assert!(update.runtime_lanes.synthesizer_thinking_mode().is_none());
    }

    #[test]
    fn openai_gpt_5_4_thinking_mode_selection_queues_runtime_update_from_prompt_box() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "gpt-4o".to_string(),
            palette,
            session(),
            "openai".to_string(),
            None,
            "Provider: `openai`. Auth: using local credential store.".to_string(),
            2,
        );
        app.set_runtime_catalog(
            RuntimeLaneConfig::new("gpt-4o".to_string(), None)
                .with_synthesizer_provider(ModelProvider::Openai),
            vec![provider_availability(
                ModelProvider::Openai,
                true,
                "using local credential store",
            )],
        );
        app.input = "/model".to_string();

        app.submit_prompt();
        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!should_exit);

        if let InputMode::ModelSelection(state) = &mut app.input_mode {
            state.selected_index = ModelProvider::Openai
                .selectable_model_ids()
                .iter()
                .position(|model_id| *model_id == "gpt-5.4")
                .expect("gpt-5.4 index");
        }

        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        let buffer = render_buffer(&app, 120, 18);
        let rendered = buffer_text(&buffer);

        assert!(!should_exit);
        assert_eq!(app.input, "/model openai gpt-5.4");
        assert!(rendered.contains("Select thinking mode"));
        assert!(rendered.contains("> None"));
        assert!(rendered.contains("High"));

        if let InputMode::ModelSelection(state) = &mut app.input_mode {
            state.selected_index = ModelProvider::Openai
                .thinking_modes("gpt-5.4")
                .iter()
                .position(|mode| mode.thinking_mode == Some("high"))
                .expect("high thinking mode index");
        }

        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(!should_exit);
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.input, "");
        let update = app
            .take_pending_runtime_update()
            .expect("pending runtime update");
        assert_eq!(
            update.runtime_lanes.synthesizer_provider(),
            ModelProvider::Openai
        );
        assert_eq!(update.runtime_lanes.synthesizer_model_id(), "gpt-5.4");
        assert_eq!(
            update.runtime_lanes.synthesizer_thinking_mode(),
            Some("high")
        );
        assert_eq!(
            update.persisted_preferences.thinking_mode.as_deref(),
            Some("high")
        );
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
    fn slash_command_model_suggestions_are_disabled() {
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
        app.input = "/model inception mercury".to_string();
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

    #[tokio::test]
    async fn escape_requests_turn_interrupt_before_falling_back_to_abort() {
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
        let turn_id = app.session.allocate_turn_id();
        app.session.mark_turn_active(turn_id.clone());

        let work_id = app.next_work_id();
        let handle = tokio::spawn(async move {
            loop {
                tokio::task::yield_now().await;
            }
        });
        app.set_active_work(InFlightWorkKind::Prompt, work_id, handle);

        app.input = "second".to_string();
        app.submit_prompt();
        assert!(app.queued_prompts.is_empty());

        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

        assert!(!should_exit);
        assert!(app.busy);
        let controls = app.session.take_turn_control_requests(&turn_id);
        assert_eq!(controls.len(), 2);
        assert_eq!(controls[0].prompt.as_deref(), Some("second"));
        assert_eq!(controls[1].prompt, None);
        assert!(
            app.rows
                .iter()
                .any(|row| row.header == "• Requested turn interrupt")
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
        app.input = "/login i".to_string();
        app.cursor_pos = app.input.chars().count();

        let frame_area = Rect::new(0, 41, 239, 9);
        let input_area = Rect::new(0, 48, 239, 2);
        let popup = app
            .command_popup_area(frame_area, input_area)
            .expect("popup area");

        assert!(popup.y >= frame_area.y);
        assert!(popup.bottom() <= frame_area.bottom());
        assert!(popup.bottom() <= input_area.y);
    }

    #[test]
    fn slash_command_popup_aligns_with_input_body() {
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

        let frame_area = Rect::new(0, 40, 120, 10);
        let input_area = Rect::new(4, 47, 100, 2);
        let popup = app
            .command_popup_area(frame_area, input_area)
            .expect("popup area");

        assert_eq!(popup.x, input_area.x);
        assert_eq!(popup.bottom(), input_area.y);
    }

    #[test]
    fn escape_hides_slash_command_popup_without_clearing_input() {
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

        let frame_area = Rect::new(0, 41, 120, 9);
        let input_area = Rect::new(0, 48, 120, 2);
        assert!(app.command_popup_area(frame_area, input_area).is_some());

        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

        assert!(!should_exit);
        assert_eq!(app.input, "/login i");
        assert_eq!(app.cursor_pos, "/login i".chars().count());
        assert!(app.command_popup_area(frame_area, input_area).is_none());
    }

    #[test]
    fn escape_cancels_model_provider_selection_without_exiting() {
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

        app.submit_prompt();
        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

        assert!(!should_exit);
        assert_eq!(app.input, "/model");
        assert_eq!(app.cursor_pos, "/model".chars().count());
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn freeform_model_provider_selection_returns_manual_input() {
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
                provider_availability(ModelProvider::Ollama, true, "auth not required"),
            ],
        );
        app.input = "/model".to_string();

        app.submit_prompt();
        if let InputMode::ModelSelection(state) = &mut app.input_mode {
            state.selected_index = ModelProvider::all()
                .iter()
                .position(|provider| *provider == ModelProvider::Ollama)
                .expect("ollama index");
        }
        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(!should_exit);
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.input, "/model ollama ");
        assert!(
            app.rows.iter().any(|row| row.header == "• Model selection"
                && row.content.contains("freeform model ids"))
        );
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
    fn model_selection_queues_runtime_update_from_prompt_box() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "mercury-2".to_string(),
            palette,
            session(),
            "inception".to_string(),
            None,
            "Provider: `inception`. Auth: using local credential store.".to_string(),
            2,
        );
        app.set_runtime_catalog(
            RuntimeLaneConfig::new("mercury-2".to_string(), None)
                .with_synthesizer_provider(ModelProvider::Inception),
            vec![provider_availability(
                ModelProvider::Inception,
                true,
                "using local credential store",
            )],
        );
        app.input = "/model".to_string();

        app.submit_prompt();
        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!should_exit);
        let should_exit =
            super::handle_key_event(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(!should_exit);
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.input, "");
        assert!(app.take_pending_runtime_update().is_some());
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
    fn resume_command_without_task_id_requests_resume_catalog() {
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
        app.input = "/resume".to_string();
        app.cursor_pos = app.input.chars().count();

        app.submit_prompt();

        assert_eq!(app.input, "");
        assert!(matches!(
            app.take_pending_resume_command(),
            Some(super::PendingResumeCommand::List)
        ));
        assert!(
            !app.rows
                .iter()
                .any(|row| row.kind == TranscriptRowKind::User)
        );
    }

    #[test]
    fn resume_command_with_task_id_requests_restore() {
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
        app.input = "/resume task-000123".to_string();
        app.cursor_pos = app.input.chars().count();

        app.submit_prompt();

        assert_eq!(app.input, "");
        assert!(matches!(
            app.take_pending_resume_command(),
            Some(super::PendingResumeCommand::Restore { task_id }) if task_id == "task-000123"
        ));
        assert!(
            !app.rows
                .iter()
                .any(|row| row.kind == TranscriptRowKind::User)
        );
    }

    #[test]
    fn slash_command_completion_does_not_expand_model_provider_or_model_ids() {
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
        app.input = "/model inc".to_string();
        app.cursor_pos = app.input.chars().count();

        assert!(!app.accept_selected_slash_completion());
        assert_eq!(app.input, "/model inc");
        assert_eq!(app.cursor_pos, "/model inc".chars().count());

        app.input = "/model inception mer".to_string();
        app.cursor_pos = app.input.chars().count();

        assert!(!app.accept_selected_slash_completion());
        assert_eq!(app.input, "/model inception mer");
        assert_eq!(app.cursor_pos, "/model inception mer".chars().count());
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
            work_id: None,
        });
        app.handle_message(super::UiMessage::TurnFinished {
            result: Ok("done".to_string()),
            occurred_at: start + Duration::from_millis(350),
            work_id: None,
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
    fn busy_label_uses_gathering_context_for_harness_snapshots() {
        let label = super::busy_label(
            BusyPhase::Thinking,
            Some(&TurnEvent::HarnessState {
                snapshot: crate::domain::model::HarnessSnapshot {
                    chamber: crate::domain::model::HarnessChamber::Gathering,
                    governor: crate::domain::model::GovernorState {
                        status: crate::domain::model::HarnessStatus::Intervening,
                        timeout: crate::domain::model::TimeoutState {
                            phase: crate::domain::model::TimeoutPhase::Expired,
                            elapsed_seconds: Some(303),
                            deadline_seconds: Some(1652),
                        },
                        intervention: Some(
                            "search Indexing has exceeded the watch threshold".to_string(),
                        ),
                    },
                    detail: Some("indexing 13617/76961 files".to_string()),
                },
            }),
        );

        assert_eq!(label, "hunting");
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
            work_id: None,
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
            work_id: None,
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
    fn tool_output_events_reuse_single_stream_row_and_append_chunks() {
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

        app.handle_message(UiMessage::TurnEvent {
            event: TurnEvent::ToolOutput {
                call_id: "tool-1".to_string(),
                tool_name: "shell".to_string(),
                stream: "stdout".to_string(),
                output: "alpha\n".to_string(),
            },
            occurred_at: Instant::now(),
            work_id: None,
        });
        let rows_after_first_chunk = app.rows.len();

        app.handle_message(UiMessage::TurnEvent {
            event: TurnEvent::ToolOutput {
                call_id: "tool-1".to_string(),
                tool_name: "shell".to_string(),
                stream: "stdout".to_string(),
                output: "beta".to_string(),
            },
            occurred_at: Instant::now(),
            work_id: None,
        });

        assert_eq!(app.rows.len(), rows_after_first_chunk);
        let row = app.rows.last().expect("tool output row");
        assert_eq!(row.header, "• shell stdout");
        assert_eq!(row.content, "alpha\nbeta");
    }

    #[test]
    fn tool_output_rows_do_not_spawn_shadow_in_flight_rows_after_silence() {
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

        app.handle_message(UiMessage::TurnEvent {
            event: TurnEvent::ToolOutput {
                call_id: "tool-1".to_string(),
                tool_name: "shell".to_string(),
                stream: "stdout".to_string(),
                output: "alpha\n".to_string(),
            },
            occurred_at: Instant::now() - Duration::from_secs(3),
            work_id: None,
        });
        let rows_after_output = app.rows.len();

        app.tick();

        assert_eq!(app.rows.len(), rows_after_output);
        assert_eq!(
            app.rows
                .iter()
                .filter(|row| row.kind == TranscriptRowKind::InFlightEvent)
                .count(),
            0,
            "tool output should remain the only visible row for that stream"
        );
    }

    #[test]
    fn tooling_harness_rows_do_not_spawn_shadow_in_flight_rows_after_tool_completion() {
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

        app.handle_message(UiMessage::TurnEvent {
            event: TurnEvent::ToolFinished {
                call_id: "tool-1".to_string(),
                tool_name: "inspect".to_string(),
                summary: "inspection completed".to_string(),
            },
            occurred_at: Instant::now() - Duration::from_secs(3),
            work_id: None,
        });
        let rows_after_completion = app.rows.len();

        app.handle_message(UiMessage::TurnEvent {
            event: TurnEvent::HarnessState {
                snapshot: crate::domain::model::HarnessSnapshot::active(
                    crate::domain::model::HarnessChamber::Tooling,
                )
                .with_detail("inspect: inspection completed".to_string()),
            },
            occurred_at: Instant::now() - Duration::from_secs(3),
            work_id: None,
        });

        app.tick();

        assert_eq!(app.rows.len(), rows_after_completion);
        assert_eq!(
            app.rows
                .iter()
                .filter(|row| row.kind == TranscriptRowKind::InFlightEvent)
                .count(),
            0,
            "tooling harness snapshots should not resurrect a duplicate inspect in-flight row"
        );
    }

    #[test]
    fn planner_owned_inspect_calls_do_not_repeat_matching_step_rows() {
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

        app.handle_message(UiMessage::TurnEvent {
            event: TurnEvent::PlannerStepProgress {
                step_number: 1,
                step_limit: 12,
                action: "inspect `git status --short`".to_string(),
                query: Some("git status --short".to_string()),
                evidence_count: 0,
            },
            occurred_at: Instant::now(),
            work_id: None,
        });
        let rows_after_step = app.rows.len();

        app.handle_message(UiMessage::TurnEvent {
            event: TurnEvent::ToolCalled {
                call_id: "tool-1".to_string(),
                tool_name: "inspect".to_string(),
                invocation: "git status --short".to_string(),
            },
            occurred_at: Instant::now(),
            work_id: None,
        });

        assert_eq!(app.rows.len(), rows_after_step);
        assert_eq!(
            app.rows.last().map(|row| row.header.as_str()),
            Some("• Step 1/12: inspect `git status --short` — git status --short"),
            "matching inspect tool calls should stay folded into the planner step row"
        );
    }

    #[test]
    fn governor_rows_do_not_pace_promote_into_default_streams() {
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

        app.handle_message(UiMessage::TurnEvent {
            event: TurnEvent::Fallback {
                stage: "premise-challenge".to_string(),
                reason: "Reviewed `inspect `git status --short`` and kept the same action."
                    .to_string(),
            },
            occurred_at: Instant::now(),
            work_id: None,
        });
        let rows_after_fallback = app.rows.len();

        app.handle_message(UiMessage::TurnEvent {
            event: TurnEvent::HarnessState {
                snapshot: crate::domain::model::HarnessSnapshot::intervening(
                    crate::domain::model::HarnessChamber::Governor,
                    "premise-challenge: kept the same action",
                ),
            },
            occurred_at: Instant::now() + Duration::from_secs(10),
            work_id: None,
        });

        assert_eq!(app.rows.len(), rows_after_fallback);
        assert!(
            app.rows
                .iter()
                .all(|row| !row.header.starts_with("• Governor:")),
            "governor state should not echo a visible fallback in the default stream"
        );
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
    fn in_flight_row_shows_tool_stream_detail_after_silence() {
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
            TurnEvent::ToolOutput {
                call_id: "tool-1".to_string(),
                tool_name: "shell".to_string(),
                stream: "stdout".to_string(),
                output: "alpha\nbeta".to_string(),
            },
            Instant::now() - Duration::from_secs(3),
        ));

        app.tick();

        let row = app.rows.last().expect("in-flight row");
        assert_eq!(row.kind, TranscriptRowKind::InFlightEvent);
        assert_eq!(row.header, "• shell stdout...");
        assert_eq!(row.content, "alpha\nbeta");
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
            work_id: None,
        });
        assert!(!app.emitted_in_flight);
        assert!(app.last_event.is_some());
    }

    #[test]
    fn direct_response_bookkeeping_does_not_promote_into_default_streams() {
        let palette = detect_palette();
        let app = InteractiveApp::new(
            "qwen-1.5b".to_string(),
            palette,
            session(),
            "sift".to_string(),
            None,
            "Provider: `sift` (local-first). Auth: not required.".to_string(),
            0,
        );

        assert!(!app.should_show_event(
            &TurnEvent::IntentClassified {
                intent: TurnIntent::DirectResponse,
            },
            Pace::Slow,
            false,
        ));
        assert!(
            !app.should_show_event(
                &TurnEvent::RouteSelected {
                    summary: "model selected a direct response; controller will render it directly"
                        .to_string(),
                },
                Pace::Slow,
                false,
            )
        );
        assert!(!app.should_show_event(
            &TurnEvent::SynthesisReady {
                grounded: false,
                citations: Vec::new(),
                insufficient_evidence: false,
            },
            Pace::Slow,
            false,
        ));
        assert!(app.should_show_event(
            &TurnEvent::SynthesisReady {
                grounded: true,
                citations: vec!["src/application/mod.rs".to_string()],
                insufficient_evidence: false,
            },
            Pace::Slow,
            false,
        ));
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
        assert_eq!(last_row.content, "line one\nline two\nline three");
    }

    #[test]
    fn multiline_user_rows_render_without_inserting_extra_blank_lines() {
        let palette = detect_palette();
        let row = TranscriptRow::new(TranscriptRowKind::User, "User", "Foo\nBar\nBaz");

        let rendered = render_row_lines(&row, &palette)
            .iter()
            .map(rendered_line_text)
            .collect::<Vec<_>>();

        assert_eq!(
            rendered,
            vec![
                "User".to_string(),
                "  └ Foo".to_string(),
                "    Bar".to_string(),
                "    Baz".to_string(),
            ]
        );
    }

    #[test]
    fn compressed_multiline_paste_rows_render_without_inserting_extra_blank_lines() {
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

        super::handle_paste_event(&mut app, "Foo\nBar\nBaz");
        app.submit_prompt();

        let last_row = app.rows.last().expect("user row exists");
        let rendered = render_row_lines(last_row, &palette)
            .iter()
            .map(rendered_line_text)
            .collect::<Vec<_>>();

        assert_eq!(
            rendered,
            vec![
                "User".to_string(),
                "  └ Foo".to_string(),
                "    Bar".to_string(),
                "    Baz".to_string(),
            ]
        );
    }

    #[test]
    fn transcript_buffer_does_not_insert_blank_lines_inside_multiline_user_rows() {
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

        app.rows.push(TranscriptRow::new(
            TranscriptRowKind::User,
            "User",
            "Foo\nBar",
        ));

        let buffer = render_buffer(&app, 40, 10);
        let lines = (0..buffer.area.height)
            .map(|y| buffer_line(&buffer, y).trim_end().to_string())
            .collect::<Vec<_>>();

        let foo_line = lines
            .iter()
            .position(|line| line.contains("Foo"))
            .expect("foo line");
        let bar_line = lines
            .iter()
            .position(|line| line.contains("Bar"))
            .expect("bar line");

        assert_eq!(bar_line, foo_line + 1);
    }

    #[test]
    fn transcript_buffer_preserves_single_blank_lines_inside_user_rows() {
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

        app.rows.push(TranscriptRow::new(
            TranscriptRowKind::User,
            "User",
            "Foo\n\nBar",
        ));

        let buffer = render_buffer(&app, 40, 10);
        let lines = (0..buffer.area.height)
            .map(|y| buffer_line(&buffer, y).trim_end().to_string())
            .collect::<Vec<_>>();

        let foo_line = lines
            .iter()
            .position(|line| line.contains("Foo"))
            .expect("foo line");
        let bar_line = lines
            .iter()
            .position(|line| line.contains("Bar"))
            .expect("bar line");

        assert_eq!(bar_line, foo_line + 2);
    }

    #[test]
    fn submit_prompt_immediately_adds_user_row_to_stream() {
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

        let last_row = app.rows.last().expect("user row exists");
        assert_eq!(last_row.kind, TranscriptRowKind::User);
        assert_eq!(last_row.header, "User");
        assert_eq!(last_row.content, "hello");
        assert_eq!(
            app.dispatch_next_prompt(),
            Some(QueuedPrompt::Prompt("hello".to_string()))
        );
    }

    #[test]
    fn user_stream_rows_expand_to_full_transcript_width() {
        let palette = detect_palette();
        let row = TranscriptRow::new(
            TranscriptRowKind::User,
            "User",
            "Why is the trace-recorder falling back?",
        );

        let rendered = super::render_row_lines_for_width(&row, &palette, 24);
        let text = rendered.iter().map(rendered_line_text).collect::<Vec<_>>();

        assert!(text.iter().all(|line| line.chars().count() == 24));
        assert_eq!(
            rendered[0].spans.last().and_then(|span| span.style.bg),
            Some(palette.input_bg)
        );
        assert!(rendered.iter().skip(1).all(|line| {
            line.spans
                .iter()
                .all(|span| span.style.bg == Some(palette.input_bg))
        }));
    }

    #[test]
    fn delegation_system_transcript_entries_sync_into_policy_rows() {
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

        app.sync_transcript(&transcript(&[(
            ConversationTranscriptSpeaker::System,
            "delegation: spawn accepted\nworker=worker-1\nrole=Worker\nownership=Own src/domain/model/projection.rs",
        )]));

        let last_row = app.rows.last().expect("system row exists");
        assert_eq!(last_row.kind, TranscriptRowKind::Event);
        assert_eq!(last_row.header, "Policy");
        assert!(last_row.content.contains("delegation: spawn accepted"));
        assert!(last_row.content.contains("worker=worker-1"));
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

    #[test]
    fn app_compresses_multiline_paste_into_a_prompt_chip_and_submits_raw_text() {
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

        super::handle_paste_event(&mut app, "alpha\nbeta\ngamma");

        assert!(app.input.is_empty());
        assert_eq!(
            app.composer_parts,
            vec![ComposerPart::Paste {
                text: "alpha\nbeta\ngamma".to_string(),
                lines: 3,
                preview: "alpha".to_string(),
            }]
        );
        let rendered = app
            .input_render_lines()
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert!(rendered.iter().any(|line| line.contains("3 lines pasted")));

        app.submit_prompt();

        assert_eq!(
            app.dispatch_next_prompt(),
            Some(QueuedPrompt::Prompt("alpha\nbeta\ngamma".to_string()))
        );
    }

    #[test]
    fn backspace_removes_a_compressed_multiline_paste_when_the_prompt_is_empty() {
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

        super::handle_paste_event(&mut app, "alpha\nbeta\ngamma");
        super::handle_key_event(
            &mut app,
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
        );

        assert!(app.composer_parts.is_empty());
        assert!(app.input.is_empty());
        let rendered = app
            .input_render_lines()
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert_eq!(rendered, vec!["Type a prompt...".to_string()]);
    }

    #[test]
    fn single_line_paste_stays_literal_in_the_tui_prompt() {
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
        app.cursor_pos = app.input.chars().count();

        super::handle_paste_event(&mut app, " world");

        assert_eq!(app.input, "hello world");
        assert!(app.composer_parts.is_empty());
    }

    #[test]
    fn pasted_text_is_not_prefixed_with_an_extra_leading_newline() {
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

        super::handle_paste_event(&mut app, "\nhello");

        assert_eq!(app.input, "hello");
        assert!(app.composer_parts.is_empty());
        let rendered = app
            .input_render_lines()
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert_eq!(rendered, vec!["hello".to_string()]);
    }

    #[test]
    fn multiline_compacted_paste_is_not_prefixed_with_a_blank_line() {
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

        super::handle_paste_event(&mut app, "\r\nhello\r\nworld\r\n");

        assert_eq!(
            app.composer_parts,
            vec![ComposerPart::Paste {
                text: "hello\nworld\n".to_string(),
                lines: 2,
                preview: "hello".to_string(),
            }]
        );
        let rendered = app
            .input_render_lines()
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert_eq!(rendered[0], "[2 lines pasted] hello");
    }

    #[test]
    fn compacted_paste_chip_renders_without_an_extra_empty_line() {
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

        super::handle_paste_event(&mut app, "• Fell back · 201ms\nsecond line");

        let rendered = app
            .input_render_lines()
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            rendered,
            vec!["[2 lines pasted] • Fell back · 201ms".to_string()]
        );
    }

    #[test]
    fn compacted_paste_after_a_typed_newline_does_not_render_a_blank_line_before_the_chip() {
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

        app.input = "Why is the trace-recorder falling back?\n".to_string();
        app.cursor_pos = app.input.chars().count();

        super::handle_paste_event(
            &mut app,
            "\n```\n• Fell back · 197ms\n  └ trace-recorder: trace recording failed: stream 'paddles.task.task-000001.root' already exists```",
        );

        let rendered = app
            .input_render_lines()
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            rendered,
            vec![
                "Why is the trace-recorder falling back?".to_string(),
                "[4 lines pasted] ```".to_string(),
            ]
        );

        app.submit_prompt();
        let last_row = app.rows.last().expect("user row exists");
        assert_eq!(
            last_row.content,
            "Why is the trace-recorder falling back?\n```\n• Fell back · 197ms\n  └ trace-recorder: trace recording failed: stream 'paddles.task.task-000001.root' already exists\n```"
        );
        let rendered = render_row_lines(last_row, &palette)
            .iter()
            .map(rendered_line_text)
            .collect::<Vec<_>>();
        assert_eq!(
            rendered,
            vec![
                "User".to_string(),
                "  └ Why is the trace-recorder falling back?".to_string(),
                "    ```".to_string(),
                "    • Fell back · 197ms".to_string(),
                "      └ trace-recorder: trace recording failed: stream 'paddles.task.task-000001.root' already exists".to_string(),
                "    ```".to_string(),
            ]
        );
    }

    #[test]
    fn compacted_paste_collapses_multiple_blank_lines_at_the_text_boundary() {
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

        app.input = "Why does the trace recorder keep falling back?\n\n".to_string();
        app.cursor_pos = app.input.chars().count();

        super::handle_paste_event(
            &mut app,
            "\n```\n• Fell back · 200ms\n  └ trace-recorder: trace recording failed: stream 'paddles.task.task-000001.root' already exists```",
        );

        let rendered = app
            .input_render_lines()
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            rendered,
            vec![
                "Why does the trace recorder keep falling back?".to_string(),
                "[4 lines pasted] ```".to_string(),
            ]
        );

        app.submit_prompt();
        let last_row = app.rows.last().expect("user row exists");
        assert_eq!(
            last_row.content,
            "Why does the trace recorder keep falling back?\n```\n• Fell back · 200ms\n  └ trace-recorder: trace recording failed: stream 'paddles.task.task-000001.root' already exists\n```"
        );
        let rendered = render_row_lines(last_row, &palette)
            .iter()
            .map(rendered_line_text)
            .collect::<Vec<_>>();
        assert_eq!(
            rendered,
            vec![
                "User".to_string(),
                "  └ Why does the trace recorder keep falling back?".to_string(),
                "    ```".to_string(),
                "    • Fell back · 200ms".to_string(),
                "      └ trace-recorder: trace recording failed: stream 'paddles.task.task-000001.root' already exists".to_string(),
                "    ```".to_string(),
            ]
        );
    }

    #[test]
    fn submitted_compacted_paste_keeps_code_fences_on_their_own_lines_in_full_width_user_rows() {
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

        app.input = "Why does the trace recorder keep falling back?\n".to_string();
        app.cursor_pos = app.input.chars().count();

        super::handle_paste_event(
            &mut app,
            "\n```\n• Fell back · 200ms\n  └ trace-recorder: trace recording failed: stream 'paddles.task.task-000001.root' already exists```",
        );

        app.submit_prompt();
        let last_row = app.rows.last().expect("user row exists");
        let rendered = super::render_row_lines_for_width(last_row, &palette, 120)
            .iter()
            .map(|line| rendered_line_text(line).trim_end().to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            rendered,
            vec![
                "User".to_string(),
                "  └ Why does the trace recorder keep falling back?".to_string(),
                "    ```".to_string(),
                "    • Fell back · 200ms".to_string(),
                "      └ trace-recorder: trace recording failed: stream 'paddles.task.task-000001.root' already exists".to_string(),
                "    ```".to_string(),
            ]
        );
    }

    #[test]
    fn multiline_paste_drops_leading_blank_lines_before_submission() {
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

        super::handle_paste_event(&mut app, "\n   \n```\n• Fell back · 197ms\n```");

        assert_eq!(
            app.composer_parts,
            vec![ComposerPart::Paste {
                text: "```\n• Fell back · 197ms\n```".to_string(),
                lines: 3,
                preview: "```".to_string(),
            }]
        );

        app.submit_prompt();
        assert_eq!(
            app.dispatch_next_prompt(),
            Some(QueuedPrompt::Prompt(
                "```\n• Fell back · 197ms\n```".to_string()
            ))
        );
    }
}
