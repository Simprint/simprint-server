//! Console Gateway 管理员和权限数据库操作

use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::dto::{ConsoleAdminDto, ConsoleAdminPermissionDto, ConsolePermissionDto};
use crate::entitys::{
    CreateConsoleAdminRequest, CreateConsolePermissionRequest, GrantConsolePermissionRequest,
    UpdateConsoleAdminRequest,
};

// ==================== 管理员操作 ====================

/// 创建管理员
pub async fn create_admin(
    pool: &Pool<Postgres>,
    request: CreateConsoleAdminRequest,
) -> Result<ConsoleAdminDto, Error> {
    let admin = sqlx::query_as::<_, ConsoleAdminDto>(
        r#"
        INSERT INTO console_admins (user_uuid, is_active)
        VALUES ($1, TRUE)
        RETURNING id, uuid, user_uuid, is_active, created_at, updated_at, deleted_at
        "#,
    )
    .bind(request.user_uuid)
    .fetch_one(pool)
    .await?;

    Ok(admin)
}

/// 根据用户 UUID 获取管理员
pub async fn get_admin_by_user_uuid(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<Option<ConsoleAdminDto>, Error> {
    let admin = sqlx::query_as::<_, ConsoleAdminDto>(
        r#"
        SELECT id, uuid, user_uuid, is_active, created_at, updated_at, deleted_at
        FROM console_admins
        WHERE user_uuid = $1 AND is_active = TRUE AND deleted_at IS NULL
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(admin)
}

/// 根据 ID 获取管理员
pub async fn get_admin_by_id(
    pool: &Pool<Postgres>,
    admin_id: i32,
) -> Result<Option<ConsoleAdminDto>, Error> {
    let admin = sqlx::query_as::<_, ConsoleAdminDto>(
        r#"
        SELECT id, uuid, user_uuid, is_active, created_at, updated_at, deleted_at
        FROM console_admins
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(admin_id)
    .fetch_optional(pool)
    .await?;

    Ok(admin)
}

/// 列出所有管理员
pub async fn list_admins(
    pool: &Pool<Postgres>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ConsoleAdminDto>, Error> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let admins = sqlx::query_as::<_, ConsoleAdminDto>(
        r#"
        SELECT id, uuid, user_uuid, is_active, created_at, updated_at, deleted_at
        FROM console_admins
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(admins)
}

/// 更新管理员
pub async fn update_admin(
    pool: &Pool<Postgres>,
    admin_id: i32,
    request: UpdateConsoleAdminRequest,
) -> Result<bool, Error> {
    if let Some(is_active) = request.is_active {
        let result = sqlx::query(
            r#"
            UPDATE console_admins
            SET is_active = $1, updated_at = CURRENT_TIMESTAMP
            WHERE id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(is_active)
        .bind(admin_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    } else {
        Ok(false)
    }
}

/// 删除管理员（软删除）
pub async fn delete_admin(pool: &Pool<Postgres>, admin_id: i32) -> Result<bool, Error> {
    let result = sqlx::query(
        r#"
        UPDATE console_admins
        SET deleted_at = CURRENT_TIMESTAMP, is_active = FALSE
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(admin_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

// ==================== 权限操作 ====================

/// 创建权限
pub async fn create_permission(
    pool: &Pool<Postgres>,
    request: CreateConsolePermissionRequest,
) -> Result<ConsolePermissionDto, Error> {
    let permission = sqlx::query_as::<_, ConsolePermissionDto>(
        r#"
        INSERT INTO console_permissions (route_path, method, description)
        VALUES ($1, $2, $3)
        RETURNING id, uuid, route_path, method, description, created_at, updated_at, deleted_at
        "#,
    )
    .bind(request.route_path)
    .bind(request.method)
    .bind(request.description)
    .fetch_one(pool)
    .await?;

    Ok(permission)
}

/// 根据路由和方法获取权限
pub async fn get_permission_by_route(
    pool: &Pool<Postgres>,
    route_path: &str,
    method: &str,
) -> Result<Option<ConsolePermissionDto>, Error> {
    let permission = sqlx::query_as::<_, ConsolePermissionDto>(
        r#"
        SELECT id, uuid, route_path, method, description, created_at, updated_at, deleted_at
        FROM console_permissions
        WHERE route_path = $1 AND method = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(route_path)
    .bind(method)
    .fetch_optional(pool)
    .await?;

    Ok(permission)
}

/// 列出所有权限
pub async fn list_permissions(
    pool: &Pool<Postgres>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ConsolePermissionDto>, Error> {
    let limit = limit.unwrap_or(9999);
    let offset = offset.unwrap_or(0);

    let permissions = sqlx::query_as::<_, ConsolePermissionDto>(
        r#"
        SELECT id, uuid, route_path, method, description, created_at, updated_at, deleted_at
        FROM console_permissions
        WHERE deleted_at IS NULL
        ORDER BY route_path, method
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(permissions)
}

/// 授予权限给管理员
pub async fn grant_permission(
    pool: &Pool<Postgres>,
    request: GrantConsolePermissionRequest,
) -> Result<ConsoleAdminPermissionDto, Error> {
    let admin_permission = sqlx::query_as::<_, ConsoleAdminPermissionDto>(
        r#"
        INSERT INTO console_admin_permissions (admin_id, permission_id)
        VALUES ($1, $2)
        RETURNING id, uuid, admin_id, permission_id, granted_at, granted_by
        "#,
    )
    .bind(request.admin_id)
    .bind(request.permission_id)
    .fetch_one(pool)
    .await?;

    Ok(admin_permission)
}

/// 撤销管理员的权限
pub async fn revoke_permission(
    pool: &Pool<Postgres>,
    admin_id: i32,
    permission_id: i32,
) -> Result<bool, Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM console_admin_permissions
        WHERE admin_id = $1 AND permission_id = $2
        "#,
    )
    .bind(admin_id)
    .bind(permission_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// 检查管理员是否有某个权限
pub async fn check_admin_permission(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    route_path: &str,
    method: &str,
) -> Result<bool, Error> {
    let result: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM console_admins a
        JOIN console_admin_permissions ap ON a.id = ap.admin_id
        JOIN console_permissions p ON ap.permission_id = p.id
        WHERE a.user_uuid = $1
          AND a.is_active = TRUE
          AND a.deleted_at IS NULL
          AND p.route_path = $2
          AND p.method = $3
          AND p.deleted_at IS NULL
        "#,
    )
    .bind(user_uuid)
    .bind(route_path)
    .bind(method)
    .fetch_optional(pool)
    .await?;

    Ok(result.unwrap_or(0) > 0)
}

/// 获取管理员的所有权限
pub async fn get_admin_permissions(
    pool: &Pool<Postgres>,
    admin_id: i32,
) -> Result<Vec<ConsolePermissionDto>, Error> {
    let permissions = sqlx::query_as::<_, ConsolePermissionDto>(
        r#"
        SELECT p.id, p.uuid, p.route_path, p.method, p.description, p.created_at, p.updated_at, p.deleted_at
        FROM console_permissions p
        JOIN console_admin_permissions ap ON p.id = ap.permission_id
        WHERE ap.admin_id = $1 AND p.deleted_at IS NULL
        ORDER BY p.route_path, p.method
        "#,
    )
    .bind(admin_id)
    .fetch_all(pool)
    .await?;

    Ok(permissions)
}

/// 从路由同步权限到数据库
pub async fn sync_permissions_from_routes(
    pool: &Pool<Postgres>,
    routes: Vec<(String, String, Option<String>)>, // (route_path, method, description)
) -> Result<(), Error> {
    let mut tx = pool.begin().await?;

    for (route_path, method, description) in routes {
        let existing: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM console_permissions 
            WHERE route_path = $1 AND method = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(&route_path)
        .bind(&method)
        .fetch_optional(&mut *tx)
        .await?;

        if existing.unwrap_or(0) == 0 {
            sqlx::query(
                r#"
                INSERT INTO console_permissions (route_path, method, description)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(route_path)
            .bind(method)
            .bind(description)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

/// 从 MetaRoute 同步权限到数据库
pub async fn sync_route_permissions(
    pool: &Pool<Postgres>,
    meta_route: &crate::routes::route::MetaRoute,
) -> Result<(), Error> {
    use crate::routes::route::{RequestMethod, RouteGroup};

    let mut routes = Vec::new();

    for route_group in &meta_route.routes {
        let RouteGroup {
            prefix,
            routes: route_items,
        } = route_group;

        for route_item in route_items {
            let full_path = format!("{}{}{}", meta_route.prefix, prefix, route_item.path);
            let method = match route_item.method {
                RequestMethod::GET => "GET",
                RequestMethod::POST => "POST",
                RequestMethod::PUT => "PUT",
                RequestMethod::DELETE => "DELETE",
                RequestMethod::PATCH => "PATCH",
            }
            .to_string();

            let description = Some(format!("{} {}", method, full_path));
            routes.push((full_path, method, description));
        }
    }

    sync_permissions_from_routes(pool, routes).await?;
    Ok(())
}
