use crate::{
    dto::machine_users::MachineUser, entitys::machine_users::*, errors::SimprintError,
    svc_ctx::SvcCtx,
};
use serde_json;
use uuid::Uuid;

/// 注册机器
pub async fn register_machine(
    svc_ctx: &SvcCtx,
    request: CreateMachineUserRequest,
) -> Result<i32, SimprintError> {
    if let Err(e) = match (request.machine_code.is_empty(), request.user_uuid) {
        (true, _) => Err("机器码不能为空".to_string()),
        (_, None) => Err("用户UUID不能为空".to_string()),
        _ => Ok(""),
    } {
        return Err(SimprintError::InvalidRequest(e));
    }
    let user_uuid = request.user_uuid.unwrap_or_default();

    // 如果提供了用户UUID，检查是否已存在绑定
    let existing = crate::models::machine_users::query_machine_user_by_code_and_user(
        &svc_ctx.db,
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
            &svc_ctx.db,
            &request.machine_code,
            &user_uuid,
            version_info.as_ref(),
        )
        .await?;

        return Ok(machine.id);
    }

    // 不存在，一次性插入所有信息（包括 version_info）
    let id = crate::models::machine_users::insert_machine_user(&svc_ctx.db, &request).await?;
    Ok(id)
}

/// 根据机器码查询机器信息
pub async fn get_machine_by_code(
    svc_ctx: &SvcCtx,
    machine_code: String,
) -> Result<MachineUser, SimprintError> {
    let machine =
        crate::models::machine_users::query_machine_user_by_code(&svc_ctx.db, &machine_code)
            .await
            .map_err(|_| SimprintError::MachineNotFound)?;

    Ok(machine)
}

/// 根据ID查询机器信息
pub async fn get_machine_by_id(
    svc_ctx: &SvcCtx,
    id: i32,
) -> Result<MachineUser, SimprintError> {
    let machine = crate::models::machine_users::query_machine_user_by_id(&svc_ctx.db, id)
        .await
        .map_err(|_| SimprintError::MachineNotFound)?;

    Ok(machine)
}

/// 查询用户的机器列表
pub async fn get_machines_by_user_uuid(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<Vec<MachineUser>, SimprintError> {
    let machines =
        crate::models::machine_users::query_machines_by_user_uuid(&svc_ctx.db, &user_uuid).await?;

    Ok(machines)
}

/// 查询机器列表
pub async fn query_machines_service(
    svc_ctx: &SvcCtx,
    params: QueryMachineUserParams,
    page_num: i32,
    page_size: i32,
) -> Result<MachineUserListResponse, SimprintError> {
    let user_uuid = params
        .user_uuid
        .as_ref()
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| SimprintError::InvalidRequest("无效的用户UUID格式".to_string()))?;

    let (total, list) = crate::models::machine_users::query_machine_users(
        &svc_ctx.db,
        user_uuid.as_ref(),
        params.platform.as_deref(),
        params.status.as_deref(),
        page_num,
        page_size,
    )
    .await?;

    Ok(MachineUserListResponse { total, list })
}

/// 更新机器信息
pub async fn update_machine_info_service(
    svc_ctx: &SvcCtx,
    id: i32,
    request: UpdateMachineUserRequest,
) -> Result<bool, SimprintError> {
    // 检查机器是否存在
    crate::models::machine_users::query_machine_user_by_id(&svc_ctx.db, id)
        .await
        .map_err(|_| SimprintError::MachineNotFound)?;

    // 更新机器信息
    let success =
        crate::models::machine_users::update_machine_user(&svc_ctx.db, id, &request).await?;

    if !success {
        return Err(SimprintError::MachineNotFound);
    }

    Ok(true)
}

/// 绑定用户到机器
pub async fn bind_user_service(
    svc_ctx: &SvcCtx,
    machine_code: String,
    user_uuid: Uuid,
) -> Result<bool, SimprintError> {
    // 检查机器是否存在
    crate::models::machine_users::query_machine_user_by_code(&svc_ctx.db, &machine_code)
        .await
        .map_err(|_| SimprintError::MachineNotFound)?;

    // 绑定用户
    let success =
        crate::models::machine_users::bind_user_to_machine(&svc_ctx.db, &machine_code, &user_uuid)
            .await?;

    if !success {
        return Err(SimprintError::MachineNotFound);
    }

    Ok(true)
}

/// 解绑用户
pub async fn unbind_user_service(
    svc_ctx: &SvcCtx,
    machine_code: String,
    user_uuid: Uuid,
) -> Result<bool, SimprintError> {
    // 检查绑定是否存在
    let binding = crate::models::machine_users::query_machine_user_by_code_and_user(
        &svc_ctx.db,
        &machine_code,
        &user_uuid,
    )
    .await?;

    if binding.is_none() {
        return Err(SimprintError::MachineNotBound);
    }

    // 解绑用户
    let success = crate::models::machine_users::unbind_user_from_machine(
        &svc_ctx.db,
        &machine_code,
        &user_uuid,
    )
    .await?;

    if !success {
        return Err(SimprintError::MachineNotFound);
    }

    Ok(true)
}

/// 获取机器版本信息
pub async fn get_machine_version_info(
    svc_ctx: &SvcCtx,
    machine_code: String,
    user_uuid: Uuid,
) -> Result<serde_json::Value, SimprintError> {
    // 查询特定用户的机器记录
    let machine = crate::models::machine_users::query_machine_user_by_code_and_user(
        &svc_ctx.db,
        &machine_code,
        &user_uuid,
    )
    .await?
    .ok_or(SimprintError::MachineNotBound)?;

    Ok(machine.version_info.unwrap_or(serde_json::json!({})))
}

/// 拉黑机器
pub async fn allow_or_blacklist_machine(
    svc_ctx: &SvcCtx,
    machine_code: String,
    allow: bool,
) -> Result<bool, SimprintError> {
    let success =
        crate::models::machine_users::allow_or_blacklist_machine(&svc_ctx.db, &machine_code, allow)
            .await?;

    Ok(success)
}

/// 判断机器是否允许
pub async fn machine_not_allow(
    svc_ctx: &SvcCtx,
    machine_code: String,
) -> Result<bool, SimprintError> {
    let allow = crate::models::machine_users::machine_not_allow(&svc_ctx.db, &machine_code).await?;

    Ok(allow)
}

/// 更新机器版本信息
pub async fn update_machine_version_info_service(
    svc_ctx: &SvcCtx,
    machine_code: String,
    user_uuid: Uuid,
    version_info: serde_json::Value,
) -> Result<bool, SimprintError> {
    // 检查机器是否存在
    crate::models::machine_users::query_machine_user_by_code(&svc_ctx.db, &machine_code)
        .await
        .map_err(|_| SimprintError::MachineNotFound)?;

    // 1. 更新整机的 version_info（所有用户记录）
    let success = crate::models::machine_users::update_machine_version_info(
        &svc_ctx.db,
        &machine_code,
        &version_info,
    )
    .await?;

    if !success {
        return Err(SimprintError::MachineNotFound);
    }

    // 2. 只更新当前用户的 updated_at
    let _ =
        crate::models::machine_users::update_user_bind_time(&svc_ctx.db, &machine_code, &user_uuid)
            .await?;

    Ok(true)
}
