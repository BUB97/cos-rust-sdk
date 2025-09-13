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

use cos_rust_sdk::{
    StsClient, Policy, GetCredentialsRequest, CosError, TemporaryCredentials,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志（如果需要的话）
    // env_logger::init();

    println!("=== 腾讯云 STS 临时密钥示例 ===");

    // 从环境变量获取配置
    let secret_id = env::var("COS_SECRET_ID")
        .map_err(|_| "请设置环境变量 COS_SECRET_ID（腾讯云访问密钥ID）")?;
    let secret_key = env::var("COS_SECRET_KEY")
        .map_err(|_| "请设置环境变量 COS_SECRET_KEY（腾讯云访问密钥Key）")?;
    let region = env::var("COS_REGION")
        .unwrap_or_else(|_| "ap-guangzhou".to_string());
    let bucket = env::var("COS_BUCKET")
        .map_err(|_| "请设置环境变量 COS_BUCKET（存储桶名称，格式：bucket-appid）")?;
    
    // 验证密钥格式
    if secret_id.len() < 10 || secret_key.len() < 10 {
        return Err("密钥格式不正确，请检查 COS_SECRET_ID 和 COS_SECRET_KEY 是否为有效的腾讯云密钥".into());
    }

    println!("配置信息:");
    println!("  区域: {}", region);
    println!("  存储桶: {}", bucket);
    println!();

    // 创建 STS 客户端
    let sts_client = StsClient::new(secret_id, secret_key, region);

    // 示例 1: 获取上传权限的临时密钥
    println!("=== 示例 1: 获取上传权限的临时密钥 ===");
    match get_upload_credentials(&sts_client, &bucket).await {
        Ok(credentials) => {
            println!("✓ 上传临时密钥获取成功");
            print_credentials(&credentials);
            println!("前端可以使用这些临时密钥进行文件上传");
        }
        Err(e) => {
            eprintln!("✗ 获取上传临时密钥失败: {}", e);
        }
    }
    println!();

    // 示例 2: 获取下载权限的临时密钥
    println!("=== 示例 2: 获取下载权限的临时密钥 ===");
    match get_download_credentials(&sts_client, &bucket).await {
        Ok(credentials) => {
            println!("✓ 下载临时密钥获取成功");
            print_credentials(&credentials);
            println!("前端可以使用这些临时密钥进行文件下载");
        }
        Err(e) => {
            eprintln!("✗ 获取下载临时密钥失败: {}", e);
        }
    }
    println!();

    // 示例 3: 获取读写权限的临时密钥
    println!("=== 示例 3: 获取读写权限的临时密钥 ===");
    match get_readwrite_credentials(&sts_client, &bucket).await {
        Ok(credentials) => {
            println!("✓ 读写临时密钥获取成功");
            print_credentials(&credentials);
            println!("前端可以使用这些临时密钥进行文件读写操作");
        }
        Err(e) => {
            eprintln!("✗ 获取读写临时密钥失败: {}", e);
        }
    }
    println!();

    // 示例 4: 获取特定前缀的临时密钥
    println!("=== 示例 4: 获取特定前缀的临时密钥 ===");
    match get_prefix_credentials(&sts_client, &bucket, "user-uploads/").await {
        Ok(credentials) => {
            println!("✓ 前缀限制临时密钥获取成功");
            print_credentials(&credentials);
            println!("前端只能在 user-uploads/ 前缀下进行操作");
        }
        Err(e) => {
            eprintln!("✗ 获取前缀限制临时密钥失败: {}", e);
        }
    }
    println!();

    println!("=== STS 临时密钥示例完成 ===");
    println!("\n使用说明:");
    println!("1. 后端服务调用 STS API 获取临时密钥");
    println!("2. 将临时密钥返回给前端（通过 API 接口）");
    println!("3. 前端使用临时密钥直接访问 COS 服务");
    println!("4. 临时密钥有时效性，过期后需要重新获取");

    Ok(())
}

/// 获取上传权限的临时密钥
async fn get_upload_credentials(
    sts_client: &StsClient,
    bucket: &str,
) -> Result<TemporaryCredentials, CosError> {
    let policy = Policy::allow_put_object(bucket, Some("uploads/"));
    let request = GetCredentialsRequest {
        policy,
        duration_seconds: Some(3600), // 1小时有效期
        name: Some("upload-session".to_string()),
    };
    
    sts_client.get_credentials(request).await
}

/// 获取下载权限的临时密钥
async fn get_download_credentials(
    sts_client: &StsClient,
    bucket: &str,
) -> Result<TemporaryCredentials, CosError> {
    let policy = Policy::allow_get_object(bucket, Some("downloads/"));
    let request = GetCredentialsRequest {
        policy,
        duration_seconds: Some(1800), // 30分钟有效期
        name: Some("download-session".to_string()),
    };
    
    sts_client.get_credentials(request).await
}

/// 获取读写权限的临时密钥
async fn get_readwrite_credentials(
    sts_client: &StsClient,
    bucket: &str,
) -> Result<TemporaryCredentials, CosError> {
    let policy = Policy::allow_read_write(bucket, Some("temp/"));
    let request = GetCredentialsRequest {
        policy,
        duration_seconds: Some(7200), // 2小时有效期
        name: Some("readwrite-session".to_string()),
    };
    
    sts_client.get_credentials(request).await
}

/// 获取特定前缀的临时密钥
async fn get_prefix_credentials(
    sts_client: &StsClient,
    bucket: &str,
    prefix: &str,
) -> Result<TemporaryCredentials, CosError> {
    let policy = Policy::allow_read_write(bucket, Some(prefix));
    let request = GetCredentialsRequest {
        policy,
        duration_seconds: Some(3600), // 1小时有效期
        name: Some(format!("prefix-{}-session", prefix.replace('/', "-"))),
    };
    
    sts_client.get_credentials(request).await
}

/// 打印临时密钥信息
fn print_credentials(credentials: &TemporaryCredentials) {
    println!("  临时访问密钥 ID: {}", &credentials.tmp_secret_id);
    println!("  临时访问密钥: {}", &credentials.tmp_secret_key);
    println!("  安全令牌: {}...", &credentials.token[..20]);
    
    match credentials.expired_time {
        Some(timestamp) => {
            println!("  过期时间戳: {}", timestamp);
            // 转换为可读的过期时间
            if let Some(datetime) = chrono::DateTime::from_timestamp(timestamp as i64, 0) {
                println!("  过期时间: {}", datetime.format("%Y-%m-%d %H:%M:%S UTC"));
            }
        }
        None => println!("  过期时间戳: 未提供")
    }
}

/// 演示如何将临时密钥转换为 JSON 格式返回给前端
#[allow(dead_code)]
fn credentials_to_json(credentials: &TemporaryCredentials) -> String {
    let expiration = credentials.expired_time
        .and_then(|timestamp| chrono::DateTime::from_timestamp(timestamp as i64, 0))
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| "Invalid timestamp".to_string());
    
    serde_json::json!({
        "tmpSecretId": credentials.tmp_secret_id,
        "tmpSecretKey": credentials.tmp_secret_key,
        "sessionToken": credentials.token,
        "expiredTime": credentials.expired_time,
        "expiration": expiration
    }).to_string()
}

/// 演示前端使用临时密钥的示例代码（注释形式）
#[allow(dead_code)]
fn frontend_usage_example() {
    println!("\n=== 前端使用示例（JavaScript） ===");
    println!("// 1. 从后端获取临时密钥");
    println!("const response = await fetch('/api/sts/credentials');");
    println!("const credentials = await response.json();");
    println!();
    println!("// 2. 使用临时密钥配置 COS SDK");
    println!("const cos = new COS({{");
    println!("    SecretId: credentials.tmpSecretId,");
    println!("    SecretKey: credentials.tmpSecretKey,");
    println!("    SecurityToken: credentials.sessionToken,");
    println!("}});");
    println!();
    println!("// 3. 使用临时密钥上传文件");
    println!("cos.putObject({{");
    println!("    Bucket: 'your-bucket-name',");
    println!("    Region: 'ap-beijing',");
    println!("    Key: 'uploads/file.jpg',");
    println!("    Body: file,");
    println!("}}, function(err, data) {{");
    println!("    if (err) {{");
    println!("        console.error('上传失败:', err);");
    println!("    }}}} else {{");
    println!("        console.log('上传成功:', data);");
    println!("    }}}}");
    println!("}});");
}