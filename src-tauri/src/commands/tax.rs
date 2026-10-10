use crate::AppState;
use crate::services::tax_engine;
use crate::services::fs_report;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct TaxReportInput {
    pub report_type: String,
    pub period: String,
    pub period_type: Option<String>, // monthly / quarterly / semiannual / annual
}

fn parse_period_range(period: &str, period_type: &str) -> Result<(String, String), String> {
    match period_type {
        "monthly" => {
            let (y, m) = period.split_once('-').ok_or("格式: YYYY-MM")?;
            let start = format!("{}-{}-01", y, m);
            let end = format!("{}-{}-{}", y, m, days_in_month(y, m)?);
            Ok((start, end))
        }
        "quarterly" => {
            let (y, q) = period.split_once('-').ok_or("格式: YYYY-Qn")?;
            let qn: i32 = q.trim_start_matches('Q').parse().map_err(|_| "季度格式错误")?;
            let (sm, em) = match qn {
                1 => ("01", "03"),
                2 => ("04", "06"),
                3 => ("07", "09"),
                4 => ("10", "12"),
                _ => return Err("季度范围 1-4".to_string()),
            };
            let start = format!("{}-{}-01", y, sm);
            let end = format!("{}-{}-{}", y, em, days_in_month(y, em)?);
            Ok((start, end))
        }
        "semiannual" => {
            let (y, h) = period.split_once('-').ok_or("格式: YYYY-Hn")?;
            let half: i32 = h.trim_start_matches('H').parse().map_err(|_| "半年格式错误")?;
            let (sm, em) = if half == 1 { ("01", "06") } else { ("07", "12") };
            let start = format!("{}-{}-01", y, sm);
            let end = format!("{}-{}-{}", y, em, days_in_month(y, em)?);
            Ok((start, end))
        }
        "annual" => {
            let start = format!("{}-01-01", period);
            let end = format!("{}-12-31", period);
            Ok((start, end))
        }
        _ => Err("无效的期间类型".to_string()),
    }
}

fn days_in_month(year: &str, month: &str) -> Result<u32, String> {
    let m: u32 = month.parse().map_err(|_| "月份格式错误")?;
    let y: i32 = year.parse().map_err(|_| "年份格式错误")?;
    Ok(match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 29 } else { 28 },
        _ => return Err("月份范围 1-12".to_string()),
    })
}

#[tauri::command]
pub fn generate_tax_report(state: State<AppState>, input: TaxReportInput) -> Result<serde_json::Value, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let (data, tax_amount) = match input.report_type.as_str() {
        "vat_monthly" => {
            let report = tax_engine::generate_vat_report(&db, &input.period)?;
            let data = serde_json::to_value(&report).map_err(|e| e.to_string())?;
            (data, report.total_tax_due)
        },
        "cit_quarterly" => {
            let report = tax_engine::generate_cit_report(&db, &input.period)?;
            let data = serde_json::to_value(&report).map_err(|e| e.to_string())?;
            (data, report.tax_due)
        },
        "personal_income" => {
            let report = tax_engine::generate_personal_income_tax_report(&db, &input.period)?;
            let data = serde_json::to_value(&report).map_err(|e| e.to_string())?;
            (data, report.total_tax)
        },
        "balance_sheet" => {
            let pt = input.period_type.as_deref().unwrap_or("monthly");
            let (_, end_date) = parse_period_range(&input.period, pt)?;
            let report = tax_engine::generate_balance_sheet(&db, &end_date)?;
            let data = serde_json::to_value(&report).map_err(|e| e.to_string())?;
            (data, 0.0)
        },
        "income_statement" => {
            let pt = input.period_type.as_deref().unwrap_or("monthly");
            let (start_date, end_date) = parse_period_range(&input.period, pt)?;
            let report = tax_engine::generate_income_statement(&db, &start_date, &end_date)?;
            let data = serde_json::to_value(&report).map_err(|e| e.to_string())?;
            (data, report.income_tax)
        },
        "cash_flow" => {
            let pt = input.period_type.as_deref().unwrap_or("monthly");
            let (start_date, end_date) = parse_period_range(&input.period, pt)?;
            let report = tax_engine::generate_cash_flow_statement(&db, &start_date, &end_date)?;
            let data = serde_json::to_value(&report).map_err(|e| e.to_string())?;
            (data, 0.0)
        },
        _ => return Err("Unsupported report type".to_string()),
    };

    // 保存到数据库（保留已申报状态）
    db.execute(
        "INSERT INTO tax_reports (report_type, period, data, tax_amount, status) VALUES (?1, ?2, ?3, ?4, 'draft')
         ON CONFLICT(report_type, period) DO UPDATE SET
            data = ?3,
            tax_amount = ?4,
            updated_at = datetime('now', 'localtime')",
        rusqlite::params![input.report_type, input.period, data.to_string(), tax_amount],
    ).map_err(|e| e.to_string())?;

    Ok(data)
}

#[tauri::command]
pub fn get_saved_reports(state: State<AppState>, report_type: Option<String>) -> Result<Vec<serde_json::Value>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let sql = if let Some(ref rtype) = report_type {
        format!(
            "SELECT id, report_type, period, data, tax_amount, status, created_at, updated_at 
             FROM tax_reports WHERE report_type = '{}' ORDER BY period DESC LIMIT 12",
            rtype
        )
    } else {
        "SELECT id, report_type, period, data, tax_amount, status, created_at, updated_at 
         FROM tax_reports ORDER BY period DESC LIMIT 50".to_string()
    };
    
    let mut stmt = db.prepare(&sql).map_err(|e| e.to_string())?;
    let reports = stmt.query_map([], |row| {
        let data_str: String = row.get(3)?;
        let data: serde_json::Value = serde_json::from_str(&data_str).unwrap_or(serde_json::Value::Null);
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "report_type": row.get::<_, String>(1)?,
            "period": row.get::<_, String>(2)?,
            "data": data,
            "tax_amount": row.get::<_, f64>(4)?,
            "status": row.get::<_, String>(5)?,
            "created_at": row.get::<_, String>(6)?,
            "updated_at": row.get::<_, String>(7)?,
        }))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(reports)
}

#[tauri::command]
pub fn export_report(state: State<AppState>, report_type: String, period: String, format: String, period_type: Option<String>, path: Option<String>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let path = path
        .filter(|p| !p.trim().is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join(format!("{}_{}.{}", report_type, period, format)));
    let pt = period_type.as_deref().unwrap_or("monthly");
    
    match format.as_str() {
        "xlsx" => {
            match report_type.as_str() {
                "vat_monthly" => {
                    let report = tax_engine::generate_vat_report(&db, &period)?;
                    crate::services::export_service::export_vat_report_to_excel(&report, &path)?;
                },
                "cit_quarterly" => {
                    let report = tax_engine::generate_cit_report(&db, &period)?;
                    crate::services::export_service::export_cit_report_to_excel(&report, &path)?;
                },
                "income_statement" => {
                    let (start_date, end_date) = parse_period_range(&period, pt)?;
                    let report = tax_engine::generate_income_statement(&db, &start_date, &end_date)?;
                    crate::services::export_service::export_income_statement_to_excel(&report, &path)?;
                },
                "balance_sheet" => {
                    let (_, end_date) = parse_period_range(&period, pt)?;
                    let report = tax_engine::generate_balance_sheet(&db, &end_date)?;
                    crate::services::export_service::export_balance_sheet_to_excel(&report, &path)?;
                },
                _ => return Err("Unsupported export type".to_string()),
            }
            Ok(path.to_string_lossy().to_string())
        },
        _ => Err("Unsupported format, only xlsx is supported".to_string()),
    }
}

fn period_label(period: &str, period_type: &str) -> String {
    match period_type {
        "quarterly" => match period.split_once('-') {
            Some((y, q)) => format!("{}年第{}季度", y, q.trim_start_matches('Q')),
            None => period.to_string(),
        },
        "semiannual" => match period.split_once('-') {
            Some((y, h)) => format!("{}年第{}半年度", y, h.trim_start_matches('H')),
            None => period.to_string(),
        },
        "annual" => format!("{}年度", period),
        _ => match period.split_once('-') {
            Some((y, m)) => format!("{}年{}月", y, m.trim_start_matches('0')),
            None => period.to_string(),
        },
    }
}

/// 将 YYYY-MM-DD 平移若干年
fn shift_year(date: &str, years: i32) -> String {
    match date.split_once('-') {
        Some((y, rest)) => {
            let y2 = y.parse::<i32>().unwrap_or(0) + years;
            format!("{:04}-{}", y2, rest)
        }
        None => date.to_string(),
    }
}

/// 一键导出标准全量财报（电子税务局模板：资产负债表/利润表/现金流量表/所有者权益变动表/附注）
#[tauri::command]
pub fn export_full_statement(
    state: State<AppState>,
    period: String,
    period_type: Option<String>,
    path: Option<String>,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let pt = period_type.as_deref().unwrap_or("quarterly");
    let (start, end) = parse_period_range(&period, pt)?;
    let prev_start = shift_year(&start, -1);
    let prev_end = shift_year(&end, -1);

    let get_setting = |key: &str| -> String {
        db.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            rusqlite::params![key],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_default()
    };
    let company = get_setting("company_name");
    let tax_id = get_setting("company_tax_id");
    let label = period_label(&period, pt);

    let report = fs_report::build(&db, &company, &tax_id, &label, &start, &end, &prev_start, &prev_end)?;

    let path = path
        .filter(|p| !p.trim().is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::temp_dir().join(format!("{}_{}.xlsx", label, end))
        });
    fs_report::write_excel(&report, &path)?;
    Ok(path.to_string_lossy().to_string())
}
