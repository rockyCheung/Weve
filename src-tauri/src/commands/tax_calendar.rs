use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use chrono::{NaiveDate, Datelike};
use std::collections::HashMap;

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
    pub is_custom: bool,
    pub custom_item_id: Option<i64>,
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
                is_custom: false,
                custom_item_id: None,
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
    let mut items: Vec<TaxCalendarItem> = stmt.query_map(rusqlite::params![pattern], |row| {
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
            is_custom: false,
            custom_item_id: None,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    // 合并自定义税务事项
    let mut custom_items = load_custom_calendar_items(&db, target_year)?;
    items.append(&mut custom_items);
    items.sort_by(|a, b| a.due_date.cmp(&b.due_date));

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
            is_custom: false,
            custom_item_id: None,
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

    // 合并自定义税务事项的提醒（当前年 + 下一年）
    for y in [today.year(), today.year() + 1] {
        for item in load_custom_calendar_items(&db, y)? {
            if item.status == "submitted" {
                continue;
            }
            if item.status == "overdue" {
                overdue_count += 1;
                items.push(item);
            } else if item.days_remaining >= 0 && item.days_remaining <= 7 {
                urgent_count += 1;
                items.push(item);
            }
        }
    }
    items.sort_by(|a, b| a.due_date.cmp(&b.due_date));

    Ok(ReminderNotification {
        items,
        urgent_count,
        overdue_count,
    })
}

// ==================== 自定义税务事项 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomTaxItem {
    pub id: i64,
    pub name: String,
    pub frequency: String,       // monthly / quarterly / annual / once
    pub due_month: Option<i64>,
    pub due_day: i64,
    pub once_date: Option<String>,
    pub remark: String,
    pub is_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomTaxItemInput {
    pub name: String,
    pub frequency: String,
    pub due_month: Option<i64>,
    pub due_day: i64,
    pub once_date: Option<String>,
    pub remark: String,
    pub is_enabled: bool,
}

fn frequency_label(frequency: &str) -> String {
    match frequency {
        "monthly" => "每月".to_string(),
        "quarterly" => "每季度".to_string(),
        "annual" => "每年".to_string(),
        "once" => "一次性".to_string(),
        _ => frequency.to_string(),
    }
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
            .and_then(|d| d.pred_opt())
            .map(|d| d.day())
            .unwrap_or(31)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
            .and_then(|d| d.pred_opt())
            .map(|d| d.day())
            .unwrap_or(31)
    }
}

/// 展开自定义事项为某一年度的具体截止日期，返回 (period, due_date)
fn expand_custom_item(item: &CustomTaxItem, year: i32) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let day = item.due_day.clamp(1, 31) as u32;

    match item.frequency.as_str() {
        "monthly" => {
            for m in 1..=12u32 {
                let d = day.min(last_day_of_month(year, m));
                if let Some(due) = NaiveDate::from_ymd_opt(year, m, d) {
                    result.push((format!("{:04}-{:02}", year, m), due.format("%Y-%m-%d").to_string()));
                }
            }
        }
        "quarterly" => {
            let start = item.due_month.unwrap_or(1).clamp(1, 12) as i32;
            for i in 0..4 {
                let m_raw = start + i * 3;
                let (y, m) = if m_raw > 12 { (year + 1, m_raw - 12) } else { (year, m_raw) };
                let d = day.min(last_day_of_month(y, m as u32));
                if let Some(due) = NaiveDate::from_ymd_opt(y, m as u32, d) {
                    result.push((format!("{:04}-{:02}", y, m), due.format("%Y-%m-%d").to_string()));
                }
            }
        }
        "annual" => {
            let m = item.due_month.unwrap_or(1).clamp(1, 12) as u32;
            let d = day.min(last_day_of_month(year, m));
            if let Some(due) = NaiveDate::from_ymd_opt(year, m, d) {
                result.push((format!("{:04}", year), due.format("%Y-%m-%d").to_string()));
            }
        }
        "once" => {
            if let Some(date_str) = &item.once_date {
                if let Ok(due) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    if due.year() == year {
                        result.push((date_str.clone(), due.format("%Y-%m-%d").to_string()));
                    }
                }
            }
        }
        _ => {}
    }
    result
}

fn build_custom_calendar_item(
    item: &CustomTaxItem,
    period: &str,
    due_date: &str,
    today: NaiveDate,
    submitted: bool,
) -> TaxCalendarItem {
    let due = NaiveDate::parse_from_str(due_date, "%Y-%m-%d").unwrap_or(today);
    let days_remaining = (due - today).num_days();
    let status = if submitted {
        "submitted"
    } else if days_remaining < 0 {
        "overdue"
    } else {
        "pending"
    };
    TaxCalendarItem {
        id: 0,
        tax_type: "custom".to_string(),
        tax_type_label: item.name.clone(),
        period: period.to_string(),
        period_label: frequency_label(&item.frequency),
        due_date: due_date.to_string(),
        days_remaining,
        status: status.to_string(),
        is_reminded: false,
        tax_amount: 0.0,
        is_custom: true,
        custom_item_id: Some(item.id),
    }
}

/// 加载某一年度所有启用的自定义事项并展开为日历条目
pub fn load_custom_calendar_items(db: &rusqlite::Connection, year: i32) -> Result<Vec<TaxCalendarItem>, String> {
    let today = chrono::Local::now().naive_local().date();

    let mut stmt = db.prepare(
        "SELECT id, name, frequency, due_month, due_day, once_date, remark, is_enabled
         FROM custom_tax_items WHERE is_enabled = 1 ORDER BY id ASC"
    ).map_err(|e| e.to_string())?;
    let rules: Vec<CustomTaxItem> = stmt.query_map([], |row| {
        Ok(CustomTaxItem {
            id: row.get(0)?,
            name: row.get(1)?,
            frequency: row.get(2)?,
            due_month: row.get(3)?,
            due_day: row.get(4)?,
            once_date: row.get(5)?,
            remark: row.get(6)?,
            is_enabled: row.get::<_, i64>(7)? != 0,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    let mut records: HashMap<(i64, String), bool> = HashMap::new();
    let mut rec_stmt = db.prepare(
        "SELECT item_id, period, submitted FROM custom_tax_item_records WHERE submitted = 1"
    ).map_err(|e| e.to_string())?;
    let rec_rows = rec_stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?))
    }).map_err(|e| e.to_string())?;
    for r in rec_rows.flatten() {
        records.insert((r.0, r.1), r.2 != 0);
    }

    let mut items = Vec::new();
    for rule in &rules {
        for (period, due_date) in expand_custom_item(rule, year) {
            let submitted = records.get(&(rule.id, period.clone())).copied().unwrap_or(false);
            items.push(build_custom_calendar_item(rule, &period, &due_date, today, submitted));
        }
    }

    Ok(items)
}

#[tauri::command]
pub fn get_custom_tax_items(state: State<AppState>) -> Result<Vec<CustomTaxItem>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT id, name, frequency, due_month, due_day, once_date, remark, is_enabled
         FROM custom_tax_items ORDER BY id ASC"
    ).map_err(|e| e.to_string())?;
    let items = stmt.query_map([], |row| {
        Ok(CustomTaxItem {
            id: row.get(0)?,
            name: row.get(1)?,
            frequency: row.get(2)?,
            due_month: row.get(3)?,
            due_day: row.get(4)?,
            once_date: row.get(5)?,
            remark: row.get(6)?,
            is_enabled: row.get::<_, i64>(7)? != 0,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(items)
}

#[tauri::command]
pub fn create_custom_tax_item(state: State<AppState>, input: CustomTaxItemInput) -> Result<CustomTaxItem, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO custom_tax_items (name, frequency, due_month, due_day, once_date, remark, is_enabled)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            input.name,
            input.frequency,
            input.due_month,
            input.due_day,
            input.once_date,
            input.remark,
            input.is_enabled as i64
        ],
    ).map_err(|e| e.to_string())?;
    let id = db.last_insert_rowid();
    Ok(CustomTaxItem {
        id,
        name: input.name,
        frequency: input.frequency,
        due_month: input.due_month,
        due_day: input.due_day,
        once_date: input.once_date,
        remark: input.remark,
        is_enabled: input.is_enabled,
    })
}

#[tauri::command]
pub fn update_custom_tax_item(state: State<AppState>, id: i64, input: CustomTaxItemInput) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "UPDATE custom_tax_items SET name = ?1, frequency = ?2, due_month = ?3, due_day = ?4,
         once_date = ?5, remark = ?6, is_enabled = ?7, updated_at = datetime('now', 'localtime')
         WHERE id = ?8",
        rusqlite::params![
            input.name,
            input.frequency,
            input.due_month,
            input.due_day,
            input.once_date,
            input.remark,
            input.is_enabled as i64,
            id
        ],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_custom_tax_item(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM custom_tax_items WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn mark_custom_item_submitted(state: State<AppState>, item_id: i64, period: String, submitted: bool) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let exists: i64 = db.query_row(
        "SELECT COUNT(*) FROM custom_tax_item_records WHERE item_id = ?1 AND period = ?2",
        rusqlite::params![item_id, period],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    if exists > 0 {
        db.execute(
            "UPDATE custom_tax_item_records SET submitted = ?1,
             submitted_at = CASE WHEN ?1 = 1 THEN datetime('now', 'localtime') ELSE NULL END
             WHERE item_id = ?2 AND period = ?3",
            rusqlite::params![submitted as i64, item_id, period],
        ).map_err(|e| e.to_string())?;
    } else if submitted {
        db.execute(
            "INSERT INTO custom_tax_item_records (item_id, period, submitted, submitted_at)
             VALUES (?1, ?2, 1, datetime('now', 'localtime'))",
            rusqlite::params![item_id, period],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}