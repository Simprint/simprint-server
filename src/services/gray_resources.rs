use crate::{
    dto::gray_resources::GrayResource, entitys::gray_resources::*, errors::SimprintError,
    svc_ctx::SvcCtx,
};

/// 添加灰度资源
pub async fn add_gray_resource_service(
    svc_ctx: &SvcCtx,
    request: CreateGrayResourceRequest,
) -> Result<i32, SimprintError> {
    let pool = &svc_ctx.db;
    // 验证灰度发布是否存在
    crate::models::gray_releases::query_gray_release_by_id(pool, request.gray_release_id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    // 验证版本是否存在
    crate::models::versions::query_version_by_id(pool, request.version_id)
        .await
        .map_err(|_| SimprintError::VersionNotFound)?;

    // 检查是否已存在该资源
    let existing_resources = crate::models::gray_resources::query_gray_resources_by_release_id(
        pool,
        request.gray_release_id,
    )
    .await?;

    for resource in existing_resources {
        if resource.version_id == request.version_id {
            return Err(SimprintError::InvalidRequest(
                "该版本已存在于灰度资源中".to_string(),
            ));
        }
    }

    let id = crate::models::gray_resources::create_gray_resource(pool, &request).await?;

    Ok(id)
}

/// 批量添加灰度资源
pub async fn batch_add_gray_resources_service(
    svc_ctx: &SvcCtx,
    payload: BatchCreateGrayResourceRequest,
) -> Result<usize, SimprintError> {
    let pool = &svc_ctx.db;
    // 验证灰度发布是否存在
    crate::models::gray_releases::query_gray_release_by_id(pool, payload.gray_release_id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    // 验证所有版本是否存在
    for version_id in &payload.version_ids {
        crate::models::versions::query_version_by_id(pool, *version_id)
            .await
            .map_err(|_| SimprintError::VersionNotFound)?;
    }

    // 检查是否有重复的版本ID
    let mut seen = std::collections::HashSet::new();
    for version_id in &payload.version_ids {
        if !seen.insert(version_id) {
            return Err(SimprintError::InvalidRequest(format!(
                "版本ID {} 重复",
                version_id
            )));
        }
    }

    let count = crate::models::gray_resources::batch_create_gray_resources(
        pool,
        payload.gray_release_id,
        &payload.version_ids,
    )
    .await?;

    Ok(count)
}

/// 移除灰度资源
pub async fn remove_gray_resource_service(
    svc_ctx: &SvcCtx,
    payload: RemoveGrayResourceParams,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;
    // 检查资源是否存在
    let _resource = crate::models::gray_resources::query_gray_resource_by_id(pool, payload.id)
        .await
        .map_err(|_| SimprintError::InvalidRequest("灰度资源不存在".to_string()))?;

    let success = crate::models::gray_resources::delete_gray_resource(pool, payload.id).await?;

    if !success {
        return Err(SimprintError::InvalidRequest("删除失败".to_string()));
    }

    Ok(true)
}

/// 查询灰度资源列表
pub async fn query_gray_resources_service(
    svc_ctx: &SvcCtx,
    payload: GetGrayResourceParams,
) -> Result<Vec<GrayResource>, SimprintError> {
    let pool = &svc_ctx.db;
    // 验证灰度发布是否存在
    crate::models::gray_releases::query_gray_release_by_id(pool, payload.gray_release_id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    let resources = crate::models::gray_resources::query_gray_resources_by_release_id(
        pool,
        payload.gray_release_id,
    )
    .await?;

    Ok(resources)
}
