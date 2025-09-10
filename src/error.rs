//! 错误处理模块

use thiserror::Error;

/// COS SDK 错误类型
#[derive(Error, Debug)]
pub enum CosError {
    /// HTTP 请求错误
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON 序列化/反序列化错误
    #[error("JSON serialization/deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    /// URL 解析错误
    #[error("URL parsing failed: {0}")]
    Url(#[from] url::ParseError),

    /// 认证错误
    #[error("Authentication failed: {message}")]
    Auth { message: String },

    /// 服务器错误
    #[error("Server error: {code} - {message}")]
    Server { code: String, message: String },

    /// 客户端错误
    #[error("Client error: {code} - {message}")]
    Client { code: String, message: String },

    /// 配置错误
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 其他错误
    #[error("Other error: {message}")]
    Other { message: String },
}

impl CosError {
    /// 创建认证错误
    pub fn auth<S: Into<String>>(message: S) -> Self {
        Self::Auth {
            message: message.into(),
        }
    }

    /// 创建服务器错误
    pub fn server<S: Into<String>>(code: S, message: S) -> Self {
        Self::Server {
            code: code.into(),
            message: message.into(),
        }
    }

    /// 创建客户端错误
    pub fn client<S: Into<String>>(code: S, message: S) -> Self {
        Self::Client {
            code: code.into(),
            message: message.into(),
        }
    }

    /// 创建配置错误
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// 创建其他错误
    pub fn other<S: Into<String>>(message: S) -> Self {
        Self::Other {
            message: message.into(),
        }
    }
}

/// COS SDK 结果类型
pub type Result<T> = std::result::Result<T, CosError>;