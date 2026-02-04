use crate::dto::machine_tags::MachineTag;
use crate::entitys::machine_tags::{CreateTagRequest, QueryTagsParams};
use sqlx::{Error, Pool, Postgres};

/// 插入或获取标签（按名称去重）
pub async fn insert_or_get_tag(
    pool: &Pool<Postgres>,
    request: &CreateTagRequest,
) -> Result<i32, Error> {
    // 先尝试查询是否存在
    let existing_tag =
        sqlx::query_as::<_, (i32,)>("SELECT id FROM machine_tags WHERE tag_name = $1")
            .bind(&request.tag_name)
            .fetch_optional(pool)
            .await?;

    if let Some((id,)) = existing_tag {
        return Ok(id);
    }

    // 不存在则插入
    let result: (i32,) = sqlx::query_as(
        "
        INSERT INTO machine_tags (tag_name, description, category, is_active, created_at)
        VALUES ($1, $2, $3, true, NOW())
        RETURNING id
        ",
    )
    .bind(&request.tag_name)
    .bind(&request.description)
    .bind(&request.category)
    .fetch_one(pool)
    .await?;

    Ok(result.0)
}

/// 批量插入或获取标签（按名称列表批量去重）
pub async fn batch_insert_or_get_tags(
    pool: &Pool<Postgres>,
    tag_names: &[String],
) -> Result<Vec<i32>, Error> {
    if tag_names.is_empty() {
        return Ok(vec![]);
    }

    let mut tag_ids = Vec::new();

    // 查询已存在的标签
    let existing_tags = sqlx::query_as::<_, (String, i32)>(
        "
        SELECT tag_name, id FROM machine_tags 
        WHERE tag_name = ANY($1)
        ",
    )
    .bind(tag_names)
    .fetch_all(pool)
    .await?;

    // 建立已存在标签的映射
    let mut existing_map = std::collections::HashMap::new();
    for (name, id) in existing_tags {
        existing_map.insert(name, id);
        tag_ids.push(id);
    }

    // 找出需要插入的标签
    let tags_to_insert: Vec<String> = tag_names
        .iter()
        .filter(|name| !existing_map.contains_key(*name))
        .cloned()
        .collect();

    // 批量插入新标签
    if !tags_to_insert.is_empty() {
        for tag_name in tags_to_insert {
            let new_id = sqlx::query_as::<_, (i32,)>(
                "
                INSERT INTO machine_tags (tag_name, is_active, created_at)
                VALUES ($1, true, NOW())
                RETURNING id
                ",
            )
            .bind(&tag_name)
            .fetch_one(pool)
            .await?;

            tag_ids.push(new_id.0);
        }
    }

    Ok(tag_ids)
}

/// 查询所有去重标签
pub async fn query_all_distinct_tags(
    pool: &Pool<Postgres>,
    params: &QueryTagsParams,
) -> Result<Vec<MachineTag>, Error> {
    let mut query = "
        SELECT * FROM machine_tags
        WHERE 1=1
    "
    .to_string();

    let _ = params; // 暂时忽略参数，直接查询所有激活标签

    if let Some(_category) = &params.category {
        query.push_str(" AND category = $1");
    }

    if let Some(_is_active) = params.is_active {
        query.push_str(" AND is_active = $2");
    }

    query.push_str(" ORDER BY created_at DESC");

    let result = if params.category.is_some() || params.is_active.is_some() {
        sqlx::query_as::<_, MachineTag>(&query)
            .bind(&params.category)
            .bind(&params.is_active)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query_as::<_, MachineTag>(&query)
            .fetch_all(pool)
            .await?
    };

    Ok(result)
}

/// 根据机器用户ID查询标签
pub async fn query_tags_by_machine_user_id(
    pool: &Pool<Postgres>,
    machine_user_id: i32,
) -> Result<Vec<MachineTag>, Error> {
    let tags = sqlx::query_as::<_, MachineTag>(
        "
        SELECT mt.* FROM machine_tags mt
        INNER JOIN machine_user_tags mut ON mt.id = mut.tag_id
        WHERE mut.machine_user_id = $1
        ORDER BY mt.tag_name
        ",
    )
    .bind(machine_user_id)
    .fetch_all(pool)
    .await?;

    Ok(tags)
}

/// 绑定标签到机器用户
pub async fn bind_tags_to_machine_user(
    pool: &Pool<Postgres>,
    machine_user_id: i32,
    tag_ids: &[i32],
) -> Result<usize, Error> {
    if tag_ids.is_empty() {
        return Ok(0);
    }

    let mut count = 0;

    for &tag_id in tag_ids {
        // 使用 ON CONFLICT DO NOTHING 避免重复插入
        let result = sqlx::query(
            "
            INSERT INTO machine_user_tags (machine_user_id, tag_id, created_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (machine_user_id, tag_id) DO NOTHING
            ",
        )
        .bind(machine_user_id)
        .bind(tag_id)
        .execute(pool)
        .await?;

        count += result.rows_affected() as usize;
    }

    Ok(count)
}

/// 解绑标签
pub async fn unbind_tags_from_machine_user(
    pool: &Pool<Postgres>,
    machine_user_id: i32,
    tag_ids: &[i32],
) -> Result<usize, Error> {
    if tag_ids.is_empty() {
        return Ok(0);
    }

    let result = sqlx::query(
        "
        DELETE FROM machine_user_tags
        WHERE machine_user_id = $1 AND tag_id = ANY($2)
        ",
    )
    .bind(machine_user_id)
    .bind(tag_ids)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() as usize)
}

/// 根据标签查询机器用户ID
pub async fn query_machine_user_ids_by_tag_names(
    pool: &Pool<Postgres>,
    tag_names: &[String],
) -> Result<Vec<i32>, Error> {
    if tag_names.is_empty() {
        return Ok(vec![]);
    }

    let machine_user_ids = sqlx::query_as::<_, (i32,)>(
        "
        SELECT DISTINCT mut.machine_user_id FROM machine_user_tags mut
        INNER JOIN machine_tags mt ON mt.id = mut.tag_id
        WHERE mt.tag_name = ANY($1)
        ",
    )
    .bind(tag_names)
    .fetch_all(pool)
    .await?;

    Ok(machine_user_ids.into_iter().map(|(id,)| id).collect())
}
