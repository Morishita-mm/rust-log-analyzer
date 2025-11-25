use anyhow::Result;
use futures_util::StreamExt;
use redis::AsyncCommands;

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
        println!("Received: {}", payload);
        
        // ここに将来、TUIへの描画処理が入る
    }

    Ok(())
}