<script setup lang="ts">
interface Props {
  visible: boolean
  title?: string
  message?: string
  confirmText?: string
  cancelText?: string
  danger?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  title: '确认',
  message: '确定要执行此操作吗？',
  confirmText: '确认',
  cancelText: '取消',
  danger: false
})

const emit = defineEmits<{
  'update:visible': [value: boolean]
  confirm: []
  cancel: []
}>()

function close() {
  emit('update:visible', false)
}

function confirm() {
  emit('confirm')
  close()
}

function cancel() {
  emit('cancel')
  close()
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50" @click.self="cancel">
        <div class="w-full max-w-sm rounded-xl bg-cream-50 dark:bg-dark-800 border border-cream-300 dark:border-dark-700 shadow-xl animate-slide-up">
          <!-- 标题 -->
          <div class="px-5 py-4 border-b border-cream-300 dark:border-dark-700">
            <h3 class="font-semibold text-lg">{{ title }}</h3>
          </div>

          <!-- 内容 -->
          <div class="px-5 py-4">
            <p class="text-sm text-primary-600 dark:text-dark-300">{{ message }}</p>
          </div>

          <!-- 按钮 -->
          <div class="px-5 py-4 flex justify-end gap-3 border-t border-cream-300 dark:border-dark-700">
            <button
              @click="cancel"
              class="px-4 py-2 text-sm font-medium rounded-lg border border-cream-400 dark:border-dark-600 hover:bg-cream-200 dark:hover:bg-dark-700 transition-colors"
            >
              {{ cancelText }}
            </button>
            <button
              @click="confirm"
              class="px-4 py-2 text-sm font-medium rounded-lg text-white transition-colors"
              :class="danger ? 'bg-error-500 hover:bg-error-600' : 'bg-accent-500 hover:bg-accent-600'"
            >
              {{ confirmText }}
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
