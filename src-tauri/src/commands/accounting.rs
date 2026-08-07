use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use std::collections::HashMap;

// ==================== 科目管理 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInput {
    pub code: String,
    pub name: String,
    pub category: String,  // asset / liability / equity / income / expense
    pub parent_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountTree {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub category: String,
    pub parent_id: Option<i64>,
    pub is_active: bool,
    pub children: Vec<AccountTree>,
}

#[tauri::command]
pub fn create_account(state: State<AppState>, input: AccountInput) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    // 检查科目编码是否已存在
    let exists: bool = db.query_row(
        "SELECT COUNT(*) > 0 FROM accounts WHERE code = ?1",
        rusqlite::params![input.code],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    if exists {
        return Err(format!("科目编码 '{}' 已存在", input.code));
    }
    
    db.execute(
        "INSERT INTO accounts (code, name, category, parent_id) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![input.code, input.name, input.category, input.parent_id],
    ).map_err(|e| e.to_string())?;
    Ok(db.last_insert_rowid())
}

#[tauri::command]
pub fn get_account_tree(state: State<AppState>) -> Result<Vec<AccountTree>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT id, code, name, category, parent_id, is_active FROM accounts ORDER BY code"
    ).map_err(|e| e.to_string())?;
    
    let accounts: Vec<AccountTree> = stmt.query_map([], |row| {
        Ok(AccountTree {
            id: row.get(0)?,
            code: row.get(1)?,
            name: row.get(2)?,
            category: row.get(3)?,
            parent_id: row.get(4)?,
            is_active: row.get::<_, i32>(5)? != 0,
            children: Vec::new(),
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    // 构建树形结构
    Ok(build_account_tree(accounts))
}

fn build_account_tree(accounts: Vec<AccountTree>) -> Vec<AccountTree> {
    let mut map: HashMap<i64, AccountTree> = HashMap::new();
    
    // 先插入所有节点
    for acc in accounts {
        map.insert(acc.id, acc);
    }
    
    // 构建父子关系（按编码排序，父级科目编码较短、id 较小，先处理深层节点）
    let mut ids: Vec<i64> = map.keys().cloned().collect();
    ids.sort_by(|a, b| b.cmp(a));
    for id in &ids {
        let (parent_id, node) = match map.get(id) {
            Some(acc) => (acc.parent_id, acc.clone()),
            None => continue,
        };
        if let Some(parent_id) = parent_id {
            if let Some(parent) = map.get_mut(&parent_id) {
                parent.children.push(node);
                parent.children.sort_by(|a, b| a.code.cmp(&b.code));
            }
        }
    }
    
    // 收集根节点
    let mut roots: Vec<AccountTree> = map
        .into_values()
        .filter(|acc| acc.parent_id.is_none())
        .collect();
    roots.sort_by(|a, b| a.code.cmp(&b.code));
    roots
}

#[tauri::command]
pub fn update_account(state: State<AppState>, id: i64, name: Option<String>, is_active: Option<bool>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    if let Some(name) = name {
        db.execute(
            "UPDATE accounts SET name = ?1 WHERE id = ?2",
            rusqlite::params![name, id],
        ).map_err(|e| e.to_string())?;
    }
    if let Some(is_active) = is_active {
        db.execute(
            "UPDATE accounts SET is_active = ?1 WHERE id = ?2",
            rusqlite::params![is_active as i32, id],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn delete_account(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    // 检查是否有子科目
    let has_children: bool = db.query_row(
        "SELECT COUNT(*) > 0 FROM accounts WHERE parent_id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    if has_children {
        return Err("该科目下有子科目，无法删除".to_string());
    }
    // 检查是否有凭证引用
    let has_vouchers: bool = db.query_row(
        "SELECT COUNT(*) > 0 FROM vouchers WHERE debit_account_id = ?1 OR credit_account_id = ?1",
        rusqlite::params![id, id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    if has_vouchers {
        return Err("该科目已被凭证引用，无法删除，建议停用".to_string());
    }
    
    db.execute("DELETE FROM accounts WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 凭证管理 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct VoucherEntry {
    pub account_id: i64,
    pub debit_amount: f64,
    pub credit_amount: f64,
    pub summary: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoucherInput {
    pub date: String,
    pub summary: String,
    pub entries: Vec<VoucherEntry>,
    pub attachment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoucherDetail {
    pub id: i64,
    pub voucher_no: String,
    pub date: String,
    pub summary: String,
    pub entries: Vec<VoucherEntryDetail>,
    pub attachment: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoucherEntryDetail {
    pub account_id: i64,
    pub account_code: String,
    pub account_name: String,
    pub debit_amount: f64,
    pub credit_amount: f64,
    pub summary: String,
}

#[tauri::command]
pub fn post_voucher(state: State<AppState>, input: VoucherInput) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    // 验证借贷平衡
    let total_debit: f64 = input.entries.iter().map(|e| e.debit_amount).sum();
    let total_credit: f64 = input.entries.iter().map(|e| e.credit_amount).sum();
    if (total_debit - total_credit).abs() > 0.001 {
        return Err(format!("借贷不平衡：借方 {:.2} ≠ 贷方 {:.2}", total_debit, total_credit));
    }
    
    // 自动生成凭证号
    let count: i64 = db.query_row(
        "SELECT COUNT(*) FROM vouchers WHERE date = ?1",
        rusqlite::params![input.date],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    let voucher_no = format!("{}-{:04}", input.date.replace("-", ""), count + 1);
    
    // 插入凭证主表
    db.execute(
        "INSERT INTO vouchers (voucher_no, date, summary, attachment) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![voucher_no, input.date, input.summary, input.attachment],
    ).map_err(|e| e.to_string())?;
    let voucher_id = db.last_insert_rowid();
    
    // 插入凭证分录
    for entry in &input.entries {
        db.execute(
            "INSERT INTO voucher_entries (voucher_id, account_id, debit_amount, credit_amount, summary) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![voucher_id, entry.account_id, entry.debit_amount, entry.credit_amount, entry.summary],
        ).map_err(|e| e.to_string())?;
    }
    
    Ok(voucher_no)
}

#[tauri::command]
pub fn get_vouchers(state: State<AppState>, start_date: String, end_date: String, account_id: Option<i64>) -> Result<Vec<VoucherDetail>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let sql = if let Some(acc_id) = account_id {
        format!(
            "SELECT DISTINCT v.id, v.voucher_no, v.date, v.summary, v.attachment, v.created_at
             FROM vouchers v
             JOIN voucher_entries ve ON v.id = ve.voucher_id
             WHERE v.date BETWEEN '{}' AND '{}' AND ve.account_id = {}
             ORDER BY v.date DESC, v.id DESC",
            start_date, end_date, acc_id
        )
    } else {
        format!(
            "SELECT v.id, v.voucher_no, v.date, v.summary, v.attachment, v.created_at
             FROM vouchers v
             WHERE v.date BETWEEN '{}' AND '{}'
             ORDER BY v.date DESC, v.id DESC",
            start_date, end_date
        )
    };
    
    let mut stmt = db.prepare(&sql).map_err(|e| e.to_string())?;
    let voucher_ids: Vec<i64> = stmt.query_map([], |row| {
        Ok(row.get::<_, i64>(0)?)
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    let mut vouchers = Vec::new();
    for vid in voucher_ids {
        let detail = get_voucher_detail(&db, vid)?;
        vouchers.push(detail);
    }
    
    Ok(vouchers)
}

fn get_voucher_detail(db: &rusqlite::Connection, voucher_id: i64) -> Result<VoucherDetail, String> {
    let mut stmt = db.prepare(
        "SELECT id, voucher_no, date, summary, attachment, created_at FROM vouchers WHERE id = ?1"
    ).map_err(|e| e.to_string())?;
    
    let voucher = stmt.query_row(rusqlite::params![voucher_id], |row| {
        Ok(VoucherDetail {
            id: row.get(0)?,
            voucher_no: row.get(1)?,
            date: row.get(2)?,
            summary: row.get(3)?,
            entries: Vec::new(),
            attachment: row.get(4)?,
            created_at: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut entry_stmt = db.prepare(
        "SELECT ve.account_id, a.code, a.name, ve.debit_amount, ve.credit_amount, COALESCE(ve.summary, '')
         FROM voucher_entries ve
         JOIN accounts a ON ve.account_id = a.id
         WHERE ve.voucher_id = ?1
         ORDER BY ve.id"
    ).map_err(|e| e.to_string())?;
    
    let entries = entry_stmt.query_map(rusqlite::params![voucher_id], |row| {
        Ok(VoucherEntryDetail {
            account_id: row.get(0)?,
            account_code: row.get(1)?,
            account_name: row.get(2)?,
            debit_amount: row.get(3)?,
            credit_amount: row.get(4)?,
            summary: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(VoucherDetail { entries, ..voucher })
}

#[tauri::command]
pub fn delete_voucher(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "DELETE FROM voucher_entries WHERE voucher_id = ?1",
        rusqlite::params![id],
    ).map_err(|e| e.to_string())?;
    db.execute(
        "DELETE FROM vouchers WHERE id = ?1",
        rusqlite::params![id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 试算平衡 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct TrialBalanceItem {
    pub account_id: i64,
    pub account_code: String,
    pub account_name: String,
    pub debit_total: f64,
    pub credit_total: f64,
    pub balance: f64,
}

#[tauri::command]
pub fn trial_balance(state: State<AppState>, end_date: String) -> Result<Vec<TrialBalanceItem>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = db.prepare(
        "SELECT a.id, a.code, a.name,
                COALESCE(SUM(CASE WHEN ve.debit_amount > 0 THEN ve.debit_amount ELSE 0 END), 0) as debit_total,
                COALESCE(SUM(CASE WHEN ve.credit_amount > 0 THEN ve.credit_amount ELSE 0 END), 0) as credit_total
         FROM accounts a
         LEFT JOIN voucher_entries ve ON a.id = ve.account_id
         LEFT JOIN vouchers v ON ve.voucher_id = v.id AND v.date <= ?1
         WHERE a.is_active = 1
         GROUP BY a.id, a.code, a.name
         ORDER BY a.code"
    ).map_err(|e| e.to_string())?;
    
    let items = stmt.query_map(rusqlite::params![end_date], |row| {
        let debit_total: f64 = row.get(3)?;
        let credit_total: f64 = row.get(4)?;
        Ok(TrialBalanceItem {
            account_id: row.get(0)?,
            account_code: row.get(1)?,
            account_name: row.get(2)?,
            debit_total,
            credit_total,
            balance: (debit_total - credit_total).abs(),
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(items)
}

// ==================== 银行流水导入 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct BankTransaction {
    pub date: String,
    pub summary: String,
    pub amount: f64,
    pub balance: f64,
}

#[tauri::command]
pub fn import_bank_statement(state: State<AppState>, transactions: Vec<BankTransaction>, bank_account_id: i64) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut count = 0;
    
    for tx in &transactions {
        // 自动生成凭证：银行存款（借方/贷方）对应科目
        let (debit_id, credit_id) = if tx.amount > 0.0 {
            // 收入：借银行存款，贷待定（使用过渡科目）
            (bank_account_id, 3) // 假设 3 为过渡科目
        } else {
            // 支出：贷银行存款，借待定
            (3, bank_account_id)
        };
        
        let voucher_no = format!("BANK-{}-{:04}", tx.date.replace("-", ""), count + 1);
        
        db.execute(
            "INSERT INTO vouchers (voucher_no, date, summary) VALUES (?1, ?2, ?3)",
            rusqlite::params![voucher_no, tx.date, tx.summary],
        ).map_err(|e| e.to_string())?;
        let voucher_id = db.last_insert_rowid();
        
        let abs_amount = tx.amount.abs();
        db.execute(
            "INSERT INTO voucher_entries (voucher_id, account_id, debit_amount, credit_amount) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![voucher_id, debit_id, if tx.amount > 0.0 { abs_amount } else { 0.0 }, if tx.amount < 0.0 { abs_amount } else { 0.0 }],
        ).map_err(|e| e.to_string())?;
        db.execute(
            "INSERT INTO voucher_entries (voucher_id, account_id, debit_amount, credit_amount) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![voucher_id, credit_id, if tx.amount < 0.0 { abs_amount } else { 0.0 }, if tx.amount > 0.0 { abs_amount } else { 0.0 }],
        ).map_err(|e| e.to_string())?;
        
        count += 1;
    }
    
    Ok(count)
}
