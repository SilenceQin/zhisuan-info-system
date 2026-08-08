<script setup lang="ts">
import { onMounted, ref, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { api, type DataCategory, type ColumnInfo } from '../api'

const route = useRoute()
const router = useRouter()

const param = computed(() => decodeURIComponent(route.params.id as string))
const isCategory = computed(() => ['chips', 'servers', 'super_nodes'].includes(param.value))

const categoryInfo: Record<string, { label: string; iconImg: string }> = {
  chips: { label: '芯片库', iconImg: './cat-chips.png' },
  servers: { label: '服务器库', iconImg: './cat-servers.png' },
  super_nodes: { label: '超节点库', iconImg: './cat-supernodes.png' }
}
const pageTitle = computed(() => {
  const info = categoryInfo[param.value]
  return info ? info.label : param.value
})
const pageIcon = computed(() => {
  const info = categoryInfo[param.value]
  return info ? info.iconImg : null
})

const allRows = ref<any[]>([])
const columns = ref<ColumnInfo[]>([])
const loading = ref(false)
const filterCollapse = ref<string[]>([])

const filters = ref<Record<string, any>>({})
const keyword = ref('')
const sortField = ref<string>('')
const sortOrder = ref<'asc' | 'desc'>('asc')
const page = ref(1)
const pageSize = ref(50)

const stringColumnOptions = computed(() => {
  const result: Record<string, string[]> = {}
  for (const col of columns.value) {
    if (col.type === 'string') {
      const values = new Set<string>()
      for (const row of allRows.value) {
        const v = row[col.name]
        if (v !== null && v !== undefined && v !== '') values.add(String(v))
      }
      result[col.name] = Array.from(values).sort()
    }
  }
  return result
})

const numberColumnRange = computed(() => {
  const result: Record<string, { min: number; max: number }> = {}
  for (const col of columns.value) {
    if (col.type === 'number') {
      let min = Infinity, max = -Infinity
      for (const row of allRows.value) {
        const v = Number(row[col.name])
        if (!isNaN(v)) { if (v < min) min = v; if (v > max) max = v }
      }
      if (isFinite(min) && isFinite(max)) result[col.name] = { min, max }
    }
  }
  return result
})

const numberRangeFilters = ref<Record<string, [number | null, number | null]>>({})

const filteredRows = computed(() => {
  let rows = allRows.value
  if (keyword.value.trim()) {
    const kw = keyword.value.trim().toLowerCase()
    rows = rows.filter((r) => {
      for (const col of columns.value) {
        if (col.type === 'string') {
          const v = r[col.name]
          if (v !== null && v !== undefined && String(v).toLowerCase().includes(kw)) return true
        }
      }
      return false
    })
  }
  for (const [col, val] of Object.entries(filters.value)) {
    if (val !== undefined && val !== null && val !== '') {
      rows = rows.filter((r) => String(r[col]) === String(val))
    }
  }
  for (const [col, range] of Object.entries(numberRangeFilters.value)) {
    const [min, max] = range
    if (min !== null && min !== undefined) {
      rows = rows.filter((r) => { const v = Number(r[col]); return !isNaN(v) && v >= Number(min) })
    }
    if (max !== null && max !== undefined) {
      rows = rows.filter((r) => { const v = Number(r[col]); return !isNaN(v) && v <= Number(max) })
    }
  }
  if (sortField.value) {
    rows = [...rows].sort((a, b) => {
      const va = a[sortField.value], vb = b[sortField.value]
      if (va === null || va === undefined) return 1
      if (vb === null || vb === undefined) return -1
      const na = Number(va), nb = Number(vb)
      const isNum = !isNaN(na) && !isNaN(nb)
      if (isNum) return sortOrder.value === 'asc' ? na - nb : nb - na
      const sa = String(va), sb = String(vb)
      return sortOrder.value === 'asc' ? sa.localeCompare(sb) : sb.localeCompare(sa)
    })
  }
  return rows
})

const pagedRows = computed(() => {
  const start = (page.value - 1) * pageSize.value
  return filteredRows.value.slice(start, start + pageSize.value)
})
const total = computed(() => filteredRows.value.length)

async function load() {
  loading.value = true
  try {
    if (isCategory.value) {
      const tables = await api.listByCategory(param.value as DataCategory)
      if (tables.length === 0) {
        ElMessage.warning('该分类下还没有数据')
        allRows.value = []
        columns.value = []
        return
      }
      const merged: any[] = []
      let cols: ColumnInfo[] = []
      for (const t of tables) {
        const rr = await api.queryAllRows(t.table_name)
        if (rr.rows) {
          if (cols.length === 0) cols = rr.columns
          merged.push(...rr.rows)
        }
      }
      allRows.value = merged
      columns.value = cols
    } else {
      const r = await api.queryAllRows(param.value)
      allRows.value = r.rows
      columns.value = r.columns
    }
  } catch (err: any) {
    ElMessage.error('加载失败: ' + (err.message || err))
  } finally {
    loading.value = false
  }
}

function onSortChange({ prop, order }: any) {
  if (prop && order) {
    sortField.value = prop
    sortOrder.value = order === 'descending' ? 'desc' : 'asc'
  } else {
    sortField.value = ''
  }
}

function resetFilters() {
  filters.value = {}
  numberRangeFilters.value = {}
  keyword.value = ''
  page.value = 1
}

function goBack() { router.push('/') }

function hasActiveFilters() {
  if (keyword.value.trim()) return true
  for (const v of Object.values(filters.value)) {
    if (v !== undefined && v !== null && v !== '') return true
  }
  for (const r of Object.values(numberRangeFilters.value)) {
    if ((r[0] !== null && r[0] !== undefined) ||
        (r[1] !== null && r[1] !== undefined)) return true
  }
  return false
}

function formatNumber(value: any, decimals = 2): string {
  if (value === null || value === undefined || value === '') return '-'
  const n = Number(value)
  if (isNaN(n)) return String(value)
  if (Number.isInteger(n)) return n.toLocaleString('en-US')
  return n.toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: decimals })
}

function formatCell(value: any, colName: string, type: string): string {
  if (value === null || value === undefined || value === '') return '-'
  if (type === 'string') return String(value)
  if (type === 'boolean') return value ? '✓' : '✗'
  if (type === 'date') {
    try { return new Date(value).toLocaleDateString('zh-CN') } catch { return String(value) }
  }
  return formatNumber(value)
}

watch(filters, () => { page.value = 1 }, { deep: true })
watch(numberRangeFilters, () => { page.value = 1 }, { deep: true })
watch(keyword, () => { page.value = 1 })

onMounted(load)
</script>

<template>
  <div class="page">
    <div class="toolbar">
      <button class="back-btn" @click="goBack" title="返回主页">
        <span class="back-icon">←</span>
        <span class="back-text">返回主页</span>
      </button>
      <h2 style="margin: 0; font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px;">
        <img v-if="pageIcon" :src="pageIcon" :alt="pageTitle" class="report-title-icon" />
        <span>{{ pageTitle }}</span>
      </h2>
      <el-tag type="info" size="small">共 {{ allRows.length }} 条</el-tag>
      <el-tag v-if="hasActiveFilters()" type="warning" size="small">筛选后 {{ total }} 条</el-tag>
      <div class="spacer"></div>
      <el-button @click="load">↻ 刷新数据</el-button>
    </div>

    <el-card v-if="columns.length > 0" shadow="never" style="margin-bottom: 12px">
      <div class="filter-row">
        <el-input v-model="keyword" placeholder="全局搜索(字符串列)" style="width: 240px" clearable>
          <template #prefix>🔍</template>
        </el-input>
        <el-button v-if="hasActiveFilters()" text type="primary" @click="resetFilters">清除筛选</el-button>
      </div>
      <el-collapse v-model="filterCollapse">
        <el-collapse-item title="列筛选" name="1">
          <div class="filter-grid">
            <div v-for="col in columns" :key="col.name" class="filter-item">
              <div class="filter-label">{{ col.name }}</div>
              <el-select
                v-if="col.type === 'string' && stringColumnOptions[col.name]"
                v-model="filters[col.name]"
                placeholder="全部" clearable size="small" style="width: 100%"
              >
                <el-option v-for="opt in stringColumnOptions[col.name]" :key="opt" :label="opt" :value="opt" />
              </el-select>
              <div v-else-if="col.type === 'number' && numberColumnRange[col.name]" class="range-input">
                <el-input-number v-model="numberRangeFilters[col.name]![0]"
                  :placeholder="`≥ ${formatNumber(numberColumnRange[col.name].min, 1)}`"
                  size="small" :controls="false" style="width: 100%" />
                <span class="range-sep">~</span>
                <el-input-number v-model="numberRangeFilters[col.name]![1]"
                  :placeholder="`≤ ${formatNumber(numberColumnRange[col.name].max, 1)}`"
                  size="small" :controls="false" style="width: 100%" />
              </div>
              <div v-else class="filter-na">-</div>
            </div>
          </div>
        </el-collapse-item>
      </el-collapse>
    </el-card>

    <el-card shadow="never">
      <el-table :data="pagedRows" v-loading="loading" stripe border
        height="calc(100vh - 320px)" @sort-change="onSortChange">
        <el-table-column
          v-for="col in columns" :key="col.name" :prop="col.name" :label="col.name"
          :sortable="col.type === 'string' || col.type === 'number' || col.type === 'date' ? 'custom' : false"
          min-width="120" show-overflow-tooltip
        >
          <template #default="{ row }">
            <span :class="['cell', col.type]">{{ formatCell(row[col.name], col.name, col.type) }}</span>
          </template>
        </el-table-column>
      </el-table>
      <div style="display: flex; align-items: center; margin-top: 16px; gap: 12px">
        <span style="color: #909399; font-size: 13px">共 {{ total }} 条</span>
        <div style="flex: 1"></div>
        <el-pagination background
          layout="prev, pager, next, sizes, jumper"
          :total="total" :page-size="pageSize" :current-page="page"
          :page-sizes="[20, 50, 100, 200]"
          @current-change="(p: number) => { page = p }"
          @size-change="(s: number) => { pageSize = s; page = 1 }" />
      </div>
    </el-card>
  </div>
</template>

<style scoped>
.filter-row { display: flex; align-items: center; gap: 12px; margin-bottom: 8px; }
.report-title-icon { height: 28px; width: 28px; object-fit: contain; display: block; }
.back-btn {
  display: inline-flex; align-items: center; gap: 8px; height: 40px; padding: 0 18px 0 14px;
  background: linear-gradient(135deg, #409eff 0%, #66b1ff 100%); border: none; border-radius: 8px;
  color: #fff; cursor: pointer; font-size: 14px; font-weight: 600;
  box-shadow: 0 2px 8px rgba(64, 158, 255, 0.3); transition: all 0.15s; user-select: none; letter-spacing: 0.5px;
}
.back-btn:hover { background: linear-gradient(135deg, #337ecc 0%, #409eff 100%); box-shadow: 0 4px 12px rgba(64, 158, 255, 0.45); transform: translateX(-2px); }
.back-btn:active { transform: translateX(-1px) scale(0.98); }
.back-icon { font-size: 22px; font-weight: 700; line-height: 1; }
.back-text { font-size: 14px; }
.filter-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 12px; padding: 4px 0; }
.filter-item { display: flex; flex-direction: column; gap: 4px; }
.filter-label { font-size: 12px; color: #606266; font-weight: 500; }
.range-input { display: flex; align-items: center; gap: 4px; }
.range-sep { color: #c0c4cc; font-size: 12px; }
.filter-na { color: #c0c4cc; font-size: 12px; text-align: center; padding: 4px; }
.cell { font-variant-numeric: tabular-nums; }
.cell.number { text-align: right; font-family: 'SF Mono', Consolas, monospace; }
</style>
