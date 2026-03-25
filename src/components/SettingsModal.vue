<script setup lang="ts">
import { ref } from "vue";

defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

// Settings state
const maxConcurrent = ref(3);
const defaultConnections = ref(3);
const downloadDir = ref("");

function handleClose() {
  emit("close");
}

function handleSave() {
  // In a real app, save to persistent storage
  localStorage.setItem("downloadSettings", JSON.stringify({
    maxConcurrent: maxConcurrent.value,
    defaultConnections: defaultConnections.value,
    downloadDir: downloadDir.value,
  }));
  emit("close");
}
</script>

<template>
  <div
    v-if="show"
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
    @click.self="handleClose"
  >
    <div class="bg-white rounded-lg shadow-xl w-full max-w-md mx-4">
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b">
        <h2 class="text-lg font-semibold text-gray-900">设置</h2>
        <button
          @click="handleClose"
          class="text-gray-400 hover:text-gray-600 transition"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Settings Form -->
      <div class="p-4 space-y-4">
        <!-- Max Concurrent -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            最大并发下载数: {{ maxConcurrent }}
          </label>
          <input
            v-model.number="maxConcurrent"
            type="range"
            min="1"
            max="5"
            step="1"
            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
          />
          <div class="flex justify-between text-xs text-gray-500 mt-1">
            <span>1</span>
            <span>5</span>
          </div>
        </div>

        <!-- Default Connections -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            默认连接数: {{ defaultConnections }}
          </label>
          <input
            v-model.number="defaultConnections"
            type="range"
            min="1"
            max="8"
            step="1"
            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
          />
          <div class="flex justify-between text-xs text-gray-500 mt-1">
            <span>1</span>
            <span>8</span>
          </div>
        </div>

        <!-- Download Directory -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            下载目录
          </label>
          <input
            v-model="downloadDir"
            type="text"
            placeholder="默认目录"
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
          />
        </div>
      </div>

      <!-- Actions -->
      <div class="flex justify-end gap-3 p-4 border-t">
        <button
          @click="handleClose"
          class="px-4 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition"
        >
          取消
        </button>
        <button
          @click="handleSave"
          class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition"
        >
          保存
        </button>
      </div>
    </div>
  </div>
</template>
