//! 腾讯云 STS 临时密钥示例
//!
//! 本示例展示如何：
//! 1. 获取临时密钥
//! 2. 将临时密钥返回给前端使用
//! 3. 演示不同的权限策略配置
//!
//! 运行示例：
//! ```bash
//! # 设置环境变量
//! export TENCENT_SECRET_ID="your-secret-id"
//! export TENCENT_SECRET_KEY="your-secret-key"
//! export COS_REGION="ap-beijing"
//! export COS_BUCKET="your-bucket-name-appid"
//!
//! # 运行示例
//! cargo run --example sts_example
//! ```

use cos_rust_sdk::sts::{StsClient, GetCredentialsRequest, Policy};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量获取配置
    let secret_id = env::var("COS_SECRET_ID")
        .expect("请设置环境变量 COS_SECRET_ID");
    let secret_key = env::var("COS_SECRET_KEY")
        .expect("请设置环境变量 COS_SECRET_KEY");
    let region = env::var("COS_REGION")
        .unwrap_or_else(|_| "ap-beijing".to_string());
    let bucket = env::var("COS_BUCKET")
        .expect("请设置环境变量 COS_BUCKET");

    // 创建 STS 客户端
    let sts_client = StsClient::new(secret_id, secret_key, region);

    println!("=== STS Policy 使用示例 ===");
    println!("存储桶: {}", bucket);
    println!();

    // 示例 1: 仅允许上传到 uploads/ 前缀
    println!("1. 仅允许上传权限 (uploads/ 前缀)");
    let upload_policy = Policy::allow_put_object(&bucket, Some("uploads/"));
    let request = GetCredentialsRequest {
        name: Some("upload-only-credentials".to_string()),
        policy: upload_policy,
        duration_seconds: Some(1800), // 30分钟
    };
    
    match sts_client.get_credentials(request).await {
        Ok(credentials) => {
            println!("  ✅ 获取上传凭证成功");
            println!("  临时 SecretId: {}...", &credentials.tmp_secret_id[..10]);
            println!("  SessionToken: {}...", &credentials.token[..20]);
        }
        Err(e) => println!("  ❌ 获取上传凭证失败: {}", e),
    }
    println!();

    // 示例 2: 仅允许下载 public/ 前缀的文件
    println!("2. 仅允许下载权限 (public/ 前缀)");
    let download_policy = Policy::allow_get_object(&bucket, Some("public/"));
    let request = GetCredentialsRequest {
        name: Some("download-only-credentials".to_string()),
        policy: download_policy,
        duration_seconds: Some(3600), // 1小时
    };
    
    match sts_client.get_credentials(request).await {
        Ok(credentials) => {
            println!("  ✅ 获取下载凭证成功");
            println!("  临时 SecretId: {}...", &credentials.tmp_secret_id[..10]);
            println!("  SessionToken: {}...", &credentials.token[..20]);
        }
        Err(e) => println!("  ❌ 获取下载凭证失败: {}", e),
    }
    println!();

    // 示例 3: 仅允许删除 temp/ 前缀的文件
    println!("3. 仅允许删除权限 (temp/ 前缀)");
    let delete_policy = Policy::allow_delete_object(&bucket, Some("temp/"));
    let request = GetCredentialsRequest {
        name: Some("delete-only-credentials".to_string()),
        policy: delete_policy,
        duration_seconds: Some(900), // 15分钟
    };
    
    match sts_client.get_credentials(request).await {
        Ok(credentials) => {
            println!("  ✅ 获取删除凭证成功");
            println!("  临时 SecretId: {}...", &credentials.tmp_secret_id[..10]);
            println!("  SessionToken: {}...", &credentials.token[..20]);
        }
        Err(e) => println!("  ❌ 获取删除凭证失败: {}", e),
    }
    println!();

    // 示例 4: 允许读写 media/ 前缀的文件
    println!("4. 允许读写权限 (media/ 前缀)");
    let readwrite_policy = Policy::allow_read_write(&bucket, Some("media/"));
    let request = GetCredentialsRequest {
        name: Some("readwrite-credentials".to_string()),
        policy: readwrite_policy,
        duration_seconds: Some(7200), // 2小时
    };
    
    match sts_client.get_credentials(request).await {
        Ok(credentials) => {
            println!("  ✅ 获取读写凭证成功");
            println!("  临时 SecretId: {}...", &credentials.tmp_secret_id[..10]);
            println!("  SessionToken: {}...", &credentials.token[..20]);
            println!("  过期时间: {:?}", credentials.expired_time);
        }
        Err(e) => println!("  ❌ 获取读写凭证失败: {}", e),
    }
    println!();

    // 示例 5: 允许读写整个存储桶
    println!("5. 允许读写整个存储桶 (无前缀限制)");
    let full_policy = Policy::allow_read_write(&bucket, None);
    let request = GetCredentialsRequest {
        name: Some("full-access-credentials".to_string()),
        policy: full_policy,
        duration_seconds: Some(3600), // 1小时
    };
    
    match sts_client.get_credentials(request).await {
        Ok(credentials) => {
            println!("  ✅ 获取完整权限凭证成功");
            println!("  临时 SecretId: {}...", &credentials.tmp_secret_id[..10]);
            println!("  SessionToken: {}...", &credentials.token[..20]);
        }
        Err(e) => println!("  ❌ 获取完整权限凭证失败: {}", e),
    }
    println!();

    println!("=== 策略使用建议 ===");
    println!("• 前端文件上传: 使用 allow_put_object，限制上传目录");
    println!("• 公共资源访问: 使用 allow_get_object，限制下载目录");
    println!("• 临时文件清理: 使用 allow_delete_object，限制删除目录");
    println!("• 完整文件管理: 使用 allow_read_write，根据需要限制前缀");
    println!("• 最小权限原则: 总是使用最小必要权限，设置合适的前缀和过期时间");

    Ok(())
}