use crate::application::MechSuitService;
use crate::domain::model::{TurnEvent, TurnEventSink};
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
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

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
) -> Result<()> {
    let _terminal_session = TerminalSession::enter()?;
    let backend = ratatui::backend::CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let (tx, mut rx) = unbounded_channel();
    let mut app = InteractiveApp::new(model_label.into(), detect_palette());

    loop {
        drain_messages(&mut app, &mut rx);
        app.tick();
        if let Some(prompt) = app.dispatch_next_prompt() {
            dispatch_prompt(prompt, Arc::clone(&service), tx.clone());
        }

        terminal.draw(|frame| app.render(frame))?;

        if event::poll(FRAME_INTERVAL)? {
            match event::read()? {
                Event::Key(key) if key.kind != KeyEventKind::Release => {
                    if handle_key_event(&mut app, key, Arc::clone(&service), tx.clone()) {
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
    _tx: UnboundedSender<UiMessage>,
) -> bool {
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
        return true;
    }

    match key.code {
        KeyCode::Esc => true,
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

fn dispatch_prompt(prompt: String, service: Arc<MechSuitService>, tx: UnboundedSender<UiMessage>) {
    let sink = Arc::new(InteractiveTurnEventSink { tx: tx.clone() });
    tokio::spawn(async move {
        let result = service
            .process_prompt_with_sink(&prompt, sink)
            .await
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
    rows: Vec<TranscriptRow>,
    input: String,
    queued_prompts: VecDeque<String>,
    busy: bool,
    busy_phase: BusyPhase,
    pending_reveal: Option<PendingReveal>,
    spinner_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BusyPhase {
    Idle,
    Thinking,
    Rendering,
}

impl InteractiveApp {
    fn new(model_label: String, palette: Palette) -> Self {
        Self {
            model_label,
            palette,
            rows: vec![TranscriptRow::new(
                TranscriptRowKind::Event,
                "• Interactive mode ready",
                "Codex-style transcript active. Enter to send, Ctrl+C to quit.",
            )],
            input: String::new(),
            queued_prompts: VecDeque::new(),
            busy: false,
            busy_phase: BusyPhase::Idle,
            pending_reveal: None,
            spinner_index: 0,
        }
    }

    fn submit_prompt(&mut self) {
        let prompt = self.input.trim().to_string();
        if prompt.is_empty() {
            return;
        }

        let was_busy = self.busy || self.pending_reveal.is_some();
        self.rows
            .push(TranscriptRow::new(TranscriptRowKind::User, "You", &prompt));
        self.input.clear();
        self.queued_prompts.push_back(prompt.clone());
        if was_busy || self.queued_prompts.len() > 1 {
            self.rows.push(TranscriptRow::new(
                TranscriptRowKind::Event,
                "• Queued steering prompt",
                format!(
                    "`{}` queued behind the active turn.",
                    trim_for_display(&prompt, 96)
                ),
            ));
        }
    }

    fn dispatch_next_prompt(&mut self) -> Option<String> {
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
            Span::styled(status, self.palette.header_status),
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
        let input_style = self.palette.input_text;
        let input_text = if self.input.is_empty() {
            "Type a prompt...".to_string()
        } else {
            self.input.clone()
        };
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
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    "Prompt",
                    self.palette.input_label.add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(hint, self.palette.input_hint),
            ]),
            Line::from(Span::styled(input_text, input_style)),
        ];

        if self.busy {
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
        BusyPhase, InteractiveApp, InteractiveFrontend, PendingReveal, TranscriptRow,
        TranscriptRowKind, collapse_event_details, detect_palette, format_turn_event_row,
        render_row_lines, select_interactive_frontend,
    };
    use crate::domain::model::TurnEvent;

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
        let mut app = InteractiveApp::new("qwen-1.5b".to_string(), palette);
        app.input = "hello".to_string();

        app.submit_prompt();
        let prompt = app.dispatch_next_prompt();
        assert_eq!(prompt.as_deref(), Some("hello"));
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
        let mut app = InteractiveApp::new("qwen-1.5b".to_string(), palette);

        app.input = "first".to_string();
        app.submit_prompt();
        assert_eq!(app.dispatch_next_prompt().as_deref(), Some("first"));
        assert!(app.busy);

        app.input = "steer harder".to_string();
        app.submit_prompt();

        assert_eq!(app.input, "");
        assert_eq!(app.queued_prompts.len(), 1);
        assert_eq!(
            app.queued_prompts.front().map(String::as_str),
            Some("steer harder")
        );
        assert!(
            app.rows
                .iter()
                .any(|row| row.header == "• Queued steering prompt")
        );
    }

    #[test]
    fn queued_prompt_dispatches_after_current_turn_finishes() {
        let palette = detect_palette();
        let mut app = InteractiveApp::new("qwen-1.5b".to_string(), palette);

        app.input = "first".to_string();
        app.submit_prompt();
        assert_eq!(app.dispatch_next_prompt().as_deref(), Some("first"));

        app.input = "second".to_string();
        app.submit_prompt();

        app.handle_message(super::UiMessage::TurnFinished(Ok("done".to_string())));
        while app.busy {
            app.tick();
        }

        assert_eq!(app.dispatch_next_prompt().as_deref(), Some("second"));
        assert!(app.busy);
        assert_eq!(app.busy_phase, BusyPhase::Thinking);
    }
}
