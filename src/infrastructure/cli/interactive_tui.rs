use crate::application::{ConversationSession, MechSuitService, RuntimeLaneConfig};
use crate::domain::model::{ThreadCandidate, TurnEvent, TurnEventSink};
use crate::infrastructure::credentials::CredentialStore;
use anyhow::Result;
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use std::cmp;
use std::collections::VecDeque;
use std::io;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// Context passed from main to the TUI for credential management.
pub struct TuiContext {
    pub credential_store: Arc<CredentialStore>,
    pub api_key_shared: Arc<RwLock<String>>,
    pub runtime_lanes: RuntimeLaneConfig,
    pub provider_name: String,
    pub credential_provider: Option<String>,
    pub credential_status: String,
}

const FRAME_INTERVAL: Duration = Duration::from_millis(32);
const ASSISTANT_REVEAL_STEP: usize = 24;
const EVENT_DETAIL_LINE_LIMIT: usize = 8;

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
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let (tx, mut rx) = unbounded_channel();
    let session = service.create_conversation_session();
    let mut app = InteractiveApp::new(
        model_label.into(),
        detect_palette(),
        session.clone(),
        tui_ctx.provider_name.clone(),
        tui_ctx.credential_provider.clone(),
        tui_ctx.credential_status.clone(),
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
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.input.clear();
            false
        }
        KeyCode::Char(ch) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                app.input.push(ch);
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
        execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, cursor::Show);
    }
}

#[derive(Debug)]
enum UiMessage {
    TurnEvent(TurnEvent),
    TurnFinished(std::result::Result<String, String>),
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
        let _ = tx.send(UiMessage::TurnFinished(result));
    });
}

#[derive(Clone)]
struct InteractiveTurnEventSink {
    tx: UnboundedSender<UiMessage>,
}

impl TurnEventSink for InteractiveTurnEventSink {
    fn emit(&self, event: TurnEvent) {
        let _ = self.tx.send(UiMessage::TurnEvent(event));
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
}

impl TranscriptRow {
    fn new(kind: TranscriptRowKind, header: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            kind,
            header: header.into(),
            content: content.into(),
        }
    }

    fn estimated_height(&self, width: usize) -> usize {
        let width = width.max(8);
        let body_width = width.saturating_sub(4).max(1);
        let mut lines = wrapped_line_count(&self.header, width);
        if self.content.is_empty() {
            return lines + 1;
        }

        for line in self.content.lines() {
            lines += wrapped_line_count(line, body_width);
        }
        lines + 1
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
    ) -> Self {
        let ready_message = match credential_provider.as_deref() {
            Some(provider) => format!(
                "Codex-style transcript active. Enter to send, Ctrl+C to quit.\n\
                 {credential_status}\n\
                 Type `/login` to set or replace your API key for `{provider}`.",
            ),
            None => format!(
                "Codex-style transcript active. Enter to send, Ctrl+C to quit.\n\
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
        }
    }

    fn submit_prompt(&mut self) {
        let raw = self.input.trim().to_string();
        if raw.is_empty() {
            return;
        }
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
        if self.busy {
            return None;
        }

        let prompt = self.queued_prompts.pop_front()?;
        self.busy = true;
        self.busy_phase = BusyPhase::Thinking;
        Some(prompt)
    }

    fn handle_message(&mut self, message: UiMessage) {
        match message {
            UiMessage::TurnEvent(event) => {
                self.rows.push(format_turn_event_row(event));
            }
            UiMessage::TurnFinished(result) => match result {
                Ok(response) => {
                    let row_index = self.rows.len();
                    self.rows.push(TranscriptRow::new(
                        TranscriptRowKind::Assistant,
                        "Paddles",
                        "",
                    ));
                    self.pending_reveal = Some(PendingReveal::new(row_index, response));
                    self.busy = true;
                    self.busy_phase = BusyPhase::Rendering;
                }
                Err(error) => {
                    self.rows.push(TranscriptRow::new(
                        TranscriptRowKind::Error,
                        "• Turn failed",
                        error,
                    ));
                    self.busy = false;
                    self.busy_phase = BusyPhase::Idle;
                }
            },
        }
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
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(8),
                Constraint::Length(4),
            ])
            .split(frame.area());

        frame.render_widget(self.render_header(), layout[0]);
        frame.render_widget(self.render_transcript(layout[1]), layout[1]);
        frame.render_widget(self.render_input(), layout[2]);
        frame.set_cursor_position(self.cursor_position(layout[2]));
    }

    fn render_header(&self) -> Paragraph<'static> {
        let spinner = SPINNER_FRAMES[self.spinner_index];
        let active_thread = self.session.active_thread().thread_ref.stable_id();
        let status = match self.busy_phase {
            BusyPhase::Idle if self.queued_prompts.is_empty() => "idle".to_string(),
            BusyPhase::Idle => format!("idle · {} queued", self.queued_prompts.len()),
            BusyPhase::Thinking => format!("{spinner} thinking"),
            BusyPhase::Rendering => format!("{spinner} rendering"),
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

    fn render_transcript(&self, area: Rect) -> Paragraph<'static> {
        let inner_width = usize::from(area.width.saturating_sub(2).max(1));
        let inner_height = usize::from(area.height.saturating_sub(2).max(1));
        let visible_rows = self.visible_rows(inner_width, inner_height);
        let mut lines = Vec::new();

        for (index, row) in visible_rows.iter().enumerate() {
            lines.extend(render_row_lines(row, &self.palette));
            if index + 1 < visible_rows.len() {
                lines.push(Line::default());
            }
        }

        Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title(" Transcript ")
                    .borders(Borders::ALL)
                    .border_style(self.palette.border),
            )
            .wrap(Wrap { trim: false })
    }

    fn render_input(&self) -> Paragraph<'static> {
        let is_masked = self.is_masked_input();
        let input_style = self.palette.input_text;
        let input_text = self.input_display_text();
        let (label, hint) = self.input_label_and_hint(is_masked);
        let mut lines = vec![
            Line::from(vec![
                Span::styled(label, self.palette.input_label.add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(hint, self.palette.input_hint),
            ]),
            Line::from(Span::styled(input_text, input_style)),
        ];

        if self.busy && !is_masked {
            lines.push(Line::from(Span::styled(
                "Action events stream above while you can keep typing and queue steering prompts.",
                self.palette.input_hint,
            )));
        }

        Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title(" Composer ")
                    .borders(Borders::ALL)
                    .border_style(self.palette.border),
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
                Some("Enter to send".to_string())
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

    fn visible_rows(&self, width: usize, height: usize) -> Vec<TranscriptRow> {
        let mut visible = Vec::new();
        let mut used = 0;

        for row in self.rows.iter().rev() {
            let row_height = row.estimated_height(width);
            if !visible.is_empty() && used + row_height > height {
                break;
            }
            used += row_height;
            visible.push(row.clone());
        }

        visible.reverse();
        visible
    }
}

fn trim_for_display(input: &str, limit: usize) -> String {
    if input.chars().count() <= limit {
        return input.to_string();
    }

    let kept = input.chars().take(limit).collect::<String>();
    format!("{}...", kept.trim_end())
}

fn render_row_lines(row: &TranscriptRow, palette: &Palette) -> Vec<Line<'static>> {
    let (header_style, body_style) = match row.kind {
        TranscriptRowKind::User => (palette.user_header, palette.user_body),
        TranscriptRowKind::Assistant => (palette.assistant_header, palette.assistant_body),
        TranscriptRowKind::Event => (palette.event_header, palette.event_body),
        TranscriptRowKind::Error => (palette.error_header, palette.error_body),
    };

    let mut lines = vec![Line::from(Span::styled(
        row.header.clone(),
        header_style.add_modifier(Modifier::BOLD),
    ))];

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

fn format_turn_event_row(event: TurnEvent) -> TranscriptRow {
    match event {
        TurnEvent::IntentClassified { intent } => {
            TranscriptRow::new(TranscriptRowKind::Event, "• Classified", intent.label())
        }
        TurnEvent::InterpretationContext { summary, sources } => {
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
                "• Assembled interpretation context",
                content,
            )
        }
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
        } => TranscriptRow::new(
            TranscriptRowKind::Event,
            "• Selected planner action",
            format!(
                "step {sequence}: {}\nRationale: {}",
                collapse_event_details(&action, EVENT_DETAIL_LINE_LIMIT),
                collapse_event_details(&rationale, EVENT_DETAIL_LINE_LIMIT)
            ),
        ),
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
        } => TranscriptRow::new(
            TranscriptRowKind::Event,
            "• Reviewed planner trace",
            format!(
                "strategy={strategy}, mode={mode}, turns={turns}, steps={steps}, stop={}, active={}, branches={}, frontier={}",
                stop_reason.as_deref().unwrap_or("none"),
                active_branch_id.as_deref().unwrap_or("none"),
                branch_count
                    .map(|value| value.to_string())
                    .as_deref()
                    .unwrap_or("n/a"),
                frontier_count
                    .map(|value| value.to_string())
                    .as_deref()
                    .unwrap_or("n/a"),
            ),
        ),
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
    code: Style,
    citation: Style,
}

fn detect_palette() -> Palette {
    if terminal_uses_light_background() {
        Palette {
            header_title: Style::default().fg(Color::Rgb(24, 63, 115)),
            header_meta: Style::default().fg(Color::Rgb(78, 87, 103)),
            header_status: Style::default().fg(Color::Rgb(19, 120, 95)),
            border: Style::default().fg(Color::Rgb(132, 145, 165)),
            user_header: Style::default().fg(Color::Rgb(18, 74, 140)),
            user_body: Style::default().fg(Color::Rgb(35, 43, 54)),
            assistant_header: Style::default().fg(Color::Rgb(0, 120, 102)),
            assistant_body: Style::default().fg(Color::Rgb(24, 33, 45)),
            event_header: Style::default().fg(Color::Rgb(138, 87, 0)),
            event_body: Style::default().fg(Color::Rgb(72, 77, 84)),
            error_header: Style::default().fg(Color::Rgb(173, 38, 45)),
            error_body: Style::default().fg(Color::Rgb(99, 39, 44)),
            input_label: Style::default().fg(Color::Rgb(24, 63, 115)),
            input_text: Style::default().fg(Color::Rgb(35, 43, 54)),
            input_hint: Style::default().fg(Color::Rgb(109, 117, 129)),
            code: Style::default().fg(Color::Rgb(87, 56, 130)),
            citation: Style::default().fg(Color::Rgb(94, 66, 0)),
        }
    } else {
        Palette {
            header_title: Style::default().fg(Color::Rgb(125, 194, 255)),
            header_meta: Style::default().fg(Color::Rgb(155, 169, 187)),
            header_status: Style::default().fg(Color::Rgb(116, 225, 175)),
            border: Style::default().fg(Color::Rgb(84, 95, 114)),
            user_header: Style::default().fg(Color::Rgb(115, 197, 255)),
            user_body: Style::default().fg(Color::Rgb(224, 229, 236)),
            assistant_header: Style::default().fg(Color::Rgb(111, 231, 183)),
            assistant_body: Style::default().fg(Color::Rgb(234, 240, 247)),
            event_header: Style::default().fg(Color::Rgb(255, 202, 92)),
            event_body: Style::default().fg(Color::Rgb(182, 191, 204)),
            error_header: Style::default().fg(Color::Rgb(255, 122, 132)),
            error_body: Style::default().fg(Color::Rgb(238, 183, 190)),
            input_label: Style::default().fg(Color::Rgb(125, 194, 255)),
            input_text: Style::default().fg(Color::Rgb(236, 242, 250)),
            input_hint: Style::default().fg(Color::Rgb(145, 154, 168)),
            code: Style::default().fg(Color::Rgb(204, 171, 255)),
            citation: Style::default().fg(Color::Rgb(255, 216, 130)),
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
        TranscriptRow, TranscriptRowKind, collapse_event_details, detect_palette,
        format_turn_event_row, render_row_lines, select_interactive_frontend,
    };
    use crate::application::ConversationSession;
    use crate::domain::model::{TaskTraceId, TurnEvent};

    fn session() -> ConversationSession {
        ConversationSession::new(TaskTraceId::new("task-1").expect("task"))
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
        let row = format_turn_event_row(TurnEvent::ToolCalled {
            call_id: "tool-1".to_string(),
            tool_name: "shell".to_string(),
            invocation: "git status --short".to_string(),
        });

        assert_eq!(row.kind, TranscriptRowKind::Event);
        assert_eq!(row.header, "• Ran shell");
        assert_eq!(row.content, "git status --short");
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
        );
        app.input = "hello".to_string();

        app.submit_prompt();
        let prompt = app.dispatch_next_prompt();
        assert_eq!(prompt, Some(QueuedPrompt::Prompt("hello".to_string())));
        assert!(app.busy);
        assert_eq!(app.busy_phase, BusyPhase::Thinking);

        app.handle_message(super::UiMessage::TurnFinished(Ok("hi there".to_string())));
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
        );

        app.input = "first".to_string();
        app.submit_prompt();
        assert_eq!(
            app.dispatch_next_prompt(),
            Some(QueuedPrompt::Prompt("first".to_string()))
        );

        app.input = "second".to_string();
        app.submit_prompt();

        app.handle_message(super::UiMessage::TurnFinished(Ok("done".to_string())));
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
    fn login_command_enters_masked_mode_for_remote_provider() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new(
            "moonshot-v1".to_string(),
            palette,
            session(),
            "moonshot".to_string(),
            Some("moonshot".to_string()),
            "Provider: `moonshot`. Auth: loaded from the local credential store.".to_string(),
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
}
