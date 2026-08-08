<script setup lang="ts">
import { ref, computed } from 'vue'
import { useDataStore } from '../stores/data'
import { ElMessage, ElMessageBox } from 'element-plus'

defineProps<{ openQuickSearch: () => void }>()

const dataStore = useDataStore()
const expanded = ref(false)
const refreshing = ref(false)

const fileName = computed(() => {
  if (!dataStore.dataSource) return '未导入'
  const p = dataStore.dataSource.filePath
  const parts = p.split(/[/\\]/)
  return parts[parts.length - 1] || p
})
const fileBaseName = computed(() => fileName.value.replace(/\.[^.]+$/, ''))

const sourceAge = computed(() => {
  if (!dataStore.dataSource) return null
  const ms = Date.now() - dataStore.dataSource.updatedAt
  const min = Math.floor(ms / 60000)
  if (min < 1) return '刚刚'
  if (min < 60) return min + ' 分钟前'
  const h = Math.floor(min / 60)
  if (h < 24) return h + ' 小时前'
  return Math.floor(h / 24) + ' 天前'
})

async function pickFile() {
  try {
    await dataStore.pickAndImport()
    ElMessage.success('已更换数据源')
  } catch (e: any) { ElMessage.error('更换失败: ' + (e.message || e)) }
}
async function reload() {
  refreshing.value = true
  try { await dataStore.reload(); ElMessage.success('已刷新') }
  catch (e: any) { ElMessage.error('刷新失败: ' + (e.message || e)) }
  finally { refreshing.value = false }
}
async function clearData() {
  try {
    await ElMessageBox.confirm('确定清除当前数据源?已导入的数据库会保留', '确认', { type: 'warning' })
    await dataStore.clear()
    ElMessage.success('已清除')
  } catch { /* 取消 */ }
}
</script>

<template>
  <div class="src-bubble" :class="{ expanded, empty: !dataStore.dataSource }">
    <div v-if="!expanded" class="src-pill" @click="expanded = true">
      <span class="src-status" :class="{ active: !!dataStore.dataSource }"></span>
      <span class="src-label" v-if="dataStore.dataSource">
        <span class="src-filename">{{ fileBaseName }}</span>
        <span class="src-age">· {{ sourceAge }}</span>
      </span>
      <span class="src-label muted" v-else>未导入数据源</span>
    </div>
    <div v-else class="src-panel">
      <div class="src-panel-header">
        <span class="src-title">📂 数据源</span>
        <button class="src-collapse" @click="expanded = false" title="折叠">×</button>
      </div>
      <div class="src-panel-body" v-if="dataStore.dataSource">
        <div class="src-row"><span class="src-key">文件</span><span class="src-val" :title="dataStore.dataSource.filePath">{{ fileName }}</span></div>
        <div class="src-row" v-if="dataStore.dataSource.sheetName"><span class="src-key">Sheet</span><span class="src-val">{{ dataStore.dataSource.sheetName }}</span></div>
        <div class="src-row" v-if="dataStore.lastImport"><span class="src-key">规模</span><span class="src-val">{{ dataStore.lastImport.row_count }} 行 · 耗时 {{ dataStore.lastImport.duration_ms }}ms</span></div>
        <div class="src-row"><span class="src-key">更新</span><span class="src-val">{{ sourceAge }}</span></div>
      </div>
      <div class="src-panel-body muted" v-else>还没有数据源,选一个 Excel/CSV 开始</div>
      <div class="src-panel-actions">
        <el-button v-if="!dataStore.dataSource" type="primary" size="small" @click="pickFile">📁 选择文件</el-button>
        <template v-else>
          <el-button size="small" :loading="refreshing" @click="reload">🔄 刷新</el-button>
          <el-button size="small" @click="pickFile">📁 更换</el-button>
          <el-button size="small" type="danger" plain @click="clearData">清除</el-button>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.src-bubble { position: fixed; right: 20px; bottom: 20px; z-index: 100; font-size: 12px; }
.src-pill { display: flex; align-items: center; gap: 8px; padding: 6px 12px; background: #fff; border: 1px solid #e4e7ed; border-radius: 100px; cursor: pointer; box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04); transition: all 0.15s; user-select: none; max-width: 280px; }
.src-pill:hover { border-color: #c0c4cc; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08); }
.src-status { width: 6px; height: 6px; border-radius: 50%; background: #c0c4cc; flex-shrink: 0; }
.src-status.active { background: #67c23a; box-shadow: 0 0 0 2px rgba(103, 194, 58, 0.2); }
.src-label { color: #606266; display: flex; gap: 4px; align-items: baseline; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.src-label.muted { color: #c0c4cc; }
.src-filename { font-weight: 500; color: #303133; }
.src-age { color: #c0c4cc; font-size: 11px; }
.src-bubble.empty .src-pill { opacity: 0.6; }
.src-panel { width: 320px; background: #fff; border: 1px solid #e4e7ed; border-radius: 10px; box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1); overflow: hidden; }
.src-panel-header { display: flex; align-items: center; padding: 10px 14px; background: #fafbfc; border-bottom: 1px solid #f0f0f0; }
.src-title { font-weight: 600; color: #303133; font-size: 13px; }
.src-collapse { margin-left: auto; background: none; border: none; font-size: 18px; color: #909399; cursor: pointer; padding: 0 4px; line-height: 1; }
.src-collapse:hover { color: #303133; }
.src-panel-body { padding: 12px 14px; }
.src-panel-body.muted { color: #909399; font-size: 12px; text-align: center; padding: 16px; }
.src-row { display: flex; align-items: baseline; gap: 8px; padding: 4px 0; font-size: 12px; }
.src-key { color: #909399; width: 40px; flex-shrink: 0; }
.src-val { color: #303133; flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.src-panel-actions { display: flex; gap: 4px; padding: 8px 10px 10px; background: #fafbfc; border-top: 1px solid #f0f0f0; justify-content: flex-end; flex-wrap: wrap; }
</style>
