use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 灰度发布
#[derive(Debug, Deserialize, Serialize, FromRow, Clone, Default)]
pub struct GrayRelease {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub platform: String,
    pub status: String,
    pub start_time: DateTime<chrono::Utc>,
    pub end_time: Option<DateTime<chrono::Utc>>,
    pub max_machines: Option<i32>,
    pub allocated_count: i32,
    pub priority: i32,
    pub strategy_type: Option<String>,
    pub strategy_config: Option<serde_json::Value>,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: Option<DateTime<chrono::Utc>>,
}
