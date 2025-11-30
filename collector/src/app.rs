use std::sync::Arc;

use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use futures_util::{StreamExt, lock::Mutex};
use std::io::{Write, stdout};
use tokio::time;

use crate::state::AppState;
use crate::tui;
use crate::types::{AggregatedStats, LOGS_CHANNEL, LogEntry, STATS_CHANNEL};

const TICK_RATE: u64 = 100;

pub async fn run() -> Result<()> {
    // 共有するアプリケーションの状態
    let app_state = Arc::new(Mutex::new(AppState::new()));

    // ログフィルタリングの動作確認
    // app_state.lock().await.set_filter("auth-service".to_string());

    let state_for_main_loop = app_state.clone();

    // TUIの初期化
    let mut terminal = tui::init()?;

    // Docker内の 'redis' ホストへ接続
    let app_result = async {
        let client = redis::Client::open("redis://redis/")?;
        let mut con = client.get_async_pubsub().await?;
        con.subscribe(LOGS_CHANNEL).await?;
        con.subscribe(STATS_CHANNEL).await?;
        let mut stream = con.on_message();

        let mut tick_rate = time::interval(std::time::Duration::from_millis(TICK_RATE));
        let mut event_stream = event::EventStream::new();

        loop {
            tokio::select! {
                // 定期的な描画タイミング
                _ = tick_rate.tick() => {
                    let state = state_for_main_loop.lock().await;
                    // ここで描画関数を呼び出す
                    terminal.draw(|f| tui::ui(f, &state))?;
                }

                // キー入力イベントの処理
                Some(Ok(event)) = event_stream.next() => {
                    if let Event::Key(key) = event {
                        let mut state = state_for_main_loop.lock().await;

                        match key.code {
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
                            // コピー処理
                            KeyCode::Char('c') => {
                                // 選択中のログのインデックスを取得
                                if let Some(index) = state.selected_log_index {
                                    // インデックスを使ってログデータを取得
                                    if let Some(log) = state.logs.get(index) {
                                        // ログのメッセージ部分をクリップボードにコピー
                                        if let Err(e) = copy_to_clipboard(&log.message) {
                                            eprintln!("Failed to copy to clipboard: {}", e);
                                        } else {
                                            // TODO: コピー完了を伝える仕組みを作成する
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                // メッセージの受信処理
                Some(msg) = stream.next() => {
                    let channel_name = msg.get_channel_name();
                    if let Ok(payload) = msg.get_payload::<String>() {
                        // 共有状態のロックを非同期に取得
                        let mut state = state_for_main_loop.lock().await;

                        match channel_name {
                            LOGS_CHANNEL => {
                                if let Ok(log_entry) = serde_json::from_str::<LogEntry>(&payload) {
                                    // フィルタリング対象の文字列
                                    let target_text = format!("{} {} {}", log_entry.level, log_entry.service, log_entry.message);

                                    // フィルタが設定されているかチェック
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

fn copy_to_clipboard(text: &str) -> Result<()> {
    // テキストをBase64にエンコード
    let encoded = general_purpose::STANDARD.encode(text);

    // OSC 52 エスケープシーケンスを構築
    // \x1b]52;c;{Base64文字列}\x07 という形式
    // これをターミナルが解釈してクリップボードに設定する
    let osc052_sequence = format!("\x1b]52;c;{}\x07", encoded);

    // 標準出力に書き出す
    let mut out = stdout();
    write!(out, "{}", osc052_sequence)?;
    out.flush()?;

    Ok(())
}