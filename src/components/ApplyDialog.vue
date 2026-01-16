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
  applied: []
}>()

const applyToGlobal = ref(false)
const applyToProject = ref(true)
const loading = ref(false)
const error = ref<string | null>(null)

watch(() => props.visible, (visible) => {
  if (visible) {
    applyToGlobal.value = false
    applyToProject.value = true
    error.value = null
  }
})

function close() {
  emit('update:visible', false)
}

async function apply() {
  if (!props.providerName) return
  if (!applyToGlobal.value && !applyToProject.value) {
    error.value = '请至少选择一个应用目标'
    return
  }

  loading.value = true
  error.value = null

  try {
    await invoke('apply_config', {
      input: {
        provider_names: [props.providerName],
        apply_to_global: applyToGlobal.value,
        apply_to_project: applyToProject.value
      }
    })
    emit('applied')
    close()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50" @click.self="close">
        <div class="w-full max-w-sm rounded-xl bg-cream-50 dark:bg-dark-800 border border-cream-300 dark:border-dark-700 shadow-xl animate-slide-up">
          <div class="px-5 py-4 border-b border-cream-300 dark:border-dark-700">
            <h3 class="font-semibold text-lg">应用配置</h3>
          </div>

          <div class="px-5 py-4 space-y-4">
            <div v-if="error" class="px-3 py-2 rounded-lg bg-error-500/10 border border-error-500/30 text-error-500 text-sm">
              {{ error }}
            </div>

            <p class="text-sm text-primary-600 dark:text-dark-300">
              将 Provider <span class="font-mono font-medium">{{ providerName }}</span> 的配置应用到：
            </p>

            <div class="space-y-3">
              <label class="flex items-center gap-3 cursor-pointer">
                <input type="checkbox" v-model="applyToProject" class="w-4 h-4 rounded border-cream-400 dark:border-dark-600" />
                <div>
                  <div class="font-medium text-sm">当前项目</div>
                  <div class="text-xs text-primary-500 dark:text-dark-400">./.opencode/opencode.json</div>
                </div>
              </label>
              <label class="flex items-center gap-3 cursor-pointer">
                <input type="checkbox" v-model="applyToGlobal" class="w-4 h-4 rounded border-cream-400 dark:border-dark-600" />
                <div>
                  <div class="font-medium text-sm">全局配置</div>
                  <div class="text-xs text-primary-500 dark:text-dark-400">~/.opencode/opencode.json</div>
                </div>
              </label>
            </div>
          </div>

          <div class="px-5 py-4 flex justify-end gap-3 border-t border-cream-300 dark:border-dark-700">
            <button @click="close" :disabled="loading" class="px-4 py-2 text-sm font-medium rounded-lg border border-cream-400 dark:border-dark-600 hover:bg-cream-200 dark:hover:bg-dark-700 disabled:opacity-50 transition-colors">
              取消
            </button>
            <button @click="apply" :disabled="loading" class="px-4 py-2 text-sm font-medium rounded-lg bg-accent-500 text-white hover:bg-accent-600 disabled:opacity-50 transition-colors">
              {{ loading ? '应用中...' : '应用' }}
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
