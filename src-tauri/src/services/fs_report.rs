use chrono::{Duration, NaiveDate};
use rusqlite::Connection;
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook};
use std::collections::HashMap;
use std::path::Path;

// ==================== 数据准备 ====================

/// 各科目截至某日的余额（借-贷）
fn balance_map(conn: &Connection, date: &str) -> Result<HashMap<String, f64>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT a.code, COALESCE(SUM(ve.debit_amount) - SUM(ve.credit_amount), 0)
             FROM accounts a
             LEFT JOIN voucher_entries ve ON a.id = ve.account_id
             LEFT JOIN vouchers v ON ve.voucher_id = v.id
             WHERE a.is_active = 1 AND (v.date IS NULL OR v.date <= ?1)
             GROUP BY a.id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([date], |r| Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?)))
        .map_err(|e| e.to_string())?;
    let mut m = HashMap::new();
    for r in rows {
        let (c, v) = r.map_err(|e| e.to_string())?;
        m.insert(c, v);
    }
    Ok(m)
}

/// 各科目在 [start,end] 区间的发生额 (借方, 贷方)
fn flow_map(conn: &Connection, start: &str, end: &str) -> Result<HashMap<String, (f64, f64)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT a.code, COALESCE(SUM(ve.debit_amount),0), COALESCE(SUM(ve.credit_amount),0)
             FROM voucher_entries ve
             JOIN accounts a ON ve.account_id = a.id
             JOIN vouchers v ON ve.voucher_id = v.id
             WHERE v.date >= ?1 AND v.date <= ?2
             GROUP BY a.id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([start, end], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?, r.get::<_, f64>(2)?))
        })
        .map_err(|e| e.to_string())?;
    let mut m = HashMap::new();
    for r in rows {
        let (c, d, cr) = r.map_err(|e| e.to_string())?;
        m.insert(c, (d, cr));
    }
    Ok(m)
}

fn b(m: &HashMap<String, f64>, code: &str) -> f64 {
    *m.get(code).unwrap_or(&0.0)
}
fn bsum(m: &HashMap<String, f64>, codes: &[&str]) -> f64 {
    codes.iter().map(|c| b(m, c)).sum()
}
/// 收入类：贷-借
fn inc(m: &HashMap<String, (f64, f64)>, code: &str) -> f64 {
    m.get(code).map(|x| x.1 - x.0).unwrap_or(0.0)
}
/// 费用类：借-贷
fn exp(m: &HashMap<String, (f64, f64)>, code: &str) -> f64 {
    m.get(code).map(|x| x.0 - x.1).unwrap_or(0.0)
}

/// 未结转损益（收入-费用）累计
fn retained_profit(m: &HashMap<String, f64>) -> f64 {
    let income: f64 = ["6001", "6051", "6111", "6301"].iter().map(|c| -b(m, c)).sum();
    let expense: f64 = ["6401", "6402", "6601", "6602", "6603", "6711", "6801"]
        .iter()
        .map(|c| b(m, c))
        .sum();
    income - expense
}

// ==================== 报表行结构 ====================

pub struct FsLine {
    pub label: String,
    pub line: i64,
    pub cur: f64,
    pub prev: f64,
}

#[derive(Clone, Copy, Default)]
pub struct EquityCols {
    pub paid_in: f64,
    pub capital_reserve: f64,
    pub surplus_reserve: f64,
    pub retained: f64,
}
impl EquityCols {
    fn total(&self) -> f64 {
        self.paid_in + self.capital_reserve + self.surplus_reserve + self.retained
    }
}

#[derive(Clone, Copy, Default)]
struct CashFlow {
    sales_in: f64,
    tax_refund_in: f64,
    other_op_in: f64,
    purchase_out: f64,
    payroll_out: f64,
    tax_out: f64,
    other_op_out: f64,
    invest_in: f64,
    invest_out: f64,
    finance_invest_in: f64, // 吸收投资
    finance_loan_in: f64,   // 取得借款
    finance_out: f64,
    begin_cash: f64,
    end_cash: f64,
}

#[allow(dead_code)]
pub struct QuarterlyReport {
    pub company_name: String,
    pub tax_id: String,
    pub period_label: String,
    pub start: String,
    pub end: String,
    pub prev_start: String,
    pub prev_end: String,
    pub assets: Vec<FsLine>,
    pub liab_equity: Vec<FsLine>,
    pub income: Vec<FsLine>,
    pub cashflow: Vec<FsLine>,
    pub equity_prev: EquityCols,
    pub equity_cur: EquityCols,
    pub equity_net_profit: f64,
}

// ==================== 现金流量表分类 ====================

fn cash_flow(conn: &Connection, start: &str, end: &str) -> Result<CashFlow, String> {
    let mut cf = CashFlow::default();

    let mut vstmt = conn
        .prepare(
            "SELECT DISTINCT v.id FROM vouchers v
             WHERE v.date >= ?1 AND v.date <= ?2
               AND EXISTS(SELECT 1 FROM voucher_entries e JOIN accounts a ON e.account_id=a.id
                          WHERE e.voucher_id=v.id AND a.code IN ('1001','1002'))",
        )
        .map_err(|e| e.to_string())?;
    let vids: Vec<i64> = vstmt
        .query_map([start, end], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for vid in vids {
        let mut estmt = conn
            .prepare("SELECT a.code, e.debit_amount, e.credit_amount FROM voucher_entries e JOIN accounts a ON e.account_id=a.id WHERE e.voucher_id=?1")
            .map_err(|e| e.to_string())?;
        let entries: Vec<(String, f64, f64)> = estmt
            .query_map([vid], |r| Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?, r.get::<_, f64>(2)?)))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        let mut cash = 0.0;
        let mut counters: HashMap<String, (f64, f64)> = HashMap::new();
        for (code, d, c) in &entries {
            if code == "1001" || code == "1002" {
                cash += d - c;
            } else {
                let e = counters.entry(code.clone()).or_insert((0.0, 0.0));
                e.0 += d;
                e.1 += c;
            }
        }
        if cash.abs() < 1e-9 {
            continue;
        }
        let has = |p: &str| counters.keys().any(|k| k.starts_with(p));
        let net = |p: &str| -> f64 {
            counters
                .iter()
                .filter(|(k, _)| k.starts_with(p))
                .map(|(_, (d, c))| d - c)
                .sum()
        };

        if has("4001") {
            cf.finance_invest_in += cash.max(0.0);
        } else if has("2211") {
            cf.payroll_out += (-cash).max(0.0);
        } else if has("2221") || has("6801") {
            if cash > 0.0 {
                cf.tax_refund_in += cash;
            } else {
                cf.tax_out += -cash;
            }
        } else if has("2241") {
            // 其他应付款：贷方=借入（筹资流入），借方=代缴员工款（经营流出）
            if net("2241") < 0.0 && cash > 0.0 {
                cf.finance_loan_in += cash;
            } else if net("2241") > 0.0 && cash < 0.0 {
                cf.payroll_out += -cash;
            } else if cash > 0.0 {
                cf.other_op_in += cash;
            } else {
                cf.other_op_out += -cash;
            }
        } else if has("2001") {
            if cash > 0.0 {
                cf.finance_loan_in += cash;
            } else {
                cf.finance_out += -cash;
            }
        } else if has("1601") {
            if cash > 0.0 {
                cf.invest_in += cash;
            } else {
                cf.invest_out += -cash;
            }
        } else if has("6001") || has("6051") {
            if cash > 0.0 {
                cf.sales_in += cash;
            } else {
                cf.other_op_out += -cash;
            }
        } else if has("6401") || has("6402") {
            if cash > 0.0 {
                cf.sales_in += cash;
            } else {
                cf.purchase_out += -cash;
            }
        } else if cash > 0.0 {
            cf.other_op_in += cash;
        } else {
            cf.other_op_out += -cash;
        }
    }

    // 期初/期末现金
    let day_before = |d: &str| -> String {
        NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map(|x| (x - Duration::days(1)).format("%Y-%m-%d").to_string())
            .unwrap_or_else(|_| d.to_string())
    };
    let b0 = balance_map(conn, &day_before(start))?;
    let b1 = balance_map(conn, end)?;
    cf.begin_cash = b(&b0, "1001") + b(&b0, "1002");
    cf.end_cash = b(&b1, "1001") + b(&b1, "1002");
    Ok(cf)
}

// ==================== 构建报表 ====================

#[allow(clippy::too_many_arguments)]
pub fn build(
    conn: &Connection,
    company_name: &str,
    tax_id: &str,
    period_label: &str,
    start: &str,
    end: &str,
    prev_start: &str,
    prev_end: &str,
) -> Result<QuarterlyReport, String> {
    let be = balance_map(conn, end)?;
    // 资产负债表“上年年末余额”取上一年 12-31
    let end_year: i32 = end.get(0..4).and_then(|s| s.parse().ok()).unwrap_or(0);
    let prev_year_end = format!("{}-12-31", end_year - 1);
    let bp = balance_map(conn, &prev_year_end)?;
    let fc = flow_map(conn, start, end)?;
    let fp = flow_map(conn, prev_start, prev_end)?;

    // ---------- 资产负债表 ----------
    let (cash_e, cash_p) = (bsum(&be, &["1001", "1002"]), bsum(&bp, &["1001", "1002"]));
    let (ar_e, ar_p) = (bsum(&be, &["1122"]), bsum(&bp, &["1122"]));
    let (oar_e, oar_p) = (bsum(&be, &["1221"]), bsum(&bp, &["1221"]));
    let (inv_e, inv_p) = (bsum(&be, &["1403", "1405"]), bsum(&bp, &["1403", "1405"]));
    let (fix_e, fix_p) = (bsum(&be, &["1601", "1602"]), bsum(&bp, &["1601", "1602"]));

    let ca_e = cash_e + ar_e + oar_e + inv_e;
    let ca_p = cash_p + ar_p + oar_p + inv_p;
    let nca_e = fix_e;
    let nca_p = fix_p;

    let sl_e = -bsum(&be, &["2001"]);
    let sl_p = -bsum(&bp, &["2001"]);
    let np_e = -bsum(&be, &["2201"]);
    let np_p = -bsum(&bp, &["2201"]);
    let ap_e = -bsum(&be, &["2202"]);
    let ap_p = -bsum(&bp, &["2202"]);
    let pr_e = -bsum(&be, &["2211"]);
    let pr_p = -bsum(&bp, &["2211"]);
    let tax_e = -bsum(&be, &["2221"]);
    let tax_p = -bsum(&bp, &["2221"]);
    let op_e = -bsum(&be, &["2241"]);
    let op_p = -bsum(&bp, &["2241"]);
    let cl_e = sl_e + np_e + ap_e + pr_e + tax_e + op_e;
    let cl_p = sl_p + np_p + ap_p + pr_p + tax_p + op_p;
    let tl_e = cl_e;
    let tl_p = cl_p;

    let pic_e = -bsum(&be, &["4001"]);
    let pic_p = -bsum(&bp, &["4001"]);
    let sur_e = -bsum(&be, &["4101"]);
    let sur_p = -bsum(&bp, &["4101"]);
    let ret_e = -bsum(&be, &["4103", "4104"]) + retained_profit(&be);
    let ret_p = -bsum(&bp, &["4103", "4104"]) + retained_profit(&bp);
    let te_e = pic_e + sur_e + ret_e;
    let te_p = pic_p + sur_p + ret_p;

    let left: Vec<(&str, i64, f64, f64)> = vec![
        ("流动资产：", 0, 0.0, 0.0),
        ("货币资金", 1, cash_e, cash_p),
        ("交易性金融资产", 2, 0.0, 0.0),
        ("衍生金融资产", 3, 0.0, 0.0),
        ("应收票据", 4, 0.0, 0.0),
        ("应收账款", 5, ar_e, ar_p),
        ("应收款项融资", 6, 0.0, 0.0),
        ("预付款项", 7, 0.0, 0.0),
        ("其他应收款", 8, oar_e, oar_p),
        ("存货", 9, inv_e, inv_p),
        ("合同资产", 10, 0.0, 0.0),
        ("持有待售资产", 11, 0.0, 0.0),
        ("一年内到期的非流动资产", 12, 0.0, 0.0),
        ("其他流动资产", 13, 0.0, 0.0),
        ("流动资产合计", 14, ca_e, ca_p),
        ("非流动资产：", 0, 0.0, 0.0),
        ("债权投资", 15, 0.0, 0.0),
        ("其他债权投资", 16, 0.0, 0.0),
        ("长期应收款", 17, 0.0, 0.0),
        ("长期股权投资", 18, 0.0, 0.0),
        ("其他权益工具投资", 19, 0.0, 0.0),
        ("其他非流动金融资产", 20, 0.0, 0.0),
        ("投资性房地产", 21, 0.0, 0.0),
        ("固定资产", 22, fix_e, fix_p),
        ("在建工程", 23, 0.0, 0.0),
        ("生产性生物资产", 24, 0.0, 0.0),
        ("油气资产", 25, 0.0, 0.0),
        ("使用权资产", 26, 0.0, 0.0),
        ("无形资产", 27, 0.0, 0.0),
        ("开发支出", 28, 0.0, 0.0),
        ("商誉", 29, 0.0, 0.0),
        ("长期待摊费用", 30, 0.0, 0.0),
        ("递延所得税资产", 31, 0.0, 0.0),
        ("其他非流动资产", 32, 0.0, 0.0),
        ("非流动资产合计", 33, nca_e, nca_p),
        ("", 0, 0.0, 0.0),
        ("", 0, 0.0, 0.0),
        ("", 0, 0.0, 0.0),
        ("", 0, 0.0, 0.0),
        ("", 0, 0.0, 0.0),
        ("资产总计", 34, ca_e + nca_e, ca_p + nca_p),
    ];
    let right: Vec<(&str, i64, f64, f64)> = vec![
        ("流动负债：", 0, 0.0, 0.0),
        ("短期借款", 35, sl_e, sl_p),
        ("交易性金融负债", 36, 0.0, 0.0),
        ("衍生金融负债", 37, 0.0, 0.0),
        ("应付票据", 38, np_e, np_p),
        ("应付账款", 39, ap_e, ap_p),
        ("预收款项", 40, 0.0, 0.0),
        ("合同负债", 41, 0.0, 0.0),
        ("应付职工薪酬", 42, pr_e, pr_p),
        ("应交税费", 43, tax_e, tax_p),
        ("其他应付款", 44, op_e, op_p),
        ("持有待售负债", 45, 0.0, 0.0),
        ("一年内到期的非流动负债", 46, 0.0, 0.0),
        ("其他流动负债", 47, 0.0, 0.0),
        ("流动负债合计", 48, cl_e, cl_p),
        ("非流动负债：", 0, 0.0, 0.0),
        ("长期借款", 49, 0.0, 0.0),
        ("应付债券", 50, 0.0, 0.0),
        ("其中：优先股", 51, 0.0, 0.0),
        ("永续债", 52, 0.0, 0.0),
        ("租赁负债", 53, 0.0, 0.0),
        ("长期应付款", 54, 0.0, 0.0),
        ("预计负债", 55, 0.0, 0.0),
        ("递延收益", 56, 0.0, 0.0),
        ("递延所得税负债", 57, 0.0, 0.0),
        ("其他非流动负债", 58, 0.0, 0.0),
        ("非流动负债合计", 59, 0.0, 0.0),
        ("负债合计", 60, tl_e, tl_p),
        ("所有者权益（或股东权益）：", 0, 0.0, 0.0),
        ("实收资本（或股本）", 61, pic_e, pic_p),
        ("其他权益工具", 62, 0.0, 0.0),
        ("其中：优先股", 63, 0.0, 0.0),
        ("永续债", 64, 0.0, 0.0),
        ("资本公积", 65, 0.0, 0.0),
        ("减：库存股", 66, 0.0, 0.0),
        ("其他综合收益", 67, 0.0, 0.0),
        ("专项储备", 68, 0.0, 0.0),
        ("盈余公积", 69, sur_e, sur_p),
        ("未分配利润", 70, ret_e, ret_p),
        ("所有者权益（或股东权益）合计", 71, te_e, te_p),
        ("负债和所有者权益（或股东权益）总计", 72, tl_e + te_e, tl_p + te_p),
    ];
    let assets = left
        .into_iter()
        .map(|(l, n, c, p)| FsLine { label: l.into(), line: n, cur: c, prev: p })
        .collect();
    let liab_equity = right
        .into_iter()
        .map(|(l, n, c, p)| FsLine { label: l.into(), line: n, cur: c, prev: p })
        .collect();

    // ---------- 利润表 ----------
    let rev_c = inc(&fc, "6001") + inc(&fc, "6051");
    let rev_p = inc(&fp, "6001") + inc(&fp, "6051");
    let cost_c = exp(&fc, "6401") + exp(&fc, "6402");
    let cost_p = exp(&fp, "6401") + exp(&fp, "6402");
    let sell_c = exp(&fc, "6601");
    let sell_p = exp(&fp, "6601");
    let adm_c = exp(&fc, "6602");
    let adm_p = exp(&fp, "6602");
    let fin_c = exp(&fc, "6603");
    let fin_p = exp(&fp, "6603");
    let oi_c = inc(&fc, "6111");
    let oi_p = inc(&fp, "6111");
    let noi_c = inc(&fc, "6301");
    let noi_p = inc(&fp, "6301");
    let noe_c = exp(&fc, "6711");
    let noe_p = exp(&fp, "6711");
    let it_c = exp(&fc, "6801");
    let it_p = exp(&fp, "6801");

    let op_c = rev_c - cost_c - sell_c - adm_c - fin_c + oi_c;
    let op_p = rev_p - cost_p - sell_p - adm_p - fin_p + oi_p;
    let tp_c = op_c + noi_c - noe_c;
    let tp_p = op_p + noi_p - noe_p;
    let npr_c = tp_c - it_c;
    let npr_p = tp_p - it_p;

    let income_src: Vec<(&str, f64, f64)> = vec![
        ("一、营业收入", rev_c, rev_p),
        ("减：营业成本", cost_c, cost_p),
        ("税金及附加", 0.0, 0.0),
        ("销售费用", sell_c, sell_p),
        ("管理费用", adm_c, adm_p),
        ("研发费用", 0.0, 0.0),
        ("财务费用", fin_c, fin_p),
        ("其中：利息费用", 0.0, 0.0),
        ("利息收入", 0.0, 0.0),
        ("加：其他收益", 0.0, 0.0),
        ("投资收益（损失以“-”号填列）", oi_c, oi_p),
        ("其中：对联营企业和合营企业的投资收益", 0.0, 0.0),
        ("以摊余成本计量的金融资产终止确认收益（损失以“-”号填列）", 0.0, 0.0),
        ("净敞口套期收益（损失以“-”号填列）", 0.0, 0.0),
        ("公允价值变动收益（损失以“-”号填列）", 0.0, 0.0),
        ("信用减值损失（损失以“-”号填列）", 0.0, 0.0),
        ("资产减值损失（损失以“-”号填列）", 0.0, 0.0),
        ("资产处置收益（损失以“-”号填列）", 0.0, 0.0),
        ("二、营业利润（亏损以“-”号填列）", op_c, op_p),
        ("加：营业外收入", noi_c, noi_p),
        ("减：营业外支出", noe_c, noe_p),
        ("三、利润总额（亏损总额以“-”号填列）", tp_c, tp_p),
        ("减：所得税费用", it_c, it_p),
        ("四、净利润（净亏损以“-”号填列）", npr_c, npr_p),
        ("（一）持续经营净利润（净亏损以“-”号填列）", npr_c, npr_p),
        ("（二）终止经营净利润（净亏损以“-”号填列）", 0.0, 0.0),
        ("五、其他综合收益的税后净额", 0.0, 0.0),
        ("（一）不能重分类进损益的其他综合收益", 0.0, 0.0),
        ("1.重新计量设定受益计划变动额", 0.0, 0.0),
        ("2.权益法下不能转损益的其他综合收益", 0.0, 0.0),
        ("3.其他权益工具投资公允价值变动", 0.0, 0.0),
        ("4.企业自身信用风险公允价值变动", 0.0, 0.0),
        ("（二）将重分类进损益的其他综合收益", 0.0, 0.0),
        ("1.权益法下可转损益的其他综合收益", 0.0, 0.0),
        ("2.其他债权投资公允价值变动", 0.0, 0.0),
        ("3.金融资产重分类计入其他综合收益的金额", 0.0, 0.0),
        ("4.其他债权投资信用减值准备", 0.0, 0.0),
        ("5.现金流量套期储备", 0.0, 0.0),
        ("6.外币财务报表折算差额", 0.0, 0.0),
        ("六、综合收益总额", npr_c, npr_p),
        ("七、每股收益：", 0.0, 0.0),
        ("（一）基本每股收益", 0.0, 0.0),
        ("（二）稀释每股收益", 0.0, 0.0),
    ];
    let income = income_src
        .into_iter()
        .map(|(l, c, p)| FsLine { label: l.into(), line: 0, cur: c, prev: p })
        .collect();

    // ---------- 现金流量表 ----------
    let cfc = cash_flow(conn, start, end)?;
    let cfp = cash_flow(conn, prev_start, prev_end)?;
    let mk = |label: &str, c: f64, p: f64| FsLine { label: label.into(), line: 0, cur: c, prev: p };
    let op_in_c = cfc.sales_in + cfc.tax_refund_in + cfc.other_op_in;
    let op_in_p = cfp.sales_in + cfp.tax_refund_in + cfp.other_op_in;
    let op_out_c = cfc.purchase_out + cfc.payroll_out + cfc.tax_out + cfc.other_op_out;
    let op_out_p = cfp.purchase_out + cfp.payroll_out + cfp.tax_out + cfp.other_op_out;
    let op_net_c = op_in_c - op_out_c;
    let op_net_p = op_in_p - op_out_p;
    let inv_in_c = cfc.invest_in;
    let inv_in_p = cfp.invest_in;
    let inv_out_c = cfc.invest_out;
    let inv_out_p = cfp.invest_out;
    let inv_net_c = inv_in_c - inv_out_c;
    let inv_net_p = inv_in_p - inv_out_p;
    let fin_in_c = cfc.finance_invest_in + cfc.finance_loan_in;
    let fin_in_p = cfp.finance_invest_in + cfp.finance_loan_in;
    let fin_out_c = cfc.finance_out;
    let fin_out_p = cfp.finance_out;
    let fin_net_c = fin_in_c - fin_out_c;
    let fin_net_p = fin_in_p - fin_out_p;
    let net_c = op_net_c + inv_net_c + fin_net_c;
    let net_p = op_net_p + inv_net_p + fin_net_p;

    let cashflow = vec![
        mk("一、经营活动产生的现金流量：", 0.0, 0.0),
        mk("销售商品、提供劳务收到的现金", cfc.sales_in, cfp.sales_in),
        mk("收到的税费返还", cfc.tax_refund_in, cfp.tax_refund_in),
        mk("收到其他与经营活动有关的现金", cfc.other_op_in, cfp.other_op_in),
        mk("经营活动现金流入小计", op_in_c, op_in_p),
        mk("购买商品、接受劳务支付的现金", cfc.purchase_out, cfp.purchase_out),
        mk("支付给职工以及为职工支付的现金", cfc.payroll_out, cfp.payroll_out),
        mk("支付的各项税费", cfc.tax_out, cfp.tax_out),
        mk("支付其他与经营活动有关的现金", cfc.other_op_out, cfp.other_op_out),
        mk("经营活动现金流出小计", op_out_c, op_out_p),
        mk("经营活动产生的现金流量净额", op_net_c, op_net_p),
        mk("二、投资活动产生的现金流量：", 0.0, 0.0),
        mk("收回投资收到的现金", 0.0, 0.0),
        mk("取得投资收益收到的现金", 0.0, 0.0),
        mk("处置固定资产、无形资产和其他长期资产收回的现金净额", 0.0, 0.0),
        mk("处置子公司及其他营业单位收到的现金净额", 0.0, 0.0),
        mk("收到其他与投资活动有关的现金", inv_in_c, inv_in_p),
        mk("投资活动现金流入小计", inv_in_c, inv_in_p),
        mk("购建固定资产、无形资产和其他长期资产支付的现金", inv_out_c, inv_out_p),
        mk("投资支付的现金", 0.0, 0.0),
        mk("取得子公司及其他营业单位支付的现金净额", 0.0, 0.0),
        mk("支付其他与投资活动有关的现金", 0.0, 0.0),
        mk("投资活动现金流出小计", inv_out_c, inv_out_p),
        mk("投资活动产生的现金流量净额", inv_net_c, inv_net_p),
        mk("三、筹资活动产生的现金流量：", 0.0, 0.0),
        mk("吸收投资收到的现金", cfc.finance_invest_in, cfp.finance_invest_in),
        mk("取得借款收到的现金", cfc.finance_loan_in, cfp.finance_loan_in),
        mk("收到其他与筹资活动有关的现金", 0.0, 0.0),
        mk("筹资活动现金流入小计", fin_in_c, fin_in_p),
        mk("偿还债务支付的现金", fin_out_c, fin_out_p),
        mk("分配股利、利润或偿付利息支付的现金", 0.0, 0.0),
        mk("支付其他与筹资活动有关的现金", 0.0, 0.0),
        mk("筹资活动现金流出小计", fin_out_c, fin_out_p),
        mk("筹资活动产生的现金流量净额", fin_net_c, fin_net_p),
        mk("四、汇率变动对现金及现金等价物的影响", 0.0, 0.0),
        mk("五、现金及现金等价物净增加额", net_c, net_p),
        mk("加：期初现金及现金等价物余额", cfc.begin_cash, cfp.begin_cash),
        mk("六、期末现金及现金等价物余额", cfc.end_cash, cfp.end_cash),
    ];

    let r2 = |x: f64| (x * 100.0).round() / 100.0;
    let rnd = |v: Vec<FsLine>| -> Vec<FsLine> {
        v.into_iter()
            .map(|mut l| {
                l.cur = r2(l.cur);
                l.prev = r2(l.prev);
                l
            })
            .collect()
    };

    Ok(QuarterlyReport {
        company_name: company_name.to_string(),
        tax_id: tax_id.to_string(),
        period_label: period_label.to_string(),
        start: start.to_string(),
        end: end.to_string(),
        prev_start: prev_start.to_string(),
        prev_end: prev_end.to_string(),
        assets: rnd(assets),
        liab_equity: rnd(liab_equity),
        income: rnd(income),
        cashflow: rnd(cashflow),
        equity_prev: EquityCols { paid_in: r2(pic_p), capital_reserve: 0.0, surplus_reserve: r2(sur_p), retained: r2(ret_p) },
        equity_cur: EquityCols { paid_in: r2(pic_e), capital_reserve: 0.0, surplus_reserve: r2(sur_e), retained: r2(ret_e) },
        // 本年累计综合收益 = 年末权益 - 上年年末权益（仅损益变动）
        equity_net_profit: r2(te_e - te_p),
    })
}

// ==================== 写 Excel ====================

pub fn write_excel(r: &QuarterlyReport, path: &Path) -> Result<(), String> {
    let mut wb = Workbook::new();
    let title_fmt = Format::new().set_bold().set_font_size(14).set_align(FormatAlign::Center);
    let sub_fmt = Format::new().set_align(FormatAlign::Right);
    let meta_fmt = Format::new().set_bold();
    let head_fmt = Format::new().set_bold().set_align(FormatAlign::Center).set_border(FormatBorder::Thin);
    let cell_fmt = Format::new().set_border(FormatBorder::Thin);
    let num_fmt = Format::new().set_num_format("#,##0.00").set_border(FormatBorder::Thin);
    let center_fmt = Format::new().set_align(FormatAlign::Center).set_border(FormatBorder::Thin);

    // ---------- 资产负债表 ----------
    {
        let ws = wb.add_worksheet();
        ws.set_name("资产负债表").map_err(|e| e.to_string())?;
        ws.merge_range(0, 1, 0, 8, "资产负债表（适用已执行新金融准则、新收入准则和新租赁准则的一般企业）", &title_fmt)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(1, 8, "会企01表   单位：元", &sub_fmt).map_err(|e| e.to_string())?;
        ws.write_with_format(2, 1, "纳税人识别号", &meta_fmt).map_err(|e| e.to_string())?;
        ws.write(2, 2, &r.tax_id).map_err(|e| e.to_string())?;
        ws.write_with_format(2, 5, "纳税人名称", &meta_fmt).map_err(|e| e.to_string())?;
        ws.write(2, 6, &r.company_name).map_err(|e| e.to_string())?;
        ws.write_with_format(3, 1, "所属期起", &meta_fmt).map_err(|e| e.to_string())?;
        ws.write(3, 2, &r.start).map_err(|e| e.to_string())?;
        ws.write_with_format(3, 5, "所属期止", &meta_fmt).map_err(|e| e.to_string())?;
        ws.write(3, 6, &r.end).map_err(|e| e.to_string())?;

        let heads = ["资产", "行次", "期末余额", "上年年末余额", "负债和所有者权益（或股东权益）", "行次", "期末余额", "上年年末余额"];
        for (i, h) in heads.iter().enumerate() {
            ws.write_with_format(4, i as u16 + 1, *h, &head_fmt).map_err(|e| e.to_string())?;
        }
        for (idx, (a, l)) in r.assets.iter().zip(r.liab_equity.iter()).enumerate() {
            let row = (5 + idx) as u32;
            ws.write_with_format(row, 1, &a.label, &cell_fmt).map_err(|e| e.to_string())?;
            if a.line > 0 {
                ws.write_with_format(row, 2, a.line, &center_fmt).map_err(|e| e.to_string())?;
            }
            ws.write_with_format(row, 3, a.cur, &num_fmt).map_err(|e| e.to_string())?;
            ws.write_with_format(row, 4, a.prev, &num_fmt).map_err(|e| e.to_string())?;
            ws.write_with_format(row, 5, &l.label, &cell_fmt).map_err(|e| e.to_string())?;
            if l.line > 0 {
                ws.write_with_format(row, 6, l.line, &center_fmt).map_err(|e| e.to_string())?;
            }
            ws.write_with_format(row, 7, l.cur, &num_fmt).map_err(|e| e.to_string())?;
            ws.write_with_format(row, 8, l.prev, &num_fmt).map_err(|e| e.to_string())?;
        }
        ws.set_column_width(1, 32).map_err(|e| e.to_string())?;
        ws.set_column_width(2, 6).map_err(|e| e.to_string())?;
        ws.set_column_width(3, 16).map_err(|e| e.to_string())?;
        ws.set_column_width(4, 16).map_err(|e| e.to_string())?;
        ws.set_column_width(5, 32).map_err(|e| e.to_string())?;
        ws.set_column_width(6, 6).map_err(|e| e.to_string())?;
        ws.set_column_width(7, 16).map_err(|e| e.to_string())?;
        ws.set_column_width(8, 16).map_err(|e| e.to_string())?;
    }

    // ---------- 利润表 / 现金流量表 ----------
    write_amount_sheet(
        &mut wb,
        "利润表",
        "利润表（适用已执行新金融准则、新收入准则和新租赁准则的一般企业）",
        "会企02表   单位：元",
        r,
        &r.income,
        &title_fmt,
        &sub_fmt,
        &meta_fmt,
        &head_fmt,
        &cell_fmt,
        &num_fmt,
    )?;
    write_amount_sheet(
        &mut wb,
        "现金流量表",
        "现金流量表（适用执行企业会计准则的一般企业）",
        "会企03表   单位：元",
        r,
        &r.cashflow,
        &title_fmt,
        &sub_fmt,
        &meta_fmt,
        &head_fmt,
        &cell_fmt,
        &num_fmt,
    )?;

    // ---------- 所有者权益变动表 ----------
    write_equity(&mut wb, r, &title_fmt, &sub_fmt, &meta_fmt, &head_fmt, &cell_fmt, &num_fmt)?;

    // ---------- 附注 ----------
    {
        let ws = wb.add_worksheet();
        ws.set_name("企业会计准则附注").map_err(|e| e.to_string())?;
        ws.merge_range(0, 1, 0, 7, "企业会计准则附注", &title_fmt).map_err(|e| e.to_string())?;
        ws.write_with_format(2, 1, "（一）企业的基本情况", &meta_fmt).map_err(|e| e.to_string())?;
        ws.write_with_format(7, 1, "（二）财务报表的编制基础", &meta_fmt).map_err(|e| e.to_string())?;
        ws.write_with_format(12, 1, "（三）遵循企业会计准则的声明", &meta_fmt).map_err(|e| e.to_string())?;
        ws.write_with_format(17, 1, "（四）重要会计政策和会计估计", &meta_fmt).map_err(|e| e.to_string())?;
        ws.write_with_format(22, 1, "（五）会计政策和会计估计变更以及差错更正的说明", &meta_fmt).map_err(|e| e.to_string())?;
        ws.write_with_format(27, 1, "（六）报表重要项目的说明", &meta_fmt).map_err(|e| e.to_string())?;
        ws.write_with_format(32, 1, "（七）或有和承诺事项、资产负债表日后非调整事项、关联方关系及其交易等需要说明的事项", &meta_fmt).map_err(|e| e.to_string())?;
        ws.write_with_format(37, 1, "（八）有助于财务报表使用者评价企业管理资本的目标、政策及程序的信息", &meta_fmt).map_err(|e| e.to_string())?;
        ws.set_column_width(1, 80).map_err(|e| e.to_string())?;
    }

    // 空表：一般企业_已执行
    {
        let ws = wb.add_worksheet();
        ws.set_name("一般企业_已执行").map_err(|e| e.to_string())?;
    }

    wb.save(path).map_err(|e| e.to_string())?;
    Ok(())
}

fn write_meta(
    ws: &mut rust_xlsxwriter::Worksheet,
    r: &QuarterlyReport,
    form_label: &str,
    meta_fmt: &Format,
    sub_fmt: &Format,
    cols: u16,
) -> Result<(), String> {
    ws.write_with_format(1, cols, form_label, sub_fmt).map_err(|e| e.to_string())?;
    ws.write_with_format(2, 1, "纳税人识别号", meta_fmt).map_err(|e| e.to_string())?;
    ws.write(2, 2, &r.tax_id).map_err(|e| e.to_string())?;
    ws.write_with_format(2, 3, "纳税人名称", meta_fmt).map_err(|e| e.to_string())?;
    ws.write(2, 4, &r.company_name).map_err(|e| e.to_string())?;
    ws.write_with_format(3, 1, "所属期起", meta_fmt).map_err(|e| e.to_string())?;
    ws.write(3, 2, &r.start).map_err(|e| e.to_string())?;
    ws.write_with_format(3, 3, "所属期止", meta_fmt).map_err(|e| e.to_string())?;
    ws.write(3, 4, &r.end).map_err(|e| e.to_string())?;
    Ok(())
}

/// 利润表 / 现金流量表：B=项目，C=本期金额，D:E 合并=上期金额（无“行次”列）
#[allow(clippy::too_many_arguments)]
fn write_amount_sheet(
    wb: &mut Workbook,
    sheet_name: &str,
    title: &str,
    form_label: &str,
    r: &QuarterlyReport,
    lines: &[FsLine],
    title_fmt: &Format,
    sub_fmt: &Format,
    meta_fmt: &Format,
    head_fmt: &Format,
    cell_fmt: &Format,
    num_fmt: &Format,
) -> Result<(), String> {
    let ws = wb.add_worksheet();
    ws.set_name(sheet_name).map_err(|e| e.to_string())?;
    ws.merge_range(0, 1, 0, 4, title, title_fmt).map_err(|e| e.to_string())?;
    write_meta(ws, r, form_label, meta_fmt, sub_fmt, 4)?;
    ws.write_with_format(4, 1, "项目", head_fmt).map_err(|e| e.to_string())?;
    ws.write_with_format(4, 2, "本期金额", head_fmt).map_err(|e| e.to_string())?;
    ws.merge_range(4, 3, 4, 4, "上期金额", head_fmt).map_err(|e| e.to_string())?;
    for (i, line) in lines.iter().enumerate() {
        let row = (5 + i) as u32;
        ws.write_with_format(row, 1, &line.label, cell_fmt).map_err(|e| e.to_string())?;
        // 合并 D:E（上期金额）
        ws.merge_range(row, 3, row, 4, "", cell_fmt).map_err(|e| e.to_string())?;
        if !line.label.ends_with('：') {
            ws.write_with_format(row, 2, line.cur, num_fmt).map_err(|e| e.to_string())?;
            // 合并区左上角写入数值（保持数值类型）
            ws.write_with_format(row, 3, line.prev, num_fmt).map_err(|e| e.to_string())?;
        }
    }
    ws.set_column_width(1, 44).map_err(|e| e.to_string())?;
    ws.set_column_width(2, 18).map_err(|e| e.to_string())?;
    ws.set_column_width(3, 15).map_err(|e| e.to_string())?;
    ws.set_column_width(4, 15).map_err(|e| e.to_string())?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_equity(
    wb: &mut Workbook,
    r: &QuarterlyReport,
    title_fmt: &Format,
    sub_fmt: &Format,
    meta_fmt: &Format,
    head_fmt: &Format,
    cell_fmt: &Format,
    num_fmt: &Format,
) -> Result<(), String> {
    let ws = wb.add_worksheet();
    ws.set_name("所有者权益变动表").map_err(|e| e.to_string())?;
    ws.merge_range(0, 1, 0, 23, "所有者权益变动表（适用执行企业会计准则的一般企业）", title_fmt)
        .map_err(|e| e.to_string())?;
    ws.write_with_format(1, 23, "会企04表   单位：元", sub_fmt).map_err(|e| e.to_string())?;
    ws.write_with_format(2, 1, "纳税人识别号", meta_fmt).map_err(|e| e.to_string())?;
    ws.write(2, 2, &r.tax_id).map_err(|e| e.to_string())?;
    ws.write_with_format(2, 3, "纳税人名称", meta_fmt).map_err(|e| e.to_string())?;
    ws.write(2, 4, &r.company_name).map_err(|e| e.to_string())?;
    ws.write_with_format(2, 6, "所属期起", meta_fmt).map_err(|e| e.to_string())?;
    ws.write(2, 7, &r.start).map_err(|e| e.to_string())?;
    ws.write_with_format(2, 8, "所属期止", meta_fmt).map_err(|e| e.to_string())?;
    ws.write(2, 9, &r.end).map_err(|e| e.to_string())?;

    // 列：本年 C=实收资本, G=资本公积, K=盈余公积, L=未分配利润, M=合计
    ws.write_with_format(4, 2, "实收资本(或股本)", head_fmt).map_err(|e| e.to_string())?;
    ws.write_with_format(4, 6, "资本公积", head_fmt).map_err(|e| e.to_string())?;
    ws.write_with_format(4, 10, "盈余公积", head_fmt).map_err(|e| e.to_string())?;
    ws.write_with_format(4, 11, "未分配利润", head_fmt).map_err(|e| e.to_string())?;
    ws.write_with_format(4, 12, "所有者权益合计", head_fmt).map_err(|e| e.to_string())?;

    let write_row = |ws: &mut rust_xlsxwriter::Worksheet, row: u32, label: &str, e: &EquityCols, net: f64| -> Result<(), String> {
        ws.write_with_format(row, 1, label, cell_fmt).map_err(|er| er.to_string())?;
        ws.write_with_format(row, 2, e.paid_in, num_fmt).map_err(|er| er.to_string())?;
        ws.write_with_format(row, 6, e.capital_reserve, num_fmt).map_err(|er| er.to_string())?;
        ws.write_with_format(row, 10, e.surplus_reserve, num_fmt).map_err(|er| er.to_string())?;
        ws.write_with_format(row, 11, e.retained + net, num_fmt).map_err(|er| er.to_string())?;
        ws.write_with_format(row, 12, e.total() + net, num_fmt).map_err(|er| er.to_string())?;
        Ok(())
    };
    // 行7 上年年末余额(idx6)
    write_row(ws, 6, "一、上年年末余额", &r.equity_prev, 0.0)?;
    // 行13 综合收益总额(idx12)：仅损益变动列（未分配利润/合计）
    write_row(ws, 12, "（一）综合收益总额", &EquityCols::default(), r.equity_net_profit)?;
    // 行30 本年年末余额(idx29)
    write_row(ws, 29, "四、本年年末余额", &r.equity_cur, 0.0)?;

    ws.set_column_width(1, 34).map_err(|e| e.to_string())?;
    for c in 2..=12u16 {
        ws.set_column_width(c, 14).map_err(|e| e.to_string())?;
    }
    Ok(())
}
