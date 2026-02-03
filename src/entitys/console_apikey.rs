//! Console Gateway API 密钥相关请求/响应实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 创建 API 密钥请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateConsoleApiKeyRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}
