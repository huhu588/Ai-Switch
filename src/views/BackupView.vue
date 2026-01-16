<script setup lang="ts">
</script>

<template>
  <div class="max-w-2xl mx-auto">
    <div class="rounded-xl bg-cream-50 dark:bg-dark-800 border border-cream-300 dark:border-dark-700 p-6">
      <div class="flex items-center gap-3 mb-6">
        <span class="text-3xl">💾</span>
        <h2 class="text-xl font-semibold">备份与恢复</h2>
      </div>

      <div class="space-y-6">
        <section>
          <h3 class="font-medium mb-2">配置备份功能</h3>
          <p class="text-sm text-primary-600 dark:text-dark-300">
            备份功能支持将您的 Coding Agent 配置备份到 WebDAV 服务器。
          </p>
        </section>

        <section>
          <h3 class="font-medium mb-2">支持的备份类型</h3>
          <ul class="text-sm text-primary-600 dark:text-dark-300 space-y-1">
            <li>• OpenCode 配置</li>
            <li>• MCP 服务器配置</li>
          </ul>
        </section>

        <section>
          <h3 class="font-medium mb-2">使用说明</h3>
          <p class="text-sm text-primary-600 dark:text-dark-300 mb-3">
            由于备份涉及网络操作，请使用命令行模式执行备份/恢复：
          </p>
          <div class="bg-cream-100 dark:bg-dark-900 rounded-lg p-4 font-mono text-sm space-y-2">
            <div class="flex items-center gap-2">
              <span class="text-accent-500">$</span>
              <span>openswitch backup</span>
              <span class="text-primary-500 dark:text-dark-400 ml-4"># 创建备份</span>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-accent-500">$</span>
              <span>openswitch restore</span>
              <span class="text-primary-500 dark:text-dark-400 ml-4"># 恢复备份</span>
            </div>
          </div>
        </section>

        <section class="pt-4 border-t border-cream-300 dark:border-dark-700">
          <div class="flex items-start gap-3 text-sm">
            <span class="text-xl">💡</span>
            <div>
              <p class="font-medium">提示</p>
              <p class="text-primary-500 dark:text-dark-400">
                备份功能需要配置 WebDAV 服务器。您可以使用坚果云、NextCloud 等支持 WebDAV 的服务。
              </p>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>
