# STS 临时密钥功能故障排除指南

## 当前状态

STS 临时密钥功能已基本实现，包括：

✅ **已完成的功能**：
- STS 客户端实现
- 权限策略管理
- API 请求构建
- 响应解析
- 完整的示例代码
- 使用文档

⚠️ **当前问题**：
- TC3-HMAC-SHA256 签名算法验证失败
- 腾讯云 STS API 返回 `AuthFailure.SignatureFailure` 错误

## 错误分析

### 错误信息
```
STS API error: AuthFailure.SignatureFailure - The provided credentials could not be validated. Please check your signature is correct.
```

### 可能原因

1. **签名算法实现问题**
   - TC3-HMAC-SHA256 签名计算可能存在细节错误
   - 规范请求字符串构建可能不完全符合腾讯云标准
   - 请求头顺序或格式可能有误

2. **密钥配置问题**
   - 使用的 SecretId/SecretKey 可能无效或过期
   - 密钥权限可能不足（需要 STS 相关权限）

3. **时间同步问题**
   - 本地时间与腾讯云服务器时间差异过大
   - 时间戳格式或时区处理错误

## 解决方案

### 1. 验证密钥配置

确保使用有效的腾讯云访问密钥：

```bash
# 设置环境变量
export COS_SECRET_ID="your-actual-secret-id"
export COS_SECRET_KEY="your-actual-secret-key"
export COS_REGION="ap-guangzhou"
export COS_BUCKET="your-bucket-appid"
```

**注意**：
- SecretId 通常以 `AKID` 开头
- SecretKey 是一个长字符串
- 确保密钥具有 STS 相关权限

### 2. 检查权限配置

在腾讯云控制台确认：
- 访问密钥状态为"启用"
- 用户或角色具有 `sts:GetFederationToken` 权限
- 如果使用子账号，确保主账号已授权相关权限

### 3. 时间同步

确保系统时间准确：
```bash
# macOS
sudo sntp -sS time.apple.com

# Linux
sudo ntpdate -s time.nist.gov
```

### 4. 签名算法调试

当前实现的签名算法基于腾讯云官方文档，但可能存在细节差异。如需调试：

1. 对比官方 SDK 的签名实现
2. 使用腾讯云 API Explorer 验证请求格式
3. 检查规范请求字符串的每个组成部分

## 替代方案

### 1. 使用官方 SDK

如果签名问题难以解决，可以考虑：
- 使用腾讯云官方 Go SDK 或 Python SDK
- 通过 FFI 调用官方 SDK
- 使用 HTTP 代理调用官方 SDK

### 2. 服务端代理

在服务端实现 STS 代理：
```
前端 -> 你的服务端 -> 腾讯云 STS -> 返回临时密钥
```

### 3. 预签名 URL

对于简单的上传/下载场景，可以使用预签名 URL 替代临时密钥。

## 测试步骤

1. **验证密钥**：
   ```bash
   # 确保环境变量正确设置
   echo $COS_SECRET_ID
   echo $COS_SECRET_KEY
   ```

2. **运行示例**：
   ```bash
   cargo run --example sts_example
   ```

3. **检查输出**：
   - 如果显示密钥格式错误，请检查环境变量
   - 如果显示签名失败，说明是签名算法问题
   - 如果成功，会显示临时密钥信息

## 后续改进

1. **完善签名算法**：
   - 参考更多官方示例
   - 添加详细的调试日志
   - 实现签名验证工具

2. **增强错误处理**：
   - 提供更详细的错误信息
   - 添加自动重试机制
   - 实现签名调试模式

3. **添加测试**：
   - 单元测试覆盖
   - 集成测试
   - 签名算法验证测试

## 联系支持

如果问题持续存在：
1. 检查腾讯云官方文档更新
2. 参考官方 SDK 实现
3. 联系腾讯云技术支持
4. 在相关开源项目中寻求帮助

---

**注意**：当前的 STS 实现已经包含了完整的功能框架，主要问题集中在签名算法的细节实现上。一旦签名问题解决，整个 STS 功能就能正常工作。