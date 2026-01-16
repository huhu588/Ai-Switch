<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useProvidersStore } from '@/stores/providers'
import ProviderList from '@/components/ProviderList.vue'
import ModelList from '@/components/ModelList.vue'
import DetailPanel from '@/components/DetailPanel.vue'
import ProviderDialog from '@/components/ProviderDialog.vue'
import ModelDialog from '@/components/ModelDialog.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import ApplyDialog from '@/components/ApplyDialog.vue'
import FetchModelsDialog from '@/components/FetchModelsDialog.vue'

const store = useProvidersStore()

// 对话框状态
const showProviderDialog = ref(false)
const showModelDialog = ref(false)
const showDeleteDialog = ref(false)
const showApplyDialog = ref(false)
const showFetchModelsDialog = ref(false)
const editingProvider = ref<string | null>(null)
const deleteTarget = ref<{ type: 'provider' | 'model'; name: string } | null>(null)

// 加载数据
onMounted(() => {
  store.loadProviders()
})

// 添加 Provider
function openAddProvider() {
  editingProvider.value = null
  showProviderDialog.value = true
}

// 编辑 Provider
function openEditProvider(name: string) {
  editingProvider.value = name
  showProviderDialog.value = true
}

// 删除 Provider
function openDeleteProvider(name: string) {
  deleteTarget.value = { type: 'provider', name }
  showDeleteDialog.value = true
}

// 添加 Model
function openAddModel() {
  showModelDialog.value = true
}

// 删除 Model
function openDeleteModel(id: string) {
  deleteTarget.value = { type: 'model', name: id }
  showDeleteDialog.value = true
}

// 确认删除
async function confirmDelete() {
  if (!deleteTarget.value) return
  
  try {
    if (deleteTarget.value.type === 'provider') {
      await store.deleteProvider(deleteTarget.value.name)
    } else {
      await store.deleteModel(deleteTarget.value.name)
    }
  } catch (e) {
    console.error('删除失败:', e)
  }
  
  showDeleteDialog.value = false
  deleteTarget.value = null
}

// 应用配置
function openApplyDialog() {
  if (store.selectedProvider) {
    showApplyDialog.value = true
  }
}

// 获取站点模型
function openFetchModels() {
  if (store.selectedProvider) {
    showFetchModelsDialog.value = true
  }
}
</script>

<template>
  <div class="h-full flex gap-4">
    <!-- Provider 列表 -->
    <div class="w-64 flex-shrink-0">
      <ProviderList
        :providers="store.providers"
        :selected="store.selectedProvider"
        @select="store.selectProvider"
        @add="openAddProvider"
        @edit="openEditProvider"
        @delete="openDeleteProvider"
        @apply="openApplyDialog"
      />
    </div>

    <!-- Model 列表 -->
    <div class="w-72 flex-shrink-0">
      <ModelList
        :models="store.models"
        :selected="store.selectedModel"
        :disabled="!store.selectedProvider"
        @select="id => store.selectedModel = id"
        @add="openAddModel"
        @delete="openDeleteModel"
        @fetch="openFetchModels"
      />
    </div>

    <!-- 详情面板 -->
    <div class="flex-1 min-w-0">
      <DetailPanel
        :provider="store.currentProvider"
        :model="store.models.find(m => m.id === store.selectedModel)"
      />
    </div>

    <!-- Provider 对话框 -->
    <ProviderDialog
      v-model:visible="showProviderDialog"
      :editing="editingProvider"
      @saved="store.loadProviders()"
    />

    <!-- Model 对话框 -->
    <ModelDialog
      v-model:visible="showModelDialog"
      :provider-name="store.selectedProvider"
      @saved="store.loadModels()"
    />

    <!-- 删除确认对话框 -->
    <ConfirmDialog
      v-model:visible="showDeleteDialog"
      title="确认删除"
      :message="`确定要删除${deleteTarget?.type === 'provider' ? 'Provider' : 'Model'} '${deleteTarget?.name}' 吗？`"
      confirm-text="删除"
      danger
      @confirm="confirmDelete"
    />

    <!-- 应用配置对话框 -->
    <ApplyDialog
      v-model:visible="showApplyDialog"
      :provider-name="store.selectedProvider"
      @applied="() => {}"
    />

    <!-- 获取站点模型对话框 -->
    <FetchModelsDialog
      v-model:visible="showFetchModelsDialog"
      :provider-name="store.selectedProvider"
      @added="store.loadModels()"
    />
  </div>
</template>
