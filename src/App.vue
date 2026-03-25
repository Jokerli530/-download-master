<script setup lang="ts">
import { ref } from "vue";
import DownloadList from "./components/DownloadList.vue";
import AddTaskModal from "./components/AddTaskModal.vue";
import SettingsModal from "./components/SettingsModal.vue";

const showAddModal = ref(false);
const showSettingsModal = ref(false);
</script>

<template>
  <div class="min-h-screen bg-gray-100 flex flex-col">
    <!-- Header -->
    <header class="bg-white shadow-sm">
      <div class="max-w-5xl mx-auto px-4 py-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <svg class="w-8 h-8 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
            <h1 class="text-xl font-bold text-gray-900">DownloadMaster</h1>
          </div>
          <div class="flex items-center gap-2">
            <button
              @click="showAddModal = true"
              class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition flex items-center gap-2"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
              新建任务
            </button>
            <button
              @click="showSettingsModal = true"
              class="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-lg transition"
              title="设置"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="flex-1 flex flex-col max-w-5xl w-full mx-auto">
      <DownloadList />
    </main>

    <!-- Footer -->
    <footer class="bg-white border-t py-2">
      <div class="max-w-5xl mx-auto px-4 text-center text-sm text-gray-500">
        DownloadMaster - 简洁高效的下载工具
      </div>
    </footer>

    <!-- Modals -->
    <AddTaskModal :show="showAddModal" @close="showAddModal = false" />
    <SettingsModal :show="showSettingsModal" @close="showSettingsModal = false" />
  </div>
</template>
