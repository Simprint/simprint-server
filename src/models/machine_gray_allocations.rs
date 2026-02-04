use crate::dto::machine_gray_allocations::MachineGrayAllocation;
use crate::entitys::machine_gray_allocations::{
    AllocateMachineRequest, BatchAllocateMachineRequest,
};
use sqlx::{Error, Pool, Postgres};

/// 分配机器到灰度
pub async fn allocate_machine(
    pool: &Pool<Postgres>,
    request: &AllocateMachineRequest,
) -> Result<bool, Error> {
    let sql = "
        INSERT INTO machine_gray_allocations (
            machine_code, gray_release_id, allocated_at, 
            effective_time, status, notes, created_at
        ) VALUES (
            $1, $2, NOW(), $3, 'active', $4, NOW()
        )
        ON CONFLICT (machine_code, gray_release_id) DO UPDATE SET
            status = 'active',
            effective_time = COALESCE(EXCLUDED.effective_time, machine_gray_allocations.effective_time),
            notes = COALESCE(EXCLUDED.notes, machine_gray_allocations.notes)
    ";

    let row = sqlx::query(sql)
        .bind(&request.machine_code)
        .bind(request.gray_release_id)
        .bind(request.effective_time)
        .bind(&request.notes)
        .execute(pool)
        .await?;

    Ok(row.rows_affected() > 0)
}

/// 批量分配机器
pub async fn batch_allocate_machines(
    pool: &Pool<Postgres>,
    request: &BatchAllocateMachineRequest,
) -> Result<i64, Error> {
    let mut tx = pool.begin().await?;

    let mut count = 0;
    for machine_code in &request.machine_codes {
        let sql = "
            INSERT INTO machine_gray_allocations (
                machine_code, gray_release_id, allocated_at, 
                effective_time, status, notes, created_at
            ) VALUES (
                $1, $2, NOW(), $3, 'active', $4, NOW()
            )
            ON CONFLICT (machine_code, gray_release_id) DO NOTHING
        ";

        let row = sqlx::query(sql)
            .bind(machine_code)
            .bind(request.gray_release_id)
            .bind(request.effective_time)
            .bind(&request.notes)
            .execute(&mut *tx)
            .await?;

        if row.rows_affected() > 0 {
            count += 1;
        }
    }

    tx.commit().await?;
    Ok(count)
}

/// 按灰度发布 ID 删除该灰度下所有机器分配（用于删除灰度前清理外键）
pub async fn delete_allocations_by_gray_release_id(
    pool: &Pool<Postgres>,
    gray_release_id: i32,
) -> Result<u64, Error> {
    let row = sqlx::query("DELETE FROM machine_gray_allocations WHERE gray_release_id = $1")
        .bind(gray_release_id)
        .execute(pool)
        .await?;
    Ok(row.rows_affected())
}

/// 移除机器分配
pub async fn remove_allocation(
    pool: &Pool<Postgres>,
    machine_code: &str,
    gray_release_id: i32,
) -> Result<bool, Error> {
    let sql = "
        UPDATE machine_gray_allocations 
        SET status = 'released'
        WHERE machine_code = $1 AND gray_release_id = $2
    ";

    let row = sqlx::query(sql)
        .bind(machine_code)
        .bind(gray_release_id)
        .execute(pool)
        .await?;

    Ok(row.rows_affected() == 1)
}

/// 查询机器分配
pub async fn query_allocation(
    pool: &Pool<Postgres>,
    machine_code: &str,
    gray_release_id: i32,
) -> Result<Option<MachineGrayAllocation>, Error> {
    let allocation: Option<MachineGrayAllocation> = sqlx::query_as(
        "SELECT * FROM machine_gray_allocations 
         WHERE machine_code = $1 AND gray_release_id = $2",
    )
    .bind(machine_code)
    .bind(gray_release_id)
    .fetch_optional(pool)
    .await?;

    Ok(allocation)
}

/// 查询机器的所有活跃分配
pub async fn query_machine_allocations(
    pool: &Pool<Postgres>,
    machine_code: &str,
) -> Result<Vec<MachineGrayAllocation>, Error> {
    let allocations: Vec<MachineGrayAllocation> = sqlx::query_as(
        "SELECT * FROM machine_gray_allocations 
         WHERE machine_code = $1 AND status = 'active'
         ORDER BY allocated_at DESC",
    )
    .bind(machine_code)
    .fetch_all(pool)
    .await?;

    Ok(allocations)
}

/// 查询灰度发布的所有分配
pub async fn query_release_allocations(
    pool: &Pool<Postgres>,
    gray_release_id: i32,
    page_num: i32,
    page_size: i32,
) -> Result<(i64, Vec<MachineGrayAllocation>), Error> {
    // 获取总数
    let total: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM machine_gray_allocations WHERE gray_release_id = $1")
            .bind(gray_release_id)
            .fetch_one(pool)
            .await?;

    // 获取分页列表
    let allocations: Vec<MachineGrayAllocation> = sqlx::query_as(
        "SELECT * FROM machine_gray_allocations 
         WHERE gray_release_id = $1 
         ORDER BY allocated_at DESC 
         LIMIT $2 OFFSET $3",
    )
    .bind(gray_release_id)
    .bind(page_size)
    .bind((page_num - 1) * page_size)
    .fetch_all(pool)
    .await?;

    Ok((total.0, allocations))
}

/// 检查机器是否在灰度中
pub async fn is_machine_in_gray(
    pool: &Pool<Postgres>,
    machine_code: &str,
    gray_release_id: i32,
) -> Result<bool, Error> {
    let result: Option<(i32,)> = sqlx::query_as(
        "SELECT 1 FROM machine_gray_allocations 
         WHERE machine_code = $1 AND gray_release_id = $2 AND status = 'active'
         LIMIT 1",
    )
    .bind(machine_code)
    .bind(gray_release_id)
    .fetch_optional(pool)
    .await?;

    Ok(result.is_some())
}
