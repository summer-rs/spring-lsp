//! LSP 服务器核心实现
//!
//! 本模块实现了 summer-lsp 的核心 LSP 服务器功能，包括：
//!
//! ## 服务器能力
//!
//! ### 文档同步 (Text Document Sync)
//! - 支持文档打开、修改、关闭通知
//! - 使用增量更新模式 (INCREMENTAL) 提高性能
//! - 自动缓存和管理文档内容
//!
//! ### 智能补全 (Completion)
//! - TOML 配置文件：配置节、配置项、枚举值补全
//! - Rust 代码：宏参数补全
//! - 环境变量：`${VAR:default}` 格式的环境变量补全
//! - 触发字符：`[`, `.`, `$`, `{`, `#`, `(`
//!
//! ### 悬停提示 (Hover)
//! - 配置项：显示类型、文档、默认值
//! - 宏：显示宏展开后的代码
//! - 路由：显示完整路径和 HTTP 方法
//! - 环境变量：显示当前值（如果可用）
//!
//! ### 定义跳转 (Go to Definition)
//! - 路由路径：跳转到处理器函数定义
//! - 组件注入：跳转到组件定义
//!
//! ### 文档符号 (Document Symbols)
//! - 显示文档中的所有路由
//! - 显示配置节和配置项
//!
//! ### 工作空间符号 (Workspace Symbols)
//! - 全局搜索路由
//! - 全局搜索组件
//!
//! ### 诊断 (Diagnostics)
//! - 配置验证：类型检查、必需项检查、废弃警告
//! - 路由验证：路径语法、参数类型、冲突检测
//! - 依赖注入验证：组件存在性、循环依赖检测
//!
//! ## LSP 协议版本
//!
//! 本实现遵循 LSP 3.17 规范。

use crate::analysis::completion::CompletionEngine;
use crate::analysis::diagnostic::DiagnosticEngine;
use crate::analysis::rust::macro_analyzer::MacroAnalyzer;
use crate::analysis::toml::toml_analyzer::TomlAnalyzer;
use crate::core::config::ServerConfig;
use crate::core::document::DocumentManager;
use crate::core::index::IndexManager;
use crate::core::schema::SchemaProvider;
use crate::scanner::route::RouteNavigator;
use crate::utils::error::{ErrorHandler, RecoveryAction};
use crate::utils::status::ServerStatus;
use crate::{Error, Result};
use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Exit, Notification as _,
    },
    request::{Completion, DocumentSymbolRequest, GotoDefinition, HoverRequest, Request as _},
    CompletionParams, CompletionResponse, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DocumentSymbolParams, DocumentSymbolResponse, GotoDefinitionParams,
    GotoDefinitionResponse, HoverParams, InitializeParams, InitializeResult, ServerCapabilities,
    ServerInfo,
};
use std::sync::Arc;

/// 服务器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// 未初始化
    Uninitialized,
    /// 已初始化
    Initialized,
    /// 正在关闭
    ShuttingDown,
}

/// LSP 服务器
pub struct LspServer {
    /// LSP 连接
    connection: Connection,
    /// 服务器状态
    pub state: ServerState,
    /// 工作空间路径
    pub workspace_path: Option<std::path::PathBuf>,
    /// 文档管理器
    pub document_manager: Arc<DocumentManager>,
    /// 错误处理器
    error_handler: ErrorHandler,
    /// 服务器配置
    pub config: ServerConfig,
    /// 服务器状态跟踪器
    pub status: ServerStatus,
    /// Schema 提供者
    pub schema_provider: Arc<SchemaProvider>,
    /// TOML 分析器
    pub toml_analyzer: Arc<TomlAnalyzer>,
    /// 宏分析器
    pub macro_analyzer: Arc<MacroAnalyzer>,
    /// 路由导航器
    pub route_navigator: Arc<RouteNavigator>,
    /// 补全引擎
    pub completion_engine: Arc<CompletionEngine>,
    /// 诊断引擎
    pub diagnostic_engine: Arc<DiagnosticEngine>,
    /// 索引管理器
    pub index_manager: Arc<IndexManager>,
}

impl LspServer {
    /// 启动 LSP 服务器
    ///
    /// 这个方法创建服务器实例并初始化 LSP 连接
    pub fn start() -> Result<Self> {
        tracing::info!("Starting summer-lsp server");

        // 通过标准输入输出创建 LSP 连接
        let (connection, _io_threads) = Connection::stdio();

        Self::new_with_connection(connection)
    }

    /// 为测试创建 LSP 服务器（不使用 stdio 连接）
    pub fn new_for_test() -> Result<Self> {
        // 创建一个假的连接，用于测试
        // 我们不会实际使用这个连接发送消息
        let (connection, _io_threads) = Connection::memory();

        Self::new_with_connection(connection)
    }

    /// 使用给定连接创建服务器实例
    fn new_with_connection(connection: Connection) -> Result<Self> {
        // 加载默认配置（在初始化时会从客户端获取工作空间路径并重新加载）
        let config = ServerConfig::load(None);

        // 验证配置
        if let Err(e) = config.validate() {
            tracing::error!("Invalid configuration: {}", e);
            return Err(Error::Config(e));
        }

        // 从配置读取是否启用详细日志
        let verbose = config.logging.verbose;

        // 初始化所有组件
        tracing::info!("Initializing components...");

        // 1. Schema 提供者（同步加载完整 Schema）
        tracing::info!("Loading configuration schema...");
        let schema_provider = Arc::new({
            // 使用 tokio 运行时同步加载 Schema
            let runtime = tokio::runtime::Runtime::new()
                .map_err(|e| Error::SchemaLoad(format!("Failed to create tokio runtime: {}", e)))?;

            match runtime.block_on(SchemaProvider::load()) {
                Ok(provider) => {
                    tracing::info!("Schema loaded successfully from URL");
                    provider
                }
                Err(e) => {
                    tracing::warn!("Failed to load schema from URL: {}, using fallback", e);
                    SchemaProvider::default()
                }
            }
        });

        // 2. TOML 分析器
        let toml_analyzer = Arc::new(TomlAnalyzer::new((*schema_provider).clone()));

        // 3. 宏分析器
        let macro_analyzer = Arc::new(MacroAnalyzer::new());

        // 4. 路由导航器
        let route_navigator = Arc::new(RouteNavigator::new());

        // 5. 补全引擎
        let completion_engine = Arc::new(CompletionEngine::new((*schema_provider).clone()));

        // 6. 诊断引擎
        let diagnostic_engine = Arc::new(DiagnosticEngine::new());

        // 7. 索引管理器
        let index_manager = Arc::new(IndexManager::new());

        tracing::info!("All components initialized successfully");

        Ok(Self {
            connection,
            state: ServerState::Uninitialized,
            workspace_path: None,
            document_manager: Arc::new(DocumentManager::new()),
            error_handler: ErrorHandler::new(verbose),
            config,
            status: ServerStatus::new(),
            schema_provider,
            toml_analyzer,
            macro_analyzer,
            route_navigator,
            completion_engine,
            diagnostic_engine,
            index_manager,
        })
    }

    /// 运行服务器主循环
    ///
    /// 这个方法处理初始化握手，然后进入主事件循环处理来自客户端的消息
    pub fn run(&mut self) -> Result<()> {
        // 处理初始化握手
        self.initialize()?;

        // 主事件循环
        self.event_loop()?;

        // 优雅关闭
        self.shutdown()?;

        Ok(())
    }

    /// 处理初始化握手
    fn initialize(&mut self) -> Result<()> {
        tracing::info!("Waiting for initialize request");

        let (id, params) = self.connection.initialize_start()?;
        let init_params: InitializeParams = serde_json::from_value(params)?;

        tracing::info!(
            "Received initialize request from client: {:?}",
            init_params.client_info
        );

        let init_result = self.handle_initialize(init_params)?;
        let init_result_json = serde_json::to_value(init_result)?;

        self.connection.initialize_finish(id, init_result_json)?;

        self.state = ServerState::Initialized;
        tracing::info!("LSP server initialized successfully");

        Ok(())
    }

    /// 主事件循环
    ///
    /// 处理来自客户端的所有消息，包括请求、响应和通知
    fn event_loop(&mut self) -> Result<()> {
        tracing::info!("Entering main event loop");

        loop {
            // 检查服务器状态
            if self.state == ServerState::ShuttingDown {
                tracing::info!("Server is shutting down, stopping event loop");
                break;
            }

            // 接收消息
            let msg = match self.connection.receiver.recv() {
                Ok(msg) => msg,
                Err(e) => {
                    let error = Error::MessageReceive(e.to_string());
                    let result = self.error_handler.handle(&error);

                    match result.action {
                        RecoveryAction::RetryConnection => {
                            tracing::info!("Attempting to recover connection...");
                            // 短暂等待后继续
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            continue;
                        }
                        RecoveryAction::Abort => {
                            tracing::error!("Fatal error receiving message, shutting down");
                            break;
                        }
                        _ => {
                            tracing::warn!("Unexpected recovery action for message receive error");
                            break;
                        }
                    }
                }
            };

            // 处理消息，捕获错误以保持服务器运行
            if let Err(e) = self.handle_message(msg) {
                // 记录错误
                self.status.record_error();

                let result = self.error_handler.handle(&e);

                // 根据恢复策略决定是否继续
                match result.action {
                    RecoveryAction::Abort => {
                        tracing::error!("Fatal error, shutting down server");
                        self.state = ServerState::ShuttingDown;
                        break;
                    }
                    _ => {
                        // 其他错误继续运行
                        if result.notify_client {
                            // 向客户端发送错误通知
                            if let Err(notify_err) = self.notify_client_error(&e) {
                                tracing::error!(
                                    "Failed to notify client about error: {}",
                                    notify_err
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 处理单个消息
    fn handle_message(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Request(req) => self.handle_request(req),
            Message::Response(resp) => {
                tracing::debug!("Received response: {:?}", resp.id);
                // 响应消息通常不需要处理
                Ok(())
            }
            Message::Notification(not) => self.handle_notification(not),
        }
    }

    /// 处理请求
    fn handle_request(&mut self, req: Request) -> Result<()> {
        tracing::debug!("Received request: {} (id: {:?})", req.method, req.id);

        // 记录请求
        self.status.record_request();

        // 处理关闭请求
        if self.connection.handle_shutdown(&req)? {
            tracing::info!("Received shutdown request");
            self.state = ServerState::ShuttingDown;
            return Ok(());
        }

        // 根据请求方法分发
        match req.method.as_str() {
            // 智能补全请求
            Completion::METHOD => self.handle_completion(req),
            // 悬停提示请求
            HoverRequest::METHOD => self.handle_hover(req),
            // 定义跳转请求
            GotoDefinition::METHOD => self.handle_goto_definition(req),
            // 文档符号请求
            DocumentSymbolRequest::METHOD => self.handle_document_symbol(req),
            // 工作空间符号请求
            "workspace/symbol" => self.handle_workspace_symbol(req),
            // 状态查询请求
            "summer-lsp/status" => self.handle_status_query(req),
            // 自定义请求：获取组件列表
            "summer/components" => self.handle_components_request(req),
            // 自定义请求：获取路由列表
            "summer/routes" => self.handle_routes_request(req),
            // 自定义请求：获取任务列表
            "summer/jobs" => self.handle_jobs_request(req),
            // 自定义请求：获取插件列表
            "summer/plugins" => self.handle_plugins_request(req),
            // 自定义请求：获取配置列表
            "summer/configurations" => self.handle_configurations_request(req),
            _ => {
                tracing::warn!("Unhandled request method: {}", req.method);
                // 返回方法未实现错误
                self.send_error_response(
                    req.id,
                    lsp_server::ErrorCode::MethodNotFound as i32,
                    format!("Method not found: {}", req.method),
                )
            }
        }?;

        Ok(())
    }

    /// 处理通知
    fn handle_notification(&mut self, not: Notification) -> Result<()> {
        tracing::debug!("Received notification: {}", not.method);

        match not.method.as_str() {
            DidOpenTextDocument::METHOD => {
                let params: DidOpenTextDocumentParams = serde_json::from_value(not.params)?;
                self.handle_did_open(params)?;
            }
            DidChangeTextDocument::METHOD => {
                let params: DidChangeTextDocumentParams = serde_json::from_value(not.params)?;
                self.handle_did_change(params)?;
            }
            DidCloseTextDocument::METHOD => {
                let params: DidCloseTextDocumentParams = serde_json::from_value(not.params)?;
                self.handle_did_close(params)?;
            }
            Exit::METHOD => {
                tracing::info!("Received exit notification");
                self.state = ServerState::ShuttingDown;
            }
            _ => {
                tracing::debug!("Unhandled notification method: {}", not.method);
            }
        }

        Ok(())
    }

    /// 处理文档打开通知
    pub fn handle_did_open(&mut self, params: DidOpenTextDocumentParams) -> Result<()> {
        let doc = params.text_document;
        tracing::info!("Document opened: {}", doc.uri);

        self.document_manager.open(
            doc.uri.clone(),
            doc.version,
            doc.text,
            doc.language_id.clone(),
        );

        // 更新状态
        self.status.increment_document_count();

        // 触发文档分析和诊断
        self.analyze_document(&doc.uri, &doc.language_id)?;

        Ok(())
    }

    /// 处理文档修改通知
    pub fn handle_did_change(&mut self, params: DidChangeTextDocumentParams) -> Result<()> {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        tracing::debug!("Document changed: {} (version: {})", uri, version);

        self.document_manager
            .change(&uri, version, params.content_changes);

        // 触发增量分析和诊断
        if let Some(doc) = self.document_manager.get(&uri) {
            self.analyze_document(&uri, &doc.language_id)?;
        }

        Ok(())
    }

    /// 处理文档关闭通知
    pub fn handle_did_close(&mut self, params: DidCloseTextDocumentParams) -> Result<()> {
        let uri = params.text_document.uri;
        tracing::info!("Document closed: {}", uri);

        self.document_manager.close(&uri);

        // 更新状态
        self.status.decrement_document_count();

        // 清理相关的诊断和缓存
        self.diagnostic_engine.clear(&uri);
        let _ = self.diagnostic_engine.publish(&self.connection, &uri);

        Ok(())
    }

    /// 处理智能补全请求
    fn handle_completion(&mut self, req: Request) -> Result<()> {
        tracing::debug!("Handling completion request");

        let params: CompletionParams = serde_json::from_value(req.params)?;
        self.status.record_completion();

        let response = self.document_manager.with_document(
            &params.text_document_position.text_document.uri,
            |doc| {
                // 根据文件类型选择补全策略
                match doc.language_id.as_str() {
                    "toml" => {
                        if let Ok(toml_doc) = self.toml_analyzer.parse(&doc.content) {
                            self.completion_engine.complete_toml_document(
                                &toml_doc,
                                params.text_document_position.position,
                            )
                        } else {
                            vec![]
                        }
                    }
                    "rust" => {
                        // TODO: 实现 Rust 补全
                        vec![]
                    }
                    _ => vec![],
                }
            },
        );

        let result = match response {
            Some(completions) => serde_json::to_value(CompletionResponse::Array(completions))?,
            None => serde_json::Value::Null,
        };

        let response = Response {
            id: req.id,
            result: Some(result),
            error: None,
        };

        self.connection
            .sender
            .send(Message::Response(response))
            .map_err(|e| Error::MessageSend(e.to_string()))?;

        Ok(())
    }

    /// 处理悬停提示请求
    fn handle_hover(&mut self, req: Request) -> Result<()> {
        tracing::debug!("Handling hover request");

        let params: HoverParams = serde_json::from_value(req.params)?;
        self.status.record_hover();

        let response = self.document_manager.with_document(
            &params.text_document_position_params.text_document.uri,
            |doc| {
                // 根据文件类型选择分析器
                match doc.language_id.as_str() {
                    "toml" => {
                        if let Ok(toml_doc) = self.toml_analyzer.parse(&doc.content) {
                            self.toml_analyzer
                                .hover(&toml_doc, params.text_document_position_params.position)
                        } else {
                            None
                        }
                    }
                    "rust" => {
                        // TODO: 实现 Rust 悬停提示
                        None
                    }
                    _ => None,
                }
            },
        );

        let result = match response {
            Some(Some(hover)) => serde_json::to_value(hover)?,
            _ => serde_json::Value::Null,
        };

        let response = Response {
            id: req.id,
            result: Some(result),
            error: None,
        };

        self.connection
            .sender
            .send(Message::Response(response))
            .map_err(|e| Error::MessageSend(e.to_string()))?;

        Ok(())
    }

    /// 处理定义跳转请求
    fn handle_goto_definition(&mut self, req: Request) -> Result<()> {
        tracing::debug!("Handling goto definition request");

        let _params: GotoDefinitionParams = serde_json::from_value(req.params)?;

        // TODO: 实现定义跳转逻辑
        let result = GotoDefinitionResponse::Array(vec![]);

        let response = Response {
            id: req.id,
            result: Some(serde_json::to_value(result)?),
            error: None,
        };

        self.connection
            .sender
            .send(Message::Response(response))
            .map_err(|e| Error::MessageSend(e.to_string()))?;

        Ok(())
    }

    /// 分析文档并生成诊断
    pub fn analyze_document(&mut self, uri: &lsp_types::Url, language_id: &str) -> Result<()> {
        tracing::debug!("Analyzing document: {} ({})", uri, language_id);

        // 清除旧的诊断
        self.diagnostic_engine.clear(uri);

        let diagnostics = self
            .document_manager
            .with_document(uri, |doc| {
                match language_id {
                    "toml" => {
                        // TOML 文档分析
                        match self.toml_analyzer.parse(&doc.content) {
                            Ok(toml_doc) => {
                                let mut diagnostics = Vec::new();

                                // 配置验证
                                let validation_diagnostics = self.toml_analyzer.validate(&toml_doc);
                                diagnostics.extend(validation_diagnostics);

                                diagnostics
                            }
                            Err(e) => {
                                // 解析错误 - 显示详细错误信息
                                tracing::error!("TOML parse error: {}", e);
                                vec![lsp_types::Diagnostic {
                                    range: lsp_types::Range {
                                        start: lsp_types::Position {
                                            line: 0,
                                            character: 0,
                                        },
                                        end: lsp_types::Position {
                                            line: 0,
                                            character: 0,
                                        },
                                    },
                                    severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                                    code: Some(lsp_types::NumberOrString::String(
                                        "parse_error".to_string(),
                                    )),
                                    code_description: None,
                                    source: Some("summer-lsp".to_string()),
                                    message: format!("TOML parse error: {}", e),
                                    related_information: None,
                                    tags: None,
                                    data: None,
                                }]
                            }
                        }
                    }
                    "rust" => {
                        // Rust 文档分析
                        // TODO: 实现完整的 Rust 分析
                        vec![]
                    }
                    _ => {
                        tracing::debug!("Unsupported language: {}", language_id);
                        vec![]
                    }
                }
            })
            .unwrap_or_default();

        // 过滤被禁用的诊断
        let filtered_diagnostics: Vec<_> = diagnostics
            .into_iter()
            .filter(|diag| {
                if let Some(lsp_types::NumberOrString::String(code)) = &diag.code {
                    !self.config.diagnostics.is_disabled(code)
                } else {
                    true
                }
            })
            .collect();

        // 添加诊断
        for diagnostic in filtered_diagnostics {
            self.diagnostic_engine.add(uri.clone(), diagnostic);
        }

        // 发布诊断
        let _ = self.diagnostic_engine.publish(&self.connection, uri);
        self.status.record_diagnostic();

        Ok(())
    }

    /// 处理状态查询请求
    ///
    /// 返回服务器的运行状态和性能指标
    fn handle_status_query(&self, req: Request) -> Result<()> {
        tracing::debug!("Handling status query request");

        let metrics = self.status.get_metrics();
        let result = serde_json::to_value(metrics)?;

        let response = Response {
            id: req.id,
            result: Some(result),
            error: None,
        };

        self.connection
            .sender
            .send(Message::Response(response))
            .map_err(|e| Error::MessageSend(e.to_string()))?;

        Ok(())
    }

    /// 处理 summer/routes 请求
    ///
    /// 扫描项目中的所有路由并返回路由列表
    fn handle_routes_request(&self, req: Request) -> Result<()> {
        tracing::info!("Handling summer/routes request");

        use crate::scanner::route::{RouteScanner, RoutesRequest, RoutesResponse};

        // 解析请求参数
        let params: RoutesRequest = serde_json::from_value(req.params)?;
        let project_path = std::path::Path::new(&params.app_path);

        tracing::info!("Scanning routes in: {:?}", project_path);

        // 创建路由扫描器
        let scanner = RouteScanner::new();

        // 扫描路由
        let routes = match scanner.scan_routes(project_path) {
            Ok(routes) => {
                tracing::info!("Successfully scanned {} routes", routes.len());
                routes
            }
            Err(e) => {
                tracing::error!("Failed to scan routes: {}", e);
                // 返回空列表而不是错误
                Vec::new()
            }
        };

        // 构建响应
        let response_data = RoutesResponse { routes };

        tracing::info!(
            "Sending response with {} routes",
            response_data.routes.len()
        );

        let result = serde_json::to_value(response_data)?;

        let response = Response {
            id: req.id,
            result: Some(result),
            error: None,
        };

        self.connection
            .sender
            .send(Message::Response(response))
            .map_err(|e| Error::MessageSend(e.to_string()))?;

        Ok(())
    }

    /// 处理 summer/components 请求
    ///
    /// 扫描项目中的所有组件并返回组件列表
    fn handle_components_request(&self, req: Request) -> Result<()> {
        tracing::info!("Handling summer/components request");

        use crate::scanner::component::{ComponentScanner, ComponentsRequest, ComponentsResponse};

        // 解析请求参数
        let params: ComponentsRequest = serde_json::from_value(req.params)?;
        let project_path = std::path::Path::new(&params.app_path);

        tracing::info!("Scanning components in: {:?}", project_path);
        tracing::info!("Project path exists: {}", project_path.exists());
        tracing::info!("Project path is dir: {}", project_path.is_dir());

        // 创建组件扫描器
        let scanner = ComponentScanner::new();

        // 扫描组件
        let components = match scanner.scan_components(project_path) {
            Ok(components) => {
                tracing::info!("Successfully scanned {} components", components.len());
                components
            }
            Err(e) => {
                tracing::error!("Failed to scan components: {}", e);
                tracing::error!("Error details: {:?}", e);
                // 返回空列表而不是错误
                Vec::new()
            }
        };

        // 构建响应
        let response_data = ComponentsResponse { components };

        tracing::info!(
            "Sending response with {} components",
            response_data.components.len()
        );

        let result = serde_json::to_value(response_data)?;

        let response = Response {
            id: req.id,
            result: Some(result),
            error: None,
        };

        self.connection
            .sender
            .send(Message::Response(response))
            .map_err(|e| Error::MessageSend(e.to_string()))?;

        Ok(())
    }

    /// 处理 summer/jobs 请求
    ///
    /// 扫描项目中的所有定时任务并返回任务列表
    fn handle_jobs_request(&self, req: Request) -> Result<()> {
        tracing::info!("Handling summer/jobs request");

        use crate::scanner::job::{JobScanner, JobsRequest, JobsResponse};

        // 解析请求参数
        let params: JobsRequest = serde_json::from_value(req.params)?;
        let project_path = std::path::Path::new(&params.app_path);

        tracing::info!("Scanning jobs in: {:?}", project_path);

        // 创建任务扫描器
        let scanner = JobScanner::new();

        // 扫描任务
        let jobs = match scanner.scan_jobs(project_path) {
            Ok(jobs) => {
                tracing::info!("Successfully scanned {} jobs", jobs.len());
                jobs
            }
            Err(e) => {
                tracing::error!("Failed to scan jobs: {}", e);
                // 返回空列表而不是错误
                Vec::new()
            }
        };

        // 构建响应
        let response_data = JobsResponse { jobs };

        tracing::info!("Sending response with {} jobs", response_data.jobs.len());

        let result = serde_json::to_value(response_data)?;

        let response = Response {
            id: req.id,
            result: Some(result),
            error: None,
        };

        self.connection
            .sender
            .send(Message::Response(response))
            .map_err(|e| Error::MessageSend(e.to_string()))?;

        Ok(())
    }

    /// 处理 summer/plugins 请求
    ///
    /// 扫描项目中的所有插件并返回插件列表
    fn handle_plugins_request(&self, req: Request) -> Result<()> {
        tracing::info!("Handling summer/plugins request");

        use crate::scanner::plugin::{PluginScanner, PluginsRequest, PluginsResponse};

        // 解析请求参数
        let params: PluginsRequest = serde_json::from_value(req.params)?;
        let project_path = std::path::Path::new(&params.app_path);

        tracing::info!("Scanning plugins in: {:?}", project_path);

        // 创建插件扫描器
        let scanner = PluginScanner::new();

        // 扫描插件
        let plugins = match scanner.scan_plugins(project_path) {
            Ok(plugins) => {
                tracing::info!("Successfully scanned {} plugins", plugins.len());
                plugins
            }
            Err(e) => {
                tracing::error!("Failed to scan plugins: {}", e);
                // 返回空列表而不是错误
                Vec::new()
            }
        };

        // 构建响应
        let response_data = PluginsResponse { plugins };

        tracing::info!(
            "Sending response with {} plugins",
            response_data.plugins.len()
        );

        let result = serde_json::to_value(response_data)?;

        let response = Response {
            id: req.id,
            result: Some(result),
            error: None,
        };

        self.connection
            .sender
            .send(Message::Response(response))
            .map_err(|e| Error::MessageSend(e.to_string()))?;

        Ok(())
    }

    /// 处理 summer/configurations 请求
    ///
    /// 扫描项目中的所有配置结构并返回配置列表
    fn handle_configurations_request(&self, req: Request) -> Result<()> {
        tracing::debug!("Handling summer/configurations request");

        use crate::scanner::config::{
            ConfigScanner, ConfigurationsRequest, ConfigurationsResponse,
        };

        // 解析请求参数
        let params: ConfigurationsRequest = serde_json::from_value(req.params)?;
        let project_path = std::path::Path::new(&params.app_path);

        // 创建配置扫描器
        let scanner = ConfigScanner::new();

        // 扫描配置
        let configurations = match scanner.scan_configurations(project_path) {
            Ok(configurations) => configurations,
            Err(e) => {
                tracing::error!("Failed to scan configurations: {}", e);
                // 返回空列表而不是错误
                Vec::new()
            }
        };

        // 构建响应
        let response_data = ConfigurationsResponse { configurations };
        let result = serde_json::to_value(response_data)?;

        let response = Response {
            id: req.id,
            result: Some(result),
            error: None,
        };

        self.connection
            .sender
            .send(Message::Response(response))
            .map_err(|e| Error::MessageSend(e.to_string()))?;

        Ok(())
    }

    /// 处理 textDocument/documentSymbol 请求
    ///
    /// 提取文档中的符号（配置节、属性、函数、结构体等）用于大纲视图
    fn handle_document_symbol(&self, req: Request) -> Result<()> {
        tracing::debug!("Handling textDocument/documentSymbol request");

        let params: DocumentSymbolParams = serde_json::from_value(req.params)?;
        let uri = &params.text_document.uri;

        let symbols = self.document_manager.with_document(uri, |doc| {
            match doc.language_id.as_str() {
                "toml" => {
                    // TOML 文档符号提取
                    self.extract_toml_symbols(&doc.content)
                }
                "rust" => {
                    // Rust 文档符号提取
                    self.extract_rust_symbols(&doc.content)
                }
                _ => {
                    tracing::debug!(
                        "Unsupported language for document symbols: {}",
                        doc.language_id
                    );
                    vec![]
                }
            }
        });

        let result = match symbols {
            Some(symbols) => serde_json::to_value(DocumentSymbolResponse::Nested(symbols))?,
            None => serde_json::Value::Null,
        };

        let response = Response {
            id: req.id,
            result: Some(result),
            error: None,
        };

        self.connection
            .sender
            .send(Message::Response(response))
            .map_err(|e| Error::MessageSend(e.to_string()))?;

        Ok(())
    }

    /// 处理 workspace/symbol 请求
    ///
    /// 在整个工作空间中搜索符号（组件、路由、配置等）
    fn handle_workspace_symbol(&self, req: Request) -> Result<()> {
        tracing::debug!("Handling workspace/symbol request");

        // 解析请求参数
        let params: serde_json::Value = req.params;
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();

        tracing::debug!("Workspace symbol query: '{}'", query);

        let mut symbols: Vec<lsp_types::SymbolInformation> = vec![];

        // 从 workspace_path 获取工作空间路径
        if let Some(workspace_path) = &self.workspace_path {
            // 搜索组件
            if let Ok(component_symbols) = self.search_component_symbols(workspace_path, &query) {
                symbols.extend(component_symbols);
            }

            // 搜索路由
            if let Ok(route_symbols) = self.search_route_symbols(workspace_path, &query) {
                symbols.extend(route_symbols);
            }

            // 搜索配置
            if let Ok(config_symbols) = self.search_config_symbols(workspace_path, &query) {
                symbols.extend(config_symbols);
            }
        }

        tracing::debug!(
            "Found {} workspace symbols matching '{}'",
            symbols.len(),
            query
        );

        let result = serde_json::to_value(symbols)?;

        let response = Response {
            id: req.id,
            result: Some(result),
            error: None,
        };

        self.connection
            .sender
            .send(Message::Response(response))
            .map_err(|e| Error::MessageSend(e.to_string()))?;

        Ok(())
    }

    /// 搜索组件符号
    fn search_component_symbols(
        &self,
        workspace_path: &std::path::Path,
        query: &str,
    ) -> Result<Vec<lsp_types::SymbolInformation>> {
        use crate::scanner::component::{ComponentScanner, ComponentSource};
        use lsp_types::{Location, Position, Range, SymbolInformation, SymbolKind, Url};

        let scanner = ComponentScanner::new();
        let components = scanner
            .scan_components(workspace_path)
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to scan components: {}", e)))?;

        let mut symbols = Vec::new();

        for component in components {
            // 过滤：如果有查询字符串，检查组件名称是否匹配
            if !query.is_empty() && !component.name.to_lowercase().contains(query) {
                continue;
            }

            // 转换 LocationResponse 到 lsp_types::Location
            let uri = Url::parse(&component.location.uri)
                .map_err(|e| Error::Other(anyhow::anyhow!("Invalid URI: {}", e)))?;

            let range = Range {
                start: Position {
                    line: component.location.range.start.line,
                    character: component.location.range.start.character,
                },
                end: Position {
                    line: component.location.range.end.line,
                    character: component.location.range.end.character,
                },
            };

            // 根据组件来源使用不同的图标
            // #[component] 使用 METHOD (symbol-method)
            // #[derive(Service)] 使用 CLASS (symbol-class)
            let kind = match component.source {
                ComponentSource::Component => SymbolKind::METHOD,
                ComponentSource::Service => SymbolKind::CLASS,
            };

            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: component.name.clone(),
                kind,
                tags: None,
                deprecated: None,
                location: Location { uri, range },
                container_name: Some(format!("Component ({})", component.type_name)),
            });
        }

        Ok(symbols)
    }

    /// 搜索路由符号
    fn search_route_symbols(
        &self,
        workspace_path: &std::path::Path,
        query: &str,
    ) -> Result<Vec<lsp_types::SymbolInformation>> {
        use crate::scanner::route::RouteScanner;
        use lsp_types::{Location, Position, Range, SymbolInformation, SymbolKind, Url};

        let scanner = RouteScanner::new();
        let routes = scanner
            .scan_routes(workspace_path)
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to scan routes: {}", e)))?;

        let mut symbols = Vec::new();

        for route in routes {
            // 构建搜索文本：方法 + 路径
            let search_text = format!("{} {}", route.method, route.path).to_lowercase();

            // 过滤：如果有查询字符串，检查路由是否匹配
            if !query.is_empty() && !search_text.contains(query) {
                continue;
            }

            // 转换 LocationResponse 到 lsp_types::Location
            let uri = Url::parse(&route.location.uri)
                .map_err(|e| Error::Other(anyhow::anyhow!("Invalid URI: {}", e)))?;

            let range = Range {
                start: Position {
                    line: route.location.range.start.line,
                    character: route.location.range.start.character,
                },
                end: Position {
                    line: route.location.range.end.line,
                    character: route.location.range.end.character,
                },
            };

            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: format!("{} {}", route.method, route.path),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                location: Location { uri, range },
                container_name: Some(format!("Route ({})", route.handler)),
            });
        }

        Ok(symbols)
    }

    /// 搜索配置符号
    fn search_config_symbols(
        &self,
        workspace_path: &std::path::Path,
        query: &str,
    ) -> Result<Vec<lsp_types::SymbolInformation>> {
        use crate::scanner::config::ConfigScanner;
        use lsp_types::{SymbolInformation, SymbolKind};

        let scanner = ConfigScanner::new();
        let configs = scanner
            .scan_configurations(workspace_path)
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to scan configurations: {}", e)))?;

        let mut symbols = Vec::new();

        for config in configs {
            // 过滤：如果有查询字符串，检查配置名称是否匹配
            if !query.is_empty() && !config.name.to_lowercase().contains(query) {
                continue;
            }

            // config.location 已经是 Option<lsp_types::Location>
            if let Some(location) = config.location {
                #[allow(deprecated)]
                symbols.push(SymbolInformation {
                    name: config.name.clone(),
                    kind: SymbolKind::STRUCT,
                    tags: None,
                    deprecated: None,
                    location,
                    container_name: Some(format!("Config [{}]", config.prefix)),
                });
            }
        }

        Ok(symbols)
    }

    /// 提取 TOML 文档符号
    fn extract_toml_symbols(&self, content: &str) -> Vec<lsp_types::DocumentSymbol> {
        use lsp_types::{DocumentSymbol, Position, Range, SymbolKind};

        let mut symbols = Vec::new();

        // 使用 taplo 解析 TOML
        let parse_result = taplo::parser::parse(content);
        let root = parse_result.into_dom();

        // 检查根节点是否为表
        if let taplo::dom::Node::Table(table) = root {
            let entries = table.entries();
            let entries_arc = entries.get();

            // 遍历顶层表
            for (key, value) in entries_arc.iter() {
                let key_str = key.value().to_string();

                // 获取键的位置信息（简化版，使用默认位置）
                let key_range = Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: key_str.len() as u32,
                    },
                };

                match value {
                    taplo::dom::Node::Table(inner_table) => {
                        // 配置节（表）
                        let mut children = Vec::new();

                        // 提取表中的属性
                        let inner_entries = inner_table.entries();
                        let inner_entries_arc = inner_entries.get();
                        for (prop_key, _prop_value) in inner_entries_arc.iter() {
                            let prop_key_str = prop_key.value().to_string();

                            let prop_symbol = DocumentSymbol {
                                name: prop_key_str.clone(),
                                detail: Some("Property".to_string()),
                                kind: SymbolKind::PROPERTY,
                                tags: None,
                                #[allow(deprecated)]
                                deprecated: None,
                                range: key_range,
                                selection_range: key_range,
                                children: None,
                            };
                            children.push(prop_symbol);
                        }

                        let symbol = DocumentSymbol {
                            name: key_str.clone(),
                            detail: Some("Configuration section".to_string()),
                            kind: SymbolKind::MODULE,
                            tags: None,
                            #[allow(deprecated)]
                            deprecated: None,
                            range: key_range,
                            selection_range: key_range,
                            children: if children.is_empty() {
                                None
                            } else {
                                Some(children)
                            },
                        };
                        symbols.push(symbol);
                    }
                    taplo::dom::Node::Array(_) => {
                        // 数组
                        let symbol = DocumentSymbol {
                            name: key_str.clone(),
                            detail: Some("Array".to_string()),
                            kind: SymbolKind::ARRAY,
                            tags: None,
                            #[allow(deprecated)]
                            deprecated: None,
                            range: key_range,
                            selection_range: key_range,
                            children: None,
                        };
                        symbols.push(symbol);
                    }
                    _ => {
                        // 其他值类型（字符串、数字、布尔等）
                        let symbol = DocumentSymbol {
                            name: key_str.clone(),
                            detail: Some("Property".to_string()),
                            kind: SymbolKind::PROPERTY,
                            tags: None,
                            #[allow(deprecated)]
                            deprecated: None,
                            range: key_range,
                            selection_range: key_range,
                            children: None,
                        };
                        symbols.push(symbol);
                    }
                }
            }
        }

        symbols
    }

    /// 提取 Rust 文档符号
    fn extract_rust_symbols(&self, _content: &str) -> Vec<lsp_types::DocumentSymbol> {
        use lsp_types::{DocumentSymbol, Range, SymbolKind};

        let mut symbols = Vec::new();

        // 使用 syn 解析 Rust 代码
        let syntax = match syn::parse_file(_content) {
            Ok(syntax) => syntax,
            Err(e) => {
                tracing::warn!("Failed to parse Rust file: {}", e);
                return symbols;
            }
        };

        // 默认范围（因为 proc_macro2::Span 不提供行列信息）
        let default_range = Range::default();

        // 遍历顶层项
        for item in syntax.items {
            match item {
                syn::Item::Fn(item_fn) => {
                    // 函数
                    let name = item_fn.sig.ident.to_string();

                    let symbol = DocumentSymbol {
                        name: name.clone(),
                        detail: Some(format!("fn {}", name)),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        #[allow(deprecated)]
                        deprecated: None,
                        range: default_range,
                        selection_range: default_range,
                        children: None,
                    };
                    symbols.push(symbol);
                }
                syn::Item::Struct(item_struct) => {
                    // 结构体
                    let name = item_struct.ident.to_string();

                    let symbol = DocumentSymbol {
                        name: name.clone(),
                        detail: Some(format!("struct {}", name)),
                        kind: SymbolKind::STRUCT,
                        tags: None,
                        #[allow(deprecated)]
                        deprecated: None,
                        range: default_range,
                        selection_range: default_range,
                        children: None,
                    };
                    symbols.push(symbol);
                }
                syn::Item::Enum(item_enum) => {
                    // 枚举
                    let name = item_enum.ident.to_string();

                    let symbol = DocumentSymbol {
                        name: name.clone(),
                        detail: Some(format!("enum {}", name)),
                        kind: SymbolKind::ENUM,
                        tags: None,
                        #[allow(deprecated)]
                        deprecated: None,
                        range: default_range,
                        selection_range: default_range,
                        children: None,
                    };
                    symbols.push(symbol);
                }
                syn::Item::Trait(item_trait) => {
                    // Trait
                    let name = item_trait.ident.to_string();

                    let symbol = DocumentSymbol {
                        name: name.clone(),
                        detail: Some(format!("trait {}", name)),
                        kind: SymbolKind::INTERFACE,
                        tags: None,
                        #[allow(deprecated)]
                        deprecated: None,
                        range: default_range,
                        selection_range: default_range,
                        children: None,
                    };
                    symbols.push(symbol);
                }
                syn::Item::Impl(item_impl) => {
                    // Impl 块
                    if let Some((_, path, _)) = &item_impl.trait_ {
                        let name = quote::quote!(#path).to_string();
                        let symbol = DocumentSymbol {
                            name: format!("impl {}", name),
                            detail: Some("Implementation".to_string()),
                            kind: SymbolKind::CLASS,
                            tags: None,
                            #[allow(deprecated)]
                            deprecated: None,
                            range: default_range,
                            selection_range: default_range,
                            children: None,
                        };
                        symbols.push(symbol);
                    } else if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                        let name = quote::quote!(#type_path).to_string();
                        let symbol = DocumentSymbol {
                            name: format!("impl {}", name),
                            detail: Some("Implementation".to_string()),
                            kind: SymbolKind::CLASS,
                            tags: None,
                            #[allow(deprecated)]
                            deprecated: None,
                            range: default_range,
                            selection_range: default_range,
                            children: None,
                        };
                        symbols.push(symbol);
                    }
                }
                _ => {
                    // 其他项暂不处理
                }
            }
        }

        symbols
    }

    /// 处理初始化请求
    ///
    /// 声明服务器支持的所有能力，包括：
    /// - 文档同步（增量更新）
    /// - 智能补全（TOML 配置、宏参数、环境变量）
    /// - 悬停提示（配置文档、宏展开、路由信息）
    /// - 诊断（配置验证、路由验证、依赖注入验证）
    /// - 定义跳转（路由导航）
    /// - 文档符号（路由列表）
    pub fn handle_initialize(&mut self, params: InitializeParams) -> Result<InitializeResult> {
        use lsp_types::{
            CompletionOptions, HoverProviderCapability, OneOf, TextDocumentSyncCapability,
            TextDocumentSyncKind, TextDocumentSyncOptions, WorkDoneProgressOptions,
        };

        // 如果客户端提供了工作空间路径，重新加载配置
        #[allow(deprecated)]
        if let Some(root_uri) = params.root_uri {
            if let Ok(workspace_path) = root_uri.to_file_path() {
                tracing::info!(
                    "Loading configuration from workspace: {}",
                    workspace_path.display()
                );

                // 存储 workspace_path
                self.workspace_path = Some(workspace_path.clone());

                self.config = ServerConfig::load(Some(&workspace_path));

                // 验证配置
                if let Err(e) = self.config.validate() {
                    tracing::error!("Invalid configuration: {}", e);
                    return Err(Error::Config(e));
                }

                tracing::info!("Configuration loaded successfully");
                tracing::debug!(
                    "Trigger characters: {:?}",
                    self.config.completion.trigger_characters
                );
                tracing::debug!("Schema URL: {}", self.config.schema.url);
                tracing::debug!(
                    "Disabled diagnostics: {:?}",
                    self.config.diagnostics.disabled
                );
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // 文档同步能力 - 支持增量更新
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        will_save: None,
                        will_save_wait_until: None,
                        save: None,
                    },
                )),

                // 智能补全能力
                // 支持 TOML 配置项、宏参数、环境变量补全
                // 使用配置文件中的触发字符
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(true),
                    trigger_characters: Some(self.config.completion.trigger_characters.clone()),
                    all_commit_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                    completion_item: None,
                }),

                // 悬停提示能力
                // 支持配置文档、宏展开、路由信息显示
                hover_provider: Some(HoverProviderCapability::Simple(true)),

                // 定义跳转能力
                // 支持路由路径跳转到处理器函数
                definition_provider: Some(OneOf::Left(true)),

                // 文档符号能力
                // 支持显示文档中的所有路由
                document_symbol_provider: Some(OneOf::Left(true)),

                // 工作空间符号能力
                // 支持全局搜索路由和组件
                workspace_symbol_provider: Some(OneOf::Left(true)),

                // 诊断能力（通过 publishDiagnostics 通知发送）
                // 支持配置验证、路由验证、依赖注入验证

                // 代码操作能力（未来支持快速修复）
                // code_action_provider: Some(CodeActionProviderCapability::Simple(true)),

                // 格式化能力（未来支持 TOML 格式化）
                // document_formatting_provider: Some(OneOf::Left(true)),

                // 重命名能力（未来支持配置项重命名）
                // rename_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "summer-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    /// 发送错误响应
    fn send_error_response(&self, id: RequestId, code: i32, message: String) -> Result<()> {
        let response = Response {
            id,
            result: None,
            error: Some(lsp_server::ResponseError {
                code,
                message,
                data: None,
            }),
        };

        self.connection
            .sender
            .send(Message::Response(response))
            .map_err(|e| Error::MessageSend(e.to_string()))?;

        Ok(())
    }

    /// 向客户端发送错误通知
    ///
    /// 使用 window/showMessage 通知向客户端显示错误消息
    fn notify_client_error(&self, error: &Error) -> Result<()> {
        use lsp_types::{MessageType, ShowMessageParams};

        let message_type = match error.severity() {
            crate::error::ErrorSeverity::Error => MessageType::ERROR,
            crate::error::ErrorSeverity::Warning => MessageType::WARNING,
            crate::error::ErrorSeverity::Info => MessageType::INFO,
        };

        let params = ShowMessageParams {
            typ: message_type,
            message: error.to_string(),
        };

        let notification = Notification {
            method: "window/showMessage".to_string(),
            params: serde_json::to_value(params)?,
        };

        self.connection
            .sender
            .send(Message::Notification(notification))
            .map_err(|e| Error::MessageSend(e.to_string()))?;

        Ok(())
    }

    /// 优雅关闭服务器
    pub fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down summer-lsp server");

        // 清理资源
        tracing::debug!("Clearing all diagnostics...");
        // TODO: 清理所有文档的诊断
        // 需要实现 DocumentManager::get_all_uris() 方法

        tracing::debug!("Clearing document cache...");
        // 清理文档缓存（DocumentManager 会自动清理）

        tracing::debug!("Clearing indexes...");
        // 索引管理器会自动清理

        tracing::info!("Server shutdown complete");
        Ok(())
    }
}

impl Default for LspServer {
    fn default() -> Self {
        Self::start().expect("Failed to start LSP server")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{
        ClientCapabilities, ClientInfo, InitializeParams, TextDocumentItem, Url,
        VersionedTextDocumentIdentifier, WorkDoneProgressParams,
    };

    /// 测试服务器状态转换
    #[test]
    fn test_server_state_transitions() {
        // 初始状态应该是未初始化
        let server = LspServer::new_for_test().unwrap();
        assert_eq!(server.state, ServerState::Uninitialized);
    }

    /// 测试文档打开
    #[test]
    fn test_document_open() {
        let mut server = LspServer::new_for_test().unwrap();
        server.state = ServerState::Initialized;

        let uri = Url::parse("file:///test.toml").unwrap();
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "toml".to_string(),
                version: 1,
                text: "host = \"localhost\"".to_string(),
            },
        };

        server.handle_did_open(params).unwrap();

        // 验证文档已缓存
        let doc = server.document_manager.get(&uri);
        assert!(doc.is_some());
        let doc = doc.unwrap();
        assert_eq!(doc.version, 1);
        assert_eq!(doc.content, "host = \"localhost\"");
        assert_eq!(doc.language_id, "toml");
    }

    /// 测试文档修改
    #[test]
    fn test_document_change() {
        let mut server = LspServer::new_for_test().unwrap();
        server.state = ServerState::Initialized;

        let uri = Url::parse("file:///test.toml").unwrap();

        // 先打开文档
        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "toml".to_string(),
                version: 1,
                text: "host = \"localhost\"".to_string(),
            },
        };
        server.handle_did_open(open_params).unwrap();

        // 修改文档
        let change_params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: 2,
            },
            content_changes: vec![lsp_types::TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "host = \"127.0.0.1\"".to_string(),
            }],
        };
        server.handle_did_change(change_params).unwrap();

        // 验证文档已更新
        let doc = server.document_manager.get(&uri).unwrap();
        assert_eq!(doc.version, 2);
        assert_eq!(doc.content, "host = \"127.0.0.1\"");
    }

    /// 测试文档关闭
    #[test]
    fn test_document_close() {
        let mut server = LspServer::new_for_test().unwrap();
        server.state = ServerState::Initialized;

        let uri = Url::parse("file:///test.toml").unwrap();

        // 先打开文档
        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "toml".to_string(),
                version: 1,
                text: "host = \"localhost\"".to_string(),
            },
        };
        server.handle_did_open(open_params).unwrap();

        // 验证文档已缓存
        assert!(server.document_manager.get(&uri).is_some());

        // 关闭文档
        let close_params = DidCloseTextDocumentParams {
            text_document: lsp_types::TextDocumentIdentifier { uri: uri.clone() },
        };
        server.handle_did_close(close_params).unwrap();

        // 验证文档已清理
        assert!(server.document_manager.get(&uri).is_none());
    }

    /// 测试初始化响应
    #[test]
    fn test_initialize_response() {
        let mut server = LspServer::new_for_test().unwrap();

        #[allow(deprecated)]
        let params = InitializeParams {
            process_id: Some(1234),
            root_uri: None,
            capabilities: ClientCapabilities::default(),
            client_info: Some(ClientInfo {
                name: "test-client".to_string(),
                version: Some("1.0.0".to_string()),
            }),
            locale: None,
            root_path: None,
            initialization_options: None,
            trace: None,
            workspace_folders: Some(vec![lsp_types::WorkspaceFolder {
                uri: Url::parse("file:///workspace").unwrap(),
                name: "workspace".to_string(),
            }]),
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        let result = server.handle_initialize(params).unwrap();

        // 验证服务器信息
        assert!(result.server_info.is_some());
        let server_info = result.server_info.unwrap();
        assert_eq!(server_info.name, "summer-lsp");
        assert!(server_info.version.is_some());

        // 验证服务器能力
        let capabilities = result.capabilities;

        // 验证文档同步能力
        assert!(capabilities.text_document_sync.is_some());
        if let Some(lsp_types::TextDocumentSyncCapability::Options(sync_options)) =
            capabilities.text_document_sync
        {
            assert_eq!(sync_options.open_close, Some(true));
            assert_eq!(
                sync_options.change,
                Some(lsp_types::TextDocumentSyncKind::INCREMENTAL)
            );
        } else {
            panic!("Expected TextDocumentSyncOptions");
        }

        // 验证补全能力
        assert!(capabilities.completion_provider.is_some());
        let completion = capabilities.completion_provider.unwrap();
        assert_eq!(completion.resolve_provider, Some(true));
        assert!(completion.trigger_characters.is_some());
        let triggers = completion.trigger_characters.unwrap();
        assert!(triggers.contains(&"[".to_string()));
        assert!(triggers.contains(&"$".to_string()));
        assert!(triggers.contains(&"{".to_string()));

        // 验证悬停能力
        assert!(capabilities.hover_provider.is_some());

        // 验证定义跳转能力
        assert!(capabilities.definition_provider.is_some());

        // 验证文档符号能力
        assert!(capabilities.document_symbol_provider.is_some());

        // 验证工作空间符号能力
        assert!(capabilities.workspace_symbol_provider.is_some());
    }

    /// 测试错误恢复
    #[test]
    fn test_error_recovery() {
        let mut server = LspServer::new_for_test().unwrap();
        server.state = ServerState::Initialized;

        // 尝试修改不存在的文档（应该不会崩溃）
        let uri = Url::parse("file:///nonexistent.toml").unwrap();
        let change_params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: 1,
            },
            content_changes: vec![lsp_types::TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "test".to_string(),
            }],
        };

        // 这不应该导致错误，只是不会有任何效果
        let result = server.handle_did_change(change_params);
        assert!(result.is_ok());

        // 验证文档仍然不存在
        assert!(server.document_manager.get(&uri).is_none());
    }
}
