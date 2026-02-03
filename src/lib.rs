use crate::utils::IConfig;

pub mod caches;
pub mod dto;
pub mod entitys;
pub mod errors;
pub mod handlers;
pub mod middlewares;
pub mod models;
pub mod routes;
pub mod services;
pub mod state;
pub mod svc_ctx;
pub mod utils;

/// 初始化加密密钥
pub async fn init_encrypt_secret(config: &IConfig) {
    let key_path = &config.app.encrypt_secret_location;
    utils::init_rsa_secret(key_path).await;
}

/// 初始化minio
pub async fn init_minio(config: &IConfig) {
    let minio_config = &config.clone().minio.expect("minio配置不存在");

    let client = {
        utils::init_minio(
            &minio_config.server_url,
            &minio_config.access_key,
            &minio_config.secret_access_key,
            None,
            None,
        )
        .await
        .expect("初始化Minio客户端失败")
    };

    // 初始化存储桶
    client
        .init_bucket(vec![
            minio_config.avatar_bucket.clone(),
            minio_config.extension_bucket.clone(),
        ])
        .await
        .expect("初始化Minio桶失败");
}
