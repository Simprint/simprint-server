use crate::{
    dto::maintenance::Maintenance, entitys::maintenance::*, errors::SimprintError, svc_ctx::SvcCtx,
};

/// 创建维护
pub async fn create_maintenance_service(
    svc_ctx: &SvcCtx,
    request: CreateMaintenanceRequest,
) -> Result<i64, SimprintError> {
    let pool = &svc_ctx.db;
    let maintenance = crate::models::maintenance::create_maintenance(pool, request).await?;

    Ok(maintenance.id)
}

/// 根据ID查询维护
pub async fn get_maintenance_by_id_service(
    svc_ctx: &SvcCtx,
    payload: GetMaintenanceParams,
) -> Result<Maintenance, SimprintError> {
    let pool = &svc_ctx.db;
    let maintenance = crate::models::maintenance::get_maintenance_by_id(pool, payload.id)
        .await?
        .ok_or(SimprintError::MaintenanceNotFound)?;

    Ok(maintenance)
}

/// 查询维护列表
pub async fn list_maintenances_service(
    svc_ctx: &SvcCtx,
    payload: QueryMaintenancesParams,
) -> Result<MaintenanceListResponse, SimprintError> {
    let pool = &svc_ctx.db;
    let maintenances = crate::models::maintenance::list_maintenances(
        pool,
        payload.limit.map(|l| l as i64),
        payload.offset.map(|o| o as i64),
    )
    .await?;

    Ok(MaintenanceListResponse { list: maintenances })
}

/// 更新维护状态
pub async fn update_maintenance_status_service(
    svc_ctx: &SvcCtx,
    payload: UpdateMaintenanceStatusRequest,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;
    let success =
        crate::models::maintenance::update_maintenance_status(pool, payload.id, &payload.status)
            .await?;

    Ok(success)
}

/// 结束维护
pub async fn end_maintenance_service(svc_ctx: &SvcCtx) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;
    let success = crate::models::maintenance::end_maintenance(pool).await?;

    Ok(success)
}

/// 获取当前活跃维护
pub async fn get_active_maintenance_service(
    svc_ctx: &SvcCtx,
) -> Result<Option<Maintenance>, SimprintError> {
    let pool = &svc_ctx.db;
    let maintenance = crate::models::maintenance::get_active_maintenances(pool).await?;

    Ok(maintenance)
}
