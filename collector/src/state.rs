use std::collections::VecDeque;
// TODO: type.rsに型定義モジュールを移動させる
use crate::{AggregatedStats, LogEntry};

/// TUIで表示するログの最大保持件数
const MAX_LOGS: usize = 500;

/// アプリケーション全体の状態を保持する構造体
#[derive(Debug)]
pub struct AppState {
    pub logs: VecDeque<LogEntry>,
    pub latest_stats: Option<AggregatedStats>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            logs: VecDeque::with_capacity(MAX_LOGS),
            latest_stats: None,
        }
    }

    pub fn add_log(&mut self, entry: LogEntry) {
        self.logs.push_front(entry);
        if self.logs.len() > MAX_LOGS {
            self.logs.pop_back();
        }
    }

    pub fn update_stats(&mut self, stats: AggregatedStats) {
        self.latest_stats = Some(stats);
    }
}