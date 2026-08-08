<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { ElMessage, ElNotification } from 'element-plus'
import { useDataStore } from './stores/data'
import { api } from './api'
import UpdateNotifier from './components/UpdateNotifier.vue'
import QuickSearch from './components/QuickSearch.vue'
import SourceInfoBubble from './components/SourceInfoBubble.vue'

const dataStore = useDataStore()
const appInfo = ref<any>(null)
const quickSearchOpen = ref(false)
// 用相对路径 (不能 /brand-logo.png,Tauri 用 tauri:// 协议)
const appIconUrl = ref<string>('./brand-logo.png')

function openQuickSearch() { quickSearchOpen.value = true }

function onGlobalKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault()
    quickSearchOpen.value = !quickSearchOpen.value
  } else if (e.key === '/' && !quickSearchOpen.value) {
    const target = e.target as HTMLElement
    const tag = target?.tagName?.toLowerCase()
    if (tag !== 'input' && tag !== 'textarea' && !target?.isContentEditable) {
      e.preventDefault()
      quickSearchOpen.value = true
    }
  } else if (e.key === 'Escape' && quickSearchOpen.value) {
    e.preventDefault()
    quickSearchOpen.value = false
  }
}

let unlistens: Array<() => void> = []

onMounted(async () => {
  await dataStore.loadDataSource()
  try {
    appInfo.value = await api.getAppInfo()
  } catch (e) { console.error(e) }

  const u1 = await api.onAutoReloaded((data) => {
    ElNotification.success({
      title: '数据已自动更新',
      message: `共 ${data.rowCount} 行,刚刚从共享盘自动重载`
    })
    if (dataStore.lastImport) {
      dataStore.lastImport = { ...dataStore.lastImport, row_count: data.rowCount, imported_at: data.importedAt } as any
    }
  })
  const u2 = await api.onAutoReloadFailed((data) => {
    ElMessage.error(`自动重载失败: ${data.error}`)
  })
  unlistens.push(u1, u2)

  window.addEventListener('keydown', onGlobalKeydown)
  window.addEventListener('open-quick-search', openQuickSearch)
})

onUnmounted(() => {
  unlistens.forEach(fn => fn())
  window.removeEventListener('keydown', onGlobalKeydown)
  window.removeEventListener('open-quick-search', openQuickSearch)
})
</script>

<template>
  <div class="app">
    <el-container style="height: 100vh">
      <el-header class="app-header">
        <div class="brand">
          <img v-if="appIconUrl" :src="appIconUrl" class="brand-icon-img" alt="logo" />
          <span v-else class="brand-icon">📊</span>
          <div class="brand-text">
            <div class="brand-name">智算信息查询系统</div>
            <div class="brand-sub">(数据中心事业部)</div>
          </div>
          <span class="brand-version">v{{ appInfo?.version || '2.0.0' }}</span>
        </div>
        <div class="spacer"></div>
        <div class="brand-meta">
          <span class="brand-platform">{{ appInfo?.platform }} · {{ appInfo?.arch }}</span>
        </div>
      </el-header>
      <el-main class="app-main">
        <router-view />
        <UpdateNotifier />
        <SourceInfoBubble :open-quick-search="openQuickSearch" />
        <QuickSearch v-model="quickSearchOpen" />
      </el-main>
    </el-container>
  </div>
</template>

<style scoped>
.app-header {
  background: #fff;
  border-bottom: 1px solid #ebeef5;
  height: 48px !important;
  padding: 0 20px;
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.brand {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 15px;
  font-weight: 600;
  color: #303133;
}

.brand-icon { font-size: 18px; }

.brand-icon-img {
  height: 28px;
  width: auto;
  max-width: 240px;
  border-radius: 2px;
  object-fit: contain;
  display: block;
  flex-shrink: 0;
  align-self: center;
  background: #fff;
  padding: 1px 4px;
  image-rendering: -webkit-optimize-contrast;
  image-rendering: crisp-edges;
}

.brand-text {
  display: flex;
  flex-direction: column;
  gap: 0;
  line-height: 1.15;
}

.brand-name {
  font-size: 16px;
  font-weight: 700;
  color: #1a3a6e;
  letter-spacing: 1px;
}

.brand-sub {
  font-size: 11px;
  font-weight: 500;
  color: #606266;
  letter-spacing: 0.5px;
  margin-top: 1px;
}

.brand-version {
  font-size: 11px;
  font-weight: 400;
  color: #c0c4cc;
  margin-left: 2px;
}

.brand-meta { font-size: 11px; color: #c0c4cc; }
.spacer { flex: 1; }
.app-main { padding: 0; background: #f5f7fa; overflow-y: auto; }
</style>
