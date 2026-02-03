use serde::{Deserialize, Serialize};

/// 获取策略类型参数
#[derive(Debug, Deserialize, Serialize)]
pub struct GetStrategyTypeParams {
    pub id: Option<i32>,
    pub code: Option<String>,
}

/// 查询策略类型列表参数
#[derive(Debug, Deserialize, Serialize)]
pub struct QueryStrategyTypesParams {
    pub category: Option<String>,
}

/// 策略类型列表响应
#[derive(Debug, Deserialize, Serialize)]
pub struct StrategyTypeListResponse {
    pub list: Vec<crate::dto::strategy_types::StrategyType>,
}
