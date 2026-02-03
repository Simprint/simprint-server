use crate::{
    dto::machine_users::MachineUser, entitys::machine_users::*, errors::SimprintError,
    svc_ctx::SvcCtx,
};
use serde_json;
use uuid::Uuid;

/// 注册机器
pub async fn register_machine_service(
    svc_ctx: &SvcCtx,
    request: CreateMachineUserRequest,
) -> Result<i32, SimprintError> {
    let pool = &svc_ctx.db;
    if request.machine_code.is_empty() {
        return Err(SimprintError::InvalidRequest("机器码不能为空".to_string()));
    }

    // 如果提供了用户UUID，检查是否已存在绑定
    if let Some(user_uuid) = request.user_uuid {
        let existing = crate::models::machine_users::query_machine_user_by_code_and_user(
            pool,
            &request.machine_code,
            &user_uuid,
        )
        .await?;

        if let Some(machine) = existing {
            // 已存在，一次性更新绑定时间和版本信息
            let version_info = request
                .version_info
                .as_ref()
                .and_then(|v| serde_json::from_str::<serde_json::Value>(v).ok());

            crate::models::machine_users::update_machine_bind_and_version(
                pool,
                &request.machine_code,
                &user_uuid,
                version_info.as_ref(),
            )
            .await?;

            return Ok(machine.id);
        }
    }

    // 不存在，一次性插入所有信息（包括 version_info）
    let id = crate::models::machine_users::insert_machine_user(pool, &request).await?;
    Ok(id)
}

/// 根据机器码查询机器信息
pub async fn get_machine_by_code_service(
    svc_ctx: &SvcCtx,
    machine_code: String,
) -> Result<MachineUser, SimprintError> {
    let pool = &svc_ctx.db;
    let machine = crate::models::machine_users::query_machine_user_by_code(pool, &machine_code)
        .await
        .map_err(|_| SimprintError::MachineNotFound)?;

    Ok(machine)
}

/// 根据ID查询机器信息
pub async fn get_machine_by_id_service(
    svc_ctx: &SvcCtx,
    id: i32,
) -> Result<MachineUser, SimprintError> {
    let pool = &svc_ctx.db;
    let machine = crate::models::machine_users::query_machine_user_by_id(pool, id)
        .await?
        .ok_or(SimprintError::MachineNotFound)?;

    Ok(machine)
}

/// 查询用户的机器列表
pub async fn get_machines_by_user_uuid_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<Vec<MachineUser>, SimprintError> {
    let pool = &svc_ctx.db;

    let machines =
        crate::models::machine_users::query_machines_by_user_uuid(pool, &user_uuid).await?;

    Ok(machines)
}

/// 查询机器列表
pub async fn query_machines_service(
    svc_ctx: &SvcCtx,
    params: QueryMachinesParams,
) -> Result<MachineUserListResponse, SimprintError> {
    let pool = &svc_ctx.db;

    let user_uuid = params
        .user_uuid
        .as_ref()
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| SimprintError::InvalidRequest("无效的用户UUID格式".to_string()))?;

    let (total, list) = crate::models::machine_users::query_machine_users(
        pool,
        user_uuid.as_ref(),
        params.platform.as_deref(),
        params.status.as_deref(),
        params.page_num.unwrap_or(1),
        params.page_size.unwrap_or(20),
    )
    .await?;

    Ok(MachineUserListResponse { total, list })
}

/// 更新机器信息
pub async fn update_machine_info_service(
    svc_ctx: &SvcCtx,
    request: UpdateMachineUserRequest,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;

    // 检查机器是否存在
    crate::models::machine_users::query_machine_user_by_id(pool, request.id)
        .await?
        .ok_or(SimprintError::MachineNotFound)?;

    // 更新机器信息
    let success =
        crate::models::machine_users::update_machine_user(pool, request.id, &request).await?;

    if !success {
        return Err(SimprintError::MachineNotFound);
    }

    Ok(true)
}

/// 绑定用户到机器
pub async fn bind_user_service(
    svc_ctx: &SvcCtx,
    payload: BindMachineRequest,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;

    // 检查机器是否存在
    crate::models::machine_users::query_machine_user_by_code(pool, &payload.machine_code)
        .await
        .map_err(|_| SimprintError::MachineNotFound)?;

    // 绑定用户
    let success = crate::models::machine_users::bind_user_to_machine(
        pool,
        &payload.machine_code,
        &payload.user_uuid,
    )
    .await?;

    if !success {
        return Err(SimprintError::MachineNotFound);
    }

    Ok(true)
}

/// 解绑用户
pub async fn unbind_user_service(
    svc_ctx: &SvcCtx,
    payload: BindMachineRequest,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;

    // 检查绑定是否存在
    let binding = crate::models::machine_users::query_machine_user_by_code_and_user(
        pool,
        &payload.machine_code,
        &payload.user_uuid,
    )
    .await?;

    if binding.is_none() {
        return Err(SimprintError::MachineNotBound);
    }

    // 解绑用户
    let success = crate::models::machine_users::unbind_user_from_machine(
        pool,
        &payload.machine_code,
        &payload.user_uuid,
    )
    .await?;

    if !success {
        return Err(SimprintError::MachineNotFound);
    }

    Ok(true)
}

/// 获取机器版本信息
pub async fn get_machine_version_info_service(
    svc_ctx: &SvcCtx,
    machine_code: String,
    user_uuid: Uuid,
) -> Result<serde_json::Value, SimprintError> {
    let pool = &svc_ctx.db;

    // 查询特定用户的机器记录
    let machine = crate::models::machine_users::query_machine_user_by_code_and_user(
        pool,
        &machine_code,
        &user_uuid,
    )
    .await?
    .ok_or(SimprintError::MachineNotBound)?;

    Ok(machine.version_info.unwrap_or(serde_json::json!({})))
}

/// 拉黑机器
pub async fn allow_or_blacklist_machine_service(
    svc_ctx: &SvcCtx,
    payload: AllowOrBlacklistMachineRequest,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;
    let success = crate::models::machine_users::allow_or_blacklist_machine(
        pool,
        &payload.machine_code,
        payload.allow,
    )
    .await?;

    Ok(success)
}

/// 判断机器是否允许
pub async fn machine_not_allow_service(
    svc_ctx: &SvcCtx,
    machine_code: String,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;
    let allow = crate::models::machine_users::machine_not_allow(pool, &machine_code).await?;

    Ok(allow)
}

/// 更新机器版本信息
pub async fn update_machine_version_info_service(
    svc_ctx: &SvcCtx,
    payload: UpdateMachineVersionRequest,
) -> Result<bool, SimprintError> {
    let pool = &svc_ctx.db;

    // 检查机器是否存在
    crate::models::machine_users::query_machine_user_by_code(pool, &payload.machine_code)
        .await
        .map_err(|_| SimprintError::MachineNotFound)?;

    // 1. 更新整机的 version_info（所有用户记录）
    let success = crate::models::machine_users::update_machine_version_info(
        pool,
        &payload.machine_code,
        &payload.version_info,
    )
    .await?;

    if !success {
        return Err(SimprintError::MachineNotFound);
    }

    // 2. 只更新当前用户的 updated_at
    let _ = crate::models::machine_users::update_user_bind_time(
        pool,
        &payload.machine_code,
        &payload.user_uuid,
    )
    .await?;

    Ok(true)
}
