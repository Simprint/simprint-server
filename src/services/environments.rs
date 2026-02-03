use std::collections::HashMap;
use uuid::Uuid;

use crate::dto::{
    EnvironmentConfigDto, EnvironmentCookieDto, EnvironmentDto, EnvironmentUrlDto, GroupSummaryDto,
    PlatformAccountDto, ProxySummaryDto, TagDto,
};
use crate::entitys::{
    AddEnvironmentCookieRequest, AddEnvironmentUrlRequest, AssignTagsRequest,
    BatchAssignTagRequest, BatchCreateEnvironmentRequest, BatchMoveToGroupRequest,
    BatchRemoveTagsRequest, CookieInput, CreateEnvironmentRequest, ListEnvironmentsRequest,
    MoveToGroupRequest, SetEnvironmentProxyRequest, UpdateEnvironmentRequest,
};
use crate::models;
use crate::services::accounts;
use crate::svc_ctx::SvcCtx;

// ============ Environments ============

/// 创建环境
pub async fn create_environment_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    payload: &CreateEnvironmentRequest,
) -> Result<Uuid, String> {
    // 1. 检查用户是否在当前工作空间的团队中（工作空间级别）
    let team_member = models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "您不是该团队的成员".to_string())?;

    // 2. 权限检查
    if let Some(group_uuid) = payload.group_uuid {
        // 如果指定了分组，检查用户是否有目标分组的 write 或 manage 权限
        let has_write = models::check_group_permission(
            &svc_ctx.db,
            workspace_uuid,
            group_uuid,
            user_uuid,
            "write",
        )
        .await
        .map_err(|e| e.to_string())?;

        let has_manage = models::check_group_permission(
            &svc_ctx.db,
            workspace_uuid,
            group_uuid,
            user_uuid,
            "manage",
        )
        .await
        .map_err(|e| e.to_string())?;

        if !has_write && !has_manage {
            return Err("您没有在该分组中创建环境的权限".to_string());
        }
    } else {
        // 如果未指定分组，检查用户是否有团队级别的环境创建权限（Editor/Admin/Owner）
        let can_create = matches!(team_member.role.as_str(), "owner" | "admin" | "editor");
        if !can_create {
            return Err("您没有创建环境的权限，需要 Editor 及以上角色".to_string());
        }
    }

    // 3. 检查工作空间配额是否充足
    let quota_available = models::check_quota(&svc_ctx.db, workspace_uuid, "environments")
        .await
        .map_err(|e| e.to_string())?;
    if !quota_available {
        return Err("工作空间环境配额不足，无法创建新环境".to_string());
    }

    // 提取系统信息
    let system_info = payload
        .config
        .window_info
        .get("system")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let kernel_info = payload
        .config
        .window_info
        .get("kernel")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // 创建环境
    let env_uuid = models::insert_environment(
        &svc_ctx.db,
        workspace_uuid,
        user_uuid,
        team_uuid,
        &payload.name,
        payload.description.as_deref(),
        payload.group_uuid,
        payload.proxy_uuid,
        system_info.as_deref(),
        kernel_info.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())?;

    // 创建环境配置
    models::upsert_environment_config(
        &svc_ctx.db,
        env_uuid,
        &payload.config.window_info,
        &payload.config.basic_settings,
        &payload.config.fingerprint_settings,
        &payload.config.device_settings,
        &payload.config.preference_settings,
        &payload.config.project_metadata,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 分配标签
    if let Some(tag_uuids) = &payload.tag_uuids {
        for tag_uuid in tag_uuids {
            let _ = models::insert_environment_tag(&svc_ctx.db, env_uuid, *tag_uuid).await;
        }
    }

    // 关联账号
    if let Some(account_uuids) = &payload.account_uuids {
        for (idx, account_uuid) in account_uuids.iter().enumerate() {
            let _ = models::accounts::insert_environment_account(
                &svc_ctx.db,
                env_uuid,
                *account_uuid,
                idx as i32,
            )
            .await;
        }
    }

    // 添加 Cookies
    if let Some(cookie_strings) = &payload.cookies {
        let parsed_cookies = parse_cookie_strings(cookie_strings)?;
        if !parsed_cookies.is_empty() {
            let _ =
                models::batch_insert_environment_cookies(&svc_ctx.db, env_uuid, &parsed_cookies)
                    .await
                    .map_err(|e| format!("添加 Cookies 失败: {}", e))?;
        }
    }

    // 4. 更新工作空间配额（创建后增加使用数）
    models::increment_used_environments(&svc_ctx.db, workspace_uuid, 1)
        .await
        .map_err(|e| format!("更新配额失败: {}", e))?;

    Ok(env_uuid)
}

/// 解析 Cookie 字符串数组为 CookieInput 数组
/// Cookie 字符串格式支持：
/// - 简单格式: "name=value"
/// - 完整格式: "name=value; domain=example.com; path=/; secure; httpOnly"
fn parse_cookie_strings(cookie_strings: &[String]) -> Result<Vec<CookieInput>, String> {
    let mut cookies = Vec::new();

    for cookie_str in cookie_strings {
        let cookie_str = cookie_str.trim();
        if cookie_str.is_empty() {
            continue;
        }

        // 解析 cookie 字符串
        let parts: Vec<&str> = cookie_str.split(';').map(|s| s.trim()).collect();
        if parts.is_empty() {
            continue;
        }

        // 解析 name=value
        let name_value = parts[0];
        let (name, value) = if let Some(eq_pos) = name_value.find('=') {
            (name_value[..eq_pos].trim(), name_value[eq_pos + 1..].trim())
        } else {
            return Err(format!("无效的 Cookie 格式: {}", cookie_str));
        };

        if name.is_empty() {
            return Err(format!("Cookie 名称不能为空: {}", cookie_str));
        }

        // 解析其他属性
        let mut domain = None;
        let mut path = Some("/".to_string());
        let mut http_only = Some(false);
        let mut secure = Some(false);
        let mut same_site = Some("Lax".to_string());

        for part in parts.iter().skip(1) {
            let part_lower = part.to_lowercase();
            if part_lower.starts_with("domain=") {
                domain = Some(part[7..].trim().to_string());
            } else if part_lower.starts_with("path=") {
                path = Some(part[5..].trim().to_string());
            } else if part_lower == "secure" {
                secure = Some(true);
            } else if part_lower == "httponly" || part_lower == "http-only" {
                http_only = Some(true);
            } else if part_lower.starts_with("samesite=") {
                same_site = Some(part[9..].trim().to_string());
            }
        }

        // 如果没有指定 domain，尝试从 cookie 值中推断（如果可能）
        // 这里简化处理，如果没有 domain 则使用空字符串或默认值
        let domain = domain.unwrap_or_else(|| "".to_string());

        cookies.push(CookieInput {
            domain,
            name: name.to_string(),
            value: value.to_string(),
            path,
            http_only,
            secure,
            same_site,
        });
    }

    Ok(cookies)
}

/// 获取环境列表（包含完整关联数据）
pub async fn get_environments_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    payload: &ListEnvironmentsRequest,
) -> Result<(Vec<crate::entitys::EnvironmentDetailResponse>, i64), String> {
    // 1. 检查用户是否在当前工作空间的团队中
    let team_member = models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "您不是该团队的成员".to_string())?;

    let offset = (payload.pagination.page - 1) * payload.pagination.page_size;

    let group_uuid = payload.filters.as_ref().and_then(|f| f.group_uuid);
    let status = payload.filters.as_ref().and_then(|f| f.status.as_deref());

    // 2. 查询环境基础列表
    let env_rows = models::fetch_environments_base(
        &svc_ctx.db,
        workspace_uuid,
        team_uuid,
        group_uuid,
        status,
        offset,
        payload.pagination.page_size,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 注意：这里不查询总数，因为权限过滤后的实际数量需要根据过滤结果计算
    // 如果需要精确的分页总数，需要在权限过滤后重新计算

    // 2. 权限过滤：根据分组权限过滤环境
    let is_owner_or_admin = matches!(team_member.role.as_str(), "owner" | "admin");

    // 收集所有需要检查权限的分组 UUID（去重）
    let unique_group_uuids: Vec<Uuid> = env_rows
        .iter()
        .filter_map(|row| row.group_uuid)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // 批量检查分组权限（如果不是 Owner/Admin）
    let mut group_permissions_cache: std::collections::HashMap<Uuid, bool> =
        std::collections::HashMap::new();
    if !is_owner_or_admin && !unique_group_uuids.is_empty() {
        for group_uuid in unique_group_uuids {
            let has_permission = models::check_group_permission(
                &svc_ctx.db,
                workspace_uuid,
                group_uuid,
                user_uuid,
                "read",
            )
            .await
            .map_err(|e| e.to_string())?;
            group_permissions_cache.insert(group_uuid, has_permission);
        }
    }

    // 8. 过滤无权限的环境
    let filtered_env_rows: Vec<_> = env_rows
        .into_iter()
        .filter(|row| {
            if let Some(group_uuid) = row.group_uuid {
                // 如果环境有分组，检查用户是否有分组的 read 权限
                if is_owner_or_admin {
                    true // Owner/Admin 自动拥有所有分组权限
                } else {
                    *group_permissions_cache.get(&group_uuid).unwrap_or(&false)
                }
            } else {
                // 如果环境无分组，所有团队成员都可以查看（已在团队成员检查中验证）
                true
            }
        })
        .collect();

    // 9. 重新收集关联 UUID（基于过滤后的环境）
    let env_uuids: Vec<Uuid> = filtered_env_rows.iter().map(|e| e.uuid).collect();
    let group_uuids: Vec<Uuid> = filtered_env_rows.iter().filter_map(|e| e.group_uuid).collect();
    let proxy_uuids: Vec<Uuid> = filtered_env_rows.iter().filter_map(|e| e.proxy_uuid).collect();

    // 10. 重新查询关联数据（基于过滤后的环境）
    // 批量查询环境配置
    let mut configs_map: HashMap<Uuid, EnvironmentConfigDto> = HashMap::new();
    if !env_uuids.is_empty() {
        let config_rows = models::fetch_environment_configs_by_uuids(&svc_ctx.db, &env_uuids)
            .await
            .map_err(|e| e.to_string())?;
        for config in config_rows {
            configs_map.insert(config.environment_uuid, config);
        }
    }

    // 批量查询分组
    let group_rows = if !group_uuids.is_empty() {
        models::fetch_groups_by_uuids(&svc_ctx.db, &group_uuids)
            .await
            .map_err(|e| e.to_string())?
    } else {
        vec![]
    };
    let groups_map: HashMap<Uuid, _> = group_rows.into_iter().map(|g| (g.uuid, g)).collect();

    // 批量查询代理
    let proxy_rows = if !proxy_uuids.is_empty() {
        models::fetch_proxies_by_uuids(&svc_ctx.db, &proxy_uuids)
            .await
            .map_err(|e| e.to_string())?
    } else {
        vec![]
    };
    let proxies_map: HashMap<Uuid, _> = proxy_rows.into_iter().map(|p| (p.uuid, p)).collect();

    // 批量查询标签
    let tag_rows = if !env_uuids.is_empty() {
        models::fetch_tags_for_environments(&svc_ctx.db, &env_uuids)
            .await
            .map_err(|e| e.to_string())?
    } else {
        vec![]
    };

    // 按环境分组标签
    let mut tags_map: HashMap<Uuid, Vec<TagDto>> = HashMap::new();
    for tag_row in tag_rows {
        tags_map.entry(tag_row.environment_uuid).or_default().push(TagDto {
            id: tag_row.tag_id,
            uuid: tag_row.tag_uuid,
            user_uuid: tag_row.tag_user_uuid,
            team_uuid: tag_row.tag_team_uuid,
            name: tag_row.tag_name,
            color: tag_row.tag_color,
            sort_order: tag_row.tag_sort_order,
            environments_count: tag_row.tag_environments_count,
            created_at: tag_row.tag_created_at,
            updated_at: tag_row.tag_updated_at,
            deleted_at: tag_row.tag_deleted_at,
        });
    }

    // 批量查询账号
    let mut accounts_map: HashMap<Uuid, Vec<PlatformAccountDto>> = HashMap::new();
    for env_uuid in &env_uuids {
        let accounts = accounts::get_environment_accounts_service(&svc_ctx, *env_uuid)
            .await
            .unwrap_or_default();
        accounts_map.insert(*env_uuid, accounts);
    }

    // 11. 组装完整数据（使用与环境详情一致的数据结构）
    let environments: Vec<crate::entitys::EnvironmentDetailResponse> = filtered_env_rows
        .into_iter()
        .map(|row| {
            // 构建 EnvironmentDto
            let environment = EnvironmentDto {
                id: row.id,
                uuid: row.uuid,
                workspace_uuid: row.workspace_uuid,
                user_uuid: row.user_uuid,
                team_uuid: row.team_uuid,
                name: row.name,
                description: row.description,
                status: row.status,
                group_uuid: row.group_uuid,
                proxy_uuid: row.proxy_uuid,
                system_info: row.system_info,
                kernel_info: row.kernel_info,
                fingerprint_summary: row.fingerprint_summary,
                last_opened_at: row.last_opened_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
                deleted_at: None,
            };

            // 分组详情
            let group = row.group_uuid.and_then(|uuid| {
                groups_map.get(&uuid).map(|g| GroupSummaryDto {
                    id: g.id,
                    uuid: g.uuid,
                    name: g.name.clone(),
                    description: g.description.clone(),
                    sort_order: g.sort_order,
                })
            });

            // 代理详情
            let proxy = row.proxy_uuid.and_then(|uuid| {
                proxies_map.get(&uuid).map(|p| ProxySummaryDto {
                    id: p.id,
                    uuid: p.uuid,
                    name: p.name.clone(),
                    host: p.host.clone(),
                    port: p.port,
                    proxy_type: p.proxy_type.clone(),
                    country: p.country.clone(),
                    city: p.city.clone(),
                    status: p.status.clone(),
                    latency: p.latency,
                    last_check_ip: p.last_check_ip.clone(),
                })
            });

            crate::entitys::EnvironmentDetailResponse {
                environment,
                config: configs_map.remove(&row.uuid), // 返回配置信息，用于传递给指纹浏览器内核
                tags: tags_map.remove(&row.uuid).unwrap_or_default(),
                accounts: accounts_map.remove(&row.uuid).unwrap_or_default(),
                group,
                proxy,
            }
        })
        .collect();

    // 返回过滤后的环境列表和实际数量（用于分页显示）
    // 注意：total 是查询时的总数，实际返回的数量是过滤后的
    let filtered_total = environments.len() as i64;
    Ok((environments, filtered_total))
}

/// 获取环境详情
pub async fn get_environment_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    user_uuid: Uuid,
    env_uuid: Uuid,
) -> Result<EnvironmentDto, String> {
    // 1. 检查用户是否在当前工作空间的团队中
    let _team_member = models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "您不是该团队的成员".to_string())?;

    // 2. 查询环境（带工作空间过滤）
    let environment = models::fetch_environment_by_uuid(&svc_ctx.db, workspace_uuid, env_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "环境不存在或不属于当前工作空间".to_string())?;

    // 3. 验证环境属于指定团队
    if environment.team_uuid != team_uuid {
        return Err("环境不属于指定团队".to_string());
    }

    // 4. 权限检查
    if let Some(group_uuid) = environment.group_uuid {
        // 如果环境有分组，检查用户是否有分组的 read/write/manage 权限
        // Owner/Admin 自动拥有所有分组权限（已在 check_group_permission 中处理）
        let has_permission = models::check_group_permission(
            &svc_ctx.db,
            workspace_uuid,
            group_uuid,
            user_uuid,
            "read",
        )
        .await
        .map_err(|e| e.to_string())?;

        if !has_permission {
            return Err("您没有查看该环境的权限".to_string());
        }
    }
    // 如果环境无分组，所有团队成员都可以查看（已在团队成员检查中验证）

    Ok(environment)
}

/// 获取环境配置
pub async fn get_environment_config_service(
    svc_ctx: &SvcCtx,
    env_uuid: Uuid,
) -> Result<EnvironmentConfigDto, String> {
    models::fetch_environment_config(&svc_ctx.db, env_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "环境配置不存在".to_string())
}

/// 获取环境的标签
pub async fn get_environment_tags_service(
    svc_ctx: &SvcCtx,
    env_uuid: Uuid,
) -> Result<Vec<TagDto>, String> {
    models::fetch_environment_tags(&svc_ctx.db, env_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 获取环境详情（包含完整关联数据）
pub async fn get_environment_detail_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    user_uuid: Uuid,
    env_uuid: Uuid,
) -> Result<crate::entitys::EnvironmentDetailResponse, String> {
    let environment =
        get_environment_service(svc_ctx, workspace_uuid, team_uuid, user_uuid, env_uuid).await?;

    let config = get_environment_config_service(svc_ctx, env_uuid).await.ok();

    let tags = get_environment_tags_service(svc_ctx, env_uuid).await?;

    let accounts = accounts::get_environment_accounts_service(svc_ctx, env_uuid)
        .await
        .unwrap_or_default();

    // 获取分组信息
    let group = if let Some(group_uuid) = environment.group_uuid {
        get_group_summary_service(svc_ctx, group_uuid).await.ok()
    } else {
        None
    };

    // 获取代理信息
    let proxy = if let Some(proxy_uuid) = environment.proxy_uuid {
        get_proxy_summary_service(svc_ctx, proxy_uuid).await.ok()
    } else {
        None
    };

    Ok(crate::entitys::EnvironmentDetailResponse {
        environment,
        config,
        tags,
        accounts,
        group,
        proxy,
    })
}

/// 获取分组摘要信息
pub async fn get_group_summary_service(
    svc_ctx: &SvcCtx,
    group_uuid: Uuid,
) -> Result<crate::dto::GroupSummaryDto, String> {
    let group = models::fetch_group_by_uuid(&svc_ctx.db, group_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "分组不存在".to_string())?;

    Ok(crate::dto::GroupSummaryDto {
        id: group.id,
        uuid: group.uuid,
        name: group.name,
        description: group.description,
        sort_order: group.sort_order,
    })
}

/// 获取代理摘要信息
pub async fn get_proxy_summary_service(
    svc_ctx: &SvcCtx,
    proxy_uuid: Uuid,
) -> Result<crate::dto::ProxySummaryDto, String> {
    let proxy = crate::models::proxies::fetch_proxy_by_uuid(&svc_ctx.db, proxy_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "代理不存在".to_string())?;

    Ok(crate::dto::ProxySummaryDto {
        id: proxy.id,
        uuid: proxy.uuid,
        name: proxy.name,
        host: proxy.host,
        port: proxy.port,
        proxy_type: proxy.proxy_type,
        country: proxy.country,
        city: proxy.city,
        status: proxy.status,
        latency: proxy.latency,
        last_check_ip: proxy.last_check_ip,
    })
}

/// 更新环境
pub async fn update_environment_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    user_uuid: Uuid,
    payload: &UpdateEnvironmentRequest,
) -> Result<(), String> {
    // 1. 检查用户是否在当前工作空间的团队中
    let team_member = models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "您不是该团队的成员".to_string())?;

    // 2. 查询环境（带工作空间过滤）
    let environment = models::fetch_environment_by_uuid(&svc_ctx.db, workspace_uuid, payload.uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "环境不存在或不属于当前工作空间".to_string())?;

    // 3. 验证环境属于指定团队
    if environment.team_uuid != team_uuid {
        return Err("环境不属于指定团队".to_string());
    }

    // 4. 权限检查
    if let Some(group_uuid) = environment.group_uuid {
        // 如果环境有分组，检查用户是否有分组的 write 或 manage 权限
        let has_write = models::check_group_permission(
            &svc_ctx.db,
            workspace_uuid,
            group_uuid,
            user_uuid,
            "write",
        )
        .await
        .map_err(|e| e.to_string())?;

        let has_manage = models::check_group_permission(
            &svc_ctx.db,
            workspace_uuid,
            group_uuid,
            user_uuid,
            "manage",
        )
        .await
        .map_err(|e| e.to_string())?;

        if !has_write && !has_manage {
            return Err("您没有编辑该环境的权限".to_string());
        }
    } else {
        // 如果环境无分组，检查用户是否有团队级别的编辑权限（Editor/Admin/Owner）
        let can_edit = matches!(team_member.role.as_str(), "owner" | "admin" | "editor");
        if !can_edit {
            return Err("您没有编辑环境的权限，需要 Editor 及以上角色".to_string());
        }
    }

    models::update_environment(
        &svc_ctx.db,
        payload.uuid,
        payload.name.as_deref(),
        payload.description.as_deref(),
        payload.group_uuid,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 更新配置
    if let Some(config) = &payload.config {
        models::upsert_environment_config(
            &svc_ctx.db,
            payload.uuid,
            &config.window_info,
            &config.basic_settings,
            &config.fingerprint_settings,
            &config.device_settings,
            &config.preference_settings,
            &config.project_metadata,
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// 设置环境代理
pub async fn set_environment_proxy_service(
    svc_ctx: &SvcCtx,
    payload: &SetEnvironmentProxyRequest,
) -> Result<(), String> {
    models::update_environment_proxy(&svc_ctx.db, payload.uuid, payload.proxy_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 分配标签
pub async fn assign_tags_service(
    svc_ctx: &SvcCtx,
    payload: &AssignTagsRequest,
) -> Result<(), String> {
    for tag_uuid in &payload.tag_uuids {
        models::insert_environment_tag(&svc_ctx.db, payload.uuid, *tag_uuid)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 移除标签
pub async fn remove_tag_service(
    svc_ctx: &SvcCtx,
    env_uuid: Uuid,
    tag_uuid: Uuid,
) -> Result<(), String> {
    models::remove_environment_tag(&svc_ctx.db, env_uuid, tag_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 移动到分组
pub async fn move_to_group_service(
    svc_ctx: &SvcCtx,
    payload: &MoveToGroupRequest,
) -> Result<(), String> {
    models::update_environment(&svc_ctx.db, payload.uuid, None, None, payload.group_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 批量移动到分组
pub async fn batch_move_to_group_service(
    svc_ctx: &SvcCtx,
    payload: &BatchMoveToGroupRequest,
) -> Result<(), String> {
    for env_uuid in &payload.env_uuids {
        models::update_environment(&svc_ctx.db, *env_uuid, None, None, Some(payload.group_uuid))
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 批量分配标签
pub async fn batch_assign_tags_service(
    svc_ctx: &SvcCtx,
    payload: &BatchAssignTagRequest,
) -> Result<(), String> {
    for env_uuid in &payload.env_uuids {
        models::insert_environment_tag(&svc_ctx.db, *env_uuid, payload.tag_uuid)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 批量移除标签
pub async fn batch_remove_tags_service(
    svc_ctx: &SvcCtx,
    payload: &BatchRemoveTagsRequest,
) -> Result<(), String> {
    if let Some(tag_uuid) = payload.tag_uuid {
        // 移除指定的标签
        for env_uuid in &payload.env_uuids {
            models::remove_environment_tag(&svc_ctx.db, *env_uuid, tag_uuid)
                .await
                .map_err(|e| e.to_string())?;
        }
    } else {
        // 移除所有标签
        for env_uuid in &payload.env_uuids {
            models::clear_environment_tags(&svc_ctx.db, *env_uuid)
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// 删除环境
pub async fn delete_environment_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    user_uuid: Uuid,
    env_uuid: Uuid,
) -> Result<(), String> {
    // 1. 检查用户是否在当前工作空间的团队中
    let team_member = models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "您不是该团队的成员".to_string())?;

    // 2. 查询环境（带工作空间过滤）
    let environment = models::fetch_environment_by_uuid(&svc_ctx.db, workspace_uuid, env_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "环境不存在或不属于当前工作空间".to_string())?;

    // 3. 验证环境属于指定团队
    if environment.team_uuid != team_uuid {
        return Err("环境不属于指定团队".to_string());
    }

    // 4. 权限检查
    if let Some(group_uuid) = environment.group_uuid {
        // 如果环境有分组，检查用户是否有分组的 manage 权限
        let has_manage = models::check_group_permission(
            &svc_ctx.db,
            workspace_uuid,
            group_uuid,
            user_uuid,
            "manage",
        )
        .await
        .map_err(|e| e.to_string())?;

        if !has_manage {
            return Err("您没有删除该环境的权限".to_string());
        }
    } else {
        // 如果环境无分组，检查用户是否有团队级别的删除权限（Owner/Admin）
        let can_delete = matches!(team_member.role.as_str(), "owner" | "admin");
        if !can_delete {
            return Err("您没有删除环境的权限，需要 Owner 或 Admin 角色".to_string());
        }
    }

    // 5. 删除环境
    models::delete_environment(&svc_ctx.db, env_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 6. 更新工作空间配额（删除后减少使用数）
    models::decrement_used_environments(&svc_ctx.db, workspace_uuid, 1)
        .await
        .map_err(|e| format!("更新配额失败: {}", e))?;

    Ok(())
}

/// 批量删除环境
pub async fn batch_delete_environments_service(
    svc_ctx: &SvcCtx,
    env_uuids: &[Uuid],
) -> Result<u64, String> {
    models::batch_delete_environments(&svc_ctx.db, env_uuids)
        .await
        .map_err(|e| e.to_string())
}

// ============ Environment URLs ============

/// 添加环境 URL
pub async fn add_environment_url_service(
    svc_ctx: &SvcCtx,
    payload: &AddEnvironmentUrlRequest,
) -> Result<i32, String> {
    models::insert_environment_url(
        &svc_ctx.db,
        payload.environment_uuid,
        &payload.url,
        payload.title.as_deref(),
        payload.sort_order,
    )
    .await
    .map_err(|e| e.to_string())
}

/// 获取环境的所有 URL
pub async fn get_environment_urls_service(
    svc_ctx: &SvcCtx,
    env_uuid: Uuid,
) -> Result<Vec<EnvironmentUrlDto>, String> {
    models::fetch_environment_urls(&svc_ctx.db, env_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 删除环境 URL
pub async fn delete_environment_url_service(svc_ctx: &SvcCtx, url_id: i32) -> Result<(), String> {
    models::delete_environment_url(&svc_ctx.db, url_id)
        .await
        .map_err(|e| e.to_string())
}

/// 清空环境的所有 URL
pub async fn clear_environment_urls_service(
    svc_ctx: &SvcCtx,
    env_uuid: Uuid,
) -> Result<u64, String> {
    models::clear_environment_urls(&svc_ctx.db, env_uuid)
        .await
        .map_err(|e| e.to_string())
}

// ============ Environment Cookies ============

/// 添加环境 Cookie
pub async fn add_environment_cookie_service(
    svc_ctx: &SvcCtx,
    payload: &AddEnvironmentCookieRequest,
) -> Result<i32, String> {
    models::insert_environment_cookie(
        &svc_ctx.db,
        payload.environment_uuid,
        &payload.domain,
        &payload.name,
        &payload.value,
        payload.path.as_deref(),
        payload.http_only,
        payload.secure,
        payload.same_site.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// 获取环境的所有 Cookies
pub async fn get_environment_cookies_service(
    svc_ctx: &SvcCtx,
    env_uuid: Uuid,
) -> Result<Vec<EnvironmentCookieDto>, String> {
    models::fetch_environment_cookies(&svc_ctx.db, env_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 删除环境 Cookie
pub async fn delete_environment_cookie_service(
    svc_ctx: &SvcCtx,
    cookie_id: i32,
) -> Result<(), String> {
    models::delete_environment_cookie(&svc_ctx.db, cookie_id)
        .await
        .map_err(|e| e.to_string())
}

/// 清空环境的所有 Cookies
pub async fn clear_environment_cookies_service(
    svc_ctx: &SvcCtx,
    env_uuid: Uuid,
) -> Result<u64, String> {
    models::clear_environment_cookies(&svc_ctx.db, env_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 批量创建环境
pub async fn batch_create_environments_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    payload: &BatchCreateEnvironmentRequest,
) -> Result<Vec<Uuid>, String> {
    let mut created_uuids = Vec::new();

    for env_request in &payload.environments {
        let env_uuid =
            create_environment_service(svc_ctx, user_uuid, workspace_uuid, team_uuid, env_request)
                .await
                .map_err(|e| format!("创建环境 '{}' 失败: {}", env_request.name, e))?;

        created_uuids.push(env_uuid);
    }

    Ok(created_uuids)
}
