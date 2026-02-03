use uuid::Uuid;

use crate::dto::ExtensionDto;
use crate::entitys::{
    ExtensionGroup, InstallExtensionRequest, InstalledExtensionItem, ListExtensionsRequest,
    UninstallExtensionRequest,
};
use crate::models;
use crate::models::environments as env_models;
use crate::svc_ctx::SvcCtx;
use crate::utils::minios::get_objects;

/// 获取扩展列表
pub async fn get_extensions_service(
    svc_ctx: &SvcCtx,
    payload: &ListExtensionsRequest,
) -> Result<(Vec<ExtensionDto>, i64), String> {
    let offset = (payload.pagination.page - 1) * payload.pagination.page_size;

    let keyword = payload.filters.as_ref().and_then(|f| f.keyword.as_deref());
    let category = payload.filters.as_ref().and_then(|f| f.category.as_deref());

    let mut extensions = models::extensions::fetch_extensions(
        &svc_ctx.db,
        keyword,
        category,
        offset,
        payload.pagination.page_size,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 将 object path 转换为完整 URL
    let resource_url = svc_ctx.config.minio.as_ref().map(|c| c.resource_url.clone());
    let extension_bucket = svc_ctx.config.minio.as_ref().map(|c| c.extension_bucket.clone());
    ExtensionDto::transform_urls_batch(&mut extensions, &resource_url, &extension_bucket);

    let total = models::extensions::fetch_extensions_count(&svc_ctx.db, keyword, category)
        .await
        .map_err(|e| e.to_string())?;

    Ok((extensions, total))
}

/// 获取扩展详情
pub async fn get_extension_service(
    svc_ctx: &SvcCtx,
    extension_id: &str,
) -> Result<ExtensionDto, String> {
    let mut extension = models::extensions::fetch_extension_by_id(&svc_ctx.db, extension_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "扩展不存在".to_string())?;

    // 将 object path 转换为完整 URL
    let resource_url = svc_ctx.config.minio.as_ref().map(|c| c.resource_url.clone());
    let extension_bucket = svc_ctx.config.minio.as_ref().map(|c| c.extension_bucket.clone());
    extension.transform_urls(&resource_url, &extension_bucket);

    Ok(extension)
}

/// 获取扩展分类
pub async fn get_extension_categories_service(svc_ctx: &SvcCtx) -> Result<Vec<String>, String> {
    models::extensions::fetch_extension_categories(&svc_ctx.db)
        .await
        .map_err(|e| e.to_string())
}

/// 获取用户已安装的扩展
pub async fn get_user_installed_extensions_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
) -> Result<Vec<InstalledExtensionItem>, String> {
    let resource_url = svc_ctx.config.minio.as_ref().map(|c| c.resource_url.clone());
    let extension_bucket = svc_ctx.config.minio.as_ref().map(|c| c.extension_bucket.clone());

    // 查询用户直接安装的扩展
    let user_installed = models::extensions::fetch_user_extensions(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 查询用户相关的分组中安装的扩展
    let group_installed =
        models::extensions::fetch_user_group_extensions(&svc_ctx.db, user_uuid, team_uuid)
            .await
            .map_err(|e| e.to_string())?;

    // 使用 HashMap 去重，以 extension_id 为 key
    use std::collections::HashMap;
    let mut extension_map: HashMap<String, InstalledExtensionItem> = HashMap::new();

    // 处理用户直接安装的扩展
    for ue in user_installed {
        if let Ok(Some(ext)) =
            models::extensions::fetch_extension_by_id(&svc_ctx.db, &ue.extension_id).await
        {
            // 转换图标 URL
            let mut icon_url = ext.icon_url.clone();
            if let Some(path) = &icon_url {
                if !path.is_empty() && !path.starts_with("http") {
                    icon_url =
                        get_objects::get_extension_icon_url(&resource_url, &extension_bucket, path);
                }
            }

            // 查询关联的分组
            let group_uuids = models::extensions::fetch_group_uuids_by_extension_id(
                &svc_ctx.db,
                &ue.extension_id,
            )
            .await
            .unwrap_or_default();

            let mut groups = Vec::new();
            for group_uuid in group_uuids {
                if let Ok(Some(group)) =
                    env_models::fetch_group_by_uuid(&svc_ctx.db, group_uuid).await
                {
                    groups.push(ExtensionGroup {
                        uuid: group.uuid,
                        name: group.name,
                    });
                }
            }

            let has_update = ext.version != ue.installed_version;
            extension_map.insert(
                ue.extension_id.clone(),
                InstalledExtensionItem {
                    extension_id: ue.extension_id,
                    name: ext.name,
                    version: ext.version,
                    installed_version: ue.installed_version,
                    has_update,
                    status: ue.status,
                    installed_at: ue.installed_at,
                    homepage: ext.homepage,
                    icon_url,
                    team_uuid: None,
                    scope: "user".to_string(),
                    description: ext.description,
                    category: Some(ext.category),
                    browser: Some(ext.browser),
                    developer: ext.developer,
                    downloads_count: ext.downloads_count,
                    rating: ext.rating,
                    permissions: ext.permissions,
                    file_size: ext.file_size,
                    updated_at: Some(ext.updated_at),
                    groups: if groups.is_empty() {
                        None
                    } else {
                        Some(groups)
                    },
                },
            );
        }
    }

    // 处理分组安装的扩展
    for ge in group_installed {
        // 如果已存在（用户直接安装），分组信息已经包含在内，直接跳过
        if extension_map.contains_key(&ge.extension_id) {
            continue;
        }

        // 只处理新扩展（仅安装在分组中，用户未直接安装）
        if let Ok(Some(ext)) =
            models::extensions::fetch_extension_by_id(&svc_ctx.db, &ge.extension_id).await
        {
            // 转换图标 URL
            let mut icon_url = ext.icon_url.clone();
            if let Some(path) = &icon_url {
                if !path.is_empty() && !path.starts_with("http") {
                    icon_url =
                        get_objects::get_extension_icon_url(&resource_url, &extension_bucket, path);
                }
            }

            // 查询关联的分组
            let group_uuids = models::extensions::fetch_group_uuids_by_extension_id(
                &svc_ctx.db,
                &ge.extension_id,
            )
            .await
            .unwrap_or_default();

            let mut groups = Vec::new();
            for group_uuid in group_uuids {
                if let Ok(Some(group)) =
                    env_models::fetch_group_by_uuid(&svc_ctx.db, group_uuid).await
                {
                    groups.push(ExtensionGroup {
                        uuid: group.uuid,
                        name: group.name,
                    });
                }
            }

            let has_update = ext.version != ge.installed_version;
            extension_map.insert(
                ge.extension_id.clone(),
                InstalledExtensionItem {
                    extension_id: ge.extension_id,
                    name: ext.name,
                    version: ext.version,
                    installed_version: ge.installed_version,
                    has_update,
                    status: ge.status,
                    installed_at: ge.installed_at,
                    homepage: ext.homepage,
                    icon_url,
                    team_uuid: None,
                    scope: "group".to_string(),
                    description: ext.description,
                    category: Some(ext.category),
                    browser: Some(ext.browser),
                    developer: ext.developer,
                    downloads_count: ext.downloads_count,
                    rating: ext.rating,
                    permissions: ext.permissions,
                    file_size: ext.file_size,
                    updated_at: Some(ext.updated_at),
                    groups: if groups.is_empty() {
                        None
                    } else {
                        Some(groups)
                    },
                },
            );
        }
    }

    Ok(extension_map.into_values().collect())
}

/// 获取团队已安装的扩展
pub async fn get_team_installed_extensions_service(
    svc_ctx: &SvcCtx,
    team_uuid: Uuid,
) -> Result<Vec<InstalledExtensionItem>, String> {
    let resource_url = svc_ctx.config.minio.as_ref().map(|c| c.resource_url.clone());
    let extension_bucket = svc_ctx.config.minio.as_ref().map(|c| c.extension_bucket.clone());

    // 查询团队直接安装的扩展
    let team_installed = models::extensions::fetch_team_extensions(&svc_ctx.db, team_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 查询团队相关的分组中安装的扩展
    let group_installed = models::extensions::fetch_team_group_extensions(&svc_ctx.db, team_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 使用 HashMap 去重，以 extension_id 为 key
    use std::collections::HashMap;
    let mut extension_map: HashMap<String, InstalledExtensionItem> = HashMap::new();

    // 处理团队直接安装的扩展
    for te in team_installed {
        if let Ok(Some(ext)) =
            models::extensions::fetch_extension_by_id(&svc_ctx.db, &te.extension_id).await
        {
            // 转换图标 URL
            let mut icon_url = ext.icon_url.clone();
            if let Some(path) = &icon_url {
                if !path.is_empty() && !path.starts_with("http") {
                    icon_url =
                        get_objects::get_extension_icon_url(&resource_url, &extension_bucket, path);
                }
            }

            // 查询关联的分组
            let group_uuids = models::extensions::fetch_group_uuids_by_extension_id(
                &svc_ctx.db,
                &te.extension_id,
            )
            .await
            .unwrap_or_default();

            let mut groups = Vec::new();
            for group_uuid in group_uuids {
                if let Ok(Some(group)) =
                    env_models::fetch_group_by_uuid(&svc_ctx.db, group_uuid).await
                {
                    groups.push(ExtensionGroup {
                        uuid: group.uuid,
                        name: group.name,
                    });
                }
            }

            let has_update = ext.version != te.installed_version;
            extension_map.insert(
                te.extension_id.clone(),
                InstalledExtensionItem {
                    extension_id: te.extension_id,
                    name: ext.name,
                    version: ext.version,
                    installed_version: te.installed_version,
                    has_update,
                    status: te.status,
                    installed_at: te.installed_at,
                    homepage: ext.homepage,
                    icon_url,
                    team_uuid: Some(team_uuid),
                    scope: "team".to_string(),
                    description: ext.description,
                    category: Some(ext.category),
                    browser: Some(ext.browser),
                    developer: ext.developer,
                    downloads_count: ext.downloads_count,
                    rating: ext.rating,
                    permissions: ext.permissions,
                    file_size: ext.file_size,
                    updated_at: Some(ext.updated_at),
                    groups: if groups.is_empty() {
                        None
                    } else {
                        Some(groups)
                    },
                },
            );
        }
    }

    // 处理分组安装的扩展
    for ge in group_installed {
        // 如果已存在（团队直接安装），分组信息已经包含在内，直接跳过
        if extension_map.contains_key(&ge.extension_id) {
            continue;
        }

        // 只处理新扩展（仅安装在分组中，团队未直接安装）
        if let Ok(Some(ext)) =
            models::extensions::fetch_extension_by_id(&svc_ctx.db, &ge.extension_id).await
        {
            // 转换图标 URL
            let mut icon_url = ext.icon_url.clone();
            if let Some(path) = &icon_url {
                if !path.is_empty() && !path.starts_with("http") {
                    icon_url =
                        get_objects::get_extension_icon_url(&resource_url, &extension_bucket, path);
                }
            }

            // 查询关联的分组
            let group_uuids = models::extensions::fetch_group_uuids_by_extension_id(
                &svc_ctx.db,
                &ge.extension_id,
            )
            .await
            .unwrap_or_default();

            let mut groups = Vec::new();
            for group_uuid in group_uuids {
                if let Ok(Some(group)) =
                    env_models::fetch_group_by_uuid(&svc_ctx.db, group_uuid).await
                {
                    groups.push(ExtensionGroup {
                        uuid: group.uuid,
                        name: group.name,
                    });
                }
            }

            let has_update = ext.version != ge.installed_version;
            extension_map.insert(
                ge.extension_id.clone(),
                InstalledExtensionItem {
                    extension_id: ge.extension_id,
                    name: ext.name,
                    version: ext.version,
                    installed_version: ge.installed_version,
                    has_update,
                    status: ge.status,
                    installed_at: ge.installed_at,
                    homepage: ext.homepage,
                    icon_url,
                    team_uuid: Some(team_uuid),
                    scope: "group".to_string(),
                    description: ext.description,
                    category: Some(ext.category),
                    browser: Some(ext.browser),
                    developer: ext.developer,
                    downloads_count: ext.downloads_count,
                    rating: ext.rating,
                    permissions: ext.permissions,
                    file_size: ext.file_size,
                    updated_at: Some(ext.updated_at),
                    groups: if groups.is_empty() {
                        None
                    } else {
                        Some(groups)
                    },
                },
            );
        }
    }

    Ok(extension_map.into_values().collect())
}

/// 安装扩展
pub async fn install_extension_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    payload: &InstallExtensionRequest,
) -> Result<(), String> {
    // 获取扩展信息
    let extension = models::extensions::fetch_extension_by_id(&svc_ctx.db, &payload.extension_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "扩展不存在".to_string())?;

    let target_type = payload.target_type.as_deref().unwrap_or("user");

    match target_type {
        "user" => {
            models::extensions::insert_user_extension(
                &svc_ctx.db,
                user_uuid,
                &payload.extension_id,
                &extension.version,
            )
            .await
            .map_err(|e| e.to_string())?;
        }
        "team" => {
            let team_uuid = team_uuid.ok_or_else(|| "未指定团队".to_string())?;
            models::extensions::insert_team_extension(
                &svc_ctx.db,
                team_uuid,
                &payload.extension_id,
                &extension.version,
                user_uuid,
            )
            .await
            .map_err(|e| e.to_string())?;
        }
        "group" => {
            // 必须提供分组数组，即使只有一个分组也需要传入数组
            let group_ids = payload.group_ids.as_ref().ok_or_else(|| "未指定分组".to_string())?;
            if group_ids.is_empty() {
                return Err("分组列表不能为空".to_string());
            }
            for group_uuid in group_ids {
                models::extensions::insert_group_extension(
                    &svc_ctx.db,
                    *group_uuid,
                    &payload.extension_id,
                    &extension.version,
                    user_uuid,
                )
                .await
                .map_err(|e| e.to_string())?;
            }
        }
        "environment" => {
            let env_uuid = payload.env_uuid.ok_or_else(|| "未指定环境".to_string())?;
            models::extensions::insert_environment_extension(
                &svc_ctx.db,
                env_uuid,
                &payload.extension_id,
                &extension.version,
            )
            .await
            .map_err(|e| e.to_string())?;
        }
        _ => return Err("无效的安装目标类型".to_string()),
    }

    Ok(())
}

/// 卸载扩展
pub async fn uninstall_extension_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    payload: &UninstallExtensionRequest,
) -> Result<(), String> {
    // 无论 target_type 是什么，都先删除所有相关的分组记录
    models::extensions::delete_group_extensions_by_extension_id(&svc_ctx.db, &payload.extension_id)
        .await
        .map_err(|e| e.to_string())?;

    // 然后根据 target_type 删除对应的记录
    let target_type = payload.target_type.as_deref().unwrap_or("user");

    match target_type {
        "user" => {
            models::extensions::delete_user_extension(
                &svc_ctx.db,
                user_uuid,
                &payload.extension_id,
            )
            .await
            .map_err(|e| e.to_string())?;
        }
        "team" => {
            let team_uuid = team_uuid.ok_or_else(|| "未指定团队".to_string())?;
            models::extensions::delete_team_extension(
                &svc_ctx.db,
                team_uuid,
                &payload.extension_id,
            )
            .await
            .map_err(|e| e.to_string())?;
        }
        "environment" => {
            let env_uuid = payload.target_uuid.ok_or_else(|| "未指定环境".to_string())?;
            models::extensions::delete_environment_extension(
                &svc_ctx.db,
                env_uuid,
                &payload.extension_id,
            )
            .await
            .map_err(|e| e.to_string())?;
        }
        _ => return Err("无效的卸载目标类型".to_string()),
    }

    Ok(())
}

/// 更新扩展
pub async fn update_extension_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    extension_id: &str,
) -> Result<(), String> {
    // 获取最新扩展信息
    let extension = models::extensions::fetch_extension_by_id(&svc_ctx.db, extension_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "扩展不存在".to_string())?;

    // 更新用户扩展版本
    models::extensions::insert_user_extension(
        &svc_ctx.db,
        user_uuid,
        extension_id,
        &extension.version,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 批量更新扩展
pub async fn batch_update_extensions_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    extension_ids: &[String],
) -> Result<u64, String> {
    let mut updated = 0u64;

    for extension_id in extension_ids {
        if update_extension_service(svc_ctx, user_uuid, extension_id).await.is_ok() {
            updated += 1;
        }
    }

    Ok(updated)
}
