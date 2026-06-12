use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShareStatus {
    Accepted,
    Rejected,
    Stale,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareCandidate {
    pub job_id: String,
    pub nonce: String,
    pub result: String,
    pub worker: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareResult {
    pub candidate: ShareCandidate,
    pub status: ShareStatus,
    pub difficulty: f64,
    pub latency_ms: u64,
    pub error_message: Option<String>,
}

pub struct ShareTracker {
    pub accepted_count: u64,
    pub rejected_count: u64,
    pub stale_count: u64,
    pub invalid_count: u64,
    pub last_accepted: Option<DateTime<Utc>>,
    pub last_rejected: Option<DateTime<Utc>>,
    pub current_difficulty: f64,
}

impl ShareTracker {
    pub fn new() -> Self {
        Self {
            accepted_count: 0,
            rejected_count: 0,
            stale_count: 0,
            invalid_count: 0,
            last_accepted: None,
            last_rejected: None,
            current_difficulty: 0.0,
        }
    }

    pub fn record_result(&mut self, result: &ShareResult) {
        match result.status {
            ShareStatus::Accepted => {
                self.accepted_count += 1;
                self.last_accepted = Some(Utc::now());
            }
            ShareStatus::Rejected => {
                self.rejected_count += 1;
                self.last_rejected = Some(Utc::now());
            }
            ShareStatus::Stale => {
                self.stale_count += 1;
            }
            ShareStatus::Invalid => {
                self.invalid_count += 1;
            }
        }
    }

    pub fn set_difficulty(&mut self, difficulty: f64) {
        self.current_difficulty = difficulty;
    }
}

impl Default for ShareTracker {
    fn default() -> Self {
        Self::new()
    }
}
