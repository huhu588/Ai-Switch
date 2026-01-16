<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface Props {
  visible: boolean
  providerName: string | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  added: []
}>()

const models = ref<string[]>([])
const selectedModels = ref<Set<string>>(new Set())
const loading = ref(false)
const adding = ref(false)
const error = ref<string | null>(null)

watch(() => props.visible, async (visible) => {
  if (visible && props.providerName) {
    await fetchModels()
  } else {
    models.value = []
    selectedModels.value.clear()
    error.value = null
  }
})

async function fetchModels() {
  if (!props.providerName) return
  
  loading.value = true
  error.value = null
  
  try {
    models.value = await invoke<string[]>('fetch_site_models', {
      providerName: props.providerName
    })
    // 默认全选
    selectedModels.value = new Set(models.value)
  } catch (e) {
    error.value = String(e)
    models.value = []
  } finally {
    loading.value = false
  }
}

function toggleModel(id: string) {
  if (selectedModels.value.has(id)) {
    selectedModels.value.delete(id)
  } else {
    selectedModels.value.add(id)
  }
}

function selectAll() {
  selectedModels.value = new Set(models.value)
}

function clearAll() {
  selectedModels.value.clear()
}

function close() {
  emit('update:visible', false)
}

async function addSelected() {
  if (selectedModels.value.size === 0 || !props.providerName) return
  
  adding.value = true
  
  try {
    await invoke('add_models_batch', {
      providerName: props.providerName,
      modelIds: Array.from(selectedModels.value)
    })
    emit('added')
    close()
  } catch (e) {
    error.value = String(e)
  } finally {
    adding.value = false
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50" @click.self="close">
        <div class="w-full max-w-lg rounded-xl bg-cream-50 dark:bg-dark-800 border border-cream-300 dark:border-dark-700 shadow-xl animate-slide-up">
          <div class="px-5 py-4 border-b border-cream-300 dark:border-dark-700">
            <h3 class="font-semibold text-lg">获取站点模型</h3>
          </div>

          <div class="px-5 py-4">
            <div v-if="error" class="mb-4 px-3 py-2 rounded-lg bg-error-500/10 border border-error-500/30 text-error-500 text-sm">
              {{ error }}
            </div>

            <div v-if="loading" class="py-8 text-center text-primary-500 dark:text-dark-400">
              正在获取模型列表...
            </div>

            <div v-else-if="models.length === 0" class="py-8 text-center text-primary-500 dark:text-dark-400">
              未获取到模型
            </div>

            <div v-else>
              <div class="flex items-center justify-between mb-3">
                <span class="text-sm text-primary-500 dark:text-dark-400">
                  共 {{ models.length }} 个模型，已选 {{ selectedModels.size }} 个
                </span>
                <div class="flex gap-2">
                  <button @click="selectAll" class="text-xs text-accent-500 hover:text-accent-600">全选</button>
                  <button @click="clearAll" class="text-xs text-accent-500 hover:text-accent-600">清空</button>
                </div>
              </div>

              <div class="max-h-64 overflow-auto border border-cream-300 dark:border-dark-700 rounded-lg">
                <label
                  v-for="model in models"
                  :key="model"
                  class="flex items-center gap-3 px-3 py-2 cursor-pointer hover:bg-cream-200 dark:hover:bg-dark-700/50"
                >
                  <input
                    type="checkbox"
                    :checked="selectedModels.has(model)"
                    @change="toggleModel(model)"
                    class="w-4 h-4 rounded"
                  />
                  <span class="text-sm font-mono truncate">{{ model }}</span>
                </label>
              </div>
            </div>
          </div>

          <div class="px-5 py-4 flex justify-end gap-3 border-t border-cream-300 dark:border-dark-700">
            <button @click="close" :disabled="adding" class="px-4 py-2 text-sm font-medium rounded-lg border border-cream-400 dark:border-dark-600 hover:bg-cream-200 dark:hover:bg-dark-700 disabled:opacity-50 transition-colors">
              取消
            </button>
            <button
              @click="addSelected"
              :disabled="adding || selectedModels.size === 0"
              class="px-4 py-2 text-sm font-medium rounded-lg bg-accent-500 text-white hover:bg-accent-600 disabled:opacity-50 transition-colors"
            >
              {{ adding ? '添加中...' : `添加 ${selectedModels.size} 个模型` }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active, .fade-leave-active { transition: opacity 0.15s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
