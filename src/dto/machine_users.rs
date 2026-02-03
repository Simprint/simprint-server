use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 机器用户绑定
#[derive(Debug, Deserialize, Serialize, FromRow, Clone, Default)]
pub struct MachineUser {
    pub id: i32,
    pub machine_code: String,
    pub user_uuid: Option<Uuid>,
    pub platform: Option<String>,
    pub bind_time: DateTime<chrono::Utc>,
    pub status: String,
    pub version_info: Option<serde_json::Value>, // 版本信息（JSON格式）
    pub hardware_hash: Option<String>,           // 硬件哈希
    pub hardware_raw: Option<String>,            // 硬件原始信息
    pub allow: bool,                             // 是否允许使用
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: Option<DateTime<chrono::Utc>>,
}
