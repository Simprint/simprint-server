use crate::{
    dto::version_types::VersionType, entitys::version_types::*, errors::SimprintError,
    svc_ctx::SvcCtx,
};

/// 创建版本类型
pub async fn create_version_type_service(
    svc_ctx: &SvcCtx,
    request: CreateVersionTypeRequest,
) -> Result<i32, SimprintError> {
    let pool = &svc_ctx.db;
    if request.type_code.is_empty() {
        return Err(SimprintError::InvalidRequest(
            "类型代码不能为空".to_string(),
        ));
    }

    // 检查类型代码是否已存在
    let existing =
        crate::models::version_types::query_version_type_by_code(pool, &request.type_code)
            .await
            .ok();

    if existing.is_some() {
        return Err(SimprintError::InvalidRequest(
            "版本类型代码已存在".to_string(),
        ));
    }

    let id = crate::models::version_types::insert_version_type(pool, &request).await?;

    Ok(id)
}

/// 根据ID查询版本类型
pub async fn get_version_type_by_id_service(
    svc_ctx: &SvcCtx,
    payload: GetVersionTypeParams,
) -> Result<VersionType, SimprintError> {
    let pool = &svc_ctx.db;
    let version_type = crate::models::version_types::query_version_type_by_id(pool, payload.id)
        .await?
        .ok_or(SimprintError::VersionTypeNotFound)?;

    Ok(version_type)
}

/// 查询所有激活的版本类型
pub async fn get_active_version_types_service(
    svc_ctx: &SvcCtx,
) -> Result<VersionTypeListResponse, SimprintError> {
    let pool = &svc_ctx.db;
    let list = crate::models::version_types::query_active_version_types(pool).await?;

    Ok(VersionTypeListResponse { list })
}

/// 查询所有版本类型
pub async fn get_all_version_types_service(
    svc_ctx: &SvcCtx,
) -> Result<VersionTypeListResponse, SimprintError> {
    let pool = &svc_ctx.db;
    let list = crate::models::version_types::query_all_version_types(pool).await?;

    Ok(VersionTypeListResponse { list })
}

/// 更新版本类型
pub async fn update_version_type_service(
    svc_ctx: &SvcCtx,
    request: UpdateVersionTypeRequest,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;
    // 检查版本类型是否存在
    crate::models::version_types::query_version_type_by_id(pool, request.id)
        .await?
        .ok_or(SimprintError::VersionTypeNotFound)?;

    let success =
        crate::models::version_types::update_version_type(pool, request.id, &request).await?;

    if !success {
        return Err(SimprintError::VersionTypeNotFound);
    }

    Ok(true)
}

/// 删除版本类型
pub async fn delete_version_type_service(
    svc_ctx: &SvcCtx,
    payload: DeleteVersionTypeRequest,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;
    // 检查版本类型是否存在
    crate::models::version_types::query_version_type_by_id(pool, payload.id)
        .await?
        .ok_or(SimprintError::VersionTypeNotFound)?;

    let success = crate::models::version_types::delete_version_type(pool, payload.id).await?;

    Ok(success)
}
