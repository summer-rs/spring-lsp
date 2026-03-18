use summer_lsp::analysis::toml::TomlAnalyzer;
use summer_lsp::schema::SchemaProvider;

#[tokio::main]
async fn main() {
    let content = std::fs::read_to_string("test_simple_nested.toml").expect("无法读取测试配置文件");

    let schema_provider = SchemaProvider::load().await.expect("无法加载 Schema");

    println!("📋 Schema 中的插件:");
    for prefix in schema_provider.get_all_prefixes() {
        println!("  - {}", prefix);

        if prefix == "web" {
            if let Some(plugin) = schema_provider.get_plugin(&prefix) {
                println!("    属性:");
                for key in plugin.properties.keys() {
                    println!("      • {}", key);
                }
            }
        }
    }
    println!();

    let analyzer = TomlAnalyzer::new(schema_provider);

    match analyzer.parse(&content) {
        Ok(doc) => {
            println!("✅ 配置文件解析成功！\n");

            let diagnostics = analyzer.validate(&doc);

            if diagnostics.is_empty() {
                println!("✅ 配置验证通过！");
            } else {
                println!("⚠️  发现 {} 个诊断信息:\n", diagnostics.len());

                for (i, diag) in diagnostics.iter().enumerate() {
                    let severity = match diag.severity {
                        Some(lsp_types::DiagnosticSeverity::ERROR) => "❌ 错误",
                        Some(lsp_types::DiagnosticSeverity::WARNING) => "⚠️  警告",
                        _ => "ℹ️  其他",
                    };

                    println!("{}. {} (行 {})", i + 1, severity, diag.range.start.line + 1);
                    println!("   {}", diag.message);
                    println!();
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 配置文件解析失败: {}", e);
            std::process::exit(1);
        }
    }
}
