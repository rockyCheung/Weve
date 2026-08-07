# 更新日志

## v1.0.0 (2025-01-01)

### 新增功能
- 🎉 首次发布
- 👥 员工花名册管理（增删改查、批量导入）
- 📅 考勤打卡（签到/签退、考勤查询）
- 💰 工资表生成（自动计算）
- 📒 记账管理（科目管理、凭证录入、试算平衡）
- 📄 发票管理（销项/进项、认证管理）
- 📊 增值税月报自动生成
- 📊 企业所得税季报（含小微优惠）
- 📊 个人所得税代扣代缴汇总
- 📈 三大财务报表（资产负债表、利润表、现金流量表）
- 📆 税务日历与到期提醒
- 📋 数据仪表盘
- ⚙️ 系统设置（企业信息、税务参数）
- 💾 数据备份与恢复
- 📤 全部数据导出
# 1. 开发模式
npm run tauri dev

# 2. 生产构建
npm run tauri build

# 3. 仅构建前端
npm run build

# 4. 清理构建缓存
npm run clean
# 或手动删除: rm -rf src-tauri/target dist node_modules
small-business-manager/
├── package.json
├── tsconfig.json
├── tsconfig.node.json
├── vite.config.ts
├── .gitignore
├── README.md
├── CHANGELOG.md
├── scripts/
│   ├── build.sh
│   └── build.bat
├── .github/
│   └── workflows/
│       └── release.yml
├── src/
│   ├── main.ts
│   ├── App.vue
│   ├── env.d.ts
│   ├── router/
│   │   └── index.ts
│   ├── views/
│   │   ├── dashboard/
│   │   │   └── Index.vue
│   │   ├── hr/
│   │   │   ├── Index.vue
│   │   │   ├── EmployeeList.vue
│   │   │   ├── Attendance.vue
│   │   │   └── Payroll.vue
│   │   ├── finance/
│   │   │   ├── Index.vue
│   │   │   ├── Accounting.vue
│   │   │   ├── Invoices.vue
│   │   │   ├── TaxReports.vue
│   │   │   └── TaxCalendar.vue
│   │   └── settings/
│   │       └── Index.vue
│   └── api/
│       ├── hr.ts
│       ├── finance.ts
│       └── types.ts
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/
│   │   ├── 32x32.png
│   │   ├── 128x128.png
│   │   ├── 128x128@2x.png
│   │   ├── icon.icns
│   │   └── icon.ico
│   ├── sql/
│   │   ├── create_tables.sql
│   │   └── seed_data.sql
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── db/
│       │   ├── mod.rs
│       │   ├── init.rs
│       │   └── models.rs
│       ├── commands/
│       │   ├── mod.rs
│       │   ├── hr.rs
│       │   ├── accounting.rs
│       │   ├── invoice.rs
│       │   ├── tax.rs
│       │   ├── tax_calendar.rs
│       │   ├── dashboard.rs
│       │   └── settings.rs
│       └── services/
│           ├── mod.rs
│           ├── tax_engine.rs
│           └── export_service.rs