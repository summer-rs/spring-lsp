//! 错误类型定义和错误处理策略
//!
//! 本模块定义了 summer-lsp 的错误类型体系和错误处理策略。
//!
//! ## 错误分类
//!
//! 1. **协议错误** (`ProtocolError`): LSP 协议违规或通信错误
//! 2. **解析错误** (`ParseError`): TOML 或 Rust 代码语法错误
//! 3. **验证错误** (`ValidationError`): 配置或代码语义错误
//! 4. **系统错误** (`SystemError`): 文件 I/O、网络请求等系统级错误
//!
//! ## 错误处理策略
//!
//! - **协议错误**: 记录日志，返回标准 LSP 错误响应，尝试恢复连接
//! - **解析错误**: 生成诊断信息，尝试部分解析，缓存错误状态
//! - **验证错误**: 生成诊断信息，不影响其他文档，提供快速修复
//! - **系统错误**: 记录日志，使用降级策略（缓存/默认值），显示友好错误
//!
//! ## 错误恢复
//!
//! - **连接恢复**: 协议错误后尝试重新建立连接
//! - **部分解析**: 解析错误时尽可能提取有效信息
//! - **缓存使用**: 系统错误时使用缓存的数据
//! - **降级策略**: 关键功能失败时使用备用方案

use lsp_types::Url;
use thiserror::Error;

/// summer-lsp 错误类型
#[derive(Debug, Error)]
pub enum Error {
    // ========== 协议错误 ==========
    /// LSP 协议错误
    #[error("LSP protocol error: {0}")]
    Protocol(#[from] lsp_server::ProtocolError),

    /// 消息发送错误
    #[error("Failed to send message: {0}")]
    MessageSend(String),

    /// 消息接收错误
    #[error("Failed to receive message: {0}")]
    MessageReceive(String),

    // ========== 解析错误 ==========
    /// TOML 解析错误
    #[error("TOML parse error in {uri}: {message}")]
    TomlParse { uri: String, message: String },

    /// Rust 语法解析错误
    #[error("Rust parse error in {uri}: {message}")]
    RustParse { uri: String, message: String },

    /// 环境变量插值语法错误
    #[error("Invalid environment variable syntax in {uri} at line {line}: {message}")]
    EnvVarSyntax {
        uri: String,
        line: u32,
        message: String,
    },

    // ========== 验证错误 ==========
    /// 配置验证错误
    #[error("Configuration validation error in {uri}: {message}")]
    ConfigValidation { uri: String, message: String },

    /// 路由验证错误
    #[error("Route validation error in {uri}: {message}")]
    RouteValidation { uri: String, message: String },

    /// 依赖注入验证错误
    #[error("Dependency injection validation error in {uri}: {message}")]
    DiValidation { uri: String, message: String },

    // ========== 系统错误 ==========
    /// Schema 加载错误
    #[error("Schema load error: {0}")]
    SchemaLoad(String),

    /// 配置错误
    #[error("Configuration error: {0}")]
    Config(String),

    /// 文件 I/O 错误
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 序列化/反序列化错误
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP 请求错误
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    /// 索引构建错误
    #[error("Index build error: {0}")]
    IndexBuild(String),

    /// 其他错误
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl Error {
    /// 获取错误类别
    pub fn category(&self) -> ErrorCategory {
        match self {
            Error::Protocol(_) | Error::MessageSend(_) | Error::MessageReceive(_) => {
                ErrorCategory::Protocol
            }
            Error::TomlParse { .. } | Error::RustParse { .. } | Error::EnvVarSyntax { .. } => {
                ErrorCategory::Parse
            }
            Error::ConfigValidation { .. }
            | Error::RouteValidation { .. }
            | Error::DiValidation { .. } => ErrorCategory::Validation,
            Error::SchemaLoad(_)
            | Error::Config(_)
            | Error::Io(_)
            | Error::Json(_)
            | Error::Http(_)
            | Error::IndexBuild(_)
            | Error::Other(_) => ErrorCategory::System,
        }
    }

    /// 判断错误是否可恢复
    pub fn is_recoverable(&self) -> bool {
        match self {
            // 协议错误可能可以恢复
            Error::Protocol(_) => true,
            Error::MessageSend(_) | Error::MessageReceive(_) => true,

            // 解析错误可以部分恢复（提供有限功能）
            Error::TomlParse { .. } | Error::RustParse { .. } | Error::EnvVarSyntax { .. } => true,

            // 验证错误不影响服务器运行
            Error::ConfigValidation { .. }
            | Error::RouteValidation { .. }
            | Error::DiValidation { .. } => true,

            // 系统错误部分可恢复
            Error::SchemaLoad(_) => true, // 可以使用备用 Schema
            Error::Config(_) => false,    // 配置错误不可恢复
            Error::Http(_) => true,       // 可以使用缓存
            Error::IndexBuild(_) => true, // 可以跳过索引构建
            Error::Io(_) => false,        // I/O 错误通常不可恢复
            Error::Json(_) => false,      // JSON 错误通常不可恢复
            Error::Other(_) => false,     // 未知错误默认不可恢复
        }
    }

    /// 获取错误的严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // 协议错误是严重的，可能导致连接中断
            Error::Protocol(_) => ErrorSeverity::Error,
            Error::MessageSend(_) | Error::MessageReceive(_) => ErrorSeverity::Error,

            // 解析错误是警告，不影响其他功能
            Error::TomlParse { .. } | Error::RustParse { .. } => ErrorSeverity::Warning,
            Error::EnvVarSyntax { .. } => ErrorSeverity::Warning,

            // 验证错误是信息，只影响诊断
            Error::ConfigValidation { .. }
            | Error::RouteValidation { .. }
            | Error::DiValidation { .. } => ErrorSeverity::Info,

            // 系统错误根据类型判断
            Error::SchemaLoad(_) => ErrorSeverity::Warning, // 可以使用备用 Schema
            Error::Config(_) => ErrorSeverity::Error,       // 配置错误是严重的
            Error::Http(_) => ErrorSeverity::Warning,       // 可以使用缓存
            Error::IndexBuild(_) => ErrorSeverity::Warning, // 可以跳过索引
            Error::Io(_) => ErrorSeverity::Error,           // I/O 错误是严重的
            Error::Json(_) => ErrorSeverity::Error,         // JSON 错误是严重的
            Error::Other(_) => ErrorSeverity::Error,        // 未知错误默认严重
        }
    }

    /// 获取错误的文档 URI（如果有）
    pub fn document_uri(&self) -> Option<&str> {
        match self {
            Error::TomlParse { uri, .. }
            | Error::RustParse { uri, .. }
            | Error::EnvVarSyntax { uri, .. }
            | Error::ConfigValidation { uri, .. }
            | Error::RouteValidation { uri, .. }
            | Error::DiValidation { uri, .. } => Some(uri),
            _ => None,
        }
    }
}

/// 错误类别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// 协议错误
    Protocol,
    /// 解析错误
    Parse,
    /// 验证错误
    Validation,
    /// 系统错误
    System,
}

/// 错误严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
}

/// 错误处理器
///
/// 提供统一的错误处理和恢复策略
pub struct ErrorHandler {
    /// 是否启用详细日志
    verbose: bool,
}

impl ErrorHandler {
    /// 创建新的错误处理器
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    /// 处理错误
    ///
    /// 根据错误类型执行相应的处理策略：
    /// - 记录日志
    /// - 尝试恢复
    /// - 返回降级结果
    pub fn handle(&self, error: &Error) -> ErrorHandlingResult {
        // 记录错误日志
        self.log_error(error);

        // 根据错误类别执行处理策略
        match error.category() {
            ErrorCategory::Protocol => self.handle_protocol_error(error),
            ErrorCategory::Parse => self.handle_parse_error(error),
            ErrorCategory::Validation => self.handle_validation_error(error),
            ErrorCategory::System => self.handle_system_error(error),
        }
    }

    /// 记录错误日志
    fn log_error(&self, error: &Error) {
        match error.severity() {
            ErrorSeverity::Error => {
                if self.verbose {
                    tracing::error!("{:?}", error);
                } else {
                    tracing::error!("{}", error);
                }
            }
            ErrorSeverity::Warning => {
                if self.verbose {
                    tracing::warn!("{:?}", error);
                } else {
                    tracing::warn!("{}", error);
                }
            }
            ErrorSeverity::Info => {
                if self.verbose {
                    tracing::info!("{:?}", error);
                } else {
                    tracing::info!("{}", error);
                }
            }
        }
    }

    /// 处理协议错误
    fn handle_protocol_error(&self, error: &Error) -> ErrorHandlingResult {
        tracing::error!("Protocol error occurred: {}", error);

        ErrorHandlingResult {
            action: RecoveryAction::RetryConnection,
            fallback: None,
            notify_client: true,
        }
    }

    /// 处理解析错误
    fn handle_parse_error(&self, error: &Error) -> ErrorHandlingResult {
        tracing::warn!("Parse error occurred: {}", error);

        ErrorHandlingResult {
            action: RecoveryAction::PartialParse,
            fallback: None,
            notify_client: true,
        }
    }

    /// 处理验证错误
    fn handle_validation_error(&self, error: &Error) -> ErrorHandlingResult {
        tracing::info!("Validation error occurred: {}", error);

        ErrorHandlingResult {
            action: RecoveryAction::GenerateDiagnostic,
            fallback: None,
            notify_client: false, // 验证错误通过诊断通知，不需要额外通知
        }
    }

    /// 处理系统错误
    fn handle_system_error(&self, error: &Error) -> ErrorHandlingResult {
        tracing::error!("System error occurred: {}", error);

        let (action, fallback) = match error {
            Error::SchemaLoad(_) => (RecoveryAction::UseFallback, Some("builtin-schema")),
            Error::Http(_) => (RecoveryAction::UseCache, None),
            Error::IndexBuild(_) => (RecoveryAction::SkipOperation, None),
            _ => (RecoveryAction::Abort, None),
        };

        ErrorHandlingResult {
            action,
            fallback: fallback.map(String::from),
            notify_client: true,
        }
    }
}

/// 错误处理结果
#[derive(Debug)]
pub struct ErrorHandlingResult {
    /// 恢复动作
    pub action: RecoveryAction,
    /// 降级方案（如果有）
    pub fallback: Option<String>,
    /// 是否通知客户端
    pub notify_client: bool,
}

/// 恢复动作
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    /// 重试连接
    RetryConnection,
    /// 部分解析
    PartialParse,
    /// 生成诊断
    GenerateDiagnostic,
    /// 使用降级方案
    UseFallback,
    /// 使用缓存
    UseCache,
    /// 跳过操作
    SkipOperation,
    /// 中止
    Abort,
}

/// Result 类型别名
pub type Result<T> = std::result::Result<T, Error>;

/// 创建 TOML 解析错误
pub fn toml_parse_error(uri: &Url, message: impl Into<String>) -> Error {
    Error::TomlParse {
        uri: uri.to_string(),
        message: message.into(),
    }
}

/// 创建 Rust 解析错误
pub fn rust_parse_error(uri: &Url, message: impl Into<String>) -> Error {
    Error::RustParse {
        uri: uri.to_string(),
        message: message.into(),
    }
}

/// 创建环境变量语法错误
pub fn env_var_syntax_error(uri: &Url, line: u32, message: impl Into<String>) -> Error {
    Error::EnvVarSyntax {
        uri: uri.to_string(),
        line,
        message: message.into(),
    }
}

/// 创建配置验证错误
pub fn config_validation_error(uri: &Url, message: impl Into<String>) -> Error {
    Error::ConfigValidation {
        uri: uri.to_string(),
        message: message.into(),
    }
}

/// 创建路由验证错误
pub fn route_validation_error(uri: &Url, message: impl Into<String>) -> Error {
    Error::RouteValidation {
        uri: uri.to_string(),
        message: message.into(),
    }
}

/// 创建依赖注入验证错误
pub fn di_validation_error(uri: &Url, message: impl Into<String>) -> Error {
    Error::DiValidation {
        uri: uri.to_string(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category() {
        // 使用消息发送错误代替协议错误（更容易构造）
        let protocol_err = Error::MessageSend("test error".to_string());
        assert_eq!(protocol_err.category(), ErrorCategory::Protocol);

        let parse_err = Error::TomlParse {
            uri: "file:///test.toml".to_string(),
            message: "syntax error".to_string(),
        };
        assert_eq!(parse_err.category(), ErrorCategory::Parse);

        let validation_err = Error::ConfigValidation {
            uri: "file:///test.toml".to_string(),
            message: "invalid config".to_string(),
        };
        assert_eq!(validation_err.category(), ErrorCategory::Validation);

        let system_err = Error::SchemaLoad("failed to load".to_string());
        assert_eq!(system_err.category(), ErrorCategory::System);
    }

    #[test]
    fn test_error_recoverability() {
        let protocol_err = Error::MessageSend("test error".to_string());
        assert!(protocol_err.is_recoverable());

        let parse_err = Error::TomlParse {
            uri: "file:///test.toml".to_string(),
            message: "syntax error".to_string(),
        };
        assert!(parse_err.is_recoverable());

        let io_err = Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(!io_err.is_recoverable());
    }

    #[test]
    fn test_error_severity() {
        let protocol_err = Error::MessageSend("test error".to_string());
        assert_eq!(protocol_err.severity(), ErrorSeverity::Error);

        let parse_err = Error::TomlParse {
            uri: "file:///test.toml".to_string(),
            message: "syntax error".to_string(),
        };
        assert_eq!(parse_err.severity(), ErrorSeverity::Warning);

        let validation_err = Error::ConfigValidation {
            uri: "file:///test.toml".to_string(),
            message: "invalid config".to_string(),
        };
        assert_eq!(validation_err.severity(), ErrorSeverity::Info);
    }

    #[test]
    fn test_error_document_uri() {
        let parse_err = Error::TomlParse {
            uri: "file:///test.toml".to_string(),
            message: "syntax error".to_string(),
        };
        assert_eq!(parse_err.document_uri(), Some("file:///test.toml"));

        let system_err = Error::SchemaLoad("failed".to_string());
        assert_eq!(system_err.document_uri(), None);
    }

    #[test]
    fn test_error_handler() {
        let handler = ErrorHandler::new(false);

        // 测试协议错误处理
        let protocol_err = Error::MessageSend("test error".to_string());
        let result = handler.handle(&protocol_err);
        assert_eq!(result.action, RecoveryAction::RetryConnection);
        assert!(result.notify_client);

        // 测试解析错误处理
        let parse_err = Error::TomlParse {
            uri: "file:///test.toml".to_string(),
            message: "syntax error".to_string(),
        };
        let result = handler.handle(&parse_err);
        assert_eq!(result.action, RecoveryAction::PartialParse);
        assert!(result.notify_client);

        // 测试验证错误处理
        let validation_err = Error::ConfigValidation {
            uri: "file:///test.toml".to_string(),
            message: "invalid config".to_string(),
        };
        let result = handler.handle(&validation_err);
        assert_eq!(result.action, RecoveryAction::GenerateDiagnostic);
        assert!(!result.notify_client);

        // 测试系统错误处理（Schema 加载失败）
        let schema_err = Error::SchemaLoad("failed".to_string());
        let result = handler.handle(&schema_err);
        assert_eq!(result.action, RecoveryAction::UseFallback);
        assert_eq!(result.fallback, Some("builtin-schema".to_string()));
        assert!(result.notify_client);
    }

    #[test]
    fn test_error_helper_functions() {
        let uri = Url::parse("file:///test.toml").unwrap();

        let toml_err = toml_parse_error(&uri, "syntax error");
        assert!(matches!(toml_err, Error::TomlParse { .. }));

        let rust_err = rust_parse_error(&uri, "syntax error");
        assert!(matches!(rust_err, Error::RustParse { .. }));

        let env_err = env_var_syntax_error(&uri, 10, "invalid syntax");
        assert!(matches!(env_err, Error::EnvVarSyntax { .. }));

        let config_err = config_validation_error(&uri, "invalid config");
        assert!(matches!(config_err, Error::ConfigValidation { .. }));

        let route_err = route_validation_error(&uri, "invalid route");
        assert!(matches!(route_err, Error::RouteValidation { .. }));

        let di_err = di_validation_error(&uri, "invalid injection");
        assert!(matches!(di_err, Error::DiValidation { .. }));
    }
}
