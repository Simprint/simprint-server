mod admin_permission;
mod auth;
mod cors;
mod decrypt;
mod dual_auth;
mod encrypt;
mod logger;
mod machine_auth;
mod real_ip;

pub use admin_permission::*;
pub use auth::*;
pub use cors::*;
pub use decrypt::*;
pub use dual_auth::*;
pub use encrypt::*;
pub use logger::*;
pub use machine_auth::*;
pub use real_ip::*;

use axum::extract::{MatchedPath, Request};

/// 获取请求路径和请求方法
pub fn get_request_path_and_method(req: &Request) -> (&str, &str) {
    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| req.uri().path(), |path| path.as_str());

    let method = req.method().as_str();

    (method, path)
}
