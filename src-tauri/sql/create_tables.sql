-- 人力资源表
CREATE TABLE IF NOT EXISTS employees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    phone TEXT NOT NULL,
    position TEXT NOT NULL DEFAULT '',
    id_card TEXT NOT NULL DEFAULT '',
    hire_date TEXT NOT NULL,
    salary REAL NOT NULL DEFAULT 0,
    insurance_base REAL NOT NULL DEFAULT 0,
    housing_fund_base REAL NOT NULL DEFAULT 0,
    grade_id INTEGER,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    FOREIGN KEY (grade_id) REFERENCES position_grades(id)
);

CREATE TABLE IF NOT EXISTS position_grades (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    code TEXT NOT NULL UNIQUE,
    base_salary_min REAL NOT NULL DEFAULT 0,
    base_salary_max REAL NOT NULL DEFAULT 0,
    performance_ratio REAL NOT NULL DEFAULT 0,
    post_coefficient REAL NOT NULL DEFAULT 1.0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

CREATE TABLE IF NOT EXISTS performance_reviews (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    period TEXT NOT NULL,
    score REAL NOT NULL DEFAULT 0,
    grade TEXT NOT NULL DEFAULT '',
    coefficient REAL NOT NULL DEFAULT 1.0,
    business_score REAL NOT NULL DEFAULT 0,
    growth_score REAL NOT NULL DEFAULT 0,
    collaboration_score REAL NOT NULL DEFAULT 0,
    attitude_score REAL NOT NULL DEFAULT 0,
    innovation_score REAL NOT NULL DEFAULT 0,
    reviewer TEXT NOT NULL DEFAULT '',
    remark TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    FOREIGN KEY (employee_id) REFERENCES employees(id),
    UNIQUE(employee_id, period)
);

CREATE TABLE IF NOT EXISTS employee_bonuses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    period TEXT NOT NULL,
    bonus_type TEXT NOT NULL,
    amount REAL NOT NULL DEFAULT 0,
    remark TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    FOREIGN KEY (employee_id) REFERENCES employees(id)
);

CREATE TABLE IF NOT EXISTS performance_targets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    period TEXT NOT NULL,
    target_amount REAL NOT NULL DEFAULT 0,
    achieved_amount REAL NOT NULL DEFAULT 0,
    commission_rate REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    FOREIGN KEY (employee_id) REFERENCES employees(id),
    UNIQUE(employee_id, period)
);

CREATE TABLE IF NOT EXISTS attendance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    date TEXT NOT NULL,
    check_in TEXT,
    check_out TEXT,
    status TEXT NOT NULL DEFAULT 'present',
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    FOREIGN KEY (employee_id) REFERENCES employees(id),
    UNIQUE(employee_id, date)
);

-- 财务管理表
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    category TEXT NOT NULL CHECK(category IN ('asset','liability','equity','income','expense')),
    parent_id INTEGER,
    is_active INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (parent_id) REFERENCES accounts(id)
);

CREATE TABLE IF NOT EXISTS vouchers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    voucher_no TEXT NOT NULL UNIQUE,
    date TEXT NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    attachment TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

CREATE TABLE IF NOT EXISTS voucher_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    voucher_id INTEGER NOT NULL,
    account_id INTEGER NOT NULL,
    debit_amount REAL NOT NULL DEFAULT 0,
    credit_amount REAL NOT NULL DEFAULT 0,
    summary TEXT NOT NULL DEFAULT '',
    FOREIGN KEY (voucher_id) REFERENCES vouchers(id) ON DELETE CASCADE,
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);

CREATE TABLE IF NOT EXISTS invoices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_type TEXT NOT NULL CHECK(invoice_type IN ('input','output')),
    invoice_no TEXT NOT NULL,
    date TEXT NOT NULL,
    buyer_name TEXT NOT NULL,
    buyer_tax_id TEXT NOT NULL,
    amount REAL NOT NULL,
    tax_rate REAL NOT NULL,
    tax_amount REAL NOT NULL,
    total_amount REAL NOT NULL,
    is_verified INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

CREATE TABLE IF NOT EXISTS tax_reports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    report_type TEXT NOT NULL,
    period TEXT NOT NULL,
    data TEXT NOT NULL DEFAULT '{}',
    tax_amount REAL NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'draft',
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    UNIQUE(report_type, period)
);

CREATE TABLE IF NOT EXISTS tax_calendar (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tax_type TEXT NOT NULL,
    period TEXT NOT NULL,
    due_date TEXT NOT NULL,
    is_reminded INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    UNIQUE(tax_type, period)
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

CREATE TABLE IF NOT EXISTS custom_tax_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    frequency TEXT NOT NULL DEFAULT 'annual',
    due_month INTEGER,
    due_day INTEGER NOT NULL DEFAULT 15,
    once_date TEXT,
    remark TEXT NOT NULL DEFAULT '',
    is_enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

CREATE TABLE IF NOT EXISTS custom_tax_item_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id INTEGER NOT NULL,
    period TEXT NOT NULL,
    submitted INTEGER NOT NULL DEFAULT 0,
    submitted_at TEXT,
    UNIQUE(item_id, period),
    FOREIGN KEY (item_id) REFERENCES custom_tax_items(id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_attendance_employee_date ON attendance(employee_id, date);
CREATE INDEX IF NOT EXISTS idx_vouchers_date ON vouchers(date);
CREATE INDEX IF NOT EXISTS idx_voucher_entries_voucher ON voucher_entries(voucher_id);
CREATE INDEX IF NOT EXISTS idx_voucher_entries_account ON voucher_entries(account_id);
CREATE INDEX IF NOT EXISTS idx_invoices_type_date ON invoices(invoice_type, date);
CREATE INDEX IF NOT EXISTS idx_tax_reports_type_period ON tax_reports(report_type, period);

-- 智能记账模块
CREATE TABLE IF NOT EXISTS smart_entitlement (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    license_token TEXT,
    payload_json TEXT,
    activated_at TEXT,
    updates_until TEXT
);
CREATE TABLE IF NOT EXISTS smart_scenarios (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scenario_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    definition TEXT NOT NULL,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    priority INTEGER NOT NULL DEFAULT 100,
    created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
);
CREATE TABLE IF NOT EXISTS smart_learning_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    raw_text TEXT NOT NULL,
    scenario_id TEXT,
    entries TEXT NOT NULL,
    adopted INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
);
