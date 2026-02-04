use crate::dto::gray_resources::GrayResource;
use crate::entitys::gray_resources::{BatchCreateGrayResourceRequest, CreateGrayResourceRequest};
use sqlx::{Error, Pool, Postgres};

/// 创建灰度资源关联
pub async fn create_gray_resource(
    pool: &Pool<Postgres>,
    request: &CreateGrayResourceRequest,
) -> Result<i32, Error> {
    let sql = "
        INSERT INTO gray_resources (gray_release_id, version_id, sort_order)
        VALUES ($1, $2, $3) RETURNING id
    ";

    let result: (i32,) = sqlx::query_as(sql)
        .bind(request.gray_release_id)
        .bind(request.version_id)
        .bind(request.sort_order.unwrap_or(0))
        .fetch_one(pool)
        .await?;

    Ok(result.0)
}

/// 批量创建灰度资源
pub async fn batch_create_gray_resources(
    pool: &Pool<Postgres>,
    request: &BatchCreateGrayResourceRequest,
) -> Result<i64, Error> {
    let mut tx = pool.begin().await?;

    let mut count = 0;
    for (index, version_id) in request.version_ids.iter().enumerate() {
        let sql = "
            INSERT INTO gray_resources (gray_release_id, version_id, sort_order)
            VALUES ($1, $2, $3)
        ";

        sqlx::query(sql)
            .bind(request.gray_release_id)
            .bind(version_id)
            .bind(index as i32)
            .execute(&mut *tx)
            .await?;

        count += 1;
    }

    tx.commit().await?;
    Ok(count)
}

/// 查询灰度发布的所有资源
pub async fn query_gray_resources(
    pool: &Pool<Postgres>,
    gray_release_id: i32,
) -> Result<Vec<GrayResource>, Error> {
    let resources: Vec<GrayResource> = sqlx::query_as(
        "SELECT * FROM gray_resources WHERE gray_release_id = $1 ORDER BY sort_order",
    )
    .bind(gray_release_id)
    .fetch_all(pool)
    .await?;

    Ok(resources)
}

/// 根据ID查询灰度资源
pub async fn query_gray_resource_by_id(
    pool: &Pool<Postgres>,
    id: i32,
) -> Result<GrayResource, Error> {
    let resource: GrayResource = sqlx::query_as("SELECT * FROM gray_resources WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(resource)
}

/// 查询最大排序值
pub async fn query_max_sort_order(
    pool: &Pool<Postgres>,
    gray_release_id: i32,
) -> Result<i32, Error> {
    let result: Option<(i32,)> =
        sqlx::query_as("SELECT MAX(sort_order) FROM gray_resources WHERE gray_release_id = $1")
            .bind(gray_release_id)
            .fetch_optional(pool)
            .await?;

    Ok(result.map(|r| r.0).unwrap_or(0))
}

/// 更新排序值
pub async fn update_sort_order(
    pool: &Pool<Postgres>,
    id: i32,
    sort_order: i32,
) -> Result<bool, Error> {
    let sql = "UPDATE gray_resources SET sort_order = $1 WHERE id = $2";
    let row = sqlx::query(sql)
        .bind(sort_order)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(row.rows_affected() == 1)
}

/// 按灰度发布 ID 删除该灰度下所有资源关联（用于删除灰度前清理外键）
pub async fn delete_resources_by_gray_release_id(
    pool: &Pool<Postgres>,
    gray_release_id: i32,
) -> Result<u64, Error> {
    let row = sqlx::query("DELETE FROM gray_resources WHERE gray_release_id = $1")
        .bind(gray_release_id)
        .execute(pool)
        .await?;
    Ok(row.rows_affected())
}

/// 删除灰度资源
pub async fn delete_gray_resource(pool: &Pool<Postgres>, id: i32) -> Result<bool, Error> {
    let sql = "DELETE FROM gray_resources WHERE id = $1";
    let row = sqlx::query(sql).bind(id).execute(pool).await?;
    Ok(row.rows_affected() == 1)
}
