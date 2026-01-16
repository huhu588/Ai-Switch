<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'

const route = useRoute()

// 主题状态
const isDark = ref(true)

// 导航菜单
const navItems = [
  { name: 'Providers', path: '/', icon: '🔌' },
  { name: 'MCP', path: '/mcp', icon: '🖥️' },
  { name: 'Backup', path: '/backup', icon: '💾' },
  { name: 'Status', path: '/status', icon: '📊' },
]

// 版本号
const version = ref('')

// 切换主题
function toggleTheme() {
  isDark.value = !isDark.value
  if (isDark.value) {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
  localStorage.setItem('theme', isDark.value ? 'dark' : 'light')
}

// 初始化主题
function initTheme() {
  const savedTheme = localStorage.getItem('theme')
  if (savedTheme === 'light') {
    isDark.value = false
    document.documentElement.classList.remove('dark')
  } else {
    isDark.value = true
    document.documentElement.classList.add('dark')
  }
}

onMounted(async () => {
  initTheme()
  try {
    version.value = await invoke<string>('get_version')
  } catch (e) {
    version.value = '0.7.0'
  }
})
</script>

<template>
  <div class="h-screen flex bg-cream-100 dark:bg-dark-900 text-primary-800 dark:text-dark-100 transition-colors duration-200">
    <!-- 侧边栏 -->
    <aside class="w-64 flex flex-col border-r border-cream-400 dark:border-dark-700 bg-cream-50 dark:bg-dark-800/50">
      <!-- Logo -->
      <div class="h-16 flex items-center px-6 border-b border-cream-400 dark:border-dark-700">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-lg bg-accent-500 dark:bg-accent-600 flex items-center justify-center text-white font-bold">
            ◇
          </div>
          <div>
            <h1 class="font-semibold text-lg">Open Switch</h1>
            <p class="text-xs text-primary-500 dark:text-dark-400">v{{ version }}</p>
          </div>
        </div>
      </div>

      <!-- 导航菜单 -->
      <nav class="flex-1 py-4 px-3">
        <ul class="space-y-1">
          <li v-for="item in navItems" :key="item.path">
            <router-link
              :to="item.path"
              class="flex items-center gap-3 px-4 py-2.5 rounded-lg transition-all duration-150"
              :class="[
                route.path === item.path
                  ? 'bg-accent-100 dark:bg-accent-900/30 text-accent-700 dark:text-accent-300'
                  : 'text-primary-600 dark:text-dark-300 hover:bg-cream-200 dark:hover:bg-dark-700/50'
              ]"
            >
              <span class="text-lg">{{ item.icon }}</span>
              <span class="font-medium">{{ item.name }}</span>
            </router-link>
          </li>
        </ul>
      </nav>

      <!-- 底部设置 -->
      <div class="p-4 border-t border-cream-400 dark:border-dark-700">
        <button
          @click="toggleTheme"
          class="w-full flex items-center justify-center gap-2 px-4 py-2 rounded-lg bg-cream-200 dark:bg-dark-700 hover:bg-cream-300 dark:hover:bg-dark-600 transition-colors"
        >
          <span>{{ isDark ? '🌙' : '☀️' }}</span>
          <span class="text-sm">{{ isDark ? '深色模式' : '浅色模式' }}</span>
        </button>
      </div>
    </aside>

    <!-- 主内容区 -->
    <main class="flex-1 flex flex-col overflow-hidden">
      <!-- 顶部标题栏 -->
      <header class="h-16 flex items-center justify-between px-6 border-b border-cream-400 dark:border-dark-700 bg-cream-50/50 dark:bg-dark-800/30">
        <h2 class="text-xl font-semibold">
          {{ navItems.find(item => item.path === route.path)?.name || 'Open Switch' }}
        </h2>
        <div class="flex items-center gap-4">
          <span class="text-sm text-primary-500 dark:text-dark-400">
            Coding Agent 配置管理工具
          </span>
        </div>
      </header>

      <!-- 页面内容 -->
      <div class="flex-1 overflow-auto p-6">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </div>
    </main>
  </div>
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
