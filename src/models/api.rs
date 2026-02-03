use chrono::{DateTime, Utc};
use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::dto::ApiKeyDto;

/// 创建 API 密钥
pub async fn insert_api_key(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    name: &str,
    key_hash: &str,
    key_prefix: &str,
    permissions: &serde_json::Value,
    rate_limit: Option<i32>,
    daily_limit: Option<i32>,
    ip_whitelist: Option<&serde_json::Value>,
    expires_at: Option<DateTime<Utc>>,
) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO api_keys (user_uuid, name, key_hash, key_prefix, permissions,
                              rate_limit, daily_limit, ip_whitelist, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING uuid;
        "#,
    )
    .bind(user_uuid)
    .bind(name)
    .bind(key_hash)
    .bind(key_prefix)
    .bind(permissions)
    .bind(rate_limit)
    .bind(daily_limit)
    .bind(ip_whitelist)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(uuid)
}

/// 查询用户的 API 密钥列表
pub async fn fetch_api_keys(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    status: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<ApiKeyDto>, Error> {
    let recs = sqlx::query_as::<_, ApiKeyDto>(
        r#"
        SELECT id, uuid, user_uuid, name, key_hash, key_prefix, permissions,
               rate_limit, daily_limit, ip_whitelist, expires_at, usage_count,
               daily_usage, last_used_at, status, created_at, updated_at
        FROM api_keys
        WHERE user_uuid = $1
          AND ($2::varchar IS NULL OR status = $2)
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(user_uuid)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询用户的 API 密钥总数
pub async fn fetch_api_keys_count(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    status: Option<&str>,
) -> Result<i64, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM api_keys
        WHERE user_uuid = $1
          AND ($2::varchar IS NULL OR status = $2)
        "#,
    )
    .bind(user_uuid)
    .bind(status)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// 根据 UUID 查询 API 密钥
pub async fn fetch_api_key_by_uuid(
    pool: &Pool<Postgres>,
    key_uuid: Uuid,
) -> Result<Option<ApiKeyDto>, Error> {
    let rec = sqlx::query_as::<_, ApiKeyDto>(
        r#"
        SELECT id, uuid, user_uuid, name, key_hash, key_prefix, permissions,
               rate_limit, daily_limit, ip_whitelist, expires_at, usage_count,
               daily_usage, last_used_at, status, created_at, updated_at
        FROM api_keys
        WHERE uuid = $1
        "#,
    )
    .bind(key_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 更新 API 密钥
pub async fn update_api_key(
    pool: &Pool<Postgres>,
    key_uuid: Uuid,
    name: Option<&str>,
    permissions: Option<&serde_json::Value>,
    rate_limit: Option<i32>,
    daily_limit: Option<i32>,
    ip_whitelist: Option<&serde_json::Value>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE api_keys
        SET name = COALESCE($1, name),
            permissions = COALESCE($2, permissions),
            rate_limit = COALESCE($3, rate_limit),
            daily_limit = COALESCE($4, daily_limit),
            ip_whitelist = COALESCE($5, ip_whitelist),
            updated_at = CURRENT_TIMESTAMP
        WHERE uuid = $6
        "#,
    )
    .bind(name)
    .bind(permissions)
    .bind(rate_limit)
    .bind(daily_limit)
    .bind(ip_whitelist)
    .bind(key_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 撤销 API 密钥
pub async fn revoke_api_key(pool: &Pool<Postgres>, key_uuid: Uuid) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE api_keys SET status = 'revoked', updated_at = CURRENT_TIMESTAMP
        WHERE uuid = $1
        "#,
    )
    .bind(key_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 验证 API 密钥（根据 key_hash 查询）
pub async fn fetch_api_key_by_hash(
    pool: &Pool<Postgres>,
    key_hash: &str,
) -> Result<Option<ApiKeyDto>, Error> {
    let rec = sqlx::query_as::<_, ApiKeyDto>(
        r#"
        SELECT id, uuid, user_uuid, name, key_hash, key_prefix, permissions,
               rate_limit, daily_limit, ip_whitelist, expires_at, usage_count,
               daily_usage, last_used_at, status, created_at, updated_at
        FROM api_keys
        WHERE key_hash = $1 AND status = 'active'
        "#,
    )
    .bind(key_hash)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 更新 API 密钥使用统计
pub async fn update_api_key_usage(pool: &Pool<Postgres>, key_uuid: Uuid) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE api_keys
        SET usage_count = COALESCE(usage_count, 0) + 1,
            daily_usage = COALESCE(daily_usage, 0) + 1,
            last_used_at = CURRENT_TIMESTAMP
        WHERE uuid = $1
        "#,
    )
    .bind(key_uuid)
    .execute(pool)
    .await?;

    Ok(())
}
