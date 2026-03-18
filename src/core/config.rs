//! 服务器配置管理
//!
//! 本模块提供 summer-lsp 服务器的配置管理功能，支持：
//! - 从配置文件读取用户配置
//! - 自定义补全触发字符
//! - 诊断过滤配置
//! - 自定义 Schema URL
//! - 日志级别配置
//!
//! ## 配置文件
//!
//! summer-lsp 支持从以下位置读取配置文件（按优先级排序）：
//! 1. 工作空间根目录下的 `.summer-lsp.toml`
//! 2. 用户主目录下的 `.config/summer-lsp/config.toml`
//! 3. 环境变量配置
//! 4. 默认配置
//!
//! ## 配置文件格式
//!
//! ```toml
//! # 日志配置
//! [logging]
//! level = "info"  # trace, debug, info, warn, error
//! verbose = false
//! log_file = "/tmp/summer-lsp.log"  # 可选
//!
//! # 补全配置
//! [completion]
//! trigger_characters = ["[", ".", "$", "{", "#", "("]
//!
//! # 诊断配置
//! [diagnostics]
//! # 禁用特定类型的诊断
//! disabled = ["deprecated_warning", "restful_style"]
//!
//! # Schema 配置
//! [schema]
//! url = "https://summer-rs.github.io/config-schema.json"
//! # 或使用本地文件
//! # url = "file:///path/to/schema.json"
//! ```
//!
//! ## 环境变量
//!
//! 环境变量会覆盖配置文件中的设置：
//! - `SUMMER_LSP_LOG_LEVEL`: 日志级别
//! - `SUMMER_LSP_VERBOSE`: 启用详细日志
//! - `SUMMER_LSP_LOG_FILE`: 日志文件路径
//! - `SUMMER_LSP_SCHEMA_URL`: Schema URL

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ServerConfig {
    /// 日志配置
    pub logging: LoggingConfig,
    /// 补全配置
    pub completion: CompletionConfig,
    /// 诊断配置
    pub diagnostics: DiagnosticsConfig,
    /// Schema 配置
    pub schema: SchemaConfig,
}

impl ServerConfig {
    /// 从配置文件和环境变量加载配置
    ///
    /// 配置加载顺序（后面的会覆盖前面的）：
    /// 1. 默认配置
    /// 2. 用户主目录配置文件
    /// 3. 工作空间配置文件
    /// 4. 环境变量
    pub fn load(workspace_root: Option<&Path>) -> Self {
        let mut config = Self::default();

        // 1. 尝试加载用户主目录配置
        if let Some(user_config_path) = Self::user_config_path() {
            if let Ok(user_config) = Self::load_from_file(&user_config_path) {
                config = config.merge(user_config);
            }
        }

        // 2. 尝试加载工作空间配置
        if let Some(workspace_root) = workspace_root {
            let workspace_config_path = workspace_root.join(".summer-lsp.toml");
            if let Ok(workspace_config) = Self::load_from_file(&workspace_config_path) {
                config = config.merge(workspace_config);
            }
        }

        // 3. 应用环境变量覆盖
        config = config.apply_env_overrides();

        config
    }

    /// 从文件加载配置
    fn load_from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        tracing::debug!("Loaded configuration from: {}", path.display());
        Ok(config)
    }

    /// 获取用户配置文件路径
    fn user_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("summer-lsp").join("config.toml"))
    }

    /// 合并另一个配置（other 的值会覆盖 self 的值）
    pub fn merge(mut self, other: Self) -> Self {
        self.logging = self.logging.merge(other.logging);
        self.completion = self.completion.merge(other.completion);
        self.diagnostics = self.diagnostics.merge(other.diagnostics);
        self.schema = self.schema.merge(other.schema);
        self
    }

    /// 应用环境变量覆盖
    fn apply_env_overrides(mut self) -> Self {
        self.logging = self.logging.apply_env_overrides();
        self.schema = self.schema.apply_env_overrides();
        self
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        self.logging.validate()?;
        self.completion.validate()?;
        self.schema.validate()?;
        Ok(())
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// 日志级别：trace, debug, info, warn, error
    pub level: String,
    /// 是否启用详细模式
    pub verbose: bool,
    /// 日志文件路径（可选）
    pub log_file: Option<PathBuf>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            verbose: false,
            log_file: None,
        }
    }
}

impl LoggingConfig {
    pub fn merge(self, other: Self) -> Self {
        Self {
            level: other.level,
            verbose: other.verbose,
            log_file: other.log_file.or(self.log_file),
        }
    }

    fn apply_env_overrides(mut self) -> Self {
        if let Ok(level) = env::var("SUMMER_LSP_LOG_LEVEL") {
            self.level = level.to_lowercase();
        }
        if let Ok(verbose) = env::var("SUMMER_LSP_VERBOSE") {
            self.verbose = verbose == "1" || verbose.to_lowercase() == "true";
        }
        if let Ok(log_file) = env::var("SUMMER_LSP_LOG_FILE") {
            self.log_file = Some(PathBuf::from(log_file));
        }
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        match self.level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => Ok(()),
            _ => Err(format!(
                "Invalid log level: {}. Valid levels are: trace, debug, info, warn, error",
                self.level
            )),
        }
    }
}

/// 补全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CompletionConfig {
    /// 触发补全的字符列表
    pub trigger_characters: Vec<String>,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            trigger_characters: vec![
                "[".to_string(), // TOML 配置节
                ".".to_string(), // 嵌套配置项
                "$".to_string(), // 环境变量
                "{".to_string(), // 环境变量插值
                "#".to_string(), // 宏属性
                "(".to_string(), // 宏参数
            ],
        }
    }
}

impl CompletionConfig {
    pub fn merge(self, other: Self) -> Self {
        Self {
            trigger_characters: if other.trigger_characters.is_empty() {
                self.trigger_characters
            } else {
                other.trigger_characters
            },
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.trigger_characters.is_empty() {
            return Err("Trigger characters list cannot be empty".to_string());
        }
        Ok(())
    }
}

/// 诊断配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DiagnosticsConfig {
    /// 禁用的诊断类型列表
    pub disabled: HashSet<String>,
}

impl DiagnosticsConfig {
    pub fn merge(self, other: Self) -> Self {
        Self {
            disabled: if other.disabled.is_empty() {
                self.disabled
            } else {
                other.disabled
            },
        }
    }

    /// 检查诊断类型是否被禁用
    pub fn is_disabled(&self, diagnostic_type: &str) -> bool {
        self.disabled.contains(diagnostic_type)
    }
}

/// Schema 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SchemaConfig {
    /// Schema URL（HTTP URL 或 file:// URL）
    pub url: String,
}

impl Default for SchemaConfig {
    fn default() -> Self {
        Self {
            url: "https://summer-rs.github.io/config-schema.json".to_string(),
        }
    }
}

impl SchemaConfig {
    pub fn merge(self, other: Self) -> Self {
        Self { url: other.url }
    }

    fn apply_env_overrides(mut self) -> Self {
        if let Ok(url) = env::var("SUMMER_LSP_SCHEMA_URL") {
            self.url = url;
        }
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.url.is_empty() {
            return Err("Schema URL cannot be empty".to_string());
        }

        // 验证 URL 格式
        if !self.url.starts_with("http://")
            && !self.url.starts_with("https://")
            && !self.url.starts_with("file://")
        {
            return Err(format!(
                "Invalid Schema URL: {}. Must start with http://, https://, or file://",
                self.url
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.logging.level, "info");
        assert!(!config.logging.verbose);
        assert!(config.logging.log_file.is_none());
        assert_eq!(config.completion.trigger_characters.len(), 6);
        assert!(config.diagnostics.disabled.is_empty());
        assert_eq!(
            config.schema.url,
            "https://summer-rs.github.io/config-schema.json"
        );
    }

    #[test]
    fn test_logging_config_validation() {
        let valid_config = LoggingConfig {
            level: "debug".to_string(),
            verbose: false,
            log_file: None,
        };
        assert!(valid_config.validate().is_ok());

        let invalid_config = LoggingConfig {
            level: "invalid".to_string(),
            verbose: false,
            log_file: None,
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_completion_config_validation() {
        let valid_config = CompletionConfig {
            trigger_characters: vec!["[".to_string()],
        };
        assert!(valid_config.validate().is_ok());

        let invalid_config = CompletionConfig {
            trigger_characters: vec![],
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_schema_config_validation() {
        let valid_http = SchemaConfig {
            url: "https://example.com/schema.json".to_string(),
        };
        assert!(valid_http.validate().is_ok());

        let valid_file = SchemaConfig {
            url: "file:///path/to/schema.json".to_string(),
        };
        assert!(valid_file.validate().is_ok());

        let invalid_empty = SchemaConfig {
            url: "".to_string(),
        };
        assert!(invalid_empty.validate().is_err());

        let invalid_protocol = SchemaConfig {
            url: "ftp://example.com/schema.json".to_string(),
        };
        assert!(invalid_protocol.validate().is_err());
    }

    #[test]
    fn test_diagnostics_is_disabled() {
        let mut config = DiagnosticsConfig::default();
        assert!(!config.is_disabled("deprecated_warning"));

        config.disabled.insert("deprecated_warning".to_string());
        assert!(config.is_disabled("deprecated_warning"));
        assert!(!config.is_disabled("type_error"));
    }

    #[test]
    fn test_config_merge() {
        let base = ServerConfig {
            logging: LoggingConfig {
                level: "info".to_string(),
                verbose: false,
                log_file: None,
            },
            completion: CompletionConfig {
                trigger_characters: vec!["[".to_string()],
            },
            diagnostics: DiagnosticsConfig {
                disabled: HashSet::new(),
            },
            schema: SchemaConfig {
                url: "https://default.com/schema.json".to_string(),
            },
        };

        let override_config = ServerConfig {
            logging: LoggingConfig {
                level: "debug".to_string(),
                verbose: true,
                log_file: Some(PathBuf::from("/tmp/test.log")),
            },
            completion: CompletionConfig {
                trigger_characters: vec!["[".to_string(), ".".to_string()],
            },
            diagnostics: DiagnosticsConfig {
                disabled: {
                    let mut set = HashSet::new();
                    set.insert("deprecated_warning".to_string());
                    set
                },
            },
            schema: SchemaConfig {
                url: "https://custom.com/schema.json".to_string(),
            },
        };

        let merged = base.merge(override_config);

        assert_eq!(merged.logging.level, "debug");
        assert!(merged.logging.verbose);
        assert_eq!(
            merged.logging.log_file,
            Some(PathBuf::from("/tmp/test.log"))
        );
        assert_eq!(merged.completion.trigger_characters.len(), 2);
        assert!(merged.diagnostics.is_disabled("deprecated_warning"));
        assert_eq!(merged.schema.url, "https://custom.com/schema.json");
    }

    #[test]
    fn test_env_overrides() {
        // 保存原始环境变量
        let original_level = env::var("SUMMER_LSP_LOG_LEVEL").ok();
        let original_verbose = env::var("SUMMER_LSP_VERBOSE").ok();
        let original_schema = env::var("SUMMER_LSP_SCHEMA_URL").ok();

        // 设置测试环境变量
        env::set_var("SUMMER_LSP_LOG_LEVEL", "trace");
        env::set_var("SUMMER_LSP_VERBOSE", "true");
        env::set_var("SUMMER_LSP_SCHEMA_URL", "https://test.com/schema.json");

        let config = ServerConfig::default().apply_env_overrides();

        assert_eq!(config.logging.level, "trace");
        assert!(config.logging.verbose);
        assert_eq!(config.schema.url, "https://test.com/schema.json");

        // 恢复原始环境变量
        match original_level {
            Some(v) => env::set_var("SUMMER_LSP_LOG_LEVEL", v),
            None => env::remove_var("SUMMER_LSP_LOG_LEVEL"),
        }
        match original_verbose {
            Some(v) => env::set_var("SUMMER_LSP_VERBOSE", v),
            None => env::remove_var("SUMMER_LSP_VERBOSE"),
        }
        match original_schema {
            Some(v) => env::set_var("SUMMER_LSP_SCHEMA_URL", v),
            None => env::remove_var("SUMMER_LSP_SCHEMA_URL"),
        }
    }

    #[test]
    fn test_load_from_toml() {
        let toml_content = r#"
[logging]
level = "debug"
verbose = true
log_file = "/tmp/summer-lsp.log"

[completion]
trigger_characters = ["[", ".", "$"]

[diagnostics]
disabled = ["deprecated_warning", "restful_style"]

[schema]
url = "https://custom.com/schema.json"
"#;

        let config: ServerConfig = toml::from_str(toml_content).unwrap();

        assert_eq!(config.logging.level, "debug");
        assert!(config.logging.verbose);
        assert_eq!(
            config.logging.log_file,
            Some(PathBuf::from("/tmp/summer-lsp.log"))
        );
        assert_eq!(config.completion.trigger_characters.len(), 3);
        assert!(config.diagnostics.is_disabled("deprecated_warning"));
        assert!(config.diagnostics.is_disabled("restful_style"));
        assert_eq!(config.schema.url, "https://custom.com/schema.json");
    }

    #[test]
    fn test_partial_toml_config() {
        // 测试部分配置（其他使用默认值）
        let toml_content = r#"
[logging]
level = "warn"

[schema]
url = "file:///local/schema.json"
"#;

        let config: ServerConfig = toml::from_str(toml_content).unwrap();

        assert_eq!(config.logging.level, "warn");
        assert!(!config.logging.verbose); // 默认值
        assert_eq!(config.completion.trigger_characters.len(), 6); // 默认值
        assert!(config.diagnostics.disabled.is_empty()); // 默认值
        assert_eq!(config.schema.url, "file:///local/schema.json");
    }

    #[test]
    fn test_config_validation() {
        let valid_config = ServerConfig::default();
        assert!(valid_config.validate().is_ok());

        let invalid_config = ServerConfig {
            logging: LoggingConfig {
                level: "invalid".to_string(),
                verbose: false,
                log_file: None,
            },
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }
}
