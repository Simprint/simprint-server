use chrono::Utc;

use crate::entitys::maintenance::HealthResponse;
use crate::utils::{Response, Result};

/// 健康检查
pub async fn health_check_handler() -> Result<HealthResponse> {
    Ok(Response::success(
        Some("服务正常"),
        Some(HealthResponse {
            status: "ok".to_string(),
            timestamp: Utc::now().to_rfc3339(),
        }),
    ))
}
