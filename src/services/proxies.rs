use uuid::Uuid;

use crate::dto::ProxyDto;
use crate::entitys::{
    BatchImportProxiesRequest, CreateProxyRequest, ListProxiesRequest, UpdateProxyRequest,
};
use crate::models;
use crate::svc_ctx::SvcCtx;
use crate::utils::encryption::{encrypt_password, get_encryption_key};

/// 创建代理
pub async fn create_proxy_service(
    svc_ctx: &SvcCtx,
    owner_uuid: Uuid,
    workspace_uuid: Uuid,
    payload: &CreateProxyRequest,
) -> Result<Uuid, String> {
    // 1. 检查工作空间代理配额是否充足
    let quota_available = models::check_quota(&svc_ctx.db, workspace_uuid, "proxies")
        .await
        .map_err(|e| e.to_string())?;
    if !quota_available {
        return Err("工作空间代理配额不足，无法创建新代理".to_string());
    }

    // 2. 加密密码
    let password_encrypted = if let Some(password) = &payload.password {
        let key = get_encryption_key();
        Some(encrypt_password(password, &key)?)
    } else {
        None
    };

    // 3. 创建代理
    let proxy_uuid = models::insert_proxy(
        &svc_ctx.db,
        workspace_uuid,
        owner_uuid,
        &payload.name,
        &payload.host,
        payload.port,
        &payload.proxy_type,
        payload.username.as_deref(),
        password_encrypted.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())?;

    // 4. 更新工作空间配额（创建后增加使用数）
    models::increment_used_proxies(&svc_ctx.db, workspace_uuid, 1)
        .await
        .map_err(|e| format!("更新配额失败: {}", e))?;

    Ok(proxy_uuid)
}

/// 获取代理列表（根据可见性过滤）
pub async fn get_proxies_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    workspace_uuid: Uuid,
    payload: &ListProxiesRequest,
) -> Result<(Vec<ProxyDto>, i64), String> {
    // 获取用户当前团队（用于过滤可见的代理）
    let current_team_uuid = models::fetch_user_current_team(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 使用可见性过滤获取代理列表
    let proxies = models::fetch_visible_proxies_for_user(
        &svc_ctx.db,
        workspace_uuid,
        user_uuid,
        current_team_uuid,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 应用类型和状态过滤
    let proxy_type = payload.filters.as_ref().and_then(|f| f.proxy_type.as_deref());
    let status = payload.filters.as_ref().and_then(|f| f.status.as_deref());

    let mut filtered_proxies: Vec<ProxyDto> = proxies
        .into_iter()
        .filter(|p| {
            (proxy_type.is_none() || p.proxy_type == proxy_type.unwrap())
                && (status.is_none() || p.status == status.unwrap())
        })
        .collect();

    // 应用分页
    let total = filtered_proxies.len() as i64;
    let offset = ((payload.pagination.page - 1) * payload.pagination.page_size) as usize;
    let limit = payload.pagination.page_size as usize;
    let paginated_proxies = if offset < filtered_proxies.len() {
        filtered_proxies
            .drain(offset..std::cmp::min(offset + limit, filtered_proxies.len()))
            .collect()
    } else {
        vec![]
    };

    Ok((paginated_proxies, total))
}

/// 获取代理详情
pub async fn get_proxy_service(svc_ctx: &SvcCtx, proxy_uuid: Uuid) -> Result<ProxyDto, String> {
    models::fetch_proxy_by_uuid(&svc_ctx.db, proxy_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "代理不存在".to_string())
}

/// 更新代理
pub async fn update_proxy_service(
    svc_ctx: &SvcCtx,
    payload: &UpdateProxyRequest,
) -> Result<(), String> {
    // 加密密码
    let password_encrypted = if let Some(password) = &payload.password {
        let key = get_encryption_key();
        Some(encrypt_password(password, &key)?)
    } else {
        None
    };

    models::update_proxy(
        &svc_ctx.db,
        payload.uuid,
        payload.name.as_deref(),
        payload.host.as_deref(),
        payload.port,
        payload.proxy_type.as_deref(),
        payload.username.as_deref(),
        password_encrypted.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// 删除代理
pub async fn delete_proxy_service(svc_ctx: &SvcCtx, proxy_uuid: Uuid) -> Result<(), String> {
    // 1. 获取代理信息（用于获取 workspace_uuid）
    let proxy = models::fetch_proxy_by_uuid(&svc_ctx.db, proxy_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "代理不存在".to_string())?;

    let workspace_uuid = proxy.workspace_uuid;

    // 2. 删除代理
    models::delete_proxy(&svc_ctx.db, proxy_uuid).await.map_err(|e| e.to_string())?;

    // 3. 更新工作空间配额（删除后减少使用数）
    models::decrement_used_proxies(&svc_ctx.db, workspace_uuid, 1)
        .await
        .map_err(|e| format!("更新配额失败: {}", e))?;

    Ok(())
}

/// 批量删除代理
pub async fn batch_delete_proxies_service(
    svc_ctx: &SvcCtx,
    proxy_uuids: &[Uuid],
) -> Result<u64, String> {
    models::batch_delete_proxies(&svc_ctx.db, proxy_uuids)
        .await
        .map_err(|e| e.to_string())
}

/// 批量导入代理
///
/// 接收客户端已解析好的代理列表，直接保存到数据库
pub async fn batch_import_proxies_service(
    svc_ctx: &SvcCtx,
    owner_uuid: Uuid,
    workspace_uuid: Uuid,
    payload: &BatchImportProxiesRequest,
) -> Result<crate::entitys::BatchImportResponse, String> {
    // 1. 检查工作空间代理配额是否充足（检查是否有足够配额导入所有代理）
    let import_count = payload.proxies.len() as i32;
    let quota = models::fetch_workspace_quota(&svc_ctx.db, workspace_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "工作空间配额不存在".to_string())?;

    if quota.used_proxies + import_count > quota.max_proxies {
        return Err(format!(
            "工作空间代理配额不足，当前已使用 {}/{}，无法导入 {} 个代理",
            quota.used_proxies, quota.max_proxies, import_count
        ));
    }

    let mut success_count = 0;
    let mut failed_count = 0;
    let mut errors: Vec<String> = vec![];
    let key = get_encryption_key();

    for (index, proxy) in payload.proxies.iter().enumerate() {
        // 加密密码
        let password_encrypted = if let Some(pwd) = &proxy.password {
            match encrypt_password(pwd, &key) {
                Ok(encrypted) => Some(encrypted),
                Err(e) => {
                    failed_count += 1;
                    errors.push(format!("第 {} 项密码加密失败: {}", index + 1, e));
                    continue;
                }
            }
        } else {
            None
        };

        let result = models::insert_proxy(
            &svc_ctx.db,
            workspace_uuid,
            owner_uuid,
            &proxy.name,
            &proxy.host,
            proxy.port,
            &proxy.proxy_type,
            proxy.username.as_deref(),
            password_encrypted.as_deref(),
        )
        .await;

        match result {
            Ok(_) => {
                success_count += 1;
                // 更新配额（每成功导入一个代理就增加配额使用数）
                if let Err(e) = models::increment_used_proxies(&svc_ctx.db, workspace_uuid, 1).await
                {
                    errors.push(format!("第 {} 项导入成功但更新配额失败: {}", index + 1, e));
                }
            }
            Err(e) => {
                failed_count += 1;
                errors.push(format!("第 {} 项: {}", index + 1, e));
            }
        }
    }

    Ok(crate::entitys::BatchImportResponse {
        success_count,
        failed_count,
        errors,
    })
}
