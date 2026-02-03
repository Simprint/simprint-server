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

/// 移除分配请求
#[derive(Debug, Deserialize, Serialize)]
pub struct RemoveAllocationRequest {
    pub machine_code: String,
    pub gray_release_id: i32,
}

/// 查询分配参数
#[derive(Debug, Deserialize, Serialize)]
pub struct GetAllocationParams {
    pub machine_code: Option<String>,
    pub gray_release_id: Option<i32>,
    pub page_num: Option<i32>,
    pub page_size: Option<i32>,
}

/// 查询机器是否在灰度中请求
#[derive(Debug, Deserialize, Serialize)]
pub struct IsMachineInGrayRequest {
    pub machine_code: String,
    pub gray_release_id: i32,
}

/// 查询机器所有分配请求
#[derive(Debug, Deserialize, Serialize)]
pub struct GetMachineAllocationsRequest {
    pub machine_code: String,
}

/// 查询灰度发布所有分配请求
#[derive(Debug, Deserialize, Serialize)]
pub struct GetReleaseAllocationsRequest {
    pub gray_release_id: i32,
    pub page_num: Option<i32>,
    pub page_size: Option<i32>,
}
