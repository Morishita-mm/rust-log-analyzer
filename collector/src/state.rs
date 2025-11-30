use std::collections::VecDeque;
use crate::types::{AggregatedStats, LogEntry};
use regex::Regex;

/// TUIで表示するログの最大保持件数
const MAX_LOGS: usize = 500;

/// アプリケーション全体の状態を保持する構造体
#[derive(Debug)]
pub struct AppState {
    pub logs: VecDeque<LogEntry>,
    pub latest_stats: Option<AggregatedStats>,
    pub selected_log_index: Option<usize>,
    pub filter_text: String,
    pub filter_regex: Option<Regex>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            logs: VecDeque::with_capacity(MAX_LOGS),
            latest_stats: None,
            selected_log_index: None,
            filter_text: String::new(),
            filter_regex: None,
        }
    }

    pub fn add_log(&mut self, entry: LogEntry) {
        self.logs.push_front(entry);

        if let Some(i) = self.selected_log_index {
            self.selected_log_index = Some(i + 1);
        }

        if self.logs.len() > MAX_LOGS {
            self.logs.pop_back();

            if let Some(i) = self.selected_log_index {
                if i >= self.logs.len() {
                    self.selected_log_index = Some(self.logs.len() - 1);
                }
            }
        }
    }

    pub fn update_stats(&mut self, stats: AggregatedStats) {
        self.latest_stats = Some(stats);
    }

    pub fn select_next_log(&mut self) {
        if self.logs.is_empty() {
            return;
        }

        let i = match self.selected_log_index {
            // まだ選択されていない場合は、一番上（0番目）を選択
            None => 0,
            // すでに選択されている場合は、次のインデックスへ
            Some(i) => {
                if i >= self.logs.len() - 1 {
                    i
                } else {
                    i + 1
                }
            }
        };
        self.selected_log_index = Some(i);
    }

    pub fn select_previous_log(&mut self) {
        if self.logs.is_empty() {
            return;
        }

        if let Some(i) = self.selected_log_index {
            self.selected_log_index = Some(i.saturating_sub(1));
        }
    }

    pub fn unselect_log(&mut self) {
        self.selected_log_index = None;
    }

    pub fn set_filter(&mut self, text: String) {
        self.filter_text = text.clone();
        if text.is_empty() {
            self.filter_regex = None;
        } else {
            match Regex::new(&text) {
                Ok(re) => self.filter_regex = Some(re),
                Err(_) => self.filter_regex = None,
            }
        }
    }
}
