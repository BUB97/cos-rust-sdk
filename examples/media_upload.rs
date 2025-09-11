//! 图片和视频上传下载示例
//!
//! 这个示例展示了如何使用腾讯云 COS Rust SDK 上传和下载图片、视频等多媒体文件。
//!
//! 运行示例：
//! ```bash
//! cargo run --example media_upload
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
    // 从环境变量获取配置
    let secret_id = env::var("COS_SECRET_ID")
        .expect("Please set COS_SECRET_ID environment variable");
    let secret_key = env::var("COS_SECRET_KEY")
        .expect("Please set COS_SECRET_KEY environment variable");
    let region = env::var("COS_REGION")
        .expect("Please set COS_REGION environment variable");
    let bucket = env::var("COS_BUCKET")
        .expect("Please set COS_BUCKET environment variable");

    println!("=== 腾讯云 COS 图片和视频上传下载示例 ===");
    println!("Region: {}", region);
    println!("Bucket: {}", bucket);
    println!();

    // 创建配置
    let config = Config::new(&secret_id, &secret_key, &region, &bucket)
        .with_timeout(Duration::from_secs(60)) // 增加超时时间，适合大文件
        .with_https(true);

    // 创建客户端
    let cos_client = CosClient::new(config)?;
    let object_client = ObjectClient::new(cos_client);

    // 示例1: 上传图片文件
    println!("1. 上传图片文件示例...");
    
    // 创建一个示例图片数据（实际使用中应该是真实的图片文件）
    // let sample_image_data = create_sample_image_data();

    // 示例2: 从本地文件上传图片
    println!("2. 从本地文件上传图片示例...");
    
    let image_key = "images/head_image.jpg";
    
    match object_client
        .put_object_from_file(
            image_key, 
            Path::new("/Users/tennis/Downloads/5eea1b8ee8d3d.jpeg"), 
            None
        )
        .await
    {
        Ok(response) => {
            println!("   ✅ 图片上传成功");
            println!("   文件路径: {}", image_key);
            println!("   ETag: {}", response.etag);
        }
        Err(e) => {
            println!("   ❌ 图片上传失败: {}", e);
        }
    }
    println!();

    // 示例3: 上传视频文件
    println!("3. 上传视频文件示例...");
    
    // let sample_video_data = create_sample_video_data();
    let video_key = "videos/sample.mp4";
    
    match object_client
        .put_object_from_file(video_key, Path::new("/Users/tennis/Documents/screenshot/iShot_2025-09-11_14.30.31.mp4"), Some("video/mp4"))
        .await
    {
        Ok(response) => {
            println!("   ✅ 视频上传成功");
            println!("   文件路径: {}", video_key);
            println!("   ETag: {}", response.etag);
        }
        Err(e) => {
            println!("   ❌ 视频上传失败: {}", e);
        }
    }
    println!();

    // 示例4: 下载图片文件
    println!("4. 下载图片文件示例...");
    
    match object_client.get_object(image_key).await {
        Ok(response) => {
            println!("   ✅ 图片下载成功");
            println!("   文件大小: {} 字节", response.content_length);
            println!("   内容类型: {}", response.content_type);
            println!("   ETag: {}", response.etag);
            
            // 保存到本地文件
            let download_path = "downloaded_image.jpg";
            if let Err(e) = fs::write(download_path, &response.data).await {
                println!("   ⚠️  保存文件失败: {}", e);
            } else {
                println!("   ✅ 图片已保存到: {}", download_path);
            }
        }
        Err(e) => {
            println!("   ❌ 图片下载失败: {}", e);
        }
    }
    println!();

    // 示例5: 下载视频到文件
    println!("5. 下载视频到文件示例...");
    
    let download_video_path = "downloaded_video.mp4";
    match object_client
        .get_object_to_file(video_key, Path::new(download_video_path))
        .await
    {
        Ok(_) => {
            println!("   ✅ 视频下载成功");
            println!("   保存路径: {}", download_video_path);
        }
        Err(e) => {
            println!("   ❌ 视频下载失败: {}", e);
        }
    }
    println!();

    // 示例6: 获取媒体文件元数据
    println!("6. 获取媒体文件元数据示例...");
    
    for (key, file_type) in [(image_key, "图片"), (video_key, "视频")] {
        match object_client.head_object(key).await {
            Ok(response) => {
                println!("   ✅ {}元数据获取成功: {}", file_type, key);
                println!("      大小: {} 字节 ({:.2} MB)", 
                    response.content_length, 
                    response.content_length as f64 / 1024.0 / 1024.0);
                println!("      内容类型: {}", response.content_type);
                println!("      ETag: {}", response.etag);
                if let Some(last_modified) = response.last_modified {
                    println!("      最后修改时间: {}", last_modified);
                }
            }
            Err(e) => {
                println!("   ❌ {}元数据获取失败: {}", file_type, e);
            }
        }
        println!();
    }

    // 示例7: 批量上传多个图片
    println!("7. 批量上传多个图片示例...");
    
    // let image_files = vec![
    //     ("images/batch/photo1.jpg", "image/jpeg"),
    //     ("images/batch/photo2.png", "image/png"),
    //     ("images/batch/photo3.gif", "image/gif"),
    // ];
    
    // for (key, content_type) in image_files {
    //     let data = create_sample_image_data();
    //     match object_client.put_object(key, data, Some(content_type)).await {
    //         Ok(response) => {
    //             println!("   ✅ 批量上传成功: {} (ETag: {})", key, response.etag);
    //         }
    //         Err(e) => {
    //             println!("   ❌ 批量上传失败: {} - {}", key, e);
    //         }
    //     }
    // }
    // println!();

    println!("=== 示例完成 ===");
    println!("注意：请及时清理测试文件以避免产生不必要的存储费用。");
    
    Ok(())
}
