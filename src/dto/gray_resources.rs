use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 灰度资源关联
#[derive(Debug, Deserialize, Serialize, FromRow, Clone, Default)]
pub struct GrayResource {
    pub id: i32,
    pub gray_release_id: i32,
    pub version_id: i32,
    pub sort_order: i32,
}
