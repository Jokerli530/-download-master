use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Download task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

/// Download task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub total_size: u64,
    pub downloaded: u64,
    pub status: TaskStatus,
    pub connections: u8,
    pub speed_limit: Option<u64>, // bytes per second, None = no limit
    pub error_message: Option<String>,
    pub created_at: i64,
    pub save_path: String,
}

impl DownloadTask {
    pub fn new(url: String, filename: String, save_path: String, connections: u8) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            filename,
            total_size: 0,
            downloaded: 0,
            status: TaskStatus::Pending,
            connections,
            speed_limit: None,
            error_message: None,
            created_at: chrono_timestamp(),
            save_path,
        }
    }

    pub fn progress(&self) -> f64 {
        if self.total_size == 0 {
            0.0
        } else {
            (self.downloaded as f64 / self.total_size as f64) * 100.0
        }
    }
}

/// Internal state for a running download
pub struct DownloadState {
    pub task: DownloadTask,
    pub abort_flag: Arc<Mutex<bool>>,
    pub speed: u64, // bytes per second, updated in real-time
}

impl DownloadState {
    pub fn new(task: DownloadTask) -> Self {
        Self {
            task,
            abort_flag: Arc::new(Mutex::new(false)),
            speed: 0,
        }
    }

    pub fn should_abort(&self) -> bool {
        *self.abort_flag.lock()
    }

    pub fn request_abort(&self) {
        *self.abort_flag.lock() = true;
    }

    pub fn reset_abort(&self) {
        *self.abort_flag.lock() = false;
    }
}

/// Get current timestamp in seconds
fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
