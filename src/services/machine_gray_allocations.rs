use crate::{
    dto::machine_gray_allocations::MachineGrayAllocation, entitys::machine_gray_allocations::*,
    errors::SimprintError, svc_ctx::SvcCtx,
};

/// 分配机器到灰度
pub async fn allocate_machine_service(
    svc_ctx: &SvcCtx,
    request: AllocateMachineRequest,
) -> Result<i32, SimprintError> {
    let pool = &svc_ctx.db;

    // 验证机器码
    if request.machine_code.is_empty() {
        return Err(SimprintError::InvalidRequest("机器码不能为空".to_string()));
    }

    // 检查灰度发布是否存在
    crate::models::gray_releases::query_gray_release_by_id(pool, request.gray_release_id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    // 检查是否已达到最大配额
    let release =
        crate::models::gray_releases::query_gray_release_by_id(pool, request.gray_release_id)
            .await?;

    if let Some(max_machines) = release.max_machines {
        if release.allocated_count >= max_machines {
            return Err(SimprintError::InvalidRequest(
                "灰度发布已达到最大配额".to_string(),
            ));
        }
    }

    // 分配机器
    let id = crate::models::machine_gray_allocations::allocate_machine_to_gray(
        pool,
        &request.machine_code,
        request.gray_release_id,
        request.effective_time,
        request.notes.as_deref(),
    )
    .await?;

    // 更新分配计数（如果之前不存在）
    let existed = crate::models::machine_gray_allocations::check_machine_allocated(
        pool,
        &request.machine_code,
        request.gray_release_id,
    )
    .await?;

    if !existed {
        let _ =
            crate::models::gray_releases::increment_allocated_count(pool, request.gray_release_id)
                .await?;
    }

    Ok(id)
}

/// 批量分配机器
pub async fn batch_allocate_machines_service(
    svc_ctx: &SvcCtx,
    request: BatchAllocateMachineRequest,
) -> Result<usize, SimprintError> {
    let pool = &svc_ctx.db;

    if request.machine_codes.is_empty() {
        return Err(SimprintError::InvalidRequest(
            "机器码列表不能为空".to_string(),
        ));
    }

    // 检查灰度发布是否存在
    crate::models::gray_releases::query_gray_release_by_id(pool, request.gray_release_id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    // 分配机器
    let count = crate::models::machine_gray_allocations::batch_allocate_machines_to_gray(
        pool,
        &request.machine_codes,
        request.gray_release_id,
        request.effective_time,
        request.notes.as_deref(),
    )
    .await?;

    // 更新分配计数
    for _ in 0..count {
        let _ =
            crate::models::gray_releases::increment_allocated_count(pool, request.gray_release_id)
                .await?;
    }

    Ok(count)
}

/// 取消机器分配
pub async fn remove_allocation_service(
    svc_ctx: &SvcCtx,
    payload: RemoveAllocationRequest,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;

    // 移除分配
    let success = crate::models::machine_gray_allocations::deallocate_machine(
        pool,
        &payload.machine_code,
        payload.gray_release_id,
    )
    .await?;

    if success {
        // 减少分配计数
        let _ =
            crate::models::gray_releases::decrement_allocated_count(pool, payload.gray_release_id)
                .await?;
    }

    Ok(success)
}

/// 查询机器是否在灰度中
pub async fn is_machine_in_gray_service(
    svc_ctx: &SvcCtx,
    payload: IsMachineInGrayRequest,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;

    let is_in_gray = crate::models::machine_gray_allocations::check_machine_allocated(
        pool,
        &payload.machine_code,
        payload.gray_release_id,
    )
    .await?;

    Ok(is_in_gray)
}

/// 查询机器的所有活跃分配
pub async fn get_machine_allocations_service(
    svc_ctx: &SvcCtx,
    payload: GetMachineAllocationsRequest,
) -> Result<Vec<MachineGrayAllocation>, SimprintError> {
    let pool = &svc_ctx.db;

    let allocations = crate::models::machine_gray_allocations::query_allocations_by_machine_code(
        pool,
        &payload.machine_code,
    )
    .await?;

    Ok(allocations)
}

/// 查询灰度发布的所有分配
pub async fn get_release_allocations_service(
    svc_ctx: &SvcCtx,
    payload: GetReleaseAllocationsRequest,
) -> Result<AllocationListResponse, SimprintError> {
    let pool = &svc_ctx.db;

    let (total, list) =
        crate::models::machine_gray_allocations::query_allocations_by_gray_release_id(
            pool,
            payload.gray_release_id,
            payload.page_num.unwrap_or(1),
            payload.page_size.unwrap_or(20),
        )
        .await?;

    Ok(AllocationListResponse { total, list })
}
