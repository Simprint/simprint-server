use serde::{Deserialize, Serialize};

/// 创建版本类型请求
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateVersionTypeRequest {
    pub type_code: String,
    pub type_name: String,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

/// 更新版本类型请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateVersionTypeRequest {
    pub id: i32,
    pub type_name: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

/// 版本类型列表响应
#[derive(Debug, Deserialize, Serialize)]
pub struct VersionTypeListResponse {
    pub list: Vec<crate::dto::version_types::VersionType>,
}

/// 获取版本类型参数
#[derive(Debug, Deserialize, Serialize)]
pub struct GetVersionTypeParams {
    pub id: i32,
}

/// 删除版本类型请求
#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteVersionTypeRequest {
    pub id: i32,
}
