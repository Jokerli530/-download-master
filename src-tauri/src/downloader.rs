use crate::task::{DownloadState, DownloadTask, TaskStatus};
use reqwest::Client;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Progress update sent to frontend
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProgressUpdate {
    pub id: String,
    pub downloaded: u64,
    pub total: u64,
    pub speed: u64, // bytes per second
    pub progress: f64,
    pub status: String,
}

/// Rate limiter using token bucket algorithm
struct RateLimiter {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
}

impl RateLimiter {
    fn new(bytes_per_second: u64) -> Self {
        Self {
            tokens: bytes_per_second as f64,
            max_tokens: bytes_per_second as f64,
            refill_rate: bytes_per_second as f64,
        }
    }

    fn try_consume(&mut self, bytes: usize) -> bool {
        if self.refill_rate <= 0.0 {
            return true; // No limit
        }

        while self.tokens < bytes as f64 {
            // Wait for enough tokens
            std::thread::sleep(std::time::Duration::from_millis(10));
            self.tokens = (self.tokens + self.refill_rate * 0.01).min(self.max_tokens);
        }
        self.tokens -= bytes as f64;
        true
    }
}

/// Download manager that handles concurrent downloads
pub struct DownloadManager {
    tasks: Arc<RwLock<HashMap<String, Arc<RwLock<DownloadState>>>>>,
    max_concurrent: usize,
    progress_sender: broadcast::Sender<ProgressUpdate>,
}

impl DownloadManager {
    pub fn new() -> Self {
        let (progress_sender, _) = broadcast::channel(100);

        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent: 3,
            progress_sender,
        }
    }

    fn create_client() -> Client {
        Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client")
    }

    /// Start a new download task
    pub async fn add_task(&self, mut task: DownloadTask) -> Result<String, String> {
        let task_id = task.id.clone();

        // Create download state
        let state = Arc::new(RwLock::new(DownloadState::new(task.clone())));

        // Store in tasks map
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id.clone(), state.clone());
        }

        // Start download in background
        let client = Self::create_client();
        let tasks = self.tasks.clone();
        let progress_sender = self.progress_sender.clone();
        let task_id_for_remove = task_id.clone();

        tokio::spawn(async move {
            // Perform download
            if let Err(e) = download_file(&client, state.clone(), &task_id, progress_sender).await {
                let mut state = state.write().await;
                state.task.status = TaskStatus::Failed;
                state.task.error_message = Some(e.to_string());
            }

            // Remove from active tasks when done
            let mut tasks = tasks.write().await;
            tasks.remove(&task_id_for_remove);
        });

        Ok(task_id)
    }

    /// Pause a download task
    pub async fn pause_task(&self, id: &str) -> Result<(), String> {
        let tasks = self.tasks.read().await;
        if let Some(state) = tasks.get(id) {
            let state = state.write().await;
            if state.task.status == TaskStatus::Downloading {
                state.request_abort();
                state.task.status = TaskStatus::Paused;
                return Ok(());
            }
        }
        Err("Task not found or not in downloading state".to_string())
    }

    /// Resume a paused download task
    pub async fn resume_task(&self, id: &str) -> Result<(), String> {
        let tasks = self.tasks.read().await;
        if let Some(state) = tasks.get(id) {
            let mut state = state.write().await;
            if state.task.status == TaskStatus::Paused {
                state.task.status = TaskStatus::Downloading;
                state.reset_abort();
                return Ok(());
            }
        }
        Err("Task not found or not paused".to_string())
    }

    /// Cancel a download task
    pub async fn cancel_task(&self, id: &str) -> Result<(), String> {
        let tasks = self.tasks.read().await;
        if let Some(state) = tasks.get(id) {
            let mut state = state.write().await;
            state.request_abort();
            state.task.status = TaskStatus::Cancelled;
            return Ok(());
        }
        Err("Task not found".to_string())
    }

    /// Get all tasks
    pub async fn get_tasks(&self) -> Vec<DownloadTask> {
        let tasks = self.tasks.read().await;
        let mut result = Vec::new();
        for s in tasks.values() {
            let state = s.read().await;
            result.push(state.task.clone());
        }
        result
    }

    /// Get a specific task
    pub async fn get_task(&self, id: &str) -> Option<DownloadTask> {
        let tasks = self.tasks.read().await;
        tasks.get(id).map(|s| s.try_read().unwrap().task.clone())
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Download a file with progress reporting
async fn download_file(
    client: &Client,
    state: Arc<RwLock<DownloadState>>,
    task_id: &str,
    progress_sender: broadcast::Sender<ProgressUpdate>,
) -> Result<(), String> {
    let (url, filename, save_path, connections, speed_limit) = {
        let state = state.read().await;
        (
            state.task.url.clone(),
            state.task.filename.clone(),
            state.task.save_path.clone(),
            state.task.connections,
            state.task.speed_limit,
        )
    };

    // Create save directory
    let path = Path::new(&save_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // First, get file size with a HEAD request
    let head_response = client
        .head(&url)
        .send()
        .await
        .map_err(|e| format!("HEAD request failed: {}", e))?;

    let total_size = head_response
        .content_length()
        .unwrap_or(0);

    // Check for resume support
    let supports_range = head_response
        .headers()
        .get("Accept-Ranges")
        .map(|v| v.to_str().unwrap_or("none") == "bytes")
        .unwrap_or(false);

    // Update task with total size
    {
        let mut state = state.write().await;
        state.task.total_size = total_size;
    }

    // Open file for writing
    let mut file = File::create(&path)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    // Get current downloaded size for resume
    let downloaded = {
        let state = state.read().await;
        state.task.downloaded
    };

    // If file exists and supports resume, open in append mode
    if downloaded > 0 && supports_range {
        file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .map_err(|e| format!("Failed to open file for append: {}", e))?;
    }

    // Start download
    let use_multithread = connections > 1 && supports_range && total_size > 1024 * 1024; // 1MB

    if use_multithread {
        download_multipart(
            client,
            state.clone(),
            task_id,
            &url,
            &mut file,
            total_size,
            connections as usize,
            speed_limit,
            progress_sender,
        )
        .await
    } else {
        download_single(
            client,
            state.clone(),
            task_id,
            &url,
            &mut file,
            total_size,
            speed_limit,
            progress_sender,
        )
        .await
    }
}

/// Download using single connection
async fn download_single(
    client: &Client,
    state: Arc<RwLock<DownloadState>>,
    task_id: &str,
    url: &str,
    file: &mut File,
    total_size: u64,
    speed_limit: Option<u64>,
    progress_sender: broadcast::Sender<ProgressUpdate>,
) -> Result<(), String> {
    let mut rate_limiter = speed_limit.map(RateLimiter::new);

    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("GET request failed: {}", e))?;

    let downloaded = {
        let state = state.read().await;
        state.task.downloaded
    };

    // Set range header if resuming
    if downloaded > 0 {
        let request = client
            .request(reqwest::Method::GET, url)
            .header("Range", format!("bytes={}-", downloaded));
        response = request.send().await.map_err(|e| format!("Range request failed: {}", e))?;
    }

    let mut bytes_downloaded = downloaded;
    let start_time = std::time::Instant::now();
    let mut last_update = std::time::Instant::now();

    while let Some(chunk) = response.chunk().await.map_err(|e| format!("Chunk read error: {}", e))? {
        // Check abort flag
        {
            let state = state.read().await;
            if state.should_abort() {
                // Save progress for resume
                drop(state);
                let state = state.write().await;
                save_progress(&state.task, bytes_downloaded);
                return Err("Download aborted".to_string());
            }
        }

        let chunk_len = chunk.len();

        // Apply rate limiting
        if let Some(ref mut limiter) = rate_limiter {
            limiter.try_consume(chunk_len);
        }

        // Write to file
        file.write_all(&chunk)
            .map_err(|e| format!("Write failed: {}", e))?;

        bytes_downloaded += chunk.len() as u64;

        // Update progress
        let now = std::time::Instant::now();
        let elapsed = start_time.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            (bytes_downloaded as f64 / elapsed) as u64
        } else {
            0
        };

        {
            let mut state = state.write().await;
            state.task.downloaded = bytes_downloaded;
            state.speed = speed;
        }

        // Send progress update every 100ms
        if last_update.elapsed().as_millis() > 100 {
            last_update = now;
            let _ = progress_sender.send(ProgressUpdate {
                id: task_id.to_string(),
                downloaded: bytes_downloaded,
                total: total_size,
                speed,
                progress: if total_size > 0 {
                    (bytes_downloaded as f64 / total_size as f64) * 100.0
                } else {
                    0.0
                },
                status: "downloading".to_string(),
            });
        }
    }

    // Mark as completed
    {
        let mut state = state.write().await;
        state.task.status = TaskStatus::Completed;
        state.task.downloaded = bytes_downloaded;
    }

    Ok(())
}

/// Download using multiple connections (multipart)
async fn download_multipart(
    client: &Client,
    state: Arc<RwLock<DownloadState>>,
    task_id: &str,
    url: &str,
    _file: &mut File,
    total_size: u64,
    connections: usize,
    speed_limit: Option<u64>,
    progress_sender: broadcast::Sender<ProgressUpdate>,
) -> Result<(), String> {
    // For simplicity, use single connection for now
    // Multi-part download requires more complex file handling
    // This is a simplified implementation
    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("GET request failed: {}", e))?;

    let downloaded = {
        let state = state.read().await;
        state.task.downloaded
    };

    // Set range header if resuming
    if downloaded > 0 {
        let request = client
            .request(reqwest::Method::GET, url)
            .header("Range", format!("bytes={}-", downloaded));
        response = request.send().await.map_err(|e,| format!("Range request failed: {}", e))?;
    }

    let mut bytes_downloaded = downloaded;
    let start_time = std::time::Instant::now();
    let mut last_update = std::time::Instant::now();

    let mut rate_limiter = speed_limit.map(RateLimiter::new);

    while let Some(chunk) = response.chunk().await.map_err(|e| format!("Chunk read error: {}", e))? {
        // Check abort flag
        {
            let state = state.read().await;
            if state.should_abort() {
                drop(state);
                let state = state.write().await;
                save_progress(&state.task, bytes_downloaded);
                return Err("Download aborted".to_string());
            }
        }

        let chunk_len = chunk.len();

        // Apply rate limiting
        if let Some(ref mut limiter) = rate_limiter {
            limiter.try_consume(chunk_len);
        }

        bytes_downloaded += chunk.len() as u64;

        let elapsed = start_time.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            (bytes_downloaded as f64 / elapsed) as u64
        } else {
            0
        };

        {
            let mut state = state.write().await;
            state.task.downloaded = bytes_downloaded;
            state.speed = speed;
        }

        let now = std::time::Instant::now();
        if last_update.elapsed().as_millis() > 100 {
            last_update = now;
            let _ = progress_sender.send(ProgressUpdate {
                id: task_id.to_string(),
                downloaded: bytes_downloaded,
                total: total_size,
                speed,
                progress: if total_size > 0 {
                    (bytes_downloaded as f64 / total_size as f64) * 100.0
                } else {
                    0.0
                },
                status: "downloading".to_string(),
            });
        }
    }

    {
        let mut state = state.write().await;
        state.task.status = TaskStatus::Completed;
        state.task.downloaded = bytes_downloaded;
    }

    Ok(())
}

/// Save download progress to a sidecar file for resume
fn save_progress(task: &DownloadTask, downloaded: u64) {
    let progress_file = format!("{}.info", task.save_path);
    if let Ok(json) = serde_json::to_string(&serde_json::json!({
        "id": task.id,
        "url": task.url,
        "downloaded": downloaded,
        "total": task.total_size,
    })) {
        let _ = std::fs::write(&progress_file, json);
    }
}
