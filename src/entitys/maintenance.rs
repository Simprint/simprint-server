use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::dto::maintenance::MaintenanceType;

/// 创建维护请求
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateMaintenanceRequest {
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub maintenance_type: MaintenanceType,
}

/// 获取维护参数
#[derive(Debug, Deserialize, Serialize)]
pub struct GetMaintenanceParams {
    pub id: i64,
}

/// 查询维护列表参数
#[derive(Debug, Deserialize, Serialize)]
pub struct QueryMaintenancesParams {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// 更新维护状态请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateMaintenanceStatusRequest {
    pub id: i64,
    pub status: String,
}

/// 维护列表响应
#[derive(Debug, Deserialize, Serialize)]
pub struct MaintenanceListResponse {
    pub list: Vec<crate::dto::maintenance::Maintenance>,
}

/// 健康检查响应
#[derive(Debug, Deserialize, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
}
