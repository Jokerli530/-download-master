<script setup lang="ts">
import { onMounted, computed } from "vue";
import { useDownloadStore } from "../stores/downloads";
import DownloadItem from "./DownloadItem.vue";

const store = useDownloadStore();

onMounted(() => {
  store.fetchTasks();
});

const sortedTasks = computed(() => {
  return [...store.tasks].sort((a, b) => {
    // downloading first, then pending, paused, completed, failed, cancelled
    const order: Record<string, number> = {
      downloading: 0,
      pending: 1,
      paused: 2,
      completed: 3,
      failed: 4,
      cancelled: 5,
    };
    return (order[a.status] || 99) - (order[b.status] || 99);
  });
});
</script>

<template>
  <div class="flex-1 overflow-y-auto">
    <div v-if="store.tasks.length === 0" class="flex flex-col items-center justify-center h-full text-gray-500">
      <svg class="w-24 h-24 mb-4 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
          d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <p class="text-lg">暂无下载任务</p>
      <p class="text-sm mt-1">点击上方「新建任务」开始下载</p>
    </div>

    <div v-else class="p-4">
      <DownloadItem
        v-for="task in sortedTasks"
        :key="task.id"
        :task="task"
      />
    </div>
  </div>
</template>
