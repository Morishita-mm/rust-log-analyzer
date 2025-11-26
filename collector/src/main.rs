use anyhow::Result;
use futures_util::StreamExt;
use redis::AsyncCommands;
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


#[tokio::main]
async fn main() -> Result<()> {
    println!("🦀 Rust Log Collector started...");

    // Docker内の 'redis' ホストへ接続
    let client = redis::Client::open("redis://redis/")?;
    let mut con = client.get_async_pubsub().await?;

    // 'logs.ingest' チャンネルを購読
    con.subscribe("logs.ingest").await?;
    println!("Listening on channel: 'logs.ingest'...");

    // ストリームとしてメッセージを処理
    let mut stream = con.on_message();
    
    while let Some(msg) = stream.next().await {
        let payload: String = msg.get_payload()?;

        // 文字列をLogEntry構造体にパース
        match serde_json::from_str::<LogEntry>(&payload) {
            Ok(log_entry) => {
                println!("✅ Parsed LogEntry: {:?}", log_entry);
                // TODO: ここに将来、TUIのログ画面用データストアに追加する
            }
            Err(e) => {
                eprintln!("❌ Failed to parse log entry: {}", e);
                eprintln!("   Payload was: {}", payload
);
            }
        }
        
        // TODO: ここに将来、TUIへの描画処理が入る
    }

    Ok(())
}