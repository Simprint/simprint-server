use crate::{
    dto::strategy_types::StrategyType, entitys::strategy_types::*, errors::SimprintError,
    svc_ctx::SvcCtx,
};

/// 根据ID查询策略类型
pub async fn get_strategy_type_by_id_service(
    svc_ctx: &SvcCtx,
    payload: GetStrategyTypeParams,
) -> Result<StrategyType, SimprintError> {
    let pool = &svc_ctx.db;
    let id = payload.id.ok_or(SimprintError::StrategyTypeNotFound)?;
    let strategy_type = crate::models::strategy_types::query_strategy_type_by_id(pool, id)
        .await
        .map_err(|_| SimprintError::StrategyTypeNotFound)?;

    Ok(strategy_type)
}

/// 根据code查询策略类型
pub async fn get_strategy_type_by_code_service(
    svc_ctx: &SvcCtx,
    payload: GetStrategyTypeParams,
) -> Result<StrategyType, SimprintError> {
    let pool = &svc_ctx.db;
    let code = payload.code.ok_or(SimprintError::StrategyTypeNotFound)?;
    let strategy_type = crate::models::strategy_types::query_strategy_type_by_code(pool, &code)
        .await
        .map_err(|_| SimprintError::StrategyTypeNotFound)?;

    Ok(strategy_type)
}

/// 查询所有可用策略类型
pub async fn query_available_strategy_types_service(
    svc_ctx: &SvcCtx,
) -> Result<StrategyTypeListResponse, SimprintError> {
    let pool = &svc_ctx.db;
    let list = crate::models::strategy_types::query_available_strategy_types(pool).await?;

    Ok(StrategyTypeListResponse { list })
}

/// 根据分类查询策略类型
pub async fn query_strategy_types_by_category_service(
    svc_ctx: &SvcCtx,
    payload: QueryStrategyTypesParams,
) -> Result<StrategyTypeListResponse, SimprintError> {
    let pool = &svc_ctx.db;
    let category = payload.category.ok_or(SimprintError::InvalidRequest(
        "分类参数不能为空".to_string(),
    ))?;
    let list =
        crate::models::strategy_types::query_strategy_types_by_category(pool, &category).await?;

    Ok(StrategyTypeListResponse { list })
}
