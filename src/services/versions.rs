use crate::{dto::versions::Version, entitys::versions::*, errors::SimprintError, svc_ctx::SvcCtx};

/// 创建版本
pub async fn create_version_service(
    svc_ctx: &SvcCtx,
    request: CreateVersionRequest,
) -> Result<i32, SimprintError> {
    let pool = &svc_ctx.db;

    // 验证版本号
    if request.version.is_empty() {
        return Err(SimprintError::VersionEmpty);
    }

    // 验证资源名称
    if request.resource_name.is_empty() {
        return Err(SimprintError::ResourceNameEmpty);
    }

    // 检查版本类型是否存在
    crate::models::version_types::query_version_type_by_id(pool, request.type_id)
        .await?
        .ok_or(SimprintError::VersionTypeNotFound)?;

    // 检查版本是否已存在
    let exists = crate::models::versions::query_version_by_name_and_version(
        pool,
        &request.resource_name,
        &request.version,
    )
    .await
    .ok();

    if exists.is_some() {
        return Err(SimprintError::VersionAlreadyExists);
    }

    // 创建版本
    let id = crate::models::versions::insert_version(pool, &request).await?;

    Ok(id)
}

/// 根据ID查询版本
pub async fn get_version_by_id_service(
    svc_ctx: &SvcCtx,
    id: i32,
) -> Result<Version, SimprintError> {
    let pool = &svc_ctx.db;
    let version = crate::models::versions::query_version_by_id(pool, id)
        .await?
        .ok_or(SimprintError::VersionNotFound)?;

    Ok(version)
}

/// 根据资源名称和版本号查询版本
pub async fn get_version_by_name_and_version_service(
    svc_ctx: &SvcCtx,
    resource_name: String,
    version: String,
) -> Result<Version, SimprintError> {
    let pool = &svc_ctx.db;
    let version_data =
        crate::models::versions::query_version_by_name_and_version(pool, &resource_name, &version)
            .await?
            .ok_or(SimprintError::VersionNotFound)?;

    Ok(version_data)
}

/// 查询最新版本
pub async fn get_latest_version_service(
    svc_ctx: &SvcCtx,
    resource_name: String,
    platform: String,
) -> Result<Version, SimprintError> {
    let pool = &svc_ctx.db;
    let version = crate::models::versions::query_latest_version(pool, &resource_name, &platform)
        .await?
        .ok_or(SimprintError::VersionNotFound)?;

    Ok(version)
}

/// 查询版本列表
pub async fn query_versions_service(
    svc_ctx: &SvcCtx,
    params: QueryVersionsParams,
) -> Result<VersionListResponse, SimprintError> {
    let pool = &svc_ctx.db;
    let (total, list) = crate::models::versions::query_versions(
        pool,
        params.resource_name.as_deref(),
        params.platform.as_deref(),
        params.status.as_deref(),
        params.page_num.unwrap_or(1),
        params.page_size.unwrap_or(20),
    )
    .await?;

    Ok(VersionListResponse { total, list })
}

/// 更新版本
pub async fn update_version_service(
    svc_ctx: &SvcCtx,
    request: UpdateVersionRequest,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;
    // 检查版本是否存在
    crate::models::versions::query_version_by_id(pool, request.id)
        .await?
        .ok_or(SimprintError::VersionNotFound)?;

    let success = crate::models::versions::update_version(pool, request.id, &request).await?;

    if !success {
        return Err(SimprintError::VersionNotFound);
    }

    Ok(true)
}

/// 删除版本
pub async fn delete_version_service(
    svc_ctx: &SvcCtx,
    payload: DeleteVersionRequest,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;
    // 检查版本是否存在
    crate::models::versions::query_version_by_id(pool, payload.id)
        .await?
        .ok_or(SimprintError::VersionNotFound)?;

    let success = crate::models::versions::delete_version(pool, payload.id).await?;

    Ok(success)
}

/// 设置最新版本
pub async fn set_latest_version_service(
    svc_ctx: &SvcCtx,
    payload: SetLatestVersionRequest,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;
    // 检查版本是否存在
    let version = crate::models::versions::query_version_by_id(pool, payload.version_id)
        .await?
        .ok_or(SimprintError::VersionNotFound)?;

    if version.type_id != payload.type_id || version.resource_name != payload.resource_name {
        return Err(SimprintError::InvalidRequest(
            "版本类型或资源名称不匹配".to_string(),
        ));
    }

    let success = crate::models::versions::set_as_latest_version(
        pool,
        payload.type_id,
        &payload.resource_name,
        payload.version_id,
    )
    .await?;

    Ok(success)
}

/// 查询所有激活版本类型的最新版本
pub async fn get_all_active_latest_versions_service(
    svc_ctx: &SvcCtx,
    platform: String,
) -> Result<std::collections::HashMap<String, Vec<Version>>, SimprintError> {
    let pool = &svc_ctx.db;
    let results =
        crate::models::versions::query_all_active_latest_versions(pool, &platform).await?;

    // 转换为 HashMap<type_code, Vec<Version>>
    let mut map: std::collections::HashMap<String, Vec<Version>> = std::collections::HashMap::new();

    for (type_code, _resource_name, version) in results {
        map.entry(type_code).or_insert_with(Vec::new).push(version);
    }

    Ok(map)
}
