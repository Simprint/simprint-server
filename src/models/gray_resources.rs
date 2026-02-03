use crate::dto::gray_resources::GrayResource;
use crate::entitys::gray_resources::CreateGrayResourceRequest;
use sqlx::{Error, Pool, Postgres};

/// 创建灰度资源
pub async fn create_gray_resource(
    pool: &Pool<Postgres>,
    request: &CreateGrayResourceRequest,
) -> Result<i32, Error> {
    let sql = "
        INSERT INTO gray_resources (
            gray_release_id, version_id, sort_order
        ) VALUES (
            $1, $2, $3
        ) RETURNING id
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
    gray_release_id: i32,
    version_ids: &[i32],
) -> Result<usize, Error> {
    if version_ids.is_empty() {
        return Ok(0);
    }

    let mut tx = pool.begin().await?;
    let mut count = 0;

    for (index, version_id) in version_ids.iter().enumerate() {
        let sql = "
            INSERT INTO gray_resources (
                gray_release_id, version_id, sort_order
            ) VALUES (
                $1, $2, $3
            ) ON CONFLICT DO NOTHING
        ";

        sqlx::query(sql)
            .bind(gray_release_id)
            .bind(version_id)
            .bind(index as i32)
            .execute(&mut *tx)
            .await?;

        count += 1;
    }

    tx.commit().await?;
    Ok(count)
}

/// 查询灰度资源
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

/// 查询灰度发布的所有资源
pub async fn query_gray_resources_by_release_id(
    pool: &Pool<Postgres>,
    gray_release_id: i32,
) -> Result<Vec<GrayResource>, Error> {
    let resources: Vec<GrayResource> = sqlx::query_as(
        "SELECT * FROM gray_resources WHERE gray_release_id = $1 ORDER BY sort_order ASC, id ASC",
    )
    .bind(gray_release_id)
    .fetch_all(pool)
    .await?;

    Ok(resources)
}

/// 删除灰度资源
pub async fn delete_gray_resource(pool: &Pool<Postgres>, id: i32) -> Result<bool, Error> {
    let sql = "DELETE FROM gray_resources WHERE id = $1";
    let row = sqlx::query(sql).bind(id).execute(pool).await?;
    Ok(row.rows_affected() == 1)
}

/// 删除灰度发布的所有资源
pub async fn delete_gray_resources_by_release_id(
    pool: &Pool<Postgres>,
    gray_release_id: i32,
) -> Result<usize, Error> {
    let sql = "DELETE FROM gray_resources WHERE gray_release_id = $1";
    let row = sqlx::query(sql).bind(gray_release_id).execute(pool).await?;
    Ok(row.rows_affected() as usize)
}
