//! 认证模块
//!
//! 实现腾讯云 COS 的签名算法和认证逻辑

use crate::error::{CosError, Result};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha1::{Digest, Sha1};
use sha2::Sha256;
use std::collections::HashMap;
use url::Url;

type HmacSha1 = Hmac<Sha1>;

/// 认证信息
#[derive(Debug, Clone)]
pub struct Auth {
    pub secret_id: String,
    pub secret_key: String,
}

impl Auth {
    /// 创建新的认证实例
    pub fn new<S: Into<String>>(secret_id: S, secret_key: S) -> Self {
        Self {
            secret_id: secret_id.into(),
            secret_key: secret_key.into(),
        }
    }

    /// 生成授权签名
    pub fn sign(
        &self,
        method: &str,
        uri: &str,
        headers: &HashMap<String, String>,
        params: &HashMap<String, String>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<String> {
        // 1. 生成 KeyTime
        let key_time = format!("{};{}", start_time.timestamp(), end_time.timestamp());

        // 2. 生成 SignKey
        let sign_key = self.hmac_sha1(&key_time)?;

        // 3. 生成 HttpString
        let http_string = self.build_http_string(method, uri, headers, params)?;

        // 4. 生成 StringToSign
        let string_to_sign = format!("sha1\n{}\n{}\n", key_time, self.sha1(&http_string)?);

        // 5. 生成 Signature
        let signature = self.hmac_sha1_with_key(&string_to_sign, &sign_key)?;

        // 6. 生成 Authorization
        let authorization = format!(
            "q-sign-algorithm=sha1&q-ak={}&q-sign-time={}&q-key-time={}&q-header-list={}&q-url-param-list={}&q-signature={}",
            self.secret_id,
            key_time,
            key_time,
            self.build_header_list(headers),
            self.build_param_list(params),
            signature
        );

        Ok(authorization)
    }

    /// 构建 HTTP 字符串
    fn build_http_string(
        &self,
        method: &str,
        uri: &str,
        headers: &HashMap<String, String>,
        params: &HashMap<String, String>,
    ) -> Result<String> {
        let method = method.to_lowercase();
        let uri_path = self.encode_uri_path(uri)?;
        let params_string = self.build_params_string(params);
        let headers_string = self.build_headers_string(headers);

        Ok(format!(
            "{}\n{}\n{}\n{}\n",
            method, uri_path, params_string, headers_string
        ))
    }

    /// 编码 URI 路径
    fn encode_uri_path(&self, uri: &str) -> Result<String> {
        let url = Url::parse(&format!("http://example.com{}", uri))
            .map_err(|e| CosError::other(format!("Invalid URI: {}", e)))?;
        Ok(url.path().to_string())
    }

    /// 构建参数字符串
    fn build_params_string(&self, params: &HashMap<String, String>) -> String {
        let mut sorted_params: Vec<_> = params.iter().collect();
        sorted_params.sort_by_key(|(k, _)| k.to_lowercase());

        sorted_params
            .iter()
            .map(|(k, v)| format!("{}={}", k.to_lowercase(), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&")
    }

    /// 构建请求头字符串
    fn build_headers_string(&self, headers: &HashMap<String, String>) -> String {
        let mut sorted_headers: Vec<_> = headers.iter().collect();
        sorted_headers.sort_by_key(|(k, _)| k.to_lowercase());

        sorted_headers
            .iter()
            .map(|(k, v)| format!("{}={}", k.to_lowercase(), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&")
    }

    /// 构建请求头列表
    fn build_header_list(&self, headers: &HashMap<String, String>) -> String {
        let mut header_keys: Vec<_> = headers.keys().map(|k| k.to_lowercase()).collect();
        header_keys.sort();
        header_keys.join(";")
    }

    /// 构建参数列表
    fn build_param_list(&self, params: &HashMap<String, String>) -> String {
        let mut param_keys: Vec<_> = params.keys().map(|k| k.to_lowercase()).collect();
        param_keys.sort();
        param_keys.join(";")
    }

    /// HMAC-SHA1 签名
    fn hmac_sha1(&self, data: &str) -> Result<String> {
        self.hmac_sha1_with_key(data, &self.secret_key)
    }

    /// 使用指定密钥进行 HMAC-SHA1 签名
    fn hmac_sha1_with_key(&self, data: &str, key: &str) -> Result<String> {
        let mut mac = HmacSha1::new_from_slice(key.as_bytes())
            .map_err(|e| CosError::auth(format!("HMAC key error: {}", e)))?;
        mac.update(data.as_bytes());
        let result = mac.finalize();
        Ok(hex::encode(result.into_bytes()))
    }

    /// SHA1 哈希
    fn sha1(&self, data: &str) -> Result<String> {
        let mut hasher = Sha1::new();
        hasher.update(data.as_bytes());
        Ok(hex::encode(hasher.finalize()))
    }
}

/// URL 编码工具
mod urlencoding {
    pub fn encode(input: &str) -> String {
        url::form_urlencoded::byte_serialize(input.as_bytes()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_auth_sign() {
        let auth = Auth::new("test_secret_id", "test_secret_key");
        let mut headers = HashMap::new();
        headers.insert("host".to_string(), "example.com".to_string());
        headers.insert("content-type".to_string(), "application/json".to_string());

        let mut params = HashMap::new();
        params.insert("param1".to_string(), "value1".to_string());

        let start_time = Utc.timestamp_opt(1234567890, 0).unwrap();
        let end_time = Utc.timestamp_opt(1234567890 + 3600, 0).unwrap();

        let result = auth.sign("GET", "/test", &headers, &params, start_time, end_time);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_params_string() {
        let auth = Auth::new("id", "key");
        let mut params = HashMap::new();
        params.insert("b".to_string(), "value2".to_string());
        params.insert("a".to_string(), "value1".to_string());

        let result = auth.build_params_string(&params);
        assert_eq!(result, "a=value1&b=value2");
    }
}