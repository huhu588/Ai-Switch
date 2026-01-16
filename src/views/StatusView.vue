<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface AppStatus {
  has_global_config: boolean
  has_project_config: boolean
  active_provider: string | null
  provider_count: number
  mcp_server_count: number
  config_paths: {
    global_config_dir: string
    global_opencode_dir: string
    project_opencode_dir: string | null
  }
}

const status = ref<AppStatus | null>(null)
const version = ref('')
const loading = ref(true)

async function loadStatus() {
  loading.value = true
  try {
    status.value = await invoke<AppStatus>('get_status')
    version.value = await invoke<string>('get_version')
  } catch (e) {
    console.error('加载状态失败:', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadStatus()
})
</script>

<template>
  <div class="max-w-2xl mx-auto">
    <div class="rounded-xl bg-cream-50 dark:bg-dark-800 border border-cream-300 dark:border-dark-700 p-6">
      <div class="flex items-center gap-3 mb-6">
        <span class="text-3xl">📊</span>
        <h2 class="text-xl font-semibold">系统状态</h2>
      </div>

      <div v-if="loading" class="py-8 text-center text-primary-500 dark:text-dark-400">
        加载中...
      </div>

      <div v-else-if="status" class="space-y-6">
        <!-- 版本信息 -->
        <section>
          <h3 class="text-xs font-semibold uppercase tracking-wide text-primary-500 dark:text-dark-400 mb-3">
            应用信息
          </h3>
          <div class="grid grid-cols-2 gap-4">
            <div class="bg-cream-100 dark:bg-dark-700/50 rounded-lg p-4">
              <div class="text-2xl font-bold">v{{ version }}</div>
              <div class="text-xs text-primary-500 dark:text-dark-400">当前版本</div>
            </div>
            <div class="bg-cream-100 dark:bg-dark-700/50 rounded-lg p-4">
              <div class="text-2xl font-bold">{{ status.provider_count }}</div>
              <div class="text-xs text-primary-500 dark:text-dark-400">Provider 数量</div>
            </div>
          </div>
        </section>

        <!-- 配置状态 -->
        <section>
          <h3 class="text-xs font-semibold uppercase tracking-wide text-primary-500 dark:text-dark-400 mb-3">
            配置状态
          </h3>
          <div class="space-y-3">
            <div class="flex items-center justify-between py-2 border-b border-cream-200 dark:border-dark-700">
              <span class="text-sm">全局配置</span>
              <span class="text-success-500">✓ 已配置</span>
            </div>
            <div class="flex items-center justify-between py-2 border-b border-cream-200 dark:border-dark-700">
              <span class="text-sm">项目配置</span>
              <span :class="status.has_project_config ? 'text-success-500' : 'text-primary-400'">
                {{ status.has_project_config ? '✓ 已配置' : '✗ 未配置' }}
              </span>
            </div>
            <div class="flex items-center justify-between py-2 border-b border-cream-200 dark:border-dark-700">
              <span class="text-sm">当前 Provider</span>
              <span class="font-mono text-sm">{{ status.active_provider || '-' }}</span>
            </div>
            <div class="flex items-center justify-between py-2">
              <span class="text-sm">MCP 服务器</span>
              <span>{{ status.mcp_server_count }} 个</span>
            </div>
          </div>
        </section>

        <!-- 配置路径 -->
        <section>
          <h3 class="text-xs font-semibold uppercase tracking-wide text-primary-500 dark:text-dark-400 mb-3">
            配置路径
          </h3>
          <div class="space-y-2 text-sm">
            <div class="flex items-start gap-3">
              <span class="text-primary-500 dark:text-dark-400 w-20 shrink-0">全局配置</span>
              <span class="font-mono text-xs break-all">{{ status.config_paths.global_config_dir }}</span>
            </div>
            <div class="flex items-start gap-3">
              <span class="text-primary-500 dark:text-dark-400 w-20 shrink-0">OpenCode</span>
              <span class="font-mono text-xs break-all">{{ status.config_paths.global_opencode_dir }}</span>
            </div>
            <div v-if="status.config_paths.project_opencode_dir" class="flex items-start gap-3">
              <span class="text-primary-500 dark:text-dark-400 w-20 shrink-0">项目配置</span>
              <span class="font-mono text-xs break-all">{{ status.config_paths.project_opencode_dir }}</span>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>
