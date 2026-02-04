use crate::services::{get_machine_by_code, machine_not_allow};
// use crate::services::{get_machine_by_code_service, machine_not_allow_service};
use crate::state::{CurrentMachineUser, RequestContext};
use crate::svc_ctx::SvcCtx;
use axum::{extract::Request, extract::State, middleware::Next, response::Response};

/// 机器码认证中间件
/// 从请求头 x-machine-code 中提取机器码，并验证机器是否允许使用
pub async fn machine_auth(
    State(svc_ctx): State<SvcCtx>,
    mut request: Request,
    next: Next,
) -> Response {
    // 从请求头获取机器码
    let machine_code = request
        .headers()
        .get("x-machine-code")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if let Some(code) = machine_code {
        // 检查机器是否被拉黑
        let current_machine = match machine_not_allow(&svc_ctx, code.clone()).await {
            Ok(true) => {
                // 机器被拉黑，但继续处理（业务层决定如何处理）
                CurrentMachineUser {
                    machine_code: code,
                    platform: None,
                    user_uuid: None,
                    is_allowed: false,
                }
            }
            Ok(false) => {
                // 机器未被拉黑，尝试获取机器信息
                match get_machine_by_code(&svc_ctx, code.clone()).await {
                    Ok(machine) => CurrentMachineUser {
                        machine_code: code,
                        platform: machine.platform,
                        user_uuid: machine.user_uuid,
                        is_allowed: true,
                    },
                    Err(_) => {
                        // 机器不存在，创建默认的机器用户信息
                        CurrentMachineUser {
                            machine_code: code,
                            platform: None,
                            user_uuid: None,
                            is_allowed: true,
                        }
                    }
                }
            }
            Err(_) => {
                // 查询出错，创建默认的机器用户信息
                CurrentMachineUser {
                    machine_code: code,
                    platform: None,
                    user_uuid: None,
                    is_allowed: true,
                }
            }
        };

        // 更新 RequestContext 中的 current_machine_user
        if let Some(ctx) = request.extensions_mut().get_mut::<RequestContext>() {
            ctx.current_machine_user = Some(current_machine);
        }
    }

    next.run(request).await
}
