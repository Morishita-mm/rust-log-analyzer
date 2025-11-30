use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Modifier, Style},
    symbols::border,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io::{Stdout, stdout};

use crate::state::AppState;
use crate::state::InputMode;
use crate::types::AggregatedStats;

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> Result<Tui> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn restore() -> Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

/// アプリケーションのUI全体を描画（仮実装）
pub fn ui(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(12),
        ])
        .split(f.area());

    render_filter_pane(f, chunks[0], &state);
    render_logs_pane(f, chunks[1], &state);
    render_stats_pane(f, chunks[2], &state.latest_stats);
}

fn render_filter_pane(f: &mut Frame, area: Rect, state: &AppState) {
    let border_style = match state.input_mode {
        InputMode::Editing => Style::default().fg(Color::Green),
        InputMode::Normal => Style::default().fg(Color::Yellow),
    };

    // 枠線とタイトル付きのブロックを作成
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Filter Input")
        .border_style(border_style);

    let text_content = if state.filter_text.is_empty() {
        Span::styled(
            "Type regex filter here... (Press 'i' to enter editing mode)",
            Style::default().add_modifier(Modifier::DIM), // 薄い色で表示
        )
    } else {
        // 入力されたテキストをそのまま表示
        // TODO: カーソル表示などの追加
        Span::raw(&state.filter_text)
    };

    let paragraph = Paragraph::new(text_content).block(block);
    f.render_widget(paragraph, area);
}

fn render_logs_pane(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Real-time Logs ({} items)", state.logs.len()))
        .border_style(Style::default().fg(Color::Blue));

    let items: Vec<ListItem> = state
        .logs
        .iter()
        .enumerate()
        .map(|(i, log)| {
            let level_style = match log.level.as_str() {
                "ERROR" => Style::default().fg(Color::Red),
                "WARN" => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::Green),
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:>3}: ", i),
                    Style::default().add_modifier(Modifier::DIM),
                ),
                Span::raw("["),
                Span::raw(&log.timestamp),
                Span::raw("] ["),
                Span::styled(&log.level, level_style),
                Span::raw("] "),
                Span::raw(": "),
                Span::raw(&log.message),
            ]);
            ListItem::new(line)
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(state.selected_log_index);

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut list_state);
}

fn render_stats_pane(f: &mut Frame, area: Rect, stats: &Option<AggregatedStats>) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Statistics (1s Window)")
        .border_style(Style::default().fg(Color::Magenta));

    let text_content = match stats {
        Some(s) => format!(
            "Window Start: {}\nWindow End:   {}\n\nTotal Logs:   {:>5}\nError Count:  {:>5}\nTop Service:  {}",
            s.window_start,
            s.window_end,
            s.total_count,
            s.error_count,
            s.top_service.as_deref().unwrap_or("N/A")
        ),
        None => "Waiting for statistics data...".to_string(),
    };

    let text = Paragraph::new(text_content).block(block);
    f.render_widget(text, area);
}
