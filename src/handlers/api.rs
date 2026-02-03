use axum::extract::{Extension, State};

use crate::entitys::api_service::{
    ApiKeysListResponse, CreateApiKeyRequest, CreateApiKeyResponse, ListApiKeysRequest,
    RevokeApiKeyRequest, UpdateApiKeyRequest,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 获取 API 密钥列表
pub async fn get_api_keys_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ListApiKeysRequest>,
) -> Result<ApiKeysListResponse> {
    let (items, total) =
        services::api::get_api_keys_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(ApiKeysListResponse {
            items,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}

/// 创建 API 密钥
pub async fn create_api_key_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<CreateApiKeyResponse> {
    let (uuid, api_key, key_prefix) =
        services::api::create_api_key_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("创建成功，请妥善保存密钥"),
        Some(CreateApiKeyResponse {
            uuid,
            api_key,
            key_prefix,
        }),
    ))
}

/// 更新 API 密钥
pub async fn update_api_key_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UpdateApiKeyRequest>,
) -> Result<()> {
    services::api::update_api_key_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("更新成功"), None))
}

/// 撤销 API 密钥
pub async fn revoke_api_key_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<RevokeApiKeyRequest>,
) -> Result<()> {
    services::api::revoke_api_key_service(&svc_ctx, ctx.user_uuid_unwrap(), payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("撤销成功"), None))
}
