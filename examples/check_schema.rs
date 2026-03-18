use summer_lsp::schema::SchemaProvider;

#[tokio::main]
async fn main() {
    let schema_provider = SchemaProvider::load().await.expect("无法加载 Schema");

    println!("=== Logger 插件属性 ===");
    if let Some(logger) = schema_provider.get_plugin("logger") {
        for (key, prop) in &logger.properties {
            println!("  - {}: {:?}", key, prop.type_info);
        }
    } else {
        println!("  未找到 logger 插件");
    }

    println!("\n=== Opendal 插件属性 ===");
    if let Some(opendal) = schema_provider.get_plugin("opendal") {
        for (key, prop) in &opendal.properties {
            println!("  - {}: {:?}", key, prop.type_info);
        }
    } else {
        println!("  未找到 opendal 插件");
    }

    println!("\n=== Web-dav-client 插件 ===");
    if schema_provider.has_plugin("web-dav-client") {
        println!("  存在");
        if let Some(plugin) = schema_provider.get_plugin("web-dav-client") {
            for (key, prop) in &plugin.properties {
                println!("  - {}: {:?}", key, prop.type_info);
            }
        }
    } else {
        println!("  不存在（这是自定义插件）");
    }
}
