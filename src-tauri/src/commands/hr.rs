use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmployeeInput {
    pub name: String,
    pub phone: String,
    pub position: String,
    pub id_card: String,
    pub hire_date: String,
    pub salary: f64,
    pub grade_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmployeeUpdateInput {
    pub id: i64,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub position: Option<String>,
    pub id_card: Option<String>,
    pub hire_date: Option<String>,
    pub salary: Option<f64>,
    pub insurance_base: Option<f64>,
    pub housing_fund_base: Option<f64>,
    pub grade_id: Option<i64>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttendanceInput {
    pub employee_id: i64,
    pub date: String,
    pub check_in: Option<String>,
    pub check_out: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayrollRecord {
    pub employee_id: i64,
    pub employee_name: String,
    pub base_salary: f64,
    pub attendance_days: i64,
    pub overtime_hours: f64,
    pub performance_pay: f64,
    pub commission: f64,
    pub bonus: f64,
    pub deductions: f64,
    pub insurance_base: f64,
    pub pension_employer: f64,
    pub pension_employee: f64,
    pub medical_employer: f64,
    pub medical_employee: f64,
    pub unemployment_employer: f64,
    pub unemployment_employee: f64,
    pub injury_employer: f64,
    pub injury_employee: f64,
    pub housing_fund_employer: f64,
    pub housing_fund_employee: f64,
    pub income_tax: f64,
    pub net_salary: f64,
    pub period: String,
}

// ==================== 员工管理 ====================

#[tauri::command]
pub fn create_employee(state: State<AppState>, input: EmployeeInput) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO employees (name, phone, position, id_card, hire_date, salary, grade_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![input.name, input.phone, input.position, input.id_card, input.hire_date, input.salary, input.grade_id],
    ).map_err(|e| e.to_string())?;
    Ok(db.last_insert_rowid())
}

#[tauri::command]
pub fn get_all_employees(state: State<AppState>) -> Result<Vec<crate::db::models::Employee>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT e.id, e.name, e.phone, e.position, e.id_card, e.hire_date, e.salary, e.insurance_base, e.housing_fund_base, e.grade_id, 
                COALESCE(g.name, ''), COALESCE(g.performance_ratio, 0), e.is_active, e.created_at, e.updated_at
         FROM employees e
         LEFT JOIN position_grades g ON e.grade_id = g.id
         ORDER BY e.id"
    ).map_err(|e| e.to_string())?;
    let employees = stmt.query_map([], |row| {
        Ok(crate::db::models::Employee {
            id: row.get(0)?,
            name: row.get(1)?,
            phone: row.get(2)?,
            position: row.get(3)?,
            id_card: row.get(4)?,
            hire_date: row.get(5)?,
            salary: row.get(6)?,
            insurance_base: row.get(7)?,
            housing_fund_base: row.get(8)?,
            grade_id: row.get(9)?,
            grade_name: row.get(10)?,
            grade_performance_ratio: row.get(11)?,
            is_active: row.get::<_, i32>(12)? != 0,
            created_at: row.get(13)?,
            updated_at: row.get(14)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(employees)
}

#[tauri::command]
pub fn update_employee(state: State<AppState>, input: EmployeeUpdateInput) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    // 构建动态 UPDATE 语句
    let mut updates = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    
    if let Some(name) = &input.name {
        updates.push("name = ?");
        params.push(Box::new(name.clone()));
    }
    if let Some(phone) = &input.phone {
        updates.push("phone = ?");
        params.push(Box::new(phone.clone()));
    }
    if let Some(position) = &input.position {
        updates.push("position = ?");
        params.push(Box::new(position.clone()));
    }
    if let Some(id_card) = &input.id_card {
        updates.push("id_card = ?");
        params.push(Box::new(id_card.clone()));
    }
    if let Some(hire_date) = &input.hire_date {
        updates.push("hire_date = ?");
        params.push(Box::new(hire_date.clone()));
    }
    if let Some(salary) = &input.salary {
        updates.push("salary = ?");
        params.push(Box::new(*salary));
    }
    if let Some(insurance_base) = &input.insurance_base {
        updates.push("insurance_base = ?");
        params.push(Box::new(*insurance_base));
    }
    if let Some(housing_fund_base) = &input.housing_fund_base {
        updates.push("housing_fund_base = ?");
        params.push(Box::new(*housing_fund_base));
    }
    if let Some(grade_id) = &input.grade_id {
        updates.push("grade_id = ?");
        params.push(Box::new(*grade_id));
    }
    if let Some(is_active) = &input.is_active {
        updates.push("is_active = ?");
        params.push(Box::new(*is_active as i32));
    }
    
    if updates.is_empty() {
        return Err("没有需要更新的字段".to_string());
    }
    
    updates.push("updated_at = datetime('now', 'localtime')");
    
    let sql = format!(
        "UPDATE employees SET {} WHERE id = ?",
        updates.join(", ")
    );
    params.push(Box::new(input.id));
    
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    db.execute(&sql, param_refs.as_slice()).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub fn delete_employee(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // 检查是否存在关联业务数据
    let attendance_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM attendance WHERE employee_id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    let review_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM performance_reviews WHERE employee_id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    let bonus_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM employee_bonuses WHERE employee_id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    let target_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM performance_targets WHERE employee_id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    if attendance_count > 0 || review_count > 0 || bonus_count > 0 || target_count > 0 {
        return Err("该员工存在考勤/绩效/奖金等业务数据，无法物理删除，请使用「离职」操作保留档案".to_string());
    }

    db.execute("DELETE FROM employees WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 强制物理删除员工及其全部关联数据（技术视角清理，仅供数据管理使用）
#[derive(Debug, Serialize, Deserialize)]
pub struct ForceDeleteResult {
    pub employee_name: String,
    pub attendance_count: i64,
    pub review_count: i64,
    pub bonus_count: i64,
    pub target_count: i64,
}

#[tauri::command]
pub fn force_delete_employee(state: State<AppState>, id: i64) -> Result<ForceDeleteResult, String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;

    let employee_name: String = db.query_row(
        "SELECT name FROM employees WHERE id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    let tx = db.transaction().map_err(|e| e.to_string())?;

    let attendance_count = tx.execute("DELETE FROM attendance WHERE employee_id = ?1", rusqlite::params![id]).map_err(|e| e.to_string())? as i64;
    let review_count = tx.execute("DELETE FROM performance_reviews WHERE employee_id = ?1", rusqlite::params![id]).map_err(|e| e.to_string())? as i64;
    let bonus_count = tx.execute("DELETE FROM employee_bonuses WHERE employee_id = ?1", rusqlite::params![id]).map_err(|e| e.to_string())? as i64;
    let target_count = tx.execute("DELETE FROM performance_targets WHERE employee_id = ?1", rusqlite::params![id]).map_err(|e| e.to_string())? as i64;
    tx.execute("DELETE FROM employees WHERE id = ?1", rusqlite::params![id]).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(ForceDeleteResult {
        employee_name,
        attendance_count,
        review_count,
        bonus_count,
        target_count,
    })
}

/// 查询员工关联数据概况（用于数据清理前的预览）
#[tauri::command]
pub fn get_employee_data_summary(state: State<AppState>, id: i64) -> Result<ForceDeleteResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let employee_name: String = db.query_row(
        "SELECT name FROM employees WHERE id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    let attendance_count: i64 = db.query_row("SELECT COUNT(*) FROM attendance WHERE employee_id = ?1", rusqlite::params![id], |r| r.get(0)).unwrap_or(0);
    let review_count: i64 = db.query_row("SELECT COUNT(*) FROM performance_reviews WHERE employee_id = ?1", rusqlite::params![id], |r| r.get(0)).unwrap_or(0);
    let bonus_count: i64 = db.query_row("SELECT COUNT(*) FROM employee_bonuses WHERE employee_id = ?1", rusqlite::params![id], |r| r.get(0)).unwrap_or(0);
    let target_count: i64 = db.query_row("SELECT COUNT(*) FROM performance_targets WHERE employee_id = ?1", rusqlite::params![id], |r| r.get(0)).unwrap_or(0);
    Ok(ForceDeleteResult {
        employee_name,
        attendance_count,
        review_count,
        bonus_count,
        target_count,
    })
}

// ==================== 考勤管理 ====================

#[tauri::command]
pub fn record_attendance(state: State<AppState>, input: AttendanceInput) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    // 判断是签到还是签退，自动设置状态
    let status = if input.check_in.is_some() || input.check_out.is_some() {
        "present"
    } else {
        "absent"
    };
    
    db.execute(
        "INSERT INTO attendance (employee_id, date, check_in, check_out, status) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(employee_id, date) DO UPDATE SET 
            check_in = COALESCE(?3, check_in), 
            check_out = COALESCE(?4, check_out),
            status = ?5",
        rusqlite::params![input.employee_id, input.date, input.check_in, input.check_out, status],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttendanceQuery {
    pub employee_id: Option<i64>,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttendanceRecord {
    pub id: i64,
    pub employee_id: i64,
    pub employee_name: String,
    pub date: String,
    pub check_in: Option<String>,
    pub check_out: Option<String>,
    pub status: String,
}

#[tauri::command]
pub fn query_attendance(state: State<AppState>, query: AttendanceQuery) -> Result<Vec<AttendanceRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let sql = if let Some(emp_id) = query.employee_id {
        format!(
            "SELECT a.id, a.employee_id, e.name, a.date, a.check_in, a.check_out, a.status 
             FROM attendance a JOIN employees e ON a.employee_id = e.id 
             WHERE a.employee_id = {} AND a.date BETWEEN '{}' AND '{}'
             ORDER BY a.date DESC",
            emp_id, query.start_date, query.end_date
        )
    } else {
        format!(
            "SELECT a.id, a.employee_id, e.name, a.date, a.check_in, a.check_out, a.status 
             FROM attendance a JOIN employees e ON a.employee_id = e.id 
             WHERE a.date BETWEEN '{}' AND '{}'
             ORDER BY a.date DESC, e.name",
            query.start_date, query.end_date
        )
    };
    
    let mut stmt = db.prepare(&sql).map_err(|e| e.to_string())?;
    let records = stmt.query_map([], |row| {
        Ok(AttendanceRecord {
            id: row.get(0)?,
            employee_id: row.get(1)?,
            employee_name: row.get(2)?,
            date: row.get(3)?,
            check_in: row.get(4)?,
            check_out: row.get(5)?,
            status: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(records)
}

// ==================== 工资表生成 ====================

#[tauri::command]
pub fn generate_payroll(state: State<AppState>, period: String) -> Result<Vec<PayrollRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let get_setting = |key: &str, default: f64| -> f64 {
        db.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            rusqlite::params![key],
            |row| {
                let v: String = row.get(0)?;
                Ok(v.parse::<f64>().unwrap_or(default))
            },
        ).unwrap_or(default)
    };

    let pension_er = get_setting("pension_employer_rate", 0.16);
    let pension_ee = get_setting("pension_employee_rate", 0.08);
    let medical_er = get_setting("medical_employer_rate", 0.098);
    let medical_ee = get_setting("medical_employee_rate", 0.02);
    let medical_fixed = get_setting("medical_employee_fixed", 3.0);
    let unemployment_er = get_setting("unemployment_employer_rate", 0.005);
    let unemployment_ee = get_setting("unemployment_employee_rate", 0.005);
    let injury_er = get_setting("injury_employer_rate", 0.002);
    let injury_ee = get_setting("injury_employee_rate", 0.0);
    let fund_er = get_setting("housing_fund_employer_rate", 0.05);
    let fund_ee = get_setting("housing_fund_employee_rate", 0.05);
    let ins_min = get_setting("insurance_base_min", 7162.0);
    let ins_max = get_setting("insurance_base_max", 35283.0);
    let fund_min = get_setting("housing_fund_base_min", 2420.0);
    let fund_max = get_setting("housing_fund_base_max", 35283.0);
    
    let (year, month) = period.split_once('-').ok_or("期间格式错误，应为 YYYY-MM")?;
    let start_date = format!("{}-{}-01", year, month);
    let next_month = if month == "12" { 
        format!("{}-01-01", year.parse::<i32>().unwrap_or(2025) + 1)
    } else {
        let m: i32 = month.parse().unwrap_or(1);
        format!("{}-{:02}-01", year, m + 1)
    };
    
    let mut emp_stmt = db.prepare(
        "SELECT e.id, e.name, e.salary, e.insurance_base, e.housing_fund_base, COALESCE(g.performance_ratio, 0)
         FROM employees e
         LEFT JOIN position_grades g ON e.grade_id = g.id
         WHERE e.is_active = 1"
    ).map_err(|e| e.to_string())?;
    
    let mut payroll_records = Vec::new();
    
    let employees: Vec<(i64, String, f64, f64, f64, f64)> = emp_stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    for (emp_id, emp_name, base_salary, ins_base, fund_base, perf_ratio) in employees {
        let attendance_days: i64 = db.query_row(
            "SELECT COUNT(*) FROM attendance WHERE employee_id = ?1 AND date >= ?2 AND date < ?3 AND status = 'present'",
            rusqlite::params![emp_id, start_date, next_month],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        
        let standard_days = 21.75;
        // 没有考勤记录时默认视为满勤
        let effective_days = if attendance_days == 0 { standard_days } else { attendance_days as f64 };
        let attendance_ratio = (effective_days / standard_days).min(1.0);
        let base_pay = base_salary * attendance_ratio;

        // 绩效工资 = 基本工资 × 绩效比例 × 绩效系数
        let coefficient: f64 = db.query_row(
            "SELECT coefficient FROM performance_reviews WHERE employee_id = ?1 AND period = ?2",
            rusqlite::params![emp_id, period],
            |row| row.get(0),
        ).unwrap_or(1.0);
        let performance_pay = base_salary * perf_ratio * coefficient;

        // 奖金汇总
        let bonus: f64 = db.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM employee_bonuses WHERE employee_id = ?1 AND period = ?2",
            rusqlite::params![emp_id, period],
            |row| row.get(0),
        ).unwrap_or(0.0);

        // 业绩提成
        let commission: f64 = db.query_row(
            "SELECT COALESCE(achieved_amount * commission_rate, 0) FROM performance_targets WHERE employee_id = ?1 AND period = ?2",
            rusqlite::params![emp_id, period],
            |row| row.get(0),
        ).unwrap_or(0.0);

        let deductions = 0.0;
        let gross_pay = base_pay + performance_pay + commission + bonus;

        let ib = if ins_base > 0.0 { ins_base } else { base_salary };
        let fb = if fund_base > 0.0 { fund_base } else { base_salary };
        let ib = ib.clamp(ins_min, ins_max);
        let fb = fb.clamp(fund_min, fund_max);

        let pe = ib * pension_er;
        let pe2 = ib * pension_ee;
        let me = ib * medical_er;
        let me2 = ib * medical_ee + medical_fixed;
        let ue = ib * unemployment_er;
        let ue2 = ib * unemployment_ee;
        let ie = ib * injury_er;
        let ie2 = ib * injury_ee;
        let fe = fb * fund_er;
        let fe2 = fb * fund_ee;

        let ee_total = pe2 + me2 + ue2 + ie2 + fe2;
        let taxable = (gross_pay - 5000.0 - ee_total).max(0.0);
        let (rate, qd) = match taxable {
            x if x <= 3000.0 => (0.03, 0.0),
            x if x <= 12000.0 => (0.10, 210.0),
            x if x <= 25000.0 => (0.20, 1410.0),
            x if x <= 35000.0 => (0.25, 2660.0),
            x if x <= 55000.0 => (0.30, 4410.0),
            x if x <= 80000.0 => (0.35, 7160.0),
            _ => (0.45, 15160.0),
        };
        let income_tax = (taxable * rate - qd).max(0.0);
        let net_salary = gross_pay - ee_total - income_tax - deductions;
        
        payroll_records.push(PayrollRecord {
            employee_id: emp_id,
            employee_name: emp_name,
            base_salary,
            attendance_days: effective_days as i64,
            overtime_hours: 0.0,
            performance_pay,
            commission,
            bonus,
            deductions,
            insurance_base: ib,
            pension_employer: pe,
            pension_employee: pe2,
            medical_employer: me,
            medical_employee: me2,
            unemployment_employer: ue,
            unemployment_employee: ue2,
            injury_employer: ie,
            injury_employee: ie2,
            housing_fund_employer: fe,
            housing_fund_employee: fe2,
            income_tax,
            net_salary,
            period: period.clone(),
        });
    }
    
    Ok(payroll_records)
}

// ==================== 薪资凭证自动生成 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct PayrollVoucherResult {
    pub voucher_no: String,
    pub total_salary: f64,
    pub employer_insurance: f64,
    pub employer_housing_fund: f64,
    pub employee_insurance: f64,
    pub employee_housing_fund: f64,
    pub income_tax_total: f64,
    pub net_pay_total: f64,
    pub total_insurance_pay: f64,
    pub total_fund_pay: f64,
}

#[tauri::command]
pub fn generate_payroll_voucher(
    state: State<AppState>,
    period: String,
    voucher_type: String,
    voucher_date: String,
) -> Result<PayrollVoucherResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let get_setting = |key: &str, default: f64| -> f64 {
        db.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            rusqlite::params![key],
            |row| {
                let v: String = row.get(0)?;
                Ok(v.parse::<f64>().unwrap_or(default))
            },
        ).unwrap_or(default)
    };

    let insurance_employer_rate = get_setting("pension_employer_rate", 0.16)
        + get_setting("medical_employer_rate", 0.098)
        + get_setting("unemployment_employer_rate", 0.005)
        + get_setting("injury_employer_rate", 0.002);
    let insurance_employee_rate = get_setting("pension_employee_rate", 0.08)
        + get_setting("medical_employee_rate", 0.02)
        + get_setting("unemployment_employee_rate", 0.005)
        + get_setting("injury_employee_rate", 0.0);
    let medical_employee_fixed = get_setting("medical_employee_fixed", 3.0);
    let housing_fund_employer_rate = get_setting("housing_fund_employer_rate", 0.05);
    let housing_fund_employee_rate = get_setting("housing_fund_employee_rate", 0.05);
    let insurance_base_min = get_setting("insurance_base_min", 7162.0);
    let insurance_base_max = get_setting("insurance_base_max", 35283.0);
    let housing_fund_base_min = get_setting("housing_fund_base_min", 2420.0);
    let housing_fund_base_max = get_setting("housing_fund_base_max", 35283.0);

    let account_id = |code: &str| -> Result<i64, String> {
        db.query_row(
            "SELECT id FROM accounts WHERE code = ?1",
            rusqlite::params![code],
            |row| row.get(0),
        ).map_err(|e| format!("科目 {} 不存在: {}", code, e))
    };

    let admin_expense_id = account_id("6602")?;
    let salary_payable_id = account_id("2211")?;
    let bank_deposit_id = account_id("1002")?;
    let other_payable_id = account_id("2241")?;
    let tax_payable_id = account_id("2221")?;

    let mut emp_stmt = db.prepare(
        "SELECT id, name, salary, insurance_base, housing_fund_base FROM employees WHERE is_active = 1"
    ).map_err(|e| e.to_string())?;

    let employees: Vec<(i64, String, f64, f64, f64)> = emp_stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    let mut total_salary = 0.0;
    let mut employer_insurance = 0.0;
    let mut employer_housing_fund = 0.0;
    let mut employee_insurance = 0.0;
    let mut employee_housing_fund = 0.0;
    let mut income_tax_total = 0.0;

    for (_, _, salary, ins_base, fund_base) in &employees {
        let ib = if *ins_base > 0.0 { *ins_base } else { *salary };
        let fb = if *fund_base > 0.0 { *fund_base } else { *salary };
        let ib = ib.clamp(insurance_base_min, insurance_base_max);
        let fb = fb.clamp(housing_fund_base_min, housing_fund_base_max);

        let ei = ib * insurance_employer_rate;
        let ee = ib * insurance_employee_rate + medical_employee_fixed;
        let ef = fb * housing_fund_employer_rate;
        let eh = fb * housing_fund_employee_rate;

        let taxable = (*salary - 5000.0 - ee - eh).max(0.0);
        let (rate, qd) = match taxable {
            x if x <= 3000.0 => (0.03, 0.0),
            x if x <= 12000.0 => (0.10, 210.0),
            x if x <= 25000.0 => (0.20, 1410.0),
            x if x <= 35000.0 => (0.25, 2660.0),
            x if x <= 55000.0 => (0.30, 4410.0),
            x if x <= 80000.0 => (0.35, 7160.0),
            _ => (0.45, 15160.0),
        };
        let tax = (taxable * rate - qd).max(0.0);

        total_salary += salary;
        employer_insurance += ei;
        employer_housing_fund += ef;
        employee_insurance += ee;
        employee_housing_fund += eh;
        income_tax_total += tax;
    }

    let net_pay_total = total_salary - employee_insurance - employee_housing_fund - income_tax_total;
    let total_insurance_pay = employer_insurance + employee_insurance;
    let total_fund_pay = employer_housing_fund + employee_housing_fund;

    let (_, month) = period.split_once('-').ok_or("期间格式错误")?;

    let gen_voucher_no = |prefix: &str| -> Result<String, String> {
        let count: i64 = db.query_row(
            "SELECT COUNT(*) FROM vouchers WHERE date = ?1 AND voucher_no LIKE ?2",
            rusqlite::params![voucher_date, format!("{}%", prefix)],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        Ok(format!("{}-{:04}", prefix, count + 1))
    };

    let insert_entries = |voucher_id: i64, rows: &[(i64, f64, f64, String)]| -> Result<(), String> {
        for (aid, debit, credit, summary) in rows {
            if *debit == 0.0 && *credit == 0.0 {
                continue;
            }
            db.execute(
                "INSERT INTO voucher_entries (voucher_id, account_id, debit_amount, credit_amount, summary) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![voucher_id, aid, debit, credit, summary],
            ).map_err(|e| e.to_string())?;
        }
        Ok(())
    };

    let (prefix, summary, entries) = match voucher_type.as_str() {
        "accrual" => (
            format!("JTGZ{}", period.replace("-", "")),
            format!("{} 计提工资及社保公积金", period),
            vec![
                (admin_expense_id, total_salary, 0.0, format!("计提{}月工资", month)),
                (admin_expense_id, employer_insurance, 0.0, format!("计提{}月社保-公司部分", month)),
                (admin_expense_id, employer_housing_fund, 0.0, format!("计提{}月公积金-公司部分", month)),
                (salary_payable_id, 0.0, total_salary, format!("应付{}月工资", month)),
                (salary_payable_id, 0.0, employer_insurance, format!("应付{}月社保-公司部分", month)),
                (salary_payable_id, 0.0, employer_housing_fund, format!("应付{}月公积金-公司部分", month)),
            ],
        ),
        "salary_payment" => (
            format!("FFGZ{}", period.replace("-", "")),
            format!("{} 发放工资", period),
            vec![
                (salary_payable_id, total_salary, 0.0, format!("支付{}月工资", month)),
                (bank_deposit_id, 0.0, net_pay_total, format!("实发{}月工资", month)),
                (other_payable_id, 0.0, employee_insurance, format!("代扣{}月个人社保", month)),
                (other_payable_id, 0.0, employee_housing_fund, format!("代扣{}月个人公积金", month)),
                (tax_payable_id, 0.0, income_tax_total, format!("代扣{}月个人所得税", month)),
            ],
        ),
        "insurance_payment" => (
            format!("JNSB{}", period.replace("-", "")),
            format!("{} 缴纳社保", period),
            vec![
                (salary_payable_id, employer_insurance, 0.0, format!("支付{}月社保-公司部分", month)),
                (other_payable_id, employee_insurance, 0.0, format!("支付{}月社保-个人部分", month)),
                (bank_deposit_id, 0.0, total_insurance_pay, format!("缴纳{}月社保", month)),
            ],
        ),
        "fund_payment" => (
            format!("JNGJJ{}", period.replace("-", "")),
            format!("{} 缴纳公积金", period),
            vec![
                (salary_payable_id, employer_housing_fund, 0.0, format!("支付{}月公积金-公司部分", month)),
                (other_payable_id, employee_housing_fund, 0.0, format!("支付{}月公积金-个人部分", month)),
                (bank_deposit_id, 0.0, total_fund_pay, format!("缴纳{}月公积金", month)),
            ],
        ),
        _ => return Err("无效的凭证类型".to_string()),
    };

    let voucher_no = gen_voucher_no(&prefix)?;
    db.execute(
        "INSERT INTO vouchers (voucher_no, date, summary) VALUES (?1, ?2, ?3)",
        rusqlite::params![voucher_no, voucher_date, summary],
    ).map_err(|e| e.to_string())?;
    let vid = db.last_insert_rowid();
    insert_entries(vid, &entries)?;

    Ok(PayrollVoucherResult {
        voucher_no,
        total_salary,
        employer_insurance,
        employer_housing_fund,
        employee_insurance,
        employee_housing_fund,
        income_tax_total,
        net_pay_total,
        total_insurance_pay,
        total_fund_pay,
    })
}

// ==================== 岗位等级管理 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct PositionGrade {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub base_salary_min: f64,
    pub base_salary_max: f64,
    pub performance_ratio: f64,
    pub post_coefficient: f64,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PositionGradeInput {
    pub name: String,
    pub code: String,
    pub base_salary_min: f64,
    pub base_salary_max: f64,
    pub performance_ratio: f64,
    pub post_coefficient: f64,
}

#[tauri::command]
pub fn get_position_grades(state: State<AppState>) -> Result<Vec<PositionGrade>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT id, name, code, base_salary_min, base_salary_max, performance_ratio, post_coefficient, is_active
         FROM position_grades ORDER BY base_salary_min"
    ).map_err(|e| e.to_string())?;
    let grades = stmt.query_map([], |row| {
        Ok(PositionGrade {
            id: row.get(0)?,
            name: row.get(1)?,
            code: row.get(2)?,
            base_salary_min: row.get(3)?,
            base_salary_max: row.get(4)?,
            performance_ratio: row.get(5)?,
            post_coefficient: row.get(6)?,
            is_active: row.get::<_, i32>(7)? != 0,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(grades)
}

#[tauri::command]
pub fn create_position_grade(state: State<AppState>, input: PositionGradeInput) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO position_grades (name, code, base_salary_min, base_salary_max, performance_ratio, post_coefficient) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![input.name, input.code, input.base_salary_min, input.base_salary_max, input.performance_ratio, input.post_coefficient],
    ).map_err(|e| e.to_string())?;
    Ok(db.last_insert_rowid())
}

#[tauri::command]
pub fn update_position_grade(state: State<AppState>, id: i64, input: PositionGradeInput) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "UPDATE position_grades SET name = ?1, code = ?2, base_salary_min = ?3, base_salary_max = ?4, performance_ratio = ?5, post_coefficient = ?6 WHERE id = ?7",
        rusqlite::params![input.name, input.code, input.base_salary_min, input.base_salary_max, input.performance_ratio, input.post_coefficient, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_position_grade(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "UPDATE position_grades SET is_active = 0 WHERE id = ?1",
        rusqlite::params![id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 绩效考核 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceReview {
    pub id: i64,
    pub employee_id: i64,
    pub employee_name: String,
    pub period: String,
    pub score: f64,
    pub grade: String,
    pub coefficient: f64,
    pub business_score: f64,
    pub growth_score: f64,
    pub collaboration_score: f64,
    pub attitude_score: f64,
    pub innovation_score: f64,
    pub reviewer: String,
    pub remark: String,
}

fn score_to_grade(score: f64) -> (String, f64) {
    if score >= 95.0 {
        ("S".to_string(), 1.2)
    } else if score >= 85.0 {
        ("A".to_string(), 1.0)
    } else if score >= 70.0 {
        ("B".to_string(), 0.9)
    } else if score >= 60.0 {
        ("C".to_string(), 0.7)
    } else {
        ("D".to_string(), 0.5)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceReviewInput {
    pub employee_id: i64,
    pub period: String,
    pub business_score: f64,
    pub growth_score: f64,
    pub collaboration_score: f64,
    pub attitude_score: f64,
    pub innovation_score: f64,
    pub reviewer: String,
    pub remark: String,
}

#[tauri::command]
pub fn save_performance_review(state: State<AppState>, input: PerformanceReviewInput) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let get_weight = |key: &str, default: f64| -> f64 {
        db.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            rusqlite::params![key],
            |row| {
                let v: String = row.get(0)?;
                Ok(v.parse::<f64>().unwrap_or(default))
            },
        ).unwrap_or(default)
    };
    let w_business = get_weight("review_weight_business", 0.3);
    let w_growth = get_weight("review_weight_growth", 0.2);
    let w_collaboration = get_weight("review_weight_collaboration", 0.2);
    let w_attitude = get_weight("review_weight_attitude", 0.15);
    let w_innovation = get_weight("review_weight_innovation", 0.15);

    let score = input.business_score * w_business
        + input.growth_score * w_growth
        + input.collaboration_score * w_collaboration
        + input.attitude_score * w_attitude
        + input.innovation_score * w_innovation;
    let (grade, coefficient) = score_to_grade(score);

    db.execute(
        "INSERT INTO performance_reviews (employee_id, period, score, grade, coefficient, business_score, growth_score, collaboration_score, attitude_score, innovation_score, reviewer, remark)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
         ON CONFLICT(employee_id, period) DO UPDATE SET
            score = ?3, grade = ?4, coefficient = ?5,
            business_score = ?6, growth_score = ?7, collaboration_score = ?8, attitude_score = ?9, innovation_score = ?10,
            reviewer = ?11, remark = ?12",
        rusqlite::params![
            input.employee_id, input.period, score, grade, coefficient,
            input.business_score, input.growth_score, input.collaboration_score, input.attitude_score, input.innovation_score,
            input.reviewer, input.remark
        ],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_performance_reviews(state: State<AppState>, period: String) -> Result<Vec<PerformanceReview>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT pr.id, pr.employee_id, e.name, pr.period, pr.score, pr.grade, pr.coefficient,
                pr.business_score, pr.growth_score, pr.collaboration_score, pr.attitude_score, pr.innovation_score,
                pr.reviewer, pr.remark
         FROM performance_reviews pr
         JOIN employees e ON pr.employee_id = e.id
         WHERE pr.period = ?1
         ORDER BY e.id"
    ).map_err(|e| e.to_string())?;
    let reviews = stmt.query_map(rusqlite::params![period], |row| {
        Ok(PerformanceReview {
            id: row.get(0)?,
            employee_id: row.get(1)?,
            employee_name: row.get(2)?,
            period: row.get(3)?,
            score: row.get(4)?,
            grade: row.get(5)?,
            coefficient: row.get(6)?,
            business_score: row.get(7)?,
            growth_score: row.get(8)?,
            collaboration_score: row.get(9)?,
            attitude_score: row.get(10)?,
            innovation_score: row.get(11)?,
            reviewer: row.get(12)?,
            remark: row.get(13)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(reviews)
}

// ==================== 奖金管理 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct EmployeeBonus {
    pub id: i64,
    pub employee_id: i64,
    pub employee_name: String,
    pub period: String,
    pub bonus_type: String,
    pub amount: f64,
    pub remark: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmployeeBonusInput {
    pub employee_id: i64,
    pub period: String,
    pub bonus_type: String,
    pub amount: f64,
    pub remark: String,
}

#[tauri::command]
pub fn save_employee_bonus(state: State<AppState>, input: EmployeeBonusInput) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO employee_bonuses (employee_id, period, bonus_type, amount, remark) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![input.employee_id, input.period, input.bonus_type, input.amount, input.remark],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_employee_bonuses(state: State<AppState>, period: String) -> Result<Vec<EmployeeBonus>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT b.id, b.employee_id, e.name, b.period, b.bonus_type, b.amount, b.remark
         FROM employee_bonuses b
         JOIN employees e ON b.employee_id = e.id
         WHERE b.period = ?1
         ORDER BY e.id, b.bonus_type"
    ).map_err(|e| e.to_string())?;
    let bonuses = stmt.query_map(rusqlite::params![period], |row| {
        Ok(EmployeeBonus {
            id: row.get(0)?,
            employee_id: row.get(1)?,
            employee_name: row.get(2)?,
            period: row.get(3)?,
            bonus_type: row.get(4)?,
            amount: row.get(5)?,
            remark: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(bonuses)
}

#[tauri::command]
pub fn delete_employee_bonus(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM employee_bonuses WHERE id = ?1", rusqlite::params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 业绩提成 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceTarget {
    pub id: i64,
    pub employee_id: i64,
    pub employee_name: String,
    pub period: String,
    pub target_amount: f64,
    pub achieved_amount: f64,
    pub commission_rate: f64,
    pub commission: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceTargetInput {
    pub employee_id: i64,
    pub period: String,
    pub target_amount: f64,
    pub achieved_amount: f64,
    pub commission_rate: f64,
}

#[tauri::command]
pub fn save_performance_target(state: State<AppState>, input: PerformanceTargetInput) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO performance_targets (employee_id, period, target_amount, achieved_amount, commission_rate) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(employee_id, period) DO UPDATE SET target_amount = ?3, achieved_amount = ?4, commission_rate = ?5",
        rusqlite::params![input.employee_id, input.period, input.target_amount, input.achieved_amount, input.commission_rate],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_performance_targets(state: State<AppState>, period: String) -> Result<Vec<PerformanceTarget>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT t.id, t.employee_id, e.name, t.period, t.target_amount, t.achieved_amount, t.commission_rate
         FROM performance_targets t
         JOIN employees e ON t.employee_id = e.id
         WHERE t.period = ?1
         ORDER BY e.id"
    ).map_err(|e| e.to_string())?;
    let targets = stmt.query_map(rusqlite::params![period], |row| {
        let achieved: f64 = row.get(5)?;
        let rate: f64 = row.get(6)?;
        Ok(PerformanceTarget {
            id: row.get(0)?,
            employee_id: row.get(1)?,
            employee_name: row.get(2)?,
            period: row.get(3)?,
            target_amount: row.get(4)?,
            achieved_amount: achieved,
            commission_rate: rate,
            commission: achieved * rate,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(targets)
}
