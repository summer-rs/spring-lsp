use summer_lsp::analysis::toml::TomlAnalyzer;
use summer_lsp::schema::SchemaProvider;

#[tokio::main]
async fn main() {
    let content = std::fs::read_to_string("test_user_config.toml").expect("无法读取测试配置文件");

    let schema_provider = SchemaProvider::load().await.expect("无法加载 Schema");

    let analyzer = TomlAnalyzer::new(schema_provider);

    match analyzer.parse(&content) {
        Ok(doc) => {
            println!("✅ 配置文件解析成功！\n");

            let diagnostics = analyzer.validate(&doc);

            if diagnostics.is_empty() {
                println!("✅ 配置验证通过，没有任何诊断信息！");
            } else {
                println!("📋 发现 {} 个诊断信息:\n", diagnostics.len());

                for (i, diag) in diagnostics.iter().enumerate() {
                    let severity = match diag.severity {
                        Some(lsp_types::DiagnosticSeverity::ERROR) => "❌ 错误",
                        Some(lsp_types::DiagnosticSeverity::WARNING) => "⚠️  警告",
                        Some(lsp_types::DiagnosticSeverity::INFORMATION) => "ℹ️  信息",
                        Some(lsp_types::DiagnosticSeverity::HINT) => "💡 提示",
                        _ => "❓ 未知",
                    };

                    println!("{}. {} (行 {})", i + 1, severity, diag.range.start.line + 1);
                    println!("   {}", diag.message);
                    println!();
                }

                // 统计各级别数量
                let errors = diagnostics
                    .iter()
                    .filter(|d| d.severity == Some(lsp_types::DiagnosticSeverity::ERROR))
                    .count();
                let warnings = diagnostics
                    .iter()
                    .filter(|d| d.severity == Some(lsp_types::DiagnosticSeverity::WARNING))
                    .count();
                let hints = diagnostics
                    .iter()
                    .filter(|d| d.severity == Some(lsp_types::DiagnosticSeverity::HINT))
                    .count();

                println!(
                    "📊 统计: {} 个错误, {} 个警告, {} 个提示",
                    errors, warnings, hints
                );
            }
        }
        Err(e) => {
            eprintln!("❌ 配置文件解析失败: {}", e);
            std::process::exit(1);
        }
    }
}
