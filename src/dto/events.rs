use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 工作空间事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceEvent {
    /// 事件 ID（Redis Stream 自动生成）
    pub event_id: String,

    /// 工作空间 ID
    pub workspace_uuid: Uuid,

    /// 事件类型
    pub event_type: EventType,

    /// 实体类型
    pub entity_type: EntityType,

    /// 实体 ID（可选，如批量删除时为空）
    pub entity_id: Option<Uuid>,

    /// 受影响的 API 列表
    pub affected_apis: Vec<String>,

    /// 事件负载（可选，用于携带额外信息）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,

    /// 创建时间戳（毫秒）
    pub created_at: i64,
}

/// 事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// 创建
    Created,
    /// 更新
    Updated,
    /// 删除
    Deleted,
    /// 批量删除
    BatchDeleted,
    /// 移动
    Moved,
    /// 分配标签
    TagsAssigned,
    /// 移除标签
    TagsRemoved,
}

/// 实体类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// 环境
    Environment,
    /// 分组
    Group,
    /// 标签
    Tag,
    /// 代理
    Proxy,
    /// 账号
    Account,
}

impl WorkspaceEvent {
    /// 创建新事件
    pub fn new(
        workspace_uuid: Uuid,
        event_type: EventType,
        entity_type: EntityType,
        entity_id: Option<Uuid>,
        affected_apis: Vec<String>,
    ) -> Self {
        Self {
            event_id: String::new(), // Redis Stream 会自动生成
            workspace_uuid,
            event_type,
            entity_type,
            entity_id,
            affected_apis,
            payload: None,
            created_at: chrono::Utc::now().timestamp_millis(),
        }
    }

    /// 设置负载
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }
}
