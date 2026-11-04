mod db;
mod commands;
mod services;

use db::init::initialize_database;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let conn = initialize_database().expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState {
            db: Mutex::new(conn),
        })
        .invoke_handler(tauri::generate_handler![
            // HR 命令
            commands::hr::create_employee,
            commands::hr::get_all_employees,
            commands::hr::update_employee,
            commands::hr::delete_employee,
            commands::hr::record_attendance,
            commands::hr::query_attendance,
            commands::hr::generate_payroll,
            commands::hr::generate_payroll_voucher,
            commands::hr::get_position_grades,
            commands::hr::create_position_grade,
            commands::hr::update_position_grade,
            commands::hr::delete_position_grade,
            commands::hr::save_performance_review,
            commands::hr::get_performance_reviews,
            commands::hr::save_employee_bonus,
            commands::hr::get_employee_bonuses,
            commands::hr::delete_employee_bonus,
            commands::hr::save_performance_target,
            commands::hr::get_performance_targets,
            commands::hr::force_delete_employee,
            commands::hr::get_employee_data_summary,
            // 记账命令
            commands::accounting::create_account,
            commands::accounting::get_account_tree,
            commands::accounting::update_account,
            commands::accounting::delete_account,
            commands::accounting::post_voucher,
            // 智能记账
            commands::smart_voucher::smart_module_status,
            commands::smart_voucher::smart_activate_code,
            commands::smart_voucher::smart_scenarios_list,
            commands::smart_voucher::smart_scenario_save,
            commands::smart_voucher::smart_scenario_delete,
            commands::smart_voucher::smart_parse,
            commands::smart_voucher::smart_commit,
            commands::smart_voucher::smart_rulepack_status,
            commands::smart_voucher::smart_rulepack_update,
            commands::smart_voucher::smart_llm_get_config,
            commands::smart_voucher::smart_llm_save_config,
            commands::accounting::get_vouchers,
            commands::accounting::delete_voucher,
            commands::accounting::trial_balance,
            commands::accounting::import_bank_statement,
            commands::accounting::save_voucher_attachment,
            commands::accounting::get_attachment_full_path,
            // 发票命令
            commands::invoice::add_invoice,
            commands::invoice::query_invoices,
            commands::invoice::get_invoice_summary,
            commands::invoice::verify_invoice,
            commands::invoice::delete_invoice,
            commands::invoice::get_invoice_due_reminders,
            // 税务命令
            commands::tax::generate_tax_report,
            commands::tax::get_saved_reports,
            commands::tax::export_report,
            commands::tax::export_full_statement,
            // 税务日历命令
            commands::tax_calendar::generate_tax_calendar,
            commands::tax_calendar::get_tax_calendar,
            commands::tax_calendar::mark_as_reminded,
            commands::tax_calendar::mark_as_submitted,
            commands::tax_calendar::check_upcoming_deadlines,
            commands::tax_calendar::get_custom_tax_items,
            commands::tax_calendar::create_custom_tax_item,
            commands::tax_calendar::update_custom_tax_item,
            commands::tax_calendar::delete_custom_tax_item,
            commands::tax_calendar::mark_custom_item_submitted,
            // 仪表盘命令
            commands::dashboard::get_dashboard_stats,
            commands::dashboard::get_monthly_trend,
            commands::dashboard::get_recent_activities,
            // 设置命令
            commands::settings::save_setting,
            commands::settings::save_settings_batch,
            commands::settings::get_setting,
            commands::settings::get_all_settings,
            commands::settings::backup_database,
            commands::settings::restore_database,
            commands::settings::list_backups,
            commands::settings::delete_backup,
            commands::settings::export_all_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
