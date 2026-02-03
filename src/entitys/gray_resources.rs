use serde::{Deserialize, Serialize};

/// 创建灰度资源请求
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateGrayResourceRequest {
    pub gray_release_id: i32,
    pub version_id: i32,
    pub sort_order: Option<i32>,
}

/// 批量创建灰度资源请求
#[derive(Debug, Deserialize, Serialize)]
pub struct BatchCreateGrayResourceRequest {
    pub gray_release_id: i32,
    pub version_ids: Vec<i32>,
}

/// 灰度资源列表响应
#[derive(Debug, Deserialize, Serialize)]
pub struct GrayResourceListResponse {
    pub list: Vec<crate::dto::gray_resources::GrayResource>,
}

/// 获取灰度资源参数
#[derive(Debug, Deserialize, Serialize)]
pub struct GetGrayResourceParams {
    pub gray_release_id: i32,
}

/// 移除灰度资源参数
#[derive(Debug, Deserialize, Serialize)]
pub struct RemoveGrayResourceParams {
    pub id: i32,
}
