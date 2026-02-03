use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// API 密钥 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ApiKeyDto {
    pub id: i32,
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub name: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub permissions: serde_json::Value,
    pub rate_limit: Option<i32>,
    pub daily_limit: Option<i32>,
    pub ip_whitelist: Option<serde_json::Value>,
    pub expires_at: Option<DateTime<Utc>>,
    pub usage_count: Option<i64>,
    pub daily_usage: Option<i32>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 用户偏好设置 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserPreferenceDto {
    pub id: i32,
    pub user_uuid: Uuid,
    pub theme: String,
    pub language: String,
    pub notifications_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 系统配置 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct SystemConfigDto {
    pub id: i32,
    pub config_key: String,
    pub config_value: serde_json::Value,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
