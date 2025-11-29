use serde::{Deserialize, Serialize};

// Redisのチャンネル名定数
pub const LOGS_CHANNEL: &str = "logs.ingest";
pub const STATS_CHANNEL: &str = "stats.update";

// 生ログ
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub service: String,
    pub message: String,
}

// 集計結果
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AggregatedStats {
    pub window_start: String,
    pub window_end: String,
    pub total_count: u64,
    pub error_count: u64,
    pub top_service: Option<String>,
}
