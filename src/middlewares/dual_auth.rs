//! 双重认证中间件
//!
//! 支持 API Key 认证和 JWT Token 认证
//! - 优先检查 X-API-KEY header
//! - 如果没有 API Key，则使用 JWT 认证

use std::str::FromStr;

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::warn;
use uuid::Uuid;

use crate::{
    middlewares::get_request_path_and_method,
    models::apikey,
    services,
    state::{CurrentIpAddr, CurrentUser},
    svc_ctx::SvcCtx,
};

/// 双重认证中间件：支持 JWT 认证和 API Key 认证
///
/// 认证流程：
/// 1. 检查是否为 OPTIONS 请求或白名单路由，直接放行
/// 2. 检查 X-API-KEY header，验证 API Key
/// 3. 如果没有 API Key，则使用 JWT 认证
///
/// API Key 格式：`key_id:key_secret`
pub async fn dual_auth(State(state): State<SvcCtx>, mut req: Request, next: Next) -> Response {
    let (resource_method, resource_path) = get_request_path_and_method(&req);

    // OPTIONS 请求直接通过
    if resource_method.eq("OPTIONS") {
        return next.run(req).await;
    }

    // 判断请求的是否在白名单，如果是白名单就直接允许访问
    {
        let combine = format!("{}+{}", resource_method, resource_path);
        let whitelists = &state.config.app.route_whitelists;

        if whitelists.contains(&combine) {
            return next.run(req).await;
        }
    }

    // 首先检查是否有 API Key
    if let Some(api_key_header) = req.headers().get("X-API-KEY") {
        if let Ok(key_str) = api_key_header.to_str() {
            let parts: Vec<&str> = key_str.split(':').collect();

            if parts.len() == 2 {
                let key_id = parts[0];
                let key_secret = parts[1];

                match apikey::validate_api_key(&state.db, key_id, key_secret).await {
                    Ok(Some(api_key)) => {
                        tracing::info!(
                            "API Key authentication successful: {} ({})",
                            api_key.name,
                            key_id
                        );
                        // API Key 验证成功，直接继续（不注入 CurrentUser）
                        return next.run(req).await;
                    }
                    Ok(None) => {
                        warn!("Invalid or expired API key: {}", key_id);
                        return (StatusCode::UNAUTHORIZED, "Invalid or expired API key")
                            .into_response();
                    }
                    Err(e) => {
                        warn!("Database error during API key validation: {:?}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Authentication service error",
                        )
                            .into_response();
                    }
                }
            } else {
                warn!("Invalid API key format - should be key_id:key_secret");
                return (StatusCode::UNAUTHORIZED, "Invalid API key format").into_response();
            }
        } else {
            warn!("Invalid API key header encoding");
            return (StatusCode::UNAUTHORIZED, "Invalid API key format").into_response();
        }
    }

    // 如果没有 API Key，则使用 JWT 认证
    let auth_header =
        match req.headers().get(header::AUTHORIZATION).and_then(|header| header.to_str().ok()) {
            Some(header) => header,
            None => {
                return (StatusCode::UNAUTHORIZED, "Missing authentication").into_response();
            }
        };

    let token_str = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else {
        return (
            StatusCode::UNAUTHORIZED,
            "Invalid authorization header format",
        )
            .into_response();
    };

    // 验证 JWT Token
    let secret = state.config.app.secret.as_bytes();
    let verify_token_response = services::verify_token_service(token_str, secret);

    if let Err(_e) = verify_token_response {
        return (StatusCode::UNAUTHORIZED, "Invalid or expired token").into_response();
    }

    let verify_token_response = verify_token_response.unwrap();

    // 解析用户 UUID
    let uuid = match Uuid::from_str(&verify_token_response) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (StatusCode::UNAUTHORIZED, "Invalid user identifier").into_response();
        }
    };

    // 记录认证信息
    if let Some(current_ip) = req.extensions().get::<CurrentIpAddr>() {
        let ip = current_ip.real_ip.as_str();
        tracing::info!("JWT auth user uuid: {}, ip: {}", uuid, ip);
    }

    // 注入当前用户信息（只有 JWT 认证才会注入）
    req.extensions_mut().insert(CurrentUser { user_uuid: uuid });
    next.run(req).await
}
