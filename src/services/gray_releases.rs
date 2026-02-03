use crate::{
    dto::gray_releases::GrayRelease, entitys::gray_releases::*, errors::SimprintError,
    svc_ctx::SvcCtx,
};

/// 创建灰度发布
pub async fn create_gray_release_service(
    svc_ctx: &SvcCtx,
    request: CreateGrayReleaseRequest,
) -> Result<i32, SimprintError> {
    let pool = &svc_ctx.db;
    let id = crate::models::gray_releases::create_gray_release(pool, &request).await?;

    Ok(id)
}

/// 查询灰度发布详情
pub async fn get_gray_release_by_id_service(
    svc_ctx: &SvcCtx,
    payload: GetGrayReleaseParams,
) -> Result<GrayRelease, SimprintError> {
    let pool = &svc_ctx.db;
    let release = crate::models::gray_releases::query_gray_release_by_id(pool, payload.id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    Ok(release)
}

/// 查询灰度发布列表
pub async fn query_gray_releases_service(
    svc_ctx: &SvcCtx,
    params: QueryGrayReleasesParams,
) -> Result<GrayReleaseListResponse, SimprintError> {
    let pool = &svc_ctx.db;
    let (total, list) = crate::models::gray_releases::query_gray_releases(
        pool,
        params.platform.as_deref(),
        params.status.as_deref(),
        params.page_num.unwrap_or(1),
        params.page_size.unwrap_or(20),
    )
    .await?;

    Ok(GrayReleaseListResponse { total, list })
}

/// 更新灰度发布
pub async fn update_gray_release_service(
    svc_ctx: &SvcCtx,
    request: UpdateGrayReleaseRequest,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;
    // 检查灰度发布是否存在
    crate::models::gray_releases::query_gray_release_by_id(pool, request.id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    // 更新灰度发布
    let success =
        crate::models::gray_releases::update_gray_release(pool, request.id, &request).await?;

    if !success {
        return Err(SimprintError::GrayReleaseNotFound);
    }

    Ok(true)
}

/// 启动灰度发布
pub async fn start_gray_release_service(
    svc_ctx: &SvcCtx,
    payload: GetGrayReleaseParams,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;
    let release = crate::models::gray_releases::query_gray_release_by_id(pool, payload.id)
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
        id: payload.id,
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
        crate::models::gray_releases::update_gray_release(pool, payload.id, &request).await?;

    if !success {
        return Err(SimprintError::GrayReleaseNotFound);
    }

    Ok(true)
}

/// 暂停灰度发布
pub async fn pause_gray_release_service(
    svc_ctx: &SvcCtx,
    payload: GetGrayReleaseParams,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;
    let release = crate::models::gray_releases::query_gray_release_by_id(pool, payload.id)
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
        id: payload.id,
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
        crate::models::gray_releases::update_gray_release(pool, payload.id, &request).await?;

    if !success {
        return Err(SimprintError::GrayReleaseNotFound);
    }

    Ok(true)
}

/// 结束灰度发布
pub async fn finish_gray_release_service(
    svc_ctx: &SvcCtx,
    payload: GetGrayReleaseParams,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;
    let release = crate::models::gray_releases::query_gray_release_by_id(pool, payload.id)
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
        id: payload.id,
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
        crate::models::gray_releases::update_gray_release(pool, payload.id, &request).await?;

    if !success {
        return Err(SimprintError::GrayReleaseNotFound);
    }

    Ok(true)
}

/// 取消灰度发布
pub async fn cancel_gray_release_service(
    svc_ctx: &SvcCtx,
    payload: GetGrayReleaseParams,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;
    let release = crate::models::gray_releases::query_gray_release_by_id(pool, payload.id)
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
        id: payload.id,
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
        crate::models::gray_releases::update_gray_release(pool, payload.id, &request).await?;

    if !success {
        return Err(SimprintError::GrayReleaseNotFound);
    }

    Ok(true)
}

/// 删除灰度发布（物理删除）
pub async fn delete_gray_release_service(
    svc_ctx: &SvcCtx,
    payload: GetGrayReleaseParams,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;
    // 先检查是否存在
    let _release = crate::models::gray_releases::query_gray_release_by_id(pool, payload.id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    // 物理删除（关联的 gray_resources 会通过 ON DELETE CASCADE 自动删除）
    let success = crate::models::gray_releases::delete_gray_release(pool, payload.id).await?;

    if !success {
        return Err(SimprintError::GrayReleaseNotFound);
    }

    Ok(true)
}

/// 检查白名单策略
pub async fn check_whitelist_strategy_service(
    svc_ctx: &SvcCtx,
    machine_code: String,
) -> Result<Option<GrayRelease>, SimprintError> {
    let pool = &svc_ctx.db;
    let release =
        crate::models::gray_releases::check_whitelist_strategy(pool, &machine_code).await?;
    Ok(release)
}

/// 查询动态策略灰度发布
pub async fn query_dynamic_strategy_releases_service(
    svc_ctx: &SvcCtx,
) -> Result<Vec<GrayRelease>, SimprintError> {
    let pool = &svc_ctx.db;
    let releases = crate::models::gray_releases::query_dynamic_strategy_releases(pool).await?;
    Ok(releases)
}
