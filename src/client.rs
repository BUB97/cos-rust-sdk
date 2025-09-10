//! HTTP 客户端模块
//!
//! 提供与腾讯云 COS 服务通信的 HTTP 客户端功能

use crate::auth::Auth;
use crate::config::Config;
use crate::error::{CosError, Result};
use chrono::{Duration, Utc};
use reqwest::{Client, Method, Response};
use serde_json::Value;
use std::collections::HashMap;

/// COS HTTP 客户端
#[derive(Debug, Clone)]
pub struct CosClient {
    config: Config,
    auth: Auth,
    http_client: Client,
}

impl CosClient {
    /// 创建新的 COS 客户端
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;
        
        let auth = Auth::new(&config.secret_id, &config.secret_key);
        let http_client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| CosError::other(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            config,
            auth,
            http_client,
        })
    }

    /// 发送 GET 请求
    pub async fn get(&self, path: &str, params: HashMap<String, String>) -> Result<Response> {
        self.request(Method::GET, path, params, None::<&[u8]>).await
    }

    /// 发送 PUT 请求
    pub async fn put<T>(&self, path: &str, params: HashMap<String, String>, body: Option<T>) -> Result<Response>
    where
        T: Into<reqwest::Body>,
    {
        self.request(Method::PUT, path, params, body).await
    }

    /// 发送 POST 请求
    pub async fn post<T>(&self, path: &str, params: HashMap<String, String>, body: Option<T>) -> Result<Response>
    where
        T: Into<reqwest::Body>,
    {
        self.request(Method::POST, path, params, body).await
    }

    /// 发送 DELETE 请求
    pub async fn delete(&self, path: &str, params: HashMap<String, String>) -> Result<Response> {
        self.request(Method::DELETE, path, params, None::<&[u8]>).await
    }

    /// 发送 HEAD 请求
    pub async fn head(&self, path: &str, params: HashMap<String, String>) -> Result<Response> {
        self.request(Method::HEAD, path, params, None::<&[u8]>).await
    }

    /// 通用请求方法
    async fn request<T>(
        &self,
        method: Method,
        path: &str,
        params: HashMap<String, String>,
        body: Option<T>,
    ) -> Result<Response>
    where
        T: Into<reqwest::Body>,
    {
        let url = self.build_url(path, &params)?;
        let mut headers = self.build_headers(&method, path, &params)?;
        
        // 构建请求
        let mut request_builder = self.http_client.request(method.clone(), &url);
        
        // 添加请求头
        for (key, value) in headers.iter() {
            request_builder = request_builder.header(key, value);
        }
        
        // 添加请求体
        if let Some(body) = body {
            request_builder = request_builder.body(body);
        }
        
        // 发送请求
        let response = request_builder
            .send()
            .await
            .map_err(|e| CosError::other(format!("Request failed: {}", e)))?;
        
        // 检查响应状态
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            return Err(CosError::server(
                status.to_string(),
                error_text
            ));
        }
        
        Ok(response)
    }

    /// 构建完整的 URL
    fn build_url(&self, path: &str, params: &HashMap<String, String>) -> Result<String> {
        let base_url = if path.starts_with('/') {
            self.config.bucket_url()?
        } else {
            self.config.service_url()
        };
        
        let mut url = format!("{}{}", base_url, path);
        
        if !params.is_empty() {
            let query_string = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                .collect::<Vec<_>>()
                .join("&");
            url.push('?');
            url.push_str(&query_string);
        }
        
        Ok(url)
    }

    /// 构建请求头
    fn build_headers(
        &self,
        method: &Method,
        path: &str,
        params: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>> {
        let mut headers = HashMap::new();
        
        // 基础请求头
        headers.insert("User-Agent".to_string(), crate::USER_AGENT.to_string());
        headers.insert("Host".to_string(), self.get_host(path)?);
        
        // 时间相关
        let now = Utc::now();
        let start_time = now - Duration::minutes(5); // 提前5分钟
        let end_time = now + Duration::hours(1);     // 1小时后过期
        
        // 生成授权签名
        let authorization = self.auth.sign(
            method.as_str(),
            path,
            &headers,
            params,
            start_time,
            end_time,
        )?;
        
        headers.insert("Authorization".to_string(), authorization);
        
        Ok(headers)
    }

    /// 获取主机名
    fn get_host(&self, path: &str) -> Result<String> {
        let url = if path.starts_with('/') {
            self.config.bucket_url()?
        } else {
            self.config.service_url()
        };
        
        let parsed_url = url::Url::parse(&url)
            .map_err(|e| CosError::other(format!("Invalid URL: {}", e)))?;
        
        Ok(parsed_url.host_str().unwrap_or("localhost").to_string())
    }

    /// 解析 XML 响应
    pub async fn parse_xml_response(response: Response) -> Result<Value> {
        let text = response
            .text()
            .await
            .map_err(|e| CosError::other(format!("Failed to read response: {}", e)))?;
        
        // 简单的 XML 到 JSON 转换（实际项目中可能需要更复杂的解析）
        serde_json::from_str(&text)
            .map_err(|e| CosError::other(format!("Failed to parse XML response: {}", e)))
    }

    /// 获取配置
    pub fn config(&self) -> &Config {
        &self.config
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
    use std::time::Duration as StdDuration;

    #[test]
    fn test_client_creation() {
        let config = Config::new("test_id", "test_key", "ap-beijing", "test-bucket-123")
            .with_timeout(StdDuration::from_secs(60));
        
        let client = CosClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_url() {
        let config = Config::new("test_id", "test_key", "ap-beijing", "test-bucket-123");
        let client = CosClient::new(config).unwrap();
        
        let mut params = HashMap::new();
        params.insert("key".to_string(), "value".to_string());
        
        let url = client.build_url("/test", &params).unwrap();
        assert!(url.contains("test-bucket-123.cos.ap-beijing.myqcloud.com"));
        assert!(url.contains("key=value"));
    }
}