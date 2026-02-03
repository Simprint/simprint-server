use axum::routing::post;

use crate::handlers::{
    batch_delete_rpa_tasks_handler, create_rpa_task_handler, delete_rpa_task_handler,
    duplicate_rpa_task_handler, export_rpa_task_handler, get_rpa_run_handler, get_rpa_runs_handler,
    get_rpa_task_handler, get_rpa_tasks_handler, run_rpa_task_handler, stop_rpa_task_handler,
    update_rpa_task_handler,
};
use crate::routes::route::{self, MetaRoute};

/// 注册 RPA 工作流相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut rpa_route = route::RouteGroup::new("/rpa");

    // 任务管理
    rpa_route.add_route_item(route::RouteItem::post(
        "/tasks",
        post(get_rpa_tasks_handler),
    ));
    rpa_route.add_route_item(route::RouteItem::post(
        "/tasks/detail",
        post(get_rpa_task_handler),
    ));
    rpa_route.add_route_item(route::RouteItem::post(
        "/tasks/create",
        post(create_rpa_task_handler),
    ));
    rpa_route.add_route_item(route::RouteItem::post(
        "/tasks/update",
        post(update_rpa_task_handler),
    ));
    rpa_route.add_route_item(route::RouteItem::post(
        "/tasks/delete",
        post(delete_rpa_task_handler),
    ));
    rpa_route.add_route_item(route::RouteItem::post(
        "/tasks/batch-delete",
        post(batch_delete_rpa_tasks_handler),
    ));

    // 任务操作
    rpa_route.add_route_item(route::RouteItem::post(
        "/tasks/run",
        post(run_rpa_task_handler),
    ));
    rpa_route.add_route_item(route::RouteItem::post(
        "/tasks/stop",
        post(stop_rpa_task_handler),
    ));
    rpa_route.add_route_item(route::RouteItem::post(
        "/tasks/duplicate",
        post(duplicate_rpa_task_handler),
    ));
    rpa_route.add_route_item(route::RouteItem::post(
        "/tasks/export",
        post(export_rpa_task_handler),
    ));

    // 执行记录
    rpa_route.add_route_item(route::RouteItem::post("/runs", post(get_rpa_runs_handler)));
    rpa_route.add_route_item(route::RouteItem::post(
        "/runs/detail",
        post(get_rpa_run_handler),
    ));

    meta_route.add_route_group(rpa_route);
}
