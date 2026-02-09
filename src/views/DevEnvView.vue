<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()

// ========== 类型定义 ==========

interface VersionManagerInfo {
  name: string
  installed: boolean
  version: string | null
  install_hint: string
  can_uninstall: boolean
}

interface RecommendedVersion {
  version: string
  label: string
  for_claude: boolean
}

interface DevEnvInfo {
  name: string
  id: string
  installed: boolean
  current_version: string | null
  installed_versions: string[]
  version_manager: VersionManagerInfo
  recommended_versions: RecommendedVersion[]
  icon: string
}

// ========== 状态 ==========

const environments = ref<DevEnvInfo[]>([])
const loading = ref(true)
const logs = ref<string[]>([])

// 每个环境独立的操作状态
const envStates = ref<Record<string, {
  switching: boolean
  installing: boolean
  uninstalling: boolean
  managerInstalling: boolean
  managerUninstalling: boolean
  expanded: boolean
  customVersion: string
}>>({})

// 环境 Logo 颜色映射
const envColors: Record<string, string> = {
  nodejs: '#68A063',
  python: '#3776AB',
  rust: '#DEA584',
  go: '#00ADD8',
  java: '#E76F00',
  cpp: '#00599C',
  dotnet: '#512BD4',
  php: '#777BB4',
  kotlin: '#7F52FF',
  swift: '#F05138',
}

// 环境 Logo 字母映射
const envLogos: Record<string, string> = {
  nodejs: 'N',
  python: 'Py',
  rust: 'Rs',
  go: 'Go',
  java: 'J',
  cpp: 'C++',
  dotnet: 'C#',
  php: 'PHP',
  kotlin: 'Kt',
  swift: 'Sw',
}

// ========== 操作日志 ==========

function addLog(message: string) {
  const time = new Date().toLocaleTimeString()
  logs.value.unshift(`[${time}] ${message}`)
  // 限制日志条数
  if (logs.value.length > 100) {
    logs.value = logs.value.slice(0, 100)
  }
}

// ========== 初始化环境状态 ==========

function initEnvState(envId: string) {
  if (!envStates.value[envId]) {
    envStates.value[envId] = {
      switching: false,
      installing: false,
      uninstalling: false,
      managerInstalling: false,
      managerUninstalling: false,
      expanded: false,
      customVersion: '',
    }
  }
}

// 获取环境状态（保证已初始化，安全访问）
function getEnvState(envId: string) {
  initEnvState(envId)
  return envStates.value[envId]
}

// ========== 检测环境 ==========

async function detectAll() {
  loading.value = true
  addLog('开始检测所有编程环境...')
  try {
    environments.value = await invoke<DevEnvInfo[]>('detect_all_dev_envs')
    environments.value.forEach(env => initEnvState(env.id))
    const installed = environments.value.filter(e => e.installed).length
    addLog(`检测完成：共 ${environments.value.length} 个环境，已安装 ${installed} 个`)
  } catch (e) {
    addLog(`${t('devenv.detectFailed')}: ${e}`)
  } finally {
    loading.value = false
  }
}

// 刷新单个环境
async function refreshEnv(envId: string) {
  try {
    const env = await invoke<DevEnvInfo>('detect_single_dev_env', { envName: envId })
    const idx = environments.value.findIndex(e => e.id === envId)
    if (idx !== -1) {
      environments.value[idx] = env
    }
    addLog(`已刷新 ${env.name} 环境信息`)
  } catch (e) {
    addLog(`刷新失败: ${e}`)
  }
}

// ========== 版本切换 ==========

async function switchVersion(envId: string, version: string) {
  const state = envStates.value[envId]
  if (!state || state.switching) return
  state.switching = true
  addLog(`正在切换 ${envId} 到 v${version}...`)
  try {
    const result = await invoke<string>('switch_env_version', { envName: envId, version })
    addLog(result)
    // 刷新环境信息
    await refreshEnv(envId)
  } catch (e) {
    addLog(`${t('devenv.switchFailed')}: ${e}`)
  } finally {
    state.switching = false
  }
}

// ========== 安装版本 ==========

async function installVersion(envId: string, version: string) {
  const state = envStates.value[envId]
  if (!state || state.installing) return
  state.installing = true
  addLog(`正在安装 ${envId} v${version}...`)
  try {
    const result = await invoke<string>('install_env_version', { envName: envId, version })
    addLog(result)
    // 刷新环境信息
    await refreshEnv(envId)
  } catch (e) {
    addLog(`${t('devenv.installFailed')}: ${e}`)
  } finally {
    state.installing = false
  }
}

// 安装自定义版本
async function installCustomVersion(envId: string) {
  const state = envStates.value[envId]
  if (!state || !state.customVersion.trim()) return
  await installVersion(envId, state.customVersion.trim())
  state.customVersion = ''
}

// ========== 卸载环境 ==========

async function uninstallEnv(envId: string, version: string) {
  const state = envStates.value[envId]
  if (!state || state.uninstalling) return
  state.uninstalling = true
  addLog(`正在卸载 ${envId} v${version}...`)
  try {
    const result = await invoke<string>('uninstall_env_version', { envName: envId, version })
    addLog(result)
    await refreshEnv(envId)
  } catch (e) {
    addLog(`${t('devenv.uninstallFailed')}: ${e}`)
  } finally {
    state.uninstalling = false
  }
}

// ========== 安装版本管理器 ==========

async function installManager(envId: string) {
  const state = envStates.value[envId]
  if (!state || state.managerInstalling) return
  state.managerInstalling = true
  addLog(`正在安装 ${envId} 版本管理器...`)
  try {
    const result = await invoke<string>('install_version_manager', { envName: envId })
    addLog(result)
    // 共享管理器（Scoop）：任意一个安装应联动全部刷新
    if (['java','php','kotlin'].includes(envId)) {
      await detectAll()
    } else {
      await refreshEnv(envId)
    }
  } catch (e) {
    addLog(`${t('devenv.managerInstallFailed')}: ${e}`)
  } finally {
    state.managerInstalling = false
  }
}

// ========== 卸载版本管理器 ==========

async function uninstallManager(envId: string) {
  const state = envStates.value[envId]
  if (!state || state.managerUninstalling) return
  state.managerUninstalling = true
  addLog(`正在卸载 ${envId} 版本管理器...`)
  try {
    const result = await invoke<string>('uninstall_version_manager', { envName: envId })
    addLog(result)
    // 共享管理器（Scoop）：任意一个卸载应联动全部刷新
    if (['java','php','kotlin'].includes(envId)) {
      await detectAll()
    } else {
      await refreshEnv(envId)
    }
  } catch (e) {
    addLog(`${t('devenv.managerUninstallFailed')}: ${e}`)
  } finally {
    state.managerUninstalling = false
  }
}

// ========== 生命周期 ==========

onMounted(() => {
  detectAll()
})
</script>

<template>
  <div class="max-w-3xl mx-auto">
    <div class="rounded-xl bg-surface/30 border border-border p-6">
      <!-- 标题栏 -->
      <div class="flex items-center justify-between mb-6">
        <div class="flex items-center gap-3">
          <SvgIcon name="code" :size="32" class="text-accent" />
          <div>
            <h2 class="text-xl font-semibold">{{ t('devenv.title') }}</h2>
            <p class="text-xs text-muted-foreground mt-0.5">{{ t('devenv.subtitle') }}</p>
          </div>
        </div>
        <button
          @click="detectAll"
          :disabled="loading"
          class="px-4 py-2 text-sm font-medium text-accent bg-accent/10 hover:bg-accent/20 rounded-lg transition-colors disabled:opacity-50 flex items-center gap-2"
        >
          <svg v-if="loading" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <SvgIcon v-else name="refresh" :size="16" />
          {{ loading ? t('devenv.detecting') : t('devenv.detectAll') }}
        </button>
      </div>

      <!-- 加载状态 -->
      <div v-if="loading && environments.length === 0" class="py-12 text-center text-muted-foreground">
        <svg class="w-8 h-8 animate-spin mx-auto mb-3 text-accent" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        {{ t('devenv.detecting') }}
      </div>

      <!-- 环境卡片列表 -->
      <div v-else class="space-y-4">
        <div
          v-for="env in environments"
          :key="env.id"
          class="rounded-lg bg-surface border border-border overflow-hidden transition-all duration-200"
        >
          <!-- 卡片主体 -->
          <div class="p-4 flex items-center gap-4">
            <!-- 环境 Logo -->
            <div
              class="flex items-center justify-center w-12 h-12 rounded-xl text-white font-bold text-lg shrink-0 shadow-sm"
              :style="{ backgroundColor: envColors[env.id] || '#6366f1' }"
            >
              {{ envLogos[env.id] || env.name.charAt(0) }}
            </div>

            <!-- 环境信息 -->
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="font-semibold text-base">{{ env.name }}</span>
                <span
                  v-if="env.installed"
                  class="px-2 py-0.5 rounded-full text-[10px] font-medium bg-emerald-500/15 text-emerald-400"
                >
                  {{ t('devenv.installed') }}
                </span>
                <span
                  v-else
                  class="px-2 py-0.5 rounded-full text-[10px] font-medium bg-zinc-500/15 text-zinc-400"
                >
                  {{ t('devenv.notInstalled') }}
                </span>
              </div>
              <div class="flex items-center gap-4 mt-1 text-xs text-muted-foreground">
                <span>
                  {{ t('devenv.currentVersion') }}:
                  <span class="text-primary font-mono">{{ env.current_version || t('devenv.noVersion') }}</span>
                </span>
                <span class="flex items-center gap-1">
                  {{ env.version_manager.name }}:
                  <span v-if="env.version_manager.installed" class="text-emerald-400">
                    v{{ env.version_manager.version || '?' }}
                  </span>
                  <span v-else class="text-zinc-400">{{ t('devenv.managerNotInstalled') }}</span>
                </span>
              </div>
            </div>

            <!-- 操作按钮 -->
            <div class="flex items-center gap-2 shrink-0">
              <!-- 安装管理器按钮（未安装时显示） -->
              <button
                v-if="!env.version_manager.installed"
                @click="installManager(env.id)"
                :disabled="envStates[env.id]?.managerInstalling"
                class="px-3 py-1.5 text-xs font-medium rounded-lg bg-amber-500/15 text-amber-400 hover:bg-amber-500/25 transition-colors disabled:opacity-50"
              >
                {{ envStates[env.id]?.managerInstalling ? t('devenv.installing') : t('devenv.installManager') }}
              </button>

              <!-- 卸载管理器按钮（专用管理器已安装时显示，共享管理器如 scoop/winget 不显示） -->
              <button
                v-if="env.version_manager.installed && env.version_manager.can_uninstall"
                @click="uninstallManager(env.id)"
                :disabled="envStates[env.id]?.managerUninstalling"
                class="px-3 py-1.5 text-xs font-medium rounded-lg bg-orange-500/15 text-orange-400 hover:bg-orange-500/25 transition-colors disabled:opacity-50"
              >
                {{ envStates[env.id]?.managerUninstalling ? t('devenv.managerUninstalling') : t('devenv.uninstallManager') }}
              </button>

              <!-- 卸载按钮（已安装时显示） -->
              <button
                v-if="env.installed"
                @click="uninstallEnv(env.id, env.current_version || '')"
                :disabled="envStates[env.id]?.uninstalling"
                class="px-3 py-1.5 text-xs font-medium rounded-lg bg-red-500/15 text-red-400 hover:bg-red-500/25 transition-colors disabled:opacity-50"
              >
                {{ envStates[env.id]?.uninstalling ? t('devenv.uninstalling') : t('devenv.uninstall') }}
              </button>

              <!-- 展开/折叠按钮 -->
              <button
                @click="envStates[env.id] && (envStates[env.id].expanded = !envStates[env.id].expanded)"
                class="p-2 rounded-lg hover:bg-surface-hover transition-colors text-muted-foreground hover:text-primary"
              >
                <svg
                  class="w-4 h-4 transition-transform duration-200"
                  :class="{ 'rotate-180': envStates[env.id]?.expanded }"
                  fill="none" stroke="currentColor" viewBox="0 0 24 24"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
              </button>
            </div>
          </div>

          <!-- 展开面板 -->
          <div
            v-if="envStates[env.id]?.expanded"
            class="border-t border-border/50 p-4 space-y-4 bg-background/30"
          >
            <!-- 已安装版本列表 + 切换 -->
            <div v-if="env.installed_versions.length > 0">
              <h4 class="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-2">
                {{ t('devenv.installedVersions') }}
              </h4>
              <div class="flex flex-wrap gap-2">
                <div
                  v-for="ver in env.installed_versions"
                  :key="ver"
                  class="group flex items-center gap-1 px-3 py-1.5 text-xs rounded-lg border transition-all"
                  :class="[
                    ver === env.current_version
                      ? 'border-accent bg-accent/10 text-accent font-semibold'
                      : 'border-border hover:border-accent/50 hover:bg-surface-hover text-muted-foreground'
                  ]"
                >
                  <button
                    @click="switchVersion(env.id, ver)"
                    :disabled="envStates[env.id]?.switching || ver === env.current_version"
                    class="disabled:cursor-default"
                  >
                    <span class="font-mono">v{{ ver }}</span>
                    <span v-if="ver === env.current_version" class="ml-1">✓</span>
                  </button>
                  <!-- 单版本卸载按钮（非当前版本 + 有多个版本时显示） -->
                  <button
                    v-if="ver !== env.current_version && env.installed_versions.length > 1"
                    @click.stop="uninstallEnv(env.id, ver)"
                    :disabled="envStates[env.id]?.uninstalling"
                    class="ml-1 opacity-0 group-hover:opacity-100 text-red-400 hover:text-red-300 transition-opacity"
                    :title="t('devenv.uninstall')"
                  >
                    <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              </div>
              <p v-if="envStates[env.id]?.switching" class="text-xs text-accent mt-2 flex items-center gap-1">
                <svg class="w-3 h-3 animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
                </svg>
                {{ t('devenv.switching') }}
              </p>
            </div>

            <!-- 推荐版本安装 -->
            <div>
              <h4 class="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-2">
                {{ t('devenv.recommendedVersions') }}
              </h4>
              <div class="space-y-2">
                <div
                  v-for="rec in env.recommended_versions"
                  :key="rec.version"
                  class="flex items-center justify-between p-3 rounded-lg bg-surface border border-border/50"
                >
                  <div class="flex items-center gap-3">
                    <span class="font-mono text-sm font-medium text-primary">v{{ rec.version }}</span>
                    <span
                      v-if="rec.for_claude"
                      class="px-2 py-0.5 rounded-full text-[10px] font-medium bg-purple-500/15 text-purple-400"
                    >
                      {{ t('devenv.forClaude') }}
                    </span>
                    <span
                      v-else
                      class="px-2 py-0.5 rounded-full text-[10px] font-medium bg-blue-500/15 text-blue-400"
                    >
                      {{ t('devenv.stableVersion') }}
                    </span>
                    <span class="text-xs text-muted-foreground">{{ rec.label }}</span>
                  </div>
                  <button
                    @click="installVersion(env.id, rec.version)"
                    :disabled="envStates[env.id]?.installing || env.installed_versions.includes(rec.version)"
                    class="px-3 py-1 text-xs font-medium rounded-lg transition-colors disabled:opacity-40"
                    :class="[
                      env.installed_versions.includes(rec.version)
                        ? 'bg-emerald-500/10 text-emerald-400'
                        : 'bg-accent/10 text-accent hover:bg-accent/20'
                    ]"
                  >
                    {{ env.installed_versions.includes(rec.version) ? t('devenv.installed') : t('devenv.installVersion') }}
                  </button>
                </div>
              </div>
            </div>

            <!-- 自定义版本安装 -->
            <div>
              <h4 class="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-2">
                {{ t('devenv.customVersion') }}
              </h4>
              <div class="flex gap-2">
                <input
                  v-model="getEnvState(env.id).customVersion"
                  :placeholder="t('devenv.customVersionPlaceholder')"
                  class="flex-1 px-3 py-2 text-sm bg-background border border-border rounded-lg focus:border-accent focus:outline-none font-mono"
                  @keyup.enter="installCustomVersion(env.id)"
                />
                <button
                  @click="installCustomVersion(env.id)"
                  :disabled="envStates[env.id]?.installing || !envStates[env.id]?.customVersion?.trim()"
                  class="px-4 py-2 text-sm font-medium rounded-lg bg-accent/10 text-accent hover:bg-accent/20 transition-colors disabled:opacity-50"
                >
                  {{ envStates[env.id]?.installing ? t('devenv.installingVersion') : t('devenv.installVersion') }}
                </button>
              </div>
            </div>

            <!-- 版本管理器信息 -->
            <div class="p-3 rounded-lg bg-surface/50 border border-border/30 text-xs text-muted-foreground">
              <span class="font-medium text-primary">{{ t('devenv.versionManager') }}:</span>
              {{ env.version_manager.name }}
              <span v-if="env.version_manager.installed" class="text-emerald-400 ml-1">
                (v{{ env.version_manager.version }})
              </span>
              <span v-else class="text-zinc-400 ml-1">
                — {{ env.version_manager.install_hint }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- 操作日志 -->
      <div v-if="logs.length > 0" class="mt-6">
        <div class="flex items-center justify-between mb-2">
          <h3 class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            {{ t('devenv.operationLog') }}
          </h3>
          <button
            @click="logs = []"
            class="text-[10px] text-muted-foreground hover:text-primary transition-colors"
          >
            {{ t('devenv.clearLogs') }}
          </button>
        </div>
        <div class="bg-background/50 border border-border/50 rounded-lg p-3 max-h-48 overflow-y-auto font-mono text-xs space-y-1">
          <div
            v-for="(log, idx) in logs"
            :key="idx"
            class="text-muted-foreground leading-relaxed"
          >
            {{ log }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
