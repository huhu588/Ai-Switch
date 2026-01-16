<script setup lang="ts">
import type { ProviderItem, ModelItem } from '@/stores/providers'

interface Props {
  provider?: ProviderItem
  model?: ModelItem
}

const props = defineProps<Props>()

// 格式化 token 数量
function formatTokens(count: number | null): string {
  if (count === null) return '-'
  if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M`
  if (count >= 1000) return `${(count / 1000).toFixed(0)}k`
  return String(count)
}
</script>

<template>
  <div class="h-full rounded-xl bg-cream-50 dark:bg-dark-800 border border-cream-300 dark:border-dark-700 overflow-hidden">
    <!-- 标题栏 -->
    <div class="px-4 py-3 border-b border-cream-300 dark:border-dark-700">
      <h3 class="font-semibold text-sm">详情</h3>
    </div>

    <!-- 内容 -->
    <div class="p-4">
      <div v-if="!provider" class="text-center text-primary-500 dark:text-dark-400 py-8">
        选择一个 Provider 查看详情
      </div>

      <div v-else class="space-y-6">
        <!-- Provider 信息 -->
        <section>
          <h4 class="text-xs font-semibold uppercase tracking-wide text-primary-500 dark:text-dark-400 mb-3">
            Provider 信息
          </h4>
          <div class="space-y-2">
            <div class="flex items-start gap-3">
              <span class="text-sm text-primary-500 dark:text-dark-400 w-16 shrink-0">名称</span>
              <span class="text-sm font-medium">{{ provider.name }}</span>
            </div>
            <div class="flex items-start gap-3">
              <span class="text-sm text-primary-500 dark:text-dark-400 w-16 shrink-0">URL</span>
              <span class="text-sm font-mono text-accent-600 dark:text-accent-400 break-all">{{ provider.base_url }}</span>
            </div>
            <div class="flex items-start gap-3">
              <span class="text-sm text-primary-500 dark:text-dark-400 w-16 shrink-0">模型数</span>
              <span class="text-sm">{{ provider.model_count }} 个</span>
            </div>
            <div v-if="provider.description" class="flex items-start gap-3">
              <span class="text-sm text-primary-500 dark:text-dark-400 w-16 shrink-0">描述</span>
              <span class="text-sm">{{ provider.description }}</span>
            </div>
          </div>
        </section>

        <!-- Model 信息 -->
        <section v-if="model">
          <h4 class="text-xs font-semibold uppercase tracking-wide text-primary-500 dark:text-dark-400 mb-3">
            选中模型
          </h4>
          <div class="space-y-2">
            <div class="flex items-start gap-3">
              <span class="text-sm text-primary-500 dark:text-dark-400 w-16 shrink-0">ID</span>
              <span class="text-sm font-mono">{{ model.id }}</span>
            </div>
            <div class="flex items-start gap-3">
              <span class="text-sm text-primary-500 dark:text-dark-400 w-16 shrink-0">名称</span>
              <span class="text-sm">{{ model.name }}</span>
            </div>
            <div v-if="model.context_limit" class="flex items-start gap-3">
              <span class="text-sm text-primary-500 dark:text-dark-400 w-16 shrink-0">Context</span>
              <span class="text-sm">{{ formatTokens(model.context_limit) }} tokens</span>
            </div>
            <div v-if="model.output_limit" class="flex items-start gap-3">
              <span class="text-sm text-primary-500 dark:text-dark-400 w-16 shrink-0">Output</span>
              <span class="text-sm">{{ formatTokens(model.output_limit) }} tokens</span>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>
