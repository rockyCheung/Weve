use serde::{Deserialize, Serialize};
use rusqlite::Connection;

// ==================== 增值税月报 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VatReport {
    pub period: String,
    pub output_count: i64,
    pub output_amount: f64,
    pub output_tax: f64,
    pub input_count: i64,
    pub input_amount: f64,
    pub input_tax: f64,
    pub tax_due: f64,
    pub surcharge_city: f64,      // 城建税 (7%)
    pub surcharge_education: f64, // 教育费附加 (3%)
    pub surcharge_local: f64,     // 地方教育附加 (2%)
    pub total_tax_due: f64,       // 合计应缴
    pub items: Vec<VatReportItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VatReportItem {
    pub tax_rate: f64,
    pub output_amount: f64,
    pub output_tax: f64,
    pub input_amount: f64,
    pub input_tax: f64,
}

/// 生成增值税月报
pub fn generate_vat_report(conn: &Connection, period: &str) -> Result<VatReport, String> {
    let (year, month) = period.split_once('-').ok_or("Invalid period format")?;
    let start_date = format!("{}-{}-01", year, month);
    
    // 计算下个月的第一天
    let next_month = if month == "12" {
        format!("{}-01-01", year.parse::<i32>().unwrap_or(2025) + 1)
    } else {
        let m: i32 = month.parse().unwrap_or(1);
        format!("{}-{:02}-01", year, m + 1)
    };

    // 销项汇总
    let (output_count, output_amount, output_tax): (i64, f64, f64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(amount), 0), COALESCE(SUM(tax_amount), 0) 
         FROM invoices WHERE invoice_type = 'output' AND date >= ?1 AND date < ?2",
        rusqlite::params![start_date, next_month],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).map_err(|e| e.to_string())?;

    // 进项汇总（只统计已认证的）
    let (input_count, input_amount, input_tax): (i64, f64, f64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(amount), 0), COALESCE(SUM(tax_amount), 0) 
         FROM invoices WHERE invoice_type = 'input' AND is_verified = 1 AND date >= ?1 AND date < ?2",
        rusqlite::params![start_date, next_month],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).map_err(|e| e.to_string())?;

    // 按税率分组统计（用于明细展示）
    let mut stmt = conn.prepare(
        "SELECT tax_rate,
                COALESCE(SUM(CASE WHEN invoice_type='output' THEN amount ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN invoice_type='output' THEN tax_amount ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN invoice_type='input' AND is_verified=1 THEN amount ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN invoice_type='input' AND is_verified=1 THEN tax_amount ELSE 0 END), 0)
         FROM invoices WHERE date >= ?1 AND date < ?2
         GROUP BY tax_rate ORDER BY tax_rate DESC"
    ).map_err(|e| e.to_string())?;

    let items: Vec<VatReportItem> = stmt.query_map(
        rusqlite::params![start_date, next_month],
        |row| {
            Ok(VatReportItem {
                tax_rate: row.get(0)?,
                output_amount: row.get(1)?,
                output_tax: row.get(2)?,
                input_amount: row.get(3)?,
                input_tax: row.get(4)?,
            })
        },
    ).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    // 计算应缴税额
    let tax_due = (output_tax - input_tax).max(0.0);
    
    // 附加税计算（基于增值税额）
    let surcharge_city = tax_due * 0.07;
    let surcharge_education = tax_due * 0.03;
    let surcharge_local = tax_due * 0.02;
    let total_tax_due = tax_due + surcharge_city + surcharge_education + surcharge_local;

    Ok(VatReport {
        period: period.to_string(),
        output_count,
        output_amount,
        output_tax,
        input_count,
        input_amount,
        input_tax,
        tax_due,
        surcharge_city,
        surcharge_education,
        surcharge_local,
        total_tax_due,
        items,
    })
}

// ==================== 企业所得税季报 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CitReport {
    pub period: String,
    pub revenue: f64,           // 营业收入
    pub cost: f64,              // 营业成本
    pub gross_profit: f64,      // 毛利
    pub expenses: f64,          // 期间费用
    pub profit: f64,            // 利润总额
    pub tax_rate: f64,          // 适用税率
    pub tax_due: f64,           // 应缴企业所得税
    pub is_small_profit: bool,  // 是否小微企业优惠
}

/// 生成企业所得税季报（period 格式：YYYY-Qn，如 2026-Q1）
pub fn generate_cit_report(conn: &Connection, period: &str) -> Result<CitReport, String> {
    let (year, quarter) = period.split_once('-').ok_or("Invalid period format")?;
    let quarter_num: i32 = quarter
        .trim_start_matches('Q')
        .parse()
        .map_err(|_| "Invalid quarter format")?;
    let start_month = match quarter_num {
        1 => "01",
        2 => "04",
        3 => "07",
        4 => "10",
        _ => return Err("季度范围 1-4".to_string()),
    };
    let quarter_start = format!("{}-{}-01", year, start_month);

    // 下一季度第一天（用于范围截止）
    let next_quarter = if quarter_num == 4 {
        format!("{}-01-01", year.parse::<i32>().unwrap_or(2025) + 1)
    } else {
        format!("{}-{:02}-01", year, quarter_num * 3 + 1)
    };

    // 从收入科目（6开头）汇总营业收入
    let revenue: f64 = conn.query_row(
        "SELECT COALESCE(SUM(ve.credit_amount), 0) 
         FROM voucher_entries ve
         JOIN accounts a ON ve.account_id = a.id
         JOIN vouchers v ON ve.voucher_id = v.id
         WHERE a.code LIKE '6%' AND v.date >= ?1 AND v.date < ?2",
        rusqlite::params![quarter_start, next_quarter],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    // 从成本科目（6401-6402）汇总营业成本
    let cost: f64 = conn.query_row(
        "SELECT COALESCE(SUM(ve.debit_amount), 0) 
         FROM voucher_entries ve
         JOIN accounts a ON ve.account_id = a.id
         JOIN vouchers v ON ve.voucher_id = v.id
         WHERE a.code IN ('6401', '6402') AND v.date >= ?1 AND v.date < ?2",
        rusqlite::params![quarter_start, next_quarter],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    // 从费用科目（66开头）汇总期间费用
    let expenses: f64 = conn.query_row(
        "SELECT COALESCE(SUM(ve.debit_amount), 0) 
         FROM voucher_entries ve
         JOIN accounts a ON ve.account_id = a.id
         JOIN vouchers v ON ve.voucher_id = v.id
         WHERE a.code LIKE '66%' AND v.date >= ?1 AND v.date < ?2",
        rusqlite::params![quarter_start, next_quarter],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    let gross_profit = revenue - cost;
    let profit = gross_profit - expenses;

    // 小微企业优惠税率判断（年应纳税所得额）
    // 简化：按季度利润 * 4 估算年利润
    let estimated_annual_profit = profit * 4.0;
    let (tax_rate, is_small_profit) = if estimated_annual_profit <= 1_000_000.0 {
        (0.025, true)  // 年利润100万以内，实际税负2.5%
    } else if estimated_annual_profit <= 3_000_000.0 {
        (0.05, true)   // 100-300万，实际税负5%
    } else {
        (0.25, false)  // 超过300万，标准税率25%
    };

    let tax_due = (profit * tax_rate).max(0.0);

    Ok(CitReport {
        period: period.to_string(),
        revenue,
        cost,
        gross_profit,
        expenses,
        profit,
        tax_rate,
        tax_due,
        is_small_profit,
    })
}

// ==================== 个人所得税代扣代缴 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonalIncomeTaxReport {
    pub period: String,
    pub employees: Vec<PersonalIncomeTaxItem>,
    pub total_income: f64,
    pub total_tax: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonalIncomeTaxItem {
    pub employee_name: String,
    pub id_card: String,
    pub income: f64,           // 收入额
    pub deduction: f64,        // 减除费用（5000）
    pub special_deduction: f64,// 专项扣除（社保等）
    pub taxable_income: f64,   // 应纳税所得额
    pub tax_rate: f64,         // 税率
    pub quick_deduction: f64,  // 速算扣除数
    pub tax_due: f64,          // 应缴税额
}

/// 生成个人所得税代扣代缴报告
pub fn generate_personal_income_tax_report(conn: &Connection, period: &str) -> Result<PersonalIncomeTaxReport, String> {
    let (_year, _month) = period.split_once('-').ok_or("Invalid period format")?;

    // 获取所有在职员工
    let mut emp_stmt = conn.prepare(
        "SELECT id, name, id_card, salary FROM employees WHERE is_active = 1"
    ).map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    let mut total_income = 0.0;
    let mut total_tax = 0.0;

    let employees: Vec<(i64, String, String, f64)> = emp_stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    for (_, name, id_card, salary) in employees {
        let income = salary;  // 月工资收入
        let deduction = 5000.0;  // 基本减除费用
        let special_deduction = income * 0.105;  // 社保公积金（简化：10.5%）
        let taxable_income = (income - deduction - special_deduction).max(0.0);

        // 个人所得税累进税率计算
        let (tax_rate, quick_deduction) = match taxable_income {
            x if x <= 3000.0 => (0.03, 0.0),
            x if x <= 12000.0 => (0.10, 210.0),
            x if x <= 25000.0 => (0.20, 1410.0),
            x if x <= 35000.0 => (0.25, 2660.0),
            x if x <= 55000.0 => (0.30, 4410.0),
            x if x <= 80000.0 => (0.35, 7160.0),
            _ => (0.45, 15160.0),
        };

        let tax_due = taxable_income * tax_rate - quick_deduction;

        items.push(PersonalIncomeTaxItem {
            employee_name: name,
            id_card,
            income,
            deduction,
            special_deduction,
            taxable_income,
            tax_rate,
            quick_deduction,
            tax_due: tax_due.max(0.0),
        });

        total_income += income;
        total_tax += tax_due.max(0.0);
    }

    Ok(PersonalIncomeTaxReport {
        period: period.to_string(),
        employees: items,
        total_income,
        total_tax,
    })
}

// ==================== 财务报表 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BalanceSheet {
    pub period: String,
    pub assets: Vec<BalanceSheetItem>,
    pub liabilities: Vec<BalanceSheetItem>,
    pub equity: Vec<BalanceSheetItem>,
    pub total_assets: f64,
    pub total_liabilities: f64,
    pub total_equity: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BalanceSheetItem {
    pub account_code: String,
    pub account_name: String,
    pub balance: f64,
}

/// 生成资产负债表
pub fn generate_balance_sheet(conn: &Connection, end_date: &str) -> Result<BalanceSheet, String> {

    // 获取所有科目余额
    let mut stmt = conn.prepare(
        "SELECT a.code, a.name, a.category,
                COALESCE(SUM(ve.debit_amount), 0) - COALESCE(SUM(ve.credit_amount), 0) as balance
         FROM accounts a
         LEFT JOIN voucher_entries ve ON a.id = ve.account_id
         LEFT JOIN vouchers v ON ve.voucher_id = v.id AND v.date <= ?1
         WHERE a.is_active = 1
         GROUP BY a.id, a.code, a.name, a.category
         ORDER BY a.code"
    ).map_err(|e| e.to_string())?;

    let mut assets = Vec::new();
    let mut liabilities = Vec::new();
    let mut equity = Vec::new();
    let mut total_assets = 0.0;
    let mut total_liabilities = 0.0;
    let mut total_equity = 0.0;

    let items = stmt.query_map(rusqlite::params![end_date], |row| {
        let code: String = row.get(0)?;
        let name: String = row.get(1)?;
        let category: String = row.get(2)?;
        let balance: f64 = row.get(3)?;
        Ok((code, name, category, balance))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect::<Vec<_>>();

    for (code, name, category, balance) in items {
        let item = BalanceSheetItem {
            account_code: code,
            account_name: name,
            balance: balance.abs(),
        };

        match category.as_str() {
            "asset" => {
                assets.push(item);
                total_assets += balance.abs();
            }
            "liability" => {
                liabilities.push(item);
                total_liabilities += balance.abs();
            }
            "equity" => {
                equity.push(item);
                total_equity += balance.abs();
            }
            _ => {}
        }
    }

    Ok(BalanceSheet {
        period: end_date.to_string(),
        assets,
        liabilities,
        equity,
        total_assets,
        total_liabilities,
        total_equity,
    })
}

// ==================== 利润表 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncomeStatement {
    pub period: String,
    pub revenue: f64,                    // 营业收入
    pub cost: f64,                       // 营业成本
    pub gross_profit: f64,               // 毛利
    pub selling_expenses: f64,           // 销售费用
    pub admin_expenses: f64,             // 管理费用
    pub finance_expenses: f64,           // 财务费用
    pub operating_profit: f64,           // 营业利润
    pub non_operating_income: f64,       // 营业外收入
    pub non_operating_expenses: f64,     // 营业外支出
    pub profit_before_tax: f64,          // 利润总额
    pub income_tax: f64,                 // 所得税费用
    pub net_profit: f64,                 // 净利润
}

/// 生成利润表
pub fn generate_income_statement(conn: &Connection, start_date: &str, end_date: &str) -> Result<IncomeStatement, String> {

    // 查询各科目发生额
    let get_balance = |code_prefix: &str, is_debit: bool| -> Result<f64, String> {
        let sql = if is_debit {
            format!(
                "SELECT COALESCE(SUM(ve.debit_amount), 0) 
                 FROM voucher_entries ve
                 JOIN accounts a ON ve.account_id = a.id
                 JOIN vouchers v ON ve.voucher_id = v.id
                 WHERE a.code LIKE '{}' AND v.date >= ?1 AND v.date <= ?2",
                code_prefix
            )
        } else {
            format!(
                "SELECT COALESCE(SUM(ve.credit_amount), 0) 
                 FROM voucher_entries ve
                 JOIN accounts a ON ve.account_id = a.id
                 JOIN vouchers v ON ve.voucher_id = v.id
                 WHERE a.code LIKE '{}' AND v.date >= ?1 AND v.date <= ?2",
                code_prefix
            )
        };
        conn.query_row(&sql, rusqlite::params![start_date, end_date], |row| row.get(0))
            .map_err(|e| e.to_string())
    };

    let revenue = get_balance("6001%", false)?;         // 主营业务收入（贷方）
    let cost = get_balance("6401%", true)?;              // 主营业务成本（借方）
    let selling_expenses = get_balance("6601%", true)?;  // 销售费用（借方）
    let admin_expenses = get_balance("6602%", true)?;    // 管理费用（借方）
    let finance_expenses = get_balance("6603%", true)?;  // 财务费用（借方）
    let non_operating_income = get_balance("6301%", false)?; // 营业外收入（贷方）
    let non_operating_expenses = get_balance("6711%", true)?; // 营业外支出（借方）
    let income_tax = get_balance("6801%", true)?;        // 所得税费用（借方）

    let gross_profit = revenue - cost;
    let operating_profit = gross_profit - selling_expenses - admin_expenses - finance_expenses;
    let profit_before_tax = operating_profit + non_operating_income - non_operating_expenses;
    let net_profit = profit_before_tax - income_tax;

    Ok(IncomeStatement {
        period: format!("{}~{}", start_date, end_date),
        revenue,
        cost,
        gross_profit,
        selling_expenses,
        admin_expenses,
        finance_expenses,
        operating_profit,
        non_operating_income,
        non_operating_expenses,
        profit_before_tax,
        income_tax,
        net_profit,
    })
}

// ==================== 现金流量表（简化版） ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CashFlowStatement {
    pub period: String,
    pub operating_cash_in: f64,      // 经营活动现金流入
    pub operating_cash_out: f64,     // 经营活动现金流出
    pub operating_net: f64,          // 经营活动净现金流
    pub investing_cash_in: f64,
    pub investing_cash_out: f64,
    pub investing_net: f64,
    pub financing_cash_in: f64,
    pub financing_cash_out: f64,
    pub financing_net: f64,
    pub net_increase: f64,           // 现金净增加额
}

/// 生成现金流量表（简化版，基于银行存款科目变动）
pub fn generate_cash_flow_statement(conn: &Connection, start_date: &str, end_date: &str) -> Result<CashFlowStatement, String> {

    // 银行存款科目（假设 code = '1002'）
    let bank_cash_in: f64 = conn.query_row(
        "SELECT COALESCE(SUM(ve.debit_amount), 0) 
         FROM voucher_entries ve
         JOIN vouchers v ON ve.voucher_id = v.id
         WHERE ve.account_id = (SELECT id FROM accounts WHERE code = '1002')
          AND v.date >= ?1 AND v.date <= ?2",
        rusqlite::params![start_date, end_date],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    let bank_cash_out: f64 = conn.query_row(
        "SELECT COALESCE(SUM(ve.credit_amount), 0) 
         FROM voucher_entries ve
         JOIN vouchers v ON ve.voucher_id = v.id
         WHERE ve.account_id = (SELECT id FROM accounts WHERE code = '1002')
         AND v.date >= ?1 AND v.date <= ?2",
        rusqlite::params![start_date, end_date],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    // 简化：经营活动占80%，投资活动占15%，筹资活动占5%
    let total_net = bank_cash_in - bank_cash_out;
    
    Ok(CashFlowStatement {
        period: format!("{}~{}", start_date, end_date),
        operating_cash_in: bank_cash_in * 0.8,
        operating_cash_out: bank_cash_out * 0.8,
        operating_net: total_net * 0.8,
        investing_cash_in: bank_cash_in * 0.15,
        investing_cash_out: bank_cash_out * 0.15,
        investing_net: total_net * 0.15,
        financing_cash_in: bank_cash_in * 0.05,
        financing_cash_out: bank_cash_out * 0.05,
        financing_net: total_net * 0.05,
        net_increase: total_net,
    })
}
