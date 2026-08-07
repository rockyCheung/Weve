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
        "INSERT INTO employees (name, phone, position, id_card, hire_date, salary) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![input.name, input.phone, input.position, input.id_card, input.hire_date, input.salary],
    ).map_err(|e| e.to_string())?;
    Ok(db.last_insert_rowid())
}

#[tauri::command]
pub fn get_all_employees(state: State<AppState>) -> Result<Vec<crate::db::models::Employee>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT id, name, phone, position, id_card, hire_date, salary, insurance_base, housing_fund_base, is_active, created_at, updated_at 
         FROM employees ORDER BY id"
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
            is_active: row.get::<_, i32>(9)? != 0,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
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
    db.execute(
        "UPDATE employees SET is_active = 0, updated_at = datetime('now', 'localtime') WHERE id = ?1",
        rusqlite::params![id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 考勤管理 ====================

#[tauri::command]
pub fn record_attendance(state: State<AppState>, input: AttendanceInput) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    // 判断是签到还是签退，自动设置状态
    let status = if input.check_in.is_some() && input.check_out.is_some() {
        "present"
    } else if input.check_in.is_some() {
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
        "SELECT id, name, salary, insurance_base, housing_fund_base FROM employees WHERE is_active = 1"
    ).map_err(|e| e.to_string())?;
    
    let mut payroll_records = Vec::new();
    
    let employees: Vec<(i64, String, f64, f64, f64)> = emp_stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    for (emp_id, emp_name, base_salary, ins_base, fund_base) in employees {
        let attendance_days: i64 = db.query_row(
            "SELECT COUNT(*) FROM attendance WHERE employee_id = ?1 AND date >= ?2 AND date < ?3 AND status = 'present'",
            rusqlite::params![emp_id, start_date, next_month],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        
        let standard_days = 21.75;
        // 没有考勤记录时默认视为满勤
        let effective_days = if attendance_days == 0 { standard_days } else { attendance_days as f64 };
        let attendance_ratio = (effective_days / standard_days).min(1.0);
        let gross_pay = base_salary * attendance_ratio;
        let bonus = 0.0;
        let deductions = 0.0;

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
        let net_salary = gross_pay - ee_total - income_tax + bonus - deductions;
        
        payroll_records.push(PayrollRecord {
            employee_id: emp_id,
            employee_name: emp_name,
            base_salary,
            attendance_days: effective_days as i64,
            overtime_hours: 0.0,
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
