//! 存储桶管理模块
//!
//! 提供存储桶的创建、删除、列表等管理功能

use crate::client::CosClient;
use crate::error::{CosError, Result};
use serde::Deserialize;
use std::collections::HashMap;

/// 存储桶操作客户端
#[derive(Debug, Clone)]
pub struct BucketClient {
    client: CosClient,
}

impl BucketClient {
    /// 创建新的存储桶操作客户端
    pub fn new(client: CosClient) -> Self {
        Self { client }
    }

    /// 创建存储桶
    pub async fn create_bucket(&self, acl: Option<BucketAcl>) -> Result<()> {
        let params = HashMap::new();
        let mut headers = HashMap::new();
        
        if let Some(acl) = acl {
            headers.insert("x-cos-acl".to_string(), acl.to_string());
        }
        
        let _response = self.client.put("/", params, None::<&[u8]>).await?;
        Ok(())
    }

    /// 删除存储桶
    pub async fn delete_bucket(&self) -> Result<()> {
        let params = HashMap::new();
        let _response = self.client.delete("/", params).await?;
        Ok(())
    }

    /// 检查存储桶是否存在
    pub async fn bucket_exists(&self) -> Result<bool> {
        let params = HashMap::new();
        match self.client.head("/", params).await {
            Ok(_) => Ok(true),
            Err(CosError::Server { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// 获取存储桶位置
    pub async fn get_bucket_location(&self) -> Result<String> {
        let mut params = HashMap::new();
        params.insert("location".to_string(), "".to_string());
        
        let response = self.client.get("/", params).await?;
        let response_text = response
            .text()
            .await
            .map_err(|e| CosError::other(format!("Failed to read response: {}", e)))?;
        
        let location_response: LocationResponse = quick_xml::de::from_str(&response_text)
            .map_err(|e| CosError::other(format!("Failed to parse location response: {}", e)))?;
        
        Ok(location_response.location_constraint)
    }

    /// 列出存储桶中的对象
    pub async fn list_objects(
        &self,
        options: Option<ListObjectsOptions>,
    ) -> Result<ListObjectsResponse> {
        let mut params = HashMap::new();
        
        if let Some(opts) = options {
            if let Some(prefix) = opts.prefix {
                params.insert("prefix".to_string(), prefix);
            }
            if let Some(delimiter) = opts.delimiter {
                params.insert("delimiter".to_string(), delimiter);
            }
            if let Some(marker) = opts.marker {
                params.insert("marker".to_string(), marker);
            }
            if let Some(max_keys) = opts.max_keys {
                params.insert("max-keys".to_string(), max_keys.to_string());
            }
        }
        
        let response = self.client.get("/", params).await?;
        let response_text = response
            .text()
            .await
            .map_err(|e| CosError::other(format!("Failed to read response: {}", e)))?;
        
        let list_response: ListObjectsResponse = quick_xml::de::from_str(&response_text)
            .map_err(|e| CosError::other(format!("Failed to parse list objects response: {}", e)))?;
        
        Ok(list_response)
    }

    /// 列出存储桶中的对象（V2版本）
    pub async fn list_objects_v2(
        &self,
        options: Option<ListObjectsV2Options>,
    ) -> Result<ListObjectsV2Response> {
        let mut params = HashMap::new();
        params.insert("list-type".to_string(), "2".to_string());
        
        if let Some(opts) = options {
            if let Some(prefix) = opts.prefix {
                params.insert("prefix".to_string(), prefix);
            }
            if let Some(delimiter) = opts.delimiter {
                params.insert("delimiter".to_string(), delimiter);
            }
            if let Some(continuation_token) = opts.continuation_token {
                params.insert("continuation-token".to_string(), continuation_token);
            }
            if let Some(max_keys) = opts.max_keys {
                params.insert("max-keys".to_string(), max_keys.to_string());
            }
            if let Some(start_after) = opts.start_after {
                params.insert("start-after".to_string(), start_after);
            }
        }
        
        let response = self.client.get("/", params).await?;
        let response_text = response
            .text()
            .await
            .map_err(|e| CosError::other(format!("Failed to read response: {}", e)))?;
        
        let list_response: ListObjectsV2Response = quick_xml::de::from_str(&response_text)
            .map_err(|e| CosError::other(format!("Failed to parse list objects v2 response: {}", e)))?;
        
        Ok(list_response)
    }

    /// 获取存储桶ACL
    pub async fn get_bucket_acl(&self) -> Result<BucketAclResponse> {
        let mut params = HashMap::new();
        params.insert("acl".to_string(), "".to_string());
        
        let response = self.client.get("/", params).await?;
        let response_text = response
            .text()
            .await
            .map_err(|e| CosError::other(format!("Failed to read response: {}", e)))?;
        
        let acl_response: BucketAclResponse = quick_xml::de::from_str(&response_text)
            .map_err(|e| CosError::other(format!("Failed to parse ACL response: {}", e)))?;
        
        Ok(acl_response)
    }

    /// 设置存储桶ACL
    pub async fn put_bucket_acl(&self, acl: BucketAcl) -> Result<()> {
        let mut params = HashMap::new();
        params.insert("acl".to_string(), "".to_string());
        
        let mut headers = HashMap::new();
        headers.insert("x-cos-acl".to_string(), acl.to_string());
        
        let _response = self.client.put("/", params, None::<&[u8]>).await?;
        Ok(())
    }

    /// 获取存储桶版本控制状态
    pub async fn get_bucket_versioning(&self) -> Result<VersioningResponse> {
        let mut params = HashMap::new();
        params.insert("versioning".to_string(), "".to_string());
        
        let response = self.client.get("/", params).await?;
        let response_text = response
            .text()
            .await
            .map_err(|e| CosError::other(format!("Failed to read response: {}", e)))?;
        
        let versioning_response: VersioningResponse = quick_xml::de::from_str(&response_text)
            .map_err(|e| CosError::other(format!("Failed to parse versioning response: {}", e)))?;
        
        Ok(versioning_response)
    }
}

/// 存储桶ACL类型
#[derive(Debug, Clone, Copy)]
pub enum BucketAcl {
    Private,
    PublicRead,
    PublicReadWrite,
    AuthenticatedRead,
}

impl ToString for BucketAcl {
    fn to_string(&self) -> String {
        match self {
            BucketAcl::Private => "private".to_string(),
            BucketAcl::PublicRead => "public-read".to_string(),
            BucketAcl::PublicReadWrite => "public-read-write".to_string(),
            BucketAcl::AuthenticatedRead => "authenticated-read".to_string(),
        }
    }
}

/// 列出对象选项
#[derive(Debug, Clone, Default)]
pub struct ListObjectsOptions {
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub marker: Option<String>,
    pub max_keys: Option<u32>,
}

/// 列出对象V2选项
#[derive(Debug, Clone, Default)]
pub struct ListObjectsV2Options {
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub continuation_token: Option<String>,
    pub max_keys: Option<u32>,
    pub start_after: Option<String>,
}

/// 存储桶位置响应
#[derive(Debug, Deserialize)]
#[serde(rename = "LocationConstraint")]
struct LocationResponse {
    #[serde(rename = "$text", default)]
    location_constraint: String,
}

/// 列出对象响应
#[derive(Debug, Deserialize)]
#[serde(rename = "ListBucketResult")]
pub struct ListObjectsResponse {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Prefix", default)]
    pub prefix: String,
    #[serde(rename = "Marker", default)]
    pub marker: String,
    #[serde(rename = "MaxKeys")]
    pub max_keys: u32,
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,
    #[serde(rename = "Contents", default)]
    pub contents: Vec<ObjectInfo>,
    #[serde(rename = "CommonPrefixes", default)]
    pub common_prefixes: Vec<CommonPrefix>,
}

/// 列出对象V2响应
#[derive(Debug, Deserialize)]
#[serde(rename = "ListBucketResult")]
pub struct ListObjectsV2Response {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Prefix", default)]
    pub prefix: String,
    #[serde(rename = "KeyCount")]
    pub key_count: u32,
    #[serde(rename = "MaxKeys")]
    pub max_keys: u32,
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,
    #[serde(rename = "ContinuationToken", default)]
    pub continuation_token: String,
    #[serde(rename = "NextContinuationToken", default)]
    pub next_continuation_token: String,
    #[serde(rename = "Contents", default)]
    pub contents: Vec<ObjectInfo>,
    #[serde(rename = "CommonPrefixes", default)]
    pub common_prefixes: Vec<CommonPrefix>,
}

/// 对象信息
#[derive(Debug, Deserialize)]
pub struct ObjectInfo {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "LastModified")]
    pub last_modified: String,
    #[serde(rename = "ETag")]
    pub etag: String,
    #[serde(rename = "Size")]
    pub size: u64,
    #[serde(rename = "StorageClass", default)]
    pub storage_class: String,
}

/// 公共前缀
#[derive(Debug, Deserialize)]
pub struct CommonPrefix {
    #[serde(rename = "Prefix")]
    pub prefix: String,
}

/// 存储桶ACL响应
#[derive(Debug, Deserialize)]
#[serde(rename = "AccessControlPolicy")]
pub struct BucketAclResponse {
    #[serde(rename = "Owner")]
    pub owner: Owner,
    #[serde(rename = "AccessControlList")]
    pub access_control_list: AccessControlList,
}

/// 所有者信息
#[derive(Debug, Deserialize)]
pub struct Owner {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "DisplayName", default)]
    pub display_name: String,
}

/// 访问控制列表
#[derive(Debug, Deserialize)]
pub struct AccessControlList {
    #[serde(rename = "Grant", default)]
    pub grants: Vec<Grant>,
}

/// 授权信息
#[derive(Debug, Deserialize)]
pub struct Grant {
    #[serde(rename = "Grantee")]
    pub grantee: Grantee,
    #[serde(rename = "Permission")]
    pub permission: String,
}

/// 被授权者
#[derive(Debug, Deserialize)]
pub struct Grantee {
    #[serde(rename = "@type")]
    pub grantee_type: String,
    #[serde(rename = "ID", default)]
    pub id: String,
    #[serde(rename = "DisplayName", default)]
    pub display_name: String,
    #[serde(rename = "URI", default)]
    pub uri: String,
}

/// 版本控制响应
#[derive(Debug, Deserialize)]
#[serde(rename = "VersioningConfiguration")]
pub struct VersioningResponse {
    #[serde(rename = "Status", default)]
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::time::Duration;

    #[tokio::test]
    async fn test_bucket_operations() {
        let config = Config::new("test_id", "test_key", "ap-beijing", "test-bucket-123")
            .with_timeout(Duration::from_secs(60));
        
        let cos_client = CosClient::new(config).unwrap();
        let bucket_client = BucketClient::new(cos_client);
        
        // 测试存储桶存在性检查
        let exists = bucket_client.bucket_exists().await;
        // 在实际测试中，这里会根据具体情况返回结果
    }
}