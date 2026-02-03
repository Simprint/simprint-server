use serde::{Deserialize, Serialize};

use uuid::Uuid;

/// 创建/绑定机器用户请求
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateMachineUserRequest {
    pub machine_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_uuid: Option<Uuid>,
    pub platform: Option<String>,
    pub hardware_hash: Option<String>, // 硬件哈希
    pub hardware_raw: Option<String>,  // 硬件原始信息
    pub version_info: Option<String>,  // JSON格式的版本信息
}

/// 更新机器用户请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateMachineUserRequest {
    pub id: i32,
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

/// 绑定机器请求
#[derive(Debug, Deserialize, Serialize)]
pub struct BindMachineRequest {
    pub machine_code: String,
    pub user_uuid: Uuid,
}

/// 更新机器版本信息请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateMachineVersionRequest {
    pub machine_code: String,
    pub user_uuid: Uuid,
    pub version_info: serde_json::Value,
}

/// 拉黑/取消拉黑机器请求
#[derive(Debug, Deserialize, Serialize)]
pub struct AllowOrBlacklistMachineRequest {
    pub machine_code: String,
    pub allow: bool,
}

/// 获取机器参数
#[derive(Debug, Deserialize, Serialize)]
pub struct GetMachineParams {
    pub id: Option<i32>,
    pub machine_code: Option<String>,
}

/// 查询机器列表参数（带分页）
#[derive(Debug, Deserialize, Serialize)]
pub struct QueryMachinesParams {
    pub user_uuid: Option<String>,
    pub platform: Option<String>,
    pub status: Option<String>,
    pub page_num: Option<i32>,
    pub page_size: Option<i32>,
}

/// 绑定机器参数（用于查询字符串）
#[derive(Debug, Deserialize, Serialize)]
pub struct BindMachineParams {
    pub machine_code: String,
    pub user_uuid: String,
}

/// 更新机器版本参数（用于查询字符串）
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateMachineVersionParams {
    pub machine_code: String,
    pub user_uuid: String,
}
