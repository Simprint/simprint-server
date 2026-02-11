use axum::extract::{Extension, State};

use crate::dto::RpaTaskRunDto;
use crate::entitys::{
    BatchUuidRequest, CreateResponse, CreateRpaTaskRequest, DuplicateRpaTaskRequest,
    ExportRpaTaskRequest, ExportRpaTaskResponse, ListRpaRunsRequest, ListRpaTasksRequest,
    RpaRunsListResponse, RpaTaskDetailResponse, RpaTaskListResponse, RunRpaTaskRequest,
    UpdateRpaTaskRequest, UuidRequest,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 获取 RPA 任务列表
pub async fn get_rpa_tasks_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ListRpaTasksRequest>,
) -> Result<RpaTaskListResponse> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let (items, total) =
        services::rpa::get_rpa_tasks_service(&svc_ctx, ctx.user_uuid_unwrap(), team_uuid, &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(RpaTaskListResponse {
            items,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}

/// 获取 RPA 任务详情
pub async fn get_rpa_task_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UuidRequest>,
) -> Result<RpaTaskDetailResponse> {
    let (task, steps, environment_uuids) =
        services::rpa::get_rpa_task_service(&svc_ctx, payload.uuid)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(RpaTaskDetailResponse {
            task,
            steps,
            environment_uuids,
        }),
    ))
}

/// 创建 RPA 任务
pub async fn create_rpa_task_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CreateRpaTaskRequest>,
) -> Result<CreateResponse> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let uuid = services::rpa::create_rpa_task_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        team_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("创建成功"),
        Some(CreateResponse { uuid }),
    ))
}

/// 更新 RPA 任务
pub async fn update_rpa_task_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UpdateRpaTaskRequest>,
) -> Result<()> {
    services::rpa::update_rpa_task_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("更新成功"), None))
}

/// 删除 RPA 任务
pub async fn delete_rpa_task_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UuidRequest>,
) -> Result<()> {
    services::rpa::delete_rpa_task_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("删除成功"), None))
}

/// 批量删除 RPA 任务
pub async fn batch_delete_rpa_tasks_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<BatchUuidRequest>,
) -> Result<()> {
    services::rpa::batch_delete_rpa_tasks_service(&svc_ctx, &payload.uuids)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("批量删除成功"), None))
}

/// 运行 RPA 任务
pub async fn run_rpa_task_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<RunRpaTaskRequest>,
) -> Result<CreateResponse> {
    let run_uuid = services::rpa::run_rpa_task_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("任务已启动"),
        Some(CreateResponse { uuid: run_uuid }),
    ))
}

/// 停止 RPA 任务
pub async fn stop_rpa_task_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UuidRequest>,
) -> Result<()> {
    services::rpa::stop_rpa_task_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("任务已停止"), None))
}

/// 复制 RPA 任务
pub async fn duplicate_rpa_task_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<DuplicateRpaTaskRequest>,
) -> Result<CreateResponse> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let uuid = services::rpa::duplicate_rpa_task_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        team_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("复制成功"),
        Some(CreateResponse { uuid }),
    ))
}

/// 导出 RPA 任务
pub async fn export_rpa_task_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<ExportRpaTaskRequest>,
) -> Result<ExportRpaTaskResponse> {
    let (content, filename) = services::rpa::export_rpa_task_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("导出成功"),
        Some(ExportRpaTaskResponse { content, filename }),
    ))
}

/// 获取执行记录列表
pub async fn get_rpa_runs_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<ListRpaRunsRequest>,
) -> Result<RpaRunsListResponse> {
    let (items, total) = services::rpa::get_rpa_runs_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(RpaRunsListResponse {
            items,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}

/// 获取执行记录详情
pub async fn get_rpa_run_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UuidRequest>,
) -> Result<RpaTaskRunDto> {
    let run = services::rpa::get_rpa_run_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(run)))
}
