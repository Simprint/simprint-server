use crate::dto::machine_gray_allocations::MachineGrayAllocation;
use sqlx::{Error, Pool, Postgres};

/// 分配机器到灰度发布
pub async fn allocate_machine_to_gray(
    pool: &Pool<Postgres>,
    machine_code: &str,
    gray_release_id: i32,
    effective_time: Option<chrono::DateTime<chrono::Utc>>,
    notes: Option<&str>,
) -> Result<i32, Error> {
    let sql = "
        INSERT INTO machine_gray_allocations (
            machine_code, gray_release_id, allocated_at, effective_time, status, notes, created_at
        ) VALUES (
            $1, $2, NOW(), $3, 'active', $4, NOW()
        )
        ON CONFLICT (machine_code, gray_release_id) 
        DO UPDATE SET
            status = 'active',
            effective_time = COALESCE(EXCLUDED.effective_time, machine_gray_allocations.effective_time),
            notes = COALESCE(EXCLUDED.notes, machine_gray_allocations.notes),
            allocated_at = NOW()
        RETURNING id
    ";

    let result: (i32,) = sqlx::query_as(sql)
        .bind(machine_code)
        .bind(gray_release_id)
        .bind(effective_time)
        .bind(notes)
        .fetch_one(pool)
        .await?;

    Ok(result.0)
}

/// 批量分配机器到灰度发布
pub async fn batch_allocate_machines_to_gray(
    pool: &Pool<Postgres>,
    machine_codes: &[String],
    gray_release_id: i32,
    effective_time: Option<chrono::DateTime<chrono::Utc>>,
    notes: Option<&str>,
) -> Result<usize, Error> {
    if machine_codes.is_empty() {
        return Ok(0);
    }

    let mut tx = pool.begin().await?;
    let mut count = 0;

    for machine_code in machine_codes {
        let sql = "
            INSERT INTO machine_gray_allocations (
                machine_code, gray_release_id, allocated_at, effective_time, status, notes, created_at
            ) VALUES (
                $1, $2, NOW(), $3, 'active', $4, NOW()
            )
            ON CONFLICT (machine_code, gray_release_id) 
            DO UPDATE SET
                status = 'active',
                effective_time = COALESCE(EXCLUDED.effective_time, machine_gray_allocations.effective_time),
                notes = COALESCE(EXCLUDED.notes, machine_gray_allocations.notes),
                allocated_at = NOW()
        ";

        sqlx::query(sql)
            .bind(machine_code)
            .bind(gray_release_id)
            .bind(effective_time)
            .bind(notes)
            .execute(&mut *tx)
            .await?;

        count += 1;
    }

    tx.commit().await?;
    Ok(count)
}

/// 查询分配记录
pub async fn query_allocation_by_id(
    pool: &Pool<Postgres>,
    id: i32,
) -> Result<MachineGrayAllocation, Error> {
    let allocation: MachineGrayAllocation =
        sqlx::query_as("SELECT * FROM machine_gray_allocations WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;

    Ok(allocation)
}

/// 查询机器的所有分配记录
pub async fn query_allocations_by_machine_code(
    pool: &Pool<Postgres>,
    machine_code: &str,
) -> Result<Vec<MachineGrayAllocation>, Error> {
    let allocations: Vec<MachineGrayAllocation> = sqlx::query_as(
        "SELECT * FROM machine_gray_allocations WHERE machine_code = $1 ORDER BY allocated_at DESC",
    )
    .bind(machine_code)
    .fetch_all(pool)
    .await?;

    Ok(allocations)
}

/// 查询灰度发布的所有分配记录
pub async fn query_allocations_by_gray_release_id(
    pool: &Pool<Postgres>,
    gray_release_id: i32,
    page_num: i32,
    page_size: i32,
) -> Result<(i64, Vec<MachineGrayAllocation>), Error> {
    // 获取总数
    let count_sql = "SELECT COUNT(*) FROM machine_gray_allocations WHERE gray_release_id = $1";
    let total: (i64,) = sqlx::query_as(count_sql).bind(gray_release_id).fetch_one(pool).await?;

    // 获取分页列表
    let list_sql = "SELECT * FROM machine_gray_allocations WHERE gray_release_id = $1 ORDER BY allocated_at DESC LIMIT $2 OFFSET $3";
    let allocations: Vec<MachineGrayAllocation> = sqlx::query_as(list_sql)
        .bind(gray_release_id)
        .bind(page_size)
        .bind((page_num - 1) * page_size)
        .fetch_all(pool)
        .await?;

    Ok((total.0, allocations))
}

/// 取消分配（软删除）
pub async fn deallocate_machine(
    pool: &Pool<Postgres>,
    machine_code: &str,
    gray_release_id: i32,
) -> Result<bool, Error> {
    let sql = "
        UPDATE machine_gray_allocations 
        SET status = 'inactive' 
        WHERE machine_code = $1 AND gray_release_id = $2
    ";

    let row = sqlx::query(sql).bind(machine_code).bind(gray_release_id).execute(pool).await?;

    Ok(row.rows_affected() > 0)
}

/// 检查机器是否已分配到灰度发布
pub async fn check_machine_allocated(
    pool: &Pool<Postgres>,
    machine_code: &str,
    gray_release_id: i32,
) -> Result<bool, Error> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM machine_gray_allocations WHERE machine_code = $1 AND gray_release_id = $2 AND status = 'active')",
    )
    .bind(machine_code)
    .bind(gray_release_id)
    .fetch_one(pool)
    .await?;

    Ok(exists)
}
