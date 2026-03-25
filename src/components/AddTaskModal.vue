<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useDownloadStore } from "../stores/downloads";

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const store = useDownloadStore();

const url = ref("");
const filename = ref("");
const connections = ref(3);
const savePath = ref("");
const isSubmitting = ref(false);
const errorMessage = ref("");

watch(() => props.show, (newVal) => {
  if (newVal) {
    // Reset form when modal opens
    url.value = "";
    filename.value = "";
    connections.value = 3;
    errorMessage.value = "";
  }
});

async function selectSavePath() {
  try {
    const path = await invoke<string>("select_save_path");
    savePath.value = path;
  } catch (e) {
    // User cancelled or error
    console.log("No path selected");
  }
}

function extractFilenameFromUrl(urlString: string): string {
  try {
    const urlObj = new URL(urlString);
    const pathParts = urlObj.pathname.split("/");
    const lastPart = pathParts[pathParts.length - 1];
    if (lastPart && lastPart.includes(".")) {
      return lastPart.split("?")[0];
    }
  } catch {
    // Invalid URL
  }
  return "";
}

function handleUrlChange() {
  if (!filename.value) {
    filename.value = extractFilenameFromUrl(url.value);
  }
}

async function handleSubmit() {
  if (!url.value.trim()) {
    errorMessage.value = "请输入下载链接";
    return;
  }

  try {
    isSubmitting.value = true;
    errorMessage.value = "";
    await store.addTask(
      url.value.trim(),
      filename.value.trim() || undefined,
      connections.value,
      savePath.value || undefined
    );
    emit("close");
  } catch (e) {
    errorMessage.value = String(e);
  } finally {
    isSubmitting.value = false;
  }
}

function handleClose() {
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
        <h2 class="text-lg font-semibold text-gray-900">新建下载任务</h2>
        <button
          @click="handleClose"
          class="text-gray-400 hover:text-gray-600 transition"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Form -->
      <form @submit.prevent="handleSubmit" class="p-4 space-y-4">
        <!-- URL -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            下载链接 <span class="text-red-500">*</span>
          </label>
          <input
            v-model="url"
            @blur="handleUrlChange"
            type="text"
            placeholder="https://example.com/file.zip"
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
          />
        </div>

        <!-- Filename -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            文件名
          </label>
          <input
            v-model="filename"
            type="text"
            placeholder="自动从链接提取"
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
          />
        </div>

        <!-- Connections -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            连接数: {{ connections }}
          </label>
          <input
            v-model.number="connections"
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

        <!-- Save Path -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            保存路径
          </label>
          <div class="flex gap-2">
            <input
              v-model="savePath"
              type="text"
              placeholder="默认保存到下载文件夹"
              class="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
              readonly
            />
            <button
              type="button"
              @click="selectSavePath"
              class="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition"
            >
              选择
            </button>
          </div>
        </div>

        <!-- Error -->
        <div v-if="errorMessage" class="text-red-500 text-sm">
          {{ errorMessage }}
        </div>

        <!-- Actions -->
        <div class="flex justify-end gap-3 pt-2">
          <button
            type="button"
            @click="handleClose"
            class="px-4 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition"
          >
            取消
          </button>
          <button
            type="submit"
            :disabled="isSubmitting"
            class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition disabled:opacity-50"
          >
            {{ isSubmitting ? "添加中..." : "开始下载" }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>
