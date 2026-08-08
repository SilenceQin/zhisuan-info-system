// 智算信息查询系统 - Tauri 后端核心
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager, State};

// ============== 类型定义 ==============

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColumnInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub col_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableInfo {
    pub table_name: String,
    pub file_path: String,
    pub row_count: i64,
    pub imported_at: i64,
    pub duration_ms: i64,
    pub columns: Vec<ColumnInfo>,
    pub sheet_name: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportResult {
    pub table_name: String,
    pub sheet_name: Option<String>,
    pub row_count: i64,
    pub columns: Vec<ColumnInfo>,
    pub imported_at: i64,
    pub duration_ms: i64,
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryResult {
    pub rows: Vec<serde_json::Value>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataSourceConfig {
    pub file_path: String,
    pub file_type: String,
    pub sheet_name: Option<String>,
    pub display_name: String,
    pub updated_at: i64,
}

// ============== 状态 ==============

pub struct AppState {
    pub db: Mutex<Connection>,
}

// ============== 数据库初始化 ==============

fn init_db(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS _import_meta (
            table_name TEXT PRIMARY KEY,
            file_path TEXT NOT NULL,
            row_count INTEGER NOT NULL,
            imported_at INTEGER NOT NULL,
            duration_ms INTEGER NOT NULL,
            columns_json TEXT NOT NULL,
            sheet_name TEXT,
            category TEXT
        );
        "#,
    )?;
    Ok(())
}

fn infer_type(value: &str) -> &'static str {
    let s = value.trim();
    if s.is_empty() {
        return "string";
    }
    if s.parse::<f64>().is_ok() {
        return "number";
    }
    let lower = s.to_lowercase();
    if lower == "true" || lower == "false" || s == "是" || s == "否" {
        return "boolean";
    }
    if chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
        || chrono::NaiveDate::parse_from_str(s, "%Y/%m/%d").is_ok()
        || chrono::NaiveDate::parse_from_str(s, "%Y.%m.%d").is_ok()
    {
        return "date";
    }
    "string"
}

fn sanitize_name(name: &str) -> String {
    let mut s: String = name
        .chars()
        .filter(|&c| c != '"' && c != '`' && c != '\\' && (c.is_ascii_graphic() || c as u32 > 0x7f))
        .collect();
    s = s.trim().to_string();
    if s.is_empty() {
        s = "unnamed".to_string();
    }
    if s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        s = format!("t_{}", s);
    }
    s
}

fn simple_hash(s: &str) -> String {
    let mut h: i64 = 0;
    for c in s.chars() {
        h = (h << 5) - h + (c as i64);
    }
    format!("{:x}", h.abs()).chars().take(6).collect()
}

fn detect_category(sheet_name: &str) -> String {
    let s = sheet_name.to_lowercase();
    if s.contains("芯片") || s.contains("chip") || s.contains("gpu") {
        "chips".into()
    } else if s.contains("服务器") || s.contains("server") || s.contains("整机") {
        "servers".into()
    } else if s.contains("超节点")
        || s.contains("super")
        || s.contains("超算")
        || s.contains("集群")
    {
        "super_nodes".into()
    } else {
        "other".into()
    }
}

fn json_value_to_sql(col_type: &str, value: &str) -> rusqlite::types::Value {
    let s = value.trim();
    if s.is_empty() {
        return rusqlite::types::Value::Null;
    }
    match col_type {
        "number" => s
            .parse::<f64>()
            .map(rusqlite::types::Value::Real)
            .unwrap_or(rusqlite::types::Value::Null),
        "boolean" => match s.to_lowercase().as_str() {
            "true" | "是" | "yes" | "1" => rusqlite::types::Value::Integer(1),
            "false" | "否" | "no" | "0" => rusqlite::types::Value::Integer(0),
            _ => rusqlite::types::Value::Null,
        },
        "date" => rusqlite::types::Value::Text(s.to_string()),
        _ => rusqlite::types::Value::Text(s.to_string()),
    }
}

// ============== 导入逻辑（从原 importer.ts 移植）==============

fn ingest_excel(
    conn: &Connection,
    file_path: &str,
    display_name: &str,
) -> anyhow::Result<ImportResult> {
    use calamine::{open_workbook_auto, Data, Reader};
    let start = Instant::now();
    let mut wb = open_workbook_auto(file_path)?;
    let sheet_names = wb.sheet_names().to_vec();
    if sheet_names.is_empty() {
        anyhow::bail!("Excel 文件没有 sheet");
    }

    let mut first_result: Option<ImportResult> = None;
    for sheet_name in sheet_names {
        let range = wb.worksheet_range(&sheet_name).unwrap();
        let mut rows_iter = range.rows();
        let header_row = match rows_iter.next() {
            Some(r) => r,
            None => continue,
        };

        // 处理表头
        let mut headers: Vec<String> = Vec::new();
        let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for (i, cell) in header_row.iter().enumerate() {
            let mut name = cell_to_string(cell).trim().to_string();
            if name.is_empty() {
                name = format!("列_{}", i + 1);
            }
            if let Some(&n) = seen.get(&name) {
                seen.insert(name.clone(), n + 1);
                name = format!("{}_{}", name, n + 1);
            } else {
                seen.insert(name.clone(), 1);
            }
            headers.push(name);
        }

        // 收集行
        let mut data_rows: Vec<Vec<String>> = Vec::new();
        for row in rows_iter {
            let mut r: Vec<String> = Vec::new();
            let mut all_empty = true;
            for cell in row {
                let s = cell_to_string(cell);
                if !s.trim().is_empty() {
                    all_empty = false;
                }
                r.push(s);
            }
            if !all_empty {
                data_rows.push(r);
            }
        }

        if data_rows.is_empty() {
            continue;
        }

        // 推断列类型
        let sample = data_rows.iter().take(100);
        let mut column_types: Vec<&str> = Vec::with_capacity(headers.len());
        for idx in 0..headers.len() {
            let votes = ["string", "number", "boolean", "date"]
                .iter()
                .map(|t| {
                    let cnt = sample
                        .clone()
                        .filter(|r| {
                            let v = r.get(idx).map(|s| s.as_str()).unwrap_or("");
                            !v.trim().is_empty() && infer_type(v) == *t
                        })
                        .count();
                    (t, cnt)
                })
                .collect::<Vec<_>>();
            let best = votes
                .iter()
                .max_by_key(|(_, c)| *c)
                .map(|(t, _)| *t)
                .copied()
                .unwrap_or("string");
            column_types.push(best);
        }

        // 建表
        let base = sanitize_name(&sheet_name);
        let hash = simple_hash(file_path);
        let table_name = format!("data_{}_{}", base, hash);
        let table_name = table_name.chars().take(60).collect::<String>();

        conn.execute(&format!("DROP TABLE IF EXISTS \"{}\"", table_name), [])?;
        let col_defs: Vec<String> = headers
            .iter()
            .zip(column_types.iter())
            .map(|(h, t)| {
                let sql_type = match *t {
                    "number" => "REAL",
                    "boolean" => "INTEGER",
                    _ => "TEXT",
                };
                format!("\"{}\" {}", sanitize_name(h), sql_type)
            })
            .collect();
        conn.execute(
            &format!(
                "CREATE TABLE \"{}\" (id INTEGER PRIMARY KEY AUTOINCREMENT, {})",
                table_name,
                col_defs.join(", ")
            ),
            [],
        )?;

        // 批量插入
        let safe_cols: Vec<String> = headers.iter().map(|h| sanitize_name(h)).collect();
        let placeholders = safe_cols.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let col_list = safe_cols
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<_>>()
            .join(", ");
        let insert_sql = format!(
            "INSERT INTO \"{}\" ({}) VALUES ({})",
            table_name, col_list, placeholders
        );
        {
            let tx = conn.unchecked_transaction()?;
            {
                let mut stmt = tx.prepare(&insert_sql)?;
                for row in &data_rows {
                    let ps: Vec<rusqlite::types::Value> = row
                        .iter()
                        .zip(column_types.iter())
                        .map(|(v, t)| json_value_to_sql(t, v))
                        .collect();
                    stmt.execute(rusqlite::params_from_iter(ps.iter()))?;
                }
            }
            tx.commit()?;
        }

        // 元信息
        let columns: Vec<ColumnInfo> = headers
            .iter()
            .zip(column_types.iter())
            .map(|(n, t)| ColumnInfo {
                name: n.clone(),
                col_type: t.to_string(),
            })
            .collect();
        let category = detect_category(&sheet_name);
        let now = chrono::Utc::now().timestamp_millis();
        let duration = start.elapsed().as_millis() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO _import_meta (table_name, file_path, row_count, imported_at, duration_ms, columns_json, sheet_name, category) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                table_name,
                file_path,
                data_rows.len() as i64,
                now,
                duration,
                serde_json::to_string(&columns)?,
                Some(sheet_name.as_str()),
                Some(category.as_str())
            ],
        )?;

        if first_result.is_none() {
            first_result = Some(ImportResult {
                table_name,
                sheet_name: Some(sheet_name),
                row_count: data_rows.len() as i64,
                columns,
                imported_at: now,
                duration_ms: duration,
                category,
            });
        }
    }

    first_result.ok_or_else(|| anyhow::anyhow!("所有 sheet 都为空"))
}

fn cell_to_string(cell: &calamine::Data) -> String {
    use calamine::Data as D;
    match cell {
        D::Empty => String::new(),
        D::String(s) => s.clone(),
        D::Int(i) => i.to_string(),
        D::Float(f) => {
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                f.to_string()
            }
        }
        D::Bool(b) => (if *b { "是" } else { "否" }).to_string(),
        D::DateTime(d) => d.to_string(),
        D::DateTimeIso(s) | D::DurationIso(s) => s.clone(),
        D::Error(_) => String::new(),
    }
}

// ============== Tauri Commands ==============

#[tauri::command]
fn get_app_info() -> serde_json::Value {
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    })
}

#[tauri::command]
fn import_file(
    state: State<AppState>,
    app: AppHandle,
    file_path: String,
) -> Result<ImportResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let display_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(&file_path)
        .to_string();
    let result = ingest_excel(&conn, &file_path, &display_name).map_err(|e| e.to_string())?;

    // 启动文件监听
    let _ = app.emit("datasource:imported", &result);
    Ok(result)
}

#[tauri::command]
fn list_tables(
    state: State<AppState>,
    category: Option<String>,
) -> Result<Vec<TableInfo>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut sql = String::from(
        "SELECT table_name, file_path, row_count, imported_at, duration_ms, columns_json, sheet_name, category FROM _import_meta",
    );
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(c) = category {
        sql += " WHERE category = ?";
        params_vec.push(Box::new(c));
    }
    sql += " ORDER BY imported_at DESC";

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
    let rows = stmt
        .query_map(params_refs.as_slice(), |r| {
            let columns_json: String = r.get(5)?;
            Ok(TableInfo {
                table_name: r.get(0)?,
                file_path: r.get(1)?,
                row_count: r.get(2)?,
                imported_at: r.get(3)?,
                duration_ms: r.get(4)?,
                columns: serde_json::from_str(&columns_json).unwrap_or_default(),
                sheet_name: r.get(6)?,
                category: r.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
fn query_table(
    state: State<AppState>,
    table_name: String,
    page: i64,
    page_size: i64,
    keyword: Option<String>,
    sort_field: Option<String>,
    sort_order: Option<String>,
) -> Result<QueryResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // 取列信息
    let columns_json: String = conn
        .query_row(
            "SELECT columns_json FROM _import_meta WHERE table_name = ?",
            params![table_name],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let columns: Vec<ColumnInfo> = serde_json::from_str(&columns_json).unwrap_or_default();

    let offset = (page.max(1) - 1) * page_size;
    let mut where_clause = String::new();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(kw) = keyword {
        if !kw.trim().is_empty() {
            let string_cols: Vec<&ColumnInfo> =
                columns.iter().filter(|c| c.col_type == "string").collect();
            if !string_cols.is_empty() {
                let cond = string_cols
                    .iter()
                    .map(|c| format!("\"{}\" LIKE ?", sanitize_name(&c.name)))
                    .collect::<Vec<_>>()
                    .join(" OR ");
                where_clause = format!("WHERE {}", cond);
                for _ in &string_cols {
                    params_vec.push(Box::new(format!("%{}%", kw)));
                }
            }
        }
    }

    // 总数
    let count_sql = format!("SELECT COUNT(*) FROM \"{}\" {}", table_name, where_clause);
    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
    let total: i64 = conn
        .query_row(&count_sql, params_refs.as_slice(), |r| r.get(0))
        .map_err(|e| e.to_string())?;

    // 排序
    let order_by = if let Some(field) = sort_field {
        format!(
            "ORDER BY \"{}\" {}",
            sanitize_name(&field),
            if sort_order.as_deref() == Some("desc") {
                "DESC"
            } else {
                "ASC"
            }
        )
    } else {
        "ORDER BY id ASC".to_string()
    };

    let data_sql = format!(
        "SELECT * FROM \"{}\" {} {} LIMIT ? OFFSET ?",
        table_name, where_clause, order_by
    );

    // 把 pageSize/offset 追加到原来的 vec(避免 Box<dyn ToSql> 不支持 clone)
    let mut all_params = params_vec;
    all_params.push(Box::new(page_size));
    all_params.push(Box::new(offset));
    let all_params_refs: Vec<&dyn rusqlite::ToSql> =
        all_params.iter().map(|b| b.as_ref()).collect();

    let mut stmt = conn.prepare(&data_sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(all_params_refs.as_slice(), |r| {
            let column_count = r.as_ref().column_count();
            let mut obj = serde_json::Map::new();
            for i in 0..column_count {
                let name = r.as_ref().column_name(i).unwrap_or("?").to_string();
                let v: rusqlite::types::Value = r.get(i)?;
                let jv = match v {
                    rusqlite::types::Value::Null => serde_json::Value::Null,
                    rusqlite::types::Value::Integer(i) => serde_json::json!(i),
                    rusqlite::types::Value::Real(f) => serde_json::json!(f),
                    rusqlite::types::Value::Text(s) => serde_json::Value::String(s),
                    rusqlite::types::Value::Blob(_) => serde_json::Value::Null,
                };
                obj.insert(name, jv);
            }
            Ok(serde_json::Value::Object(obj))
        })
        .map_err(|e| e.to_string())?;

    let mut result_rows = Vec::new();
    for row in rows {
        result_rows.push(row.map_err(|e| e.to_string())?);
    }

    Ok(QueryResult {
        rows: result_rows,
        total,
        page,
        page_size,
        columns,
    })
}

#[tauri::command]
fn query_all_rows(
    state: State<AppState>,
    table_name: String,
) -> Result<QueryResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let columns_json: String = conn
        .query_row(
            "SELECT columns_json FROM _import_meta WHERE table_name = ?",
            params![table_name],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let columns: Vec<ColumnInfo> = serde_json::from_str(&columns_json).unwrap_or_default();

    let total: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM \"{}\"", table_name), [], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(&format!("SELECT * FROM \"{}\" ORDER BY id ASC", table_name))
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            let column_count = r.as_ref().column_count();
            let mut obj = serde_json::Map::new();
            for i in 0..column_count {
                let name = r.as_ref().column_name(i).unwrap_or("?").to_string();
                let v: rusqlite::types::Value = r.get(i)?;
                let jv = match v {
                    rusqlite::types::Value::Null => serde_json::Value::Null,
                    rusqlite::types::Value::Integer(i) => serde_json::json!(i),
                    rusqlite::types::Value::Real(f) => serde_json::json!(f),
                    rusqlite::types::Value::Text(s) => serde_json::Value::String(s),
                    rusqlite::types::Value::Blob(_) => serde_json::Value::Null,
                };
                obj.insert(name, jv);
            }
            Ok(serde_json::Value::Object(obj))
        })
        .map_err(|e| e.to_string())?;

    let mut result_rows = Vec::new();
    for row in rows {
        result_rows.push(row.map_err(|e| e.to_string())?);
    }

    Ok(QueryResult {
        rows: result_rows,
        total,
        page: 1,
        page_size: total,
        columns,
    })
}

// ============== 入口 ==============

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri_plugin_log::{Target, TargetKind};

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                ])
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            // 决定 db 文件位置
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).ok();
            let db_path = app_data_dir.join("data.db");
            log::info!("db path: {}", db_path.display());

            let conn = Connection::open(&db_path).expect("failed to open db");
            init_db(&conn).expect("failed to init db");

            // 首次启动:如果 db 是空的,自动从内置资源导入数据
            let existing: i64 = conn
                .query_row("SELECT COUNT(*) FROM _import_meta", [], |r| r.get(0))
                .unwrap_or(0);
            if existing == 0 {
                // 找内置 Excel 资源(打包时通过 tauri.conf.json 的 bundle.resources 注入)
                let candidates = vec![
                    app.path().resource_dir().ok().map(|d| d.join("resources/内置数据.xlsx")),
                    app.path().resource_dir().ok().map(|d| d.join("_up_/resources/内置数据.xlsx")),
                    app.path().resource_dir().ok().map(|d| d.join("内置数据.xlsx")),
                    Some(std::env::current_exe().ok()
                        .and_then(|e| e.parent().map(|p| p.to_path_buf()))
                        .unwrap_or_default()
                        .join("resources/内置数据.xlsx")),
                ];
                for path in candidates.into_iter().flatten() {
                    if path.exists() {
                        log::info!("auto-importing built-in data from: {}", path.display());
                        match ingest_excel(&conn, path.to_str().unwrap(), "算力数据.xlsx") {
                            Ok(r) => log::info!("auto-imported {} rows, category={}", r.row_count, r.category),
                            Err(e) => log::error!("auto-import failed: {}", e),
                        }
                        break;
                    }
                }
            } else {
                log::info!("db already has {} tables, skip auto-import", existing);
            }

            app.manage(AppState {
                db: Mutex::new(conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            import_file,
            list_tables,
            query_table,
            query_all_rows,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
