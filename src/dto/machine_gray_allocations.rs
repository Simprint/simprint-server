use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 机器灰度分配 DTO
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MachineGrayAllocation {
    pub id: i32,
    pub machine_code: String,
    pub gray_release_id: i32,
    pub allocated_at: DateTime<chrono::Utc>,
    pub effective_time: Option<DateTime<chrono::Utc>>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<chrono::Utc>,
}
