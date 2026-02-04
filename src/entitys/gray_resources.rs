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

/// 移除灰度资源请求
#[derive(Debug, Deserialize, Serialize)]
pub struct RemoveGrayResourceRequest {
    pub id: i32,
}

/// 更新资源排序请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateResourceSortOrderRequest {
    pub id: i32,
    pub sort_order: i32,
}

/// 查询灰度资源列表请求
#[derive(Debug, Deserialize, Serialize)]
pub struct ListGrayResourcesRequest {
    pub gray_release_id: i32,
}
