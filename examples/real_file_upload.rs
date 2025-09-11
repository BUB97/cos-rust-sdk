//! 真实文件上传示例
//!
//! 这个示例展示了如何上传真实的图片和视频文件到腾讯云 COS。
//! 与 media_upload.rs 不同，这个示例处理真实的本地文件。
//!
//! 运行示例：
//! ```bash
//! cargo run --example real_file_upload -- /path/to/your/image.jpg
//! ```
//!
//! 或者批量上传：
//! ```bash
//! cargo run --example real_file_upload -- /path/to/file1.jpg /path/to/file2.mp4
//! ```
//!
//! 注意：运行前请设置环境变量：
//! - COS_SECRET_ID: 腾讯云 SecretId
//! - COS_SECRET_KEY: 腾讯云 SecretKey
//! - COS_REGION: 地域，如 ap-beijing
//! - COS_BUCKET: 存储桶名称（包含 APPID）

use cos_rust_sdk::{Config, CosClient, ObjectClient};
use std::env;
use std::path::Path;
use std::time::Duration;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("❌ 使用方法:");
        println!("   cargo run --example real_file_upload -- <文件路径1> [文件路径2] ...");
        println!();
        println!("📝 示例:");
        println!("   cargo run --example real_file_upload -- ./image.jpg");
        println!("   cargo run --example real_file_upload -- ./photo.png ./video.mp4");
        println!();
        println!("💡 提示:");
        println!("   - 支持图片格式：JPG, PNG, GIF, WebP, BMP, TIFF, SVG 等");
        println!("   - 支持视频格式：MP4, AVI, MOV, WMV, FLV, WebM, MKV 等");
        println!("   - 支持音频格式：MP3, WAV, FLAC, AAC, OGG 等");
        println!("   - 文件路径可以是相对路径或绝对路径");
        return Ok(());
    }

    // 从环境变量获取配置
    let secret_id = env::var("COS_SECRET_ID")
        .expect("❌ 请设置 COS_SECRET_ID 环境变量");
    let secret_key = env::var("COS_SECRET_KEY")
        .expect("❌ 请设置 COS_SECRET_KEY 环境变量");
    let region = env::var("COS_REGION")
        .expect("❌ 请设置 COS_REGION 环境变量");
    let bucket = env::var("COS_BUCKET")
        .expect("❌ 请设置 COS_BUCKET 环境变量");

    println!("🚀 腾讯云 COS 真实文件上传示例");
    println!("📍 Region: {}", region);
    println!("🪣 Bucket: {}", bucket);
    println!();

    // 创建配置
    let config = Config::new(&secret_id, &secret_key, &region, &bucket)
        .with_timeout(Duration::from_secs(300)) // 5分钟超时，适合大文件
        .with_https(true);

    // 创建客户端
    let cos_client = CosClient::new(config)?;
    let object_client = ObjectClient::new(cos_client);

    // 获取文件路径列表（跳过程序名）
    let file_paths = &args[1..];
    
    println!("📁 准备上传 {} 个文件:", file_paths.len());
    for (i, path) in file_paths.iter().enumerate() {
        println!("   {}. {}", i + 1, path);
    }
    println!();

    let mut success_count = 0;
    let mut failed_count = 0;

    // 逐个处理文件
    for (index, file_path) in file_paths.iter().enumerate() {
        let path = Path::new(file_path);
        
        println!("📤 [{}/{}] 正在上传: {}", index + 1, file_paths.len(), file_path);
        
        // 检查文件是否存在
        if !path.exists() {
            println!("   ❌ 文件不存在: {}", file_path);
            failed_count += 1;
            println!();
            continue;
        }
        
        // 检查是否为文件（不是目录）
        if !path.is_file() {
            println!("   ❌ 不是文件: {}", file_path);
            failed_count += 1;
            println!();
            continue;
        }
        
        // 获取文件信息
        match fs::metadata(path).await {
            Ok(metadata) => {
                let file_size = metadata.len();
                println!("   📊 文件大小: {} 字节 ({:.2} MB)", 
                    file_size, 
                    file_size as f64 / 1024.0 / 1024.0);
                
                // 对于大文件给出提示
                if file_size > 100 * 1024 * 1024 { // 100MB
                    println!("   ⚠️  大文件上传，请耐心等待...");
                }
            }
            Err(e) => {
                println!("   ❌ 无法获取文件信息: {}", e);
                failed_count += 1;
                println!();
                continue;
            }
        }
        
        // 生成 COS 对象键
        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        
        let file_extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        // 根据文件类型分类存储
        let cos_key = match file_extension.to_lowercase().as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tiff" | "tif" | "svg" | "ico" | "heic" | "heif" | "avif" | "jxl" => {
                format!("images/{}", file_name)
            }
            "mp4" | "avi" | "mov" | "wmv" | "flv" | "webm" | "mkv" | "m4v" | "3gp" | "3g2" | "ts" | "mts" | "m2ts" | "ogv" => {
                format!("videos/{}", file_name)
            }
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" | "opus" => {
                format!("audio/{}", file_name)
            }
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "rtf" => {
                format!("documents/{}", file_name)
            }
            _ => {
                format!("files/{}", file_name)
            }
        };
        
        println!("   🎯 COS 路径: {}", cos_key);
        
        // 上传文件
        let start_time = std::time::Instant::now();
        match object_client.put_object_from_file(&cos_key, path, None).await {
            Ok(response) => {
                let duration = start_time.elapsed();
                println!("   ✅ 上传成功!");
                println!("   📝 ETag: {}", response.etag);
                println!("   ⏱️  耗时: {:.2} 秒", duration.as_secs_f64());
                
                // 验证上传结果
                match object_client.head_object(&cos_key).await {
                    Ok(metadata) => {
                        println!("   ✅ 验证成功 - 文件大小: {} 字节, 类型: {}", 
                            metadata.content_length, 
                            metadata.content_type);
                    }
                    Err(e) => {
                        println!("   ⚠️  验证失败: {}", e);
                    }
                }
                
                success_count += 1;
            }
            Err(e) => {
                let duration = start_time.elapsed();
                println!("   ❌ 上传失败: {}", e);
                println!("   ⏱️  耗时: {:.2} 秒", duration.as_secs_f64());
                failed_count += 1;
                
                // 提供错误处理建议
                if e.to_string().contains("timeout") {
                    println!("   💡 建议: 文件可能过大，请尝试增加超时时间或检查网络连接");
                } else if e.to_string().contains("permission") || e.to_string().contains("access") {
                    println!("   💡 建议: 请检查 COS 访问权限和存储桶配置");
                } else if e.to_string().contains("network") || e.to_string().contains("connection") {
                    println!("   💡 建议: 请检查网络连接");
                }
            }
        }
        
        println!();
    }

    // 输出总结
    println!("📊 上传总结:");
    println!("   ✅ 成功: {} 个文件", success_count);
    println!("   ❌ 失败: {} 个文件", failed_count);
    println!("   📁 总计: {} 个文件", file_paths.len());
    
    if success_count > 0 {
        println!();
        println!("🎉 上传完成！您可以在腾讯云 COS 控制台查看上传的文件。");
        println!("💰 注意：请及时清理不需要的文件以避免产生存储费用。");
    }
    
    if failed_count > 0 {
        println!();
        println!("⚠️  部分文件上传失败，请检查:");
        println!("   1. 文件路径是否正确");
        println!("   2. 文件是否存在且可读");
        println!("   3. 网络连接是否正常");
        println!("   4. COS 配置是否正确");
        println!("   5. 存储桶权限是否足够");
    }
    
    Ok(())
}