use std::sync::Arc;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use futures_util::{StreamExt, lock::Mutex};
use tokio::time;

use crate::state::AppState;
use crate::tui;
use crate::types::{AggregatedStats, LOGS_CHANNEL, LogEntry, STATS_CHANNEL};

const TICK_RATE: u64 = 100;

pub async fn run() -> Result<()> {
    // 共有するアプリケーションの状態
    let app_state = Arc::new(Mutex::new(AppState::new()));
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
                                    state.add_log(log_entry);
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
