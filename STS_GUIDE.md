# 腾讯云 COS STS 临时密钥使用指南

## 概述

STS（Security Token Service）临时密钥是腾讯云提供的临时访问凭证服务。通过 STS，您可以获取有限时间和权限的临时密钥，用于访问腾讯云 COS 服务。这种方式特别适合以下场景：

- **前端直传**：前端应用直接上传文件到 COS，无需通过后端服务器中转
- **移动应用**：移动 App 直接访问 COS 服务
- **第三方应用**：为第三方应用提供有限的 COS 访问权限
- **临时授权**：为临时用户或任务提供短期访问权限

## 快速开始

### 1. 添加依赖

在您的 `Cargo.toml` 中添加：

```toml
[dependencies]
cos-rust-sdk = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
env_logger = "0.10"
```

### 2. 基本使用

```rust
use cos_rust_sdk::{
    StsClient, Policy, GetCredentialsRequest, TemporaryCredentials,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 STS 客户端
    let sts_client = StsClient::new(
        "your-secret-id".to_string(),
        "your-secret-key".to_string(),
        "ap-beijing".to_string(), // 区域
    );

    // 创建权限策略（允许上传到指定前缀）
    let policy = Policy::allow_put_object("your-bucket-name-appid", Some("uploads/"));

    // 创建获取临时密钥的请求
    let request = GetCredentialsRequest {
        policy,
        duration_seconds: Some(3600), // 1小时有效期
        name: Some("upload-session".to_string()),
    };

    // 获取临时密钥
    let credentials = sts_client.get_credentials(request).await?;

    println!("临时密钥获取成功:");
    println!("  临时访问密钥 ID: {}", credentials.tmp_secret_id);
    println!("  临时访问密钥: {}", credentials.tmp_secret_key);
    println!("  安全令牌: {}", credentials.token);
    println!("  过期时间戳: {}", credentials.expired_time);

    Ok(())
}
```

## 权限策略配置

### 预定义策略

本 SDK 提供了几种常用的预定义策略：

```rust
// 1. 允许上传对象（PUT 操作）
let policy = Policy::allow_put_object("bucket-name", Some("uploads/"));

// 2. 允许下载对象（GET 操作）
let policy = Policy::allow_get_object("bucket-name", Some("downloads/"));

// 3. 允许读写操作（GET + PUT + DELETE）
let policy = Policy::allow_read_write("bucket-name", Some("temp/"));

// 4. 允许删除对象
let policy = Policy::allow_delete_object("bucket-name", Some("temp/"));
```

### 自定义策略

您也可以创建自定义的权限策略：

```rust
use cos_rust_sdk::{Policy, Statement};

let policy = Policy {
    version: "2.0".to_string(),
    statement: vec![
        Statement {
            effect: "allow".to_string(),
            action: vec![
                "cos:PutObject".to_string(),
                "cos:GetObject".to_string(),
            ],
            resource: vec![
                "qcs::cos:ap-beijing:uid/appid:bucket-name/prefix/*".to_string(),
            ],
            condition: None,
        },
    ],
};
```

## 前端集成示例

### 后端 API 接口

创建一个后端 API 接口来获取临时密钥：

```rust
// 使用 axum 框架的示例
use axum::{Json, response::Json as ResponseJson};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct CredentialsResponse {
    tmp_secret_id: String,
    tmp_secret_key: String,
    session_token: String,
    expired_time: u64,
    expiration: String,
}

async fn get_upload_credentials() -> ResponseJson<CredentialsResponse> {
    let sts_client = StsClient::new(
        std::env::var("TENCENT_SECRET_ID").unwrap(),
        std::env::var("TENCENT_SECRET_KEY").unwrap(),
        "ap-beijing".to_string(),
    );

    let policy = Policy::allow_put_object("your-bucket", Some("uploads/"));
    let request = GetCredentialsRequest {
        policy,
        duration_seconds: Some(3600),
        name: Some("upload-session".to_string()),
    };

    let credentials = sts_client.get_credentials(request).await.unwrap();

    let response = CredentialsResponse {
        tmp_secret_id: credentials.tmp_secret_id,
        tmp_secret_key: credentials.tmp_secret_key,
        session_token: credentials.token,
        expired_time: credentials.expired_time,
        expiration: chrono::DateTime::from_timestamp(credentials.expired_time as i64, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            .unwrap_or_default(),
    };

    ResponseJson(response)
}
```

### 前端使用（JavaScript）

```javascript
// 1. 从后端获取临时密钥
async function getTemporaryCredentials() {
    const response = await fetch('/api/sts/upload-credentials');
    return await response.json();
}

// 2. 使用临时密钥配置 COS SDK
async function uploadFile(file) {
    const credentials = await getTemporaryCredentials();
    
    const cos = new COS({
        SecretId: credentials.tmp_secret_id,
        SecretKey: credentials.tmp_secret_key,
        SecurityToken: credentials.session_token,
    });

    // 3. 上传文件
    return new Promise((resolve, reject) => {
        cos.putObject({
            Bucket: 'your-bucket-name',
            Region: 'ap-beijing',
            Key: `uploads/${file.name}`,
            Body: file,
        }, (err, data) => {
            if (err) {
                reject(err);
            } else {
                resolve(data);
            }
        });
    });
}

// 使用示例
document.getElementById('fileInput').addEventListener('change', async (event) => {
    const file = event.target.files[0];
    if (file) {
        try {
            const result = await uploadFile(file);
            console.log('上传成功:', result);
        } catch (error) {
            console.error('上传失败:', error);
        }
    }
});
```

## 安全最佳实践

### 1. 最小权限原则

只授予必要的权限，避免过度授权：

```rust
// ✅ 好的做法：只允许上传到特定前缀
let policy = Policy::allow_put_object("bucket", Some("user-123/uploads/"));

// ❌ 不好的做法：允许访问整个存储桶
let policy = Policy::allow_read_write("bucket", None);
```

### 2. 合理设置有效期

根据使用场景设置合适的有效期：

```rust
// 文件上传：较短的有效期
let request = GetCredentialsRequest {
    policy,
    duration_seconds: Some(1800), // 30分钟
    name: Some("upload-session".to_string()),
};

// 批量操作：较长的有效期
let request = GetCredentialsRequest {
    policy,
    duration_seconds: Some(7200), // 2小时
    name: Some("batch-operation".to_string()),
};
```

### 3. 前缀隔离

为不同用户或应用使用不同的前缀：

```rust
// 为每个用户创建独立的前缀
let user_prefix = format!("user-{}/", user_id);
let policy = Policy::allow_put_object("bucket", Some(&user_prefix));
```

### 4. 环境变量管理

使用环境变量管理敏感信息：

```bash
# 设置环境变量
export TENCENT_SECRET_ID="your-secret-id"
export TENCENT_SECRET_KEY="your-secret-key"
export COS_REGION="ap-beijing"
export COS_BUCKET="your-bucket-name-appid"
```

## 错误处理

```rust
use cos_rust_sdk::CosError;

match sts_client.get_credentials(request).await {
    Ok(credentials) => {
        // 成功获取临时密钥
        println!("临时密钥获取成功");
    }
    Err(CosError::AuthError(msg)) => {
        eprintln!("认证错误: {}", msg);
    }
    Err(CosError::NetworkError(msg)) => {
        eprintln!("网络错误: {}", msg);
    }
    Err(CosError::ServerError { code, message }) => {
        eprintln!("服务器错误 {}: {}", code, message);
    }
    Err(e) => {
        eprintln!("其他错误: {}", e);
    }
}
```

## 运行示例

本 SDK 提供了完整的示例代码：

```bash
# 设置环境变量
export TENCENT_SECRET_ID="your-secret-id"
export TENCENT_SECRET_KEY="your-secret-key"
export COS_REGION="ap-beijing"
export COS_BUCKET="your-bucket-name-appid"

# 运行 STS 示例
cargo run --example sts_example
```

## 常见问题

### Q: 临时密钥的有效期是多长？
A: 临时密钥的有效期可以在 15 分钟到 12 小时之间设置，默认为 1 小时。

### Q: 如何检查临时密钥是否过期？
A: 可以通过比较当前时间戳和 `expired_time` 字段来判断：

```rust
let current_time = chrono::Utc::now().timestamp() as u64;
if current_time >= credentials.expired_time {
    println!("临时密钥已过期，需要重新获取");
}
```

### Q: 前端如何处理临时密钥过期？
A: 建议在前端实现自动刷新机制：

```javascript
class CredentialsManager {
    constructor() {
        this.credentials = null;
        this.refreshPromise = null;
    }

    async getValidCredentials() {
        if (!this.credentials || this.isExpired(this.credentials)) {
            if (!this.refreshPromise) {
                this.refreshPromise = this.refreshCredentials();
            }
            this.credentials = await this.refreshPromise;
            this.refreshPromise = null;
        }
        return this.credentials;
    }

    isExpired(credentials) {
        const now = Math.floor(Date.now() / 1000);
        return now >= credentials.expired_time - 300; // 提前5分钟刷新
    }

    async refreshCredentials() {
        const response = await fetch('/api/sts/credentials');
        return await response.json();
    }
}
```

## 相关链接

- [腾讯云 STS 官方文档](https://cloud.tencent.com/document/product/1312)
- [腾讯云 COS 官方文档](https://cloud.tencent.com/document/product/436)
- [权限策略语法](https://cloud.tencent.com/document/product/436/12469)