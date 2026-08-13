use rusqlite::{Connection, Result};
use std::path::PathBuf;

/// 获取数据库文件路径（存放在用户数据目录下）
fn get_db_path() -> PathBuf {
    let mut path = dirs_next::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("small-business-manager");
    std::fs::create_dir_all(&path).ok();
    path.push("data.db");
    path
}

/// 检查表是否存在
fn table_exists(conn: &Connection, name: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
        rusqlite::params![name],
        |row| row.get::<_, i64>(0),
    )
    .map(|count| count > 0)
    .unwrap_or(false)
}

/// 检查列是否存在
fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    let sql = format!("PRAGMA table_info({})", table);
    conn.prepare(&sql)
        .and_then(|mut stmt| {
            let mut rows = stmt.query_map([], |row| {
                row.get::<_, String>(1)
            })?;
            Ok(rows.any(|name| name.as_deref() == Ok(column)))
        })
        .unwrap_or(false)
}

/// 从旧 schema 迁移到新 schema（vouchers 拆分 + invoices 修复）
fn migrate_old_schema(conn: &Connection) -> Result<()> {
    // 检查 vouchers 表是否还有旧 schema 的 debit_account_id 列
    if table_exists(conn, "vouchers") && column_exists(conn, "vouchers", "debit_account_id") {
        conn.execute_batch("DROP TABLE IF EXISTS vouchers")?;
        conn.execute_batch("DROP TABLE IF EXISTS invoices")?;
        conn.execute_batch("DROP TABLE IF EXISTS voucher_entries")?;

        conn.execute_batch(
            "CREATE TABLE vouchers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                voucher_no TEXT NOT NULL UNIQUE,
                date TEXT NOT NULL,
                summary TEXT NOT NULL DEFAULT '',
                attachment TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            )",
        )?;

        conn.execute_batch(
            "CREATE TABLE voucher_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                voucher_id INTEGER NOT NULL,
                account_id INTEGER NOT NULL,
                debit_amount REAL NOT NULL DEFAULT 0,
                credit_amount REAL NOT NULL DEFAULT 0,
                summary TEXT NOT NULL DEFAULT '',
                FOREIGN KEY (voucher_id) REFERENCES vouchers(id) ON DELETE CASCADE,
                FOREIGN KEY (account_id) REFERENCES accounts(id)
            )",
        )?;

        conn.execute_batch(
            "CREATE TABLE invoices (
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
            )",
        )?;

        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_voucher_entries_voucher ON voucher_entries(voucher_id);
             CREATE INDEX IF NOT EXISTS idx_voucher_entries_account ON voucher_entries(account_id);
             CREATE INDEX IF NOT EXISTS idx_vouchers_date ON vouchers(date);
             CREATE INDEX IF NOT EXISTS idx_invoices_type_date ON invoices(invoice_type, date);",
        )?;
    }

    // 为旧 employees 表添加 insurance_base / housing_fund_base 列
    if table_exists(conn, "employees") {
        if !column_exists(conn, "employees", "insurance_base") {
            conn.execute_batch("ALTER TABLE employees ADD COLUMN insurance_base REAL NOT NULL DEFAULT 0")?;
        }
        if !column_exists(conn, "employees", "housing_fund_base") {
            conn.execute_batch("ALTER TABLE employees ADD COLUMN housing_fund_base REAL NOT NULL DEFAULT 0")?;
        }
        if !column_exists(conn, "employees", "grade_id") {
            conn.execute_batch("ALTER TABLE employees ADD COLUMN grade_id INTEGER")?;
        }
    }

    // 新建岗位等级表（依赖 employees 的 grade_id 外键，故放在 employees 迁移之后）
    if !table_exists(conn, "position_grades") {
        conn.execute_batch(
            "CREATE TABLE position_grades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                code TEXT NOT NULL UNIQUE,
                base_salary_min REAL NOT NULL DEFAULT 0,
                base_salary_max REAL NOT NULL DEFAULT 0,
                performance_ratio REAL NOT NULL DEFAULT 0,
                post_coefficient REAL NOT NULL DEFAULT 1.0,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            )",
        )?;
    }

    // 为绩效考核表添加多维评价字段
    if table_exists(conn, "performance_reviews") {
        for col in ["business_score", "growth_score", "attitude_score", "contribution_score"] {
            if !column_exists(conn, "performance_reviews", col) {
                conn.execute_batch(&format!("ALTER TABLE performance_reviews ADD COLUMN {} REAL NOT NULL DEFAULT 0", col))?;
            }
        }
    }

    Ok(())
}

/// 初始化数据库：创建连接并建立所有表
pub fn initialize_database() -> Result<Connection> {
    let db_path = get_db_path();
    let conn = Connection::open(&db_path)?;

    // 启用 WAL 模式提升并发性能
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    // 迁移旧 schema
    migrate_old_schema(&conn)?;

    // 创建所有表
    conn.execute_batch(include_str!("../../sql/create_tables.sql"))?;

    // 插入默认数据（如预置会计科目）
    conn.execute_batch(include_str!("../../sql/seed_data.sql"))?;

    Ok(conn)
}
