//! # summer-lsp
//!
//! Language Server Protocol implementation for summer-rs framework.
//!
//! summer-lsp 提供智能的开发体验，包括：
//! - TOML 配置文件的智能补全和验证
//! - Rust 宏的分析和展开
//! - 路由的识别和导航
//! - 依赖注入验证
//!
//! ## 架构
//!
//! summer-lsp 采用分层架构：
//! - **LSP Protocol Layer**: 处理 LSP 协议通信
//! - **Server Core Layer**: 消息分发和状态管理
//! - **Analysis Modules**: 各种分析功能模块
//! - **Foundation Layer**: 基础设施和工具
//!
//! ## 模块组织
//!
//! ```text
//! summer-lsp/
//! ├── protocol/          # LSP 协议层
//! │   ├── server.rs      # LSP 服务器核心
//! │   ├── handlers/      # 请求处理器
//! │   └── types.rs       # 协议类型定义
//! ├── analysis/          # 分析引擎层
//! │   ├── toml/          # TOML 分析
//! │   ├── rust/          # Rust 代码分析
//! │   ├── completion/    # 补全引擎
//! │   ├── diagnostic/    # 诊断引擎
//! │   └── validation/    # 验证引擎
//! ├── scanner/           # 扫描器层
//! │   ├── component.rs   # 组件扫描
//! │   ├── route.rs       # 路由扫描
//! │   ├── job.rs         # 任务扫描
//! │   ├── plugin.rs      # 插件扫描
//! │   └── config.rs      # 配置扫描
//! ├── core/              # 核心层
//! │   ├── document.rs    # 文档管理
//! │   ├── index.rs       # 符号索引
//! │   ├── schema.rs      # Schema 管理
//! │   └── config.rs      # 配置管理
//! └── utils/             # 工具层
//!     ├── error.rs       # 错误定义
//!     ├── logging.rs     # 日志系统
//!     └── status.rs      # 状态管理
//! ```

// ============================================================================
// 协议层 (Protocol Layer)
// ============================================================================
pub mod protocol {
    //! LSP 协议处理模块

    pub mod handlers;
    pub mod server;
    pub mod types;

    pub use server::LspServer;
}

// ============================================================================
// 分析层 (Analysis Layer)
// ============================================================================
pub mod analysis {
    //! 代码分析模块

    pub mod completion;
    pub mod diagnostic;
    pub mod rust;
    pub mod toml;
    pub mod validation;

    pub use completion::CompletionEngine;
    pub use diagnostic::DiagnosticEngine;
}

// ============================================================================
// 扫描器层 (Scanner Layer)
// ============================================================================
pub mod scanner {
    //! 项目扫描模块

    pub mod component;
    pub mod config;
    pub mod job;
    pub mod plugin;
    pub mod route;

    pub use component::ComponentScanner;
    pub use config::ConfigScanner;
    pub use job::JobScanner;
    pub use plugin::PluginScanner;
    pub use route::RouteScanner;
}

// ============================================================================
// 核心层 (Core Layer)
// ============================================================================
pub mod core {
    //! 核心功能模块

    pub mod config;
    pub mod document;
    pub mod index;
    pub mod schema;

    pub use document::DocumentManager;
    pub use index::SymbolIndex;
    pub use schema::SchemaProvider;
}

// ============================================================================
// 工具层 (Utils Layer)
// ============================================================================
pub mod utils {
    //! 工具和辅助模块

    pub mod error;
    pub mod logging;
    pub mod status;

    pub use error::{Error, Result};
    pub use logging::init_logging;
    pub use status::ServerStatus;
}

// ============================================================================
// 向后兼容的重导出
// ============================================================================

// 协议层
pub use protocol::server;
pub use protocol::LspServer;

// 分析层
pub use analysis::completion;
pub use analysis::diagnostic;
pub use analysis::rust::macro_analyzer;
pub use analysis::toml::toml_analyzer;
pub use analysis::validation::di_validator;

// 扫描器层
// 注意：这些是模块重导出，不是类型重导出
pub use scanner::component;
// config 模块与 core::config 冲突，使用别名
pub use scanner::config as scanner_config;
pub use scanner::job;
pub use scanner::plugin;
// route 模块与下面的 route 模块冲突，使用别名
pub use scanner::route as scanner_route;

// 核心层
pub use core::config;
pub use core::document;
pub use core::index;
pub use core::schema;

// 工具层
pub use utils::error;
pub use utils::logging;
pub use utils::status;

// 类型重导出
pub use utils::error::{Error, Result};

// 路由相关（保持向后兼容）
pub mod route {
    pub use crate::scanner::route::*;
}
