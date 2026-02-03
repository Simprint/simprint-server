//! Console Gateway 管理员和权限 DTO

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// 管理员 DTO
#[derive(Debug, Clone, FromRow)]
pub struct ConsoleAdminDto {
    pub id: i32,
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// 权限 DTO
#[derive(Debug, Clone, FromRow)]
pub struct ConsolePermissionDto {
    pub id: i32,
    pub uuid: Uuid,
    pub route_path: String,
    pub method: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// 管理员权限关联 DTO
#[derive(Debug, Clone, FromRow)]
pub struct ConsoleAdminPermissionDto {
    pub id: i32,
    pub uuid: Uuid,
    pub admin_id: i32,
    pub permission_id: i32,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<Uuid>,
}
