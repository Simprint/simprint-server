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
    pub version_ids: Vec<i32>,
    pub auto_start: bool,
}

/// 启动灰度发布请求
#[derive(Debug, Deserialize, Serialize)]
pub struct StartGrayReleaseRequest {
    pub gray_release_id: i32,
}

/// 停止灰度发布请求
#[derive(Debug, Deserialize, Serialize)]
pub struct StopGrayReleaseRequest {
    pub gray_release_id: i32,
}

/// 获取灰度发布详情请求
#[derive(Debug, Deserialize, Serialize)]
pub struct GetGrayReleaseRequest {
    pub gray_release_id: i32,
}

/// 查询灰度发布列表请求
#[derive(Debug, Deserialize, Serialize)]
pub struct ListGrayReleasesRequest {
    pub platform: Option<String>,
    pub status: Option<String>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// 灰度发布列表响应
#[derive(Debug, Deserialize, Serialize)]
pub struct GrayReleaseListResponse {
    pub total: i64,
    pub list: Vec<crate::dto::gray_releases::GrayRelease>,
}

/// 更新灰度发布请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateGrayReleaseRequest {
    pub gray_release_id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub end_time: Option<DateTime<chrono::Utc>>,
    pub max_machines: Option<i32>,
    pub priority: Option<i32>,
    pub strategy_type: Option<String>,
    pub strategy_config: Option<serde_json::Value>,
}

/// 删除灰度发布请求
#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteGrayReleaseRequest {
    pub gray_release_id: i32,
}
