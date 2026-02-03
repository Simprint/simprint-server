use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use rand::RngCore;
use sha2::{Digest, Sha256};

const NONCE_SIZE: usize = 12;

/// 应用程序固定标识符，用于派生加密密钥
/// 注意：此值一旦确定后不可修改，否则已加密的数据将无法解密
const APP_SECRET_SALT: &str = "simprint-fingerprint-browser-v1-secure-salt-2024";

/// 加密密码
///
/// 使用 AES-256-GCM 加密，返回 base64 编码的密文
pub fn encrypt_password(password: &str, key: &[u8; 32]) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;

    // 生成随机 nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 加密
    let ciphertext = cipher.encrypt(nonce, password.as_bytes()).map_err(|e| e.to_string())?;

    // 组合 nonce + ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);

    Ok(BASE64.encode(result))
}

/// 解密密码
///
/// 接收 base64 编码的密文，返回原始密码
pub fn decrypt_password(encrypted: &str, key: &[u8; 32]) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;

    // Base64 解码
    let data = BASE64.decode(encrypted).map_err(|e| e.to_string())?;

    if data.len() < NONCE_SIZE {
        return Err("Invalid encrypted data".to_string());
    }

    // 分离 nonce 和 ciphertext
    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    // 解密
    let plaintext =
        cipher.decrypt(nonce, ciphertext).map_err(|_| "Decryption failed".to_string())?;

    String::from_utf8(plaintext).map_err(|e| e.to_string())
}

/// 获取加密密钥
///
/// 使用确定性算法从固定盐值派生 32 字节 AES-256 密钥
/// 每次调用都会返回相同的密钥，无需外部配置
pub fn get_encryption_key() -> [u8; 32] {
    // 使用 SHA-256 从固定盐值派生密钥
    let mut hasher = Sha256::new();
    hasher.update(APP_SECRET_SALT.as_bytes());
    let result = hasher.finalize();

    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = get_encryption_key();
        let password = "my_secret_password";

        let encrypted = encrypt_password(password, &key).unwrap();
        let decrypted = decrypt_password(&encrypted, &key).unwrap();

        assert_eq!(password, decrypted);
    }

    #[test]
    fn test_key_deterministic() {
        // 确保多次调用返回相同的密钥
        let key1 = get_encryption_key();
        let key2 = get_encryption_key();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_encrypt_decrypt_special_chars() {
        let key = get_encryption_key();
        let password = "P@ssw0rd!#$%^&*()_+-=[]{}|;':\",./<>?中文密码";

        let encrypted = encrypt_password(password, &key).unwrap();
        let decrypted = decrypt_password(&encrypted, &key).unwrap();

        assert_eq!(password, decrypted);
    }
}
