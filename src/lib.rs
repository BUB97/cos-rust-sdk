//! # 腾讯云 COS Rust SDK
//!
//! 这是一个用于腾讯云对象存储（COS）的 Rust SDK，提供了完整的 COS API 功能。
//!
//! ## 快速开始
//!
//! ```rust
//! use cos_rust_sdk::{Config, CosClient, ObjectClient};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 创建配置
//!     let config = Config::new(
//!         "your-secret-id",
//!         "your-secret-key",
//!         "ap-beijing",
//!         "your-bucket-name-appid"
//!     ).with_timeout(Duration::from_secs(30));
//!
//!     // 创建客户端
//!     let cos_client = CosClient::new(config)?;
//!     let object_client = ObjectClient::new(cos_client.clone());
//!
//!     // 上传文件
//!     let data = b"Hello, COS!";
//!     object_client.put_object("test.txt", data.to_vec(), Some("text/plain")).await?;
//!
//!     // 下载文件
//!     let response = object_client.get_object("test.txt").await?;
//!     println!("Downloaded: {}", String::from_utf8_lossy(&response.data));
//!
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod bucket;
pub mod client;
pub mod config;
pub mod error;
pub mod object;

// 重新导出主要类型
pub use auth::Auth;
pub use bucket::{BucketClient, BucketAcl, ListObjectsOptions, ListObjectsV2Options};
pub use client::CosClient;
pub use config::Config;
pub use error::{CosError, Result};
pub use object::{ObjectClient, PutObjectResponse, GetObjectResponse, DeleteObjectResponse, HeadObjectResponse};

/// SDK 版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 用户代理字符串
pub const USER_AGENT: &str = concat!("cos-rust-sdk/", env!("CARGO_PKG_VERSION"));
