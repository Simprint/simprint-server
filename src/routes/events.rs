use axum::routing::post;

use crate::handlers::events;
use crate::routes::route::{self, MetaRoute, RequestMethod};

/// 注册事件相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut events_route = route::RouteGroup::new("/events");

    // POST /events/sync - 同步事件
    events_route.add_route_item(route::RouteItem::new(
        "/sync",
        RequestMethod::POST,
        post(events::sync_events_handler),
    ));

    meta_route.add_route_group(events_route);
}
