use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 机器标签
#[derive(Debug, Deserialize, Serialize, FromRow, Clone)]
pub struct MachineTag {
    pub id: i32,
    pub tag_name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<chrono::Utc>,
}

/// 机器用户标签关联
#[derive(Debug, Deserialize, Serialize, FromRow, Clone)]
pub struct MachineUserTag {
    pub id: i32,
    pub machine_user_id: i32,
    pub tag_id: i32,
    pub created_at: DateTime<chrono::Utc>,
}

/// 带标签信息的机器用户（关联查询结果）
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MachineUserWithTags {
    pub id: i32,
    pub machine_code: String,
    pub user_uuid: Option<uuid::Uuid>,
    pub platform: Option<String>,
    pub bind_time: DateTime<chrono::Utc>,
    pub status: String,
    pub version_info: Option<serde_json::Value>,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: Option<DateTime<chrono::Utc>>,
    pub tags: Vec<MachineTag>,
}
