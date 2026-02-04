//! MinIO 对象 URL 获取模块
//!
//! 提供获取 MinIO 存储对象访问 URL 的功能

/// 获取对象的完整 URL
///
/// # Arguments
/// - `resource_url`: MinIO 资源访问基础 URL（如 `http://localhost:9000`）
/// - `bucket_name`: 存储桶名称
/// - `object_path`: 对象路径
///
/// # Returns
/// 返回完整的访问 URL
pub fn get_object_url(resource_url: &str, bucket_name: &str, object_path: &str) -> String {
    format!("{}/{}/{}", resource_url, bucket_name, object_path)
}

/// 获取扩展 CRX 文件的完整 URL
///
/// # Arguments
/// - `resource_url`: MinIO 资源访问基础 URL
/// - `bucket_name`: 扩展存储桶名称
/// - `object_path`: CRX 对象路径（数据库中存储的路径）
pub fn get_extension_crx_url(resource_url: &str, bucket_name: &str, object_path: &str) -> String {
    get_object_url(resource_url, bucket_name, object_path)
}

/// 获取扩展图标的完整 URL
///
/// # Arguments
/// - `resource_url`: MinIO 资源访问基础 URL
/// - `bucket_name`: 扩展存储桶名称
/// - `object_path`: 图标对象路径（数据库中存储的路径）
// pub fn get_extension_icon_url(resource_url: &str, bucket_name: &str, object_path: &str) -> String {
pub fn get_extension_icon_url(resource_url: &str, bucket_name: &str, object_path: &str) -> String {
    get_object_url(resource_url, bucket_name, object_path)
}

/// 获取头像的完整 URL
///
/// # Arguments
/// - `resource_url`: MinIO 资源访问基础 URL
/// - `bucket_name`: 头像存储桶名称
/// - `resource_hash`: 头像文件哈希
pub fn get_avatar_url(resource_url: &str, bucket_name: &str, resource_hash: &str) -> String {
    get_object_url(resource_url, bucket_name, resource_hash)
}

/// 获取版本资源的完整 URL
///
/// # Arguments
/// - `resource_url`: MinIO 资源访问基础 URL
/// - `bucket_name`: 版本资源存储桶名称
/// - `object_path`: 版本资源对象路径（数据库中存储的路径）
pub fn get_version_resource_url(
    resource_url: &str,
    bucket_name: &str,
    object_path: &str,
) -> String {
    get_object_url(resource_url, bucket_name, object_path)
}
