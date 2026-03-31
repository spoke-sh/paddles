use crate::application::{ConversationSession, MechSuitService, RuntimeLaneConfig};
use crate::domain::model::{ThreadCandidate, TurnEvent, TurnEventSink};
use crate::infrastructure::credentials::CredentialStore;
use crate::infrastructure::step_timing::{Pace, StepTimingReservoir};
use anyhow::Result;
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size as terminal_size};
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};
use ratatui::{Frame, Terminal, TerminalOptions, Viewport};
use std::cmp;
use std::collections::VecDeque;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// Context passed from main to the TUI for credential management.
pub struct TuiContext {
    pub credential_store: Arc<CredentialStore>,
    pub api_key_shared: Arc<RwLock<String>>,
    pub runtime_lanes: RuntimeLaneConfig,
    pub provider_name: String,
    pub credential_provider: Option<String>,
    pub credential_status: String,
    pub verbose: u8,
}

const FRAME_INTERVAL: Duration = Duration::from_millis(32);
const ASSISTANT_REVEAL_STEP: usize = 24;
const EVENT_DETAIL_LINE_LIMIT: usize = 8;
const INLINE_VIEWPORT_MIN_HEIGHT: u16 = 5;
const INLINE_VIEWPORT_MAX_HEIGHT: u16 = 9;

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
    model_label: impl Into<String>,
    tui_ctx: TuiContext,
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
    let session = service.create_conversation_session();
    let mut app = InteractiveApp::new(
        model_label.into(),
        detect_palette(),
        session.clone(),
        tui_ctx.provider_name.clone(),
        tui_ctx.credential_provider.clone(),
        tui_ctx.credential_status.clone(),
        tui_ctx.verbose,
    );

    loop {
        drain_messages(&mut app, &mut rx);
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
                Ok(()) => {
                    *tui_ctx.api_key_shared.write().unwrap() = login.api_key;
                    match service.prepare_runtime_lanes(&tui_ctx.runtime_lanes).await {
                        Ok(_) => {
                            app.push_event(
                                "API key saved",
                                format!(
                                    "Credentials stored for `{}`. Runtime reconnected.",
                                    login.provider,
                                ),
                            );
                        }
                        Err(err) => {
                            app.push_error(
                                "Login failed",
                                format!("Key saved but runtime rebuild failed: {err:#}",),
                            );
                        }
                    }
                }
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
                    if handle_key_event(
                        &mut app,
                        key,
                        Arc::clone(&service),
                        session.clone(),
                        tx.clone(),
                    ) {
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

fn handle_key_event(
    app: &mut InteractiveApp,
    key: KeyEvent,
    _service: Arc<MechSuitService>,
    _session: ConversationSession,
    _tx: UnboundedSender<UiMessage>,
) -> bool {
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
        return true;
    }

    match key.code {
        KeyCode::Esc => {
            if matches!(app.input_mode, InputMode::MaskedKey { .. }) {
                app.input.clear();
                app.input_mode = InputMode::Normal;
                app.push_event("Login cancelled", "Returned to normal input.");
                false
            } else {
                true
            }
        }
        KeyCode::Enter => {
            app.submit_prompt();
            false
        }
        KeyCode::Backspace => {
            app.input.pop();
            false
        }
        KeyCode::Up => {
            app.history_back();
            false
        }
        KeyCode::Down => {
            app.history_forward();
            false
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.input.clear();
            false
        }
        KeyCode::Char(ch) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                app.input.push(ch);
                app.history_cursor = None;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TranscriptRowKind {
    User,
    Assistant,
    Event,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TranscriptRow {
    kind: TranscriptRowKind,
    header: String,
    content: String,
    timing: Option<TranscriptTiming>,
}

impl TranscriptRow {
    fn new(kind: TranscriptRowKind, header: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            kind,
            header: header.into(),
            content: content.into(),
            timing: None,
        }
    }

    fn estimated_height(&self, width: usize) -> usize {
        let width = width.max(8);
        let body_width = width.saturating_sub(4).max(1);
        let mut lines = wrapped_line_count(&self.display_header(), width);
        if self.content.is_empty() {
            return lines + 1;
        }

        for line in self.content.lines() {
            lines += wrapped_line_count(line, body_width);
        }
        lines + 1
    }

    fn timed(mut self, timing: TranscriptTiming) -> Self {
        self.timing = Some(timing);
        self
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
}

impl PendingReveal {
    fn new(row_index: usize, full_text: String) -> Self {
        Self {
            row_index,
            full_text,
            visible_chars: 0,
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

struct InteractiveApp {
    model_label: String,
    palette: Palette,
    session: ConversationSession,
    rows: Vec<TranscriptRow>,
    input: String,
    queued_prompts: VecDeque<QueuedPrompt>,
    busy: bool,
    busy_phase: BusyPhase,
    pending_reveal: Option<PendingReveal>,
    spinner_index: usize,
    input_mode: InputMode,
    pending_login: Option<PendingLogin>,
    provider_name: String,
    credential_provider: Option<String>,
    active_turn_timing: Option<ActiveTurnTiming>,
    flushed_row_count: usize,
    search_progress_row: Option<usize>,
    planner_progress_row: Option<usize>,
    step_timing: StepTimingReservoir,
    step_timing_path: PathBuf,
    verbose: u8,
    prompt_history: Vec<String>,
    history_cursor: Option<usize>,
    history_draft: String,
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
                 Type `/login` to set or replace your API key for `{provider}`.",
            ),
            None => format!(
                "Enter to send, Ctrl+C to quit.\n\
                 {credential_status}\n\
                 No API login required.",
            ),
        };
        Self {
            model_label,
            palette,
            session,
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
            provider_name,
            credential_provider,
            active_turn_timing: None,
            flushed_row_count: 0,
            search_progress_row: None,
            planner_progress_row: None,
            step_timing: StepTimingReservoir::load(&step_timing_cache_path()),
            step_timing_path: step_timing_cache_path(),
            verbose,
            prompt_history: Vec::new(),
            history_cursor: None,
            history_draft: String::new(),
        }
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
    }

    fn submit_prompt(&mut self) {
        let raw = self.input.trim().to_string();
        if raw.is_empty() {
            return;
        }
        self.prompt_history.push(raw.clone());
        self.history_cursor = None;
        self.history_draft.clear();
        self.input.clear();

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
        if raw.eq_ignore_ascii_case("/login") {
            if let Some(provider) = self.credential_provider.clone() {
                self.rows.push(TranscriptRow::new(
                    TranscriptRowKind::Event,
                    "• Login",
                    format!(
                        "Enter your API key for `{provider}`. Input is masked.\n\
                         Press Esc to cancel.",
                    ),
                ));
                self.input_mode = InputMode::MaskedKey { provider };
            } else {
                self.push_error(
                    "Login unavailable",
                    format!(
                        "The current provider `{}` does not use API-key login.",
                        self.provider_name
                    ),
                );
            }
            return;
        }

        // Normal prompt submission.
        let was_busy = self.busy || self.pending_reveal.is_some();
        self.rows
            .push(TranscriptRow::new(TranscriptRowKind::User, "You", &raw));
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
                    trim_for_display(&raw, 96)
                ),
            ));
        }
    }

    fn take_pending_login(&mut self) -> Option<PendingLogin> {
        self.pending_login.take()
    }

    fn push_event(&mut self, header: impl Into<String>, content: impl Into<String>) {
        self.rows.push(TranscriptRow::new(
            TranscriptRowKind::Event,
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

    fn handle_message(&mut self, message: UiMessage) {
        match message {
            UiMessage::TurnEvent { event, occurred_at } => {
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

                let is_search_progress = matches!(event, TurnEvent::GathererSearchProgress { .. });
                let is_planner_progress = matches!(event, TurnEvent::PlannerStepProgress { .. });

                if self.should_show_event(&event, pace, is_first_step) {
                    let row = format_turn_event_row(event, self.verbose);
                    let row = if let Some(timing) = self.active_turn_timing.as_mut() {
                        row.timed(timing.mark_step(occurred_at, pace))
                    } else {
                        row
                    };

                    if is_planner_progress {
                        // Replace existing planner progress row in-place.
                        if let Some(idx) = self.planner_progress_row {
                            if idx < self.rows.len() {
                                self.rows[idx] = row;
                            } else {
                                self.planner_progress_row = Some(self.rows.len());
                                self.rows.push(row);
                            }
                        } else {
                            self.planner_progress_row = Some(self.rows.len());
                            self.rows.push(row);
                        }
                    } else if is_search_progress {
                        // Replace existing search progress row in-place.
                        if let Some(idx) = self.search_progress_row {
                            if idx < self.rows.len() {
                                self.rows[idx] = row;
                            } else {
                                self.search_progress_row = Some(self.rows.len());
                                self.rows.push(row);
                            }
                        } else {
                            self.search_progress_row = Some(self.rows.len());
                            self.rows.push(row);
                        }
                    } else {
                        // Any non-progress event supersedes the search progress row.
                        self.search_progress_row = None;
                        self.rows.push(row);
                    }
                } else if let Some(timing) = self.active_turn_timing.as_mut() {
                    timing.mark_step(occurred_at, pace);
                }
            }
            UiMessage::TurnFinished {
                result,
                occurred_at,
            } => {
                self.search_progress_row = None;
                self.planner_progress_row = None;
                match result {
                    Ok(response) => {
                        let row_index = self.rows.len();
                        let row = self.annotate_turn_total(
                            TranscriptRow::new(TranscriptRowKind::Assistant, "Paddles", ""),
                            occurred_at,
                        );
                        self.rows.push(row);
                        self.pending_reveal = Some(PendingReveal::new(row_index, response));
                        self.busy = true;
                        self.busy_phase = BusyPhase::Rendering;
                    }
                    Err(error) => {
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

        if let Some(pending) = &mut self.pending_reveal {
            let finished = pending.advance();
            if let Some(row) = self.rows.get_mut(pending.row_index) {
                row.content = pending.visible_text();
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
        let input_height = self.input_area_height();
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
        let label = match self.busy_phase {
            BusyPhase::Thinking => "thinking",
            BusyPhase::Rendering => "rendering",
            _ => "working",
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
        let is_masked = self.is_masked_input();
        let input_style = self.palette.input_text;
        let input_text = self.input_display_text();
        let (label, hint) = self.input_label_and_hint(is_masked);
        let mut prompt_line = vec![Span::styled(
            label,
            self.palette.input_label.add_modifier(Modifier::BOLD),
        )];
        if !hint.is_empty() {
            prompt_line.push(Span::raw(" "));
            prompt_line.push(Span::styled(hint, self.palette.input_hint));
        }
        let lines = vec![
            Line::from(prompt_line),
            Line::from(Span::styled(input_text, input_style)),
        ];

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

    fn is_masked_input(&self) -> bool {
        matches!(self.input_mode, InputMode::MaskedKey { .. })
    }

    fn input_display_text(&self) -> String {
        if self.input.is_empty() {
            if self.is_masked_input() {
                "Paste or type your API key...".to_string()
            } else {
                "Type a prompt...".to_string()
            }
        } else if self.is_masked_input() {
            "\u{2022}".repeat(self.input.chars().count())
        } else {
            self.input.clone()
        }
    }

    fn input_label_and_hint(&self, is_masked: bool) -> (String, String) {
        if is_masked {
            (
                "API Key".to_string(),
                "Enter to save · Esc to cancel".to_string(),
            )
        } else {
            let queue_hint = if self.queued_prompts.is_empty() {
                None
            } else {
                Some(format!("{} queued", self.queued_prompts.len()))
            };
            let turn_hint = if self.busy {
                Some("Turn in progress".to_string())
            } else {
                None
            };
            let hint = match (turn_hint, queue_hint) {
                (Some(turn), Some(queue)) => format!("{turn} • {queue}"),
                (Some(turn), None) => turn,
                (None, Some(queue)) => queue,
                (None, None) => String::new(),
            };
            ("Prompt".to_string(), hint)
        }
    }

    fn cursor_position(&self, area: Rect) -> (u16, u16) {
        let x = area.x.saturating_add(1 + self.input.chars().count() as u16);
        let y = area.y.saturating_add(2);
        (x.min(area.right().saturating_sub(1)), y)
    }

    fn input_area_height(&self) -> u16 {
        2 + 2 // label + input + top/bottom border
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

fn trim_for_display(input: &str, limit: usize) -> String {
    if input.chars().count() <= limit {
        return input.to_string();
    }

    let kept = input.chars().take(limit).collect::<String>();
    format!("{}...", kept.trim_end())
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
            detail,
        } => {
            let elapsed = format_duration_compact(Duration::from_secs(elapsed_seconds));
            let content = detail.unwrap_or_default();
            TranscriptRow::new(
                TranscriptRowKind::Event,
                format!("• Searching ({phase}) — {elapsed}"),
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
        } => TranscriptRow::new(
            TranscriptRowKind::Event,
            format!("• Completed {tool_name}"),
            collapse_event_details(&summary, EVENT_DETAIL_LINE_LIMIT),
        ),
        TurnEvent::Fallback { stage, reason } => TranscriptRow::new(
            TranscriptRowKind::Event,
            "• Fell back",
            format!("{stage}: {reason}"),
        ),
        TurnEvent::ContextPressure { pressure } => {
            let factors: Vec<_> = pressure.factors.iter().map(|f| f.label()).collect();
            TranscriptRow::new(
                TranscriptRowKind::Event,
                format!("• Context pressure: {}", pressure.level.label()),
                format!(
                    "{} truncation(s), factors: [{}]",
                    pressure.truncation_count,
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
    error_header: Style,
    error_body: Style,
    input_label: Style,
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
            error_header: Style::default().fg(Color::Rgb(173, 38, 45)),
            error_body: Style::default().fg(Color::Rgb(99, 39, 44)),
            input_label: Style::default().fg(Color::Rgb(24, 63, 115)),
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
            error_header: Style::default().fg(Color::Rgb(255, 122, 132)),
            error_body: Style::default().fg(Color::Rgb(238, 183, 190)),
            input_label: Style::default().fg(Color::Rgb(125, 194, 255)),
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
        TranscriptRow, TranscriptRowKind, TranscriptTiming, TranscriptTimingKind,
        collapse_event_details, detect_palette, format_duration_compact, format_turn_event_row,
        inline_viewport_height_for_terminal, render_row_lines, select_interactive_frontend,
    };
    use crate::application::ConversationSession;
    use crate::domain::model::{TaskTraceId, TurnEvent};
    use crate::infrastructure::step_timing::Pace;
    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};
    use std::time::{Duration, Instant};

    fn session() -> ConversationSession {
        ConversationSession::new(TaskTraceId::new("task-1").expect("task"))
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
        let mut reveal = PendingReveal::new(0, "hello world".to_string());
        while !reveal.advance() {}

        assert_eq!(reveal.visible_text(), "hello world");
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
        // transcript (4 empty) + prompt box (4) + status bar (1) = 9
        // Prompt box border with title starts at line 4.
        assert!(buffer_line(&buffer, 4).contains("Prompt"));
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
        let user_rows = app.take_scrollback_rows();
        assert_eq!(user_rows.len(), 1);
        assert_eq!(user_rows[0].header, "You");

        app.dispatch_next_prompt();
        app.handle_message(super::UiMessage::TurnFinished {
            result: Ok("hi there".to_string()),
            occurred_at: Instant::now(),
        });

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

        let (label, hint) = app.input_label_and_hint(false);
        assert_eq!(label, "Prompt");
        assert!(hint.is_empty());
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

        assert_eq!(
            app.input_display_text(),
            "\u{2022}".repeat("sk-secret-123".chars().count())
        );

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
}
