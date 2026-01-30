<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { setLocale, getLocale, getNextLocale } from '@/i18n'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()

const currentLocale = computed(() => getLocale())

// 当前语言的显示名称
const currentLocaleName = computed(() => {
  const locale = currentLocale.value
  if (locale === 'zh-CN') return t('language.zh')
  if (locale === 'ja') return t('language.ja')
  return t('language.en')
})

function toggleLanguage() {
  const nextLocale = getNextLocale()
  setLocale(nextLocale)
}
</script>

<template>
  <button
    @click="toggleLanguage"
    class="flex w-full items-center justify-between rounded-md border border-border bg-background/50 px-3 py-2 text-xs font-medium transition-all hover:border-accent/40 hover:bg-surface-hover active:scale-[0.98]"
    :title="t('language.switch')"
  >
    <div class="flex items-center gap-2">
      <SvgIcon name="book" :size="14" />
      <span class="text-muted-foreground group-hover:text-primary transition-colors">
        {{ currentLocaleName }}
      </span>
    </div>
    <div class="flex items-center gap-1 text-[10px] text-muted-foreground font-mono">
      <span :class="{ 'text-accent font-medium': currentLocale === 'zh-CN' }">中</span>
      <span>/</span>
      <span :class="{ 'text-accent font-medium': currentLocale === 'en' }">EN</span>
      <span>/</span>
      <span :class="{ 'text-accent font-medium': currentLocale === 'ja' }">JP</span>
    </div>
  </button>
</template>
