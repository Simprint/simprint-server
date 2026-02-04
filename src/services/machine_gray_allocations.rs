use crate::{
    entitys::machine_gray_allocations::*, errors::SimprintError, svc_ctx::SvcCtx,
    utils::gray_checker::check_machine_by_strategy,
};

/// 分配机器到灰度
pub async fn allocate_machine_service(
    svc_ctx: &SvcCtx,
    request: AllocateMachineRequest,
) -> Result<bool, SimprintError> {
    // 验证机器码
    if request.machine_code.is_empty() {
        return Err(SimprintError::InvalidRequest("机器码不能为空".to_string()));
    }

    // 检查灰度发布是否存在
    crate::models::gray_releases::query_gray_release_by_id(&svc_ctx.db, request.gray_release_id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    // 检查是否已达到最大配额
    let release = crate::models::gray_releases::query_gray_release_by_id(
        &svc_ctx.db,
        request.gray_release_id,
    )
    .await?;

    if let Some(max_machines) = release.max_machines {
        if release.allocated_count >= max_machines {
            return Err(SimprintError::InvalidRequest(
                "灰度发布已达到最大配额".to_string(),
            ));
        }
    }

    // 分配机器
    let success =
        crate::models::machine_gray_allocations::allocate_machine(&svc_ctx.db, &request).await?;

    if success {
        // 更新分配计数
        let _ = crate::models::gray_releases::increment_allocated_count(
            &svc_ctx.db,
            request.gray_release_id,
        )
        .await?;
    }

    Ok(success)
}

/// 批量分配机器
pub async fn batch_allocate_machines_service(
    svc_ctx: &SvcCtx,
    request: BatchAllocateMachineRequest,
) -> Result<i64, SimprintError> {
    if request.machine_codes.is_empty() {
        return Err(SimprintError::InvalidRequest(
            "机器码列表不能为空".to_string(),
        ));
    }

    // 检查灰度发布是否存在
    crate::models::gray_releases::query_gray_release_by_id(&svc_ctx.db, request.gray_release_id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    // 分配机器
    let count =
        crate::models::machine_gray_allocations::batch_allocate_machines(&svc_ctx.db, &request)
            .await?;

    // 更新分配计数
    for _ in 0..count {
        let _ = crate::models::gray_releases::increment_allocated_count(
            &svc_ctx.db,
            request.gray_release_id,
        )
        .await?;
    }

    Ok(count)
}

/// 取消机器分配
pub async fn remove_allocation_service(
    svc_ctx: &SvcCtx,
    machine_code: String,
    gray_release_id: i32,
) -> Result<bool, SimprintError> {
    // 检查分配是否存在
    let allocation = crate::models::machine_gray_allocations::query_allocation(
        &svc_ctx.db,
        &machine_code,
        gray_release_id,
    )
    .await?;

    if allocation.is_none() {
        return Err(SimprintError::Other("分配不存在".to_string()));
    }

    // 移除分配
    let success = crate::models::machine_gray_allocations::remove_allocation(
        &svc_ctx.db,
        &machine_code,
        gray_release_id,
    )
    .await?;

    if success {
        // 减少分配计数
        let _ =
            crate::models::gray_releases::decrement_allocated_count(&svc_ctx.db, gray_release_id)
                .await?;
    }

    Ok(success)
}

/// 查询机器是否在灰度中（旧方法，仅用于白名单）
pub async fn is_machine_in_gray_service(
    svc_ctx: &SvcCtx,
    machine_code: String,
    gray_release_id: i32,
) -> Result<bool, SimprintError> {
    let is_in_gray = crate::models::machine_gray_allocations::is_machine_in_gray(
        &svc_ctx.db,
        &machine_code,
        gray_release_id,
    )
    .await?;

    Ok(is_in_gray)
}

/// 检查机器是否在任意活跃灰度中（优化版：分策略类型查询）
pub async fn check_machine_in_any_gray_service(
    svc_ctx: &SvcCtx,
    machine_code: String,
) -> Result<
    (
        bool,
        Option<i32>,
        Option<crate::dto::gray_releases::GrayRelease>,
    ),
    SimprintError,
> {
    // 1. 优先检查白名单（最快，单次 JOIN 查询）
    if let Some(release) =
        crate::models::gray_releases::check_whitelist_strategy(&svc_ctx.db, &machine_code).await?
    {
        return Ok((true, Some(release.id), Some(release)));
    }

    // 2. 检查标签策略（较快，单次数据库过滤）
    if let Some(release) =
        crate::models::gray_releases::check_tags_strategy(&svc_ctx.db, &machine_code).await?
    {
        return Ok((true, Some(release.id), Some(release)));
    }

    // 3. 检查动态策略（百分比/随机，需要计算但数据量小）
    let dynamic_releases =
        crate::models::gray_releases::query_dynamic_strategy_releases(&svc_ctx.db).await?;

    for release in dynamic_releases {
        if check_machine_by_strategy(&machine_code, None, &release)? {
            return Ok((true, Some(release.id), Some(release)));
        }
    }

    // 4. 未命中任何灰度
    Ok((false, None, None))
}

/// 查询机器的所有活跃分配
pub async fn get_machine_allocations_service(
    svc_ctx: &SvcCtx,
    machine_code: String,
) -> Result<Vec<crate::dto::machine_gray_allocations::MachineGrayAllocation>, SimprintError> {
    let allocations = crate::models::machine_gray_allocations::query_machine_allocations(
        &svc_ctx.db,
        &machine_code,
    )
    .await?;

    Ok(allocations)
}

/// 查询灰度发布的所有分配
pub async fn get_release_allocations_service(
    svc_ctx: &SvcCtx,
    gray_release_id: i32,
    page_num: i32,
    page_size: i32,
) -> Result<AllocationListResponse, SimprintError> {
    let (total, list) = crate::models::machine_gray_allocations::query_release_allocations(
        &svc_ctx.db,
        gray_release_id,
        page_num,
        page_size,
    )
    .await?;

    Ok(AllocationListResponse { total, list })
}
