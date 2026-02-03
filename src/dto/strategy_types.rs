use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 策略类型
#[derive(Debug, Deserialize, Serialize, FromRow, Clone)]
pub struct StrategyType {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub processor_type: String,
    pub config_schema: Option<String>,
    pub is_active: bool,
}
