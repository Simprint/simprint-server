//! Console Gateway API 密钥 DTO

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// API 密钥 DTO
#[derive(Debug, Clone, FromRow)]
pub struct ConsoleApiKeyDto {
    pub id: i32,
    pub uuid: Uuid,
    pub key_id: String,
    pub key_secret: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub deleted_at: Option<DateTime<Utc>>,
}
