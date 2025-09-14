//! 腾讯云 STS (Security Token Service) 临时密钥模块
//!
//! 本模块提供获取临时密钥的功能，主要用于前端应用的临时授权场景。
//! 临时密钥包含 TmpSecretId、TmpSecretKey 和 Token 三个字段。
//!
//! 基于腾讯云官方STS SDK实现，使用腾讯云SDK的签名方法
//! 参考文档：https://cloud.tencent.com/document/product/436/14048

use crate::error::CosError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use url::form_urlencoded;

/// STS 临时密钥客户端
#[derive(Debug, Clone)]
pub struct StsClient {
    secret_id: String,
    secret_key: String,
    region: String,
    client: Client,
}

/// 临时密钥响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryCredentials {
    /// 临时访问密钥 ID
    #[serde(rename = "TmpSecretId")]
    pub tmp_secret_id: String,
    /// 临时访问密钥
    #[serde(rename = "TmpSecretKey")]
    pub tmp_secret_key: String,
    /// 安全令牌
    #[serde(rename = "Token")]
    pub token: String,
    /// 过期时间戳（可选，因为新版API可能不返回此字段）
    #[serde(rename = "ExpiredTime", skip_serializing_if = "Option::is_none")]
    pub expired_time: Option<u64>,
}

/// STS API 响应
#[derive(Debug, Deserialize)]
struct StsResponse {
    #[serde(rename = "Response")]
    response: StsResponseData,
}

#[derive(Debug, Deserialize)]
struct StsResponseData {
    #[serde(rename = "Credentials")]
    credentials: Option<TemporaryCredentials>,
    #[serde(rename = "Error")]
    error: Option<StsError>,
    #[serde(rename = "ExpiredTime")]
    expired_time: Option<u64>,
    #[serde(rename = "Expiration")]
    expiration: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StsError {
    #[serde(rename = "Code")]
    code: String,
    #[serde(rename = "Message")]
    message: String,
}

/// 权限策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// 策略语法版本
    pub version: String,
    /// 策略声明列表
    pub statement: Vec<Statement>,
}

/// 策略声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    /// 效果：allow 或 deny
    pub effect: String,
    /// 允许的操作列表
    pub action: Vec<String>,
    /// 资源列表
    pub resource: Vec<String>,
    /// 条件（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// 临时密钥请求参数
#[derive(Debug, Clone)]
pub struct GetCredentialsRequest {
    /// 权限策略
    pub policy: Policy,
    /// 有效期（秒），默认 1800 秒
    pub duration_seconds: Option<u32>,
    /// 会话名称
    pub name: Option<String>,
}

impl StsClient {
    /// 创建 STS 客户端
    pub fn new(secret_id: String, secret_key: String, region: String) -> Self {
        Self {
            secret_id,
            secret_key,
            region,
            client: Client::new(),
        }
    }

    /// 获取临时密钥
    /// 使用腾讯云官方STS SDK的签名方法
    pub async fn get_credentials(
        &self,
        request: GetCredentialsRequest,
    ) -> Result<TemporaryCredentials, CosError> {
        let policy_json = serde_json::to_string(&request.policy)
            .map_err(|e| CosError::other(format!("Policy serialization error: {}", e)))?;
        
        let duration_seconds = request.duration_seconds.unwrap_or(1800);
        let name = request.name.unwrap_or_else(|| "temp-user".to_string());
        
        // 使用腾讯云STS SDK的方式：GET请求 + URL参数
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let nonce = timestamp; // 使用时间戳作为随机数
        
        // 构建请求参数 - 先创建所有字符串变量以确保生命周期
          let timestamp_str = timestamp.to_string();
          let nonce_str = nonce.to_string();
          // Policy参数需要URL编码，不是base64编码
          let encoded_policy = urlencoding::encode(&policy_json).to_string();
          let duration_str = duration_seconds.to_string();
         
         let mut params = HashMap::new();
          params.insert("Action", "GetFederationToken");
          params.insert("Version", "2018-08-13");
          params.insert("Region", &self.region);
          params.insert("SecretId", &self.secret_id);
          params.insert("Timestamp", &timestamp_str);
          params.insert("Nonce", &nonce_str);
          params.insert("Name", &name);
           params.insert("Policy", &encoded_policy);
           params.insert("DurationSeconds", &duration_str);
         
         // 生成签名
         let signature = self.generate_signature(&params)?;
         params.insert("Signature", &signature);
        
        // 构建URL
        let query_string = params.iter()
            .map(|(k, v)| {
                let encoded_value = form_urlencoded::byte_serialize(v.as_bytes()).collect::<String>();
                format!("{}={}", k, encoded_value)
            })
            .collect::<Vec<_>>()
            .join("&");
        
        let url = format!("https://sts.tencentcloudapi.com/?{}", query_string);
        
        // 发送GET请求
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| CosError::other(format!("Request failed: {}", e)))?;
        
        let response_text = response.text().await
            .map_err(|e| CosError::other(format!("Failed to read response: {}", e)))?;
        

        // 解析响应 - 使用新版API格式
        if response_text.contains("\"Response\"") {
            // 新版API响应格式
            let sts_response: StsResponse = serde_json::from_str(&response_text)
                .map_err(|e| CosError::other(format!("Response parsing error: {}\nResponse: {}", e, response_text)))?;
            
            if let Some(error) = sts_response.response.error {
                return Err(CosError::other(format!("STS API error: {} - {}", error.code, error.message)));
            }
            
            let mut credentials = sts_response.response.credentials
                .ok_or_else(|| CosError::other("No credentials in response".to_string()))?;
            
            // 从响应的顶层获取ExpiredTime并设置到credentials中
            if let Some(expired_time) = sts_response.response.expired_time {
                credentials.expired_time = Some(expired_time);
            }
            
            Ok(credentials)
        } else {
            // 旧版API响应格式
            #[derive(Deserialize)]
            struct LegacyStsResponse {
                code: i32,
                message: String,
                #[serde(rename = "codeDesc")]
                data: Option<LegacyCredentialsData>,
            }
            
            #[derive(Deserialize)]
            struct LegacyCredentialsData {
                credentials: LegacyCredentials,
                #[serde(rename = "expiredTime")]
                expired_time: u64,
            }
            
            #[derive(Deserialize)]
            struct LegacyCredentials {
                #[serde(rename = "tmpSecretId")]
                tmp_secret_id: String,
                #[serde(rename = "tmpSecretKey")]
                tmp_secret_key: String,
                #[serde(rename = "sessionToken")]
                session_token: String,
            }
            
            let legacy_response: LegacyStsResponse = serde_json::from_str(&response_text)
                .map_err(|e| CosError::other(format!("Legacy response parsing error: {}\nResponse: {}", e, response_text)))?;
            
            if legacy_response.code != 0 {
                return Err(CosError::other(format!("STS API error: {} - {}", legacy_response.code, legacy_response.message)));
            }
            
            let data = legacy_response.data
                .ok_or_else(|| CosError::other("No data in legacy response".to_string()))?;
            
            Ok(TemporaryCredentials {
                tmp_secret_id: data.credentials.tmp_secret_id,
                tmp_secret_key: data.credentials.tmp_secret_key,
                token: data.credentials.session_token,
                expired_time: Some(data.expired_time),
            })
        }
    }
    
    /// 生成腾讯云 STS API 签名（使用官方SDK的简单签名方法）
    fn generate_signature(
        &self,
        params: &HashMap<&str, &str>,
    ) -> Result<String, CosError> {
        use hmac::{Hmac, Mac};
        use sha1::Sha1;
        
        type HmacSha1 = Hmac<Sha1>;
        
        // 1. 对参数进行排序
        let mut sorted_params: Vec<(&str, &str)> = params.iter()
            .filter(|(k, _)| **k != "Signature") // 排除Signature参数
            .map(|(k, v)| (*k, *v))
            .collect();
        sorted_params.sort_by(|a, b| a.0.cmp(b.0));
        
        // 2. 构建查询字符串
         let query_string = sorted_params.iter()
             .map(|(k, v)| format!("{}={}", k, v))
             .collect::<Vec<_>>()
             .join("&");
         
         // 3. 构建签名原文字符串 - 按照腾讯云签名方法v1格式
         // 格式：请求方法 + 请求主机 + 请求路径 + ? + 请求字符串
         let string_to_sign = format!("GET{}/?{}", "sts.tencentcloudapi.com", query_string);
        
        // 4. 计算签名 - 使用HMAC-SHA1算法，然后base64编码
         let mut mac = HmacSha1::new_from_slice(self.secret_key.as_bytes())
             .map_err(|e| CosError::other(format!("HMAC key error: {}", e)))?;
         mac.update(string_to_sign.as_bytes());
         
         let signature = base64::encode(mac.finalize().into_bytes());
         Ok(signature)
    }
}

impl Policy {
    /// 创建新的权限策略
    pub fn new() -> Self {
        Self {
            version: "2.0".to_string(),
            statement: Vec::new(),
        }
    }
    
    /// 添加策略声明
    pub fn add_statement(mut self, statement: Statement) -> Self {
        self.statement.push(statement);
        self
    }
    
    /// 创建允许上传对象的策略
    pub fn allow_put_object(bucket: &str, prefix: Option<&str>) -> Self {
        // 从bucket名称中提取appid (格式: bucket-appid)
        let parts: Vec<&str> = bucket.rsplitn(2, '-').collect();
        let (bucket_name, appid) = if parts.len() == 2 {
            (parts[1], parts[0])
        } else {
            (bucket, "*")
        };
        
        let resource = if let Some(prefix) = prefix {
            format!("qcs::cos:*:uid/{}:prefix//{}/{}/{}*", appid, appid, bucket_name, prefix)
        } else {
            format!("qcs::cos:*:uid/{}:prefix//{}/{}/*", appid, appid, bucket_name)
        };
        
        Self::new().add_statement(Statement {
            effect: "allow".to_string(),
            action: vec![
                "name/cos:PutObject".to_string(),
                "name/cos:PostObject".to_string(),
                "name/cos:InitiateMultipartUpload".to_string(),
                "name/cos:ListMultipartUploads".to_string(),
                "name/cos:ListParts".to_string(),
                "name/cos:UploadPart".to_string(),
                "name/cos:CompleteMultipartUpload".to_string(),
            ],
            resource: vec![resource],
            condition: None,
        })
    }
    
    /// 创建允许下载对象的策略
    pub fn allow_get_object(bucket: &str, prefix: Option<&str>) -> Self {
        // 从bucket名称中提取appid (格式: bucket-appid)
        let parts: Vec<&str> = bucket.rsplitn(2, '-').collect();
        let (bucket_name, appid) = if parts.len() == 2 {
            (parts[1], parts[0])
        } else {
            (bucket, "*")
        };
        
        let resource = if let Some(prefix) = prefix {
            format!("qcs::cos:*:uid/{}:prefix//{}/{}/{}*", appid, appid, bucket_name, prefix)
        } else {
            format!("qcs::cos:*:uid/{}:prefix//{}/{}/*", appid, appid, bucket_name)
        };
        
        Self::new().add_statement(Statement {
            effect: "allow".to_string(),
            action: vec![
                "name/cos:GetObject".to_string(),
                "name/cos:HeadObject".to_string(),
            ],
            resource: vec![resource],
            condition: None,
        })
    }
    
    /// 创建允许删除对象的策略
    pub fn allow_delete_object(bucket: &str, prefix: Option<&str>) -> Self {
        // 从bucket名称中提取appid (格式: bucket-appid)
        let parts: Vec<&str> = bucket.rsplitn(2, '-').collect();
        let (bucket_name, appid) = if parts.len() == 2 {
            (parts[1], parts[0])
        } else {
            (bucket, "*")
        };
        
        let resource = if let Some(prefix) = prefix {
            format!("qcs::cos:*:uid/{}:prefix//{}/{}/{}*", appid, appid, bucket_name, prefix)
        } else {
            format!("qcs::cos:*:uid/{}:prefix//{}/{}/*", appid, appid, bucket_name)
        };
        
        Self::new().add_statement(Statement {
            effect: "allow".to_string(),
            action: vec![
                "name/cos:DeleteObject".to_string(),
            ],
            resource: vec![resource],
            condition: None,
        })
    }
    
    /// 创建允许上传和下载对象的策略
    pub fn allow_read_write(bucket: &str, prefix: Option<&str>) -> Self {
        // 从bucket名称中提取appid (格式: bucket-appid)
        let parts: Vec<&str> = bucket.rsplitn(2, '-').collect();
        let (bucket_name, appid) = if parts.len() == 2 {
            (parts[1], parts[0])
        } else {
            (bucket, "*")
        };
        
        let resource = if let Some(prefix) = prefix {
            format!("qcs::cos:*:uid/{}:prefix//{}/{}/{}*", appid, appid, bucket_name, prefix)
        } else {
            format!("qcs::cos:*:uid/{}:prefix//{}/{}/*", appid, appid, bucket_name)
        };
        
        Self::new().add_statement(Statement {
            effect: "allow".to_string(),
            action: vec![
                "name/cos:PutObject".to_string(),
                "name/cos:PostObject".to_string(),
                "name/cos:GetObject".to_string(),
                "name/cos:HeadObject".to_string(),
                "name/cos:DeleteObject".to_string(),
                "name/cos:InitiateMultipartUpload".to_string(),
                "name/cos:ListMultipartUploads".to_string(),
                "name/cos:ListParts".to_string(),
                "name/cos:UploadPart".to_string(),
                "name/cos:CompleteMultipartUpload".to_string(),
            ],
            resource: vec![resource],
            condition: None,
        })
    }
}

impl Default for Policy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_policy_creation() {
        let policy = Policy::allow_put_object("test-bucket-1234567890", Some("uploads/"));
        assert_eq!(policy.version, "2.0");
        assert_eq!(policy.statement.len(), 1);
        assert_eq!(policy.statement[0].effect, "allow");
        assert!(policy.statement[0].action.contains(&"cos:PutObject".to_string()));
    }
    
    #[test]
    fn test_policy_serialization() {
        let policy = Policy::allow_read_write("test-bucket", None);
        let json = serde_json::to_string(&policy).unwrap();
        assert!(json.contains("version"));
        assert!(json.contains("statement"));
    }
}