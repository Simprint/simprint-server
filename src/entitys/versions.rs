use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 创建版本请求
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateVersionRequest {
    pub type_id: i32,
    pub resource_name: String,
    pub version: String,
    pub name: Option<String>,
    pub notes: Option<String>,
    pub platform: Option<String>,
    pub url: Option<String>,
    pub hash: Option<String>,
    pub signature: Option<String>,
    pub install_path: Option<String>,
    pub file_size: Option<i32>,
    pub pub_date: Option<DateTime<chrono::Utc>>,
}

/// 更新版本请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateVersionRequest {
    pub id: i32,
    pub name: Option<String>,
    pub notes: Option<String>,
    pub platform: Option<String>,
    pub url: Option<String>,
    pub hash: Option<String>,
    pub signature: Option<String>,
    pub install_path: Option<String>,
    pub file_size: Option<i32>,
    pub status: Option<String>,
    pub is_latest: Option<bool>,
}

/// 版本列表响应
#[derive(Debug, Deserialize, Serialize)]
pub struct VersionListResponse {
    pub total: i64,
    pub list: Vec<crate::dto::versions::Version>,
}

/// 查询版本参数
#[derive(Debug, Deserialize, Serialize)]
pub struct QueryVersionParams {
    pub resource_name: Option<String>,
    pub platform: Option<String>,
    pub status: Option<String>,
}

/// 获取版本参数（用于查询字符串）
#[derive(Debug, Deserialize, Serialize)]
pub struct GetVersionParams {
    pub id: Option<i32>,
    pub resource_name: Option<String>,
    pub version: Option<String>,
    pub platform: Option<String>,
}

/// 查询版本列表参数（带分页）
#[derive(Debug, Deserialize, Serialize)]
pub struct QueryVersionsParams {
    pub resource_name: Option<String>,
    pub platform: Option<String>,
    pub status: Option<String>,
    pub page_num: Option<i32>,
    pub page_size: Option<i32>,
}

/// 设置最新版本请求
#[derive(Debug, Deserialize, Serialize)]
pub struct SetLatestVersionRequest {
    pub type_id: i32,
    pub resource_name: String,
    pub version_id: i32,
}

/// 删除版本请求
#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteVersionRequest {
    pub id: i32,
}

/// 版本检查请求参数
#[derive(Debug, Deserialize, Serialize)]
pub struct CheckVersionRequestParam {
    pub machine_code: Option<String>, // 机器码可选，用于灰度判定
}

/// 版本检查响应
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct CheckVersionResponse {
    pub is_gray: bool,
    pub versions: HashMap<String, Vec<crate::dto::versions::Version>>, // key: type_code, value: 版本列表
}
