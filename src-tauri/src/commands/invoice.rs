use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

// ==================== 发票录入 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceInput {
    pub invoice_type: String,  // input / output
    pub invoice_no: String,
    pub date: String,
    pub buyer_name: String,
    pub buyer_tax_id: String,
    pub amount: f64,
    pub tax_rate: f64,
}

#[tauri::command]
pub fn add_invoice(state: State<AppState>, input: InvoiceInput) -> Result<i64, String> {
    let tax_amount = input.amount * input.tax_rate;
    let total_amount = input.amount + tax_amount;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    db.execute(
        "INSERT INTO invoices (invoice_type, invoice_no, date, buyer_name, buyer_tax_id, amount, tax_rate, tax_amount, total_amount) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            input.invoice_type, input.invoice_no, input.date, 
            input.buyer_name, input.buyer_tax_id, 
            input.amount, input.tax_rate, tax_amount, total_amount
        ],
    ).map_err(|e| e.to_string())?;
    
    Ok(db.last_insert_rowid())
}

// ==================== 发票查询 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceQuery {
    pub invoice_type: Option<String>,  // input / output / None(全部)
    pub start_date: String,
    pub end_date: String,
    pub keyword: Option<String>,       // 搜索发票号码或购买方名称
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvoiceRecord {
    pub id: i64,
    pub invoice_type: String,
    pub invoice_no: String,
    pub date: String,
    pub buyer_name: String,
    pub buyer_tax_id: String,
    pub amount: f64,
    pub tax_rate: f64,
    pub tax_amount: f64,
    pub total_amount: f64,
    pub is_verified: bool,
    pub created_at: String,
}

#[tauri::command]
pub fn query_invoices(state: State<AppState>, query: InvoiceQuery) -> Result<Vec<InvoiceRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let mut sql = String::from(
        "SELECT id, invoice_type, invoice_no, date, buyer_name, buyer_tax_id, amount, tax_rate, tax_amount, total_amount, is_verified, created_at 
         FROM invoices WHERE date BETWEEN ?1 AND ?2"
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    params.push(Box::new(query.start_date));
    params.push(Box::new(query.end_date));
    
    if let Some(ref inv_type) = query.invoice_type {
        sql.push_str(" AND invoice_type = ?3");
        params.push(Box::new(inv_type.clone()));
    }
    
    if let Some(ref keyword) = query.keyword {
        if query.invoice_type.is_some() {
            sql.push_str(" AND (invoice_no LIKE ?4 OR buyer_name LIKE ?4)");
        } else {
            sql.push_str(" AND (invoice_no LIKE ?3 OR buyer_name LIKE ?3)");
        }
        params.push(Box::new(format!("%{}%", keyword)));
    }
    
    sql.push_str(" ORDER BY date DESC, id DESC");
    
    let mut stmt = db.prepare(&sql).map_err(|e| e.to_string())?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    
    let invoices = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(InvoiceRecord {
            id: row.get(0)?,
            invoice_type: row.get(1)?,
            invoice_no: row.get(2)?,
            date: row.get(3)?,
            buyer_name: row.get(4)?,
            buyer_tax_id: row.get(5)?,
            amount: row.get(6)?,
            tax_rate: row.get(7)?,
            tax_amount: row.get(8)?,
            total_amount: row.get(9)?,
            is_verified: row.get::<_, i32>(10)? != 0,
            created_at: row.get(11)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(invoices)
}

// ==================== 发票统计 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceSummary {
    pub total_count: i64,
    pub total_amount: f64,
    pub total_tax: f64,
    pub total_with_tax: f64,
    pub by_rate: Vec<TaxRateSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaxRateSummary {
    pub tax_rate: f64,
    pub count: i64,
    pub amount: f64,
    pub tax: f64,
}

#[tauri::command]
pub fn get_invoice_summary(state: State<AppState>, invoice_type: String, start_date: String, end_date: String) -> Result<InvoiceSummary, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    // 总统计
    let (total_count, total_amount, total_tax, total_with_tax): (i64, f64, f64, f64) = db.query_row(
        "SELECT COUNT(*), COALESCE(SUM(amount), 0), COALESCE(SUM(tax_amount), 0), COALESCE(SUM(total_amount), 0) 
         FROM invoices WHERE invoice_type = ?1 AND date BETWEEN ?2 AND ?3",
        rusqlite::params![invoice_type, start_date, end_date],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).map_err(|e| e.to_string())?;
    
    // 按税率分组统计
    let mut stmt = db.prepare(
        "SELECT tax_rate, COUNT(*), COALESCE(SUM(amount), 0), COALESCE(SUM(tax_amount), 0) 
         FROM invoices WHERE invoice_type = ?1 AND date BETWEEN ?2 AND ?3 
         GROUP BY tax_rate ORDER BY tax_rate DESC"
    ).map_err(|e| e.to_string())?;
    
    let by_rate: Vec<TaxRateSummary> = stmt.query_map(
        rusqlite::params![invoice_type, start_date, end_date],
        |row| {
            Ok(TaxRateSummary {
                tax_rate: row.get(0)?,
                count: row.get(1)?,
                amount: row.get(2)?,
                tax: row.get(3)?,
            })
        },
    ).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(InvoiceSummary {
        total_count,
        total_amount,
        total_tax,
        total_with_tax,
        by_rate,
    })
}

// ==================== 发票认证管理 ====================

#[tauri::command]
pub fn verify_invoice(state: State<AppState>, id: i64, verified: bool) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "UPDATE invoices SET is_verified = ?1 WHERE id = ?2",
        rusqlite::params![verified as i32, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_invoice(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM invoices WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 发票到期提醒 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceDueReminder {
    pub id: i64,
    pub invoice_no: String,
    pub date: String,
    pub buyer_name: String,
    pub amount: f64,
    pub due_days: i64,  // 负数表示已逾期
}

#[tauri::command]
pub fn get_invoice_due_reminders(state: State<AppState>, days_ahead: i64) -> Result<Vec<InvoiceDueReminder>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    // 进项发票认证有效期 360 天，这里简化检查 90 天内到期的
    let mut stmt = db.prepare(
        "SELECT id, invoice_no, date, buyer_name, amount 
         FROM invoices 
         WHERE invoice_type = 'input' AND is_verified = 0
         AND date >= date('now', ?1) 
         ORDER BY date ASC"
    ).map_err(|e| e.to_string())?;
    
    let reminders = stmt.query_map(
        rusqlite::params![format!("-{} days", days_ahead)],
        |row| {
            let date: String = row.get(2)?;
            // 简单计算距离今天的天数（简化处理）
            let due_days = 90 - days_ahead; // 简化示例
            Ok(InvoiceDueReminder {
                id: row.get(0)?,
                invoice_no: row.get(1)?,
                date,
                buyer_name: row.get(3)?,
                amount: row.get(4)?,
                due_days,
            })
        },
    ).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(reminders)
}
