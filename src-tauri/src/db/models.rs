use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Employee {
    pub id: i64,
    pub name: String,
    pub phone: String,
    pub position: String,
    pub id_card: String,
    pub hire_date: String,
    pub salary: f64,
    pub insurance_base: f64,
    pub housing_fund_base: f64,
    pub grade_id: Option<i64>,
    pub grade_name: String,
    pub grade_performance_ratio: f64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}
