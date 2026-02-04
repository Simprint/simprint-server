use crate::{
    dto::gray_releases::GrayRelease, entitys::gray_releases::*, errors::SimprintError,
    svc_ctx::SvcCtx,
};

/// 创建灰度发布
pub async fn create_gray_release(
    svc_ctx: &SvcCtx,
    req: CreateGrayReleaseRequest,
) -> Result<i32, SimprintError> {
    let id = crate::models::gray_releases::create_gray_release(&svc_ctx.db, &req).await?;

    Ok(id)
}

/// 查询灰度发布详情
pub async fn get_gray_release_by_id(
    svc_ctx: &SvcCtx,
    id: i32,
) -> Result<GrayRelease, SimprintError> {
    let release = crate::models::gray_releases::query_gray_release_by_id(&svc_ctx.db, id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    Ok(release)
}

/// 查询灰度发布列表
pub async fn query_gray_releases_service(
    svc_ctx: &SvcCtx,
    request: ListGrayReleasesRequest,
    // params: QueryGrayReleaseParams,
    // page_num: i32,
    // page_size: i32,
) -> Result<GrayReleaseListResponse, SimprintError> {
    let (total, list) = crate::models::gray_releases::query_gray_releases(
        &svc_ctx.db,
        request.platform.as_deref(),
        request.status.as_deref(),
        request.page.unwrap_or(1),
        request.page_size.unwrap_or(20),
    )
    .await?;

    Ok(GrayReleaseListResponse { total, list })
}

/// 更新灰度发布
pub async fn update_gray_release_service(
    svc_ctx: &SvcCtx,
    request: UpdateGrayReleaseRequest,
) -> Result<bool, SimprintError> {
    let id = request.gray_release_id;

    // 检查灰度发布是否存在
    crate::models::gray_releases::query_gray_release_by_id(&svc_ctx.db, id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    // 更新灰度发布
    let success =
        crate::models::gray_releases::update_gray_release(&svc_ctx.db, id, &request).await?;

    if !success {
        return Err(SimprintError::GrayReleaseNotFound);
    }

    Ok(true)
}

/// 启动灰度发布
pub async fn start_gray_release_service(svc_ctx: &SvcCtx, id: i32) -> Result<bool, SimprintError> {
    let release = crate::models::gray_releases::query_gray_release_by_id(&svc_ctx.db, id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    // 检查状态
    if release.status == "active" {
        return Ok(true); // 已经是活跃状态
    }

    if release.status != "pending" {
        return Err(SimprintError::InvalidRequest(format!(
            "灰度发布状态为 {}，无法启动",
            release.status
        )));
    }

    // 更新状态为 active
    let request = UpdateGrayReleaseRequest {
        gray_release_id: id,
        name: None,
        description: None,
        status: Some("active".to_string()),
        end_time: None,
        max_machines: None,
        priority: None,
        strategy_type: None,
        strategy_config: None,
    };

    let success =
        crate::models::gray_releases::update_gray_release(&svc_ctx.db, id, &request).await?;

    if !success {
        return Err(SimprintError::GrayReleaseNotFound);
    }

    Ok(true)
}

/// 暂停灰度发布
pub async fn pause_gray_release_service(svc_ctx: &SvcCtx, id: i32) -> Result<bool, SimprintError> {
    let release = crate::models::gray_releases::query_gray_release_by_id(&svc_ctx.db, id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    // 检查状态
    if release.status != "active" {
        return Err(SimprintError::InvalidRequest(format!(
            "灰度发布状态为 {}，无法暂停",
            release.status
        )));
    }

    // 更新状态为 paused
    let request = UpdateGrayReleaseRequest {
        gray_release_id: id,
        name: None,
        description: None,
        status: Some("paused".to_string()),
        end_time: None,
        max_machines: None,
        priority: None,
        strategy_type: None,
        strategy_config: None,
    };

    let success =
        crate::models::gray_releases::update_gray_release(&svc_ctx.db, id, &request).await?;

    if !success {
        return Err(SimprintError::GrayReleaseNotFound);
    }

    Ok(true)
}

/// 结束灰度发布
pub async fn finish_gray_release_service(svc_ctx: &SvcCtx, id: i32) -> Result<bool, SimprintError> {
    let release = crate::models::gray_releases::query_gray_release_by_id(&svc_ctx.db, id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    // 检查状态
    if release.status == "finished" {
        return Ok(true); // 已经结束
    }

    if !["active", "paused"].contains(&release.status.as_str()) {
        return Err(SimprintError::InvalidRequest(format!(
            "灰度发布状态为 {}，无法结束",
            release.status
        )));
    }

    // 更新状态为 finished，并设置结束时间
    let request = UpdateGrayReleaseRequest {
        gray_release_id: id,
        name: None,
        description: None,
        status: Some("finished".to_string()),
        end_time: Some(chrono::Utc::now()),
        max_machines: None,
        priority: None,
        strategy_type: None,
        strategy_config: None,
    };

    let success =
        crate::models::gray_releases::update_gray_release(&svc_ctx.db, id, &request).await?;

    if !success {
        return Err(SimprintError::GrayReleaseNotFound);
    }

    Ok(true)
}

/// 取消灰度发布
pub async fn cancel_gray_release_service(svc_ctx: &SvcCtx, id: i32) -> Result<bool, SimprintError> {
    let release = crate::models::gray_releases::query_gray_release_by_id(&svc_ctx.db, id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    // 检查状态
    if release.status == "cancelled" {
        return Ok(true); // 已经取消
    }

    if !["pending", "active", "paused"].contains(&release.status.as_str()) {
        return Err(SimprintError::InvalidRequest(format!(
            "灰度发布状态为 {}，无法取消",
            release.status
        )));
    }

    // 更新状态为 cancelled
    let request = UpdateGrayReleaseRequest {
        gray_release_id: id,
        name: None,
        description: None,
        status: Some("cancelled".to_string()),
        end_time: Some(chrono::Utc::now()),
        max_machines: None,
        priority: None,
        strategy_type: None,
        strategy_config: None,
    };

    let success =
        crate::models::gray_releases::update_gray_release(&svc_ctx.db, id, &request).await?;

    if !success {
        return Err(SimprintError::GrayReleaseNotFound);
    }

    Ok(true)
}

/// 删除灰度发布（物理删除）
pub async fn delete_gray_release_service(svc_ctx: &SvcCtx, id: i32) -> Result<bool, SimprintError> {
    // 先检查是否存在
    let _release = crate::models::gray_releases::query_gray_release_by_id(&svc_ctx.db, id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    // 先删除关联表，避免外键约束
    let _ = crate::models::machine_gray_allocations::delete_allocations_by_gray_release_id(
        &svc_ctx.db,
        id,
    )
    .await?;
    let _ = crate::models::gray_resources::delete_resources_by_gray_release_id(&svc_ctx.db, id)
        .await?;

    let success = crate::models::gray_releases::delete_gray_release(&svc_ctx.db, id).await?;

    if !success {
        return Err(SimprintError::GrayReleaseNotFound);
    }

    Ok(true)
}
