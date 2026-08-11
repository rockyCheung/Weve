use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use chrono::{NaiveDate, Datelike};

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub employee_count: i64,
    pub active_employee_count: i64,
    pub monthly_income: f64,
    pub monthly_expense: f64,
    pub monthly_profit: f64,
    pub pending_approvals: i64,
    pub pending_tax_reports: i64,
    pub overdue_tax_items: i64,
    pub upcoming_deadlines: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyTrend {
    pub month: String,
    pub income: f64,
    pub expense: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentActivity {
    pub id: i64,
    pub activity_type: String,  // tax_due / invoice_due / approval
    pub title: String,
    pub description: String,
    pub date: String,
    pub urgency: String,        // normal / urgent / overdue
}

#[tauri::command]
pub fn get_dashboard_stats(state: State<AppState>) -> Result<DashboardStats, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let today = chrono::Local::now().naive_local().date();
    let current_month = format!("{:04}-{:02}", today.year(), today.month());
    let month_start = format!("{}-01", current_month);
    
    // 计算下个月第一天
    let next_month = if today.month() == 12 {
        format!("{}-01-01", today.year() + 1)
    } else {
        format!("{:04}-{:02}-01", today.year(), today.month() + 1)
    };

    // 员工统计
    let employee_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM employees",
        [],
        |row| row.get(0),
    ).unwrap_or(0);
    
    let active_employee_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM employees WHERE is_active = 1",
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    // 本月收入（从收入科目贷方汇总）
    let monthly_income: f64 = db.query_row(
        "SELECT COALESCE(SUM(ve.credit_amount), 0) 
         FROM voucher_entries ve
         JOIN accounts a ON ve.account_id = a.id
         JOIN vouchers v ON ve.voucher_id = v.id
         WHERE a.code LIKE '6%' AND v.date >= ?1 AND v.date < ?2",
        rusqlite::params![month_start, next_month],
        |row| row.get(0),
    ).unwrap_or(0.0);

    // 本月支出（从费用科目借方汇总）
    let monthly_expense: f64 = db.query_row(
        "SELECT COALESCE(SUM(ve.debit_amount), 0) 
         FROM voucher_entries ve
         JOIN accounts a ON ve.account_id = a.id
         JOIN vouchers v ON ve.voucher_id = v.id
         WHERE a.code LIKE '66%' AND v.date >= ?1 AND v.date < ?2",
        rusqlite::params![month_start, next_month],
        |row| row.get(0),
    ).unwrap_or(0.0);

    let monthly_profit = monthly_income - monthly_expense;

    // 待审批事项（简化：统计未处理的报销单）
    let pending_approvals: i64 = 0; // 待扩展

    // 待申报税种
    let pending_tax_reports: i64 = db.query_row(
        "SELECT COUNT(*) FROM tax_calendar tc
         LEFT JOIN tax_reports tr ON 
            CASE tc.tax_type
                WHEN 'vat' THEN 'vat_monthly'
                WHEN 'cit' THEN 'cit_quarterly'
                WHEN 'surcharge' THEN 'surcharge'
                WHEN 'personal_income' THEN 'personal_income'
                ELSE tc.tax_type
            END = tr.report_type AND tc.period = tr.period
         WHERE (tr.status IS NULL OR tr.status != 'submitted')
         AND date(tc.due_date) >= date('now')",
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    // 已逾期税种
    let overdue_tax_items: i64 = db.query_row(
        "SELECT COUNT(*) FROM tax_calendar tc
         LEFT JOIN tax_reports tr ON 
            CASE tc.tax_type
                WHEN 'vat' THEN 'vat_monthly'
                WHEN 'cit' THEN 'cit_quarterly'
                WHEN 'surcharge' THEN 'surcharge'
                WHEN 'personal_income' THEN 'personal_income'
                ELSE tc.tax_type
            END = tr.report_type AND tc.period = tr.period
         WHERE (tr.status IS NULL OR tr.status != 'submitted')
         AND date(tc.due_date) < date('now')",
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    // 即将到期（7天内）
    let upcoming_deadlines: i64 = db.query_row(
        "SELECT COUNT(*) FROM tax_calendar tc
         LEFT JOIN tax_reports tr ON 
            CASE tc.tax_type
                WHEN 'vat' THEN 'vat_monthly'
                WHEN 'cit' THEN 'cit_quarterly'
                WHEN 'surcharge' THEN 'surcharge'
                WHEN 'personal_income' THEN 'personal_income'
                ELSE tc.tax_type
            END = tr.report_type AND tc.period = tr.period
         WHERE (tr.status IS NULL OR tr.status != 'submitted')
         AND date(tc.due_date) BETWEEN date('now') AND date('now', '+7 days')",
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    Ok(DashboardStats {
        employee_count,
        active_employee_count,
        monthly_income,
        monthly_expense,
        monthly_profit,
        pending_approvals,
        pending_tax_reports,
        overdue_tax_items,
        upcoming_deadlines,
    })
}

#[tauri::command]
pub fn get_monthly_trend(state: State<AppState>, months: i32) -> Result<Vec<MonthlyTrend>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let today = chrono::Local::now().naive_local().date();
    let mut trends = Vec::new();

    for i in (0..months).rev() {
        let target_date = today - chrono::Duration::days(i as i64 * 30);
        let month = format!("{:04}-{:02}", target_date.year(), target_date.month());
        let month_start = format!("{}-01", month);
        
        let next_month = if target_date.month() == 12 {
            format!("{}-01-01", target_date.year() + 1)
        } else {
            format!("{:04}-{:02}-01", target_date.year(), target_date.month() + 1)
        };

        let income: f64 = db.query_row(
            "SELECT COALESCE(SUM(ve.credit_amount), 0) 
             FROM voucher_entries ve
             JOIN accounts a ON ve.account_id = a.id
             JOIN vouchers v ON ve.voucher_id = v.id
             WHERE a.code LIKE '6%' AND v.date >= ?1 AND v.date < ?2",
            rusqlite::params![month_start, next_month],
            |row| row.get(0),
        ).unwrap_or(0.0);

        let expense: f64 = db.query_row(
            "SELECT COALESCE(SUM(ve.debit_amount), 0) 
             FROM voucher_entries ve
             JOIN accounts a ON ve.account_id = a.id
             JOIN vouchers v ON ve.voucher_id = v.id
             WHERE a.code LIKE '66%' AND v.date >= ?1 AND v.date < ?2",
            rusqlite::params![month_start, next_month],
            |row| row.get(0),
        ).unwrap_or(0.0);

        trends.push(MonthlyTrend {
            month: format!("{}月", target_date.month()),
            income,
            expense,
        });
    }

    Ok(trends)
}

#[tauri::command]
pub fn get_recent_activities(state: State<AppState>) -> Result<Vec<RecentActivity>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let today = chrono::Local::now().naive_local().date();
    let mut activities = Vec::new();

    // 获取即将到期的税务项目
    let mut stmt = db.prepare(
        "SELECT tc.id, tc.tax_type, tc.period, tc.due_date
         FROM tax_calendar tc
         LEFT JOIN tax_reports tr ON 
            CASE tc.tax_type
                WHEN 'vat' THEN 'vat_monthly'
                WHEN 'cit' THEN 'cit_quarterly'
                WHEN 'surcharge' THEN 'surcharge'
                WHEN 'personal_income' THEN 'personal_income'
                ELSE tc.tax_type
            END = tr.report_type AND tc.period = tr.period
         WHERE (tr.status IS NULL OR tr.status != 'submitted')
         ORDER BY tc.due_date ASC
         LIMIT 10"
    ).map_err(|e| e.to_string())?;

    let tax_items = stmt.query_map([], |row| {
        let tax_type: String = row.get(1)?;
        let period: String = row.get(2)?;
        let due_date: String = row.get(3)?;
        Ok((tax_type, period, due_date))
    }).map_err(|e| e.to_string())?;

    for (tax_type, period, due_date) in tax_items.flatten() {
        let due = NaiveDate::parse_from_str(&due_date, "%Y-%m-%d").unwrap_or(today);
        let days_remaining = (due - today).num_days();
        
        let title = match tax_type.as_str() {
            "vat" => format!("增值税申报截止（{}）", period),
            "cit" => format!("企业所得税申报截止（{}）", period),
            "personal_income" => format!("个人所得税申报截止（{}）", period),
            _ => format!("{}申报截止（{}）", tax_type, period),
        };
        let urgency = if days_remaining < 0 { "overdue" } else if days_remaining <= 7 { "urgent" } else { "normal" };

        activities.push(RecentActivity {
            id: activities.len() as i64 + 1,
            activity_type: "tax_due".to_string(),
            title,
            description: format!("截止日期: {}, 剩余{}天", due_date, days_remaining.max(0)),
            date: due_date,
            urgency: urgency.to_string(),
        });
    }

    // 添加发票到期提醒
    let mut inv_stmt = db.prepare(
        "SELECT id, invoice_no, date, buyer_name
         FROM invoices 
         WHERE invoice_type = 'input' AND is_verified = 0
         AND date >= date('now', '-300 days')
         ORDER BY date ASC
         LIMIT 5"
    ).map_err(|e| e.to_string())?;

    let invoices = inv_stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let invoice_no: String = row.get(1)?;
        let date: String = row.get(2)?;
        let buyer: String = row.get(3)?;
        Ok((id, invoice_no, date, buyer))
    }).map_err(|e| e.to_string())?;

    for (id, invoice_no, date, buyer) in invoices.flatten() {
        let inv_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap_or(today);
        let days_to_expiry = 360 - (today - inv_date).num_days();
        
        if days_to_expiry <= 90 {
            activities.push(RecentActivity {
                id: 1000 + id,
                activity_type: "invoice_due".to_string(),
                title: format!("发票认证到期提醒 - {}", invoice_no),
                description: format!("销售方: {}, 剩余{}天认证期", buyer, days_to_expiry.max(0)),
                date: (inv_date + chrono::Duration::days(360)).format("%Y-%m-%d").to_string(),
                urgency: if days_to_expiry <= 30 { "overdue".to_string() } else if days_to_expiry <= 90 { "urgent".to_string() } else { "normal".to_string() },
            });
        }
    }

    // 按日期排序
    activities.sort_by(|a, b| a.date.cmp(&b.date));
    
    Ok(activities.into_iter().take(10).collect())
}