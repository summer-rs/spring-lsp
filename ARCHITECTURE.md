# Spring-LSP 架构文档

## 概述

summer-lsp 是为 summer-rs 框架提供 IDE 支持的语言服务器协议（LSP）实现。采用分层架构设计，代码按功能模块清晰组织，易于维护和扩展。

## 设计原则

1. **分层架构** - 清晰的层次划分，上层依赖下层
2. **模块化** - 每个模块职责单一，高内聚低耦合
3. **可扩展** - 易于添加新功能和新的分析器
4. **向后兼容** - 保持 API 稳定性
5. **类型安全** - 充分利用 Rust 的类型系统

## 模块组织

```
summer-lsp/src/
├── protocol/              # LSP 协议层
│   ├── server.rs          # LSP 服务器核心（消息循环、状态管理）
│   ├── handlers/          # 请求处理器
│   │   ├── custom.rs      # 自定义请求（spring/components 等）
│   │   ├── standard.rs    # 标准 LSP 请求（completion、hover 等）
│   │   └── mod.rs
│   ├── types.rs           # 共享协议类型（LocationResponse 等）
│   └── mod.rs
│
├── analysis/              # 分析引擎层
│   ├── toml/              # TOML 分析
│   │   ├── toml_analyzer.rs  # TOML 解析、环境变量处理
│   │   └── mod.rs
│   ├── rust/              # Rust 代码分析
│   │   ├── macro_analyzer.rs # Spring 宏识别和解析
│   │   ├── macro_analyzer/   # 宏分析器测试
│   │   │   └── tests.rs
│   │   └── mod.rs
│   ├── completion/        # 补全引擎
│   │   ├── engine_impl.rs    # 当前实现（完整功能）
│   │   ├── engine.rs         # 未来的简化实现（待完成）
│   │   ├── providers.rs      # 未来的提供器（待完成）
│   │   ├── tests.rs          # 补全测试
│   │   ├── README.md         # 重构计划
│   │   └── mod.rs
│   ├── diagnostic/        # 诊断引擎
│   │   ├── engine_impl.rs    # 当前实现（完整功能）
│   │   ├── engine.rs         # 未来的简化实现（待完成）
│   │   ├── validators.rs     # 未来的验证器（待完成）
│   │   ├── README.md         # 重构计划
│   │   └── mod.rs
│   ├── validation/        # 高级验证
│   │   ├── di_validator.rs   # 依赖注入验证（循环依赖检测）
│   │   └── mod.rs
│   └── mod.rs
│
├── scanner/               # 扫描器层
│   ├── component.rs       # 组件扫描（#[derive(Service)]）
│   ├── route.rs           # 路由扫描 + 路由类型定义
│   ├── job.rs             # 任务扫描（#[cron]、#[fix_delay]）
│   ├── plugin.rs          # 插件扫描（.add_plugin()）
│   ├── config.rs          # 配置扫描
│   └── mod.rs
│
├── core/                  # 核心层
│   ├── document.rs        # 文档管理（打开、修改、关闭）
│   ├── index.rs           # 符号索引（路由、组件、符号）
│   ├── schema.rs          # Schema 管理（配置验证）
│   ├── config.rs          # 服务器配置管理
│   └── mod.rs
│
├── utils/                 # 工具层
│   ├── error.rs           # 错误定义（Error、Result）
│   ├── logging.rs         # 日志系统（tracing 初始化）
│   ├── status.rs          # 服务器状态管理
│   └── mod.rs
│
├── lib.rs                 # 库入口（提供向后兼容的重导出）
└── main.rs                # 可执行文件入口
```

## 层次说明

### 1. 协议层 (Protocol Layer)

**职责：** LSP 协议通信和消息分发

**主要组件：**

#### server.rs
- LSP 服务器核心实现
- 消息循环处理（Request、Notification、Response）
- 服务器状态管理
- 文档同步（打开、修改、关闭）
- 能力声明（Capabilities）

**关键功能：**
```rust
pub struct LspServer {
    pub connection: Connection,
    pub document_manager: Arc<DocumentManager>,
    pub completion_engine: Arc<CompletionEngine>,
    pub diagnostic_engine: Arc<DiagnosticEngine>,
    pub schema_provider: Arc<SchemaProvider>,
    // ...
}
```

#### handlers/custom.rs
处理 summer-rs 特定的自定义请求：
- `spring/components` - 获取组件列表
- `spring/routes` - 获取路由列表
- `spring/jobs` - 获取任务列表
- `spring/plugins` - 获取插件列表
- `spring/configurations` - 获取配置列表

#### handlers/standard.rs
处理标准 LSP 请求：
- `textDocument/completion` - 代码补全
- `textDocument/hover` - 悬停提示
- `textDocument/definition` - 跳转到定义
- `textDocument/references` - 查找引用
- `textDocument/rename` - 重命名

#### types.rs
共享的协议类型定义：
```rust
pub struct LocationResponse {
    pub uri: String,
    pub range: RangeResponse,
}

pub struct RangeResponse {
    pub start: PositionResponse,
    pub end: PositionResponse,
}

pub struct PositionResponse {
    pub line: u32,
    pub character: u32,
}
```

### 2. 分析层 (Analysis Layer)

**职责：** 提供各种代码分析功能

**主要组件：**

#### toml/toml_analyzer.rs
- 使用 Taplo 解析 TOML 文件
- 环境变量插值处理 `${VAR:default}`
- 配置项提取和验证
- 支持 Schema 验证

**关键功能：**
```rust
pub struct TomlAnalyzer {
    schema_provider: SchemaProvider,
}

impl TomlAnalyzer {
    pub fn parse(&self, content: String) -> Result<TomlDocument>
    pub fn find_section_at_position(&self, doc: &TomlDocument, pos: Position) -> Option<String>
    pub fn get_property_value(&self, doc: &TomlDocument, path: &str) -> Option<Value>
}
```

#### rust/macro_analyzer.rs
- 使用 syn 解析 Rust 代码
- 识别 summer-rs 宏：
  - `#[derive(Service)]` - 组件定义
  - `#[get]`, `#[post]`, `#[put]`, `#[delete]` - 路由定义
  - `#[cron]`, `#[fix_delay]`, `#[fix_rate]` - 任务定义
  - `#[inject]` - 依赖注入
- 提取宏参数和元数据

**关键类型：**
```rust
pub enum SpringMacro {
    DeriveService(ServiceMacro),
    Route(RouteMacro),
    Job(JobMacro),
}

pub struct ServiceMacro {
    pub struct_name: String,
    pub fields: Vec<FieldInfo>,
    pub range: Range,
}
```

#### completion/engine_impl.rs
**当前使用的完整实现**

提供智能补全功能：
- TOML 配置补全（配置节、配置项、枚举值）
- Rust 宏参数补全
- 环境变量补全
- 上下文感知的补全

**关键功能：**
```rust
pub struct CompletionEngine {
    toml_analyzer: TomlAnalyzer,
}

impl CompletionEngine {
    pub fn complete_toml_document(
        &self,
        toml_doc: &TomlDocument,
        position: Position,
    ) -> Vec<CompletionItem>
    
    pub fn complete_macro_params(
        &self,
        macro_type: &str,
    ) -> Vec<CompletionItem>
}
```

**未来重构计划：**
- `engine.rs` - 核心补全引擎
- `providers.rs` - 各种补全提供器
  - TomlCompletionProvider
  - MacroCompletionProvider
  - EnvVarCompletionProvider

#### diagnostic/engine_impl.rs
**当前使用的完整实现**

提供诊断功能：
- 并发安全的诊断缓存（使用 DashMap）
- 诊断发布到客户端
- 支持增量更新

**关键功能：**
```rust
pub struct DiagnosticEngine {
    diagnostics: DashMap<Url, Vec<Diagnostic>>,
}

impl DiagnosticEngine {
    pub fn clear(&self, uri: &Url)
    pub fn add(&self, uri: Url, diagnostic: Diagnostic)
    pub fn publish(&self, connection: &Connection, uri: &Url) -> Result<()>
}
```

**未来重构计划：**
- `engine.rs` - 核心诊断引擎
- `validators.rs` - 各种验证器
  - TomlSyntaxValidator
  - SchemaValidator
  - RouteConflictValidator

#### validation/di_validator.rs
依赖注入验证：
- 组件注册验证
- 组件类型存在性验证
- 循环依赖检测（DFS 算法）
- 配置注入验证

**关键功能：**
```rust
pub struct DependencyInjectionValidator {
    index_manager: IndexManager,
}

impl DependencyInjectionValidator {
    pub fn validate(
        &self,
        rust_docs: &[RustDocument],
        toml_docs: &[(Url, TomlDocument)],
    ) -> Vec<Diagnostic>
    
    fn detect_circular_dependencies(&self, graph: &DependencyGraph) -> Vec<Vec<String>>
}
```

### 3. 扫描器层 (Scanner Layer)

**职责：** 扫描项目中的各种 summer-rs 元素

**主要组件：**

#### component.rs
扫描 `#[derive(Service)]` 组件：
```rust
pub struct ComponentScanner {
    macro_analyzer: MacroAnalyzer,
}

pub struct ComponentInfoResponse {
    pub name: String,
    pub type_name: String,
    pub scope: ComponentScope,
    pub dependencies: Vec<String>,
    pub location: LocationResponse,
}
```

#### route.rs
扫描路由定义 + 路由类型：
```rust
pub struct RouteScanner {
    macro_analyzer: MacroAnalyzer,
}

pub struct RouteInfoResponse {
    pub method: String,
    pub path: String,
    pub handler: String,
    pub is_openapi: bool,
    pub location: LocationResponse,
}

// 路由类型定义
pub struct RouteNavigator { /* ... */ }
pub struct RouteIndex { /* ... */ }
pub enum HttpMethod { GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS }
pub struct Route { /* ... */ }
```

#### job.rs
扫描定时任务：
```rust
pub struct JobScanner {
    macro_analyzer: MacroAnalyzer,
}

pub struct JobInfoResponse {
    pub name: String,
    pub job_type: JobType,  // Cron, FixDelay, FixRate
    pub schedule: String,
    pub location: LocationResponse,
}
```

#### plugin.rs
扫描插件注册：
```rust
pub struct PluginScanner;

pub struct PluginInfoResponse {
    pub name: String,
    pub type_name: String,
    pub config_prefix: Option<String>,
    pub location: LocationResponse,
}
```

#### config.rs
扫描配置文件：
```rust
pub struct ConfigScanner;

pub struct ConfigurationResponse {
    pub section: String,
    pub properties: Vec<PropertyInfo>,
    pub location: LocationResponse,
}
```

### 4. 核心层 (Core Layer)

**职责：** 提供核心功能支持

**主要组件：**

#### document.rs
文档管理：
```rust
pub struct DocumentManager {
    documents: DashMap<Url, String>,
}

impl DocumentManager {
    pub fn open(&self, uri: Url, content: String)
    pub fn update(&self, uri: &Url, content: String)
    pub fn close(&self, uri: &Url)
    pub fn get(&self, uri: &Url) -> Option<String>
}
```

#### index.rs
符号索引：
```rust
pub struct IndexManager {
    symbol_index: Arc<RwLock<SymbolIndex>>,
    route_index: Arc<RwLock<RouteIndex>>,
    component_index: Arc<RwLock<ComponentIndex>>,
}

impl IndexManager {
    pub fn rebuild(&self, root_uri: &Url, documents: &[(Url, String)])
    pub fn get_all_routes(&self) -> Vec<Route>
    pub fn get_all_components(&self) -> Vec<Component>
}
```

#### schema.rs
Schema 管理：
```rust
pub struct SchemaProvider {
    schema: ConfigSchema,
}

impl SchemaProvider {
    pub async fn load() -> Result<Self>
    pub fn get_plugin_schema(&self, prefix: &str) -> Option<&PluginSchema>
    pub fn get_property_schema(&self, prefix: &str, property: &str) -> Option<&PropertySchema>
}
```

#### config.rs
服务器配置：
```rust
pub struct ServerConfig {
    pub log_level: String,
    pub schema_url: Option<String>,
    // ...
}
```

### 5. 工具层 (Utils Layer)

**职责：** 提供通用工具和辅助功能

**主要组件：**

#### error.rs
错误定义：
```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Schema load error: {0}")]
    SchemaLoad(String),
    
    // ...
}

pub type Result<T> = std::result::Result<T, Error>;
```

#### logging.rs
日志系统：
```rust
pub fn init_logging() -> Result<()> {
    // 初始化 tracing-subscriber
    // 支持环境变量配置：
    // - SPRING_LSP_LOG_LEVEL
    // - SPRING_LSP_VERBOSE
    // - SPRING_LSP_LOG_FILE
}
```

#### status.rs
服务器状态：
```rust
pub struct ServerStatus {
    pub state: ServerState,
    pub capabilities: ServerCapabilities,
    pub workspace_folders: Vec<WorkspaceFolder>,
}

pub enum ServerState {
    Uninitialized,
    Initializing,
    Running,
    ShuttingDown,
}
```

## 数据流

### 典型的 LSP 请求处理流程

```
VSCode 扩展
    ↓ (LSP 请求: textDocument/completion)
协议层 (server.rs)
    ↓ (解析请求参数)
    ↓ (获取文档内容)
核心层 (document_manager)
    ↓ (返回文档内容)
分析层 (toml_analyzer)
    ↓ (解析 TOML)
分析层 (completion_engine)
    ↓ (生成补全项)
核心层 (schema_provider)
    ↓ (提供 Schema 信息)
协议层 (server.rs)
    ↓ (构建响应)
    ↓ (LSP 响应: CompletionList)
VSCode 扩展
```

### 自定义请求处理流程

```
VSCode 扩展
    ↓ (自定义请求: spring/routes)
协议层 (handlers/custom.rs)
    ↓ (解析请求参数)
扫描器层 (route_scanner)
    ↓ (扫描项目文件)
分析层 (macro_analyzer)
    ↓ (解析 Rust 宏)
扫描器层 (route_scanner)
    ↓ (构建路由列表)
协议层 (handlers/custom.rs)
    ↓ (构建响应)
    ↓ (自定义响应: RoutesResponse)
VSCode 扩展
```

### 诊断发布流程

```
文档变更事件
    ↓
协议层 (server.rs)
    ↓ (textDocument/didChange)
核心层 (document_manager)
    ↓ (更新文档内容)
分析层 (toml_analyzer)
    ↓ (解析并验证)
分析层 (diagnostic_engine)
    ↓ (收集诊断信息)
核心层 (schema_provider)
    ↓ (Schema 验证)
分析层 (diagnostic_engine)
    ↓ (发布诊断)
    ↓ (textDocument/publishDiagnostics)
VSCode 扩展
```

## 模块依赖关系

### 依赖图

```
protocol → analysis → scanner → core
   ↓          ↓         ↓         ↓
   └──────────┴─────────┴─────→ utils
```

### 详细依赖关系

```
protocol/
├── 依赖 analysis (completion, diagnostic, toml, rust)
├── 依赖 scanner (component, route, job, plugin, config)
├── 依赖 core (document, index, schema, config)
└── 依赖 utils (error, logging, status)

analysis/
├── toml/
│   ├── 依赖 core (schema)
│   └── 依赖 utils (error)
├── rust/
│   └── 依赖 utils (error)
├── completion/
│   ├── 依赖 analysis/toml
│   ├── 依赖 analysis/rust
│   ├── 依赖 core (schema)
│   └── 依赖 utils (error)
├── diagnostic/
│   └── 依赖 utils (error)
└── validation/
    ├── 依赖 analysis/toml
    ├── 依赖 analysis/rust
    ├── 依赖 core (index)
    └── 依赖 utils (error)

scanner/
├── 依赖 analysis/rust (macro_analyzer)
├── 依赖 protocol (types)
└── 依赖 utils (error)

core/
└── 依赖 utils (error)

utils/
└── 无外部依赖（基础层）
```

### 依赖规则

✅ **允许的依赖：**
- 上层可以依赖下层
- 所有层都可以使用 utils 层
- 同层模块可以相互依赖（但应尽量减少）

❌ **禁止的依赖：**
- 下层不能依赖上层
- 避免循环依赖
- 避免跨层依赖（例如 scanner 直接依赖 protocol）

### 特殊说明

1. **scanner 依赖 protocol/types**
   - 这是为了共享协议类型定义（LocationResponse 等）
   - 只依赖类型定义，不依赖协议逻辑
   - 未来可以考虑将这些类型移到 core 或 utils

2. **analysis/completion 依赖 analysis/toml 和 analysis/rust**
   - 补全引擎需要使用 TOML 和 Rust 分析器
   - 这是合理的同层依赖

3. **protocol 依赖多个层**
   - 作为顶层，协调各个模块
   - 这是分层架构的正常模式

## 向后兼容

为了保持向后兼容，`lib.rs` 中提供了旧模块路径的重导出：

### 旧路径（仍然可用）

```rust
// 协议层
use summer_lsp::server::LspServer;

// 分析层
use summer_lsp::completion::CompletionEngine;
use summer_lsp::diagnostic::DiagnosticEngine;
use summer_lsp::toml_analyzer::TomlAnalyzer;
use summer_lsp::macro_analyzer::MacroAnalyzer;

// 扫描器层
use summer_lsp::component_scanner::ComponentScanner;
use summer_lsp::route_scanner::RouteScanner;

// 核心层
use summer_lsp::document::DocumentManager;
use summer_lsp::schema::SchemaProvider;

// 工具层
use summer_lsp::error::{Error, Result};
use summer_lsp::logging::init_logging;

// 路由相关
use summer_lsp::route::{Route, RouteIndex, HttpMethod};
```

### 新路径（推荐）

```rust
// 协议层
use summer_lsp::protocol::LspServer;

// 分析层
use summer_lsp::analysis::CompletionEngine;
use summer_lsp::analysis::DiagnosticEngine;
use summer_lsp::analysis::toml::toml_analyzer::TomlAnalyzer;
use summer_lsp::analysis::rust::macro_analyzer::MacroAnalyzer;

// 扫描器层
use summer_lsp::scanner::ComponentScanner;
use summer_lsp::scanner::RouteScanner;

// 核心层
use summer_lsp::core::DocumentManager;
use summer_lsp::core::SchemaProvider;

// 工具层
use summer_lsp::utils::{Error, Result};
use summer_lsp::utils::init_logging;

// 路由相关
use summer_lsp::scanner::route::{Route, RouteIndex, HttpMethod};
```

### 迁移建议

1. **新代码** - 使用新路径
2. **现有代码** - 可以继续使用旧路径，但建议逐步迁移
3. **库 API** - 保持稳定，不会移除旧路径

## 扩展指南

### 添加新的扫描器

**场景：** 需要扫描新的 summer-rs 元素（例如中间件）

**步骤：**

1. **创建扫描器文件**
   ```bash
   # 在 scanner/ 目录创建新文件
   touch src/scanner/middleware.rs
   ```

2. **实现扫描器**
   ```rust
   // src/scanner/middleware.rs
   use crate::analysis::rust::macro_analyzer::{MacroAnalyzer, SpringMacro};
   use crate::protocol::types::{LocationResponse, PositionResponse, RangeResponse};
   
   pub struct MiddlewareScanner {
       macro_analyzer: MacroAnalyzer,
   }
   
   impl MiddlewareScanner {
       pub fn new() -> Self {
           Self {
               macro_analyzer: MacroAnalyzer::new(),
           }
       }
       
       pub fn scan_middlewares(&self, project_path: &Path) -> Result<Vec<MiddlewareInfo>> {
           // 实现扫描逻辑
       }
   }
   
   #[derive(Debug, Serialize, Deserialize)]
   pub struct MiddlewareInfo {
       pub name: String,
       pub priority: i32,
       pub location: LocationResponse,
   }
   ```

3. **在 mod.rs 中导出**
   ```rust
   // src/scanner/mod.rs
   pub mod middleware;
   pub use middleware::MiddlewareScanner;
   ```

4. **添加自定义请求处理**
   ```rust
   // src/protocol/handlers/custom.rs
   fn handle_middlewares_request(req: Request) -> Option<Response> {
       // 实现请求处理
   }
   ```

5. **在 server.rs 中注册**
   ```rust
   // src/protocol/server.rs
   "spring/middlewares" => self.handle_middlewares_request(req),
   ```

### 添加新的分析功能

**场景：** 需要添加新的代码分析功能（例如性能分析）

**步骤：**

1. **创建分析器目录**
   ```bash
   mkdir -p src/analysis/performance
   touch src/analysis/performance/mod.rs
   touch src/analysis/performance/analyzer.rs
   ```

2. **实现分析器**
   ```rust
   // src/analysis/performance/analyzer.rs
   pub struct PerformanceAnalyzer {
       // 字段
   }
   
   impl PerformanceAnalyzer {
       pub fn analyze(&self, code: &str) -> Vec<PerformanceIssue> {
           // 实现分析逻辑
       }
   }
   ```

3. **在 mod.rs 中导出**
   ```rust
   // src/analysis/performance/mod.rs
   mod analyzer;
   pub use analyzer::PerformanceAnalyzer;
   ```

4. **在 analysis/mod.rs 中添加**
   ```rust
   // src/analysis/mod.rs
   pub mod performance;
   ```

5. **在需要的地方使用**
   ```rust
   use crate::analysis::performance::PerformanceAnalyzer;
   ```

### 添加新的 LSP 功能

**场景：** 需要支持新的 LSP 请求（例如 Code Action）

**步骤：**

1. **在 handlers/standard.rs 中添加处理函数**
   ```rust
   // src/protocol/handlers/standard.rs
   fn handle_code_action(req: Request) -> Option<Response> {
       // 解析参数
       let params: CodeActionParams = serde_json::from_value(req.params)?;
       
       // 调用分析器
       let actions = analyze_code_actions(&params);
       
       // 构建响应
       let result = serde_json::to_value(actions).ok()?;
       Some(Response::new_ok(req.id, result))
   }
   ```

2. **在 handle_standard_request 中注册**
   ```rust
   pub fn handle_standard_request(req: Request) -> Option<Response> {
       match req.method.as_str() {
           "textDocument/codeAction" => handle_code_action(req),
           // ...
       }
   }
   ```

3. **在 server.rs 中声明能力**
   ```rust
   // src/protocol/server.rs
   let server_capabilities = ServerCapabilities {
       code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
       // ...
   };
   ```

### 添加新的验证器

**场景：** 需要添加新的代码验证规则

**步骤：**

1. **在 diagnostic/validators.rs 中添加**
   ```rust
   // src/analysis/diagnostic/validators.rs
   pub struct CustomValidator;
   
   impl CustomValidator {
       pub fn validate(&self, code: &str) -> Vec<Diagnostic> {
           // 实现验证逻辑
       }
   }
   ```

2. **在 diagnostic_engine 中集成**
   ```rust
   // src/analysis/diagnostic/engine_impl.rs
   use super::validators::CustomValidator;
   
   impl DiagnosticEngine {
       pub fn validate_with_custom(&self, uri: &Url, content: &str) {
           let validator = CustomValidator;
           let diagnostics = validator.validate(content);
           // 添加到诊断列表
       }
   }
   ```

## 测试策略

### 单元测试

每个模块都应该有自己的单元测试：

```rust
// src/scanner/component.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_component_scanner_new() {
        let scanner = ComponentScanner::new();
        assert!(scanner.macro_analyzer.is_some());
    }
    
    #[test]
    fn test_scan_components() {
        let scanner = ComponentScanner::new();
        let result = scanner.scan_components(Path::new("test_project"));
        assert!(result.is_ok());
    }
}
```

### 集成测试

在 `tests/` 目录中编写集成测试：

```rust
// tests/integration_test.rs
use summer_lsp::protocol::LspServer;
use lsp_types::*;

#[tokio::test]
async fn test_completion_in_config_file() {
    let server = TestServer::new().await;
    
    // 打开文档
    server.open_document(
        "file:///test/config/app.toml",
        r#"
[web]
host = "0.0.0.0"
|
        "#
    ).await;
    
    // 请求补全
    let completions = server.completion(
        "file:///test/config/app.toml",
        Position { line: 2, character: 0 }
    ).await;
    
    // 验证结果
    assert!(completions.iter().any(|c| c.label == "port"));
}
```

### 属性测试

使用 proptest 进行属性测试：

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_toml_parser_never_panics(content in "\\PC*") {
        // 解析器不应该对任何输入 panic
        let _ = parse_toml(&content);
    }
    
    #[test]
    fn test_route_path_parsing(
        path in "[a-z/{}]+",
        method in prop::sample::select(vec!["GET", "POST", "PUT", "DELETE"])
    ) {
        let route = Route {
            method: method.parse().unwrap(),
            path: path.clone(),
            handler: "test".to_string(),
            location: Location::default(),
        };
        
        // 路径解析应该是幂等的
        let parsed = parse_route_path(&route.path);
        let reparsed = parse_route_path(&format_route_path(&parsed));
        assert_eq!(parsed, reparsed);
    }
}
```

### 性能测试

```rust
#[test]
fn bench_completion_performance() {
    let content = generate_large_config_file(1000); // 1000 行配置
    
    let start = std::time::Instant::now();
    let completions = complete_at_position(&content, Position::default());
    let duration = start.elapsed();
    
    assert!(duration.as_millis() < 100, "Completion took too long: {:?}", duration);
    assert!(!completions.is_empty());
}
```

### 测试覆盖率

运行测试并生成覆盖率报告：

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 运行测试并生成覆盖率
cargo tarpaulin --out Html --output-dir coverage
```

## 性能考虑

### 1. 并发安全

使用并发安全的数据结构：

```rust
// 使用 DashMap 代替 Mutex<HashMap>
use dashmap::DashMap;

pub struct DiagnosticEngine {
    diagnostics: DashMap<Url, Vec<Diagnostic>>,  // 并发安全
}

// 使用 Arc<RwLock<T>> 用于读多写少的场景
pub struct IndexManager {
    route_index: Arc<RwLock<RouteIndex>>,  // 读多写少
}
```

### 2. 增量解析和更新

只解析变化的部分：

```rust
impl DocumentManager {
    pub fn update(&self, uri: &Url, changes: Vec<TextDocumentContentChangeEvent>) {
        // 增量更新，而不是重新解析整个文档
        for change in changes {
            self.apply_change(uri, change);
        }
    }
}
```

### 3. 延迟加载和按需分析

```rust
impl SchemaProvider {
    pub async fn load() -> Result<Self> {
        // 异步加载 Schema，不阻塞启动
        let schema = tokio::spawn(async {
            load_schema_from_url().await
        }).await?;
        
        Ok(Self { schema })
    }
}
```

### 4. 缓存策略

```rust
pub struct CompletionEngine {
    // 缓存解析结果
    cache: DashMap<Url, ParsedDocument>,
}

impl CompletionEngine {
    pub fn complete(&self, uri: &Url, content: &str) -> Vec<CompletionItem> {
        // 使用缓存避免重复解析
        let doc = self.cache.entry(uri.clone())
            .or_insert_with(|| parse_document(content));
        
        self.generate_completions(doc)
    }
}
```

### 5. 符号索引加速查找

```rust
pub struct SymbolIndex {
    // 使用 HashMap 加速符号查找
    symbols: HashMap<String, Symbol>,
    // 使用倒排索引加速搜索
    inverted_index: HashMap<String, Vec<String>>,
}
```

### 6. 避免不必要的克隆

```rust
// 使用引用而不是克隆
pub fn get_routes(&self) -> Vec<&Route> {
    self.routes.values().collect()  // 返回引用
}

// 使用 Arc 共享数据
pub struct LspServer {
    pub schema_provider: Arc<SchemaProvider>,  // 共享，不克隆
}
```

### 7. 批量操作

```rust
impl DiagnosticEngine {
    // 批量发布诊断，而不是逐个发布
    pub fn publish_all(&self, connection: &Connection) -> Result<()> {
        let diagnostics: Vec<_> = self.diagnostics.iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
        
        for (uri, diags) in diagnostics {
            self.publish_one(connection, &uri, diags)?;
        }
        Ok(())
    }
}
```

### 性能指标

**目标：**
- 补全响应时间 < 100ms
- 诊断响应时间 < 200ms
- 文档打开时间 < 50ms
- 内存使用 < 100MB（中等项目）

**监控：**
```rust
use std::time::Instant;

let start = Instant::now();
let result = expensive_operation();
let duration = start.elapsed();

tracing::debug!("Operation took {:?}", duration);
if duration.as_millis() > 100 {
    tracing::warn!("Slow operation detected: {:?}", duration);
}
```

## 未来计划

### 短期（1-3 个月）

#### 1. 完善补全引擎
- [ ] 重构 `completion/engine_impl.rs` 到模块化结构
- [ ] 实现 `TomlCompletionProvider`
- [ ] 实现 `MacroCompletionProvider`
- [ ] 实现 `EnvVarCompletionProvider`
- [ ] 添加更多上下文感知的补全

#### 2. 完善诊断引擎
- [ ] 重构 `diagnostic/engine_impl.rs` 到模块化结构
- [ ] 实现 `TomlSyntaxValidator`
- [ ] 实现 `SchemaValidator`
- [ ] 实现 `RouteConflictValidator`
- [ ] 添加更多诊断规则

#### 3. 实现处理器
- [ ] 完成 `protocol/handlers/custom.rs` 中的自定义请求处理
- [ ] 完成 `protocol/handlers/standard.rs` 中的标准 LSP 请求处理
- [ ] 实现 Hover 提示
- [ ] 实现 Go to Definition
- [ ] 实现 Find References

### 中期（3-6 个月）

#### 1. 代码重构功能
- [ ] 实现 Rename
- [ ] 实现 Code Action
- [ ] 实现 Extract Variable/Function
- [ ] 实现 Inline Variable/Function

#### 2. 更多 summer-rs 支持
- [ ] 支持更多 summer-rs 宏
- [ ] 支持中间件分析
- [ ] 支持拦截器分析
- [ ] 支持事件监听器分析

#### 3. 性能优化
- [ ] 实现增量解析
- [ ] 优化缓存策略
- [ ] 减少不必要的克隆
- [ ] 实现并行分析

### 长期（6-12 个月）

#### 1. 高级功能
- [ ] 代码生成（生成路由、组件等）
- [ ] 重构工具（重构路由、组件等）
- [ ] 代码片段（Snippets）
- [ ] 项目模板

#### 2. 集成和工具
- [ ] 集成到 rust-analyzer
- [ ] 支持更多编辑器（Neovim、Emacs 等）
- [ ] 命令行工具
- [ ] CI/CD 集成

#### 3. 文档和社区
- [ ] 完善文档
- [ ] 添加示例
- [ ] 编写教程
- [ ] 建立社区

### 持续改进

- [ ] 更新测试覆盖率
- [ ] 性能基准测试
- [ ] 用户反馈收集
- [ ] Bug 修复
- [ ] 依赖更新


## 常见问题

### Q: 为什么有 engine_impl.rs 和 engine.rs 两个文件？

**A:** 这是重构过程中的过渡状态：
- `engine_impl.rs` - 当前使用的完整实现，功能稳定
- `engine.rs` - 未来的模块化实现，待完成

我们保留两者是为了：
1. 确保功能稳定（使用 engine_impl.rs）
2. 为未来重构做准备（engine.rs 作为目标）
3. 允许逐步迁移，而不是一次性重写

### Q: 为什么 scanner 依赖 protocol/types？

**A:** 为了共享协议类型定义（LocationResponse 等），避免重复定义。这些类型在多个扫描器中使用，统一定义可以：
1. 避免代码重复
2. 保证类型一致性
3. 简化维护

未来可以考虑将这些类型移到 core 或 utils 层。

### Q: 如何添加新的 summer-rs 宏支持？

**A:** 
1. 在 `analysis/rust/macro_analyzer.rs` 中添加宏识别逻辑
2. 在 `SpringMacro` 枚举中添加新的变体
3. 创建对应的扫描器（如果需要）
4. 在 `protocol/handlers/custom.rs` 中添加请求处理

### Q: 如何调试 LSP 服务器？

**A:**
1. 设置日志级别：`SPRING_LSP_LOG_LEVEL=debug`
2. 启用详细日志：`SPRING_LSP_VERBOSE=1`
3. 指定日志文件：`SPRING_LSP_LOG_FILE=/tmp/summer-lsp.log`
4. 在 VSCode 中查看 LSP 日志：View -> Output -> Summer LSP

### Q: 性能问题如何排查？

**A:**
1. 使用 `tracing::debug!` 记录关键操作的耗时
2. 使用 `cargo flamegraph` 生成火焰图
3. 检查是否有不必要的克隆或重复解析
4. 考虑添加缓存或使用增量更新

## 开发工作流

### 本地开发

```bash
# 克隆仓库
git clone https://github.com/summer-rs/summer-lsp.git
cd summer-lsp

# 安装依赖（Rust 工具链）
rustup update stable

# 运行测试
cargo test

# 运行 LSP 服务器
cargo run

# 构建 release 版本
cargo build --release

# 检查代码
cargo clippy

# 格式化代码
cargo fmt
```

### VSCode 扩展开发

```bash
# 进入 VSCode 扩展目录
cd vscode

# 安装依赖
npm install

# 编译 TypeScript
npm run compile

# 监听文件变化
npm run watch

# 打包扩展
npm run vscode:prepublish

# 本地测试（按 F5 启动调试）
```

### 调试技巧

1. **使用 rust-analyzer**
   - 安装 rust-analyzer 扩展
   - 自动补全和类型检查
   - 快速导航和重构

2. **使用 tracing**
   ```rust
   tracing::debug!("Processing request: {:?}", req);
   tracing::info!("Completion items: {}", items.len());
   tracing::error!("Failed to parse: {}", error);
   ```

3. **使用 VSCode 调试器**
   - 在 `.vscode/launch.json` 中配置
   - 设置断点
   - 查看变量和调用栈

## 贡献指南

### 代码风格

- 遵循 Rust 官方风格指南
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 为公共 API 添加文档注释

### 提交规范

```
<type>(<scope>): <subject>

<body>

<footer>
```

**类型：**
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `test`: 测试
- `chore`: 构建或辅助工具

**示例：**
```
feat(completion): add environment variable completion

- Add EnvVarCompletionProvider
- Support ${VAR:default} syntax
- Add tests for env var completion

Closes #123
```

### Pull Request 流程

1. Fork 仓库
2. 创建特性分支：`git checkout -b feature/my-feature`
3. 提交更改：`git commit -am 'feat: add my feature'`
4. 推送分支：`git push origin feature/my-feature`
5. 创建 Pull Request
6. 等待代码审查
7. 根据反馈修改
8. 合并到主分支

## 相关资源

### 文档

- [README.md](./README.md) - 项目说明
- [REFACTORING_COMPLETE.md](./REFACTORING_COMPLETE.md) - 重构完成报告
- [MODULE_REFACTORING.md](./MODULE_REFACTORING.md) - 模块重构总结
- [CLEANUP_SUMMARY.md](./CLEANUP_SUMMARY.md) - 代码清理总结

### 外部资源

- [LSP 规范](https://microsoft.github.io/language-server-protocol/) - LSP 协议文档
- [rust-analyzer](https://github.com/rust-lang/rust-analyzer) - Rust LSP 实现参考
- [taplo](https://taplo.tamasfe.dev/) - TOML 工具包
- [syn](https://docs.rs/syn/) - Rust 语法解析库
- [summer-rs](https://summer-rs.github.io/) - summer-rs 框架文档

### 社区

- GitHub Issues - 报告 Bug 和功能请求
- GitHub Discussions - 讨论和问答
- Discord - 实时交流（如果有）

## 许可证

本项目采用 MIT 或 Apache-2.0 双重许可。

---

**最后更新：** 2024-02-07  
**版本：** 0.1.0  
**维护者：** summer-rs contributors
