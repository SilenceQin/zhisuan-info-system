// 主进程和渲染进程共享的类型定义

/** 数据源文件类型 */
export type DataSourceType = 'xlsx' | 'xls' | 'csv'

/** 业务分类(根据 sheet 名前缀自动归类) */
export type DataCategory = 'chips' | 'servers' | 'super_nodes' | 'other'

/** 数据源配置(存到 electron-store) */
export interface DataSourceConfig {
  filePath: string
  fileType: DataSourceType
  sheetName?: string          // xlsx 才有
  displayName: string
  updatedAt: number
}

/** 表格列信息(导入时推断) */
export interface ColumnInfo {
  name: string                // 中文表头原名
  type: 'string' | 'number' | 'date' | 'boolean'
  sample?: string | number | null
}

/** 导入结果 */
export interface ImportResult {
  tableName: string
  sheetName?: string
  rowCount: number
  columns: ColumnInfo[]
  importedAt: number
  durationMs: number
  category?: DataCategory     // 自动识别的业务分类
}

/** 通用结果包装 */
export interface ApiResult<T = unknown> {
  ok: boolean
  data?: T
  error?: string
}

/** 更新信息 */
export interface UpdateInfo {
  hasUpdate: boolean
  version?: string
  releaseDate?: string
  releaseNotes?: string
}

/** 更新进度 */
export interface UpdateProgress {
  percent: number
  transferred: number
  total: number
  bytesPerSecond: number
}

/** 搜索结果(命令面板用) */
export interface SearchHit {
  category: DataCategory
  sheetName: string
  row: Record<string, any>
  /** 用于显示的主标题(型号/名称) */
  title: string
  /** 用于显示的副标题(厂商/分类等) */
  subtitle: string
  /** 匹配得分(数字越大越相关) */
  score: number
  /** 高亮区间(title 上匹配的字符位置) */
  titleMatch?: [number, number]
}
