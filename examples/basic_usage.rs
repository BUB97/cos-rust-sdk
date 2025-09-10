//! 基本使用示例
//!
//! 这个示例展示了如何使用腾讯云 COS Rust SDK 进行基本的对象存储操作。
//!
//! 运行示例：
//! ```bash
//! cargo run --example basic_usage
//! ```
//!
//! 注意：运行前请设置环境变量：
//! - COS_SECRET_ID: 腾讯云 SecretId
//! - COS_SECRET_KEY: 腾讯云 SecretKey
//! - COS_REGION: 地域，如 ap-beijing
//! - COS_BUCKET: 存储桶名称（包含 APPID）

use cos_rust_sdk::{
    Config, CosClient, ObjectClient, BucketClient,
    BucketAcl, ListObjectsV2Options, CosError
};
use std::env;
use std::time::Duration;

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

    println!("=== 腾讯云 COS Rust SDK 基本使用示例 ===");
    println!("Region: {}", region);
    println!("Bucket: {}", bucket);
    println!();

    // 创建配置
    let config = Config::new(&secret_id, &secret_key, &region, &bucket)
        .with_timeout(Duration::from_secs(30))
        .with_https(true);

    // 创建客户端
    let cos_client = CosClient::new(config)?;
    let object_client = ObjectClient::new(cos_client.clone());
    let bucket_client = BucketClient::new(cos_client);

    // 1. 检查存储桶是否存在
    println!("1. 检查存储桶是否存在...");
    match bucket_client.bucket_exists().await {
        Ok(exists) => {
            if exists {
                println!("   ✅ 存储桶存在");
            } else {
                println!("   ❌ 存储桶不存在");
                return Ok(());
            }
        }
        Err(e) => {
            println!("   ❌ 检查存储桶失败: {}", e);
            return Err(e.into());
        }
    }
    println!();

    // 2. 上传对象
    println!("2. 上传对象...");
    let test_content = format!("Hello, COS! 当前时间: {}", chrono::Utc::now());
    let test_key = "rust-sdk-test.txt";
    
    match object_client
        .put_object(test_key, test_content.as_bytes().to_vec(), Some("text/plain; charset=utf-8"))
        .await
    {
        Ok(response) => {
            println!("   ✅ 上传成功");
            println!("   ETag: {}", response.etag);
            if let Some(version_id) = response.version_id {
                println!("   Version ID: {}", version_id);
            }
        }
        Err(e) => {
            println!("   ❌ 上传失败: {}", e);
            return Err(e.into());
        }
    }
    println!();

    // 3. 检查对象是否存在
    println!("3. 检查对象是否存在...");
    match object_client.object_exists(test_key).await {
        Ok(exists) => {
            if exists {
                println!("   ✅ 对象存在");
            } else {
                println!("   ❌ 对象不存在");
            }
        }
        Err(e) => {
            println!("   ❌ 检查对象失败: {}", e);
        }
    }
    println!();

    // 4. 获取对象元数据
    println!("4. 获取对象元数据...");
    match object_client.head_object(test_key).await {
        Ok(response) => {
            println!("   ✅ 获取元数据成功");
            println!("   大小: {} 字节", response.content_length);
            println!("   内容类型: {}", response.content_type);
            println!("   ETag: {}", response.etag);
            if let Some(last_modified) = response.last_modified {
                println!("   最后修改时间: {}", last_modified);
            }
        }
        Err(e) => {
            println!("   ❌ 获取元数据失败: {}", e);
        }
    }
    println!();

    // 5. 下载对象
    println!("5. 下载对象...");
    match object_client.get_object(test_key).await {
        Ok(response) => {
            println!("   ✅ 下载成功");
            println!("   内容: {}", String::from_utf8_lossy(&response.data));
            println!("   大小: {} 字节", response.content_length);
        }
        Err(e) => {
            println!("   ❌ 下载失败: {}", e);
        }
    }
    println!();

    // 6. 列出存储桶中的对象
    println!("6. 列出存储桶中的对象（最多10个）...");
    let list_options = ListObjectsV2Options {
        max_keys: Some(10),
        ..Default::default()
    };
    
    match bucket_client.list_objects_v2(Some(list_options)).await {
        Ok(response) => {
            println!("   ✅ 列出对象成功");
            println!("   对象数量: {}", response.key_count);
            println!("   是否截断: {}", response.is_truncated);
            
            if response.contents.is_empty() {
                println!("   📁 存储桶为空");
            } else {
                println!("   📁 对象列表:");
                for (i, object) in response.contents.iter().enumerate() {
                    println!("   {}. {} ({} 字节) - {}", 
                        i + 1, object.key, object.size, object.last_modified);
                }
            }
        }
        Err(e) => {
            println!("   ❌ 列出对象失败: {}", e);
        }
    }
    println!();

    // 7. 删除对象
    println!("7. 删除测试对象...");
    match object_client.delete_object(test_key).await {
        Ok(response) => {
            println!("   ✅ 删除成功");
            if let Some(version_id) = response.version_id {
                println!("   Version ID: {}", version_id);
            }
            if response.delete_marker {
                println!("   删除标记: true");
            }
        }
        Err(e) => {
            println!("   ❌ 删除失败: {}", e);
        }
    }
    println!();

    // 8. 验证对象已被删除
    println!("8. 验证对象已被删除...");
    match object_client.object_exists(test_key).await {
        Ok(exists) => {
            if !exists {
                println!("   ✅ 对象已成功删除");
            } else {
                println!("   ⚠️  对象仍然存在");
            }
        }
        Err(e) => {
            println!("   ❌ 检查对象失败: {}", e);
        }
    }

    println!();
    println!("=== 示例完成 ===");
    
    Ok(())
}