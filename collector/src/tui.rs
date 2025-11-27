use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use std::io::{Stdout, stdout};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// TUIの初期化：Rawモードへの移行、代替スクリーンの開始、ターミナルの設定
pub fn init() -> Result<Tui> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// TUIの終了処理：Rawモードの解除、代替スクリーンからの復帰
pub fn restore() -> Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

/// アプリケーションのUI全体を描画（仮実装）
/// TODO: アプリケーションの状態を受け取りそれを元に更新するように変更
pub fn ui(f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(12),
        ])
        .split(f.area());
    render_filter_pane(f, chunks[0]);
    render_logs_pane(f, chunks[1]);
    render_stats_pane(f, chunks[2]);
}

fn render_filter_pane(f: &mut Frame, area: Rect) {
    // 枠線とタイトル付きのブロックを作成
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Filter Input")
        .border_style(Style::default().fg(Color::Yellow));
    let text = Paragraph::new("Type regex filter here... (Press 'i' to edit)").block(block);
    f.render_widget(text, area);
}

fn render_logs_pane(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Real-time Logs")
        .border_style(Style::default().fg(Color::Blue));

    // 仮のログデータを複数行表示
    let text = vec![
        Line::from(vec![
            Span::styled("[INFO]", Style::default().fg(Color::Green)),
            Span::raw("auth-service: User login successful."),
        ]),
        Line::from(vec![
            Span::styled("[ERROR]", Style::default().fg(Color::Red)),
            Span::raw("do-service: Connection timeout."),
        ]),
        Line::from("... Waiting for incoming logs ..."),
    ];

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn render_stats_pane(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Real-time Logs")
        .border_style(Style::default().fg(Color::Magenta));

    let text = Paragraph::new(
        "Total Logs: 0\nError Rate: 0.0%\nTop Service: N/A"
    ).block(block);
    f.render_widget(text, area);
}
