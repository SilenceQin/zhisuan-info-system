<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { ElMessage } from 'element-plus'
import DetailCard from './DetailCard.vue'
import { api, type DataCategory } from '../api'

interface SearchHit {
  category: DataCategory
  sheetName: string
  row: any
  title: string
  subtitle: string
  score: number
  titleMatch?: [number, number]
}

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>()

const allData = ref<Record<DataCategory, Array<{ sheetName: string; row: any }>>>({
  chips: [], servers: [], super_nodes: [], other: []
})
const loaded = ref(false)

const keyword = ref('')
const selectedIndex = ref(0)
const selectedDetail = ref<SearchHit | null>(null)
const detailVisible = ref(false)
const inputRef = ref<any>(null)

const categoryMeta: Record<DataCategory, { label: string; icon: string; iconImg: string | null; color: string }> = {
  chips: { label: '芯片库', icon: '🔬', iconImg: './cat-chips.png', color: '#409eff' },
  servers: { label: '服务器库', icon: '🖥️', iconImg: './cat-servers.png', color: '#67c23a' },
  super_nodes: { label: '超节点库', icon: '🧠', iconImg: './cat-supernodes.png', color: '#e6a23c' },
  other: { label: '其他', icon: '📋', iconImg: null, color: '#909399' }
}

async function ensureLoaded() {
  if (loaded.value) return
  loaded.value = true
  for (const cat of ['chips', 'servers', 'super_nodes'] as DataCategory[]) {
    try {
      const tables = await api.listByCategory(cat)
      const merged: Array<{ sheetName: string; row: any }> = []
      for (const t of tables) {
        const rowsRes = await api.queryAllRows(t.table_name)
        if (rowsRes.rows) {
          for (const r of rowsRes.rows) {
            merged.push({ sheetName: t.sheet_name || cat, row: r })
          }
        }
      }
      allData.value[cat] = merged
    } catch (e) {
      console.error(e)
    }
  }
}

function close() {
  emit('update:modelValue', false)
  keyword.value = ''
  selectedIndex.value = 0
}

watch(() => props.modelValue, async (v) => {
  if (v) { await ensureLoaded(); await nextTick(); inputRef.value?.focus() }
})

const titleKeyMap: Record<DataCategory, string[]> = {
  chips: ['芯片型号', '型号', '名称'],
  servers: ['服务器名称', '名称'],
  super_nodes: ['超节点名称', '名称'],
  other: ['名称', '型号']
}
const subtitleKeyMap: Record<DataCategory, string[]> = {
  chips: ['芯片厂商', '产品形态', '芯片架构'],
  servers: ['厂商', '芯片'],
  super_nodes: ['厂商', '芯片'],
  other: ['名称']
}

function getVal(row: any, keys: string[]): any {
  for (const k of keys) {
    if (row[k] !== undefined && row[k] !== null && row[k] !== '') return row[k]
  }
  return undefined
}

function matchScore(text: string, kw: string): { score: number; range?: [number, number] } {
  if (!kw) return { score: 1 }
  const t = text.toLowerCase()
  const k = kw.toLowerCase()
  if (t === k) return { score: 1000, range: [0, text.length] }
  if (t.startsWith(k)) return { score: 500, range: [0, k.length] }
  const idx = t.indexOf(k)
  if (idx >= 0) return { score: 100 - idx, range: [idx, idx + k.length] }
  let i = 0, j = 0, lastMatch = -1, matched = 0
  while (i < t.length && j < k.length) {
    if (t[i] === k[j]) { if (lastMatch === i - 1) matched++; lastMatch = i; j++ }
    i++
  }
  if (j === k.length) return { score: 10 + matched }
  return { score: 0 }
}

const results = computed<SearchHit[]>(() => {
  const kw = keyword.value.trim()
  if (!kw) {
    const out: SearchHit[] = []
    for (const cat of ['chips', 'servers', 'super_nodes'] as DataCategory[]) {
      for (const item of allData.value[cat].slice(0, 5)) {
        const title = String(getVal(item.row, titleKeyMap[cat]) || '-')
        const subtitle = String(getVal(item.row, subtitleKeyMap[cat]) || '')
        out.push({ category: cat, sheetName: item.sheetName, row: item.row, title, subtitle, score: 0 })
      }
    }
    return out
  }
  const hits: SearchHit[] = []
  for (const cat of ['chips', 'servers', 'super_nodes'] as DataCategory[]) {
    for (const item of allData.value[cat]) {
      let bestScore = 0
      let titleMatch: [number, number] | undefined
      const title = String(getVal(item.row, titleKeyMap[cat]) || '')
      const tScore = matchScore(title, kw)
      if (tScore.score > 0) { bestScore += tScore.score * 3; if (tScore.range) titleMatch = tScore.range }
      const subtitle = String(getVal(item.row, subtitleKeyMap[cat]) || '')
      const sScore = matchScore(subtitle, kw)
      if (sScore.score > 0) bestScore += sScore.score * 1.5
      for (const key of Object.keys(item.row)) {
        const val = item.row[key]
        if (typeof val !== 'string' && typeof val !== 'number') continue
        const m = matchScore(String(val), kw)
        if (m.score > 0) bestScore += m.score * 0.5
      }
      if (bestScore > 0) {
        hits.push({ category: cat, sheetName: item.sheetName, row: item.row, title: title || '-', subtitle, score: bestScore, titleMatch })
      }
    }
  }
  return hits.sort((a, b) => b.score - a.score).slice(0, 50)
})

const groupedResults = computed(() => {
  const groups: Record<DataCategory, SearchHit[]> = { chips: [], servers: [], super_nodes: [], other: [] }
  for (const r of results.value) groups[r.category].push(r)
  return groups
})
const flatResults = computed(() => results.value)
watch(results, () => { selectedIndex.value = 0 })

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = Math.min(flatResults.value.length - 1, selectedIndex.value + 1)
    scrollSelectedIntoView()
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = Math.max(0, selectedIndex.value - 1)
    scrollSelectedIntoView()
  } else if (e.key === 'Enter') {
    e.preventDefault(); selectCurrent()
  } else if (e.key === 'Escape') {
    e.preventDefault()
    if (detailVisible.value) detailVisible.value = false
    else close()
  }
}

function scrollSelectedIntoView() {
  nextTick(() => {
    const el = document.querySelector('.qs-result-item.selected')
    if (el) el.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
  })
}

function selectCurrent() {
  const item = flatResults.value[selectedIndex.value]
  if (!item) return
  selectedDetail.value = item
  detailVisible.value = true
}

function highlightTitle(title: string, range?: [number, number]): string {
  if (!range) return escape(title)
  const [s, e] = range
  return escape(title.slice(0, s)) + '<mark>' + escape(title.slice(s, e)) + '</mark>' + escape(title.slice(e))
}
function escape(s: string): string {
  return s.replace(/[<>&"']/g, (c) => ({ '<': '&lt;', '>': '&gt;', '&': '&amp;', '"': '&quot;', "'": '&#39;' }[c]!))
}
function copyValue(v: any) {
  if (v === undefined || v === null) return
  navigator.clipboard.writeText(String(v)).then(() => {
    ElMessage.success('已复制: ' + v)
  }).catch(() => { ElMessage.error('复制失败') })
}
function openFullTable(hit: SearchHit) {
  detailVisible.value = false
  close()
  window.location.hash = `#/report/${hit.category}`
}

function onModelValueUpdate(v: boolean) {
  emit('update:modelValue', v)
}
</script>

<template>
  <el-dialog
    :model-value="modelValue"
    @update:model-value="onModelValueUpdate"
    :show-close="false" :align-center="true" width="640px" top="10vh"
    custom-class="qs-dialog">
    <div class="qs-container" @keydown="onKeydown">
      <div class="qs-search-bar">
        <span class="qs-search-icon">🔍</span>
        <input ref="inputRef" v-model="keyword" class="qs-input"
          placeholder="搜索芯片 / 服务器 / 超节点(型号、名称、厂商...)" @keydown="onKeydown" />
        <div class="qs-hints">
          <span class="qs-chip"><kbd>↑</kbd><kbd>↓</kbd><span>选择</span></span>
          <span class="qs-chip primary"><kbd>Enter</kbd><span>详情</span></span>
          <span class="qs-chip danger"><kbd>Esc</kbd><span>关闭</span></span>
        </div>
      </div>
      <div class="qs-results">
        <div v-if="results.length === 0" class="qs-empty">
          <div v-if="!loaded">加载中…</div>
          <div v-else>没有匹配的结果</div>
        </div>
        <template v-else>
          <div v-for="cat in (['servers', 'chips', 'super_nodes'] as DataCategory[])" :key="cat">
            <div v-if="groupedResults[cat].length > 0" class="qs-group">
              <div class="qs-group-header">
                <span class="qs-group-icon">
                  <img v-if="categoryMeta[cat].iconImg" :src="categoryMeta[cat].iconImg" :alt="categoryMeta[cat].label" class="qs-cat-icon" />
                  <span v-else>{{ categoryMeta[cat].icon }}</span>
                </span>
                <span>{{ categoryMeta[cat].label }}</span>
                <span class="qs-group-count">{{ groupedResults[cat].length }}</span>
              </div>
              <div v-for="(hit, idx) in groupedResults[cat]" :key="cat + '-' + idx"
                class="qs-result-item"
                :class="{ selected: flatResults[selectedIndex] === hit }"
                @click="selectedIndex = flatResults.indexOf(hit); selectCurrent()"
                @mouseenter="selectedIndex = flatResults.indexOf(hit)">
                <div class="qs-result-icon">
                  <img v-if="categoryMeta[cat].iconImg" :src="categoryMeta[cat].iconImg" :alt="categoryMeta[cat].label" class="qs-cat-icon" />
                  <span v-else>{{ categoryMeta[cat].icon }}</span>
                </div>
                <div class="qs-result-body">
                  <div class="qs-result-title" v-html="highlightTitle(hit.title, hit.titleMatch)"></div>
                  <div class="qs-result-subtitle" v-if="hit.subtitle">
                    <span v-html="highlightTitle(hit.subtitle)"></span>
                  </div>
                </div>
                <div class="qs-result-arrow" v-if="flatResults[selectedIndex] === hit">↵</div>
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>
  </el-dialog>

  <el-dialog v-model="detailVisible" :show-close="true" :align-center="true" width="780px" top="8vh"
    custom-class="qs-detail-dialog" :modal="true" :append-to-body="true">
    <template #header>
      <div class="qs-detail-header">
        <img v-if="categoryMeta[selectedDetail?.category || 'other'].iconImg" :src="categoryMeta[selectedDetail?.category || 'other'].iconImg || ''" :alt="categoryMeta[selectedDetail?.category || 'other'].label" class="qs-cat-icon" />
        <span v-else>{{ categoryMeta[selectedDetail?.category || 'other'].icon }}</span>
        <span>{{ categoryMeta[selectedDetail?.category || 'other'].label }} · 详情</span>
      </div>
    </template>
    <DetailCard v-if="selectedDetail" :category="selectedDetail.category" :row="selectedDetail.row" />
    <template #footer>
      <div class="qs-detail-footer">
        <el-button @click="copyValue(JSON.stringify(selectedDetail?.row, null, 2))">📋 复制全部字段</el-button>
        <el-button type="primary" @click="openFullTable(selectedDetail!)" v-if="selectedDetail">📊 打开完整表格</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<style>
.qs-dialog { border-radius: 12px !important; overflow: hidden; box-shadow: 0 20px 50px rgba(0, 0, 0, 0.25) !important; }
.qs-dialog .el-dialog__header { display: none; }
.qs-dialog .el-dialog__body { padding: 0 !important; }
.qs-dialog .el-dialog__footer { padding: 0 !important; }
.qs-detail-dialog { border-radius: 16px !important; overflow: hidden; }
.qs-detail-dialog .el-dialog__header { padding: 18px 24px !important; margin: 0 !important; border-bottom: 1px solid #ebeef5; background: #fafbfc; }
.qs-detail-dialog .el-dialog__body { padding: 24px !important; }
.qs-detail-dialog .el-dialog__footer { padding: 16px 24px !important; border-top: 1px solid #ebeef5; background: #fafbfc; }
.qs-detail-dialog .el-dialog__close { width: 40px !important; height: 40px !important; font-size: 24px !important; border-radius: 50% !important; background: #fef0f0 !important; color: #f56c6c !important; border: 2px solid #fbc4c4 !important; transition: all 0.15s !important; top: 12px !important; right: 12px !important; }
.qs-detail-dialog .el-dialog__close:hover { background: #f56c6c !important; color: #fff !important; transform: rotate(90deg) scale(1.1); border-color: #f56c6c !important; }
.qs-dialog .el-dialog__close { width: 32px !important; height: 32px !important; font-size: 20px !important; }
</style>

<style scoped>
.qs-container { display: flex; flex-direction: column; max-height: 70vh; }
.qs-search-bar { display: flex; align-items: center; padding: 16px 20px; border-bottom: 1px solid #ebeef5; background: #fafbfc; gap: 10px; }
.qs-search-icon { font-size: 18px; flex-shrink: 0; }
.qs-input { flex: 1; border: none; outline: none; font-size: 16px; background: transparent; color: #303133; }
.qs-input::placeholder { color: #c0c4cc; }
.qs-hints { display: flex; gap: 6px; flex-shrink: 0; align-items: center; }
.qs-chip { display: inline-flex; align-items: center; gap: 4px; padding: 4px 8px; background: #fff; border: 1.5px solid #dcdfe6; border-radius: 6px; font-size: 11px; color: #606266; font-weight: 500; line-height: 1; }
.qs-chip kbd { font-family: 'SF Mono', Consolas, monospace; font-size: 11px; font-weight: 600; padding: 2px 4px; background: #f5f7fa; border: 1px solid #dcdfe6; border-radius: 3px; color: #303133; line-height: 1; min-width: 14px; text-align: center; }
.qs-chip span { font-size: 11px; color: #606266; }
.qs-chip.primary { background: #ecf5ff; border-color: #b3d8ff; color: #409eff; }
.qs-chip.primary kbd { background: #409eff; color: #fff; border-color: #409eff; }
.qs-chip.primary span { color: #409eff; font-weight: 600; }
.qs-chip.danger { background: #fef0f0; border-color: #fbc4c4; color: #f56c6c; }
.qs-chip.danger kbd { background: #f56c6c; color: #fff; border-color: #f56c6c; }
.qs-chip.danger span { color: #f56c6c; font-weight: 600; }
.qs-results { overflow-y: auto; max-height: 60vh; padding: 8px 0; }
.qs-empty { padding: 60px 20px; text-align: center; color: #909399; font-size: 14px; }
.qs-group { margin-bottom: 4px; }
.qs-group-header { display: flex; align-items: center; gap: 6px; padding: 8px 20px 4px; font-size: 11px; color: #909399; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; }
.qs-group-icon { font-size: 13px; }
.qs-group-count { margin-left: auto; background: #f0f0f0; padding: 0 6px; border-radius: 8px; font-size: 10px; color: #606266; }
.qs-result-item { display: flex; align-items: center; padding: 10px 20px; cursor: pointer; gap: 12px; border-left: 3px solid transparent; transition: background 0.1s; }
.qs-result-item.selected { background: linear-gradient(90deg, #e1f0ff 0%, #f0f7ff 100%); border-left-color: #409eff; }
.qs-result-icon { font-size: 20px; width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; background: #f5f7fa; border-radius: 6px; flex-shrink: 0; }
.qs-cat-icon { height: 22px; width: 22px; object-fit: contain; display: block; }
.qs-result-item.selected .qs-result-icon { background: #fff; }
.qs-result-body { flex: 1; min-width: 0; }
.qs-result-title { font-size: 14px; font-weight: 500; color: #303133; margin-bottom: 2px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.qs-result-title :deep(mark) { background: #fff3a3; color: #d4380d; padding: 0 1px; border-radius: 2px; }
.qs-result-subtitle { font-size: 12px; color: #909399; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.qs-result-arrow { color: #409eff; font-size: 16px; }
.qs-detail-header { display: flex; align-items: center; gap: 8px; font-size: 16px; font-weight: 600; }
.qs-detail-footer { display: flex; gap: 8px; justify-content: flex-end; }
</style>
