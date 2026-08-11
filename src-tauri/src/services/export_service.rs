use rust_xlsxwriter::*;
use std::path::Path;
use crate::services::tax_engine;

/// 导出增值税申报表为 Excel
pub fn export_vat_report_to_excel(report: &tax_engine::VatReport, path: &Path) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // 标题格式
    let title_format = Format::new().set_bold().set_font_size(14);
    let header_format = Format::new().set_bold().set_background_color(0xD9E1F2);
    let money_format = Format::new().set_num_format("#,##0.00");

    worksheet.merge_range(0, 0, 0, 3, &format!("增值税申报表 - {} 月", report.period), &title_format)
        .map_err(|e| e.to_string())?;
    
    // 表头
    worksheet.write_with_format(2, 0, "项目", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(2, 1, "金额（元）", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(2, 2, "数量", &header_format).map_err(|e| e.to_string())?;
    
    // 数据行
    worksheet.write(3, 0, "一、销项税额").map_err(|e| e.to_string())?;
    worksheet.write_with_format(3, 1, report.output_tax, &money_format).map_err(|e| e.to_string())?;
    worksheet.write(3, 2, report.output_count).map_err(|e| e.to_string())?;
    
    worksheet.write(4, 0, "  其中：销项不含税金额").map_err(|e| e.to_string())?;
    worksheet.write_with_format(4, 1, report.output_amount, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(5, 0, "二、进项税额").map_err(|e| e.to_string())?;
    worksheet.write_with_format(5, 1, report.input_tax, &money_format).map_err(|e| e.to_string())?;
    worksheet.write(5, 2, report.input_count).map_err(|e| e.to_string())?;
    
    worksheet.write(6, 0, "  其中：进项不含税金额").map_err(|e| e.to_string())?;
    worksheet.write_with_format(6, 1, report.input_amount, &money_format).map_err(|e| e.to_string())?;
    
worksheet.write(7, 0, "三、应缴增值税").map_err(|e| e.to_string())?;
    worksheet.write_with_format(7, 1, report.tax_due, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(8, 0, "四、附加税").map_err(|e| e.to_string())?;
    worksheet.write(9, 0, "  城建税（7%）").map_err(|e| e.to_string())?;
    worksheet.write_with_format(9, 1, report.surcharge_city, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(10, 0, "  教育费附加（3%）").map_err(|e| e.to_string())?;
    worksheet.write_with_format(10, 1, report.surcharge_education, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(11, 0, "  地方教育附加（2%）").map_err(|e| e.to_string())?;
    worksheet.write_with_format(11, 1, report.surcharge_local, &money_format).map_err(|e| e.to_string())?;
    
    // 合计行
    let total_format = Format::new().set_bold().set_background_color(0xE2EFDA);
    worksheet.write_with_format(13, 0, "五、合计应缴税额", &total_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(13, 1, report.total_tax_due, &money_format).map_err(|e| e.to_string())?;
    
    // 税率明细
    worksheet.write(15, 0, "附：税率明细").map_err(|e| e.to_string())?;
    worksheet.write_with_format(16, 0, "税率", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(16, 1, "销项金额", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(16, 2, "销项税额", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(16, 3, "进项金额", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(16, 4, "进项税额", &header_format).map_err(|e| e.to_string())?;
    
    for (row, item) in (17..).zip(report.items.iter()) {
        worksheet.write(row, 0, format!("{:.0}%", item.tax_rate * 100.0)).map_err(|e| e.to_string())?;
        worksheet.write_with_format(row, 1, item.output_amount, &money_format).map_err(|e| e.to_string())?;
        worksheet.write_with_format(row, 2, item.output_tax, &money_format).map_err(|e| e.to_string())?;
        worksheet.write_with_format(row, 3, item.input_amount, &money_format).map_err(|e| e.to_string())?;
        worksheet.write_with_format(row, 4, item.input_tax, &money_format).map_err(|e| e.to_string())?;
    }
    
    // 设置列宽
    worksheet.set_column_width(0, 30).map_err(|e| e.to_string())?;
    worksheet.set_column_width(1, 18).map_err(|e| e.to_string())?;
    worksheet.set_column_width(2, 12).map_err(|e| e.to_string())?;
    worksheet.set_column_width(3, 18).map_err(|e| e.to_string())?;
    worksheet.set_column_width(4, 18).map_err(|e| e.to_string())?;
    
    workbook.save(path).map_err(|e| e.to_string())?;
    Ok(())
}

/// 导出企业所得税季报为 Excel
pub fn export_cit_report_to_excel(report: &tax_engine::CitReport, path: &Path) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    
    let title_format = Format::new().set_bold().set_font_size(14);
    let header_format = Format::new().set_bold().set_background_color(0xD9E1F2);
    let money_format = Format::new().set_num_format("#,##0.00");
    let total_format = Format::new().set_bold().set_background_color(0xE2EFDA);
    
    worksheet.merge_range(0, 0, 0, 1, &format!("企业所得税预缴申报表 - {} 季度", report.period), &title_format)
        .map_err(|e| e.to_string())?;
    
    worksheet.write_with_format(2, 0, "项目", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(2, 1, "金额（元）", &header_format).map_err(|e| e.to_string())?;
    
    worksheet.write(3, 0, "一、营业收入").map_err(|e| e.to_string())?;
    worksheet.write_with_format(3, 1, report.revenue, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(4, 0, "二、营业成本").map_err(|e| e.to_string())?;
    worksheet.write_with_format(4, 1, report.cost, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(5, 0, "三、毛利").map_err(|e| e.to_string())?;
    worksheet.write_with_format(5, 1, report.gross_profit, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(6, 0, "四、期间费用").map_err(|e| e.to_string())?;
    worksheet.write_with_format(6, 1, report.expenses, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(7, 0, "五、利润总额").map_err(|e| e.to_string())?;
    worksheet.write_with_format(7, 1, report.profit, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(8, 0, format!("六、适用税率（{:.1}%）", report.tax_rate * 100.0)).map_err(|e| e.to_string())?;
    worksheet.write(8, 1, report.tax_rate).map_err(|e| e.to_string())?;
    
    if report.is_small_profit {
        worksheet.write(9, 0, "  注：享受小微企业优惠税率").map_err(|e| e.to_string())?;
    }
    
    worksheet.write_with_format(10, 0, "七、应缴企业所得税", &total_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(10, 1, report.tax_due, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.set_column_width(0, 35).map_err(|e| e.to_string())?;
    worksheet.set_column_width(1, 18).map_err(|e| e.to_string())?;
    
    workbook.save(path).map_err(|e| e.to_string())?;
    Ok(())
}

/// 导出利润表为 Excel
pub fn export_income_statement_to_excel(report: &tax_engine::IncomeStatement, path: &Path) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    
    let title_format = Format::new().set_bold().set_font_size(14);
    let header_format = Format::new().set_bold().set_background_color(0xD9E1F2);
    let money_format = Format::new().set_num_format("#,##0.00");
    let total_format = Format::new().set_bold().set_background_color(0xE2EFDA);
    
    worksheet.merge_range(0, 0, 0, 1, &format!("利润表 - {}", report.period), &title_format)
        .map_err(|e| e.to_string())?;
    
    worksheet.write_with_format(2, 0, "项目", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(2, 1, "金额（元）", &header_format).map_err(|e| e.to_string())?;
    
    worksheet.write(3, 0, "一、营业收入").map_err(|e| e.to_string())?;
    worksheet.write_with_format(3, 1, report.revenue, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(4, 0, "减：营业成本").map_err(|e| e.to_string())?;
    worksheet.write_with_format(4, 1, report.cost, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(5, 0, "二、毛利").map_err(|e| e.to_string())?;
    worksheet.write_with_format(5, 1, report.gross_profit, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(6, 0, "减：销售费用").map_err(|e| e.to_string())?;
    worksheet.write_with_format(6, 1, report.selling_expenses, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(7, 0, "减：管理费用").map_err(|e| e.to_string())?;
    worksheet.write_with_format(7, 1, report.admin_expenses, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(8, 0, "减：财务费用").map_err(|e| e.to_string())?;
    worksheet.write_with_format(8, 1, report.finance_expenses, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(9, 0, "三、营业利润").map_err(|e| e.to_string())?;
    worksheet.write_with_format(9, 1, report.operating_profit, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(10, 0, "加：营业外收入").map_err(|e| e.to_string())?;
    worksheet.write_with_format(10, 1, report.non_operating_income, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(11, 0, "减：营业外支出").map_err(|e| e.to_string())?;
    worksheet.write_with_format(11, 1, report.non_operating_expenses, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(12, 0, "四、利润总额").map_err(|e| e.to_string())?;
    worksheet.write_with_format(12, 1, report.profit_before_tax, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write(13, 0, "减：所得税费用").map_err(|e| e.to_string())?;
    worksheet.write_with_format(13, 1, report.income_tax, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.write_with_format(14, 0, "五、净利润", &total_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(14, 1, report.net_profit, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.set_column_width(0, 30).map_err(|e| e.to_string())?;
    worksheet.set_column_width(1, 18).map_err(|e| e.to_string())?;
    
    workbook.save(path).map_err(|e| e.to_string())?;
    Ok(())
}

/// 导出资产负债表为 Excel
pub fn export_balance_sheet_to_excel(report: &tax_engine::BalanceSheet, path: &Path) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    
    let title_format = Format::new().set_bold().set_font_size(14);
    let header_format = Format::new().set_bold().set_background_color(0xD9E1F2);
    let money_format = Format::new().set_num_format("#,##0.00");
    let total_format = Format::new().set_bold().set_background_color(0xE2EFDA);
    
    worksheet.merge_range(0, 0, 0, 3, &format!("资产负债表 - {}", report.period), &title_format)
        .map_err(|e| e.to_string())?;
    
    // 资产列
    worksheet.write_with_format(2, 0, "资产", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(2, 1, "金额（元）", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(2, 2, "负债及所有者权益", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(2, 3, "金额（元）", &header_format).map_err(|e| e.to_string())?;
    
    let max_len = report.assets.len().max(report.liabilities.len()).max(report.equity.len());
    let mut row = 3;
    
    for i in 0..max_len {
        if i < report.assets.len() {
            worksheet.write(row, 0, &report.assets[i].account_name).map_err(|e| e.to_string())?;
            worksheet.write_with_format(row, 1, report.assets[i].balance, &money_format).map_err(|e| e.to_string())?;
        }
        if i < report.liabilities.len() {
            worksheet.write(row, 2, &report.liabilities[i].account_name).map_err(|e| e.to_string())?;
            worksheet.write_with_format(row, 3, report.liabilities[i].balance, &money_format).map_err(|e| e.to_string())?;
        } else if i >= report.liabilities.len() && i < report.liabilities.len() + report.equity.len() {
            let eq_idx = i - report.liabilities.len();
            if eq_idx < report.equity.len() {
                worksheet.write(row, 2, &report.equity[eq_idx].account_name).map_err(|e| e.to_string())?;
                worksheet.write_with_format(row, 3, report.equity[eq_idx].balance, &money_format).map_err(|e| e.to_string())?;
            }
        }
        row += 1;
    }
    
    // 合计行
    row += 1;
    worksheet.write_with_format(row, 0, "资产总计", &total_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(row, 1, report.total_assets, &money_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(row, 2, "负债及所有者权益总计", &total_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(row, 3, report.total_liabilities + report.total_equity, &money_format).map_err(|e| e.to_string())?;
    
    worksheet.set_column_width(0, 25).map_err(|e| e.to_string())?;
    worksheet.set_column_width(1, 18).map_err(|e| e.to_string())?;
    worksheet.set_column_width(2, 25).map_err(|e| e.to_string())?;
    worksheet.set_column_width(3, 18).map_err(|e| e.to_string())?;
    
    workbook.save(path).map_err(|e| e.to_string())?;
    Ok(())
}
