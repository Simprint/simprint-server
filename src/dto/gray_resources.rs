use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 灰度资源 DTO
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct GrayResource {
    pub id: i32,
    pub gray_release_id: i32,
    pub version_id: i32,
    pub sort_order: i32,
}
