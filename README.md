# 腾讯云 COS Rust SDK

[![Crates.io](https://img.shields.io/crates/v/cos-rust-sdk.svg)](https://crates.io/crates/cos-rust-sdk)
[![Documentation](https://docs.rs/cos-rust-sdk/badge.svg)](https://docs.rs/cos-rust-sdk)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

这是一个用于腾讯云对象存储（COS）的 Rust SDK，提供了完整的 COS API 功能。

## 特性

- ✅ 完整的对象存储操作（上传、下载、删除等）
- ✅ 存储桶管理功能
- ✅ 腾讯云签名算法实现
- ✅ 异步支持（基于 tokio）
- ✅ 类型安全的 API
- ✅ 详细的错误处理
- ✅ 支持自定义域名
- ✅ 支持 HTTPS/HTTP

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
cos-rust-sdk = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## 快速开始

### 基本配置

```rust
use cos_rust_sdk::{Config, CosClient, ObjectClient};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建配置
    let config = Config::new(
        "your-secret-id",
        "your-secret-key",
        "ap-beijing",  // 地域
        "your-bucket-name-1234567890"  // 存储桶名称（包含 APPID）
    )
    .with_timeout(Duration::from_secs(30))
    .with_https(true);

    // 创建客户端
    let cos_client = CosClient::new(config)?;
    let object_client = ObjectClient::new(cos_client.clone());

    Ok(())
}
```

### 对象操作

#### 上传对象

```rust
// 上传字节数据
let data = b"Hello, COS!";
let response = object_client
    .put_object("test.txt", data.to_vec(), Some("text/plain"))
    .await?;
println!("ETag: {}", response.etag);

// 从文件上传
use std::path::Path;
let response = object_client
    .put_object_from_file("remote-file.jpg", Path::new("local-file.jpg"), None)
    .await?;
```

#### 下载对象

```rust
// 下载到内存
let response = object_client.get_object("test.txt").await?;
println!("Content: {}", String::from_utf8_lossy(&response.data));
println!("Content-Type: {}", response.content_type);

// 下载到文件
use std::path::Path;
object_client
    .get_object_to_file("remote-file.jpg", Path::new("downloaded-file.jpg"))
    .await?;
```

#### 删除对象

```rust
// 删除单个对象
let response = object_client.delete_object("test.txt").await?;

// 批量删除对象
let keys = vec!["file1.txt".to_string(), "file2.txt".to_string()];
let response = object_client.delete_objects(&keys).await?;
for deleted in response.deleted {
    println!("Deleted: {}", deleted.key);
}
```

#### 获取对象元数据

```rust
// 获取对象元数据
let response = object_client.head_object("test.txt").await?;
println!("Size: {} bytes", response.content_length);
println!("Last Modified: {:?}", response.last_modified);

// 检查对象是否存在
let exists = object_client.object_exists("test.txt").await?;
println!("Object exists: {}", exists);
```

### 存储桶操作

```rust
use cos_rust_sdk::{BucketClient, BucketAcl, ListObjectsV2Options};

let bucket_client = BucketClient::new(cos_client.clone());

// 检查存储桶是否存在
let exists = bucket_client.bucket_exists().await?;

// 列出对象
let options = ListObjectsV2Options {
    prefix: Some("photos/".to_string()),
    max_keys: Some(100),
    ..Default::default()
};
let response = bucket_client.list_objects_v2(Some(options)).await?;
for object in response.contents {
    println!("Key: {}, Size: {}", object.key, object.size);
}

// 设置存储桶 ACL
bucket_client.put_bucket_acl(BucketAcl::PublicRead).await?;
```

## 配置选项

### 基本配置

```rust
let config = Config::new(secret_id, secret_key, region, bucket)
    .with_timeout(Duration::from_secs(60))  // 请求超时时间
    .with_https(true)                       // 使用 HTTPS
    .with_domain("custom.domain.com");     // 自定义域名
```

### 地域列表

常用地域代码：
- `ap-beijing` - 北京
- `ap-shanghai` - 上海
- `ap-guangzhou` - 广州
- `ap-chengdu` - 成都
- `ap-singapore` - 新加坡
- `ap-hongkong` - 香港

## 错误处理

```rust
use cos_rust_sdk::CosError;

match object_client.get_object("nonexistent.txt").await {
    Ok(response) => println!("Success: {} bytes", response.data.len()),
    Err(CosError::Server { message, .. }) => {
        println!("Server error: {}", message);
    }
    Err(CosError::Http { message, .. }) => {
        println!("HTTP error: {}", message);
    }
    Err(e) => println!("Other error: {}", e),
}
```

## 完整示例

```rust
use cos_rust_sdk::{
    Config, CosClient, ObjectClient, BucketClient,
    BucketAcl, ListObjectsV2Options
};
use std::time::Duration;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 配置
    let config = Config::new(
        "your-secret-id",
        "your-secret-key",
        "ap-beijing",
        "your-bucket-1234567890"
    ).with_timeout(Duration::from_secs(30));

    // 创建客户端
    let cos_client = CosClient::new(config)?;
    let object_client = ObjectClient::new(cos_client.clone());
    let bucket_client = BucketClient::new(cos_client);

    // 检查存储桶
    if !bucket_client.bucket_exists().await? {
        println!("Bucket does not exist");
        return Ok(());
    }

    // 上传文件
    let content = b"Hello, Tencent COS!";
    let upload_response = object_client
        .put_object("hello.txt", content.to_vec(), Some("text/plain"))
        .await?;
    println!("Uploaded with ETag: {}", upload_response.etag);

    // 列出对象
    let list_response = bucket_client
        .list_objects_v2(Some(ListObjectsV2Options {
            max_keys: Some(10),
            ..Default::default()
        }))
        .await?;
    
    println!("Objects in bucket:");
    for object in list_response.contents {
        println!("  {}: {} bytes", object.key, object.size);
    }

    // 下载文件
    let download_response = object_client.get_object("hello.txt").await?;
    println!("Downloaded: {}", String::from_utf8_lossy(&download_response.data));

    // 删除文件
    object_client.delete_object("hello.txt").await?;
    println!("File deleted");

    Ok(())
}
```

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！



## 媒体文件处理

### 批量上传图片

```rust
use cos_rust_sdk::{Config, CosClient, ObjectClient};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new(secret_id, secret_key, region, bucket)
        .with_timeout(Duration::from_secs(120)); // 增加超时时间
    
    let cos_client = CosClient::new(config)?;
    let object_client = ObjectClient::new(cos_client);
    
    // 批量上传图片
    let images = vec![
        ("photos/image1.jpg", image1_data),
        ("photos/image2.png", image2_data),
        ("photos/image3.webp", image3_data),
    ];
    
    for (key, data) in images {
        object_client.put_object(key, data, None).await?;
        println!("上传成功: {}", key);
    }
    
    Ok(())
}
```

### 下载并保存图片

```rust
// 下载图片到本地
let local_path = "./downloaded_image.jpg";
object_client.get_object_to_file("photos/image1.jpg", local_path).await?;
println!("图片已保存到: {}", local_path);
```

### 支持的文件格式

SDK 支持自动检测以下文件格式的 MIME 类型：

- **图片格式**：JPEG, PNG, GIF, WebP, BMP, TIFF, SVG, ICO, HEIC, HEIF, AVIF, JXL
- **视频格式**：MP4, AVI, MOV, WMV, FLV, WebM, MKV, M4V, 3GP, 3G2, TS, MTS, M2TS, OGV
- **音频格式**：MP3, WAV, FLAC, AAC, OGG, WMA, M4A, Opus
- **文档格式**：PDF, DOC, DOCX, XLS, XLSX, PPT, PPTX, RTF
- **压缩格式**：ZIP, RAR, 7Z, TAR, GZ, BZ2
- **文本格式**：TXT, HTML, CSS, JS, JSON, XML, CSV, Markdown

详细的媒体文件处理指南请参考：[MEDIA_GUIDE.md](MEDIA_GUIDE.md)

更多示例请查看：
- [examples/media_upload.rs](examples/media_upload.rs) - 媒体文件上传下载示例（使用模拟数据）
- [examples/real_file_upload.rs](examples/real_file_upload.rs) - 真实文件上传示例（处理本地文件）
- [examples/format_support.rs](examples/format_support.rs) - 多格式文件支持演示
- [examples/quick_start_media.rs](examples/quick_start_media.rs) - 快速开始媒体处理

运行示例：
```bash
# 媒体上传示例（使用模拟数据）
cargo run --example media_upload

# 真实文件上传示例
cargo run --example real_file_upload -- /path/to/your/file.jpg
cargo run --example real_file_upload -- ./image.png ./video.mp4

# 格式支持演示
cargo run --example format_support

# 快速开始
cargo run --example quick_start_media
```

## 相关链接

- [腾讯云 COS 官方文档](https://cloud.tencent.com/document/product/436)
- [腾讯云 COS API 文档](https://cloud.tencent.com/document/product/436/7751)