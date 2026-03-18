use summer_lsp::analysis::toml::TomlAnalyzer;
use summer_lsp::schema::SchemaProvider;

#[tokio::main]
async fn main() {
    // 读取测试配置文件
    let content = std::fs::read_to_string("test_nested_config.toml").expect("无法读取测试配置文件");

    // 加载真实的 Schema
    let schema_provider = SchemaProvider::load().await.expect("无法加载 Schema");

    let analyzer = TomlAnalyzer::new(schema_provider);

    // 解析配置
    match analyzer.parse(&content) {
        Ok(doc) => {
            println!("✅ 配置文件解析成功！\n");

            // 验证配置
            let diagnostics = analyzer.validate(&doc);

            if diagnostics.is_empty() {
                println!("✅ 配置验证通过，没有错误或警告！");
            } else {
                println!("⚠️  发现 {} 个诊断信息:\n", diagnostics.len());

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

                    if let Some(code) = &diag.code {
                        match code {
                            lsp_types::NumberOrString::String(s) => {
                                println!("   代码: {}", s);
                            }
                            lsp_types::NumberOrString::Number(n) => {
                                println!("   代码: {}", n);
                            }
                        }
                    }
                    println!();
                }

                // 统计错误和警告数量
                let errors = diagnostics
                    .iter()
                    .filter(|d| d.severity == Some(lsp_types::DiagnosticSeverity::ERROR))
                    .count();
                let warnings = diagnostics
                    .iter()
                    .filter(|d| d.severity == Some(lsp_types::DiagnosticSeverity::WARNING))
                    .count();

                println!("📊 统计: {} 个错误, {} 个警告", errors, warnings);
            }
        }
        Err(e) => {
            eprintln!("❌ 配置文件解析失败: {}", e);
            std::process::exit(1);
        }
    }
}
