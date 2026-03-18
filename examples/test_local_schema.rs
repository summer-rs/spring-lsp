use std::path::Path;
use summer_lsp::schema::SchemaProvider;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    let workspace_path = Path::new("test_project");

    println!("🔍 扫描本地配置结构...\n");

    // 加载 Schema（包含本地扫描）
    let schema_provider = SchemaProvider::load_with_workspace(workspace_path)
        .await
        .expect("无法加载 Schema");

    println!("📋 所有可用的配置前缀:\n");
    let mut prefixes = schema_provider.get_all_prefixes();
    prefixes.sort();

    for prefix in &prefixes {
        let marker = if prefix == "web-dav-client" || prefix == "custom-db" {
            "🆕" // 本地扫描的
        } else {
            "🌐" // 远程的
        };

        println!("  {} {}", marker, prefix);

        // 显示属性
        if let Some(plugin) = schema_provider.get_plugin(prefix) {
            for (key, prop) in &plugin.properties {
                let type_str = match &prop.type_info {
                    summer_lsp::schema::TypeInfo::String { .. } => "string",
                    summer_lsp::schema::TypeInfo::Integer { .. } => "integer",
                    summer_lsp::schema::TypeInfo::Float { .. } => "number",
                    summer_lsp::schema::TypeInfo::Boolean => "boolean",
                    summer_lsp::schema::TypeInfo::Array { .. } => "array",
                    summer_lsp::schema::TypeInfo::Object { .. } => "object",
                };
                println!("      • {}: {}", key, type_str);
                if !prop.description.is_empty() {
                    println!("        {}", prop.description);
                }
            }
        }
        println!();
    }

    println!("✅ 本地配置已成功集成到 Schema 中！");
}
