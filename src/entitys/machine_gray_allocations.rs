use chrono::DateTime;
use serde::{Deserialize, Serialize};

/// 分配机器到灰度请求
#[derive(Debug, Deserialize, Serialize)]
pub struct AllocateMachineRequest {
    pub machine_code: String,
    pub gray_release_id: i32,
    pub effective_time: Option<DateTime<chrono::Utc>>,
    pub notes: Option<String>,
}

/// 批量分配机器请求
#[derive(Debug, Deserialize, Serialize)]
pub struct BatchAllocateMachineRequest {
    pub machine_codes: Vec<String>,
    pub gray_release_id: i32,
    pub effective_time: Option<DateTime<chrono::Utc>>,
    pub notes: Option<String>,
}

/// 分配列表响应
#[derive(Debug, Deserialize, Serialize)]
pub struct AllocationListResponse {
    pub total: i64,
    pub list: Vec<crate::dto::machine_gray_allocations::MachineGrayAllocation>,
}

/// 取消机器分配请求
#[derive(Debug, Deserialize, Serialize)]
pub struct DeallocateMachineRequest {
    pub machine_code: String,
    pub gray_release_id: i32,
}

/// 查询灰度分配列表请求
#[derive(Debug, Deserialize, Serialize)]
pub struct ListAllocationsByReleaseRequest {
    pub gray_release_id: i32,
    pub page_num: Option<i32>,
    pub page_size: Option<i32>,
}

/// 检查机器是否在灰度中请求
#[derive(Debug, Deserialize, Serialize)]
pub struct CheckMachineInGrayRequest {
    pub machine_code: String,
}

/// 检查机器是否在灰度中响应
#[derive(Debug, Deserialize, Serialize)]
pub struct CheckMachineInGrayResponse {
    pub in_gray: bool,
    pub gray_release_id: Option<i32>,
    pub gray_release: Option<crate::dto::gray_releases::GrayRelease>,
}
