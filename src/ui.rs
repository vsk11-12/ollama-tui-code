use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{App, Mode, Role};

// ── Palette (Claude brand) ────────────────────────────────────────────────────

const C_ORANGE:    Color = Color::Rgb(207, 109,  65);   // Claude brand orange
const C_WHITE:     Color = Color::Rgb(235, 235, 235);   // near-white for user text
const C_BODY:      Color = Color::Rgb(208, 202, 196);   // slightly warm for assistant text
const C_DIM:       Color = Color::Rgb(115, 115, 115);   // secondary labels
const C_SUBDIM:    Color = Color::Rgb(52,  52,  52 );   // separators / hints
const C_BG:        Color = Color::Rgb(12,  12,  12 );   // page background
const C_HEADER_BG: Color = Color::Rgb(18,  18,  18 );   // header strip
const C_INPUT_BG:  Color = Color::Rgb(16,  16,  16 );   // input box — slightly raised
const C_BORDER:    Color = Color::Rgb(45,  45,  45 );   // resting border
const C_BORDER_HI: Color = Color::Rgb(68,  68,  68 );   // ready-state border (slightly brighter)
const C_SEL_BG:    Color = Color::Rgb(42,  28,  18 );   // warm selection highlight
const C_ERR:       Color = Color::Rgb(215,  72,  72);
const C_WARN:      Color = Color::Rgb(195, 155,  70);
const C_GREEN:     Color = Color::Rgb(80,  185, 120);

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn render(f: &mut Frame, app: &App) {
    let area = f.size();

    // Paint the entire background first
    f.render_widget(
        Block::default().style(Style::default().bg(C_BG)),
        area,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header bar
            Constraint::Min(3),    // message history
            Constraint::Length(3), // input box
            Constraint::Length(1), // key-hint footer
        ])
        .split(area);

    render_header(f, app, chunks[0]);
    render_messages(f, app, chunks[1]);
    render_input(f, app, chunks[2]);
    render_footer(f, chunks[3]);

    if app.mode == Mode::ModelSelect {
        render_model_popup(f, app);
    }
}

// ── Header ────────────────────────────────────────────────────────────────────

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    // Split: left = branding, right = model + status (right-aligned)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(52)])
        .split(area);

    // Left: ◆  ollama-tui-code
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("◆", Style::default().fg(C_ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled("  ollama-tui-code", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
        ]))
        .style(Style::default().bg(C_HEADER_BG)),
        cols[0],
    );

    // Status indicator
    let (dot, dot_color) = if app.status.starts_with('⚠') || app.status.starts_with("Error") {
        ("●", C_ERR)
    } else if app.is_loading {
        ("◌", C_WARN)
    } else if matches!(app.current_model.as_str(), "none" | "disconnected" | "Loading…") {
        ("○", C_DIM)
    } else {
        ("●", C_GREEN)
    };

    let model  = truncate_str(&app.current_model, 18);
    let status = truncate_str(&app.status, 22);

    // Right: model  ·  ● status  (right-aligned)
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(model, Style::default().fg(C_DIM)),
            Span::styled("  ·  ", Style::default().fg(C_SUBDIM)),
            Span::styled(dot, Style::default().fg(dot_color)),
            Span::raw("  "),
            Span::styled(status, Style::default().fg(C_DIM)),
            Span::raw("  "),
        ]))
        .alignment(Alignment::Right)
        .style(Style::default().bg(C_HEADER_BG)),
        cols[1],
    );
}

// ── Messages ──────────────────────────────────────────────────────────────────

fn render_messages(f: &mut Frame, app: &App, area: Rect) {
    // 4 cols left indent + 2 right margin
    let wrap_width = area.width.saturating_sub(6) as usize;

    let mut lines: Vec<Line<'static>> = vec![Line::from("")];

    if app.messages.is_empty() {
        push_welcome(&mut lines, &app.current_model);
    } else {
        for msg in &app.messages {
            push_message(&mut lines, msg, &app.current_model, wrap_width);
        }
    }

    let total      = lines.len() as u16;
    let visible    = area.height;
    let max_scroll = total.saturating_sub(visible);
    let scroll     = app.scroll.min(max_scroll);

    f.render_widget(
        Paragraph::new(lines)
            .scroll((scroll, 0))
            .style(Style::default().bg(C_BG)),
        area,
    );
}

// ── Welcome screen ────────────────────────────────────────────────────────────

fn push_welcome(lines: &mut Vec<Line<'static>>, current_model: &str) {
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Logo mark
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("◆", Style::default().fg(C_ORANGE).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("ollama-tui-code", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("via  {}", current_model),
            Style::default().fg(C_DIM),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "──────────────────────────────────────────",
            Style::default().fg(C_SUBDIM),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("Start typing to begin a conversation.", Style::default().fg(C_DIM)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "Keyboard shortcuts",
            Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    for (key, desc) in &[
        ("Enter",   "Send message"),
        ("Ctrl+M",  "Switch model"),
        ("Ctrl+L",  "Clear chat"),
        ("↑ / ↓",   "Scroll messages"),
        ("PgUp/Dn", "Page up / down"),
        ("Ctrl+C",  "Quit"),
    ] {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("[", Style::default().fg(C_SUBDIM)),
            Span::styled(key.to_string(), Style::default().fg(C_ORANGE)),
            Span::styled("]", Style::default().fg(C_SUBDIM)),
            Span::styled(format!("  {}", desc), Style::default().fg(C_DIM)),
        ]));
    }

    lines.push(Line::from(""));
}

// ── Message rendering ─────────────────────────────────────────────────────────

fn push_message(
    lines: &mut Vec<Line<'static>>,
    msg: &crate::app::Message,
    current_model: &str,
    wrap_width: usize,
) {
    match msg.role {
        // ── User ─────────────────────────────────────────────────────────────
        Role::User => {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "You",
                    Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD),
                ),
            ]));
            for line in wrap_text(&msg.content, wrap_width) {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(line, Style::default().fg(C_WHITE)),
                ]));
            }
            // Single blank after user turn; assistant header follows immediately
            lines.push(Line::from(""));
        }

        // ── Assistant ─────────────────────────────────────────────────────────
        Role::Assistant => {
            // "◆ ollama-tui-code  ·  model" header
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "◆ ollama-tui-code",
                    Style::default().fg(C_ORANGE).add_modifier(Modifier::BOLD),
                ),
                Span::styled("  ·  ", Style::default().fg(C_SUBDIM)),
                Span::styled(
                    truncate_str(current_model, 30),
                    Style::default().fg(C_SUBDIM),
                ),
            ]));

            let content_lines = wrap_text(&msg.content, wrap_width);
            let is_blank = content_lines.is_empty()
                || (content_lines.len() == 1 && content_lines[0].is_empty());

            if is_blank && msg.is_streaming {
                // Waiting for first token — show blinking cursor placeholder
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled("▊", Style::default().fg(C_ORANGE)),
                ]));
            } else {
                let last_i = content_lines.len().saturating_sub(1);
                for (i, line) in content_lines.into_iter().enumerate() {
                    let is_last = i == last_i;
                    let mut spans: Vec<Span<'static>> = vec![Span::raw("  ")];
                    spans.push(Span::styled(line, Style::default().fg(C_BODY)));
                    if is_last && msg.is_streaming {
                        spans.push(Span::styled("▊", Style::default().fg(C_ORANGE)));
                    }
                    lines.push(Line::from(spans));
                }
            }

            // After a completed turn: draw a thin separator, then a blank.
            // While still streaming, just a blank — separator appears when done.
            if !msg.is_streaming {
                lines.push(Line::from(""));
                let sep = "─".repeat(wrap_width.min(50));
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(sep, Style::default().fg(C_SUBDIM)),
                ]));
            }
            lines.push(Line::from(""));
        }
    }
}

// ── Input box ─────────────────────────────────────────────────────────────────

fn render_input(f: &mut Frame, app: &App, area: Rect) {
    let is_ready = !app.is_loading
        && !matches!(
            app.current_model.as_str(),
            "none" | "disconnected" | "Loading…"
        );

    // Border colour encodes the current state at a glance
    let border_color = if app.status.starts_with('⚠') || app.status.starts_with("Error") {
        C_ERR
    } else if app.is_loading {
        C_WARN
    } else if is_ready {
        C_BORDER_HI
    } else {
        Color::Rgb(28, 28, 28)
    };

    let prompt_color = if is_ready { C_ORANGE } else { C_DIM };
    let text_color   = if is_ready { C_WHITE  } else { C_DIM  };

    // Model name pinned to bottom-right of the input border.
    // title_bottom in ratatui 0.26 takes Into<Line>, so we fake right-alignment
    // by left-padding the label to fill the inner width (area - 2 border cols).
    let model_str   = truncate_str(&app.current_model, 22);
    let label_inner = format!(" {} ", model_str);
    let inner_w     = area.width.saturating_sub(2) as usize; // subtract L+R borders
    let pad         = inner_w.saturating_sub(label_inner.chars().count());
    let model_label = format!("{}{}", " ".repeat(pad), label_inner);

    let block = Block::default()
        .title_bottom(Line::from(
            Span::styled(model_label, Style::default().fg(C_SUBDIM)),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(C_INPUT_BG));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // "> " prompt (2 cols)
    const PROMPT: &str = "> ";
    const PROMPT_COLS: u16 = 2;

    let avail = inner.width.saturating_sub(PROMPT_COLS) as usize;

    // Horizontal scroll to keep cursor visible
    let h_off = if app.cursor_pos >= avail {
        app.cursor_pos.saturating_sub(avail) + 1
    } else {
        0
    };

    let display_text: String = app.input.chars().skip(h_off).take(avail).collect();
    let display_cursor = app.cursor_pos.saturating_sub(h_off);

    let placeholder = if app.is_loading {
        "Waiting for response…"
    } else if !is_ready {
        "Press Ctrl+M to select a model"
    } else {
        "Type a message…"
    };

    let text_span = if display_text.is_empty() {
        Span::styled(placeholder.to_string(), Style::default().fg(C_SUBDIM))
    } else {
        Span::styled(display_text, Style::default().fg(text_color))
    };

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(PROMPT.to_string(), Style::default().fg(prompt_color)),
            text_span,
        ]))
        .style(Style::default().bg(C_INPUT_BG)),
        inner,
    );

    // Place the OS cursor for IME / selection
    if is_ready {
        #[allow(deprecated)]
        f.set_cursor(
            inner.x + PROMPT_COLS + display_cursor as u16,
            inner.y,
        );
    }
}

// ── Footer ────────────────────────────────────────────────────────────────────

fn render_footer(f: &mut Frame, area: Rect) {
    let hints: &[(&str, &str)] = &[
        ("Enter",   "send"),
        ("Ctrl+M",  "model"),
        ("↑↓",      "scroll"),
        ("PgUp/Dn", "page"),
        ("Ctrl+L",  "clear"),
        ("Ctrl+C",  "quit"),
    ];

    let mut spans: Vec<Span<'static>> = vec![Span::raw("  ")];
    let bracket_color = Color::Rgb(38, 38, 38);

    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("   ", Style::default()));
        }
        spans.push(Span::styled("[", Style::default().fg(bracket_color)));
        spans.push(Span::styled(key.to_string(), Style::default().fg(C_DIM)));
        spans.push(Span::styled("]", Style::default().fg(bracket_color)));
        spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(C_SUBDIM),
        ));
    }

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(C_BG)),
        area,
    );
}

// ── Model picker popup ────────────────────────────────────────────────────────

fn render_model_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 65, f.size());
    f.render_widget(Clear, area);

    if app.models.is_empty() {
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled("No models installed.", Style::default().fg(C_WARN)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled("Run:  ", Style::default().fg(C_DIM)),
                    Span::styled(
                        "ollama pull llama3.2",
                        Style::default().fg(C_ORANGE),
                    ),
                ]),
            ])
            .block(popup_block(0)),
            area,
        );
        return;
    }

    let items: Vec<ListItem> = app
        .models
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let is_current  = *name == app.current_model;
            let is_selected = i == app.model_list_selected;

            let marker = if is_current { "◆  " } else { "   " };
            let label  = format!("  {}{}", marker, name);

            let style = if is_selected {
                Style::default()
                    .fg(C_ORANGE)
                    .bg(C_SEL_BG)
                    .add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default().fg(C_ORANGE)
            } else {
                Style::default().fg(C_WHITE)
            };

            ListItem::new(label).style(style)
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.model_list_selected));

    f.render_stateful_widget(
        List::new(items).block(popup_block(app.models.len())),
        area,
        &mut state,
    );
}

fn popup_block(count: usize) -> Block<'static> {
    let title_text = if count == 0 {
        " Select Model ".to_string()
    } else {
        format!(" Select Model  ({} installed) ", count)
    };

    Block::default()
        .title(Span::styled(
            title_text,
            Style::default().fg(C_ORANGE).add_modifier(Modifier::BOLD),
        ))
        .title_bottom(Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓]", Style::default().fg(C_DIM)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBDIM)),
            Span::styled("[Enter]", Style::default().fg(C_DIM)),
            Span::styled(" select  ", Style::default().fg(C_SUBDIM)),
            Span::styled("[Esc]", Style::default().fg(C_DIM)),
            Span::styled(" close ", Style::default().fg(C_SUBDIM)),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(C_BORDER))
        .style(Style::default().bg(Color::Rgb(16, 16, 16)))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn truncate_str(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        format!("{}…", s.chars().take(max - 1).collect::<String>())
    } else {
        s.to_string()
    }
}

/// Return a `Rect` centred in `r` with the given percentage dimensions.
fn centered_rect(pct_x: u16, pct_y: u16, r: Rect) -> Rect {
    let pad_y = (100 - pct_y) / 2;
    let pad_x = (100 - pct_x) / 2;

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(pad_y),
            Constraint::Percentage(pct_y),
            Constraint::Percentage(pad_y),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(pad_x),
            Constraint::Percentage(pct_x),
            Constraint::Percentage(pad_x),
        ])
        .split(vert[1])[1]
}

/// Word-wrap `text` to `max_width` columns, preserving explicit newlines.
/// Long words that exceed `max_width` are hard-broken.
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut result: Vec<String> = Vec::new();

    for paragraph in text.split('\n') {
        let mut current = String::new();
        let mut cur_len: usize = 0;

        for word in paragraph.split_whitespace() {
            let wlen = word.chars().count();

            if cur_len == 0 {
                if wlen <= max_width {
                    current.push_str(word);
                    cur_len = wlen;
                } else {
                    // Hard-break oversized word
                    let chars: Vec<char> = word.chars().collect();
                    let mut i = 0;
                    while i < chars.len() {
                        let end   = (i + max_width).min(chars.len());
                        let chunk: String = chars[i..end].iter().collect();
                        if end == chars.len() {
                            current = chunk;
                            cur_len = end - i;
                        } else {
                            result.push(chunk);
                        }
                        i = end;
                    }
                }
            } else if cur_len + 1 + wlen > max_width {
                result.push(current.clone());
                current = word.to_string();
                cur_len = wlen;
            } else {
                current.push(' ');
                current.push_str(word);
                cur_len += 1 + wlen;
            }
        }

        result.push(current);
    }

    if result.is_empty() {
        result.push(String::new());
    }

    result
}
