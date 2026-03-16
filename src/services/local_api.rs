use chrono::{Timelike, Utc};
use uuid::Uuid;

use crate::dto::{LocalApiConfigDto, ResetLocalApiKeyDto, ValidateLocalApiKeyDto};
use crate::entitys::{UpdateLocalApiConfigRequest, ValidateLocalApiKeyRequest};
use crate::models;
use crate::svc_ctx::SvcCtx;

const DEFAULT_DAILY_LIMIT: i32 = 1000;

pub async fn get_local_api_config_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<LocalApiConfigDto, String> {
    init_local_api_for_user_service(svc_ctx, user_uuid).await?;

    let settings = models::local_api::fetch_local_api_settings(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "API 服务配置不存在".to_string())?;

    let mut api_key = models::local_api::fetch_active_api_key(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "API 密钥不存在".to_string())?;

    if api_key.api_key.is_none() {
        rotate_local_api_key_for_user(svc_ctx, user_uuid).await?;
        api_key = models::local_api::fetch_active_api_key(&svc_ctx.db, user_uuid)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "API 密钥不存在".to_string())?;
    }

    Ok(LocalApiConfigDto {
        enabled: settings.enabled,
        api_key: api_key
            .api_key
            .clone()
            .ok_or_else(|| "API 密钥不存在".to_string())?,
        port: settings.port,
        remote_access: settings.remote_access,
        cors_origins: models::local_api::parse_cors_origins(&settings.cors_origins),
        requests_today: api_key.requests_today,
        daily_limit: api_key.daily_limit,
    })
}

pub async fn update_local_api_config_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &UpdateLocalApiConfigRequest,
) -> Result<LocalApiConfigDto, String> {
    if let Some(port) = payload.port {
        if !(1..=65535).contains(&port) {
            return Err("端口范围无效".to_string());
        }
    }

    init_local_api_for_user_service(svc_ctx, user_uuid).await?;

    let cors_origins = payload
        .cors_origins
        .as_ref()
        .map(|origins| models::local_api::build_cors_origins_value(origins));

    models::local_api::upsert_local_api_settings(
        &svc_ctx.db,
        user_uuid,
        payload.enabled,
        payload.port,
        payload.remote_access,
        cors_origins.as_ref(),
    )
    .await
    .map_err(|e| e.to_string())?;

    get_local_api_config_service(svc_ctx, user_uuid).await
}

pub async fn reset_local_api_key_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<ResetLocalApiKeyDto, String> {
    init_local_api_for_user_service(svc_ctx, user_uuid).await?;

    models::local_api::deactivate_api_keys_for_user(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    let api_key = generate_api_key();
    let key_hash = models::local_api::hash_api_key(&api_key);
    let key_prefix = api_key.chars().take(16).collect::<String>();
    let created_key = models::local_api::insert_api_key(
        &svc_ctx.db,
        user_uuid,
        &key_prefix,
        &key_hash,
        &api_key,
        DEFAULT_DAILY_LIMIT,
    )
    .await
    .map_err(|e| e.to_string())?;

    ensure_default_permissions(svc_ctx, created_key.id).await?;

    Ok(ResetLocalApiKeyDto {
        api_key,
    })
}

pub async fn validate_local_api_key_service(
    svc_ctx: &SvcCtx,
    payload: &ValidateLocalApiKeyRequest,
) -> Result<ValidateLocalApiKeyDto, String> {
    let key_hash = models::local_api::hash_api_key(payload.api_key.as_str());
    let api_key = models::local_api::fetch_api_key_by_hash(&svc_ctx.db, &key_hash)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "API 密钥无效".to_string())?;

    if !api_key.is_active {
        return Err("API 密钥已停用".to_string());
    }

    if let Some(expires_at) = api_key.expires_at {
        if expires_at < Utc::now() {
            return Err("API 密钥已过期".to_string());
        }
    }

    let today = Utc::now().date_naive();
    if api_key.last_reset_date != today {
        models::local_api::reset_api_key_daily_usage(&svc_ctx.db, api_key.id, today)
            .await
            .map_err(|e| e.to_string())?;
    }

    let refreshed_key = models::local_api::fetch_api_key_by_hash(&svc_ctx.db, &key_hash)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "API 密钥无效".to_string())?;

    if refreshed_key.requests_today >= refreshed_key.daily_limit {
        return Err("API 密钥已达到今日调用上限".to_string());
    }

    let definition = models::local_api::fetch_permission_definition(
        &svc_ctx.db,
        &payload.permission_code,
    )
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "未知的 permissionCode".to_string())?;

    let permission = fetch_effective_permission(svc_ctx, refreshed_key.id, &payload.permission_code)
        .await?
        .ok_or_else(|| "当前密钥无权访问该接口".to_string())?;

    if !permission.is_enabled {
        return Err("当前密钥已被禁止访问该接口".to_string());
    }

    let now = Utc::now();
    let minute_start = now
        .with_second(0)
        .and_then(|dt| dt.with_nanosecond(0))
        .ok_or_else(|| "时间窗口计算失败".to_string())?;
    let hour_start = minute_start
        .with_minute(0)
        .ok_or_else(|| "时间窗口计算失败".to_string())?;
    let day_start = hour_start
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| "时间窗口计算失败".to_string())?
        .and_utc();

    let minute_count = models::local_api::fetch_request_count(
        &svc_ctx.db,
        refreshed_key.id,
        payload.permission_code.as_str(),
        "minute",
        minute_start,
    )
    .await
    .map_err(|e| e.to_string())?;
    if minute_count >= permission.rate_limit_per_minute {
        return Err("接口每分钟调用次数已达上限".to_string());
    }

    let hour_count = models::local_api::fetch_request_count(
        &svc_ctx.db,
        refreshed_key.id,
        payload.permission_code.as_str(),
        "hour",
        hour_start,
    )
    .await
    .map_err(|e| e.to_string())?;
    if hour_count >= permission.rate_limit_per_hour {
        return Err("接口每小时调用次数已达上限".to_string());
    }

    models::local_api::increment_request_counter(
        &svc_ctx.db,
        refreshed_key.id,
        payload.permission_code.as_str(),
        "minute",
        minute_start,
    )
    .await
    .map_err(|e| e.to_string())?;
    models::local_api::increment_request_counter(
        &svc_ctx.db,
        refreshed_key.id,
        payload.permission_code.as_str(),
        "hour",
        hour_start,
    )
    .await
    .map_err(|e| e.to_string())?;
    models::local_api::increment_request_counter(
        &svc_ctx.db,
        refreshed_key.id,
        payload.permission_code.as_str(),
        "day",
        day_start,
    )
    .await
    .map_err(|e| e.to_string())?;
    models::local_api::increment_api_key_usage(&svc_ctx.db, refreshed_key.id, now)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ValidateLocalApiKeyDto {
        valid: true,
        permission_code: definition.permission_code,
        requests_today: refreshed_key.requests_today + 1,
        daily_limit: refreshed_key.daily_limit,
        rate_limit_per_minute: permission.rate_limit_per_minute,
        rate_limit_per_hour: permission.rate_limit_per_hour,
    })
}

pub async fn init_local_api_for_user_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<(), String> {
    models::local_api::upsert_local_api_settings(
        &svc_ctx.db,
        user_uuid,
        None,
        None,
        None,
        None,
    )
    .await
    .map_err(|e| e.to_string())?;

    let current_key = models::local_api::fetch_active_api_key(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(api_key) = current_key {
        if api_key.api_key.is_none() {
            rotate_local_api_key_for_user(svc_ctx, user_uuid).await?;
            return Ok(());
        }
        ensure_default_permissions(svc_ctx, api_key.id).await?;
        return Ok(());
    }

    let api_key = generate_api_key();
    let key_hash = models::local_api::hash_api_key(&api_key);
    let key_prefix = api_key.chars().take(16).collect::<String>();

    let created_key = models::local_api::insert_api_key(
        &svc_ctx.db,
        user_uuid,
        &key_prefix,
        &key_hash,
        &api_key,
        DEFAULT_DAILY_LIMIT,
    )
    .await
    .map_err(|e| e.to_string())?;

    ensure_default_permissions(svc_ctx, created_key.id).await
}

async fn ensure_default_permissions(svc_ctx: &SvcCtx, api_key_id: i32) -> Result<(), String> {
    let definitions = models::local_api::fetch_permission_definitions(&svc_ctx.db)
        .await
        .map_err(|e| e.to_string())?;

    for definition in definitions {
        models::local_api::insert_api_key_permission(
            &svc_ctx.db,
            api_key_id,
            definition.permission_code.as_str(),
            definition.default_rate_limit_per_minute,
            definition.default_rate_limit_per_hour,
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

async fn fetch_effective_permission(
    svc_ctx: &SvcCtx,
    api_key_id: i32,
    permission_code: &str,
) -> Result<Option<crate::dto::LocalApiKeyPermissionDto>, String> {
    models::local_api::fetch_api_key_permission(&svc_ctx.db, api_key_id, permission_code)
        .await
        .map_err(|e| e.to_string())
}

fn generate_api_key() -> String {
    let raw = Uuid::new_v4().simple().to_string();
    format!("sk_local_{}", raw)
}

async fn rotate_local_api_key_for_user(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<ResetLocalApiKeyDto, String> {
    models::local_api::deactivate_api_keys_for_user(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    let api_key = generate_api_key();
    let key_hash = models::local_api::hash_api_key(&api_key);
    let key_prefix = api_key.chars().take(16).collect::<String>();

    let created_key = models::local_api::insert_api_key(
        &svc_ctx.db,
        user_uuid,
        &key_prefix,
        &key_hash,
        &api_key,
        DEFAULT_DAILY_LIMIT,
    )
    .await
    .map_err(|e| e.to_string())?;

    ensure_default_permissions(svc_ctx, created_key.id).await?;

    Ok(ResetLocalApiKeyDto { api_key })
}
