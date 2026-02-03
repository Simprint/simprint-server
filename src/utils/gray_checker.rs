use crate::dto::gray_releases::GrayRelease;
use crate::errors::SimprintError;

/// 根据策略类型检查机器是否匹配
/// 注意：filter_whitelist 策略已在数据库层实现，此函数仅处理 filter_percentage 和 filter_random
pub fn check_machine_by_strategy(
    machine_code: &str,
    _user_uuid: Option<&uuid::Uuid>,
    release: &GrayRelease,
) -> Result<bool, SimprintError> {
    // 如果没有配置策略，默认返回 false
    let strategy_type = match &release.strategy_type {
        Some(st) => st,
        None => return Ok(false),
    };

    // 如果没有配置参数，返回 false
    let config = match &release.strategy_config {
        Some(cfg) => cfg,
        None => return Ok(false),
    };

    match strategy_type.as_str() {
        "filter_percentage" => {
            let percent: f64 = config
                .get("percent")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| SimprintError::InvalidStrategyConfig)?;
            is_in_percentage_range(machine_code, release.id, percent)
        }
        "filter_random" => {
            let seed = config.get("seed").and_then(|v| v.as_u64()).unwrap_or(12345);
            is_in_random_selection(machine_code, release.id, seed)
        }
        "filter_user_tag" | "filter_whitelist" => {
            // 这些策略已在数据库查询阶段处理，不应该到达这里
            Ok(false)
        }
        _ => Err(SimprintError::InvalidStrategyConfig),
    }
}

/// 检查是否在百分比范围内
fn is_in_percentage_range(
    machine_code: &str,
    release_id: i32,
    percent: f64,
) -> Result<bool, SimprintError> {
    if percent < 0.0 || percent > 100.0 {
        return Err(SimprintError::InvalidStrategyConfig);
    }

    // 使用机器码和灰度发布ID的哈希值计算
    let combined = format!("{}:{}", machine_code, release_id);
    let hash = calculate_hash(&combined);

    // 将哈希值映射到 0-100 范围
    let value = (hash % 10000) as f64 / 100.0;

    Ok(value < percent)
}

/// 检查是否在随机选择中
fn is_in_random_selection(
    machine_code: &str,
    release_id: i32,
    seed: u64,
) -> Result<bool, SimprintError> {
    // 使用机器码、灰度发布ID和种子值的哈希值计算
    let combined = format!("{}:{}:{}", machine_code, release_id, seed);
    let hash = calculate_hash(&combined);

    // 将哈希值映射到 0-1 范围
    let value = (hash % 10000) as f64 / 10000.0;

    // 默认概率为 0.5（50%）
    Ok(value < 0.5)
}

/// 计算字符串的哈希值
fn calculate_hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}
