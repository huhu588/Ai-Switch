<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface Props {
  visible: boolean
  editing?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  editing: null
})

const emit = defineEmits<{
  'update:visible': [value: boolean]
  saved: []
}>()

// 表单数据
const form = ref({
  name: '',
  api_key: '',
  base_url: '',
  description: ''
})

const loading = ref(false)
const error = ref<string | null>(null)

// 监听 editing 变化，加载数据
watch(() => props.visible, async (visible) => {
  if (visible && props.editing) {
    try {
      const provider = await invoke<any>('get_provider', { name: props.editing })
      if (provider) {
        form.value = {
          name: props.editing,
          api_key: provider.options.api_key || '',
          base_url: provider.options.base_url || '',
          description: provider.metadata?.description || ''
        }
      }
    } catch (e) {
      console.error('加载 Provider 失败:', e)
    }
  } else if (visible) {
    // 添加模式，清空表单
    form.value = {
      name: '',
      api_key: '',
      base_url: 'https://api.openai.com/v1',
      description: ''
    }
  }
  error.value = null
})

function close() {
  emit('update:visible', false)
}

async function save() {
  if (!form.value.name.trim()) {
    error.value = '请输入名称'
    return
  }
  if (!form.value.api_key.trim()) {
    error.value = '请输入 API Key'
    return
  }

  loading.value = true
  error.value = null

  try {
    if (props.editing) {
      await invoke('update_provider', {
        name: props.editing,
        input: {
          name: form.value.name,
          api_key: form.value.api_key,
          base_url: form.value.base_url || 'https://api.openai.com/v1',
          description: form.value.description || null
        }
      })
    } else {
      await invoke('add_provider', {
        input: {
          name: form.value.name,
          api_key: form.value.api_key,
          base_url: form.value.base_url || 'https://api.openai.com/v1',
          description: form.value.description || null
        }
      })
    }
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
        <div class="w-full max-w-md rounded-xl bg-cream-50 dark:bg-dark-800 border border-cream-300 dark:border-dark-700 shadow-xl animate-slide-up">
          <!-- 标题 -->
          <div class="px-5 py-4 border-b border-cream-300 dark:border-dark-700">
            <h3 class="font-semibold text-lg">{{ editing ? '编辑 Provider' : '添加 Provider' }}</h3>
          </div>

          <!-- 表单 -->
          <div class="px-5 py-4 space-y-4">
            <!-- 错误提示 -->
            <div v-if="error" class="px-3 py-2 rounded-lg bg-error-500/10 border border-error-500/30 text-error-500 text-sm">
              {{ error }}
            </div>

            <!-- 名称 -->
            <div>
              <label class="block text-sm font-medium mb-1.5">名称 *</label>
              <input
                v-model="form.name"
                type="text"
                placeholder="my-provider"
                :disabled="!!editing"
                class="w-full px-3 py-2 rounded-lg border border-cream-400 dark:border-dark-600 bg-cream-100 dark:bg-dark-700 disabled:opacity-60"
              />
            </div>

            <!-- API Key -->
            <div>
              <label class="block text-sm font-medium mb-1.5">API Key *</label>
              <input
                v-model="form.api_key"
                type="password"
                placeholder="sk-..."
                class="w-full px-3 py-2 rounded-lg border border-cream-400 dark:border-dark-600 bg-cream-100 dark:bg-dark-700 font-mono"
              />
            </div>

            <!-- Base URL -->
            <div>
              <label class="block text-sm font-medium mb-1.5">Base URL</label>
              <input
                v-model="form.base_url"
                type="text"
                placeholder="https://api.openai.com/v1"
                class="w-full px-3 py-2 rounded-lg border border-cream-400 dark:border-dark-600 bg-cream-100 dark:bg-dark-700 font-mono text-sm"
              />
            </div>

            <!-- 描述 -->
            <div>
              <label class="block text-sm font-medium mb-1.5">描述</label>
              <input
                v-model="form.description"
                type="text"
                placeholder="可选"
                class="w-full px-3 py-2 rounded-lg border border-cream-400 dark:border-dark-600 bg-cream-100 dark:bg-dark-700"
              />
            </div>
          </div>

          <!-- 按钮 -->
          <div class="px-5 py-4 flex justify-end gap-3 border-t border-cream-300 dark:border-dark-700">
            <button
              @click="close"
              :disabled="loading"
              class="px-4 py-2 text-sm font-medium rounded-lg border border-cream-400 dark:border-dark-600 hover:bg-cream-200 dark:hover:bg-dark-700 disabled:opacity-50 transition-colors"
            >
              取消
            </button>
            <button
              @click="save"
              :disabled="loading"
              class="px-4 py-2 text-sm font-medium rounded-lg bg-accent-500 text-white hover:bg-accent-600 disabled:opacity-50 transition-colors"
            >
              {{ loading ? '保存中...' : '保存' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
