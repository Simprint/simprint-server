use crate::dto::machine_users::MachineUser;
use crate::entitys::machine_users::{CreateMachineUserRequest, UpdateMachineUserRequest};
use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

/// 插入新机器用户（支持 version_info 一次性插入）
pub async fn insert_machine_user(
    pool: &Pool<Postgres>,
    request: &CreateMachineUserRequest,
) -> Result<i32, Error> {
    let sql = "
        INSERT INTO machine_users (
            machine_code, user_uuid, platform, status, version_info, hardware_hash, hardware_raw, bind_time, created_at
        ) VALUES (
            $1, $2, $3, 'active', $4, $5, $6, NOW(), NOW()
        ) RETURNING id
    ";

    // 解析 version_info JSON 字符串
    let version_info_value = request
        .version_info
        .as_ref()
        .and_then(|v| serde_json::from_str::<serde_json::Value>(v).ok());

    let result: (i32,) = sqlx::query_as(sql)
        .bind(&request.machine_code)
        .bind(&request.user_uuid)
        .bind(&request.platform)
        .bind(&version_info_value)
        .bind(&request.hardware_hash)
        .bind(&request.hardware_raw)
        .fetch_one(pool)
        .await?;

    Ok(result.0)
}

/// 根据ID查询机器用户
pub async fn query_machine_user_by_id(
    pool: &Pool<Postgres>,
    id: i32,
) -> Result<MachineUser, Error> {
    let machine_user: MachineUser = sqlx::query_as("SELECT * FROM machine_users WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(machine_user)
}

/// 根据机器码查询机器用户（返回多个记录中的第一个）
pub async fn query_machine_user_by_code(
    pool: &Pool<Postgres>,
    machine_code: &str,
) -> Result<MachineUser, Error> {
    let machine_user: MachineUser = sqlx::query_as(
        "SELECT * FROM machine_users WHERE machine_code = $1 ORDER BY bind_time DESC LIMIT 1",
    )
    .bind(machine_code)
    .fetch_one(pool)
    .await?;

    Ok(machine_user)
}

/// 根据机器码查询所有机器用户记录
pub async fn query_machines_by_code(
    pool: &Pool<Postgres>,
    machine_code: &str,
) -> Result<Vec<MachineUser>, Error> {
    let machines: Vec<MachineUser> = sqlx::query_as(
        "SELECT * FROM machine_users WHERE machine_code = $1 ORDER BY bind_time DESC",
    )
    .bind(machine_code)
    .fetch_all(pool)
    .await?;

    Ok(machines)
}

/// 根据机器码和用户UUID查询机器用户
pub async fn query_machine_user_by_code_and_user(
    pool: &Pool<Postgres>,
    machine_code: &str,
    user_uuid: &Uuid,
) -> Result<Option<MachineUser>, Error> {
    let machine_user: Option<MachineUser> =
        sqlx::query_as("SELECT * FROM machine_users WHERE machine_code = $1 AND user_uuid = $2")
            .bind(machine_code)
            .bind(user_uuid)
            .fetch_optional(pool)
            .await?;

    Ok(machine_user)
}

/// 判断机器是否允许
pub async fn machine_not_allow(pool: &Pool<Postgres>, machine_code: &str) -> Result<bool, Error> {
    let allow: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM machine_users WHERE machine_code = $1 AND allow = false) ",
    )
    .bind(machine_code)
    .fetch_one(pool)
    .await?;

    Ok(allow)
}

/// 拉黑或取消拉黑机器
pub async fn allow_or_blacklist_machine(pool: &Pool<Postgres>, machine_code: &str, allow: bool) -> Result<bool, Error> {
    let row = sqlx::query("UPDATE machine_users SET allow = $1 WHERE machine_code = $2")
        .bind(allow)
        .bind(machine_code)
        .execute(pool)
        .await?;

    Ok(row.rows_affected() > 1)
}

/// 根据用户UUID查询机器列表
pub async fn query_machines_by_user_uuid(
    pool: &Pool<Postgres>,
    user_uuid: &Uuid,
) -> Result<Vec<MachineUser>, Error> {
    let machines: Vec<MachineUser> =
        sqlx::query_as("SELECT * FROM machine_users WHERE user_uuid = $1 ORDER BY bind_time DESC")
            .bind(user_uuid)
            .fetch_all(pool)
            .await?;

    Ok(machines)
}

/// 根据多个用户 UUID 批量查询机器列表（返回所有匹配的 machine_users，含 user_uuid 便于按用户分组）
pub async fn query_machines_by_user_uuids(
    pool: &Pool<Postgres>,
    user_uuids: &[Uuid],
) -> Result<Vec<MachineUser>, Error> {
    if user_uuids.is_empty() {
        return Ok(vec![]);
    }
    let machines: Vec<MachineUser> = sqlx::query_as(
        "SELECT * FROM machine_users WHERE user_uuid = ANY($1) ORDER BY user_uuid, bind_time DESC",
    )
    .bind(user_uuids)
    .fetch_all(pool)
    .await?;

    Ok(machines)
}

/// 查询机器用户列表（分页）
pub async fn query_machine_users(
    pool: &Pool<Postgres>,
    user_uuid: Option<&Uuid>,
    platform: Option<&str>,
    status: Option<&str>,
    page_num: i32,
    page_size: i32,
) -> Result<(i64, Vec<MachineUser>), Error> {
    // 构建查询条件
    let mut where_clauses = vec!["1=1".to_string()];
    let mut param_index = 1;

    if let Some(_uuid) = user_uuid {
        where_clauses.push(format!("user_uuid = ${}", param_index));
        param_index += 1;
    }
    if let Some(_plat) = platform {
        where_clauses.push(format!("platform = ${}", param_index));
        param_index += 1;
    }
    if let Some(_st) = status {
        where_clauses.push(format!("status = ${}", param_index));
        param_index += 1;
    }

    let where_sql = where_clauses.join(" AND ");

    // 获取总数
    let count_sql = format!("SELECT COUNT(*) FROM machine_users WHERE {}", where_sql);
    let count_result: (i64,) = match (user_uuid, platform, status) {
        (Some(u), Some(p), Some(s)) => {
            sqlx::query_as(&count_sql)
                .bind(u)
                .bind(p)
                .bind(s)
                .fetch_one(pool)
                .await?
        }
        (Some(u), Some(p), None) => {
            sqlx::query_as(&count_sql)
                .bind(u)
                .bind(p)
                .fetch_one(pool)
                .await?
        }
        (Some(u), None, Some(s)) => {
            sqlx::query_as(&count_sql)
                .bind(u)
                .bind(s)
                .fetch_one(pool)
                .await?
        }
        (Some(u), None, None) => sqlx::query_as(&count_sql).bind(u).fetch_one(pool).await?,
        (None, Some(p), Some(s)) => {
            sqlx::query_as(&count_sql)
                .bind(p)
                .bind(s)
                .fetch_one(pool)
                .await?
        }
        (None, Some(p), None) => sqlx::query_as(&count_sql).bind(p).fetch_one(pool).await?,
        (None, None, Some(s)) => sqlx::query_as(&count_sql).bind(s).fetch_one(pool).await?,
        (None, None, None) => sqlx::query_as(&count_sql).fetch_one(pool).await?,
    };

    // 获取分页列表
    let list_sql = format!(
        "SELECT * FROM machine_users WHERE {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
        where_sql,
        param_index,
        param_index + 1
    );
    let machines: Vec<MachineUser> = match (user_uuid, platform, status) {
        (Some(u), Some(p), Some(s)) => {
            sqlx::query_as(&list_sql)
                .bind(u)
                .bind(p)
                .bind(s)
                .bind(page_size)
                .bind((page_num - 1) * page_size)
                .fetch_all(pool)
                .await?
        }
        (Some(u), Some(p), None) => {
            sqlx::query_as(&list_sql)
                .bind(u)
                .bind(p)
                .bind(page_size)
                .bind((page_num - 1) * page_size)
                .fetch_all(pool)
                .await?
        }
        (Some(u), None, Some(s)) => {
            sqlx::query_as(&list_sql)
                .bind(u)
                .bind(s)
                .bind(page_size)
                .bind((page_num - 1) * page_size)
                .fetch_all(pool)
                .await?
        }
        (Some(u), None, None) => {
            sqlx::query_as(&list_sql)
                .bind(u)
                .bind(page_size)
                .bind((page_num - 1) * page_size)
                .fetch_all(pool)
                .await?
        }
        (None, Some(p), Some(s)) => {
            sqlx::query_as(&list_sql)
                .bind(p)
                .bind(s)
                .bind(page_size)
                .bind((page_num - 1) * page_size)
                .fetch_all(pool)
                .await?
        }
        (None, Some(p), None) => {
            sqlx::query_as(&list_sql)
                .bind(p)
                .bind(page_size)
                .bind((page_num - 1) * page_size)
                .fetch_all(pool)
                .await?
        }
        (None, None, Some(s)) => {
            sqlx::query_as(&list_sql)
                .bind(s)
                .bind(page_size)
                .bind((page_num - 1) * page_size)
                .fetch_all(pool)
                .await?
        }
        (None, None, None) => {
            sqlx::query_as(&list_sql)
                .bind(page_size)
                .bind((page_num - 1) * page_size)
                .fetch_all(pool)
                .await?
        }
    };

    Ok((count_result.0, machines))
}

/// 更新机器用户
pub async fn update_machine_user(
    pool: &Pool<Postgres>,
    id: i32,
    request: &UpdateMachineUserRequest,
) -> Result<bool, Error> {
    let sql = "
        UPDATE machine_users SET
            user_uuid = COALESCE($1, user_uuid),
            platform = COALESCE($2, platform),
            status = COALESCE($3, status),
            hardware_hash = COALESCE($4, hardware_hash),
            hardware_raw = COALESCE($5, hardware_raw),
            updated_at = NOW()
        WHERE id = $6
    ";

    let row = sqlx::query(sql)
        .bind(&request.user_uuid)
        .bind(&request.platform)
        .bind(&request.status)
        .bind(&request.hardware_hash)
        .bind(&request.hardware_raw)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(row.rows_affected() == 1)
}

/// 更新机器版本信息（整机更新，不更新 updated_at）
pub async fn update_machine_version_info(
    pool: &Pool<Postgres>,
    machine_code: &str,
    version_info: &serde_json::Value,
) -> Result<bool, Error> {
    let sql = "
        UPDATE machine_users SET
            version_info = $1
        WHERE machine_code = $2
    ";

    let row = sqlx::query(sql)
        .bind(version_info)
        .bind(machine_code)
        .execute(pool)
        .await?;

    Ok(row.rows_affected() > 0)
}

/// 一次性更新机器绑定时间和版本信息（用于已存在记录的更新场景）
pub async fn update_machine_bind_and_version(
    pool: &Pool<Postgres>,
    machine_code: &str,
    user_uuid: &Uuid,
    version_info: Option<&serde_json::Value>,
) -> Result<bool, Error> {
    let sql = "
        UPDATE machine_users SET
            bind_time = NOW(),
            updated_at = NOW(),
            version_info = COALESCE($3, version_info)
        WHERE machine_code = $1 AND user_uuid = $2
    ";

    let row = sqlx::query(sql)
        .bind(machine_code)
        .bind(user_uuid)
        .bind(version_info)
        .execute(pool)
        .await?;

    Ok(row.rows_affected() > 0)
}

/// 更新用户的 updated_at（仅更新指定用户的绑定时间）
pub async fn update_user_bind_time(
    pool: &Pool<Postgres>,
    machine_code: &str,
    user_uuid: &Uuid,
) -> Result<bool, Error> {
    let sql = "
        UPDATE machine_users SET
            updated_at = NOW()
        WHERE machine_code = $1 AND user_uuid = $2
    ";

    let row = sqlx::query(sql)
        .bind(machine_code)
        .bind(user_uuid)
        .execute(pool)
        .await?;

    Ok(row.rows_affected() > 0)
}

/// 绑定用户到机器（如果已存在则更新，否则创建新记录）
pub async fn bind_user_to_machine(
    pool: &Pool<Postgres>,
    machine_code: &str,
    user_uuid: &Uuid,
) -> Result<bool, Error> {
    // 先检查是否已有绑定记录
    let existing = query_machine_user_by_code_and_user(pool, machine_code, user_uuid).await?;

    if existing.is_some() {
        // 已存在，只更新绑定时间
        let sql = "
            UPDATE machine_users SET
                bind_time = NOW(),
                updated_at = NOW()
            WHERE machine_code = $1 AND user_uuid = $2
        ";

        let row = sqlx::query(sql)
            .bind(machine_code)
            .bind(user_uuid)
            .execute(pool)
            .await?;

        Ok(row.rows_affected() > 0)
    } else {
        // 不存在，创建新记录
        let sql = "
            INSERT INTO machine_users (
                machine_code, user_uuid, status, bind_time, created_at
            ) VALUES (
                $1, $2, 'active', NOW(), NOW()
            )
        ";

        let row = sqlx::query(sql)
            .bind(machine_code)
            .bind(user_uuid)
            .execute(pool)
            .await?;

        Ok(row.rows_affected() > 0)
    }
}

/// 解绑用户（删除绑定记录）
pub async fn unbind_user_from_machine(
    pool: &Pool<Postgres>,
    machine_code: &str,
    user_uuid: &Uuid,
) -> Result<bool, Error> {
    let sql = "
        DELETE FROM machine_users 
        WHERE machine_code = $1 AND user_uuid = $2
    ";

    let row = sqlx::query(sql)
        .bind(machine_code)
        .bind(user_uuid)
        .execute(pool)
        .await?;

    Ok(row.rows_affected() > 0)
}

/// 更新机器绑定时间（当用户重新登录已存在的机器时）
pub async fn update_machine_bind_time(
    pool: &Pool<Postgres>,
    machine_code: &str,
    user_uuid: &Uuid,
) -> Result<bool, Error> {
    let sql = "
        UPDATE machine_users SET
            updated_at = NOW(),
            bind_time = NOW()
        WHERE machine_code = $1 AND user_uuid = $2
    ";

    let row = sqlx::query(sql)
        .bind(machine_code)
        .bind(user_uuid)
        .execute(pool)
        .await?;

    Ok(row.rows_affected() > 0)
}
