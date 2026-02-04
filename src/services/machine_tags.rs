use crate::{
    dto::machine_tags::MachineTag, entitys::machine_tags::QueryTagsParams,
    errors::SimprintError, svc_ctx::SvcCtx,
};

/// 查询所有标签（去重）
pub async fn query_all_tags_service(
    svc_ctx: &SvcCtx,
) -> Result<Vec<MachineTag>, SimprintError> {
    let params = QueryTagsParams {
        category: None,
        is_active: Some(true),
    };

    let tags = crate::models::machine_tags::query_all_distinct_tags(&svc_ctx.db, &params).await?;

    Ok(tags)
}

/// 根据机器用户ID查询标签
pub async fn query_tags_by_machine_user_id_service(
    svc_ctx: &SvcCtx,
    machine_user_id: i32,
) -> Result<Vec<MachineTag>, SimprintError> {
    let tags =
        crate::models::machine_tags::query_tags_by_machine_user_id(&svc_ctx.db, machine_user_id)
            .await?;

    Ok(tags)
}

/// 绑定标签到机器用户（内部使用，处理标签字符串）
pub async fn bind_tags_to_machine_user_service(
    svc_ctx: &SvcCtx,
    machine_user_id: i32,
    tag_names: &[String],
) -> Result<usize, SimprintError> {
    if tag_names.is_empty() {
        return Ok(0);
    }

    // 批量插入或获取标签
    let tag_ids =
        crate::models::machine_tags::batch_insert_or_get_tags(&svc_ctx.db, tag_names).await?;

    // 绑定标签到机器用户
    let count = crate::models::machine_tags::bind_tags_to_machine_user(
        &svc_ctx.db,
        machine_user_id,
        &tag_ids,
    )
    .await?;

    Ok(count)
}
