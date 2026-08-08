import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api, type DataSourceConfig, type ImportResult } from '../api'
import { Store } from '@tauri-apps/plugin-store'

let _store: Store | null = null
async function getStore() {
  if (!_store) _store = await Store.load('app-config.json')
  return _store
}

export const useDataStore = defineStore('data', () => {
  const dataSource = ref<DataSourceConfig | null>(null)
  const lastImport = ref<ImportResult | null>(null)
  const loading = ref(false)

  async function loadDataSource() {
    const store = await getStore()
    const saved = await store.get<DataSourceConfig>('dataSource')
    if (saved) dataSource.value = saved
  }

  async function pickAndImport() {
    loading.value = true
    try {
      const filePath = await api.pickFile()
      if (!filePath) return
      const result = await api.importFile(filePath)
      const fileName = filePath.split(/[/\\]/).pop() || filePath
      const cfg: DataSourceConfig = {
        filePath,
        fileType: filePath.split('.').pop()?.toLowerCase() || 'xlsx',
        sheetName: result.sheet_name ?? undefined,
        displayName: fileName,
        updatedAt: Date.now()
      }
      dataSource.value = cfg
      lastImport.value = result
      const store = await getStore()
      await store.set('dataSource', cfg)
      await store.save()
    } finally {
      loading.value = false
    }
  }

  async function reload() {
    if (!dataSource.value) throw new Error('未配置数据源')
    loading.value = true
    try {
      const r = await api.reloadData(dataSource.value.filePath)
      lastImport.value = r
      dataSource.value = { ...dataSource.value, updatedAt: Date.now() }
    } finally {
      loading.value = false
    }
  }

  async function clear() {
    dataSource.value = null
    lastImport.value = null
    const store = await getStore()
    await store.set('dataSource', null)
    await store.save()
  }

  return { dataSource, lastImport, loading, loadDataSource, pickAndImport, reload, clear }
})
