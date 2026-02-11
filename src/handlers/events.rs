use axum::{extract::State, Extension};
use serde::{Deserialize, Serialize};

use crate::dto::events::WorkspaceEvent;
use crate::services::events::EventService;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 同步事件请求
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncEventsRequest {
    /// 上次同步的事件 ID（可选）
    pub last_event_id: Option<String>,
}

/// 同步事件响应
#[derive(Debug, Serialize)]
pub struct SyncEventsResponse {
    /// 事件列表
    pub events: Vec<WorkspaceEvent>,

    /// 是否需要全量刷新
    pub full_refresh: bool,

    /// 最新事件 ID
    pub latest_event_id: Option<String>,
}

/// 同步事件
///
/// POST /api/v1/events/sync
#[axum::debug_handler]
pub async fn sync_events_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(req): Json<SyncEventsRequest>,
) -> Result<SyncEventsResponse> {
    let workspace_uuid = ctx
        .current_workspace_uuid
        .ok_or_else(|| Response::fail(Some("未选择工作空间")))?;

    // 获取事件
    let events = EventService::get_events_since(&svc_ctx, workspace_uuid, req.last_event_id.clone())
        .await
        .map_err(|e| {
            tracing::error!("获取事件失败: {}", e);
            Response::fail(Some("获取事件失败"))
        })?;

    // 判断是否需要全量刷新
    let full_refresh = if let Some(last_id) = &req.last_event_id {
        // 如果提供了 last_event_id 但没有获取到任何事件
        // 可能是因为事件已过期（> 72 小时）
        if !last_id.is_empty() && events.is_empty() {
            // 检查事件是否真的过期了
            match EventService::event_exists(&svc_ctx, workspace_uuid, last_id).await {
                Ok(exists) => !exists,
                Err(e) => {
                    tracing::error!("检查事件是否存在失败: {}", e);
                    false
                }
            }
        } else {
            false
        }
    } else {
        false
    };

    let latest_event_id = events.last().map(|e| e.event_id.clone());

    tracing::debug!(
        "同步事件: workspace={}, last_id={:?}, count={}, full_refresh={}",
        workspace_uuid,
        req.last_event_id,
        events.len(),
        full_refresh
    );

    Ok(Response::success(
        Some("同步成功"),
        Some(SyncEventsResponse {
            events,
            full_refresh,
            latest_event_id,
        }),
    ))
}
