mod tui;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::time;

#[derive(Serialize, Deserialize, Debug)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AggregatedStats {
    window_start: String,
    window_end: String,
    total_count: u64,
    error_count: u64,
    top_service: Option<String>, // ログが0件の場合はサービス名がないのでOption
}

const LOGS_CHANNEL: &str = "logs.ingest";
const STATS_CHANNEL: &str = "stats.update";
const TICK_RATE: u64 = 100;

#[tokio::main]
async fn main() -> Result<()> {
    // TUIの初期化
    let mut terminal = tui::init()?;

    // Docker内の 'redis' ホストへ接続
    let app_result = async {
        let client = redis::Client::open("redis://redis/")?;
        let mut con = client.get_async_pubsub().await?;
    
        // 複数のチャンネルを購読する
        con.subscribe(LOGS_CHANNEL).await?;
        con.subscribe(STATS_CHANNEL).await?;
        // println!("Listening on channel: '{LOGS_CHANNEL}' and '{STATS_CHANNEL}'...");
    
        // ストリームとしてメッセージを処理
        let mut stream = con.on_message();
        
        // 描画の更新間隔
        let mut tick_rate = time::interval(std::time::Duration::from_millis(TICK_RATE));

        // キー入力イベント監視用のストリームを作成
        let mut event_stream = event::EventStream::new();
    
        loop {
            tokio::select! {
                // 定期的な描画タイミング
                _ = tick_rate.tick() => {
                    // ここで描画関数を呼び出す
                    terminal.draw(|f| tui::ui(f))?;
                }

                // キー入力イベントの処理
                Some(Ok(event)) = event_stream.next() => {
                    // キーが「押された」時で、かつキーコードが 'q' の場合
                    if let Event::Key(key) = event {
                        if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                            break;
                        }
                    }
                    // TODO: 他のキー操作についてもここ、もしくは別ファイルに分割して追加する
                },

                // メッセージの受信処理
                Some(msg) = stream.next() => {
                    let channel_name = msg.get_channel_name();
                    if let Ok(payload) = msg.get_payload::<String>() {
                        match channel_name {
                            LOGS_CHANNEL => {
                                if let Ok(_log_entry) = serde_json::from_str::<LogEntry>(&payload) {
                                    // TODO: 状態の更新処理の実装
                                }
                            }
                            STATS_CHANNEL => {
                                if let Ok(_stats_vec) = serde_json::from_str::<Vec<AggregatedStats>>(&payload) {
                                    // TODO: 状態の更新処理
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
