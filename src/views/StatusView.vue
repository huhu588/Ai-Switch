<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import SvgIcon from '@/components/SvgIcon.vue'
import UpdateDialog from '@/components/UpdateDialog.vue'

const { t } = useI18n()

// 更新相关
const updateDialogRef = ref<InstanceType<typeof UpdateDialog> | null>(null)
const isCheckingUpdate = ref(false)
const updateMessage = ref('')

// 关闭行为设置
type CloseAction = 'ask' | 'tray' | 'quit'
const closeAction = ref<CloseAction>('ask')

// 加载关闭行为设置
async function loadCloseAction() {
  try {
    const action = await invoke<string>('get_close_action')
    closeAction.value = action as CloseAction
  } catch (e) {
    console.error('加载关闭行为设置失败:', e)
  }
}

// 保存关闭行为设置
async function setCloseAction(action: CloseAction) {
  try {
    await invoke('set_close_action', { action })
    closeAction.value = action
  } catch (e) {
    console.error('保存关闭行为设置失败:', e)
  }
}

async function checkForUpdates() {
  isCheckingUpdate.value = true
  updateMessage.value = ''
  try {
    await updateDialogRef.value?.checkForUpdate()
  } catch (e) {
    updateMessage.value = t('status.noUpdates')
  } finally {
    isCheckingUpdate.value = false
  }
}

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
  loadCloseAction()
})
</script>

<template>
  <div class="max-w-2xl mx-auto">
    <div class="rounded-xl bg-surface/30 border border-border p-6">
      <div class="flex items-center gap-3 mb-6">
        <SvgIcon name="activity" :size="32" class="text-accent" />
        <h2 class="text-xl font-semibold">{{ t('status.title') }}</h2>
      </div>

      <div v-if="loading" class="py-8 text-center text-muted-foreground">
        {{ t('common.loading') }}
      </div>

      <div v-else-if="status" class="space-y-6">
        <!-- 版本信息 -->
        <section>
          <h3 class="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-3">
            {{ t('status.appInfo') }}
          </h3>
          <div class="grid grid-cols-2 gap-4">
            <div class="bg-surface rounded-lg p-4">
              <div class="flex items-center justify-between">
                <div>
                  <div class="text-2xl font-bold">v{{ version }}</div>
                  <div class="text-xs text-muted-foreground">{{ t('status.currentVersion') }}</div>
                </div>
                <button
                  @click="checkForUpdates"
                  :disabled="isCheckingUpdate"
                  class="px-3 py-1.5 text-xs font-medium text-accent bg-accent/10 hover:bg-accent/20 rounded-lg transition-colors disabled:opacity-50 flex items-center gap-1.5"
                >
                  <svg v-if="isCheckingUpdate" class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  <svg v-else class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>
                  {{ isCheckingUpdate ? t('status.checking') : t('status.checkUpdate') }}
                </button>
              </div>
              <p v-if="updateMessage" class="text-xs text-emerald-400 mt-2">{{ updateMessage }}</p>
            </div>
            <div class="bg-surface rounded-lg p-4">
              <div class="text-2xl font-bold">{{ status.provider_count }}</div>
              <div class="text-xs text-muted-foreground">{{ t('status.providerCount') }}</div>
            </div>
          </div>
        </section>

        <!-- 配置状态 -->
        <section>
          <h3 class="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-3">
            {{ t('status.configStatus') }}
          </h3>
          <div class="space-y-3">
            <div class="flex items-center justify-between py-2 border-b border-border">
              <span class="text-sm">{{ t('status.globalConfig') }}</span>
              <span class="text-emerald-500">{{ t('status.configured') }}</span>
            </div>
            <div class="flex items-center justify-between py-2 border-b border-border">
              <span class="text-sm">{{ t('status.projectConfig') }}</span>
              <span :class="status.has_project_config ? 'text-emerald-500' : 'text-muted-foreground'">
                {{ status.has_project_config ? t('status.configured') : t('status.notConfigured') }}
              </span>
            </div>
            <div class="flex items-center justify-between py-2 border-b border-border">
              <span class="text-sm">{{ t('status.currentProvider') }}</span>
              <span class="font-mono text-sm">{{ status.active_provider || '-' }}</span>
            </div>
            <div class="flex items-center justify-between py-2">
              <span class="text-sm">{{ t('status.mcpServers') }}</span>
              <span>{{ t('status.count', { count: status.mcp_server_count }) }}</span>
            </div>
          </div>
        </section>

        <!-- 配置路径 -->
        <section>
          <h3 class="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-3">
            {{ t('status.configPaths') }}
          </h3>
          <div class="space-y-2 text-sm">
            <div class="flex items-start gap-3">
              <span class="text-muted-foreground w-20 shrink-0">{{ t('status.globalConfig') }}</span>
              <span class="font-mono text-xs break-all">{{ status.config_paths.global_config_dir }}</span>
            </div>
            <div class="flex items-start gap-3">
              <span class="text-muted-foreground w-20 shrink-0">{{ t('status.openCode') }}</span>
              <span class="font-mono text-xs break-all">{{ status.config_paths.global_opencode_dir }}</span>
            </div>
            <div v-if="status.config_paths.project_opencode_dir" class="flex items-start gap-3">
              <span class="text-muted-foreground w-20 shrink-0">{{ t('status.projectConfig') }}</span>
              <span class="font-mono text-xs break-all">{{ status.config_paths.project_opencode_dir }}</span>
            </div>
          </div>
        </section>
      </div>
    </div>

    <!-- 应用设置 -->
    <div class="rounded-xl bg-surface/30 border border-border p-6 mt-6">
      <div class="flex items-center gap-3 mb-6">
        <SvgIcon name="setting" :size="32" class="text-accent" />
        <h2 class="text-xl font-semibold">{{ t('settings.title') }}</h2>
      </div>
      
      <div class="space-y-4">
        <p class="text-sm text-muted-foreground">{{ t('settings.description') }}</p>
        
        <!-- 关闭窗口时的行为 -->
        <div class="bg-surface rounded-lg p-4">
          <div class="flex items-center justify-between">
            <div>
              <div class="font-medium">{{ t('settings.closeAction') }}</div>
              <div class="text-xs text-muted-foreground mt-1">{{ t('settings.closeActionDesc') }}</div>
            </div>
            <div class="flex gap-2">
              <button
                @click="setCloseAction('ask')"
                class="px-3 py-1.5 text-sm rounded-lg transition-all flex items-center gap-1.5"
                :class="closeAction === 'ask' 
                  ? 'bg-accent text-white' 
                  : 'bg-surface-hover hover:bg-accent/20 text-foreground'"
              >
                <SvgIcon name="info" :size="14" />
                {{ t('settings.closeAsk') }}
              </button>
              <button
                @click="setCloseAction('tray')"
                class="px-3 py-1.5 text-sm rounded-lg transition-all flex items-center gap-1.5"
                :class="closeAction === 'tray' 
                  ? 'bg-accent text-white' 
                  : 'bg-surface-hover hover:bg-accent/20 text-foreground'"
              >
                <SvgIcon name="monitor" :size="14" />
                {{ t('settings.closeTray') }}
              </button>
              <button
                @click="setCloseAction('quit')"
                class="px-3 py-1.5 text-sm rounded-lg transition-all flex items-center gap-1.5"
                :class="closeAction === 'quit' 
                  ? 'bg-accent text-white' 
                  : 'bg-surface-hover hover:bg-accent/20 text-foreground'"
              >
                <SvgIcon name="close" :size="14" />
                {{ t('settings.closeQuit') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 更新对话框 -->
    <UpdateDialog ref="updateDialogRef" />
  </div>
</template>
