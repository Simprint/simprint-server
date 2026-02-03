use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 版本类型 DTO
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct VersionType {
    pub id: i32,
    pub type_code: String,
    pub type_name: String,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: bool,
}
