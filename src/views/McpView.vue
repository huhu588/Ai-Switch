<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface McpServer {
  name: string
  server_type: string
  enabled: boolean
  url: string | null
  command: string[] | null
}

const servers = ref<McpServer[]>([])
const selectedServer = ref<string | null>(null)
const loading = ref(false)

async function loadServers() {
  loading.value = true
  try {
    servers.value = await invoke<McpServer[]>('get_mcp_servers')
    if (servers.value.length > 0 && !selectedServer.value) {
      selectedServer.value = servers.value[0].name
    }
  } catch (e) {
    console.error('加载 MCP 服务器失败:', e)
  } finally {
    loading.value = false
  }
}

async function toggleServer(name: string) {
  try {
    await invoke('toggle_mcp_server', { name })
    await loadServers()
  } catch (e) {
    console.error('切换状态失败:', e)
  }
}

onMounted(() => {
  loadServers()
})

const currentServer = () => servers.value.find(s => s.name === selectedServer.value)
</script>

<template>
  <div class="h-full flex gap-4">
    <!-- 服务器列表 -->
    <div class="w-72 flex-shrink-0">
      <div class="h-full flex flex-col rounded-xl bg-cream-50 dark:bg-dark-800 border border-cream-300 dark:border-dark-700 overflow-hidden">
        <div class="flex items-center justify-between px-4 py-3 border-b border-cream-300 dark:border-dark-700">
          <h3 class="font-semibold text-sm">MCP 服务器</h3>
          <span class="text-xs text-primary-500 dark:text-dark-400">({{ servers.length }})</span>
        </div>

        <div class="flex-1 overflow-auto">
          <div v-if="loading" class="p-4 text-center text-primary-500 dark:text-dark-400">
            加载中...
          </div>
          <div v-else-if="servers.length === 0" class="p-4 text-center text-primary-500 dark:text-dark-400">
            暂无 MCP 服务器
          </div>
          <ul v-else class="p-2 space-y-1">
            <li
              v-for="server in servers"
              :key="server.name"
              @click="selectedServer = server.name"
              class="px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-150"
              :class="[
                server.name === selectedServer
                  ? 'bg-accent-100 dark:bg-accent-900/30 border border-accent-300 dark:border-accent-700'
                  : 'hover:bg-cream-200 dark:hover:bg-dark-700/50 border border-transparent'
              ]"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <span>{{ server.server_type === 'local' ? '📦' : '🌐' }}</span>
                  <span class="font-medium text-sm truncate">{{ server.name }}</span>
                </div>
                <button
                  @click.stop="toggleServer(server.name)"
                  class="text-xs px-2 py-0.5 rounded"
                  :class="server.enabled ? 'bg-success-500/20 text-success-600' : 'bg-primary-200 dark:bg-dark-600 text-primary-500'"
                >
                  {{ server.enabled ? '启用' : '禁用' }}
                </button>
              </div>
              <div class="mt-1 text-xs text-primary-500 dark:text-dark-400 truncate">
                {{ server.server_type === 'local' ? server.command?.join(' ') : server.url }}
              </div>
            </li>
          </ul>
        </div>
      </div>
    </div>

    <!-- 详情面板 -->
    <div class="flex-1">
      <div class="h-full rounded-xl bg-cream-50 dark:bg-dark-800 border border-cream-300 dark:border-dark-700 p-4">
        <div v-if="!currentServer()" class="text-center text-primary-500 dark:text-dark-400 py-8">
          选择一个 MCP 服务器查看详情
        </div>
        <div v-else class="space-y-4">
          <h3 class="font-semibold text-lg">{{ currentServer()?.name }}</h3>
          <div class="space-y-2 text-sm">
            <div class="flex gap-3">
              <span class="text-primary-500 dark:text-dark-400 w-20">类型</span>
              <span>{{ currentServer()?.server_type === 'local' ? '本地' : '远程' }}</span>
            </div>
            <div class="flex gap-3">
              <span class="text-primary-500 dark:text-dark-400 w-20">状态</span>
              <span :class="currentServer()?.enabled ? 'text-success-500' : 'text-primary-400'">
                {{ currentServer()?.enabled ? '已启用' : '已禁用' }}
              </span>
            </div>
            <div v-if="currentServer()?.command" class="flex gap-3">
              <span class="text-primary-500 dark:text-dark-400 w-20">命令</span>
              <span class="font-mono text-xs">{{ currentServer()?.command?.join(' ') }}</span>
            </div>
            <div v-if="currentServer()?.url" class="flex gap-3">
              <span class="text-primary-500 dark:text-dark-400 w-20">URL</span>
              <span class="font-mono text-xs">{{ currentServer()?.url }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
