//! Console Gateway API 密钥数据库操作

use rand::Rng;
use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::dto::ConsoleApiKeyDto;
use crate::entitys::CreateConsoleApiKeyRequest;

/// 创建 API 密钥
pub async fn create_api_key(
    pool: &Pool<Postgres>,
    request: CreateConsoleApiKeyRequest,
) -> Result<ConsoleApiKeyDto, Error> {
    let key_id = Uuid::new_v4().to_string();
    let key_secret = generate_api_secret();

    let api_key = sqlx::query_as::<_, ConsoleApiKeyDto>(
        r#"
        INSERT INTO console_api_keys (key_id, key_secret, name, description, is_active, expires_at)
        VALUES ($1, $2, $3, $4, TRUE, $5)
        RETURNING id, uuid, key_id, key_secret, name, description, is_active, 
                  created_at, updated_at, expires_at, last_used_at, created_by, deleted_at
        "#,
    )
    .bind(&key_id)
    .bind(&key_secret)
    .bind(&request.name)
    .bind(&request.description)
    .bind(request.expires_at)
    .fetch_one(pool)
    .await?;

    Ok(api_key)
}

/// 列出 API 密钥
pub async fn list_api_keys(
    pool: &Pool<Postgres>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ConsoleApiKeyDto>, Error> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let api_keys = sqlx::query_as::<_, ConsoleApiKeyDto>(
        r#"
        SELECT id, uuid, key_id, key_secret, name, description, is_active,
               created_at, updated_at, expires_at, last_used_at, created_by, deleted_at
        FROM console_api_keys
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(api_keys)
}

/// 停用 API 密钥
pub async fn deactivate_api_key(pool: &Pool<Postgres>, key_id: &str) -> Result<bool, Error> {
    let result = sqlx::query(
        r#"
        UPDATE console_api_keys 
        SET is_active = FALSE, updated_at = CURRENT_TIMESTAMP
        WHERE key_id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(key_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// 验证 API 密钥
pub async fn validate_api_key(
    pool: &Pool<Postgres>,
    key_id: &str,
    key_secret: &str,
) -> Result<Option<ConsoleApiKeyDto>, Error> {
    let api_key = sqlx::query_as::<_, ConsoleApiKeyDto>(
        r#"
        SELECT id, uuid, key_id, key_secret, name, description, is_active,
               created_at, updated_at, expires_at, last_used_at, created_by, deleted_at
        FROM console_api_keys 
        WHERE key_id = $1 AND key_secret = $2 AND is_active = TRUE AND deleted_at IS NULL
        "#,
    )
    .bind(key_id)
    .bind(key_secret)
    .fetch_optional(pool)
    .await?;

    if let Some(ref api_key) = api_key {
        // 检查是否过期
        if let Some(expires_at) = api_key.expires_at {
            if chrono::Utc::now() > expires_at {
                return Ok(None);
            }
        }

        // 更新最后使用时间
        update_last_used(pool, api_key.id).await?;
    }

    Ok(api_key)
}

/// 更新最后使用时间
async fn update_last_used(pool: &Pool<Postgres>, api_key_id: i32) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE console_api_keys 
        SET last_used_at = CURRENT_TIMESTAMP
        WHERE id = $1
        "#,
    )
    .bind(api_key_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// 根据 key_id 获取 API 密钥
pub async fn get_api_key_by_key_id(
    pool: &Pool<Postgres>,
    key_id: &str,
) -> Result<Option<ConsoleApiKeyDto>, Error> {
    let api_key = sqlx::query_as::<_, ConsoleApiKeyDto>(
        r#"
        SELECT id, uuid, key_id, key_secret, name, description, is_active,
               created_at, updated_at, expires_at, last_used_at, created_by, deleted_at
        FROM console_api_keys 
        WHERE key_id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(key_id)
    .fetch_optional(pool)
    .await?;

    Ok(api_key)
}

/// 删除 API 密钥（软删除）
pub async fn delete_api_key(pool: &Pool<Postgres>, key_id: &str) -> Result<bool, Error> {
    let result = sqlx::query(
        r#"
        UPDATE console_api_keys 
        SET deleted_at = CURRENT_TIMESTAMP, is_active = FALSE
        WHERE key_id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(key_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// 生成 API 密钥密文
fn generate_api_secret() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             abcdefghijklmnopqrstuvwxyz\
                             0123456789";
    let mut rng = rand::rng();

    (0..64)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
