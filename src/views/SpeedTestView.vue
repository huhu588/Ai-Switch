<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useProvidersStore } from '@/stores/providers'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()
const store = useProvidersStore()

interface SpeedTestResult {
  success: boolean
  latency_ms: number | null
  status_code: number | null
  error_message: string | null
  quality: 'excellent' | 'good' | 'fair' | 'poor' | 'failed'
}

interface BatchSpeedTestResult {
  provider_name: string
  base_url: string
  results: SpeedTestResult[]
  average_latency_ms: number | null
  success_rate: number
  overall_quality: 'excellent' | 'good' | 'fair' | 'poor' | 'failed'
}

const testResults = ref<BatchSpeedTestResult[]>([])
const testing = ref(false)
const selectedProviders = ref<string[]>([])

// 获取所有启用的 Provider
const enabledProviders = computed(() => {
  return store.providers.filter(p => p.enabled)
})

// 质量颜色映射
const qualityColors: Record<string, string> = {
  excellent: 'bg-green-500',
  good: 'bg-blue-500',
  fair: 'bg-yellow-500',
  poor: 'bg-orange-500',
  failed: 'bg-red-500',
}

// 质量标签映射
const qualityLabels: Record<string, string> = {
  excellent: '优秀',
  good: '良好',
  fair: '一般',
  poor: '较差',
  failed: '失败',
}

function toggleProvider(name: string) {
  const index = selectedProviders.value.indexOf(name)
  if (index > -1) {
    selectedProviders.value.splice(index, 1)
  } else {
    selectedProviders.value.push(name)
  }
}

function selectAll() {
  selectedProviders.value = enabledProviders.value.map(p => p.name)
}

async function runSpeedTest() {
  if (selectedProviders.value.length === 0) return
  
  testing.value = true
  testResults.value = []
  
  try {
    // 构建测试配置（API key 由后端自行获取）
    const providers = selectedProviders.value.map(name => {
      const provider = store.providers.find(p => p.name === name)
      return {
        name,
        base_url: provider?.base_url || '',
        api_key: null as string | null, // 由后端获取
        model_type: provider?.model_type || 'claude',
      }
    })
    
    testResults.value = await invoke('test_multiple_providers', { providers })
  } catch (e) {
    console.error('Speed test failed:', e)
  } finally {
    testing.value = false
  }
}

async function testSingleProvider(name: string) {
  const provider = store.providers.find(p => p.name === name)
  if (!provider) return
  
  testing.value = true
  
  try {
    const result = await invoke<BatchSpeedTestResult>('batch_test_endpoint', {
      providerName: name,
      baseUrl: provider.base_url,
      apiKey: null as string | null, // 由后端获取
      modelType: provider.model_type || 'claude',
      testCount: 3,
    })
    
    // 更新或添加结果
    const index = testResults.value.findIndex(r => r.provider_name === name)
    if (index > -1) {
      testResults.value[index] = result
    } else {
      testResults.value.push(result)
    }
    
    // 按延迟排序
    testResults.value.sort((a, b) => {
      if (a.average_latency_ms === null) return 1
      if (b.average_latency_ms === null) return -1
      return a.average_latency_ms - b.average_latency_ms
    })
  } catch (e) {
    console.error('Speed test failed:', e)
  } finally {
    testing.value = false
  }
}
</script>

<template>
  <div class="h-full flex flex-col gap-4 p-4">
    <!-- 标题栏 -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-lg bg-cyan-500/10 flex items-center justify-center">
          <SvgIcon name="activity" class="w-6 h-6 text-cyan-500" />
        </div>
        <div>
          <h1 class="text-xl font-semibold">{{ t('speedTest.title') }}</h1>
          <p class="text-sm text-gray-500">{{ t('speedTest.description') }}</p>
        </div>
      </div>
      <div class="flex gap-2">
        <button 
          @click="selectAll"
          class="px-3 py-1.5 text-sm rounded-lg bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
        >
          {{ t('speedTest.selectAll') }}
        </button>
        <button 
          @click="runSpeedTest"
          :disabled="testing || selectedProviders.length === 0"
          class="px-3 py-1.5 text-sm rounded-lg bg-blue-500 text-white hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          <span v-if="testing" class="flex items-center gap-2">
            <span class="animate-spin">⏳</span>
            {{ t('speedTest.testing') }}
          </span>
          <span v-else>{{ t('speedTest.runTest') }}</span>
        </button>
      </div>
    </div>

    <!-- Provider 选择列表 -->
    <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
      <h3 class="text-sm font-medium mb-3">{{ t('speedTest.selectProviders') }}</h3>
      
      <div v-if="enabledProviders.length === 0" class="text-center py-4 text-gray-500">
        {{ t('speedTest.noProviders') }}
      </div>
      
      <div v-else class="flex flex-wrap gap-2">
        <button
          v-for="provider in enabledProviders"
          :key="provider.name"
          @click="toggleProvider(provider.name)"
          :class="[
            'px-3 py-1.5 text-sm rounded-lg border transition-colors',
            selectedProviders.includes(provider.name)
              ? 'bg-blue-500 text-white border-blue-500'
              : 'bg-gray-100 dark:bg-gray-700 border-gray-300 dark:border-gray-600 hover:border-blue-500'
          ]"
        >
          {{ provider.name }}
        </button>
      </div>
    </div>

    <!-- 测试结果 -->
    <div class="flex-1 overflow-auto">
      <div v-if="testResults.length === 0 && !testing" class="text-center py-12 text-gray-500">
        {{ t('speedTest.noResults') }}
      </div>
      
      <div v-else class="space-y-3">
        <div
          v-for="result in testResults"
          :key="result.provider_name"
          class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700"
        >
          <div class="flex items-center justify-between mb-3">
            <div class="flex items-center gap-3">
              <div :class="['w-3 h-3 rounded-full', qualityColors[result.overall_quality]]"></div>
              <span class="font-medium">{{ result.provider_name }}</span>
              <span class="text-sm px-2 py-0.5 rounded-full bg-gray-100 dark:bg-gray-700">
                {{ qualityLabels[result.overall_quality] }}
              </span>
            </div>
            <button
              @click="testSingleProvider(result.provider_name)"
              :disabled="testing"
              class="text-sm text-blue-500 hover:text-blue-600 disabled:opacity-50"
            >
              {{ t('speedTest.retest') }}
            </button>
          </div>
          
          <div class="grid grid-cols-3 gap-4 text-sm">
            <div>
              <span class="text-gray-500">{{ t('speedTest.avgLatency') }}</span>
              <p class="font-mono text-lg">
                {{ result.average_latency_ms !== null ? `${result.average_latency_ms}ms` : 'N/A' }}
              </p>
            </div>
            <div>
              <span class="text-gray-500">{{ t('speedTest.successRate') }}</span>
              <p class="font-mono text-lg">
                {{ (result.success_rate * 100).toFixed(0) }}%
              </p>
            </div>
            <div>
              <span class="text-gray-500">{{ t('speedTest.testCount') }}</span>
              <p class="font-mono text-lg">
                {{ result.results.length }}
              </p>
            </div>
          </div>
          
          <p class="text-xs text-gray-400 mt-2 truncate">{{ result.base_url }}</p>
        </div>
      </div>
    </div>
  </div>
</template>
