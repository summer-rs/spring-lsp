use anyhow::Result;
use summer_lsp::protocol::LspServer;
use summer_lsp::utils::init_logging;

fn main() -> Result<()> {
    // 初始化日志系统
    // 使用环境变量配置：
    // - SUMMER_LSP_LOG_LEVEL: 日志级别（trace, debug, info, warn, error）
    // - SUMMER_LSP_VERBOSE: 启用详细日志（1 或 true）
    // - SUMMER_LSP_LOG_FILE: 日志文件路径（可选）
    init_logging().expect("Failed to initialize logging system");

    tracing::info!("Starting summer-lsp language server");

    // 启动 LSP 服务器
    let mut server = LspServer::start()?;
    server.run()?;

    tracing::info!("summer-lsp language server stopped");
    Ok(())
}
