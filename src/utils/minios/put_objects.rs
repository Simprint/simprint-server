//! MinIO 对象上传模块
//!
//! 提供各类资源上传到 MinIO 的功能

use bytes::Bytes;
use minio::s3::{segmented_bytes::SegmentedBytes, types::S3Api};
use sha2::{Digest, Sha256};

use super::get_minio_client;

/// 上传通用对象到 MinIO
///
/// # Arguments
/// - `bucket_name`: 存储桶名称
/// - `object_path`: 对象路径（如 `hash/version/filename`）
/// - `data`: 对象数据
pub async fn put_object(
    bucket_name: &str,
    object_path: &str,
    data: Bytes,
) -> Result<(), anyhow::Error> {
    let client = get_minio_client()?;
    let data: SegmentedBytes = SegmentedBytes::from(data);
    client.put_object(bucket_name, object_path, data).send().await?;
    Ok(())
}

/// 上传扩展 CRX 文件到 MinIO
///
/// 存储路径格式：`{extension_id_hash}/{version}/{crx_hash}.crx`
///
/// # Arguments
/// - `bucket_name`: 扩展存储桶名称
/// - `extension_id`: 扩展 ID（如 Chrome 扩展 ID）
/// - `version`: 扩展版本号
/// - `crx_hash`: CRX 文件内容的哈希值
/// - `data`: CRX 文件数据
///
/// # Returns
/// 返回存储的对象路径（不包含 bucket 名称）
pub async fn put_extension_crx(
    bucket_name: &str,
    extension_id: &str,
    version: &str,
    crx_hash: &str,
    data: Bytes,
) -> Result<String, anyhow::Error> {
    let client = get_minio_client()?;

    // 计算扩展 ID 的哈希作为目录名
    let extension_id_hash = calculate_short_hash(extension_id);

    // 构建对象路径：{extension_id_hash}/{version}/{crx_hash}.crx
    let object_path = format!("{}/{}/{}.crx", extension_id_hash, version, crx_hash);

    let data: SegmentedBytes = SegmentedBytes::from(data);
    client.put_object(bucket_name, &object_path, data).send().await?;

    tracing::info!(
        "Extension CRX uploaded: bucket={}, path={}",
        bucket_name,
        object_path
    );

    Ok(object_path)
}

/// 上传扩展图标到 MinIO
///
/// 存储路径格式：`{extension_id_hash}/icons/{icon_hash}.{ext}`
///
/// # Arguments
/// - `bucket_name`: 扩展存储桶名称
/// - `extension_id`: 扩展 ID
/// - `icon_hash`: 图标文件的哈希值
/// - `extension`: 文件扩展名（如 `png`、`svg`）
/// - `data`: 图标数据
///
/// # Returns
/// 返回存储的对象路径
pub async fn put_extension_icon(
    bucket_name: &str,
    extension_id: &str,
    icon_hash: &str,
    extension: &str,
    data: Bytes,
) -> Result<String, anyhow::Error> {
    let client = get_minio_client()?;

    let extension_id_hash = calculate_short_hash(extension_id);
    let object_path = format!("{}/icons/{}.{}", extension_id_hash, icon_hash, extension);

    let data: SegmentedBytes = SegmentedBytes::from(data);
    client.put_object(bucket_name, &object_path, data).send().await?;

    tracing::info!(
        "Extension icon uploaded: bucket={}, path={}",
        bucket_name,
        object_path
    );

    Ok(object_path)
}

/// 上传头像对象到 MinIO
pub async fn put_avatar_object(
    bucket_name: &str,
    object: &str,
    data: Bytes,
) -> Result<(), anyhow::Error> {
    let client = get_minio_client()?;
    let data: SegmentedBytes = SegmentedBytes::from(data);
    client.put_object(bucket_name, object, data).send().await?;
    Ok(())
}

/// 计算字符串的短哈希值（用于目录命名）
///
/// 使用 SHA256 取前 16 个字符
fn calculate_short_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8]) // 取前 8 字节 = 16 个十六进制字符
}

/// 计算文件内容的哈希值（SHA256）
pub fn calculate_file_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}
