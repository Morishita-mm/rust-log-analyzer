use std::sync::Arc;

use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};
use crossterm::event::{self, Event, KeyCode};
use futures_util::{StreamExt, lock::Mutex};
use std::io::{Write, stdout};
use tokio::time;

use crate::state::{AppState, InputMode};
use crate::tui;
use crate::types::{AggregatedStats, LOGS_CHANNEL, LogEntry, STATS_CHANNEL};

const TICK_RATE: u64 = 100;

pub async fn run() -> Result<()> {
    let app_state = Arc::new(Mutex::new(AppState::new()));

    let state_for_main_loop = app_state.clone();

    let mut terminal = tui::init()?;

    let app_result = async {
        let client = redis::Client::open("redis://redis/")?;
        let mut con = client.get_async_pubsub().await?;
        con.subscribe(LOGS_CHANNEL).await?;
        con.subscribe(STATS_CHANNEL).await?;
        let mut stream = con.on_message();

        let mut tick_rate = time::interval(std::time::Duration::from_millis(TICK_RATE));
        let mut event_stream = event::EventStream::new();

        loop {
            {
                let state = state_for_main_loop.lock().await;
                terminal.draw(|f| tui::ui(f, &state))?;
            }
            tokio::select! {
                _ = tick_rate.tick() => {
                    let state = state_for_main_loop.lock().await;
                    terminal.draw(|f| tui::ui(f, &state))?;
                }

                // キー入力イベントの処理
                Some(Ok(event)) = event_stream.next() => {
                    if let Event::Key(key) = event {
                        let mut state = state_for_main_loop.lock().await;

                        match state.input_mode {
                            InputMode::Normal =>  match key.code {                                
                                // 終了
                                KeyCode::Char('q') => {
                                    break;
                                }
                                // 上へスクロール
                                KeyCode::Up | KeyCode::Char('k') => {
                                    state.select_previous_log();
                                }
                                // 下へスクロール
                                KeyCode::Down | KeyCode::Char('j') => {
                                    state.select_next_log();
                                }
                                // 選択解除（最新のログ表示に戻る）
                                KeyCode::Esc => {
                                    state.unselect_log();
                                }
                                // 'i'キーで入力モードに入る
                                KeyCode::Char('i') => {
                                    state.input_mode = InputMode::Editing;
                                    state.start_editing();
                                }
                                // コピー処理
                                KeyCode::Char('c') => {
                                    handle_copy_action(&state);
                                }
                                _ => {}
                            },

                            InputMode::Editing => match key.code {
                                // Escでキャンセルして閲覧モードへ
                                KeyCode::Esc => {
                                    state.cancel_editing();
                                }
                                // Enterで確定して閲覧モードへ
                                KeyCode::Enter => {
                                    state.submit_editing();
                                }

                                // Backspaceキーで1文字削除
                                KeyCode::Backspace => {
                                    state.editing_text.pop();
                                }
                                // 文字キー入力で追加
                                KeyCode::Char(c) => {
                                    state.editing_text.push(c);
                                }
                                
                                _ => {}
                            }
                        }
                    }
                }

                // メッセージの受信処理
                Some(msg) = stream.next() => {
                    let channel_name = msg.get_channel_name();
                    if let Ok(payload) = msg.get_payload::<String>() {
                        let mut state = state_for_main_loop.lock().await;

                        match channel_name {
                            LOGS_CHANNEL => {
                                if let Ok(log_entry) = serde_json::from_str::<LogEntry>(&payload) {
                                    let target_text = format!("{} {} {}", log_entry.level, log_entry.service, log_entry.message);

                                    let should_add = if let Some(regex) = &state.filter_regex {
                                        regex.is_match(&target_text)
                                    } else {
                                        true
                                    };

                                    if should_add {
                                        state.add_log(log_entry);
                                    }
                                }
                            }
                            STATS_CHANNEL => {
                                if let Ok(stats_vec) = serde_json::from_str::<Vec<AggregatedStats>>(&payload) {
                                    if let Some(first_stats) = stats_vec.first() {
                                        state.update_stats(first_stats.clone());
                                    }
                                }
                            }
                            _ => {
                                println!("Received unknown message on channel: {}", channel_name);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }.await;

    // TUIの終了処理
    tui::restore()?;

    app_result
}

fn handle_copy_action(state: &AppState) {
    if let Some(index) = state.selected_log_index {
        if let Some(log) = state.logs.get(index) {
            if let Err(e) = copy_to_clipboard(&log.message) {
                eprintln!("Failed to copy to clipboard: {}", e);
            }
        }
    }
}

fn copy_to_clipboard(text: &str) -> Result<()> {
    // テキストをBase64にエンコード
    let encoded = general_purpose::STANDARD.encode(text);

    // OSC 52 エスケープシーケンスを構築
    // \x1b]52;c;{Base64文字列}\x07 という形式
    // これをターミナルが解釈してクリップボードに設定する
    let osc052_sequence = format!("\x1b]52;c;{}\x07", encoded);

    let mut out = stdout();
    write!(out, "{}", osc052_sequence)?;
    out.flush()?;

    Ok(())
}
