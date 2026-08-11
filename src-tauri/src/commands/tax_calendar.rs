use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use chrono::{NaiveDate, Datelike};

// ==================== 税务日历生成 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxCalendarItem {
    pub id: i64,
    pub tax_type: String,
    pub tax_type_label: String,
    pub period: String,
    pub period_label: String,
    pub due_date: String,
    pub days_remaining: i64,
    pub status: String,        // pending / submitted / overdue
    pub is_reminded: bool,
    pub tax_amount: f64,
}

/// 获取纳税人类型（从设置中读取）
fn get_taxpayer_type(db: &rusqlite::Connection) -> Result<String, String> {
    let result = db.query_row(
        "SELECT value FROM settings WHERE key = 'taxpayer_type'",
        [],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(v) => Ok(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok("一般纳税人".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// 计算各税种申报截止日期
fn calculate_due_dates(year: i32, month: u32, taxpayer_type: &str) -> Vec<(String, String, String)> {
    let mut items = Vec::new();
    let period = format!("{:04}-{:02}", year, month);
    let _period_label = format!("{}年{}月", year, month);
    
    // 增值税申报截止日：一般纳税人每月15日，小规模每季度15日
    let vat_due = if taxpayer_type == "一般纳税人" {
        NaiveDate::from_ymd_opt(year, month, 15).unwrap_or(NaiveDate::from_ymd_opt(year, month, 28).unwrap())
    } else {
        // 小规模纳税人按季度申报
        let quarter_month = match month {
            1..=3 => 4,
            4..=6 => 7,
            7..=9 => 10,
            10..=12 => 1,
            _ => month,
        };
        let q_year = if quarter_month == 1 && month >= 10 { year + 1 } else { year };
        NaiveDate::from_ymd_opt(q_year, quarter_month, 15).unwrap_or(NaiveDate::from_ymd_opt(q_year, quarter_month, 28).unwrap())
    };
    items.push(("vat".to_string(), period.clone(), vat_due.format("%Y-%m-%d").to_string()));
    
    // 附加税与增值税同步
    items.push(("surcharge".to_string(), period.clone(), vat_due.format("%Y-%m-%d").to_string()));
    
    // 个人所得税申报截止日：每月15日
    let pit_due = NaiveDate::from_ymd_opt(year, month, 15).unwrap_or(NaiveDate::from_ymd_opt(year, month, 28).unwrap());
    items.push(("personal_income".to_string(), period.clone(), pit_due.format("%Y-%m-%d").to_string()));
    
    // 企业所得税季报截止日：季度后15日
    let quarter_end = match month {
        3 => NaiveDate::from_ymd_opt(year, 4, 15),
        6 => NaiveDate::from_ymd_opt(year, 7, 15),
        9 => NaiveDate::from_ymd_opt(year, 10, 15),
        12 => NaiveDate::from_ymd_opt(year + 1, 1, 15),
        _ => None,
    };
    if let Some(cit_due) = quarter_end {
        let quarter_period = format!("{:04}-Q{}", year, month.div_ceil(3));
        items.push(("cit".to_string(), quarter_period, cit_due.format("%Y-%m-%d").to_string()));
    }
    
    items
}

/// 获取税种标签
fn get_tax_type_label(tax_type: &str) -> &str {
    match tax_type {
        "vat" => "增值税",
        "cit" => "企业所得税",
        "surcharge" => "附加税",
        "personal_income" => "个人所得税",
        _ => tax_type,
    }
}

/// 税种编码映射（tax_calendar → tax_reports）
fn map_report_type(tax_type: &str) -> &str {
    match tax_type {
        "vat" => "vat_monthly",
        "cit" => "cit_quarterly",
        "surcharge" => "surcharge",
        "personal_income" => "personal_income",
        _ => tax_type,
    }
}

/// 获取期间标签
fn get_period_label(tax_type: &str, period: &str) -> String {
    match tax_type {
        "vat" | "surcharge" | "personal_income" => {
            if let Some((y, m)) = period.split_once('-') {
                format!("{}年{}月", y, m)
            } else {
                period.to_string()
            }
        },
        "cit" => {
            if let Some((y, q)) = period.split_once('-') {
                format!("{}年{}", y, q.replace("Q", "第").replace("1", "一季度").replace("2", "二季度").replace("3", "三季度").replace("4", "四季度"))
            } else {
                period.to_string()
            }
        },
        _ => period.to_string(),
    }
}

#[tauri::command]
pub fn generate_tax_calendar(state: State<AppState>, year: i32) -> Result<Vec<TaxCalendarItem>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let taxpayer_type = get_taxpayer_type(&db)?;
    let today = chrono::Local::now().naive_local().date();
    
    let mut all_items = Vec::new();
    
    // 生成全年日历
    for month in 1..=12 {
        let due_items = calculate_due_dates(year, month, &taxpayer_type);
        for (tax_type, period, due_date_str) in due_items {
            let due_date = NaiveDate::parse_from_str(&due_date_str, "%Y-%m-%d").unwrap_or(today);
            let days_remaining = (due_date - today).num_days();
            
            // 检查是否已申报
            let mapped_type = map_report_type(&tax_type);
            let is_submitted: bool = db.query_row(
                "SELECT COUNT(*) > 0 FROM tax_reports WHERE report_type = ?1 AND period = ?2 AND status = 'submitted'",
                rusqlite::params![mapped_type, period],
                |row| row.get(0),
            ).unwrap_or(false);
            
            let status = if is_submitted {
                "submitted"
            } else if days_remaining < 0 {
                "overdue"
            } else {
                "pending"
            };
            
            // 检查是否已提醒
            let is_reminded: bool = db.query_row(
                "SELECT COUNT(*) > 0 FROM tax_calendar WHERE tax_type = ?1 AND period = ?2 AND is_reminded = 1",
                rusqlite::params![tax_type, period],
                |row| row.get(0),
            ).unwrap_or(false);
            
            // 获取税额
            let tax_amount: f64 = db.query_row(
                "SELECT COALESCE(tax_amount, 0) FROM tax_reports WHERE report_type = ?1 AND period = ?2",
                rusqlite::params![mapped_type, period],
                |row| row.get(0),
            ).unwrap_or(0.0);
            
            all_items.push(TaxCalendarItem {
                id: 0,
                tax_type: tax_type.clone(),
                tax_type_label: get_tax_type_label(&tax_type).to_string(),
                period: period.clone(),
                period_label: get_period_label(&tax_type, &period),
                due_date: due_date_str,
                days_remaining,
                status: status.to_string(),
                is_reminded,
                tax_amount,
            });
        }
    }
    
    // 保存到数据库
    for item in &all_items {
        db.execute(
            "INSERT OR IGNORE INTO tax_calendar (tax_type, period, due_date, is_reminded) VALUES (?1, ?2, ?3, 0)",
            rusqlite::params![item.tax_type, item.period, item.due_date],
        ).ok();
    }
    
    // 排序：按截止日期升序
    let mut sorted = all_items;
    sorted.sort_by(|a, b| a.due_date.cmp(&b.due_date));
    
    Ok(sorted)
}

#[tauri::command]
pub fn get_tax_calendar(state: State<AppState>, year: Option<i32>) -> Result<Vec<TaxCalendarItem>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let today = chrono::Local::now().naive_local().date();
    let target_year = year.unwrap_or_else(|| today.year());
    
    let mut stmt = db.prepare(
        "SELECT tc.id, tc.tax_type, tc.period, tc.due_date, tc.is_reminded,
                COALESCE(tr.tax_amount, 0) as tax_amount,
                CASE WHEN tr.status = 'submitted' THEN 'submitted' 
                     WHEN date(tc.due_date) < date('now') THEN 'overdue' 
                     ELSE 'pending' END as status
         FROM tax_calendar tc
         LEFT JOIN tax_reports tr ON 
            CASE tc.tax_type
                WHEN 'vat' THEN 'vat_monthly'
                WHEN 'cit' THEN 'cit_quarterly'
                WHEN 'surcharge' THEN 'surcharge'
                WHEN 'personal_income' THEN 'personal_income'
                ELSE tc.tax_type
            END = tr.report_type AND tc.period = tr.period
         WHERE tc.due_date LIKE ?1
         ORDER BY tc.due_date ASC"
    ).map_err(|e| e.to_string())?;
    
    let pattern = format!("{}-%", target_year);
    let items = stmt.query_map(rusqlite::params![pattern], |row| {
        let tax_type: String = row.get(1)?;
        let period: String = row.get(2)?;
        let due_date_str: String = row.get(3)?;
        let due_date = NaiveDate::parse_from_str(&due_date_str, "%Y-%m-%d").unwrap_or(today);
        let days_remaining = (due_date - today).num_days();
        
        Ok(TaxCalendarItem {
            id: row.get(0)?,
            tax_type: tax_type.clone(),
            tax_type_label: get_tax_type_label(&tax_type).to_string(),
            period: period.clone(),
            period_label: get_period_label(&tax_type, &period),
            due_date: due_date_str,
            days_remaining,
            status: row.get(6)?,
            is_reminded: row.get::<_, i32>(4)? != 0,
            tax_amount: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(items)
}

// ==================== 提醒管理 ====================

#[tauri::command]
pub fn mark_as_reminded(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "UPDATE tax_calendar SET is_reminded = 1 WHERE id = ?1",
        rusqlite::params![id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn mark_as_submitted(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // 获取税种和期间信息
    let (tax_type, period): (String, String) = db.query_row(
        "SELECT tax_type, period FROM tax_calendar WHERE id = ?1",
        rusqlite::params![id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|e| e.to_string())?;

    // 更新 tax_reports（税种编码映射：vat→vat_monthly, cit→cit_quarterly 等）
    let report_type = map_report_type(&tax_type);

    let exists: i64 = db.query_row(
        "SELECT COUNT(*) FROM tax_reports WHERE report_type = ?1 AND period = ?2",
        rusqlite::params![report_type, period],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    if exists > 0 {
        db.execute(
            "UPDATE tax_reports SET status = 'submitted', updated_at = datetime('now', 'localtime') WHERE report_type = ?1 AND period = ?2",
            rusqlite::params![report_type, period],
        ).map_err(|e| e.to_string())?;
    } else {
        db.execute(
            "INSERT INTO tax_reports (report_type, period, data, tax_amount, status) VALUES (?1, ?2, '{}', 0, 'submitted')",
            rusqlite::params![report_type, period],
        ).map_err(|e| e.to_string())?;
    }

    Ok(())
}

// ==================== 提醒检查 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct ReminderNotification {
    pub items: Vec<TaxCalendarItem>,
    pub urgent_count: i64,     // 7天内到期
    pub overdue_count: i64,    // 已逾期
}

#[tauri::command]
pub fn check_upcoming_deadlines(state: State<AppState>) -> Result<ReminderNotification, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let today = chrono::Local::now().naive_local().date();
    
    // 获取7天内到期的未申报项目
    let mut stmt = db.prepare(
        "SELECT tc.id, tc.tax_type, tc.period, tc.due_date, tc.is_reminded,
                COALESCE(tr.tax_amount, 0) as tax_amount,
                CASE WHEN tr.status = 'submitted' THEN 'submitted'
                     WHEN date(tc.due_date) < date('now') THEN 'overdue'
                     ELSE 'pending' END as status
         FROM tax_calendar tc
         LEFT JOIN tax_reports tr ON 
            CASE tc.tax_type
                WHEN 'vat' THEN 'vat_monthly'
                WHEN 'cit' THEN 'cit_quarterly'
                WHEN 'surcharge' THEN 'surcharge'
                WHEN 'personal_income' THEN 'personal_income'
                ELSE tc.tax_type
            END = tr.report_type AND tc.period = tr.period
         WHERE date(tc.due_date) BETWEEN date('now') AND date('now', '+7 days')
            OR date(tc.due_date) < date('now')
         ORDER BY tc.due_date ASC"
    ).map_err(|e| e.to_string())?;
    
    let mut items = Vec::new();
    let mut urgent_count = 0;
    let mut overdue_count = 0;
    
    let rows = stmt.query_map([], |row| {
        let tax_type: String = row.get(1)?;
        let period: String = row.get(2)?;
        let due_date_str: String = row.get(3)?;
        let due_date = NaiveDate::parse_from_str(&due_date_str, "%Y-%m-%d").unwrap_or(today);
        let days_remaining = (due_date - today).num_days();
        let status: String = row.get(6)?;
        
        Ok(TaxCalendarItem {
            id: row.get(0)?,
            tax_type: tax_type.clone(),
            tax_type_label: get_tax_type_label(&tax_type).to_string(),
            period: period.clone(),
            period_label: get_period_label(&tax_type, &period),
            due_date: due_date_str,
            days_remaining,
            status,
            is_reminded: row.get::<_, i32>(4)? != 0,
            tax_amount: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;
    
    for item in rows.flatten() {
        if item.status == "submitted" {
            continue;
        }
        if item.status == "overdue" {
            overdue_count += 1;
        } else if item.days_remaining >= 0 && item.days_remaining <= 7 {
            urgent_count += 1;
        }
        items.push(item);
    }
    
    Ok(ReminderNotification {
        items,
        urgent_count,
        overdue_count,
    })
}