use std::{path::Path, sync::Arc};

pub mod get_objects;
pub mod put_objects;
pub mod remove_objects;

use minio::s3::types::S3Api;
use minio::s3::{client::ClientBuilder, creds::StaticProvider, http::BaseUrl};
use tokio::sync::OnceCell;

/// Minio client for interacting with MinIO server
pub struct Minio {
    pub client: minio::s3::client::Client,
}

impl Minio {
    /// 创建一个新的Minio客户端
    ///
    /// # Arguments
    /// - `base_url`: MinIO服务器的基础URL。
    /// - `access_key`: MinIO认证的访问密钥。
    /// - `secret_key`: MinIO认证的密钥。
    /// - `session_token`: 可选的临时凭证会话令牌。
    /// - `cert_path`: 可选的SSL证书文件路径。
    pub fn new(
        base_url: &str,
        access_key: &str,
        secret_key: &str,
        session_token: Option<&str>,
        cert_path: Option<&Path>,
    ) -> Result<Self, anyhow::Error> {
        let base_url = base_url.parse::<BaseUrl>()?;
        let static_provider = StaticProvider::new(access_key, secret_key, session_token);

        let client = ClientBuilder::new(base_url)
            .provider(Some(Box::new(static_provider)))
            .ssl_cert_file(cert_path)
            .build()?;

        Ok(Self { client })
    }

    /// 初始化bucket
    pub async fn init_bucket(&self, buckets: Vec<String>) -> Result<(), anyhow::Error> {
        for bucket_name in &buckets {
            let resp = self.client.bucket_exists(bucket_name).send().await?;

            if !resp.exists {
                self.client.create_bucket(bucket_name).send().await?;
                // 自定义桶策略
                // 这里设置桶策略为公开读取
                let config = format!(
                    r#"{{
                        "Version": "2012-10-17",
                        "Statement": [
                            {{
                                "Effect": "Allow",
                                "Principal": {{"AWS": "*"}},
                                "Action": "s3:GetObject",
                                "Resource": "arn:aws:s3:::{}/*"
                            }}
                        ]
                    }}"#,
                    bucket_name
                );
                self.client.put_bucket_policy(bucket_name).config(config).send().await?;
            }
        }

        Ok(())
    }

    // 获取Minio客户端
    pub fn get_client(&self) -> &minio::s3::client::Client {
        &self.client
    }
}

static MINIO: OnceCell<Arc<Minio>> = OnceCell::const_new();

/// 初始化Minio客户端
pub async fn init_minio(
    base_url: &str,
    access_key: &str,
    secret_key: &str,
    session_token: Option<&str>,
    cert_path: Option<&Path>,
) -> Result<&'static Arc<Minio>, anyhow::Error> {
    Ok(MINIO
        .get_or_init(|| async {
            let minio = match Minio::new(base_url, access_key, secret_key, session_token, cert_path)
            {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Failed to initialize the MinIO client: {:?}", e);
                    std::process::exit(-1);
                }
            };

            Arc::new(minio)
        })
        .await)
}

/// 获取Minio客户端
pub fn get_minio_client() -> Result<&'static minio::s3::client::Client, anyhow::Error> {
    match MINIO.get() {
        Some(minio) => Ok(minio.get_client()),
        None => Err(anyhow::anyhow!("MinIO client not initialized")),
    }
}
