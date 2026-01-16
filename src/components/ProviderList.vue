<script setup lang="ts">
import type { ProviderItem } from '@/stores/providers'

interface Props {
  providers: ProviderItem[]
  selected: string | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  select: [name: string]
  add: []
  edit: [name: string]
  delete: [name: string]
  apply: []
}>()
</script>

<template>
  <div class="h-full flex flex-col rounded-xl bg-cream-50 dark:bg-dark-800 border border-cream-300 dark:border-dark-700 overflow-hidden">
    <!-- 标题栏 -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-cream-300 dark:border-dark-700">
      <h3 class="font-semibold text-sm">Providers</h3>
      <span class="text-xs text-primary-500 dark:text-dark-400">({{ providers.length }})</span>
    </div>

    <!-- 工具栏 -->
    <div class="flex items-center gap-2 px-3 py-2 border-b border-cream-200 dark:border-dark-700/50">
      <button
        @click="emit('add')"
        class="flex-1 px-3 py-1.5 text-xs font-medium rounded-lg bg-accent-500 text-white hover:bg-accent-600 transition-colors"
      >
        + 添加
      </button>
      <button
        @click="emit('apply')"
        :disabled="!selected"
        class="px-3 py-1.5 text-xs font-medium rounded-lg border border-cream-400 dark:border-dark-600 hover:bg-cream-200 dark:hover:bg-dark-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        应用
      </button>
    </div>

    <!-- 列表 -->
    <div class="flex-1 overflow-auto">
      <div v-if="providers.length === 0" class="p-4 text-center text-sm text-primary-500 dark:text-dark-400">
        暂无 Provider
      </div>
      <ul v-else class="p-2 space-y-1">
        <li
          v-for="provider in providers"
          :key="provider.name"
          @click="emit('select', provider.name)"
          class="group px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-150"
          :class="[
            provider.name === selected
              ? 'bg-accent-100 dark:bg-accent-900/30 border border-accent-300 dark:border-accent-700'
              : 'hover:bg-cream-200 dark:hover:bg-dark-700/50 border border-transparent'
          ]"
        >
          <div class="flex items-center justify-between">
            <span class="font-medium text-sm truncate">{{ provider.name }}</span>
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                @click.stop="emit('edit', provider.name)"
                class="p-1 rounded hover:bg-cream-300 dark:hover:bg-dark-600"
                title="编辑"
              >
                ✏️
              </button>
              <button
                @click.stop="emit('delete', provider.name)"
                class="p-1 rounded hover:bg-error-500/20"
                title="删除"
              >
                🗑️
              </button>
            </div>
          </div>
          <div class="mt-1 flex items-center gap-2 text-xs text-primary-500 dark:text-dark-400">
            <span>{{ provider.model_count }} 模型</span>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>
