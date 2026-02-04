use axum::{extract::Extension, extract::State};

use crate::audit_log;
use crate::entitys::{
    AddEnvironmentCookieRequest, AddEnvironmentUrlRequest, AssignTagsRequest,
    BatchAssignTagRequest, BatchCreateEnvironmentRequest, BatchMoveToGroupRequest,
    BatchRemoveTagsRequest, BatchUuidRequest, ClearEnvironmentCookiesRequest,
    ClearEnvironmentUrlsRequest, CreateEnvironmentRequest, CreateGroupRequest, CreateResponse,
    DeleteEnvironmentCookieRequest, DeleteEnvironmentUrlRequest, EnvironmentDetailResponse,
    EnvironmentListResponse, IdResponse, ListEnvironmentsRequest, MoveToGroupRequest,
    RemoveTagRequest, SetEnvironmentAccountsRequest, SetEnvironmentProxyRequest,
    UpdateEnvironmentRequest, UpdateGroupRequest, UpdateTagRequest, UuidRequest,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

// ============ Groups ============

/// 创建分组
pub async fn create_group_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CreateGroupRequest>,
) -> Result<CreateResponse> {
    let workspace_uuid = ctx
        .current_workspace_uuid
        .ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let team_uuid = ctx.current_team_uuid.ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    let group_uuid = services::groups::create_group_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        workspace_uuid,
        team_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "create",
        "group",
        group_uuid,
        &payload.name,
        "创建分组"
    )
    .await;

    Ok(Response::success(
        Some("创建成功"),
        Some(CreateResponse { uuid: group_uuid }),
    ))
}

/// 获取分组列表
pub async fn get_groups_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<Vec<crate::dto::GroupDto>> {
    // 从当前团队获取工作空间和团队 UUID
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?
        .ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    let team = services::teams::get_team_service(&svc_ctx, team_uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let groups =
        services::groups::get_groups_service(&svc_ctx, team.workspace_uuid, team_uuid, 1, 100)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(groups)))
}

/// 更新分组
pub async fn update_group_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UpdateGroupRequest>,
) -> Result<()> {
    let workspace_uuid = ctx
        .current_workspace_uuid
        .ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let team_uuid = ctx.current_team_uuid.ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    services::groups::update_group_service(
        &svc_ctx,
        workspace_uuid,
        team_uuid,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("更新成功"), None))
}

/// 删除分组
pub async fn delete_group_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UuidRequest>,
) -> Result<()> {
    let workspace_uuid = ctx
        .current_workspace_uuid
        .ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let team_uuid = ctx.current_team_uuid.ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    services::groups::delete_group_service(
        &svc_ctx,
        workspace_uuid,
        team_uuid,
        ctx.user_uuid_unwrap(),
        payload.uuid,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(&svc_ctx, &ctx, "delete", "group", payload.uuid, "删除分组").await;

    Ok(Response::success(Some("删除成功"), None))
}

// ============ Tags ============

/// 创建标签
pub async fn create_tag_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<crate::entitys::tags::CreateTagRequest>,
) -> Result<CreateResponse> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let tag_uuid =
        services::tags::create_tag_service(&svc_ctx, ctx.user_uuid_unwrap(), team_uuid, &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "create",
        "tag",
        tag_uuid,
        &payload.name,
        "创建标签"
    )
    .await;

    Ok(Response::success(
        Some("创建成功"),
        Some(CreateResponse { uuid: tag_uuid }),
    ))
}

/// 获取标签列表
pub async fn get_tags_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<Vec<crate::dto::TagDto>> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let tags = services::tags::get_tags_service(&svc_ctx, ctx.user_uuid_unwrap(), team_uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(tags)))
}

/// 更新标签
pub async fn update_tag_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UpdateTagRequest>,
) -> Result<()> {
    services::tags::update_tag_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("更新成功"), None))
}

/// 删除标签
pub async fn delete_tag_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UuidRequest>,
) -> Result<()> {
    services::tags::delete_tag_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(&svc_ctx, &ctx, "delete", "tag", payload.uuid, "删除标签").await;

    Ok(Response::success(Some("删除成功"), None))
}

// ============ Environments ============

/// 获取环境列表
pub async fn get_environments_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ListEnvironmentsRequest>,
) -> Result<EnvironmentListResponse> {
    let workspace_uuid = ctx
        .current_workspace_uuid
        .ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let team_uuid = ctx.current_team_uuid.ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    let (environments, total) = services::environments::get_environments_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        workspace_uuid,
        team_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(EnvironmentListResponse {
            items: environments,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}

/// 获取环境详情
pub async fn get_environment_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UuidRequest>,
) -> Result<EnvironmentDetailResponse> {
    let workspace_uuid = ctx
        .current_workspace_uuid
        .ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let team_uuid = ctx.current_team_uuid.ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    let environment = services::environments::get_environment_service(
        &svc_ctx,
        workspace_uuid,
        team_uuid,
        ctx.user_uuid_unwrap(),
        payload.uuid,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    let config = services::environments::get_environment_config_service(&svc_ctx, payload.uuid)
        .await
        .ok();

    let tags = services::environments::get_environment_tags_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let accounts = services::accounts::get_environment_accounts_service(&svc_ctx, payload.uuid)
        .await
        .unwrap_or_default();

    // 获取分组信息
    let group = if let Some(group_uuid) = environment.group_uuid {
        services::environments::get_group_summary_service(&svc_ctx, group_uuid)
            .await
            .ok()
    } else {
        None
    };

    // 获取代理信息
    let proxy = if let Some(proxy_uuid) = environment.proxy_uuid {
        services::environments::get_proxy_summary_service(&svc_ctx, proxy_uuid)
            .await
            .ok()
    } else {
        None
    };

    Ok(Response::success(
        Some("获取成功"),
        Some(EnvironmentDetailResponse {
            environment,
            config,
            tags,
            accounts,
            group,
            proxy,
        }),
    ))
}

/// 创建环境
pub async fn create_environment_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CreateEnvironmentRequest>,
) -> Result<CreateResponse> {
    let workspace_uuid = ctx
        .current_workspace_uuid
        .ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let team_uuid = ctx.current_team_uuid.ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    let env_uuid = services::environments::create_environment_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        workspace_uuid,
        team_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "create",
        "environment",
        env_uuid,
        &payload.name,
        "创建环境"
    )
    .await;

    Ok(Response::success(
        Some("创建成功"),
        Some(CreateResponse { uuid: env_uuid }),
    ))
}

/// 批量创建环境
pub async fn batch_create_environments_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<BatchCreateEnvironmentRequest>,
) -> Result<Vec<CreateResponse>> {
    let workspace_uuid = ctx
        .current_workspace_uuid
        .ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let team_uuid = ctx.current_team_uuid.ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    let env_uuids = services::environments::batch_create_environments_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        workspace_uuid,
        team_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    let count = env_uuids.len();
    let responses: Vec<CreateResponse> =
        env_uuids.into_iter().map(|uuid| CreateResponse { uuid }).collect();

    audit_log!(
        &svc_ctx,
        &ctx,
        "batch_create",
        "environment",
        &format!("批量创建 {} 个环境", count)
    )
    .await;

    Ok(Response::success(Some("批量创建成功"), Some(responses)))
}

/// 更新环境
pub async fn update_environment_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UpdateEnvironmentRequest>,
) -> Result<()> {
    let workspace_uuid = ctx
        .current_workspace_uuid
        .ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let team_uuid = ctx.current_team_uuid.ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    services::environments::update_environment_service(
        &svc_ctx,
        workspace_uuid,
        team_uuid,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("更新成功"), None))
}

/// 删除环境
pub async fn delete_environment_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UuidRequest>,
) -> Result<()> {
    let workspace_uuid = ctx
        .current_workspace_uuid
        .ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let team_uuid = ctx.current_team_uuid.ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    services::environments::delete_environment_service(
        &svc_ctx,
        workspace_uuid,
        team_uuid,
        ctx.user_uuid_unwrap(),
        payload.uuid,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "delete",
        "environment",
        payload.uuid,
        "删除环境"
    )
    .await;

    Ok(Response::success(Some("删除成功"), None))
}

/// 批量删除环境
pub async fn batch_delete_environments_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<BatchUuidRequest>,
) -> Result<u64> {
    let count = services::environments::batch_delete_environments_service(&svc_ctx, &payload.uuids)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "batch_delete",
        "environment",
        &format!("批量删除 {} 个环境", count)
    )
    .await;

    Ok(Response::success(Some("删除成功"), Some(count)))
}

/// 设置环境代理
pub async fn set_proxy_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<SetEnvironmentProxyRequest>,
) -> Result<()> {
    services::environments::set_environment_proxy_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("设置成功"), None))
}

/// 分配标签
pub async fn assign_tags_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<AssignTagsRequest>,
) -> Result<()> {
    services::environments::assign_tags_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("分配成功"), None))
}

/// 移除标签
pub async fn remove_tag_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<RemoveTagRequest>,
) -> Result<()> {
    services::environments::remove_tag_service(&svc_ctx, payload.uuid, payload.tag_uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("移除成功"), None))
}

/// 移动到分组
pub async fn move_to_group_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<MoveToGroupRequest>,
) -> Result<()> {
    services::environments::move_to_group_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("移动成功"), None))
}

/// 批量移动到分组
pub async fn batch_move_to_group_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<BatchMoveToGroupRequest>,
) -> Result<()> {
    services::environments::batch_move_to_group_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("移动成功"), None))
}

/// 设置环境账号
pub async fn set_environment_accounts_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<SetEnvironmentAccountsRequest>,
) -> Result<()> {
    services::accounts::set_environment_accounts_service(
        &svc_ctx,
        payload.uuid,
        &payload.account_uuids,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("设置成功"), None))
}

/// 批量分配标签
pub async fn batch_assign_tags_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<BatchAssignTagRequest>,
) -> Result<()> {
    services::environments::batch_assign_tags_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("分配成功"), None))
}

/// 批量移除标签
pub async fn batch_remove_tags_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<BatchRemoveTagsRequest>,
) -> Result<()> {
    services::environments::batch_remove_tags_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("移除成功"), None))
}

// ============ Environment URLs ============

/// 添加环境 URL
pub async fn add_environment_url_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<AddEnvironmentUrlRequest>,
) -> Result<IdResponse> {
    let id = services::environments::add_environment_url_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("添加成功"), Some(IdResponse { id })))
}

/// 获取环境 URL 列表
pub async fn get_environment_urls_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UuidRequest>,
) -> Result<Vec<crate::dto::EnvironmentUrlDto>> {
    let urls = services::environments::get_environment_urls_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(urls)))
}

/// 删除环境 URL
pub async fn delete_environment_url_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<DeleteEnvironmentUrlRequest>,
) -> Result<()> {
    services::environments::delete_environment_url_service(&svc_ctx, payload.id)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("删除成功"), None))
}

/// 清空环境 URL
pub async fn clear_environment_urls_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<ClearEnvironmentUrlsRequest>,
) -> Result<u64> {
    let count =
        services::environments::clear_environment_urls_service(&svc_ctx, payload.environment_uuid)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("清空成功"), Some(count)))
}

// ============ Environment Cookies ============

/// 添加环境 Cookie
pub async fn add_environment_cookie_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<AddEnvironmentCookieRequest>,
) -> Result<IdResponse> {
    let id = services::environments::add_environment_cookie_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("添加成功"), Some(IdResponse { id })))
}

/// 获取环境 Cookie 列表
pub async fn get_environment_cookies_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UuidRequest>,
) -> Result<Vec<crate::dto::EnvironmentCookieDto>> {
    let cookies = services::environments::get_environment_cookies_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(cookies)))
}

/// 删除环境 Cookie
pub async fn delete_environment_cookie_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<DeleteEnvironmentCookieRequest>,
) -> Result<()> {
    services::environments::delete_environment_cookie_service(&svc_ctx, payload.id)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("删除成功"), None))
}

/// 清空环境 Cookies
pub async fn clear_environment_cookies_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<ClearEnvironmentCookiesRequest>,
) -> Result<u64> {
    let count = services::environments::clear_environment_cookies_service(
        &svc_ctx,
        payload.environment_uuid,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("清空成功"), Some(count)))
}
