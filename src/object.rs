//! 对象存储操作模块
//!
//! 提供对象的上传、下载、删除等核心功能

use crate::client::CosClient;
use crate::error::{CosError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// 对象操作客户端
#[derive(Debug, Clone)]
pub struct ObjectClient {
    client: CosClient,
}

impl ObjectClient {
    /// 创建新的对象操作客户端
    pub fn new(client: CosClient) -> Self {
        Self { client }
    }

    /// 上传对象
    pub async fn put_object(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: Option<&str>,
    ) -> Result<PutObjectResponse> {
        let params = HashMap::new();
        
        let mut headers = HashMap::new();
        if let Some(ct) = content_type {
            headers.insert("Content-Type".to_string(), ct.to_string());
        }
        headers.insert("Content-Length".to_string(), data.len().to_string());
        
        let response = self.client.put(&format!("/{}", key), params, Some(data)).await?;
        
        Ok(PutObjectResponse {
            etag: response
                .headers()
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string(),
            version_id: response
                .headers()
                .get("x-cos-version-id")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
        })
    }

    /// 从文件上传对象
    pub async fn put_object_from_file(
        &self,
        key: &str,
        file_path: &Path,
        content_type: Option<&str>,
    ) -> Result<PutObjectResponse> {
        let mut file = File::open(file_path)
            .await
            .map_err(|e| CosError::other(format!("Failed to open file: {}", e)))?;
        
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .await
            .map_err(|e| CosError::other(format!("Failed to read file: {}", e)))?;
        
        let content_type = content_type.or_else(|| {
            file_path
                .extension()
                .and_then(|ext| ext.to_str())
                .and_then(|ext| match ext.to_lowercase().as_str() {
                    // 文本文件
                    "txt" => Some("text/plain"),
                    "html" | "htm" => Some("text/html"),
                    "css" => Some("text/css"),
                    "js" => Some("application/javascript"),
                    "json" => Some("application/json"),
                    "xml" => Some("application/xml"),
                    "csv" => Some("text/csv"),
                    "md" => Some("text/markdown"),
                    
                    // 图片格式
                    "jpg" | "jpeg" => Some("image/jpeg"),
                    "png" => Some("image/png"),
                    "gif" => Some("image/gif"),
                    "webp" => Some("image/webp"),
                    "bmp" => Some("image/bmp"),
                    "tiff" | "tif" => Some("image/tiff"),
                    "svg" => Some("image/svg+xml"),
                    "ico" => Some("image/x-icon"),
                    "heic" => Some("image/heic"),
                    "heif" => Some("image/heif"),
                    "avif" => Some("image/avif"),
                    "jxl" => Some("image/jxl"),
                    
                    // 视频格式
                    "mp4" => Some("video/mp4"),
                    "avi" => Some("video/x-msvideo"),
                    "mov" => Some("video/quicktime"),
                    "wmv" => Some("video/x-ms-wmv"),
                    "flv" => Some("video/x-flv"),
                    "webm" => Some("video/webm"),
                    "mkv" => Some("video/x-matroska"),
                    "m4v" => Some("video/x-m4v"),
                    "3gp" => Some("video/3gpp"),
                    "3g2" => Some("video/3gpp2"),
                    "ts" => Some("video/mp2t"),
                    "mts" => Some("video/mp2t"),
                    "m2ts" => Some("video/mp2t"),
                    "ogv" => Some("video/ogg"),
                    
                    // 音频格式
                    "mp3" => Some("audio/mpeg"),
                    "wav" => Some("audio/wav"),
                    "flac" => Some("audio/flac"),
                    "aac" => Some("audio/aac"),
                    "ogg" => Some("audio/ogg"),
                    "wma" => Some("audio/x-ms-wma"),
                    "m4a" => Some("audio/mp4"),
                    "opus" => Some("audio/opus"),
                    
                    // 文档格式
                    "pdf" => Some("application/pdf"),
                    "doc" => Some("application/msword"),
                    "docx" => Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
                    "xls" => Some("application/vnd.ms-excel"),
                    "xlsx" => Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
                    "ppt" => Some("application/vnd.ms-powerpoint"),
                    "pptx" => Some("application/vnd.openxmlformats-officedocument.presentationml.presentation"),
                    "rtf" => Some("application/rtf"),
                    
                    // 压缩文件
                    "zip" => Some("application/zip"),
                    "rar" => Some("application/vnd.rar"),
                    "7z" => Some("application/x-7z-compressed"),
                    "tar" => Some("application/x-tar"),
                    "gz" => Some("application/gzip"),
                    "bz2" => Some("application/x-bzip2"),
                    
                    // 其他常见格式
                    "bin" => Some("application/octet-stream"),
                    "exe" => Some("application/octet-stream"),
                    "dmg" => Some("application/x-apple-diskimage"),
                    "iso" => Some("application/x-iso9660-image"),
                    
                    _ => None,
                })
        });
        
        self.put_object(key, data, content_type).await
    }

    /// 获取对象
    pub async fn get_object(&self, key: &str) -> Result<GetObjectResponse> {
        let params = HashMap::new();
        let response = self.client.get(&format!("/{}", key), params).await?;
        
        let content_length = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        
        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        
        let last_modified = response
            .headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        
        let data = response
            .bytes()
            .await
            .map_err(|e| CosError::other(format!("Failed to read response body: {}", e)))?
            .to_vec();
        
        Ok(GetObjectResponse {
            data,
            content_length,
            content_type,
            etag,
            last_modified,
        })
    }

    /// 下载对象到文件
    pub async fn get_object_to_file(&self, key: &str, file_path: &Path) -> Result<()> {
        let response = self.get_object(key).await?;
        
        let mut file = File::create(file_path)
            .await
            .map_err(|e| CosError::other(format!("Failed to create file: {}", e)))?;
        
        file.write_all(&response.data)
            .await
            .map_err(|e| CosError::other(format!("Failed to write file: {}", e)))?;
        
        Ok(())
    }

    /// 删除对象
    pub async fn delete_object(&self, key: &str) -> Result<DeleteObjectResponse> {
        let params = HashMap::new();
        let response = self.client.delete(&format!("/{}", key), params).await?;
        
        Ok(DeleteObjectResponse {
            version_id: response
                .headers()
                .get("x-cos-version-id")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
            delete_marker: response
                .headers()
                .get("x-cos-delete-marker")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(false),
        })
    }

    /// 批量删除对象
    pub async fn delete_objects(&self, keys: &[String]) -> Result<DeleteObjectsResponse> {
        let delete_request = DeleteRequest {
            objects: keys.iter().map(|key| DeleteObject {
                key: key.clone(),
                version_id: None,
            }).collect(),
            quiet: false,
        };
        
        let xml_body = quick_xml::se::to_string(&delete_request)
            .map_err(|e| CosError::other(format!("Failed to serialize delete request: {}", e)))?;
        
        let mut params = HashMap::new();
        params.insert("delete".to_string(), "".to_string());
        
        let response = self.client.post("/", params, Some(xml_body)).await?;
        
        let response_text = response
            .text()
            .await
            .map_err(|e| CosError::other(format!("Failed to read response: {}", e)))?;
        
        let delete_response: DeleteObjectsResponse = quick_xml::de::from_str(&response_text)
            .map_err(|e| CosError::other(format!("Failed to parse delete response: {}", e)))?;
        
        Ok(delete_response)
    }

    /// 获取对象元数据
    pub async fn head_object(&self, key: &str) -> Result<HeadObjectResponse> {
        let params = HashMap::new();
        let response = self.client.head(&format!("/{}", key), params).await?;
        
        let content_length = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        
        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        
        let last_modified = response
            .headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        
        Ok(HeadObjectResponse {
            content_length,
            content_type,
            etag,
            last_modified,
        })
    }

    /// 检查对象是否存在
    pub async fn object_exists(&self, key: &str) -> Result<bool> {
        match self.head_object(key).await {
            Ok(_) => Ok(true),
            Err(CosError::Server { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

/// 上传对象响应
#[derive(Debug, Clone)]
pub struct PutObjectResponse {
    pub etag: String,
    pub version_id: Option<String>,
}

/// 获取对象响应
#[derive(Debug, Clone)]
pub struct GetObjectResponse {
    pub data: Vec<u8>,
    pub content_length: u64,
    pub content_type: String,
    pub etag: String,
    pub last_modified: Option<String>,
}

/// 删除对象响应
#[derive(Debug, Clone)]
pub struct DeleteObjectResponse {
    pub version_id: Option<String>,
    pub delete_marker: bool,
}

/// 获取对象元数据响应
#[derive(Debug, Clone)]
pub struct HeadObjectResponse {
    pub content_length: u64,
    pub content_type: String,
    pub etag: String,
    pub last_modified: Option<String>,
}

/// 批量删除请求
#[derive(Debug, Serialize)]
#[serde(rename = "Delete")]
struct DeleteRequest {
    #[serde(rename = "Object")]
    objects: Vec<DeleteObject>,
    #[serde(rename = "Quiet")]
    quiet: bool,
}

/// 删除对象项
#[derive(Debug, Serialize)]
struct DeleteObject {
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "VersionId", skip_serializing_if = "Option::is_none")]
    version_id: Option<String>,
}

/// 批量删除响应
#[derive(Debug, Deserialize)]
#[serde(rename = "DeleteResult")]
pub struct DeleteObjectsResponse {
    #[serde(rename = "Deleted", default)]
    pub deleted: Vec<DeletedObject>,
    #[serde(rename = "Error", default)]
    pub errors: Vec<DeleteError>,
}

/// 已删除对象
#[derive(Debug, Deserialize)]
pub struct DeletedObject {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "VersionId")]
    pub version_id: Option<String>,
    #[serde(rename = "DeleteMarker")]
    pub delete_marker: Option<bool>,
}

/// 删除错误
#[derive(Debug, Deserialize)]
pub struct DeleteError {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::time::Duration;

    #[tokio::test]
    async fn test_object_operations() {
        let config = Config::new("test_id", "test_key", "ap-beijing", "test-bucket-123")
            .with_timeout(Duration::from_secs(60));
        
        let cos_client = CosClient::new(config).unwrap();
        let object_client = ObjectClient::new(cos_client);
        
        // 测试对象存在性检查
        let exists = object_client.object_exists("test-key").await;
        // 在实际测试中，这里会根据具体情况返回结果
    }
}