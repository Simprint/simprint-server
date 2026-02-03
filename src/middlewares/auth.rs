use std::str::FromStr;

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::{
    middlewares::get_request_path_and_method,
    services,
    state::{CurrentUser, RequestContext},
    svc_ctx::SvcCtx,
};

pub async fn auth(
    State(state): State<SvcCtx>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let (resource_method, resource_path) = get_request_path_and_method(&req);

    if resource_method.eq("OPTIONS") {
        return Ok(next.run(req).await);
    }

    // 判断请求的是否在白名单，如果是白名单就直接允许访问。
    {
        let combine = format!("{}+{}", resource_method, resource_path);
        let whitelists = &state.config.app.route_whitelists;

        if whitelists.contains(&combine) {
            return Ok(next.run(req).await);
        }
    }

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| StatusCode::UNAUTHORIZED)?;

    let token_str = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    // 先判断token是否正常，如果正常再去与远程的匹配。匹配成功后返回uuid.
    let secret = state.config.app.secret.as_bytes();
    let verify_token_response = services::verify_token_service(token_str, secret);

    if let Err(_e) = verify_token_response {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let verify_token_response = verify_token_response.unwrap();

    // TODO: 根据获取到的uuid可以获取到用户并调用检查权限的方法。
    // 2025-08-27 10:05:00: 权限检查可能移除, 该网关服务只面向用户
    let uuid = if let Ok(uuid) = Uuid::from_str(&verify_token_response) {
        uuid
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    // 更新 RequestContext 中的 current_user、current_workspace 和 current_team
    if let Some(ctx) = req.extensions_mut().get_mut::<RequestContext>() {
        let ip = ctx.ip_or_unknown();
        tracing::info!("auth user uuid: {}, ip: {}", uuid, ip);

        ctx.current_user = Some(CurrentUser { user_uuid: uuid });

        // 获取用户信息（包含 current_workspace_uuid 和 current_team_uuid）
        let user_info = crate::models::user::fetch_user_info_by_uuid(&state.db, uuid)
            .await
            .ok()
            .flatten();

        // 初始化 current_team_uuid 和 current_workspace_uuid
        if let Some(user_info) = user_info {
            ctx.current_team_uuid = user_info.current_team_uuid;

            // 优先使用 current_workspace_uuid
            // 如果不存在，尝试从当前团队获取工作空间
            ctx.current_workspace_uuid = if let Some(ws_uuid) = user_info.current_workspace_uuid {
                Some(ws_uuid)
            } else if let Some(team_uuid) = user_info.current_team_uuid {
                // 从当前团队获取工作空间
                crate::models::teams::fetch_team_by_uuid(&state.db, team_uuid)
                    .await
                    .ok()
                    .flatten()
                    .map(|team| team.workspace_uuid)
            } else {
                None
            };
        }
    }

    Ok(next.run(req).await)
}
