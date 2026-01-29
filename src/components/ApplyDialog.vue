<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { MODEL_TYPES } from '@/config/modelTypes'

const { t } = useI18n()

interface Props {
  visible: boolean
  providerNames: string[]
  modelType: string
}

const props = defineProps<Props>()

// 获取厂家显示名称
const modelTypeLabel = computed(() => {
  const type = MODEL_TYPES.find(t => t.id === props.modelType)
  return type ? type.name : props.modelType
})

const emit = defineEmits<{
  'update:visible': [value: boolean]
  applied: []
}>()

// OpenCode 应用目标
const applyToGlobal = ref(false)
const applyToProject = ref(true)

// CLI 工具应用目标
const applyToClaudeCode = ref(false)
const applyToCodex = ref(false)
const applyToGemini = ref(false)

const loading = ref(false)
const checkingStatus = ref(false)
const error = ref<string | null>(null)

watch(() => props.visible, async (visible) => {
  if (visible && props.providerNames.length > 0) {
    error.value = null
    checkingStatus.value = true
    
    // 重置 CLI 工具选项
    applyToClaudeCode.value = false
    applyToCodex.value = false
    applyToGemini.value = false
    
    try {
      // 检查第一个 provider 是否已应用到全局/项目配置
      const status = await invoke<{ in_global: boolean; in_project: boolean }>('check_provider_applied', {
        providerName: props.providerNames[0]
      })
      // 检查对话框是否仍然打开
      if (!props.visible) return
      applyToGlobal.value = status.in_global
      applyToProject.value = status.in_project
    } catch (e) {
      if (!props.visible) return
      console.error('检查配置状态失败:', e)
      applyToGlobal.value = false
      applyToProject.value = true
    } finally {
      if (props.visible) {
        checkingStatus.value = false
      }
    }
  }
})

function close() {
  emit('update:visible', false)
}

async function apply() {
  if (props.providerNames.length === 0) return
  
  const hasOpenCodeTarget = applyToGlobal.value || applyToProject.value
  const hasCliTarget = applyToClaudeCode.value || applyToCodex.value || applyToGemini.value
  
  if (!hasOpenCodeTarget && !hasCliTarget) {
    error.value = t('applyConfig.selectTarget')
    return
  }

  loading.value = true
  error.value = null

  try {
    // 1. 应用到 OpenCode（如果选中）
    if (hasOpenCodeTarget) {
      await invoke('apply_config', {
        input: {
          provider_names: props.providerNames,
          apply_to_global: applyToGlobal.value,
          apply_to_project: applyToProject.value
        }
      })
    }
    
    // 2. 应用到 CLI 工具（只取第一个 provider）
    if (hasCliTarget && props.providerNames.length > 0) {
      const providerName = props.providerNames[0]
      // 从后端获取完整的 provider 信息（包含 api_key）
      const providerConfig = await invoke<{
        name: string
        api_key: string
        base_url: string
        model_type: string
      }>('get_provider_for_apply', { providerName })
      
      // 应用到 Claude Code
      if (applyToClaudeCode.value) {
        await invoke('apply_provider_to_claude_code', {
          provider: {
            api_key: providerConfig.api_key,
            base_url: providerConfig.base_url || null,
            model: null
          }
        })
      }
      
      // 应用到 Codex
      if (applyToCodex.value) {
        await invoke('apply_provider_to_codex', {
          provider: {
            api_key: providerConfig.api_key,
            base_url: providerConfig.base_url || null
          }
        })
      }
      
      // 应用到 Gemini
      if (applyToGemini.value) {
        await invoke('apply_provider_to_gemini', {
          provider: {
            api_key: providerConfig.api_key,
            base_url: providerConfig.base_url || null,
            model: null
          }
        })
      }
    }
    
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
        <div class="w-full max-w-sm rounded-xl bg-background border border-border shadow-xl animate-slide-up">
          <div class="px-5 py-4 border-b border-border">
            <h3 class="font-semibold text-lg">{{ t('applyConfig.title') }}</h3>
          </div>

          <div class="px-5 py-4 space-y-4">
            <div v-if="error" class="px-3 py-2 rounded-lg bg-error-500/10 border border-error-500/30 text-error-500 text-sm">
              {{ error }}
            </div>

            <p class="text-sm text-muted-foreground">
              将 <span class="font-semibold text-accent-500">{{ modelTypeLabel }}</span> 厂家的
              <span class="font-mono font-medium">{{ providerNames.length }}</span> 个服务商配置应用到：
            </p>
            <div class="mt-2 text-xs text-muted-foreground bg-surface rounded-lg p-2 max-h-20 overflow-y-auto">
              <div v-for="name in providerNames" :key="name" class="font-mono">• {{ name }}</div>
            </div>

            <!-- OpenCode 配置 -->
            <div class="space-y-2">
              <div class="text-xs font-medium text-muted-foreground uppercase tracking-wider">OpenCode</div>
              <div class="space-y-2 pl-1">
                <label class="flex items-center gap-3 cursor-pointer">
                  <input type="checkbox" v-model="applyToProject" class="w-4 h-4 rounded border-border" />
                  <div>
                    <div class="font-medium text-sm">{{ t('applyConfig.currentProject') }}</div>
                    <div class="text-xs text-muted-foreground">{{ t('applyConfig.projectPath') }}</div>
                  </div>
                </label>
                <label class="flex items-center gap-3 cursor-pointer">
                  <input type="checkbox" v-model="applyToGlobal" class="w-4 h-4 rounded border-border" />
                  <div>
                    <div class="font-medium text-sm">{{ t('applyConfig.globalConfig') }}</div>
                    <div class="text-xs text-muted-foreground">~/.opencode/opencode.json</div>
                  </div>
                </label>
              </div>
            </div>
            
            <!-- CLI 工具配置 -->
            <div class="space-y-2 pt-2 border-t border-border">
              <div class="text-xs font-medium text-muted-foreground uppercase tracking-wider">{{ t('applyConfig.cliTools') }}</div>
              <div class="space-y-2 pl-1">
                <label class="flex items-center gap-3 cursor-pointer">
                  <input type="checkbox" v-model="applyToClaudeCode" class="w-4 h-4 rounded border-border" />
                  <div>
                    <div class="font-medium text-sm">Claude Code</div>
                    <div class="text-xs text-muted-foreground">~/.claude/settings.json</div>
                  </div>
                </label>
                <label class="flex items-center gap-3 cursor-pointer">
                  <input type="checkbox" v-model="applyToCodex" class="w-4 h-4 rounded border-border" />
                  <div>
                    <div class="font-medium text-sm">Codex CLI</div>
                    <div class="text-xs text-muted-foreground">~/.codex/config.toml</div>
                  </div>
                </label>
                <label class="flex items-center gap-3 cursor-pointer">
                  <input type="checkbox" v-model="applyToGemini" class="w-4 h-4 rounded border-border" />
                  <div>
                    <div class="font-medium text-sm">Gemini CLI</div>
                    <div class="text-xs text-muted-foreground">~/.gemini/.env</div>
                  </div>
                </label>
              </div>
            </div>
          </div>

          <div class="px-5 py-4 flex justify-end gap-3 border-t border-border">
            <button @click="close" :disabled="loading" class="px-4 py-2 text-sm font-medium rounded-lg border border-border hover:bg-surface-hover disabled:opacity-50 transition-colors">
              {{ t('common.cancel') }}
            </button>
            <button @click="apply" :disabled="loading" class="px-4 py-2 text-sm font-medium rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all shadow-sm">
              {{ loading ? t('common.applying') : t('common.apply') }}
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
