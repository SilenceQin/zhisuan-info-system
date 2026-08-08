<script setup lang="ts">
import { computed } from 'vue'
import { ElTag } from 'element-plus'
import type { DataCategory } from '../api'

const props = defineProps<{ category: DataCategory; row: Record<string, any> }>()

const isChip = computed(() => props.category === 'chips')
const isServer = computed(() => props.category === 'servers')
const isSuper = computed(() => props.category === 'super_nodes')

function v(...keys: string[]): any {
  for (const k of keys) {
    if (props.row[k] !== undefined && props.row[k] !== null && props.row[k] !== '') return props.row[k]
  }
  return undefined
}

function num(n: any, digits = 2): string {
  if (n === null || n === undefined || n === '') return '-'
  const x = Number(n)
  if (isNaN(x)) return String(n)
  if (Number.isInteger(x)) return x.toLocaleString('en-US')
  return x.toLocaleString('en-US', { maximumFractionDigits: digits })
}

function date(s: any): string {
  if (!s) return '-'
  const d = new Date(s)
  if (isNaN(d.getTime())) return String(s)
  return d.toLocaleDateString('zh-CN')
}

const title = computed(() => v('芯片型号', '服务器名称', '超节点名称', '名称'))
const vendor = computed(() => v('芯片厂商', '厂商'))

const computeBarStyle = computed(() => {
  const val = Number(v('服务器算力（P@FP16）', '服务器算力(P@FP16)') ?? 0)
  const pct = Math.max(0, Math.min(100, (val / 12) * 100))
  return { width: pct + '%' }
})
const powerBarStyle = computed(() => {
  const isSuper = props.category === 'super_nodes'
  const val = Number(
    v('服务器功率（kW）', '服务器功率(kW)', '单柜功率（kW）', '单柜功率(kW)') ?? 0
  )
  const cap = isSuper ? 200 : 20
  const pct = Math.max(0, Math.min(100, (val / cap) * 100))
  return { width: pct + '%' }
})
</script>

<template>
  <div class="detail-card" :class="category">
    <div class="card-header">
      <div class="card-icon">
        <span v-if="isChip">🔬</span>
        <span v-else-if="isServer">🖥️</span>
        <span v-else>🧠</span>
      </div>
      <div class="card-title-block">
        <h2 class="card-title">{{ title || '—' }}</h2>
        <div class="card-meta">
          <el-tag v-if="vendor" size="small" type="info">{{ vendor }}</el-tag>
          <el-tag v-if="isChip" size="small" type="success">{{ v('产品形态') || 'GPU' }}</el-tag>
          <el-tag v-if="isChip && v('芯片架构')" size="small">{{ v('芯片架构') }}</el-tag>
          <el-tag v-if="isServer && v('芯片')" size="small" type="success">芯片: {{ v('芯片') }}</el-tag>
          <el-tag v-if="isSuper && v('芯片')" size="small" type="success">芯片: {{ v('芯片') }}</el-tag>
        </div>
      </div>
    </div>

    <div v-if="isChip" class="kpi-row precision">
      <div class="kpi precision-fp4"><div class="kpi-label">FP4</div><div class="kpi-value"><span>{{ num(v('算力(P@FP4)')) }}</span><small>P</small></div></div>
      <div class="kpi precision-fp8"><div class="kpi-label">FP8</div><div class="kpi-value"><span>{{ num(v('算力(P@FP8)')) }}</span><small>P</small></div></div>
      <div class="kpi precision-fp16 primary"><div class="kpi-label">FP16</div><div class="kpi-value"><span>{{ num(v('算力(P@FP16)')) }}</span><small>P</small></div></div>
      <div class="kpi precision-fp32"><div class="kpi-label">FP32</div><div class="kpi-value"><span>{{ num(v('算力(P@FP32)')) }}</span><small>P</small></div></div>
      <div class="kpi precision-fp64"><div class="kpi-label">FP64</div><div class="kpi-value"><span>{{ num(v('算力(P@FP64)')) }}</span><small>P</small></div></div>
    </div>

    <div v-if="isServer" class="kpi-row hero">
      <div class="kpi hero-compute">
        <div class="kpi-label">整机算力 <span class="kpi-sub">P@FP16</span></div>
        <div class="kpi-value"><span>{{ num(v('服务器算力（P@FP16）', '服务器算力(P@FP16)')) }}</span><small>P</small></div>
        <div class="kpi-bar"><div class="kpi-bar-fill" :style="computeBarStyle"></div></div>
      </div>
      <div class="kpi hero-power">
        <div class="kpi-label">整机功耗</div>
        <div class="kpi-value"><span>{{ num(v('服务器功率（kW）', '服务器功率(kW)')) }}</span><small>kW</small></div>
        <div class="kpi-bar"><div class="kpi-bar-fill power" :style="powerBarStyle"></div></div>
      </div>
    </div>

    <div v-if="isSuper" class="kpi-row hero triple">
      <div class="kpi hero-compute">
        <div class="kpi-label">单节点算力 <span class="kpi-sub">P@FP16</span></div>
        <div class="kpi-value"><span>{{ num(v('节点算力')) }}</span><small>P</small></div>
      </div>
      <div class="kpi hero-count">
        <div class="kpi-label">算力卡数</div>
        <div class="kpi-value"><span>{{ num(v('节点规模', '单compute tray/服务器GPU卡数'), 0) }}</span><small>张</small></div>
        <div class="kpi-sub">节点规模</div>
      </div>
      <div class="kpi hero-power">
        <div class="kpi-label">整体功耗</div>
        <div class="kpi-value"><span>{{ num(v('单柜功率（kW）', '单柜功率(kW)')) }}</span><small>kW</small></div>
        <div class="kpi-bar"><div class="kpi-bar-fill power" :style="powerBarStyle"></div></div>
      </div>
    </div>

    <div class="sub-grid">
      <template v-if="isChip">
        <div class="sub-item"><label>上市时间</label><span>{{ date(v('上市时间')) }}</span></div>
        <div class="sub-item"><label>显存</label><span>{{ num(v('显存（G）', '显存(G)'), 0) }} G · {{ v('显存类型') || '-' }}</span></div>
        <div class="sub-item"><label>芯片功耗 TDP</label><span>{{ num(v('芯片功耗TDP（W）', '芯片功耗TDP(W)'), 0) }} W</span></div>
        <div class="sub-item"><label>显存带宽</label><span>{{ num(v('显存带宽（TB/s）', '显存带宽(TB/s)'), 2) }} TB/s</span></div>
        <div class="sub-item"><label>互联</label><span>{{ v('GPU互联') || '-' }} · {{ num(v('GPU互联速率（GB/s）', 'GPU互联速率(GB/s)'), 0) }} GB/s</span></div>
      </template>
      <template v-if="isServer">
        <div class="sub-item"><label>GPU 卡数</label><span>{{ num(v('GPU卡数'), 0) }} 张</span></div>
        <div class="sub-item"><label>服务器高度</label><span>{{ v('服务器高度（U）', '服务器高度(U)') || '-' }} U</span></div>
        <div class="sub-item"><label>制冷方式</label><span>{{ v('服务器制冷方式', '制冷方式') || '-' }}</span></div>
        <div class="sub-item v-colspan" v-if="v('备注')"><label>备注</label><span>{{ v('备注') }}</span></div>
      </template>
      <template v-if="isSuper">
        <div class="sub-item"><label>节点规模</label><span>{{ num(v('节点规模'), 0) }}</span></div>
        <div class="sub-item"><label>服务器数</label><span>{{ num(v('compute tray/节点服务器数量'), 0) }} 台</span></div>
        <div class="sub-item"><label>单 server 卡数</label><span>{{ num(v('单compute tray/服务器GPU卡数'), 0) }} 张</span></div>
        <div class="sub-item"><label>单 server 算力</label><span>{{ num(v('compute tray/服务器算力（P@FP16）', 'compute tray/服务器算力(P@FP16)')) }} P</span></div>
        <div class="sub-item"><label>服务器高度</label><span>{{ v('服务器高度（U）', '服务器高度(U)') || '-' }} U</span></div>
        <div class="sub-item"><label>制冷方式</label><span>{{ v('服务器制冷方式', '制冷方式') || '-' }}</span></div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.detail-card { padding: 4px 0; }
.card-header { display: flex; align-items: center; gap: 16px; padding-bottom: 20px; border-bottom: 1px solid #ebeef5; margin-bottom: 24px; }
.card-icon { font-size: 36px; width: 56px; height: 56px; display: flex; align-items: center; justify-content: center; background: linear-gradient(135deg, #f0f7ff 0%, #e1f0ff 100%); border-radius: 12px; }
.card-title { margin: 0; font-size: 22px; font-weight: 600; color: #303133; }
.card-meta { display: flex; gap: 6px; margin-top: 6px; flex-wrap: wrap; }
.kpi-row { display: grid; gap: 12px; margin-bottom: 20px; }
.kpi-row.precision { grid-template-columns: repeat(5, 1fr); }
.kpi-row.precision .kpi { background: #fafbfc; border: 1px solid #ebeef5; border-radius: 8px; padding: 16px 8px; text-align: center; transition: all 0.15s; }
.kpi-row.precision .kpi:hover { background: #f0f7ff; border-color: #b3d8ff; transform: translateY(-1px); }
.kpi-row.precision .kpi.primary { background: linear-gradient(135deg, #e1f0ff 0%, #c6e2ff 100%); border-color: #409eff; }
.kpi-row.precision .kpi-label { font-size: 12px; color: #909399; font-weight: 500; margin-bottom: 6px; }
.kpi-row.precision .kpi.primary .kpi-label { color: #409eff; font-weight: 600; }
.kpi-row.precision .kpi-value { font-size: 22px; font-weight: 700; color: #303133; font-variant-numeric: tabular-nums; line-height: 1; }
.kpi-row.precision .kpi.primary .kpi-value { color: #409eff; }
.kpi-row.precision .kpi-value small { font-size: 11px; font-weight: 500; color: #909399; margin-left: 2px; }
.kpi-row.hero { grid-template-columns: 1fr 1fr; gap: 16px; }
.kpi-row.hero.triple { grid-template-columns: 1fr 1fr 1fr; }
.kpi.hero-compute, .kpi.hero-power, .kpi.hero-count { background: linear-gradient(135deg, #fff8e6 0%, #ffefc1 100%); border: 1px solid #ffd666; border-radius: 10px; padding: 20px 24px; }
.kpi.hero-power { background: linear-gradient(135deg, #fff1f0 0%, #ffccc7 100%); border-color: #ff9c6e; }
.kpi.hero-count { background: linear-gradient(135deg, #f6ffed 0%, #d9f7be 100%); border-color: #95de64; }
.kpi-label { font-size: 13px; color: #606266; font-weight: 500; margin-bottom: 8px; }
.kpi-sub { font-size: 11px; color: #909399; font-weight: 400; margin-left: 4px; }
.kpi-value { font-size: 36px; font-weight: 700; color: #d48806; font-variant-numeric: tabular-nums; line-height: 1.1; display: flex; align-items: baseline; gap: 4px; }
.kpi.hero-power .kpi-value { color: #d4380d; }
.kpi.hero-count .kpi-value { color: #389e0d; }
.kpi-value small { font-size: 16px; font-weight: 500; color: #8c8c8c; }
.kpi-bar { margin-top: 12px; height: 4px; background: rgba(0, 0, 0, 0.06); border-radius: 2px; overflow: hidden; }
.kpi-bar-fill { height: 100%; background: #faad14; border-radius: 2px; transition: width 0.3s; }
.kpi-bar-fill.power { background: #fa541c; }
.sub-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 10px 20px; padding: 16px 0 4px; border-top: 1px solid #ebeef5; }
.sub-item { display: flex; flex-direction: column; gap: 2px; font-size: 13px; }
.sub-item.v-colspan { grid-column: 1 / -1; }
.sub-item label { color: #909399; font-size: 12px; }
.sub-item span { color: #303133; font-weight: 500; }
</style>
