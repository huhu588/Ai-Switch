<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()

interface CodexStatus {
  is_configured: boolean
  has_auth: boolean
  provider_count: number
  mcp_server_count: number
  credentials_store: string | null
}

interface CodexProvider {
  name: string
  base_url: string
  env_key: string | null
  requires_openai_auth: boolean | null
}

const status = ref<CodexStatus | null>(null)
const providers = ref<Record<string, CodexProvider>>({})
const loading = ref(false)
const error = ref<string | null>(null)

// 添加 Provider 表单
const showAddForm = ref(false)
const newProvider = ref({
  name: '',
  displayName: '',
  baseUrl: '',
  envKey: '',
})

async function loadStatus() {
  loading.value = true
  error.value = null
  try {
    status.value = await invoke('get_codex_status')
    providers.value = await invoke('get_codex_providers')
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

async function addProvider() {
  if (!newProvider.value.name || !newProvider.value.baseUrl) return
  loading.value = true
  try {
    const provider: CodexProvider = {
      name: newProvider.value.displayName || newProvider.value.name,
      base_url: newProvider.value.baseUrl,
      env_key: newProvider.value.envKey || null,
      requires_openai_auth: null,
    }
    await invoke('add_codex_provider', { 
      name: newProvider.value.name, 
      provider 
    })
    newProvider.value = { name: '', displayName: '', baseUrl: '', envKey: '' }
    showAddForm.value = false
    await loadStatus()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

async function removeProvider(name: string) {
  loading.value = true
  try {
    await invoke('remove_codex_provider', { name })
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
        <div class="w-10 h-10 rounded-lg bg-green-500/10 flex items-center justify-center">
          <SvgIcon name="icon-codex" class="w-6 h-6 text-green-500" />
        </div>
        <div>
          <h1 class="text-xl font-semibold">Codex</h1>
          <p class="text-sm text-gray-500">{{ t('codex.description') }}</p>
        </div>
      </div>
      <div class="flex gap-2">
        <button 
          @click="showAddForm = true"
          class="px-3 py-1.5 text-sm rounded-lg bg-blue-500 text-white hover:bg-blue-600 transition-colors"
        >
          {{ t('codex.addProvider') }}
        </button>
        <button 
          @click="loadStatus" 
          :disabled="loading"
          class="px-3 py-1.5 text-sm rounded-lg bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
        >
          {{ t('common.refresh') }}
        </button>
      </div>
    </div>

    <!-- 状态卡片 -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <!-- 认证状态 -->
      <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
        <div class="flex items-center gap-2 mb-2">
          <div :class="['w-2 h-2 rounded-full', status?.has_auth ? 'bg-green-500' : 'bg-yellow-500']"></div>
          <span class="text-sm font-medium">{{ t('codex.authStatus') }}</span>
        </div>
        <p class="text-lg">
          {{ status?.has_auth ? t('codex.authenticated') : t('codex.notAuthenticated') }}
        </p>
      </div>

      <!-- Provider 数量 -->
      <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
        <div class="flex items-center gap-2 mb-2">
          <span class="text-sm font-medium">{{ t('codex.providers') }}</span>
        </div>
        <p class="text-lg">
          {{ status?.provider_count || 0 }} {{ t('codex.configured') }}
        </p>
      </div>

      <!-- MCP 服务器数量 -->
      <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
        <div class="flex items-center gap-2 mb-2">
          <span class="text-sm font-medium">{{ t('codex.mcpServers') }}</span>
        </div>
        <p class="text-lg">
          {{ status?.mcp_server_count || 0 }} {{ t('codex.servers') }}
        </p>
      </div>
    </div>

    <!-- Provider 列表 -->
    <div class="flex-1 overflow-auto">
      <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
        <h3 class="text-lg font-medium mb-4">{{ t('codex.modelProviders') }}</h3>
        
        <div v-if="Object.keys(providers).length === 0" class="text-center py-8 text-gray-500">
          {{ t('codex.noProviders') }}
        </div>
        
        <div v-else class="space-y-3">
          <div 
            v-for="(provider, name) in providers" 
            :key="name"
            class="p-4 rounded-lg bg-gray-50 dark:bg-gray-900 border border-gray-200 dark:border-gray-700"
          >
            <div class="flex items-center justify-between">
              <div>
                <h4 class="font-medium">{{ name }}</h4>
                <p class="text-sm text-gray-500">{{ provider.name }}</p>
                <p class="text-xs text-gray-400 truncate">{{ provider.base_url }}</p>
              </div>
              <button
                @click="removeProvider(String(name))"
                class="p-2 text-red-500 hover:bg-red-100 dark:hover:bg-red-900/30 rounded-lg transition-colors"
              >
                <SvgIcon name="icon-delete" class="w-5 h-5" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 添加 Provider 对话框 -->
    <Teleport to="body">
      <div v-if="showAddForm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showAddForm = false">
        <div class="bg-white dark:bg-gray-800 rounded-xl p-6 w-full max-w-md mx-4">
          <h3 class="text-lg font-semibold mb-4">{{ t('codex.addProvider') }}</h3>
          
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium mb-1">{{ t('codex.providerKey') }}</label>
              <input
                v-model="newProvider.name"
                type="text"
                :placeholder="t('codex.providerKeyPlaceholder')"
                class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900"
              />
            </div>
            
            <div>
              <label class="block text-sm font-medium mb-1">{{ t('codex.displayName') }}</label>
              <input
                v-model="newProvider.displayName"
                type="text"
                :placeholder="t('codex.displayNamePlaceholder')"
                class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900"
              />
            </div>
            
            <div>
              <label class="block text-sm font-medium mb-1">Base URL</label>
              <input
                v-model="newProvider.baseUrl"
                type="text"
                placeholder="https://api.example.com/v1"
                class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900"
              />
            </div>
            
            <div>
              <label class="block text-sm font-medium mb-1">{{ t('codex.envKey') }}</label>
              <input
                v-model="newProvider.envKey"
                type="text"
                :placeholder="t('codex.envKeyPlaceholder')"
                class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900"
              />
            </div>
          </div>
          
          <div class="flex justify-end gap-2 mt-6">
            <button
              @click="showAddForm = false"
              class="px-4 py-2 rounded-lg bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              @click="addProvider"
              :disabled="!newProvider.name || !newProvider.baseUrl"
              class="px-4 py-2 rounded-lg bg-blue-500 text-white hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {{ t('common.add') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- 错误提示 -->
    <div v-if="error" class="p-3 rounded-lg bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400">
      {{ error }}
    </div>
  </div>
</template>
