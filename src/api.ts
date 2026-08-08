// Tauri 2 API 包装层 - 替代原 Electron preload 暴露的 window.api
// 所有 invoke / event 调用都在这里统一封装

import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { open as openDialog } from '@tauri-apps/plugin-dialog'

// ============ 类型定义（与 Rust 端一一对应）============

export interface ColumnInfo {
  name: string
  type: string
}

export interface TableInfo {
  table_name: string
  file_path: string
  row_count: number
  imported_at: number
  duration_ms: number
  columns: ColumnInfo[]
  sheet_name?: string
  category?: string
}

export interface ImportResult {
  table_name: string
  sheet_name?: string
  row_count: number
  columns: ColumnInfo[]
  imported_at: number
  duration_ms: number
  category: string
}

export interface QueryResult {
  rows: Record<string, any>[]
  total: number
  page: number
  page_size: number
  columns: ColumnInfo[]
}

export interface DataSourceConfig {
  filePath: string
  fileType: string
  sheetName?: string
  displayName: string
  updatedAt: number
}

export interface AppInfo {
  version: string
  platform: string
  arch: string
}

export type DataCategory = 'chips' | 'servers' | 'super_nodes' | 'other'

// ============ API 包装 ============

export const api = {
  // ---- 数据源 ----
  pickFile: async (): Promise<string | null> => {
    const selected = await openDialog({
      multiple: false,
      filters: [{ name: '数据文件', extensions: ['xlsx', 'xls', 'csv'] }]
    })
    if (!selected || Array.isArray(selected)) return null
    return selected
  },

  importFile: async (filePath: string): Promise<ImportResult> => {
    return await invoke<ImportResult>('import_file', { filePath })
  },

  reloadData: async (filePath: string): Promise<ImportResult> => {
    return await invoke<ImportResult>('import_file', { filePath })
  },

  // ---- 表格查询 ----
  listTables: async (): Promise<TableInfo[]> => {
    return await invoke<TableInfo[]>('list_tables')
  },

  listByCategory: async (category: DataCategory): Promise<TableInfo[]> => {
    return await invoke<TableInfo[]>('list_tables', { category })
  },

  queryAllRows: async (tableName: string): Promise<QueryResult> => {
    return await invoke<QueryResult>('query_all_rows', { tableName })
  },

  // ---- 应用信息 ----
  getAppInfo: async (): Promise<AppInfo> => {
    return await invoke<AppInfo>('get_app_info')
  },

  // ---- 事件订阅 ----
  onAutoReloaded: (cb: (data: { rowCount: number; importedAt: number }) => void): Promise<UnlistenFn> => {
    return listen<{ rowCount: number; importedAt: number }>('datasource:auto-reloaded', (e) => cb(e.payload))
  },

  onAutoReloadFailed: (cb: (data: { error: string }) => void): Promise<UnlistenFn> => {
    return listen<{ error: string }>('datasource:auto-reload-failed', (e) => cb(e.payload))
  }
}
