use redis::AsyncCommands;
use uuid::Uuid;

use crate::dto::events::{EntityType, EventType, WorkspaceEvent};
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;

/// 事件服务
pub struct EventService;

impl EventService {
    /// 发布事件到 Redis Stream
    ///
    /// # Arguments
    /// * `svc_ctx` - 服务上下文
    /// * `event` - 工作空间事件
    ///
    /// # Returns
    /// * `Ok(String)` - 事件 ID
    /// * `Err` - 发布失败
    pub async fn publish_event(
        svc_ctx: &SvcCtx,
        event: WorkspaceEvent,
    ) -> anyhow::Result<String> {
        let stream_key = format!("events:workspace:{}", event.workspace_uuid);

        // 序列化事件
        let event_json = serde_json::to_string(&event)?;

        // 写入 Redis Stream
        let mut conn = svc_ctx.redis.clone();
        let event_id: String = conn
            .xadd_maxlen(
                &stream_key,
                redis::streams::StreamMaxlen::Approx(15000),
                "*", // 自动生成 ID
                &[("data", event_json)],
            )
            .await?;

        // 设置 TTL（72 小时 = 259200 秒）
        let _: () = conn.expire(&stream_key, 259200).await?;

        tracing::debug!(
            "事件已发布: stream={}, event_id={}, type={:?}, entity={:?}",
            stream_key,
            event_id,
            event.event_type,
            event.entity_type
        );

        Ok(event_id)
    }

    /// 获取指定时间后的事件
    ///
    /// # Arguments
    /// * `svc_ctx` - 服务上下文
    /// * `workspace_uuid` - 工作空间 ID
    /// * `since_id` - 起始事件 ID（可选，默认从头开始）
    ///
    /// # Returns
    /// * `Ok(Vec<WorkspaceEvent>)` - 事件列表
    /// * `Err` - 获取失败
    pub async fn get_events_since(
        svc_ctx: &SvcCtx,
        workspace_uuid: Uuid,
        since_id: Option<String>,
    ) -> anyhow::Result<Vec<WorkspaceEvent>> {
        let stream_key = format!("events:workspace:{}", workspace_uuid);
        let start_id = since_id.unwrap_or_else(|| "0".to_string());

        let mut conn = svc_ctx.redis.clone();

        // 读取事件
        let results: redis::streams::StreamReadReply = conn
            .xread(&[&stream_key], &[&start_id])
            .await?;

        let mut events = Vec::new();
        for stream_key_result in results.keys {
            for stream_id in stream_key_result.ids {
                if let Some(data) = stream_id.map.get("data") {
                    // redis 0.31.0 使用 BulkString 而不是 Data
                    match data {
                        redis::Value::BulkString(bytes) => {
                            match serde_json::from_slice::<WorkspaceEvent>(bytes) {
                                Ok(mut event) => {
                                    // 设置 Redis Stream 生成的 event_id
                                    event.event_id = stream_id.id.clone();
                                    events.push(event);
                                }
                                Err(e) => {
                                    tracing::error!("反序列化事件失败: {}", e);
                                }
                            }
                        }
                        _ => {
                            tracing::warn!("意外的 Redis Value 类型: {:?}", data);
                        }
                    }
                }
            }
        }

        tracing::debug!(
            "获取事件: workspace={}, since={}, count={}",
            workspace_uuid,
            start_id,
            events.len()
        );

        Ok(events)
    }

    /// 检查事件 ID 是否仍然存在于 Stream 中
    ///
    /// # Arguments
    /// * `svc_ctx` - 服务上下文
    /// * `workspace_uuid` - 工作空间 ID
    /// * `event_id` - 事件 ID
    ///
    /// # Returns
    /// * `Ok(bool)` - true 表示存在，false 表示已过期
    pub async fn event_exists(
        svc_ctx: &SvcCtx,
        workspace_uuid: Uuid,
        event_id: &str,
    ) -> anyhow::Result<bool> {
        let stream_key = format!("events:workspace:{}", workspace_uuid);

        let mut conn = svc_ctx.redis.clone();

        // 尝试读取指定 ID 的事件
        let results: redis::streams::StreamReadReply = conn
            .xread_options(
                &[&stream_key],
                &[event_id],
                &redis::streams::StreamReadOptions::default().count(1),
            )
            .await?;

        Ok(!results.keys.is_empty())
    }

    /// 便捷方法：从 RequestContext 发布事件
    ///
    /// # Arguments
    /// * `svc_ctx` - 服务上下文
    /// * `ctx` - 请求上下文（包含 workspace_uuid）
    /// * `event_type` - 事件类型
    /// * `entity_type` - 实体类型
    /// * `entity_id` - 实体 ID（可选）
    /// * `affected_apis` - 受影响的查询接口列表
    ///
    /// # Returns
    /// * `Ok(())` - 发布成功
    /// * `Err` - 发布失败
    pub async fn publish_from_context(
        svc_ctx: &SvcCtx,
        ctx: &RequestContext,
        event_type: EventType,
        entity_type: EntityType,
        entity_id: Option<Uuid>,
        affected_apis: Vec<String>,
    ) -> anyhow::Result<()> {
        let workspace_uuid = ctx.workspace_uuid().ok_or_else(|| {
            anyhow::anyhow!("工作空间未设置")
        })?;

        let event = WorkspaceEvent::new(
            workspace_uuid,
            event_type,
            entity_type,
            entity_id,
            affected_apis,
        );

        Self::publish_event(svc_ctx, event).await?;
        Ok(())
    }
}
