-- 预置小微企业常用会计科目（小企业会计准则简化版）
INSERT OR IGNORE INTO accounts (code, name, category, parent_id) VALUES
-- 资产类 (1开头)
('1001', '库存现金', 'asset', NULL),
('1002', '银行存款', 'asset', NULL),
('1122', '应收账款', 'asset', NULL),
('1221', '其他应收款', 'asset', NULL),
('1403', '原材料', 'asset', NULL),
('1405', '库存商品', 'asset', NULL),
('1601', '固定资产', 'asset', NULL),
('1602', '累计折旧', 'asset', NULL),
-- 负债类 (2开头)
('2001', '短期借款', 'liability', NULL),
('2201', '应付票据', 'liability', NULL),
('2202', '应付账款', 'liability', NULL),
('2211', '应付职工薪酬', 'liability', NULL),
('2221', '应交税费', 'liability', NULL),
('2241', '其他应付款', 'liability', NULL),
-- 所有者权益类 (4开头)
('4001', '实收资本', 'equity', NULL),
('4101', '盈余公积', 'equity', NULL),
('4103', '本年利润', 'equity', NULL),
('4104', '利润分配', 'equity', NULL),
-- 收入类 (6开头)
('6001', '主营业务收入', 'income', NULL),
('6051', '其他业务收入', 'income', NULL),
('6111', '投资收益', 'income', NULL),
('6301', '营业外收入', 'income', NULL),
-- 费用类 (7开头)
('6401', '主营业务成本', 'expense', NULL),
('6402', '其他业务成本', 'expense', NULL),
('6601', '销售费用', 'expense', NULL),
('6602', '管理费用', 'expense', NULL),
('6603', '财务费用', 'expense', NULL),
('6711', '营业外支出', 'expense', NULL),
('6801', '所得税费用', 'expense', NULL);

-- 预置税务日历默认提醒规则（仅作为初始化示例，实际日期动态计算）
INSERT OR IGNORE INTO settings (key, value) VALUES
('company_name', ''),
('company_tax_id', ''),
('company_address', ''),
('company_bank', ''),
('company_bank_account', ''),
('taxpayer_type', '一般纳税人'),
('vat_rate', '0.13'),
('cit_rate', '0.25'),
('small_profit_threshold', '3000000'),
-- 社保公积金费率配置（北京地区默认）
('pension_employer_rate', '0.16'),
('pension_employee_rate', '0.08'),
('medical_employer_rate', '0.098'),
('medical_employee_rate', '0.02'),
('medical_employee_fixed', '3'),
('unemployment_employer_rate', '0.005'),
('unemployment_employee_rate', '0.005'),
('injury_employer_rate', '0.002'),
('injury_employee_rate', '0'),
('housing_fund_employer_rate', '0.05'),
('housing_fund_employee_rate', '0.05'),
('insurance_base_min', '7162'),
('insurance_base_max', '35283'),
('housing_fund_base_min', '2420'),
('housing_fund_base_max', '35283');

-- 绩效考评维度权重
INSERT OR IGNORE INTO settings (key, value) VALUES
('review_weight_business', '0.3'),
('review_weight_growth', '0.2'),
('review_weight_collaboration', '0.2'),
('review_weight_attitude', '0.15'),
('review_weight_innovation', '0.15');

-- 预置岗位等级
INSERT OR IGNORE INTO position_grades (name, code, base_salary_min, base_salary_max, performance_ratio, post_coefficient) VALUES
('助理', 'P1', 3000, 6000, 0.15, 0.9),
('专员', 'P2', 5000, 10000, 0.2, 1.0),
('主管', 'M1', 8000, 15000, 0.3, 1.2),
('经理', 'M2', 12000, 25000, 0.35, 1.5);
