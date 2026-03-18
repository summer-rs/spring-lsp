//! 日志系统配置和管理
//!
//! 本模块提供 summer-lsp 的日志系统配置，支持：
//! - 通过环境变量配置日志级别
//! - 输出到文件和标准错误流
//! - 结构化日志（包含时间戳、级别、模块等）
//! - 调试模式支持
//! - 确保日志不干扰 LSP 协议通信（LSP 使用 stdio）
//!
//! ## 环境变量
//!
//! - `SUMMER_LSP_LOG_LEVEL`: 日志级别（trace, debug, info, warn, error），默认为 info
//! - `SUMMER_LSP_VERBOSE`: 启用详细日志模式（设置为 1 或 true）
//! - `SUMMER_LSP_LOG_FILE`: 日志文件路径（可选，如果不设置则只输出到 stderr）
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use summer_lsp::logging::init_logging;
//!
//! // 初始化日志系统
//! init_logging().expect("Failed to initialize logging");
//!
//! tracing::info!("Application started");
//! ```

use std::env;
use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// 日志配置
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// 日志级别
    pub level: String,
    /// 是否启用详细模式
    pub verbose: bool,
    /// 日志文件路径（可选）
    pub log_file: Option<PathBuf>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            verbose: false,
            log_file: None,
        }
    }
}

impl LogConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        let level = env::var("SUMMER_LSP_LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string())
            .to_lowercase();

        let verbose = env::var("SUMMER_LSP_VERBOSE")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let log_file = env::var("SUMMER_LSP_LOG_FILE").ok().map(PathBuf::from);

        Self {
            level,
            verbose,
            log_file,
        }
    }

    /// 创建环境过滤器
    fn create_env_filter(&self) -> EnvFilter {
        // 如果设置了 RUST_LOG 环境变量，优先使用它
        if let Ok(filter) = EnvFilter::try_from_default_env() {
            return filter;
        }

        // 否则使用配置的日志级别
        let level = &self.level;

        // 在详细模式下，显示所有模块的日志
        if self.verbose {
            EnvFilter::new(format!("summer_lsp={},lsp_server={}", level, level))
        } else {
            // 正常模式下，只显示 summer_lsp 的日志
            EnvFilter::new(format!("summer_lsp={}", level))
        }
    }

    /// 验证日志级别
    pub fn validate_level(&self) -> Result<(), String> {
        match self.level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => Ok(()),
            _ => Err(format!(
                "Invalid log level: {}. Valid levels are: trace, debug, info, warn, error",
                self.level
            )),
        }
    }
}

/// 初始化日志系统
///
/// 根据环境变量配置日志系统，支持输出到 stderr 和文件。
///
/// # 错误
///
/// 如果日志系统初始化失败，返回错误信息。
///
/// # 注意
///
/// - LSP 协议使用 stdin/stdout 通信，因此日志必须输出到 stderr 或文件
/// - 此函数应该在程序启动时调用一次
/// - 重复调用会返回错误
pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    let config = LogConfig::from_env();
    init_logging_with_config(config)
}

/// 使用指定配置初始化日志系统
///
/// # 参数
///
/// - `config`: 日志配置
///
/// # 错误
///
/// 如果日志系统初始化失败，返回错误信息。
pub fn init_logging_with_config(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 验证日志级别
    config.validate_level()?;

    let env_filter = config.create_env_filter();

    // 创建 stderr 输出层
    let stderr_layer = fmt::layer()
        .with_writer(io::stderr)
        .with_ansi(atty::is(atty::Stream::Stderr)) // 只在终端时使用颜色
        .with_target(config.verbose) // 详细模式显示目标模块
        .with_thread_ids(config.verbose) // 详细模式显示线程 ID
        .with_thread_names(config.verbose) // 详细模式显示线程名称
        .with_line_number(config.verbose) // 详细模式显示行号
        .with_file(config.verbose) // 详细模式显示文件名
        .with_filter(env_filter.clone());

    // 如果配置了日志文件，创建文件输出层
    if let Some(log_file) = &config.log_file {
        // 确保日志文件的父目录存在
        if let Some(parent) = log_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 打开或创建日志文件（追加模式）
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;

        // 创建文件输出层（使用 JSON 格式以便于解析）
        let file_layer = fmt::layer()
            .with_writer(file)
            .json() // 使用 JSON 格式
            .with_current_span(true) // 包含当前 span 信息
            .with_span_list(true) // 包含 span 列表
            .with_filter(env_filter);

        // 组合两个层
        tracing_subscriber::registry()
            .with(stderr_layer)
            .with(file_layer)
            .try_init()?;
    } else {
        // 只使用 stderr 输出
        tracing_subscriber::registry()
            .with(stderr_layer)
            .try_init()?;
    }

    // 记录日志系统初始化信息
    tracing::info!(
        level = %config.level,
        verbose = config.verbose,
        log_file = ?config.log_file,
        "Logging system initialized"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, "info");
        assert!(!config.verbose);
        assert!(config.log_file.is_none());
    }

    #[test]
    fn test_log_config_from_env() {
        // 保存原始环境变量
        let original_level = env::var("SUMMER_LSP_LOG_LEVEL").ok();
        let original_verbose = env::var("SUMMER_LSP_VERBOSE").ok();
        let original_file = env::var("SUMMER_LSP_LOG_FILE").ok();

        // 设置测试环境变量
        env::set_var("SUMMER_LSP_LOG_LEVEL", "debug");
        env::set_var("SUMMER_LSP_VERBOSE", "true");
        env::set_var("SUMMER_LSP_LOG_FILE", "/tmp/summer-lsp.log");

        let config = LogConfig::from_env();
        assert_eq!(config.level, "debug");
        assert!(config.verbose);
        assert_eq!(config.log_file, Some(PathBuf::from("/tmp/summer-lsp.log")));

        // 恢复原始环境变量
        match original_level {
            Some(v) => env::set_var("SUMMER_LSP_LOG_LEVEL", v),
            None => env::remove_var("SUMMER_LSP_LOG_LEVEL"),
        }
        match original_verbose {
            Some(v) => env::set_var("SUMMER_LSP_VERBOSE", v),
            None => env::remove_var("SUMMER_LSP_VERBOSE"),
        }
        match original_file {
            Some(v) => env::set_var("SUMMER_LSP_LOG_FILE", v),
            None => env::remove_var("SUMMER_LSP_LOG_FILE"),
        }
    }

    #[test]
    fn test_log_config_verbose_variants() {
        // 保存原始环境变量
        let original = env::var("SUMMER_LSP_VERBOSE").ok();

        // 测试 "1"
        env::set_var("SUMMER_LSP_VERBOSE", "1");
        let config = LogConfig::from_env();
        assert!(config.verbose);

        // 测试 "true"
        env::set_var("SUMMER_LSP_VERBOSE", "true");
        let config = LogConfig::from_env();
        assert!(config.verbose);

        // 测试 "TRUE"
        env::set_var("SUMMER_LSP_VERBOSE", "TRUE");
        let config = LogConfig::from_env();
        assert!(config.verbose);

        // 测试 "false"
        env::set_var("SUMMER_LSP_VERBOSE", "false");
        let config = LogConfig::from_env();
        assert!(!config.verbose);

        // 测试未设置
        env::remove_var("SUMMER_LSP_VERBOSE");
        let config = LogConfig::from_env();
        assert!(!config.verbose);

        // 恢复原始环境变量
        match original {
            Some(v) => env::set_var("SUMMER_LSP_VERBOSE", v),
            None => env::remove_var("SUMMER_LSP_VERBOSE"),
        }
    }

    #[test]
    fn test_validate_level() {
        let valid_levels = vec!["trace", "debug", "info", "warn", "error"];
        for level in valid_levels {
            let config = LogConfig {
                level: level.to_string(),
                ..Default::default()
            };
            assert!(config.validate_level().is_ok());
        }

        let invalid_config = LogConfig {
            level: "invalid".to_string(),
            ..Default::default()
        };
        assert!(invalid_config.validate_level().is_err());
    }

    #[test]
    fn test_create_env_filter() {
        let config = LogConfig {
            level: "debug".to_string(),
            verbose: false,
            log_file: None,
        };
        let _filter = config.create_env_filter();
        // EnvFilter 已创建，无法直接测试其内容
        // 只验证创建成功即可

        let verbose_config = LogConfig {
            level: "info".to_string(),
            verbose: true,
            log_file: None,
        };
        let _filter = verbose_config.create_env_filter();
        // EnvFilter 已创建，无法直接测试其内容
        // 只验证创建成功即可
    }

    #[test]
    fn test_log_config_case_insensitive() {
        // 保存原始环境变量
        let original = env::var("SUMMER_LSP_LOG_LEVEL").ok();

        // 测试大写
        env::set_var("SUMMER_LSP_LOG_LEVEL", "DEBUG");
        let config = LogConfig::from_env();
        assert_eq!(config.level, "debug");

        // 测试混合大小写
        env::set_var("SUMMER_LSP_LOG_LEVEL", "WaRn");
        let config = LogConfig::from_env();
        assert_eq!(config.level, "warn");

        // 恢复原始环境变量
        match original {
            Some(v) => env::set_var("SUMMER_LSP_LOG_LEVEL", v),
            None => env::remove_var("SUMMER_LSP_LOG_LEVEL"),
        }
    }
}
