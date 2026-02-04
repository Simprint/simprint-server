use serde::{Deserialize, Serialize};

use uuid::Uuid;

/// 创建/绑定机器用户请求
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateMachineUserRequest {
    pub machine_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_uuid: Option<Uuid>,
    pub platform: Option<String>,
    /// 标签列表（可选，注册成功后会绑定到机器）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub hardware_hash: Option<String>, // 硬件哈希
    pub hardware_raw: Option<String>,  // 硬件原始信息
    pub version_info: Option<String>,  // JSON格式的版本信息
}

/// 更新机器用户请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateMachineUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_uuid: Option<Uuid>,
    pub platform: Option<String>,
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_hash: Option<String>, // 硬件哈希
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_raw: Option<String>, // 硬件原始信息
}

/// 机器用户列表响应
#[derive(Debug, Deserialize, Serialize)]
pub struct MachineUserListResponse {
    pub total: i64,
    pub list: Vec<crate::dto::machine_users::MachineUser>,
}

/// 查询机器用户参数
#[derive(Debug, Deserialize, Serialize)]
pub struct QueryMachineUserParams {
    pub user_uuid: Option<String>, // UUID字符串格式
    pub platform: Option<String>,
    pub status: Option<String>,
}

/// 获取机器信息请求（id 或 machine_code 二选一）
#[derive(Debug, Deserialize, Serialize)]
pub struct GetMachineRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine_code: Option<String>,
}

/// 查询机器列表请求
#[derive(Debug, Deserialize, Serialize)]
pub struct QueryMachinesRequest {
    pub params: QueryMachineUserParams,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_num: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,
}

/// 更新机器信息请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateMachineRequest {
    pub id: i32,
    pub request: UpdateMachineUserRequest,
}

/// 绑定用户到机器请求
#[derive(Debug, Deserialize, Serialize)]
pub struct BindUserRequest {
    pub machine_code: String,
    pub user_uuid: Uuid,
}

/// 解绑用户请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UnbindUserRequest {
    pub machine_code: String,
    pub user_uuid: Uuid,
}

/// 获取用户机器列表请求
#[derive(Debug, Deserialize, Serialize)]
pub struct GetUserMachinesRequest {
    pub user_uuid: Uuid,
}

/// 更新机器版本信息请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateMachineVersionRequest {
    pub machine_code: String,
    pub user_uuid: Uuid,
    pub version_info: serde_json::Value,
}

/// 拉黑/恢复机器请求
#[derive(Debug, Deserialize, Serialize)]
pub struct AllowOrBlacklistMachineRequest {
    pub machine_code: String,
    pub allow: bool,
}
