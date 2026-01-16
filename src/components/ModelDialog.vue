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
  saved: []
}>()

const form = ref({
  id: '',
  name: ''
})

const loading = ref(false)
const error = ref<string | null>(null)

watch(() => props.visible, (visible) => {
  if (visible) {
    form.value = { id: '', name: '' }
    error.value = null
  }
})

function close() {
  emit('update:visible', false)
}

async function save() {
  if (!form.value.id.trim()) {
    error.value = '请输入 Model ID'
    return
  }
  if (!props.providerName) {
    error.value = '未选择 Provider'
    return
  }

  loading.value = true
  error.value = null

  try {
    await invoke('add_model', {
      providerName: props.providerName,
      input: {
        id: form.value.id,
        name: form.value.name || form.value.id
      }
    })
    emit('saved')
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
            <h3 class="font-semibold text-lg">添加 Model</h3>
          </div>

          <div class="px-5 py-4 space-y-4">
            <div v-if="error" class="px-3 py-2 rounded-lg bg-error-500/10 border border-error-500/30 text-error-500 text-sm">
              {{ error }}
            </div>

            <div>
              <label class="block text-sm font-medium mb-1.5">Model ID *</label>
              <input
                v-model="form.id"
                type="text"
                placeholder="gpt-4o"
                class="w-full px-3 py-2 rounded-lg border border-cream-400 dark:border-dark-600 bg-cream-100 dark:bg-dark-700 font-mono"
              />
            </div>

            <div>
              <label class="block text-sm font-medium mb-1.5">显示名称</label>
              <input
                v-model="form.name"
                type="text"
                placeholder="可选，默认使用 ID"
                class="w-full px-3 py-2 rounded-lg border border-cream-400 dark:border-dark-600 bg-cream-100 dark:bg-dark-700"
              />
            </div>
          </div>

          <div class="px-5 py-4 flex justify-end gap-3 border-t border-cream-300 dark:border-dark-700">
            <button @click="close" :disabled="loading" class="px-4 py-2 text-sm font-medium rounded-lg border border-cream-400 dark:border-dark-600 hover:bg-cream-200 dark:hover:bg-dark-700 disabled:opacity-50 transition-colors">
              取消
            </button>
            <button @click="save" :disabled="loading" class="px-4 py-2 text-sm font-medium rounded-lg bg-accent-500 text-white hover:bg-accent-600 disabled:opacity-50 transition-colors">
              {{ loading ? '添加中...' : '添加' }}
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
