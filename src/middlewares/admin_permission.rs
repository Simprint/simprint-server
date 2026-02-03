//! 管理员权限检查中间件
//!
//! 对通过 JWT 认证的用户进行细粒度权限检查
//! API Key 认证的请求直接通过（不进行权限检查）

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::{
    middlewares::get_request_path_and_method, models::admin, state::CurrentUser, svc_ctx::SvcCtx,
};

/// 管理员权限检查中间件
///
/// 认证逻辑：
/// 1. 检查是否为 OPTIONS 请求，直接放行
/// 2. 检查是否有 CurrentUser 扩展（只有 JWT 认证才会有）
///    - 有：检查用户是否为管理员且拥有访问该路由的权限
///    - 无：说明是 API Key 认证，直接放行
///
/// 权限检查：
/// - 通过 user_uuid 查询 console_admins 表确认是否为管理员
/// - 通过 console_admin_permissions 和 console_permissions 表检查权限
pub async fn admin_permission_check(
    State(state): State<SvcCtx>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let (method, path) = get_request_path_and_method(&req);

    // OPTIONS 请求直接通过
    if method.eq("OPTIONS") {
        return Ok(next.run(req).await);
    }

    // 检查是否有当前用户信息（只有 JWT 认证的请求才会有）
    if let Some(current_user) = req.extensions().get::<CurrentUser>() {
        let user_uuid = current_user.user_uuid;

        // 检查用户是否为管理员以及是否有访问权限
        match admin::check_admin_permission(&state.db, user_uuid, path, method).await {
            Ok(has_permission) => {
                if !has_permission {
                    tracing::warn!(
                        "Admin permission denied for user {} on {} {}",
                        user_uuid,
                        method,
                        path
                    );
                    return Err(StatusCode::FORBIDDEN);
                }
            }
            Err(e) => {
                tracing::error!("Failed to check admin permission: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }

        tracing::debug!(
            "Admin permission granted for user {} on {} {}",
            user_uuid,
            method,
            path
        );
    }
    // 如果没有 CurrentUser 扩展，说明是 API Key 认证，直接通过

    Ok(next.run(req).await)
}
