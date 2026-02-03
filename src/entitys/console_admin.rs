//! Console Gateway 管理员相关请求/响应实体

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 创建管理员请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateConsoleAdminRequest {
    pub user_uuid: Uuid,
}

/// 更新管理员请求
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct UpdateConsoleAdminRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

/// 创建权限请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateConsolePermissionRequest {
    pub route_path: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 授予权限请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GrantConsolePermissionRequest {
    pub admin_id: i32,
    pub permission_id: i32,
}
