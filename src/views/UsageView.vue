<script setup lang="ts">
import { ref, onMounted, computed, watch, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()

interface UsageSummary {
  totalRequests: number
  totalCost: string
  totalInputTokens: number
  totalOutputTokens: number
  totalCacheCreationTokens: number
  totalCacheReadTokens: number
  successRate: number
}

interface UsageTrend {
  period: string
  requestCount: number
  totalCost: number
  inputTokens: number
  outputTokens: number
}

interface ProxyStatus {
  running: boolean
  address: string
  port: number
  totalRequests: number
  successRequests: number
  failedRequests: number
  uptimeSeconds: number
}

interface TakeoverStatus {
  claude: boolean
  codex: boolean
  gemini: boolean
}

interface ProviderStats {
  providerId: string
  providerName: string
  requestCount: number
  totalTokens: number
  totalCost: string
  successRate: number
}

const loading = ref(false)
const period = ref<'24h' | '7d' | '30d'>('24h')
const summary = ref<UsageSummary | null>(null)
const trend = ref<UsageTrend[]>([])
const proxyStatus = ref<ProxyStatus | null>(null)
const takeoverStatus = ref<TakeoverStatus>({ claude: false, codex: false, gemini: false })
const providerStats = ref<ProviderStats[]>([])
const proxyInitialized = ref(false)

let statusInterval: number | null = null

// 计算图表最大值
const maxTrendValue = computed(() => {
  if (trend.value.length === 0) return 1
  return Math.max(...trend.value.map(t => t.totalCost), 0.01)
})

// 格式化成本
function formatCost(cost: number | string): string {
  const num = typeof cost === 'string' ? parseFloat(cost) : cost
  return `$${num.toFixed(4)}`
}

// 格式化 token 数
function formatTokens(tokens: number): string {
  if (tokens >= 1000000) {
    return `${(tokens / 1000000).toFixed(1)}M`
  }
  if (tokens >= 1000) {
    return `${(tokens / 1000).toFixed(1)}k`
  }
  return tokens.toString()
}

// 格式化运行时间
function formatUptime(seconds: number): string {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const secs = seconds % 60
  if (hours > 0) {
    return `${hours}h ${minutes}m`
  }
  if (minutes > 0) {
    return `${minutes}m ${secs}s`
  }
  return `${secs}s`
}

// 初始化代理服务
async function initProxy() {
  try {
    await invoke('init_proxy_service')
    proxyInitialized.value = true
    await loadProxyStatus()
  } catch (e) {
    console.error('初始化代理服务失败:', e)
  }
}

// 加载代理状态
async function loadProxyStatus() {
  try {
    proxyStatus.value = await invoke<ProxyStatus>('get_proxy_status')
    takeoverStatus.value = await invoke<TakeoverStatus>('get_takeover_status')
  } catch (e) {
    console.error('加载代理状态失败:', e)
  }
}

// 加载统计数据
async function loadData() {
  loading.value = true
  try {
    const [summaryData, trendData, statsData] = await Promise.all([
      invoke<UsageSummary>('get_proxy_usage_summary', { period: period.value }),
      invoke<UsageTrend[]>('get_proxy_usage_trend', { period: period.value }),
      invoke<ProviderStats[]>('get_provider_stats', { period: period.value }),
    ])
    summary.value = summaryData
    trend.value = trendData
    providerStats.value = statsData
  } catch (e) {
    console.error('加载使用统计失败:', e)
  } finally {
    loading.value = false
  }
}

// 启动代理
async function startProxy() {
  try {
    // 获取选中的应用列表
    const apps: string[] = []
    if (takeoverStatus.value.claude) apps.push('claude')
    if (takeoverStatus.value.codex) apps.push('codex')
    if (takeoverStatus.value.gemini) apps.push('gemini')
    
    if (apps.length > 0) {
      await invoke('start_proxy_with_takeover', { apps })
    } else {
      await invoke('start_proxy')
    }
    await loadProxyStatus()
  } catch (e) {
    console.error('启动代理失败:', e)
    alert(`启动代理失败: ${e}`)
  }
}

// 停止代理
async function stopProxy() {
  try {
    await invoke('stop_proxy_with_restore')
    await loadProxyStatus()
  } catch (e) {
    console.error('停止代理失败:', e)
    alert(`停止代理失败: ${e}`)
  }
}

// 切换应用接管
async function toggleTakeover(app: 'claude' | 'codex' | 'gemini') {
  const newValue = !takeoverStatus.value[app]
  try {
    await invoke('set_takeover_for_app', { appType: app, enabled: newValue })
    await loadProxyStatus()
  } catch (e) {
    console.error('切换接管失败:', e)
    alert(`切换接管失败: ${e}`)
  }
}

// 清除统计
async function clearStats() {
  if (!confirm(t('usage.confirmClear'))) return
  try {
    await invoke('clear_proxy_usage_stats')
    await loadData()
  } catch (e) {
    console.error('清除统计失败:', e)
  }
}

// 监听周期变化
watch(period, () => {
  loadData()
})

onMounted(async () => {
  await initProxy()
  await loadData()
  
  // 定期刷新代理状态
  statusInterval = window.setInterval(() => {
    if (proxyStatus.value?.running) {
      loadProxyStatus()
    }
  }, 5000)
})

onUnmounted(() => {
  if (statusInterval) {
    clearInterval(statusInterval)
  }
})
</script>

<template>
  <div class="h-full flex flex-col gap-4 p-4 overflow-y-auto">
    <!-- 标题栏 -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center">
          <SvgIcon name="activity" class="w-6 h-6 text-blue-500" />
        </div>
        <div>
          <h1 class="text-xl font-semibold">{{ t('usage.title') }}</h1>
          <p class="text-sm text-gray-500">{{ t('usage.description') }}</p>
        </div>
      </div>
      
      <!-- 时间周期选择 -->
      <div class="flex items-center gap-2">
        <div class="flex rounded-lg bg-surface border border-border overflow-hidden">
          <button
            v-for="p in (['24h', '7d', '30d'] as const)"
            :key="p"
            @click="period = p"
            :class="[
              'px-4 py-1.5 text-sm font-medium transition-colors',
              period === p 
                ? 'bg-blue-500 text-white' 
                : 'text-muted-foreground hover:bg-surface-hover'
            ]"
          >
            {{ p === '24h' ? '24小时' : p === '7d' ? '7天' : '30天' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 代理控制面板 -->
    <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
      <div class="flex items-center justify-between mb-4">
        <div class="flex items-center gap-3">
          <div :class="[
            'w-3 h-3 rounded-full',
            proxyStatus?.running ? 'bg-green-500 animate-pulse' : 'bg-gray-400'
          ]"></div>
          <h3 class="font-semibold">{{ t('usage.proxyControl') }}</h3>
          <span v-if="proxyStatus?.running" class="text-xs text-gray-500">
            {{ proxyStatus.address }}:{{ proxyStatus.port }} | 
            {{ t('usage.uptime') }}: {{ formatUptime(proxyStatus.uptimeSeconds) }}
          </span>
        </div>
        
        <button
          @click="proxyStatus?.running ? stopProxy() : startProxy()"
          :class="[
            'px-4 py-2 rounded-lg text-sm font-medium transition-colors',
            proxyStatus?.running 
              ? 'bg-red-500 hover:bg-red-600 text-white' 
              : 'bg-green-500 hover:bg-green-600 text-white'
          ]"
        >
          {{ proxyStatus?.running ? t('usage.stopProxy') : t('usage.startProxy') }}
        </button>
      </div>
      
      <!-- 接管设置 -->
      <div class="flex items-center gap-4">
        <span class="text-sm text-gray-500">{{ t('usage.takeover') }}:</span>
        
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            :checked="takeoverStatus.claude"
            @change="toggleTakeover('claude')"
            :disabled="proxyStatus?.running"
            class="w-4 h-4 rounded border-gray-300 text-blue-500 focus:ring-blue-500"
          />
          <span class="text-sm">Claude Code</span>
        </label>
        
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            :checked="takeoverStatus.codex"
            @change="toggleTakeover('codex')"
            :disabled="proxyStatus?.running"
            class="w-4 h-4 rounded border-gray-300 text-blue-500 focus:ring-blue-500"
          />
          <span class="text-sm">Codex</span>
        </label>
        
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            :checked="takeoverStatus.gemini"
            @change="toggleTakeover('gemini')"
            :disabled="proxyStatus?.running"
            class="w-4 h-4 rounded border-gray-300 text-blue-500 focus:ring-blue-500"
          />
          <span class="text-sm">Gemini CLI</span>
        </label>
      </div>
      
      <!-- 实时统计 -->
      <div v-if="proxyStatus?.running" class="flex items-center gap-6 mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
        <div class="text-center">
          <p class="text-2xl font-bold text-blue-500">{{ proxyStatus.totalRequests }}</p>
          <p class="text-xs text-gray-500">{{ t('usage.totalRequests') }}</p>
        </div>
        <div class="text-center">
          <p class="text-2xl font-bold text-green-500">{{ proxyStatus.successRequests }}</p>
          <p class="text-xs text-gray-500">{{ t('usage.success') }}</p>
        </div>
        <div class="text-center">
          <p class="text-2xl font-bold text-red-500">{{ proxyStatus.failedRequests }}</p>
          <p class="text-xs text-gray-500">{{ t('usage.failed') }}</p>
        </div>
      </div>
    </div>

    <!-- 统计卡片 -->
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <!-- 总请求数 -->
      <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm text-gray-500">{{ t('usage.totalRequests') }}</span>
          <div class="w-8 h-8 rounded-lg bg-violet-500/10 flex items-center justify-center">
            <SvgIcon name="activity" class="w-4 h-4 text-violet-500" />
          </div>
        </div>
        <p class="text-2xl font-bold">{{ summary?.totalRequests || 0 }}</p>
        <p class="text-xs text-gray-500 mt-1">
          {{ t('usage.successRate') }}: {{ (summary?.successRate || 0).toFixed(1) }}%
        </p>
      </div>

      <!-- 总成本 -->
      <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm text-gray-500">{{ t('usage.totalCost') }}</span>
          <div class="w-8 h-8 rounded-lg bg-green-500/10 flex items-center justify-center">
            <span class="text-green-500 font-bold">$</span>
          </div>
        </div>
        <p class="text-2xl font-bold">{{ formatCost(summary?.totalCost || 0) }}</p>
      </div>

      <!-- 总 Token 数 -->
      <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm text-gray-500">{{ t('usage.totalTokens') }}</span>
          <div class="w-8 h-8 rounded-lg bg-blue-500/10 flex items-center justify-center">
            <SvgIcon name="layers" class="w-4 h-4 text-blue-500" />
          </div>
        </div>
        <p class="text-2xl font-bold">{{ formatTokens((summary?.totalInputTokens || 0) + (summary?.totalOutputTokens || 0)) }}</p>
        <div class="flex gap-4 mt-1 text-xs text-gray-500">
          <span>Input: {{ formatTokens(summary?.totalInputTokens || 0) }}</span>
          <span>Output: {{ formatTokens(summary?.totalOutputTokens || 0) }}</span>
        </div>
      </div>

      <!-- 缓存 Token -->
      <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm text-gray-500">{{ t('usage.cacheTokens') }}</span>
          <div class="w-8 h-8 rounded-lg bg-orange-500/10 flex items-center justify-center">
            <SvgIcon name="save" class="w-4 h-4 text-orange-500" />
          </div>
        </div>
        <p class="text-2xl font-bold">{{ formatTokens((summary?.totalCacheCreationTokens || 0) + (summary?.totalCacheReadTokens || 0)) }}</p>
        <div class="flex gap-4 mt-1 text-xs text-gray-500">
          <span>{{ t('usage.cacheCreation') }}: {{ formatTokens(summary?.totalCacheCreationTokens || 0) }}</span>
          <span>{{ t('usage.cacheHit') }}: {{ formatTokens(summary?.totalCacheReadTokens || 0) }}</span>
        </div>
      </div>
    </div>

    <!-- 服务商统计 -->
    <div v-if="providerStats.length > 0" class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
      <h3 class="font-semibold mb-4">{{ t('usage.byProvider') }}</h3>
      <div class="space-y-3">
        <div 
          v-for="stat in providerStats" 
          :key="stat.providerId"
          class="flex items-center justify-between p-3 rounded-lg bg-gray-50 dark:bg-gray-700/50"
        >
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center">
              <span class="text-blue-500 font-bold text-sm">{{ stat.providerName.charAt(0).toUpperCase() }}</span>
            </div>
            <div>
              <p class="font-medium">{{ stat.providerName }}</p>
              <p class="text-xs text-gray-500">{{ stat.requestCount }} requests</p>
            </div>
          </div>
          <div class="text-right">
            <p class="font-bold">{{ formatCost(stat.totalCost) }}</p>
            <p class="text-xs text-gray-500">{{ formatTokens(stat.totalTokens) }} tokens</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 使用趋势图表 -->
    <div class="flex-1 min-h-[200px] p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
      <div class="flex items-center justify-between mb-4">
        <h3 class="font-semibold">{{ t('usage.trend') }}</h3>
        <span class="text-sm text-gray-500">
          {{ period === '24h' ? t('usage.past24h') : period === '7d' ? t('usage.past7d') : t('usage.past30d') }}
        </span>
      </div>
      
      <!-- 简单的柱状图 -->
      <div v-if="trend.length > 0" class="h-48 flex items-end gap-1">
        <div
          v-for="(item, index) in trend"
          :key="index"
          class="flex-1 flex flex-col items-center gap-1"
        >
          <!-- 柱子 -->
          <div 
            class="w-full bg-blue-500/20 hover:bg-blue-500/40 rounded-t transition-all cursor-pointer relative group"
            :style="{ height: `${Math.max((item.totalCost / maxTrendValue) * 100, 2)}%` }"
          >
            <!-- Tooltip -->
            <div class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 bg-gray-900 text-white text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap z-10">
              <div>{{ item.period }}</div>
              <div>{{ t('usage.requests') }}: {{ item.requestCount }}</div>
              <div>{{ t('usage.cost') }}: {{ formatCost(item.totalCost) }}</div>
              <div>Tokens: {{ formatTokens(item.inputTokens + item.outputTokens) }}</div>
            </div>
          </div>
          <!-- 标签（只显示部分） -->
          <span 
            v-if="index % Math.ceil(trend.length / 12) === 0 || index === trend.length - 1"
            class="text-[10px] text-gray-500 truncate w-full text-center"
          >
            {{ item.period.split(' ').pop() }}
          </span>
        </div>
      </div>
      
      <!-- 空状态 -->
      <div v-else class="h-48 flex items-center justify-center text-gray-400">
        {{ t('usage.noData') }}
      </div>
      
      <!-- Y 轴标签 -->
      <div v-if="trend.length > 0" class="flex justify-between text-xs text-gray-400 mt-2 px-2">
        <span>0</span>
        <span>{{ formatCost(maxTrendValue) }}</span>
      </div>
    </div>

    <!-- 底部操作 -->
    <div class="flex justify-end">
      <button
        @click="clearStats"
        class="px-4 py-2 text-sm text-red-500 hover:bg-red-500/10 rounded-lg transition-colors"
      >
        {{ t('usage.clearStats') }}
      </button>
    </div>
  </div>
</template>
