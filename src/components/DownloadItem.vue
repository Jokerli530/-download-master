<script setup lang="ts">
import { computed } from "vue";
import type { DownloadTask } from "../stores/downloads";
import { useDownloadStore } from "../stores/downloads";

const props = defineProps<{
  task: DownloadTask;
}>();

const store = useDownloadStore();

const progress = computed(() => {
  if (props.task.total_size === 0) return 0;
  return (props.task.downloaded / props.task.total_size) * 100;
});

const statusText = computed(() => {
  switch (props.task.status) {
    case "pending": return "等待中";
    case "downloading": return "下载中";
    case "paused": return "已暂停";
    case "completed": return "已完成";
    case "failed": return "失败";
    case "cancelled": return "已取消";
    default: return props.task.status;
  }
});

const statusColor = computed(() => {
  switch (props.task.status) {
    case "downloading": return "bg-blue-500";
    case "paused": return "bg-yellow-500";
    case "completed": return "bg-green-500";
    case "failed": return "bg-red-500";
    case "cancelled": return "bg-gray-500";
    default: return "bg-gray-400";
  }
});

function handlePause() {
  store.pauseTask(props.task.id);
}

function handleResume() {
  store.resumeTask(props.task.id);
}

function handleCancel() {
  store.cancelTask(props.task.id);
}
</script>

<template>
  <div class="bg-white rounded-lg shadow p-4 mb-3">
    <div class="flex items-center justify-between mb-2">
      <div class="flex-1 min-w-0">
        <h3 class="font-medium text-gray-900 truncate">{{ task.filename }}</h3>
        <p class="text-sm text-gray-500 truncate" :title="task.url">{{ task.url }}</p>
      </div>
      <div class="flex items-center gap-2 ml-4">
        <span
          :class="[
            'px-2 py-1 text-xs rounded-full text-white',
            statusColor
          ]"
        >
          {{ statusText }}
        </span>
      </div>
    </div>

    <!-- Progress bar -->
    <div class="mb-2">
      <div class="flex justify-between text-sm text-gray-600 mb-1">
        <span>{{ store.formatBytes(task.downloaded) }} / {{ store.formatBytes(task.total_size) }}</span>
        <span>{{ progress.toFixed(1) }}%</span>
      </div>
      <div class="w-full bg-gray-200 rounded-full h-2">
        <div
          :class="['h-2 rounded-full transition-all duration-300', statusColor]"
          :style="{ width: `${progress}%` }"
        ></div>
      </div>
    </div>

    <!-- Actions -->
    <div class="flex justify-between items-center">
      <div class="text-sm text-gray-500">
        <span v-if="task.status === 'downloading'" class="text-blue-600">
          {{ store.formatSpeed(0) }}
        </span>
        <span v-else-if="task.status === 'paused'">
          暂停
        </span>
        <span v-else-if="task.status === 'completed'">
          {{ store.formatBytes(task.total_size) }}
        </span>
        <span v-else-if="task.status === 'failed'" class="text-red-500">
          {{ task.error_message || '下载失败' }}
        </span>
      </div>
      <div class="flex gap-2">
        <button
          v-if="task.status === 'downloading'"
          @click="handlePause"
          class="px-3 py-1 text-sm bg-yellow-500 text-white rounded hover:bg-yellow-600 transition"
        >
          暂停
        </button>
        <button
          v-if="task.status === 'paused'"
          @click="handleResume"
          class="px-3 py-1 text-sm bg-green-500 text-white rounded hover:bg-green-600 transition"
        >
          继续
        </button>
        <button
          v-if="task.status === 'downloading' || task.status === 'paused'"
          @click="handleCancel"
          class="px-3 py-1 text-sm bg-red-500 text-white rounded hover:bg-red-600 transition"
        >
          取消
        </button>
      </div>
    </div>
  </div>
</template>
