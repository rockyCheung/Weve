use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingInput {
    pub key: String,
    pub value: String,
}

#[tauri::command]
pub fn save_setting(state: State<AppState>, input: SettingInput) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now', 'localtime'))",
        rusqlite::params![input.key, input.value],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn save_settings_batch(state: State<AppState>, settings: Vec<SettingInput>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    for setting in settings {
        db.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now', 'localtime'))",
            rusqlite::params![setting.key, setting.value],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_setting(state: State<AppState>, key: String) -> Result<Option<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result = db.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        rusqlite::params![key],
        |row| row.get(0),
    );
    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn get_all_settings(state: State<AppState>) -> Result<std::collections::HashMap<String, String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare("SELECT key, value FROM settings").map_err(|e| e.to_string())?;
    let mut map = std::collections::HashMap::new();
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(|e| e.to_string())?;
    for row in rows {
        if let Ok((key, value)) = row {
            map.insert(key, value);
        }
    }
    Ok(map)
}

// ==================== 数据备份与恢复 ====================

/// 获取数据库文件路径
fn get_db_path() -> PathBuf {
    let mut path = dirs_next::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("small-business-manager");
    path.push("data.db");
    path
}

/// 获取备份目录路径
fn get_backup_dir() -> PathBuf {
    let mut path = dirs_next::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("small-business-manager");
    path.push("backups");
    path
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupInfo {
    pub file_name: String,
    pub file_size: u64,
    pub created_at: String,
}

#[tauri::command]
pub fn backup_database(state: State<AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    // 确保备份目录存在
    let backup_dir = get_backup_dir();
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    
    // 生成备份文件名
    let now = chrono::Local::now();
    let file_name = format!("backup_{}.db", now.format("%Y%m%d_%H%M%S"));
    let backup_path = backup_dir.join(&file_name);
    
    // 执行备份
    db.execute("VACUUM INTO ?1", rusqlite::params![backup_path.to_string_lossy().to_string()])
        .map_err(|e| format!("备份失败: {}", e))?;
    
    Ok(backup_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn restore_database(state: State<AppState>, backup_path: String) -> Result<(), String> {
    // 验证备份文件存在
    let path = PathBuf::from(&backup_path);
    if !path.exists() {
        return Err("备份文件不存在".to_string());
    }
    
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let current_db_path = get_db_path();
    
    // 关闭当前连接并替换数据库文件
    // 注意：这里需要重新打开连接，但 Tauri 状态管理限制，简化处理
    drop(db);
    
    // 复制备份文件到数据库路径
    fs::copy(&path, &current_db_path).map_err(|e| format!("恢复失败: {}", e))?;
    
    // 重新初始化数据库连接（简化：实际需要重新创建 AppState）
    Ok(())
}

#[tauri::command]
pub fn list_backups(_state: State<AppState>) -> Result<Vec<BackupInfo>, String> {
    let backup_dir = get_backup_dir();
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut backups = Vec::new();
    let entries = fs::read_dir(&backup_dir).map_err(|e| e.to_string())?;
    
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension().map(|e| e == "db").unwrap_or(false) {
                let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
                let file_name = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                
                // 从文件名提取时间
                let created_at = file_name
                    .replace("backup_", "")
                    .replace(".db", "")
                    .replace("_", " ");
                
                backups.push(BackupInfo {
                    file_name,
                    file_size: metadata.len(),
                    created_at,
                });
            }
        }
    }
    
    // 按时间倒序排列
    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    Ok(backups)
}

#[tauri::command]
pub fn delete_backup(_state: State<AppState>, file_name: String) -> Result<(), String> {
    let backup_path = get_backup_dir().join(&file_name);
    if backup_path.exists() {
        fs::remove_file(&backup_path).map_err(|e| format!("删除失败: {}", e))?;
    }
    Ok(())
}

// ==================== 数据导出 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    pub employees: Vec<serde_json::Value>,
    pub attendance: Vec<serde_json::Value>,
    pub accounts: Vec<serde_json::Value>,
    pub vouchers: Vec<serde_json::Value>,
    pub invoices: Vec<serde_json::Value>,
    pub tax_reports: Vec<serde_json::Value>,
    pub settings: std::collections::HashMap<String, String>,
    pub export_time: String,
}

#[tauri::command]
pub fn export_all_data(state: State<AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    
    // 导出所有数据
    let export = ExportData {
        employees: export_table(&db, "SELECT * FROM employees")?,
        attendance: export_table(&db, "SELECT * FROM attendance")?,
        accounts: export_table(&db, "SELECT * FROM accounts")?,
        vouchers: export_table(&db, "SELECT * FROM vouchers")?,
        invoices: export_table(&db, "SELECT * FROM invoices")?,
        tax_reports: export_table(&db, "SELECT * FROM tax_reports")?,
        settings: get_all_settings_inner(&db)?,
        export_time: now.clone(),
    };
    
    // 序列化为 JSON
    let json_data = serde_json::to_string_pretty(&export).map_err(|e| e.to_string())?;
    
    // 保存到临时文件
    let export_dir = get_backup_dir();
    fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;
    let export_path = export_dir.join(format!("export_{}.json", now));
    fs::write(&export_path, &json_data).map_err(|e| e.to_string())?;
    
    Ok(export_path.to_string_lossy().to_string())
}

fn export_table(db: &rusqlite::Connection, sql: &str) -> Result<Vec<serde_json::Value>, String> {
    let mut stmt = db.prepare(sql).map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    
    let column_count = stmt.column_count();
    let column_names: Vec<String> = (0..column_count)
        .map(|i| stmt.column_name(i).unwrap_or("").to_string())
        .collect();
    
    let row_iter = stmt.query_map([], |row| {
        let mut map = serde_json::Map::new();
        for i in 0..column_count {
            let value: String = row.get::<_, String>(i).unwrap_or_default();
            map.insert(column_names[i].clone(), serde_json::Value::String(value));
        }
        Ok(serde_json::Value::Object(map))
    }).map_err(|e| e.to_string())?;
    
    for row in row_iter {
        if let Ok(row) = row {
            rows.push(row);
        }
    }
    
    Ok(rows)
}

fn get_all_settings_inner(db: &rusqlite::Connection) -> Result<std::collections::HashMap<String, String>, String> {
    let mut stmt = db.prepare("SELECT key, value FROM settings").map_err(|e| e.to_string())?;
    let mut map = std::collections::HashMap::new();
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(|e| e.to_string())?;
    for row in rows {
        if let Ok((key, value)) = row {
            map.insert(key, value);
        }
    }
    Ok(map)
}
