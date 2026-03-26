use crate::downloader::{DownloadManager, ProgressUpdate};
use crate::task::{DownloadTask, TaskStatus};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::RwLock;

/// App state holding the download manager
pub struct AppState {
    pub manager: Arc<RwLock<DownloadManager>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(RwLock::new(DownloadManager::new())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Add a new download task
#[tauri::command]
pub async fn add_task(
    url: String,
    filename: Option<String>,
    connections: Option<u8>,
    save_path: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<String, String> {
    // Extract filename from URL if not provided
    let fname = filename.unwrap_or_else(|| {
        url.split('/')
            .last()
            .unwrap_or("download")
            .split('?')
            .next()
            .unwrap_or("download")
            .to_string()
    });

    let connections = connections.unwrap_or(3).max(1).min(8);

    let task = DownloadTask::new(url, fname, save_path, connections);

    // Clone task_id for the response
    let task_id = task.id.clone();

    // Start download - use write().await to release lock before await
    {
        let manager = state.manager.write().await;
        manager.add_task(task).await?;
    }

    Ok(task_id)
}

/// Pause a download task
#[tauri::command]
pub async fn pause_task(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let manager = state.manager.write().await;
    manager.pause_task(&id).await
}

/// Resume a paused download task
#[tauri::command]
pub async fn resume_task(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let manager = state.manager.write().await;
    manager.resume_task(&id).await
}

/// Cancel a download task
#[tauri::command]
pub async fn cancel_task(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let manager = state.manager.write().await;
    manager.cancel_task(&id).await
}

/// Get all download tasks
#[tauri::command]
pub async fn get_tasks(state: State<'_, AppState>) -> Result<Vec<DownloadTask>, String> {
    let manager = state.manager.read().await;
    Ok(manager.get_tasks().await)
}

/// Get a specific task
#[tauri::command]
pub async fn get_task(id: String, state: State<'_, AppState>) -> Result<Option<DownloadTask>, String> {
    let manager = state.manager.read().await;
    Ok(manager.get_task(&id).await)
}

/// Set speed limit for a task (bytes per second, 0 = no limit)
#[tauri::command]
pub async fn set_speed_limit(id: String, bytes_per_second: u64, state: State<'_, AppState>) -> Result<(), String> {
    let manager = state.manager.write().await;
    if let Some(mut task) = manager.get_task(&id).await {
        task.speed_limit = if bytes_per_second == 0 { None } else { Some(bytes_per_second) };
        Ok(())
    } else {
        Err("Task not found".to_string())
    }
}

/// Clear all completed tasks
#[tauri::command]
pub fn clear_completed(state: State<'_, AppState>) -> Result<(), String> {
    // For now, this is a no-op since we're using in-memory storage
    // In a real app, you'd persist tasks to disk and filter here
    Ok(())
}

/// Open file save dialog
#[tauri::command]
pub async fn select_save_path(app: AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;

    let file_path = app
        .dialog()
        .file()
        .add_filter("All Files", &["*"])
        .blocking_pick_file();

    file_path
        .map(|p| p.to_string())
        .ok_or_else(|| "No file selected".to_string())
}
