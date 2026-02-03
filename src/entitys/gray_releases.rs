use chrono::DateTime;
use serde::{Deserialize, Serialize};

/// 创建灰度发布请求
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateGrayReleaseRequest {
    pub name: String,
    pub description: Option<String>,
    pub platform: String,
    pub start_time: DateTime<chrono::Utc>,
    pub end_time: Option<DateTime<chrono::Utc>>,
    pub max_machines: Option<i32>,
    pub priority: Option<i32>,
    pub strategy_type: Option<String>,
    pub strategy_config: Option<serde_json::Value>,
}

/// 更新灰度发布请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateGrayReleaseRequest {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub end_time: Option<DateTime<chrono::Utc>>,
    pub max_machines: Option<i32>,
    pub priority: Option<i32>,
    pub strategy_type: Option<String>,
    pub strategy_config: Option<serde_json::Value>,
}

/// 灰度发布列表响应
#[derive(Debug, Deserialize, Serialize)]
pub struct GrayReleaseListResponse {
    pub total: i64,
    pub list: Vec<crate::dto::gray_releases::GrayRelease>,
}

/// 查询灰度发布参数
#[derive(Debug, Deserialize, Serialize)]
pub struct QueryGrayReleaseParams {
    pub platform: Option<String>,
    pub status: Option<String>,
}

/// 获取灰度发布参数
#[derive(Debug, Deserialize, Serialize)]
pub struct GetGrayReleaseParams {
    pub id: i32,
}

/// 查询灰度发布列表参数（带分页）
#[derive(Debug, Deserialize, Serialize)]
pub struct QueryGrayReleasesParams {
    pub platform: Option<String>,
    pub status: Option<String>,
    pub page_num: Option<i32>,
    pub page_size: Option<i32>,
}
