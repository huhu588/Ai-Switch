import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath } from 'node:url'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  // Tauri 配置
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  },
  // 优化依赖预构建
  optimizeDeps: {
    include: [
      'vue',
      'vue-router',
      'pinia',
      'vue-i18n',
      '@tauri-apps/api',
      '@tauri-apps/plugin-dialog',
      '@tauri-apps/plugin-shell',
      '@tauri-apps/plugin-updater',
      '@tauri-apps/plugin-process'
    ]
  },
  build: {
    target: ['es2021', 'chrome100', 'safari13'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    // 优化构建
    rollupOptions: {
      output: {
        manualChunks: {
          'vue-vendor': ['vue', 'vue-router', 'pinia'],
          'tauri-vendor': ['@tauri-apps/api']
        }
      }
    }
  }
})
