import { createI18n } from 'vue-i18n'
import zhCN from './locales/zh-CN'
import en from './locales/en'
import ja from './locales/ja'

export type MessageSchema = typeof zhCN
export type LocaleType = 'zh-CN' | 'en' | 'ja'

// 获取存储的语言偏好或使用系统语言
function getDefaultLocale(): LocaleType {
  const savedLocale = localStorage.getItem('locale')
  if (savedLocale && ['zh-CN', 'en', 'ja'].includes(savedLocale)) {
    return savedLocale as LocaleType
  }
  
  // 检测操作系统语言设置
  const systemLang = navigator.language.toLowerCase()
  if (systemLang.startsWith('zh')) {
    return 'zh-CN'
  }
  if (systemLang.startsWith('ja')) {
    return 'ja'
  }
  return 'en'
}

const i18n = createI18n<[MessageSchema], LocaleType>({
  legacy: false, // 使用 Composition API 模式
  locale: getDefaultLocale(),
  fallbackLocale: 'en',
  messages: {
    'zh-CN': zhCN,
    'en': en,
    'ja': ja
  }
})

export default i18n

// 语言列表（循环切换顺序）
export const locales: LocaleType[] = ['zh-CN', 'en', 'ja']

// 切换语言的辅助函数
export function setLocale(locale: LocaleType) {
  ;(i18n.global.locale as any).value = locale
  localStorage.setItem('locale', locale)
  const htmlLang = locale === 'zh-CN' ? 'zh-CN' : locale === 'ja' ? 'ja' : 'en'
  document.documentElement.lang = htmlLang
}

// 获取当前语言
export function getLocale(): LocaleType {
  return (i18n.global.locale as any).value as LocaleType
}

// 获取下一个语言（用于循环切换）
export function getNextLocale(): LocaleType {
  const current = getLocale()
  const currentIndex = locales.indexOf(current)
  const nextIndex = (currentIndex + 1) % locales.length
  return locales[nextIndex]
}
