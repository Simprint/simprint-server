use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Pagination;

/// 创建扩展参数（模型层入参）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExtensionParams {
    pub extension_id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub category: String,
    pub browser: String,
    pub developer: Option<String>,
    pub homepage: Option<String>,
    pub icon_url: Option<String>,
    pub download_url: Option<String>,
    pub file_size: Option<i64>,
    pub downloads_count: Option<i64>,
    pub permissions: Option<serde_json::Value>,
    pub rating: Option<Decimal>,
    pub changelog: Option<serde_json::Value>,
    pub published_at: Option<DateTime<Utc>>,
}

/// 更新扩展参数（模型层入参）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateExtensionParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub category: Option<String>,
    pub developer: Option<String>,
    pub homepage: Option<String>,
    pub icon_url: Option<String>,
    pub download_url: Option<String>,
    pub file_size: Option<i64>,
    pub downloads_count: Option<i64>,
    pub permissions: Option<serde_json::Value>,
    pub rating: Option<Decimal>,
    pub changelog: Option<serde_json::Value>,
    pub published_at: Option<DateTime<Utc>>,
}

/// 查询扩展列表请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListExtensionsRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub filters: Option<ExtensionFilters>,
}

/// 同步扩展响应
#[derive(Debug, Clone, Serialize)]
pub struct SyncExtensionResponse {
    /// 扩展 UUID
    pub uuid: Uuid,
    /// 扩展 ID
    pub extension_id: String,
    /// 是否为新创建
    pub created: bool,
}

/// 扩展筛选条件
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExtensionFilters {
    pub keyword: Option<String>,
    pub category: Option<String>,
    pub installed_only: Option<bool>,
}

/// 安装扩展请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstallExtensionRequest {
    pub extension_id: String,
    /// 安装目标: user, team, group, environment
    pub target_type: Option<String>,
    /// 分组 UUID 数组（用于安装到分组，即使只有一个分组也需要传入数组）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_ids: Option<Vec<Uuid>>,
    /// 环境 UUID（用于安装到环境）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_uuid: Option<Uuid>,
}

/// 卸载扩展请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UninstallExtensionRequest {
    pub extension_id: String,
    /// 卸载类型：user、team、environment
    /// 如果不指定，默认为 user
    /// 注意：无论 target_type 是什么，后端都会自动删除所有相关的分组记录
    pub target_type: Option<String>,
    /// 目标 UUID（用于 team 和 environment 类型）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_uuid: Option<Uuid>,
}

/// 更新扩展请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateExtensionRequest {
    pub extension_id: String,
}

/// 批量更新扩展请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchUpdateExtensionsRequest {
    pub extension_ids: Vec<String>,
}

/// 扩展 ID 请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExtensionIdRequest {
    pub extension_id: String,
}

// ========== 响应结构体 ==========

use crate::dto::ExtensionDto;

/// 扩展列表响应
#[derive(Debug, Clone, Serialize)]
pub struct ExtensionsListResponse {
    pub items: Vec<ExtensionDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 获取已安装扩展请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetInstalledExtensionsRequest {
    /// 范围过滤：all, user, team
    #[serde(default = "default_scope")]
    pub scope: String,
}

fn default_scope() -> String {
    "all".to_string()
}

/// 已安装扩展响应
#[derive(Debug, Clone, Serialize)]
pub struct InstalledExtensionsResponse {
    pub user_extensions: Vec<InstalledExtensionItem>,
    pub team_extensions: Vec<InstalledExtensionItem>,
}

/// 已安装扩展项（包含完整扩展详情）
#[derive(Debug, Clone, Serialize)]
pub struct InstalledExtensionItem {
    pub extension_id: String,
    pub name: String,
    pub version: String,
    pub installed_version: String,
    pub has_update: bool,
    pub status: String,
    pub installed_at: chrono::DateTime<chrono::Utc>,
    /// 扩展主页 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    /// 扩展图标 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// 团队 UUID（如果是团队安装的）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_uuid: Option<Uuid>,
    /// 安装范围：user 或 team
    pub scope: String,
    /// 扩展描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 扩展分类
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// 浏览器类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser: Option<String>,
    /// 开发者/作者
    #[serde(skip_serializing_if = "Option::is_none")]
    pub developer: Option<String>,
    /// 下载量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads_count: Option<i64>,
    /// 评分
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<rust_decimal::Decimal>,
    /// 权限列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<serde_json::Value>,
    /// 文件大小
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
    /// 更新时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 关联的分组列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<ExtensionGroup>>,
}

/// 扩展关联的分组信息
#[derive(Debug, Clone, Serialize)]
pub struct ExtensionGroup {
    pub uuid: Uuid,
    pub name: String,
}

/// 批量更新响应
#[derive(Debug, Clone, Serialize)]
pub struct BatchUpdateResponse {
    pub updated: u64,
}
