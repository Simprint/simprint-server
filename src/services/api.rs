use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::dto::ApiKeyDto;
use crate::entitys::api_service::{
    ApiKeyItem, CreateApiKeyRequest, ListApiKeysRequest, UpdateApiKeyRequest,
};
use crate::models;
use crate::svc_ctx::SvcCtx;

/// 生成随机 API 密钥
fn generate_api_key() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.random()).collect();
    hex::encode(bytes)
}

/// 计算密钥哈希
fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

/// 获取密钥前缀
fn get_key_prefix(key: &str) -> String {
    format!("sp_{}", &key[..8])
}

/// 获取 API 密钥列表
pub async fn get_api_keys_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &ListApiKeysRequest,
) -> Result<(Vec<ApiKeyItem>, i64), String> {
    let offset = (payload.pagination.page - 1) * payload.pagination.page_size;

    let keys = models::api::fetch_api_keys(
        &svc_ctx.db,
        user_uuid,
        payload.status.as_deref(),
        offset,
        payload.pagination.page_size,
    )
    .await
    .map_err(|e| e.to_string())?;

    let total =
        models::api::fetch_api_keys_count(&svc_ctx.db, user_uuid, payload.status.as_deref())
            .await
            .map_err(|e| e.to_string())?;

    let items = keys.into_iter().map(ApiKeyItem::from).collect();

    Ok((items, total))
}

/// 创建 API 密钥
pub async fn create_api_key_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &CreateApiKeyRequest,
) -> Result<(Uuid, String, String), String> {
    // 生成密钥
    let api_key = generate_api_key();
    let key_hash = hash_api_key(&api_key);
    let key_prefix = get_key_prefix(&api_key);

    // 处理权限
    let permissions = serde_json::json!(payload.permissions);

    // 处理 IP 白名单
    let ip_whitelist = payload.ip_whitelist.as_ref().map(|ips| serde_json::json!(ips));

    let uuid = models::api::insert_api_key(
        &svc_ctx.db,
        user_uuid,
        &payload.name,
        &key_hash,
        &key_prefix,
        &permissions,
        payload.rate_limit,
        payload.daily_limit,
        ip_whitelist.as_ref(),
        payload.expires_at,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok((uuid, api_key, key_prefix))
}

/// 更新 API 密钥
pub async fn update_api_key_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &UpdateApiKeyRequest,
) -> Result<(), String> {
    // 验证密钥所有权
    let key = models::api::fetch_api_key_by_uuid(&svc_ctx.db, payload.uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "API 密钥不存在".to_string())?;

    if key.user_uuid != user_uuid {
        return Err("无权操作此密钥".to_string());
    }

    let permissions = payload.permissions.as_ref().map(|p| serde_json::json!(p));
    let ip_whitelist = payload.ip_whitelist.as_ref().map(|ips| serde_json::json!(ips));

    models::api::update_api_key(
        &svc_ctx.db,
        payload.uuid,
        payload.name.as_deref(),
        permissions.as_ref(),
        payload.rate_limit,
        payload.daily_limit,
        ip_whitelist.as_ref(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// 撤销 API 密钥
pub async fn revoke_api_key_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    key_uuid: Uuid,
) -> Result<(), String> {
    // 验证密钥所有权
    let key = models::api::fetch_api_key_by_uuid(&svc_ctx.db, key_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "API 密钥不存在".to_string())?;

    if key.user_uuid != user_uuid {
        return Err("无权操作此密钥".to_string());
    }

    models::api::revoke_api_key(&svc_ctx.db, key_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 验证 API 密钥
pub async fn validate_api_key_service(
    svc_ctx: &SvcCtx,
    api_key: &str,
) -> Result<ApiKeyDto, String> {
    let key_hash = hash_api_key(api_key);

    let key = models::api::fetch_api_key_by_hash(&svc_ctx.db, &key_hash)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "无效的 API 密钥".to_string())?;

    // 检查是否过期
    if let Some(expires_at) = key.expires_at {
        if expires_at < Utc::now() {
            return Err("API 密钥已过期".to_string());
        }
    }

    // 更新使用统计
    let _ = models::api::update_api_key_usage(&svc_ctx.db, key.uuid).await;

    Ok(key)
}
