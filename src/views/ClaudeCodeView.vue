<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()

interface ClaudeCodeStatus {
  is_configured: boolean
  has_api_key: boolean
  api_key_masked: string | null
  base_url: string | null
  model: string | null
  mcp_server_count: number
}

const status = ref<ClaudeCodeStatus | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)

// 表单数据
const apiKey = ref('')
const baseUrl = ref('')
const model = ref('')
const showApiKey = ref(false)

// 加载状态
async function loadStatus() {
  loading.value = true
  error.value = null
  try {
    status.value = await invoke('get_claude_code_status')
    if (status.value?.base_url) {
      baseUrl.value = status.value.base_url
    }
    if (status.value?.model) {
      model.value = status.value.model
    }
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

// 保存 API Key
async function saveApiKey() {
  if (!apiKey.value.trim()) return
  loading.value = true
  try {
    await invoke('set_claude_code_api_key', { apiKey: apiKey.value })
    apiKey.value = ''
    await loadStatus()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

// 保存 Base URL
async function saveBaseUrl() {
  loading.value = true
  try {
    await invoke('set_claude_code_base_url', { baseUrl: baseUrl.value || null })
    await loadStatus()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

// 保存模型
async function saveModel() {
  loading.value = true
  try {
    await invoke('set_claude_code_model', { model: model.value || null })
    await loadStatus()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadStatus()
})
</script>

<template>
  <div class="h-full flex flex-col gap-4 p-4">
    <!-- 标题栏 -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-lg bg-orange-500/10 flex items-center justify-center">
          <SvgIcon name="icon-claude" class="w-6 h-6 text-orange-500" />
        </div>
        <div>
          <h1 class="text-xl font-semibold">Claude Code</h1>
          <p class="text-sm text-gray-500">{{ t('claudeCode.description') }}</p>
        </div>
      </div>
      <button 
        @click="loadStatus" 
        :disabled="loading"
        class="px-3 py-1.5 text-sm rounded-lg bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
      >
        {{ t('common.refresh') }}
      </button>
    </div>

    <!-- 状态卡片 -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <!-- API Key 状态 -->
      <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
        <div class="flex items-center gap-2 mb-2">
          <div :class="['w-2 h-2 rounded-full', status?.has_api_key ? 'bg-green-500' : 'bg-yellow-500']"></div>
          <span class="text-sm font-medium">API Key</span>
        </div>
        <p class="text-lg font-mono">
          {{ status?.api_key_masked || t('claudeCode.notConfigured') }}
        </p>
      </div>

      <!-- Base URL -->
      <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
        <div class="flex items-center gap-2 mb-2">
          <span class="text-sm font-medium">Base URL</span>
        </div>
        <p class="text-lg truncate">
          {{ status?.base_url || t('claudeCode.usingOfficial') }}
        </p>
      </div>

      <!-- MCP 服务器数量 -->
      <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
        <div class="flex items-center gap-2 mb-2">
          <span class="text-sm font-medium">{{ t('claudeCode.mcpServers') }}</span>
        </div>
        <p class="text-lg">
          {{ status?.mcp_server_count || 0 }} {{ t('claudeCode.servers') }}
        </p>
      </div>
    </div>

    <!-- 配置表单 -->
    <div class="flex-1 overflow-auto">
      <div class="space-y-6">
        <!-- API Key 设置 -->
        <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
          <h3 class="text-lg font-medium mb-4">{{ t('claudeCode.setApiKey') }}</h3>
          <div class="flex gap-2">
            <div class="flex-1 relative">
              <input
                v-model="apiKey"
                :type="showApiKey ? 'text' : 'password'"
                :placeholder="t('claudeCode.apiKeyPlaceholder')"
                class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              />
              <button
                @click="showApiKey = !showApiKey"
                class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-700"
              >
                <SvgIcon :name="showApiKey ? 'icon-eye-off' : 'icon-eye'" class="w-5 h-5" />
              </button>
            </div>
            <button
              @click="saveApiKey"
              :disabled="!apiKey.trim() || loading"
              class="px-4 py-2 rounded-lg bg-blue-500 text-white hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {{ t('common.save') }}
            </button>
          </div>
        </div>

        <!-- Base URL 设置 -->
        <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
          <h3 class="text-lg font-medium mb-4">{{ t('claudeCode.setBaseUrl') }}</h3>
          <div class="flex gap-2">
            <input
              v-model="baseUrl"
              type="text"
              :placeholder="t('claudeCode.baseUrlPlaceholder')"
              class="flex-1 px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <button
              @click="saveBaseUrl"
              :disabled="loading"
              class="px-4 py-2 rounded-lg bg-blue-500 text-white hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {{ t('common.save') }}
            </button>
          </div>
          <p class="text-sm text-gray-500 mt-2">{{ t('claudeCode.baseUrlHint') }}</p>
        </div>

        <!-- 模型设置 -->
        <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
          <h3 class="text-lg font-medium mb-4">{{ t('claudeCode.setModel') }}</h3>
          <div class="flex gap-2">
            <input
              v-model="model"
              type="text"
              :placeholder="t('claudeCode.modelPlaceholder')"
              class="flex-1 px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <button
              @click="saveModel"
              :disabled="loading"
              class="px-4 py-2 rounded-lg bg-blue-500 text-white hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {{ t('common.save') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="p-3 rounded-lg bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400">
      {{ error }}
    </div>
  </div>
</template>
