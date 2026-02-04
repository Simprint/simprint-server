use crate::dto::gray_releases::GrayRelease;
use crate::entitys::gray_releases::{CreateGrayReleaseRequest, UpdateGrayReleaseRequest};
use sqlx::{Error, Pool, Postgres};

/// 创建灰度发布
pub async fn create_gray_release(
    pool: &Pool<Postgres>,
    request: &CreateGrayReleaseRequest,
) -> Result<i32, Error> {
    let sql = "
        INSERT INTO gray_releases (
            name, description, platform, status, start_time, end_time, 
            max_machines, priority, strategy_type, strategy_config, created_at
        ) VALUES (
            $1, $2, $3, 'pending', $4, $5, $6, $7, $8, $9, NOW()
        ) RETURNING id
    ";

    let result: (i32,) = sqlx::query_as(sql)
        .bind(&request.name)
        .bind(&request.description)
        .bind(&request.platform)
        .bind(request.start_time)
        .bind(request.end_time)
        .bind(request.max_machines)
        .bind(request.priority.unwrap_or(0))
        .bind(&request.strategy_type)
        .bind(&request.strategy_config)
        .fetch_one(pool)
        .await?;

    Ok(result.0)
}

/// 查询灰度发布
pub async fn query_gray_release_by_id(
    pool: &Pool<Postgres>,
    id: i32,
) -> Result<GrayRelease, Error> {
    let release: GrayRelease = sqlx::query_as("SELECT * FROM gray_releases WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(release)
}

/// 查询灰度发布列表
pub async fn query_gray_releases(
    pool: &Pool<Postgres>,
    platform: Option<&str>,
    status: Option<&str>,
    page_num: i32,
    page_size: i32,
) -> Result<(i64, Vec<GrayRelease>), Error> {
    let mut where_conditions = vec![];
    let mut param_index = 1;

    if let Some(_plat) = platform {
        where_conditions.push(format!("platform = ${}", param_index));
        param_index += 1;
    }
    if let Some(_st) = status {
        where_conditions.push(format!("status = ${}", param_index));
        param_index += 1;
    }

    let where_clause = if where_conditions.is_empty() {
        "".to_string()
    } else {
        format!("WHERE {}", where_conditions.join(" AND "))
    };

    // 获取总数
    let count_sql = format!("SELECT COUNT(*) FROM gray_releases {}", where_clause);
    let count_query = match (platform, status) {
        (Some(plat), Some(st)) => sqlx::query_as(&count_sql).bind(plat).bind(st),
        (Some(plat), None) => sqlx::query_as(&count_sql).bind(plat),
        (None, Some(st)) => sqlx::query_as(&count_sql).bind(st),
        (None, None) => sqlx::query_as(&count_sql),
    };
    let total: (i64,) = count_query.fetch_one(pool).await?;

    // 获取分页列表
    let list_sql = format!(
        "SELECT * FROM gray_releases {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
        where_clause,
        param_index,
        param_index + 1
    );
    let list_query = match (platform, status) {
        (Some(plat), Some(st)) => sqlx::query_as(&list_sql)
            .bind(plat)
            .bind(st)
            .bind(page_size)
            .bind((page_num - 1) * page_size),
        (Some(plat), None) => sqlx::query_as(&list_sql)
            .bind(plat)
            .bind(page_size)
            .bind((page_num - 1) * page_size),
        (None, Some(st)) => sqlx::query_as(&list_sql)
            .bind(st)
            .bind(page_size)
            .bind((page_num - 1) * page_size),
        (None, None) => sqlx::query_as(&list_sql)
            .bind(page_size)
            .bind((page_num - 1) * page_size),
    };
    let releases: Vec<GrayRelease> = list_query.fetch_all(pool).await?;

    Ok((total.0, releases))
}

/// 更新灰度发布
pub async fn update_gray_release(
    pool: &Pool<Postgres>,
    id: i32,
    request: &UpdateGrayReleaseRequest,
) -> Result<bool, Error> {
    let sql = "
        UPDATE gray_releases SET
            name = COALESCE($1, name),
            description = COALESCE($2, description),
            status = COALESCE($3, status),
            end_time = COALESCE($4, end_time),
            max_machines = COALESCE($5, max_machines),
            priority = COALESCE($6, priority),
            strategy_type = COALESCE($7, strategy_type),
            strategy_config = COALESCE($8, strategy_config),
            updated_at = NOW()
        WHERE id = $9
    ";

    let row = sqlx::query(sql)
        .bind(&request.name)
        .bind(&request.description)
        .bind(&request.status)
        .bind(request.end_time)
        .bind(request.max_machines)
        .bind(request.priority)
        .bind(&request.strategy_type)
        .bind(&request.strategy_config)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(row.rows_affected() == 1)
}

/// 更新配额计数
pub async fn update_allocated_count(
    pool: &Pool<Postgres>,
    id: i32,
    increment: i32,
) -> Result<bool, Error> {
    let sql = "
        UPDATE gray_releases 
        SET allocated_count = allocated_count + $1, updated_at = NOW()
        WHERE id = $2
    ";

    let row = sqlx::query(sql)
        .bind(increment)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(row.rows_affected() == 1)
}

/// 增加分配计数
pub async fn increment_allocated_count(pool: &Pool<Postgres>, id: i32) -> Result<bool, Error> {
    update_allocated_count(pool, id, 1).await
}

/// 减少分配计数
pub async fn decrement_allocated_count(pool: &Pool<Postgres>, id: i32) -> Result<bool, Error> {
    update_allocated_count(pool, id, -1).await
}

/// 查询活跃的灰度发布
pub async fn query_active_gray_releases(
    pool: &Pool<Postgres>,
    machine_code: &str,
) -> Result<Vec<GrayRelease>, Error> {
    let releases: Vec<GrayRelease> = sqlx::query_as(
        r#"
        SELECT gr.* FROM gray_releases gr
        INNER JOIN machine_gray_allocations mga ON gr.id = mga.gray_release_id
        WHERE mga.machine_code = $1 
          AND mga.status = 'active'
          AND gr.status = 'active'
          AND NOW() >= gr.start_time
          AND (gr.end_time IS NULL OR NOW() <= gr.end_time)
        ORDER BY gr.priority DESC, gr.created_at DESC
        "#,
    )
    .bind(machine_code)
    .fetch_all(pool)
    .await?;

    Ok(releases)
}

/// 删除灰度发布（物理删除）
pub async fn delete_gray_release(pool: &Pool<Postgres>, id: i32) -> Result<bool, Error> {
    let sql = "DELETE FROM gray_releases WHERE id = $1";
    let row = sqlx::query(sql).bind(id).execute(pool).await?;
    Ok(row.rows_affected() == 1)
}

/// 检查白名单策略（直接 JOIN 查询，单次查询 O(1)）
pub async fn check_whitelist_strategy(
    pool: &Pool<Postgres>,
    machine_code: &str,
) -> Result<Option<GrayRelease>, Error> {
    let release = sqlx::query_as::<_, GrayRelease>(
        r#"
        SELECT gr.* FROM gray_releases gr
        INNER JOIN machine_gray_allocations mga ON gr.id = mga.gray_release_id
        WHERE mga.machine_code = $1 
          AND mga.status = 'active'
          AND gr.status = 'active'
          AND gr.start_time <= NOW()
          AND (gr.end_time IS NULL OR gr.end_time >= NOW())
        ORDER BY gr.priority DESC, gr.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(machine_code)
    .fetch_optional(pool)
    .await?;

    Ok(release)
}

/// 检查标签策略（数据库层面过滤，单次查询）
pub async fn check_tags_strategy(
    pool: &Pool<Postgres>,
    machine_code: &str,
) -> Result<Option<GrayRelease>, Error> {
    let release = sqlx::query_as::<_, GrayRelease>(
        r#"
        SELECT DISTINCT gr.* FROM gray_releases gr
        INNER JOIN machine_users mu ON mu.machine_code = $1
        INNER JOIN machine_user_tags mut ON mut.machine_user_id = mu.id
        INNER JOIN machine_tags mt ON mt.id = mut.tag_id
        WHERE gr.status = 'active'
          AND gr.strategy_type = 'filter_user_tag'
          AND gr.start_time <= NOW()
          AND (gr.end_time IS NULL OR gr.end_time >= NOW())
          AND mt.tag_name = ANY(
            SELECT jsonb_array_elements_text(gr.strategy_config->'tags')
          )
        GROUP BY gr.id
        HAVING CASE 
          WHEN COALESCE((gr.strategy_config->>'match_all')::boolean, false) = true 
          THEN COUNT(DISTINCT mt.tag_name) >= (
            SELECT jsonb_array_length(gr.strategy_config->'tags')
          )
          ELSE COUNT(DISTINCT mt.tag_name) >= 1
        END
        ORDER BY gr.priority DESC, gr.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(machine_code)
    .fetch_optional(pool)
    .await?;

    Ok(release)
}

/// 查询需要动态计算的灰度发布（百分比、随机策略）
pub async fn query_dynamic_strategy_releases(
    pool: &Pool<Postgres>,
) -> Result<Vec<GrayRelease>, Error> {
    let releases = sqlx::query_as::<_, GrayRelease>(
        r#"
        SELECT * FROM gray_releases
        WHERE status = 'active'
          AND strategy_type IN ('filter_percentage', 'filter_random')
          AND start_time <= NOW()
          AND (end_time IS NULL OR end_time >= NOW())
        ORDER BY priority DESC, created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(releases)
}
