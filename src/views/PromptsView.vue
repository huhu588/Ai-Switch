<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()

interface PromptInfo {
  prompt_type: 'claude' | 'codex' | 'gemini'
  name: string
  file_name: string
  exists: boolean
  content: string | null
  char_count: number
}

interface PromptsStatus {
  claude: PromptInfo
  codex: PromptInfo
  gemini: PromptInfo
}

interface PromptPreset {
  id: string
  name: string
  description: string
  content: string
}

const status = ref<PromptsStatus | null>(null)
const presets = ref<PromptPreset[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const successMsg = ref<string | null>(null)

// 编辑器状态
const activeTab = ref<'claude' | 'codex' | 'gemini'>('claude')
const editContent = ref('')
const showEditor = ref(false)

// 同步目标
const syncTargets = ref({
  claude: false,
  codex: false,
  gemini: false,
})

async function loadStatus() {
  loading.value = true
  error.value = null
  try {
    status.value = await invoke('get_prompts_status')
    presets.value = await invoke('get_prompt_presets')
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

function openEditor(type: 'claude' | 'codex' | 'gemini') {
  activeTab.value = type
  const promptInfo = status.value?.[type]
  editContent.value = promptInfo?.content || ''
  showEditor.value = true
}

async function savePrompt() {
  loading.value = true
  error.value = null
  try {
    await invoke('save_prompt', { 
      promptType: activeTab.value, 
      content: editContent.value 
    })
    successMsg.value = t('prompts.saved')
    setTimeout(() => successMsg.value = null, 3000)
    await loadStatus()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

async function syncToTargets() {
  const targets: string[] = []
  if (syncTargets.value.claude) targets.push('claude')
  if (syncTargets.value.codex) targets.push('codex')
  if (syncTargets.value.gemini) targets.push('gemini')
  
  if (targets.length === 0) return
  
  loading.value = true
  error.value = null
  try {
    const results = await invoke<string[]>('sync_prompt', { 
      content: editContent.value, 
      targets 
    })
    successMsg.value = results.join(', ')
    setTimeout(() => successMsg.value = null, 5000)
    await loadStatus()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

function applyPreset(preset: PromptPreset) {
  editContent.value = preset.content
}

async function deletePrompt(type: 'claude' | 'codex' | 'gemini') {
  if (!confirm(t('prompts.confirmDelete'))) return
  
  loading.value = true
  error.value = null
  try {
    await invoke('delete_prompt', { promptType: type })
    await loadStatus()
    if (activeTab.value === type) {
      editContent.value = ''
    }
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

const tabItems = computed(() => [
  { id: 'claude', name: 'Claude Code', file: 'CLAUDE.md', icon: 'icon-claude', color: 'orange' },
  { id: 'codex', name: 'Codex', file: 'AGENTS.md', icon: 'icon-codex', color: 'green' },
  { id: 'gemini', name: 'Gemini CLI', file: 'GEMINI.md', icon: 'icon-gemini', color: 'blue' },
])

onMounted(() => {
  loadStatus()
})
</script>

<template>
  <div class="h-full flex flex-col gap-4 p-4">
    <!-- 标题栏 -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-lg bg-purple-500/10 flex items-center justify-center">
          <SvgIcon name="icon-prompt" class="w-6 h-6 text-purple-500" />
        </div>
        <div>
          <h1 class="text-xl font-semibold">{{ t('prompts.title') }}</h1>
          <p class="text-sm text-gray-500">{{ t('prompts.description') }}</p>
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

    <!-- Prompt 卡片 -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <div 
        v-for="item in tabItems"
        :key="item.id"
        class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 cursor-pointer hover:border-blue-500 transition-colors"
        @click="openEditor(item.id as 'claude' | 'codex' | 'gemini')"
      >
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <div :class="['w-8 h-8 rounded-lg flex items-center justify-center', `bg-${item.color}-500/10`]">
              <SvgIcon :name="item.icon" :class="['w-5 h-5', `text-${item.color}-500`]" />
            </div>
            <span class="font-medium">{{ item.name }}</span>
          </div>
          <div :class="['w-2 h-2 rounded-full', status?.[item.id as keyof PromptsStatus]?.exists ? 'bg-green-500' : 'bg-gray-400']"></div>
        </div>
        <p class="text-sm text-gray-500 mb-2">{{ item.file }}</p>
        <p class="text-sm">
          {{ status?.[item.id as keyof PromptsStatus]?.char_count || 0 }} {{ t('prompts.characters') }}
        </p>
      </div>
    </div>

    <!-- 编辑器 -->
    <div v-if="showEditor" class="flex-1 flex flex-col min-h-0 p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
      <!-- 标签页 -->
      <div class="flex items-center gap-4 mb-4 pb-4 border-b border-gray-200 dark:border-gray-700">
        <div class="flex gap-2">
          <button
            v-for="item in tabItems"
            :key="item.id"
            @click="openEditor(item.id as 'claude' | 'codex' | 'gemini')"
            :class="[
              'px-3 py-1.5 text-sm rounded-lg transition-colors',
              activeTab === item.id 
                ? 'bg-blue-500 text-white' 
                : 'bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600'
            ]"
          >
            {{ item.name }}
          </button>
        </div>
        
        <div class="flex-1"></div>
        
        <button
          @click="deletePrompt(activeTab)"
          class="px-3 py-1.5 text-sm rounded-lg text-red-500 hover:bg-red-100 dark:hover:bg-red-900/30 transition-colors"
        >
          {{ t('common.delete') }}
        </button>
      </div>

      <!-- 编辑区域 -->
      <textarea
        v-model="editContent"
        :placeholder="t('prompts.placeholder')"
        class="flex-1 w-full p-4 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 resize-none font-mono text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent"
      ></textarea>

      <!-- 操作栏 -->
      <div class="flex items-center justify-between mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
        <!-- 同步选项 -->
        <div class="flex items-center gap-4">
          <span class="text-sm text-gray-500">{{ t('prompts.syncTo') }}:</span>
          <label v-for="item in tabItems" :key="item.id" class="flex items-center gap-1.5">
            <input
              type="checkbox"
              v-model="syncTargets[item.id as keyof typeof syncTargets]"
              class="rounded"
            />
            <span class="text-sm">{{ item.name }}</span>
          </label>
          <button
            @click="syncToTargets"
            :disabled="!Object.values(syncTargets).some(v => v)"
            class="px-3 py-1.5 text-sm rounded-lg bg-purple-500 text-white hover:bg-purple-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {{ t('prompts.sync') }}
          </button>
        </div>

        <!-- 保存按钮 -->
        <button
          @click="savePrompt"
          :disabled="loading"
          class="px-4 py-2 rounded-lg bg-blue-500 text-white hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {{ t('common.save') }}
        </button>
      </div>
    </div>

    <!-- 预设模板 -->
    <div v-if="showEditor && presets.length > 0" class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
      <h3 class="text-sm font-medium mb-3">{{ t('prompts.presets') }}</h3>
      <div class="flex flex-wrap gap-2">
        <button
          v-for="preset in presets"
          :key="preset.id"
          @click="applyPreset(preset)"
          class="px-3 py-1.5 text-sm rounded-lg bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
          :title="preset.description"
        >
          {{ preset.name }}
        </button>
      </div>
    </div>

    <!-- 消息提示 -->
    <div v-if="successMsg" class="p-3 rounded-lg bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400">
      {{ successMsg }}
    </div>
    <div v-if="error" class="p-3 rounded-lg bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400">
      {{ error }}
    </div>
  </div>
</template>
