use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Pagination;

/// 查询 RPA 任务列表请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListRpaTasksRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub filters: Option<RpaTaskFilters>,
}

/// RPA 任务筛选条件
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RpaTaskFilters {
    pub keyword: Option<String>,
    pub status: Option<String>,
    pub trigger_type: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// 创建 RPA 任务请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateRpaTaskRequest {
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub trigger_type: String,
    pub schedule: Option<String>,
    pub cron_expression: Option<String>,
    pub run_mode: String,
    pub retry_count: Option<i32>,
    pub retry_interval: Option<i32>,
    pub timeout: Option<i32>,
    pub concurrency: Option<i32>,
    pub stop_on_error: Option<bool>,
    pub notify_on_complete: Option<bool>,
    pub notify_on_error: Option<bool>,
    pub environment_uuids: Option<Vec<Uuid>>,
    pub steps: Option<Vec<RpaTaskStepRequest>>,
}

/// RPA 任务步骤请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RpaTaskStepRequest {
    pub step_type: String,
    pub name: String,
    pub config: serde_json::Value,
    pub enabled: Option<bool>,
    pub position_x: Option<i32>,
    pub position_y: Option<i32>,
    pub sort_order: Option<i32>,
}

/// 更新 RPA 任务请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateRpaTaskRequest {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub trigger_type: Option<String>,
    pub schedule: Option<String>,
    pub cron_expression: Option<String>,
    pub run_mode: Option<String>,
    pub retry_count: Option<i32>,
    pub retry_interval: Option<i32>,
    pub timeout: Option<i32>,
    pub concurrency: Option<i32>,
    pub stop_on_error: Option<bool>,
    pub notify_on_complete: Option<bool>,
    pub notify_on_error: Option<bool>,
    pub environment_uuids: Option<Vec<Uuid>>,
    pub steps: Option<Vec<RpaTaskStepRequest>>,
}

/// 运行 RPA 任务请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RunRpaTaskRequest {
    pub uuid: Uuid,
    pub environment_uuids: Option<Vec<Uuid>>,
}

/// 复制 RPA 任务请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DuplicateRpaTaskRequest {
    pub uuid: Uuid,
    pub new_name: Option<String>,
}

/// 导出 RPA 任务请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExportRpaTaskRequest {
    pub uuid: Uuid,
}

/// 导入 RPA 任务请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImportRpaTaskRequest {
    pub import_data: String,
    pub name: Option<String>,
}

/// 查询执行记录请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListRpaRunsRequest {
    pub task_uuid: Uuid,
    #[serde(flatten)]
    pub pagination: Pagination,
    pub status: Option<String>,
}

// ========== 响应结构体 ==========

use crate::dto::{RpaTaskDto, RpaTaskRunDto, RpaTaskStepDto};

/// RPA 任务列表响应
#[derive(Debug, Clone, Serialize)]
pub struct RpaTaskListResponse {
    pub items: Vec<RpaTaskDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// RPA 任务详情响应
#[derive(Debug, Clone, Serialize)]
pub struct RpaTaskDetailResponse {
    pub task: RpaTaskDto,
    pub steps: Vec<RpaTaskStepDto>,
    pub environment_uuids: Vec<Uuid>,
}

/// RPA 任务执行记录列表响应
#[derive(Debug, Clone, Serialize)]
pub struct RpaRunsListResponse {
    pub items: Vec<RpaTaskRunDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 导出 RPA 任务响应
#[derive(Debug, Clone, Serialize)]
pub struct ExportRpaTaskResponse {
    pub content: String,
    pub filename: String,
}
