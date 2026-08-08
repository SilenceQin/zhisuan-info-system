<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { api, type TableInfo } from '../api'

const router = useRouter()
const allTables = ref<TableInfo[]>([])
const loading = ref(false)

const isMac = ref(false)
onMounted(() => {
  if (typeof navigator !== 'undefined' && navigator.platform) {
    isMac.value = navigator.platform.toLowerCase().includes('mac')
  }
})
const cmdKey = computed(() => isMac.value ? '⌘' : 'Ctrl')

const categories = computed(() => {
  const groups: Record<string, { name: string; label: string; icon: string; iconImg: string; desc: string; tables: TableInfo[]; totalRows: number; latestAt: number; gradient: string }> = {
    chips: {
      name: 'chips', label: '芯片库', icon: '🔬', iconImg: './cat-chips.png',
      desc: '各厂商智算芯片 · 各精度算力', tables: [], totalRows: 0, latestAt: 0,
      gradient: 'linear-gradient(135deg, #409eff 0%, #66b1ff 100%)'
    },
    servers: {
      name: 'servers', label: '服务器库', icon: '🖥️', iconImg: './cat-servers.png',
      desc: '整机配置 · 算力 · 功耗', tables: [], totalRows: 0, latestAt: 0,
      gradient: 'linear-gradient(135deg, #67c23a 0%, #85ce61 100%)'
    },
    super_nodes: {
      name: 'super_nodes', label: '超节点库', icon: '🧠', iconImg: './cat-supernodes.png',
      desc: '单节点算力 · 节点规模 · 功耗', tables: [], totalRows: 0, latestAt: 0,
      gradient: 'linear-gradient(135deg, #e6a23c 0%, #ebb563 100%)'
    }
  }
  for (const t of allTables.value) {
    const cat = t.category || 'chips'
    if (groups[cat]) {
      groups[cat].tables.push(t)
      groups[cat].totalRows += t.row_count
      if (t.imported_at > groups[cat].latestAt) groups[cat].latestAt = t.imported_at
    }
  }
  return Object.values(groups)
})

async function loadTables() {
  loading.value = true
  try {
    allTables.value = await api.listTables()
  } finally {
    loading.value = false
  }
}

function openCategory(category: string) {
  router.push(`/report/${category}`)
}

function onCardClick(cat: any) {
  if (cat.totalRows === 0) return
  openCategory(cat.name)
}

function focusSearch() {
  window.dispatchEvent(new CustomEvent('open-quick-search'))
}

onMounted(loadTables)
</script>

<template>
  <div class="home">
    <section class="hero">
      <div class="hero-bg">
        <div class="hero-blob hero-blob-1"></div>
        <div class="hero-blob hero-blob-2"></div>
      </div>

      <div class="hero-content">
        <h1 class="hero-title">智算芯片 · 服务器 · 超节点 数据库</h1>
        <p class="hero-subtitle">从这里开始 — 搜索任意芯片型号、服务器、超节点,或选择下方分类</p>

        <div class="hero-search" @click="focusSearch">
          <span class="hero-search-icon">🔍</span>
          <span class="hero-search-text">搜索芯片型号、服务器、超节点...</span>
          <span class="hero-search-kbd">
            <kbd>{{ cmdKey }}</kbd>
            <kbd>K</kbd>
          </span>
        </div>

        <div class="hero-hints">
          <span>试试:</span>
          <a class="hero-hint" @click="focusSearch">B200</a>
          <a class="hero-hint" @click="focusSearch">华为</a>
          <a class="hero-hint" @click="focusSearch">910C</a>
          <a class="hero-hint" @click="focusSearch">GB300</a>
        </div>
      </div>
    </section>

    <section class="categories">
      <h2 class="section-title">
        <span>分类浏览</span>
        <span class="section-sub">按数据类型查看</span>
      </h2>

      <div class="category-grid">
        <div
          v-for="cat in categories"
          :key="cat.name"
          class="category-card"
          :class="{ disabled: cat.totalRows === 0 }"
          @click="onCardClick(cat)"
        >
          <div class="category-glow" :style="{ background: cat.gradient }"></div>
          <div class="category-inner">
            <div class="category-icon">
              <img :src="cat.iconImg" :alt="cat.label" class="category-icon-img" @error="(e) => { (e.target as HTMLImageElement).style.display = 'none'; ((e.target as HTMLImageElement).nextElementSibling as HTMLElement).style.display = 'inline' }" />
              <span class="category-icon-emoji" style="display: none">{{ cat.icon }}</span>
            </div>
            <div class="category-name">{{ cat.label }}</div>
            <div class="category-desc">{{ cat.desc }}</div>
            <div class="category-meta">
              <span v-if="cat.totalRows > 0" class="category-count">
                <strong>{{ cat.totalRows }}</strong> 项
              </span>
              <span v-else class="category-empty">暂无数据</span>
              <span class="category-arrow" v-if="cat.totalRows > 0">→</span>
            </div>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.home { min-height: 100%; padding-bottom: 80px; }
.hero { position: relative; padding: 80px 24px 60px; overflow: hidden; }
.hero-bg { position: absolute; inset: 0; background: linear-gradient(180deg, #f0f7ff 0%, #fafbfc 60%, #f5f7fa 100%); z-index: 0; }
.hero-blob { position: absolute; border-radius: 50%; filter: blur(80px); opacity: 0.4; }
.hero-blob-1 { top: -100px; left: 10%; width: 400px; height: 400px; background: radial-gradient(circle, #b3d8ff 0%, transparent 70%); }
.hero-blob-2 { top: 50px; right: 5%; width: 350px; height: 350px; background: radial-gradient(circle, #d4f0d4 0%, transparent 70%); }
.hero-content { position: relative; z-index: 1; max-width: 720px; margin: 0 auto; text-align: center; }
.hero-title { margin: 0 0 12px; font-size: 32px; font-weight: 700; color: #1a1a1a; letter-spacing: -0.5px; line-height: 1.3; }
.hero-subtitle { margin: 0 0 32px; font-size: 15px; color: #606266; line-height: 1.6; }
.hero-search { display: flex; align-items: center; gap: 12px; height: 64px; padding: 0 20px; background: #fff; border: 2px solid #e4e7ed; border-radius: 16px; cursor: pointer; box-shadow: 0 8px 24px rgba(64, 158, 255, 0.08), 0 2px 6px rgba(0, 0, 0, 0.04); transition: all 0.2s; user-select: none; }
.hero-search:hover { border-color: #409eff; box-shadow: 0 12px 32px rgba(64, 158, 255, 0.15), 0 4px 8px rgba(0, 0, 0, 0.05); transform: translateY(-1px); }
.hero-search-icon { font-size: 24px; flex-shrink: 0; color: #409eff; }
.hero-search-text { flex: 1; font-size: 17px; color: #909399; text-align: left; font-weight: 400; }
.hero-search-kbd { display: flex; gap: 4px; flex-shrink: 0; }
.hero-search-kbd kbd { font-family: 'SF Mono', Consolas, monospace; font-size: 12px; padding: 4px 8px; background: #f5f7fa; border: 1px solid #dcdfe6; border-radius: 4px; color: #606266; box-shadow: 0 1px 0 #dcdfe6; line-height: 1; }
.hero-hints { display: flex; align-items: center; justify-content: center; gap: 8px; margin-top: 20px; font-size: 13px; color: #909399; flex-wrap: wrap; }
.hero-hint { padding: 3px 10px; background: rgba(255, 255, 255, 0.6); border: 1px solid #e4e7ed; border-radius: 12px; cursor: pointer; color: #409eff; font-weight: 500; transition: all 0.15s; }
.hero-hint:hover { background: #409eff; color: #fff; border-color: #409eff; }
.categories { max-width: 960px; margin: 0 auto; padding: 24px 24px 40px; }
.section-title { display: flex; align-items: baseline; gap: 12px; margin: 0 0 20px; font-size: 18px; font-weight: 600; color: #303133; }
.section-sub { font-size: 13px; font-weight: 400; color: #909399; }
.category-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 16px; }
.category-card { position: relative; padding: 0; background: #fff; border: 1px solid #ebeef5; border-radius: 14px; cursor: pointer; overflow: hidden; transition: all 0.2s; min-height: 160px; }
.category-card:not(.disabled):hover { transform: translateY(-3px); box-shadow: 0 12px 28px rgba(0, 0, 0, 0.08); border-color: transparent; }
.category-card.disabled { opacity: 0.5; cursor: not-allowed; }
.category-glow { position: absolute; left: 0; top: 0; bottom: 0; width: 5px; opacity: 0.85; transition: width 0.2s; }
.category-card:not(.disabled):hover .category-glow { width: 8px; }
.category-inner { padding: 24px 24px 24px 28px; display: flex; flex-direction: column; gap: 6px; height: 100%; }
.category-icon { font-size: 32px; line-height: 1; margin-bottom: 6px; height: 48px; display: flex; align-items: center; }
.category-icon-img { height: 48px; width: 48px; object-fit: contain; display: block; }
.category-icon-emoji { font-size: 32px; line-height: 1; }
.category-name { font-size: 18px; font-weight: 600; color: #303133; }
.category-desc { font-size: 13px; color: #909399; line-height: 1.5; flex: 1; }
.category-meta { display: flex; align-items: center; justify-content: space-between; margin-top: 8px; }
.category-count { font-size: 13px; color: #606266; }
.category-count strong { font-size: 20px; font-weight: 700; color: #303133; font-variant-numeric: tabular-nums; margin-right: 2px; }
.category-empty { font-size: 12px; color: #c0c4cc; }
.category-arrow { font-size: 18px; color: #c0c4cc; transition: all 0.15s; }
.category-card:not(.disabled):hover .category-arrow { color: #409eff; transform: translateX(3px); }
</style>
