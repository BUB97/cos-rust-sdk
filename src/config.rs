//! 配置模块

use crate::error::{CosError, Result};
use std::time::Duration;

/// COS 客户端配置
#[derive(Debug, Clone)]
pub struct Config {
    /// 腾讯云 SecretId
    pub secret_id: String,
    /// 腾讯云 SecretKey
    pub secret_key: String,
    /// 地域
    pub region: String,
    /// 存储桶名称
    pub bucket: String,
    /// 请求超时时间
    pub timeout: Duration,
    /// 是否使用 HTTPS
    pub use_https: bool,
    /// 自定义域名
    pub domain: Option<String>,
    /// 应用 ID（从存储桶名称中提取）
    pub app_id: Option<String>,
}

impl Config {
    /// 创建新的配置
    pub fn new<S: Into<String>>(
        secret_id: S,
        secret_key: S,
        region: S,
        bucket: S,
    ) -> Self {
        let bucket_name = bucket.into();
        let app_id = extract_app_id(&bucket_name);
        
        Self {
            secret_id: secret_id.into(),
            secret_key: secret_key.into(),
            region: region.into(),
            bucket: bucket_name,
            timeout: Duration::from_secs(30),
            use_https: true,
            domain: None,
            app_id,
        }
    }

    /// 设置请求超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 设置是否使用 HTTPS
    pub fn with_https(mut self, use_https: bool) -> Self {
        self.use_https = use_https;
        self
    }

    /// 设置自定义域名
    pub fn with_domain<S: Into<String>>(mut self, domain: S) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// 获取存储桶的完整 URL
    pub fn bucket_url(&self) -> Result<String> {
        if let Some(ref domain) = self.domain {
            Ok(format!(
                "{}://{}",
                if self.use_https { "https" } else { "http" },
                domain
            ))
        } else {
            Ok(format!(
                "{}://{}.cos.{}.myqcloud.com",
                if self.use_https { "https" } else { "http" },
                self.bucket,
                self.region
            ))
        }
    }

    /// 获取服务 URL（用于获取存储桶列表等操作）
    pub fn service_url(&self) -> String {
        format!(
            "{}://cos.{}.myqcloud.com",
            if self.use_https { "https" } else { "http" },
            self.region
        )
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        if self.secret_id.is_empty() {
            return Err(CosError::config("SecretId cannot be empty"));
        }
        if self.secret_key.is_empty() {
            return Err(CosError::config("SecretKey cannot be empty"));
        }
        if self.region.is_empty() {
            return Err(CosError::config("Region cannot be empty"));
        }
        if self.bucket.is_empty() {
            return Err(CosError::config("Bucket cannot be empty"));
        }
        Ok(())
    }
}

/// 从存储桶名称中提取应用 ID
/// 存储桶名称格式：{bucket-name}-{app-id}
fn extract_app_id(bucket_name: &str) -> Option<String> {
    bucket_name
        .rfind('-')
        .and_then(|pos| {
            let app_id = &bucket_name[pos + 1..];
            if app_id.chars().all(|c| c.is_ascii_digit()) && !app_id.is_empty() {
                Some(app_id.to_string())
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_app_id() {
        assert_eq!(extract_app_id("mybucket-1234567890"), Some("1234567890".to_string()));
        assert_eq!(extract_app_id("test-bucket-1234567890"), Some("1234567890".to_string()));
        assert_eq!(extract_app_id("mybucket"), None);
        assert_eq!(extract_app_id("mybucket-abc"), None);
    }

    #[test]
    fn test_config_validation() {
        let config = Config::new("id", "key", "region", "bucket-123");
        assert!(config.validate().is_ok());

        let config = Config::new("", "key", "region", "bucket-123");
        assert!(config.validate().is_err());
    }
}