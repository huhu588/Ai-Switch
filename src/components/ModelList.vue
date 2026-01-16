<script setup lang="ts">
import type { ModelItem } from '@/stores/providers'

interface Props {
  models: ModelItem[]
  selected: string | null
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false
})

const emit = defineEmits<{
  select: [id: string]
  add: []
  delete: [id: string]
  fetch: []
}>()

// 格式化 token 数量
function formatTokens(count: number | null): string {
  if (count === null) return '-'
  if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M`
  if (count >= 1000) return `${(count / 1000).toFixed(0)}k`
  return String(count)
}
</script>

<template>
  <div class="h-full flex flex-col rounded-xl bg-cream-50 dark:bg-dark-800 border border-cream-300 dark:border-dark-700 overflow-hidden"
       :class="{ 'opacity-60': disabled }">
    <!-- 标题栏 -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-cream-300 dark:border-dark-700">
      <h3 class="font-semibold text-sm">Models</h3>
      <span class="text-xs text-primary-500 dark:text-dark-400">({{ models.length }})</span>
    </div>

    <!-- 工具栏 -->
    <div class="flex items-center gap-2 px-3 py-2 border-b border-cream-200 dark:border-dark-700/50">
      <button
        @click="emit('add')"
        :disabled="disabled"
        class="flex-1 px-3 py-1.5 text-xs font-medium rounded-lg bg-accent-500 text-white hover:bg-accent-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        + 添加
      </button>
      <button
        @click="emit('fetch')"
        :disabled="disabled"
        class="px-3 py-1.5 text-xs font-medium rounded-lg border border-cream-400 dark:border-dark-600 hover:bg-cream-200 dark:hover:bg-dark-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        title="从站点获取模型列表"
      >
        🔄
      </button>
    </div>

    <!-- 列表 -->
    <div class="flex-1 overflow-auto">
      <div v-if="disabled" class="p-4 text-center text-sm text-primary-500 dark:text-dark-400">
        请先选择 Provider
      </div>
      <div v-else-if="models.length === 0" class="p-4 text-center text-sm text-primary-500 dark:text-dark-400">
        暂无模型
      </div>
      <ul v-else class="p-2 space-y-1">
        <li
          v-for="model in models"
          :key="model.id"
          @click="emit('select', model.id)"
          class="group px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-150"
          :class="[
            model.id === selected
              ? 'bg-accent-100 dark:bg-accent-900/30 border border-accent-300 dark:border-accent-700'
              : 'hover:bg-cream-200 dark:hover:bg-dark-700/50 border border-transparent'
          ]"
        >
          <div class="flex items-center justify-between">
            <span class="font-medium text-sm truncate font-mono">{{ model.id }}</span>
            <button
              @click.stop="emit('delete', model.id)"
              class="p-1 rounded hover:bg-error-500/20 opacity-0 group-hover:opacity-100 transition-opacity"
              title="删除"
            >
              🗑️
            </button>
          </div>
          <div v-if="model.context_limit || model.output_limit" class="mt-1 flex items-center gap-3 text-xs text-primary-500 dark:text-dark-400">
            <span v-if="model.context_limit">Context: {{ formatTokens(model.context_limit) }}</span>
            <span v-if="model.output_limit">Output: {{ formatTokens(model.output_limit) }}</span>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>
