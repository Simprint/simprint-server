use axum::routing::post;

use crate::handlers::{
    create_api_key_handler, get_api_keys_handler, revoke_api_key_handler, update_api_key_handler,
};
use crate::routes::route::{self, MetaRoute};

/// 注册 API 服务相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut api_route = route::RouteGroup::new("/api-keys");

    api_route.add_route_item(route::RouteItem::post("/list", post(get_api_keys_handler)));
    api_route.add_route_item(route::RouteItem::post(
        "/create",
        post(create_api_key_handler),
    ));
    api_route.add_route_item(route::RouteItem::post(
        "/update",
        post(update_api_key_handler),
    ));
    api_route.add_route_item(route::RouteItem::post(
        "/revoke",
        post(revoke_api_key_handler),
    ));

    meta_route.add_route_group(api_route);
}
