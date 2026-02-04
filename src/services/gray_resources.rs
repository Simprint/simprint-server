use crate::{
    dto::gray_resources::GrayResource, entitys::gray_resources::*, errors::SimprintError,
    svc_ctx::SvcCtx,
};

/// 添加灰度资源
pub async fn add_gray_resource(
    svc_ctx: &SvcCtx,
    request: CreateGrayResourceRequest,
) -> Result<i32, SimprintError> {
    // 验证灰度发布是否存在
    crate::models::gray_releases::query_gray_release_by_id(&svc_ctx.db, request.gray_release_id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    // 验证版本是否存在
    crate::models::versions::query_version_by_id(&svc_ctx.db, request.version_id)
        .await
        .map_err(|_| SimprintError::VersionNotFound)?;

    // 检查是否已存在该资源
    let existing_resources =
        crate::models::gray_resources::query_gray_resources(&svc_ctx.db, request.gray_release_id)
            .await?;

    for resource in existing_resources {
        if resource.version_id == request.version_id {
            return Err(SimprintError::InvalidRequest(
                "该版本已存在于灰度资源中".to_string(),
            ));
        }
    }

    // 如果没有指定排序，使用最大排序值+1
    let sort_order = if request.sort_order.is_some() {
        request.sort_order.unwrap()
    } else {
        let max_sort = crate::models::gray_resources::query_max_sort_order(
            &svc_ctx.db,
            request.gray_release_id,
        )
        .await
        .unwrap_or(0);
        max_sort + 1
    };

    let id = crate::models::gray_resources::create_gray_resource(
        &svc_ctx.db,
        &CreateGrayResourceRequest {
            gray_release_id: request.gray_release_id,
            version_id: request.version_id,
            sort_order: Some(sort_order),
        },
    )
    .await?;

    Ok(id)
}

/// 批量添加灰度资源
pub async fn batch_add_gray_resources(
    svc_ctx: &SvcCtx,
    request: BatchCreateGrayResourceRequest,
) -> Result<i64, SimprintError> {
    // 验证灰度发布是否存在
    crate::models::gray_releases::query_gray_release_by_id(&svc_ctx.db, request.gray_release_id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    // 验证所有版本是否存在
    for version_id in &request.version_ids {
        crate::models::versions::query_version_by_id(&svc_ctx.db, *version_id)
            .await
            .map_err(|_| SimprintError::VersionNotFound)?;
    }

    // 检查是否有重复的版本ID
    let mut seen = std::collections::HashSet::new();
    for version_id in &request.version_ids {
        if !seen.insert(version_id) {
            return Err(SimprintError::InvalidRequest(format!(
                "版本ID {} 重复",
                version_id
            )));
        }
    }

    let count =
        crate::models::gray_resources::batch_create_gray_resources(&svc_ctx.db, &request).await?;

    Ok(count)
}

/// 移除灰度资源
pub async fn remove_gray_resource(svc_ctx: &SvcCtx, id: i32) -> Result<bool, SimprintError> {
    // 检查资源是否存在
    let _resource = crate::models::gray_resources::query_gray_resource_by_id(&svc_ctx.db, id)
        .await
        .map_err(|_| SimprintError::InvalidRequest("灰度资源不存在".to_string()))?;

    let success = crate::models::gray_resources::delete_gray_resource(&svc_ctx.db, id).await?;

    if !success {
        return Err(SimprintError::InvalidRequest("删除失败".to_string()));
    }

    Ok(true)
}

/// 调整资源顺序
pub async fn update_resource_sort_order(
    svc_ctx: &SvcCtx,
    id: i32,
    sort_order: i32,
) -> Result<bool, SimprintError> {
    // 检查资源是否存在
    let _resource = crate::models::gray_resources::query_gray_resource_by_id(&svc_ctx.db, id)
        .await
        .map_err(|_| SimprintError::InvalidRequest("灰度资源不存在".to_string()))?;

    let success =
        crate::models::gray_resources::update_sort_order(&svc_ctx.db, id, sort_order).await?;

    if !success {
        return Err(SimprintError::InvalidRequest("更新失败".to_string()));
    }

    Ok(true)
}

/// 查询灰度资源列表
pub async fn query_gray_resources(
    svc_ctx: &SvcCtx,
    gray_release_id: i32,
) -> Result<Vec<GrayResource>, SimprintError> {
    // 验证灰度发布是否存在
    crate::models::gray_releases::query_gray_release_by_id(&svc_ctx.db, gray_release_id)
        .await
        .map_err(|_| SimprintError::GrayReleaseNotFound)?;

    let resources =
        crate::models::gray_resources::query_gray_resources(&svc_ctx.db, gray_release_id).await?;

    Ok(resources)
}
