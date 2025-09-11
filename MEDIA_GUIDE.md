# 图片和视频上传下载指南

本指南详细介绍如何使用腾讯云 COS Rust SDK 上传和下载图片、视频等多媒体文件。

## 快速开始

### 1. 环境配置

首先设置必要的环境变量：

```bash
export COS_SECRET_ID="your-secret-id"
export COS_SECRET_KEY="your-secret-key"
export COS_REGION="ap-beijing"  # 或其他地域
export COS_BUCKET="your-bucket-name-appid"
```

### 2. 基本依赖

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
cos-rust-sdk = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## 图片上传和下载

### 上传图片

#### 方法1：从内存数据上传

```rust
use cos_rust_sdk::{Config, CosClient, ObjectClient};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建配置
    let config = Config::new(
        &std::env::var("COS_SECRET_ID")?,
        &std::env::var("COS_SECRET_KEY")?,
        &std::env::var("COS_REGION")?,
        &std::env::var("COS_BUCKET")?
    ).with_timeout(Duration::from_secs(30));

    // 创建客户端
    let cos_client = CosClient::new(config)?;
    let object_client = ObjectClient::new(cos_client);

    // 读取图片数据（示例）
    let image_data = std::fs::read("local_image.jpg")?;
    
    // 上传图片
    let response = object_client
        .put_object("images/uploaded_image.jpg", image_data, Some("image/jpeg"))
        .await?;
    
    println!("上传成功，ETag: {}", response.etag);
    Ok(())
}
```

#### 方法2：直接从文件上传

```rust
use std::path::Path;

// 直接从本地文件上传
let response = object_client
    .put_object_from_file(
        "images/photo.jpg", 
        Path::new("local_photo.jpg"), 
        Some("image/jpeg")
    )
    .await?;

println!("文件上传成功，ETag: {}", response.etag);
```

### 下载图片

#### 方法1：下载到内存

```rust
// 下载图片到内存
let response = object_client.get_object("images/photo.jpg").await?;

println!("图片大小: {} 字节", response.content_length);
println!("内容类型: {}", response.content_type);

// 保存到本地文件
std::fs::write("downloaded_photo.jpg", &response.data)?;
```

#### 方法2：直接下载到文件

```rust
use std::path::Path;

// 直接下载到文件
object_client
    .get_object_to_file("images/photo.jpg", Path::new("downloaded_photo.jpg"))
    .await?;

println!("图片下载完成");
```

## 视频上传和下载

### 上传视频

```rust
// 上传大视频文件时，建议增加超时时间
let config = Config::new(
    &std::env::var("COS_SECRET_ID")?,
    &std::env::var("COS_SECRET_KEY")?,
    &std::env::var("COS_REGION")?,
    &std::env::var("COS_BUCKET")?
).with_timeout(Duration::from_secs(300)); // 5分钟超时

let cos_client = CosClient::new(config)?;
let object_client = ObjectClient::new(cos_client);

// 从文件上传视频
let response = object_client
    .put_object_from_file(
        "videos/my_video.mp4", 
        Path::new("local_video.mp4"), 
        Some("video/mp4")
    )
    .await?;

println!("视频上传成功，ETag: {}", response.etag);
```

### 下载视频

```rust
// 下载视频文件
object_client
    .get_object_to_file("videos/my_video.mp4", Path::new("downloaded_video.mp4"))
    .await?;

println!("视频下载完成");
```

## 支持的媒体格式

### 图片格式

| 格式 | MIME 类型 | 扩展名 | 说明 |
|------|-----------|--------|------|
| JPEG | image/jpeg | .jpg, .jpeg | 最常用的有损压缩图片格式 |
| PNG | image/png | .png | 支持透明度的无损压缩格式 |
| GIF | image/gif | .gif | 支持动画的图片格式 |
| WebP | image/webp | .webp | Google 开发的现代图片格式 |
| BMP | image/bmp | .bmp | Windows 位图格式 |
| TIFF | image/tiff | .tiff, .tif | 高质量的无损压缩格式 |
| SVG | image/svg+xml | .svg | 可缩放矢量图形 |
| ICO | image/x-icon | .ico | Windows 图标格式 |
| HEIC | image/heic | .heic | Apple 高效图像格式 |
| HEIF | image/heif | .heif | 高效图像文件格式 |
| AVIF | image/avif | .avif | AV1 图像文件格式 |
| JXL | image/jxl | .jxl | JPEG XL 下一代图像格式 |

### 视频格式

| 格式 | MIME 类型 | 扩展名 | 说明 |
|------|-----------|--------|------|
| MP4 | video/mp4 | .mp4 | 最常用的视频容器格式 |
| AVI | video/x-msvideo | .avi | Microsoft 视频格式 |
| MOV | video/quicktime | .mov | Apple QuickTime 格式 |
| WMV | video/x-ms-wmv | .wmv | Windows Media Video |
| FLV | video/x-flv | .flv | Flash 视频格式 |
| WebM | video/webm | .webm | Google 开发的开源格式 |
| MKV | video/x-matroska | .mkv | Matroska 多媒体容器 |
| M4V | video/x-m4v | .m4v | iTunes 视频格式 |
| 3GP | video/3gpp | .3gp | 3G 移动设备视频格式 |
| 3G2 | video/3gpp2 | .3g2 | 3G2 移动设备视频格式 |
| TS | video/mp2t | .ts, .mts, .m2ts | MPEG-2 传输流 |
| OGV | video/ogg | .ogv | Ogg 视频格式 |

### 音频格式

| 格式 | MIME 类型 | 扩展名 | 说明 |
|------|-----------|--------|------|
| MP3 | audio/mpeg | .mp3 | 最常用的有损音频格式 |
| WAV | audio/wav | .wav | 无损音频格式 |
| FLAC | audio/flac | .flac | 自由无损音频编解码器 |
| AAC | audio/aac | .aac | 高级音频编码 |
| OGG | audio/ogg | .ogg | Ogg Vorbis 音频格式 |
| WMA | audio/x-ms-wma | .wma | Windows Media Audio |
| M4A | audio/mp4 | .m4a | MPEG-4 音频格式 |
| Opus | audio/opus | .opus | 现代低延迟音频编解码器 |

## 最佳实践

### 1. 文件命名规范

```rust
// 推荐的文件路径结构
let image_key = format!("images/{}/{}.jpg", 
    chrono::Utc::now().format("%Y/%m/%d"), 
    uuid::Uuid::new_v4());

let video_key = format!("videos/{}/{}.mp4", 
    chrono::Utc::now().format("%Y/%m"), 
    uuid::Uuid::new_v4());
```

### 2. 大文件上传优化

```rust
// 对于大文件，增加超时时间
let config = Config::new(
    secret_id, secret_key, region, bucket
).with_timeout(Duration::from_secs(600)); // 10分钟

// 检查文件大小
let file_size = std::fs::metadata("large_video.mp4")?.len();
if file_size > 100 * 1024 * 1024 { // 100MB
    println!("警告：文件较大，上传可能需要较长时间");
}
```

### 3. 错误处理

```rust
match object_client.put_object_from_file(key, path, content_type).await {
    Ok(response) => {
        println!("上传成功: {}", response.etag);
    }
    Err(CosError::NetworkError(e)) => {
        eprintln!("网络错误，请检查网络连接: {}", e);
    }
    Err(CosError::AuthError(e)) => {
        eprintln!("认证错误，请检查密钥配置: {}", e);
    }
    Err(e) => {
        eprintln!("其他错误: {}", e);
    }
}
```

### 4. 批量操作

```rust
// 批量上传图片
let image_files = vec![
    ("photo1.jpg", "images/2024/01/photo1.jpg"),
    ("photo2.png", "images/2024/01/photo2.png"),
    ("photo3.gif", "images/2024/01/photo3.gif"),
];

for (local_path, remote_key) in image_files {
    match object_client
        .put_object_from_file(remote_key, Path::new(local_path), None)
        .await
    {
        Ok(response) => println!("✅ {} 上传成功", local_path),
        Err(e) => eprintln!("❌ {} 上传失败: {}", local_path, e),
    }
}
```

### 5. 获取文件信息

```rust
// 检查文件是否存在
if object_client.object_exists("images/photo.jpg").await? {
    // 获取文件元数据
    let metadata = object_client.head_object("images/photo.jpg").await?;
    println!("文件大小: {} 字节", metadata.content_length);
    println!("内容类型: {}", metadata.content_type);
    println!("最后修改: {:?}", metadata.last_modified);
} else {
    println!("文件不存在");
}
```

## 运行示例

### 模拟数据示例
```bash
# 运行基本媒体上传示例（使用模拟数据）
cargo run --example media_upload

# 运行快速开始示例
cargo run --example quick_start_media

# 运行格式支持演示
cargo run --example format_support
```

### 真实文件上传示例
```bash
# 上传单个文件
cargo run --example real_file_upload -- /path/to/your/image.jpg

# 上传多个文件
cargo run --example real_file_upload -- ./photo1.jpg ./video.mp4 ./audio.mp3

# 上传不同格式的文件
cargo run --example real_file_upload -- ./document.pdf ./archive.zip
```

## 注意事项

1. **文件大小限制**：单个文件最大支持 5TB
2. **并发限制**：建议控制并发上传数量，避免超出 API 限制
3. **网络超时**：大文件上传时适当增加超时时间
4. **存储费用**：及时清理不需要的测试文件
5. **安全性**：不要在代码中硬编码密钥，使用环境变量或配置文件

## 故障排除

### 文件不存在错误

**错误信息**: `Failed to open file: No such file or directory (os error 2)`

**原因**: 指定的文件路径不存在或无法访问

**解决方案**:
1. **检查文件路径**
   ```bash
   # 使用绝对路径
   cargo run --example real_file_upload -- /Users/username/Pictures/image.jpg
   
   # 使用相对路径（确保文件在当前目录）
   cargo run --example real_file_upload -- ./image.jpg
   ```

2. **验证文件存在**
   ```bash
   # 检查文件是否存在
   ls -la /path/to/your/file.jpg
   
   # 检查当前目录的文件
   ls -la *.jpg *.png *.mp4
   ```

3. **使用模拟数据示例**
   ```bash
   # 如果没有真实文件，可以使用模拟数据示例
   cargo run --example media_upload
   ```

### 常见问题

1. **上传失败**
   - 检查网络连接
   - 验证存储桶权限
   - 确认文件大小限制
   - 确保文件路径正确且文件存在

2. **超时错误**
   - 增加超时时间（特别是大文件）
   - 检查文件大小
   - 优化网络环境
   - 使用 `real_file_upload` 示例（已设置5分钟超时）

3. **权限错误**
   - 检查 SecretId 和 SecretKey
   - 验证存储桶 ACL 设置
   - 确认 CORS 配置

4. **文件路径问题**
   - 使用绝对路径避免相对路径混淆
   - 检查文件名中的特殊字符
   - 确保文件不是目录
   - 验证文件读取权限

### 常见错误

1. **认证失败**：检查 SecretId 和 SecretKey 是否正确
2. **存储桶不存在**：确认存储桶名称包含 APPID
3. **网络超时**：增加超时时间或检查网络连接
4. **权限不足**：确认账号有相应的 COS 操作权限

### 调试技巧

1. **查看详细错误信息**
   ```bash
   RUST_LOG=debug cargo run --example real_file_upload -- ./file.jpg
   ```

2. **测试文件访问**
   ```bash
   # 测试文件是否可读
   cat /path/to/file.jpg > /dev/null && echo "文件可读" || echo "文件不可读"
   ```

3. **检查文件类型**
   ```bash
   file /path/to/file.jpg
   ```

```rust
// 启用日志
env_logger::init();

// 检查配置
println!("Region: {}", config.region);
println!("Bucket: {}", config.bucket);

// 测试连接
match bucket_client.bucket_exists().await {
    Ok(true) => println!("存储桶连接正常"),
    Ok(false) => println!("存储桶不存在"),
    Err(e) => println!("连接失败: {}", e),
}
```

通过以上指南，您应该能够成功使用腾讯云 COS Rust SDK 进行图片和视频的上传下载操作。如有问题，请参考 SDK 文档或提交 Issue。