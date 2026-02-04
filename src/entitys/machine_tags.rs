use serde::{Deserialize, Serialize};

/// 创建标签请求
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTagRequest {
    pub tag_name: String,
    pub description: Option<String>,
    pub category: Option<String>,
}

/// 标签查询参数
#[derive(Debug, Deserialize)]
pub struct QueryTagsParams {
    pub category: Option<String>,
    pub is_active: Option<bool>,
}

/// 标签列表响应
#[derive(Debug, Deserialize, Serialize)]
pub struct TagListResponse {
    pub total: i64,
    pub list: Vec<crate::dto::machine_tags::MachineTag>,
}

/// 绑定标签到机器用户的请求
#[derive(Debug, Deserialize, Serialize)]
pub struct BindTagsToMachineUserRequest {
    pub machine_user_id: i32,
    pub tag_names: Vec<String>,
}

/// 解绑标签的请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UnbindTagsFromMachineUserRequest {
    pub machine_user_id: i32,
    pub tag_ids: Vec<i32>,
}
