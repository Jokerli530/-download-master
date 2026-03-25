import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface DownloadTask {
  id: string;
  url: string;
  filename: string;
  total_size: number;
  downloaded: number;
  status: "pending" | "downloading" | "paused" | "completed" | "failed" | "cancelled";
  connections: number;
  speed_limit: number | null;
  error_message: string | null;
  created_at: number;
  save_path: string;
}

export interface ProgressUpdate {
  id: string;
  downloaded: number;
  total: number;
  speed: number;
  progress: number;
  status: string;
}

export const useDownloadStore = defineStore("downloads", () => {
  // State
  const tasks = ref<DownloadTask[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  // Getters
  const activeTasks = computed(() =>
    tasks.value.filter((t) => t.status === "downloading" || t.status === "pending")
  );

  const completedTasks = computed(() =>
    tasks.value.filter((t) => t.status === "completed")
  );

  const pausedTasks = computed(() =>
    tasks.value.filter((t) => t.status === "paused")
  );

  const failedTasks = computed(() =>
    tasks.value.filter((t) => t.status === "failed")
  );

  // Actions
  async function fetchTasks() {
    try {
      isLoading.value = true;
      error.value = null;
      tasks.value = await invoke<DownloadTask[]>("get_tasks");
    } catch (e) {
      error.value = String(e);
      console.error("Failed to fetch tasks:", e);
    } finally {
      isLoading.value = false;
    }
  }

  async function addTask(url: string, filename?: string, connections?: number, savePath?: string) {
    try {
      error.value = null;
      const taskId = await invoke<string>("add_task", {
        url,
        filename: filename || null,
        connections: connections || null,
        savePath: savePath || "",
      });
      await fetchTasks();
      return taskId;
    } catch (e) {
      error.value = String(e);
      console.error("Failed to add task:", e);
      throw e;
    }
  }

  async function pauseTask(id: string) {
    try {
      error.value = null;
      await invoke("pause_task", { id });
      const task = tasks.value.find((t) => t.id === id);
      if (task) {
        task.status = "paused";
      }
    } catch (e) {
      error.value = String(e);
      console.error("Failed to pause task:", e);
      throw e;
    }
  }

  async function resumeTask(id: string) {
    try {
      error.value = null;
      await invoke("resume_task", { id });
      const task = tasks.value.find((t) => t.id === id);
      if (task) {
        task.status = "downloading";
      }
    } catch (e) {
      error.value = String(e);
      console.error("Failed to resume task:", e);
      throw e;
    }
  }

  async function cancelTask(id: string) {
    try {
      error.value = null;
      await invoke("cancel_task", { id });
      const task = tasks.value.find((t) => t.id === id);
      if (task) {
        task.status = "cancelled";
      }
    } catch (e) {
      error.value = String(e);
      console.error("Failed to cancel task:", e);
      throw e;
    }
  }

  async function clearCompleted() {
    try {
      error.value = null;
      await invoke("clear_completed");
      tasks.value = tasks.value.filter((t) => t.status !== "completed");
    } catch (e) {
      error.value = String(e);
      console.error("Failed to clear completed:", e);
      throw e;
    }
  }

  function updateTaskProgress(update: ProgressUpdate) {
    const task = tasks.value.find((t) => t.id === update.id);
    if (task) {
      task.downloaded = update.downloaded;
      task.total_size = update.total;
      task.status = update.status as DownloadTask["status"];
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  function formatSpeed(bytesPerSecond: number): string {
    return formatBytes(bytesPerSecond) + "/s";
  }

  return {
    // State
    tasks,
    isLoading,
    error,
    // Getters
    activeTasks,
    completedTasks,
    pausedTasks,
    failedTasks,
    // Actions
    fetchTasks,
    addTask,
    pauseTask,
    resumeTask,
    cancelTask,
    clearCompleted,
    updateTaskProgress,
    formatBytes,
    formatSpeed,
  };
});
