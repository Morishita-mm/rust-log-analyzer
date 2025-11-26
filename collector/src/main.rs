use anyhow::Result;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

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
    top_service: Option<String>,    // ログが0件の場合はサービス名がないのでOption
}

const LOGS_CHANNEL: &str = "logs.ingest";
const STATS_CHANNEL: &str = "stats.update";


#[tokio::main]
async fn main() -> Result<()> {
    println!("🦀 Rust Log Collector started...");

    // Docker内の 'redis' ホストへ接続
    let client = redis::Client::open("redis://redis/")?;
    let mut con = client.get_async_pubsub().await?;

    // 複数のチャンネルを購読する
    con.subscribe(LOGS_CHANNEL).await?;
    con.subscribe(STATS_CHANNEL).await?;
    println!("Listening on channel: '{LOGS_CHANNEL}' and '{STATS_CHANNEL}'...");

    // ストリームとしてメッセージを処理
    let mut stream = con.on_message();
    
    while let Some(msg) = stream.next().await {
        // メッセージがどのチャンネルから来たかを取得
        let channel_name = msg.get_channel_name();
        let payload: String = msg.get_payload()?;

        match channel_name {
            // 生ログの場合
            LOGS_CHANNEL => {
                match serde_json::from_str::<LogEntry>(&payload) {
                    Ok (log_entry) => {
                        println!("[LOG] ✅ Received: {} - {}", log_entry.timestamp, log_entry.message);
                        // TODO: TUIのログ画面要データストアに追加
                    }
                    Err(e) => eprintln!("[LOG] ❌ Parse error: {}", e),
                }
            }
            // 集計結果の場合
            STATS_CHANNEL => {
                // Polarsからリスト形式で送信されるので、Vec<AggregatedStats>で受け取る必要がある
                match serde_json::from_str::<Vec<AggregatedStats>>(&payload) {
                    Ok(stats_vec) => {
                        // 通常は1要素のリストが来る想定
                        if let Some(stats) = stats_vec.first() {
                            println!("  [STAT] 📊 Updated: Time={} | Total={} | Eror={}",
                        stats.window_start, stats.total_count, stats.error_count);
                        println!("          -> Top Service: {:?}", stats.top_service);

                        // TODO: TUIの統計画面用データストアを更新
                        }
                    }
                    Err(e) => {
                        eprintln!("[STAT] ❌ Parse error: {e}");
                        eprintln!("         Payload: {payload}");
                    }
                }
            }
            _ => {
                println!("Received unknown message on channel: {}", channel_name);
            }
        }
        
        // TODO: ここに将来、TUIへの描画処理が入る
    }

    Ok(())
}