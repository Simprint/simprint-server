use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 创建 API 密钥请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub permissions: Vec<String>,
    pub rate_limit: Option<i32>,
    pub daily_limit: Option<i32>,
    pub ip_whitelist: Option<Vec<String>>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// 重置 API 密钥请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResetApiKeyRequest {
    pub uuid: Uuid,
}

/// 删除 API 密钥请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeleteApiKeyRequest {
    pub uuid: Uuid,
}

/// API 使用统计请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiStatsRequest {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

use super::Pagination;

/// 查询 API 密钥列表请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListApiKeysRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub status: Option<String>,
}

/// 更新 API 密钥请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateApiKeyRequest {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub rate_limit: Option<i32>,
    pub daily_limit: Option<i32>,
    pub ip_whitelist: Option<Vec<String>>,
}

/// 撤销 API 密钥请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RevokeApiKeyRequest {
    pub uuid: Uuid,
}

// ========== 响应结构体 ==========

/// API 密钥列表响应
#[derive(Debug, Clone, Serialize)]
pub struct ApiKeysListResponse {
    pub items: Vec<ApiKeyItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// API 密钥项（不包含敏感信息）
#[derive(Debug, Clone, Serialize)]
pub struct ApiKeyItem {
    pub uuid: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub permissions: serde_json::Value,
    pub rate_limit: Option<i32>,
    pub daily_limit: Option<i32>,
    pub ip_whitelist: Option<serde_json::Value>,
    pub expires_at: Option<DateTime<Utc>>,
    pub usage_count: Option<i64>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// 创建 API 密钥响应
#[derive(Debug, Clone, Serialize)]
pub struct CreateApiKeyResponse {
    pub uuid: Uuid,
    pub api_key: String,
    pub key_prefix: String,
}

impl From<crate::dto::ApiKeyDto> for ApiKeyItem {
    fn from(dto: crate::dto::ApiKeyDto) -> Self {
        Self {
            uuid: dto.uuid,
            name: dto.name,
            key_prefix: dto.key_prefix,
            permissions: dto.permissions,
            rate_limit: dto.rate_limit,
            daily_limit: dto.daily_limit,
            ip_whitelist: dto.ip_whitelist,
            expires_at: dto.expires_at,
            usage_count: dto.usage_count,
            last_used_at: dto.last_used_at,
            status: dto.status,
            created_at: dto.created_at,
        }
    }
}
