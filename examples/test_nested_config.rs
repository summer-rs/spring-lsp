use summer_lsp::analysis::toml::{ConfigValue, TomlAnalyzer};
use summer_lsp::schema::SchemaProvider;

fn main() {
    // 读取测试配置文件
    let content = std::fs::read_to_string("test_nested_config.toml").expect("无法读取测试配置文件");

    // 创建分析器
    let schema_provider = SchemaProvider::new();
    let analyzer = TomlAnalyzer::new(schema_provider);

    // 解析配置
    match analyzer.parse(&content) {
        Ok(doc) => {
            println!("✅ 配置文件解析成功！\n");

            // 显示提取的配置段
            println!("📋 配置段数量: {}", doc.config_sections.len());
            for (prefix, section) in &doc.config_sections {
                println!("\n[{}]", prefix);
                println!("  属性数量: {}", section.properties.len());

                for (key, property) in &section.properties {
                    println!("  - {}: {:?}", key, property.value);
                }
            }

            // 显示提取的环境变量
            println!("\n🔧 环境变量数量: {}", doc.env_vars.len());
            for env_var in &doc.env_vars {
                println!("  - ${{{}}}", env_var.name);
                if let Some(default) = &env_var.default {
                    println!("    默认值: {}", default);
                }
            }

            // 验证嵌套配置
            println!("\n✨ 嵌套配置验证:");

            if let Some(web_section) = doc.config_sections.get("web") {
                println!("  ✓ web 配置段存在");

                if web_section.properties.contains_key("middlewares") {
                    println!("  ✓ web.middlewares 嵌套配置存在");
                }

                if web_section.properties.contains_key("host") {
                    println!("  ✓ web.host 顶层属性存在");
                }
            }

            if let Some(opendal_section) = doc.config_sections.get("opendal") {
                println!("  ✓ opendal 配置段存在");

                if let Some(options_prop) = opendal_section.properties.get("options") {
                    println!("  ✓ opendal.options 内联表存在");

                    if let ConfigValue::Table(table) = &options_prop.value {
                        println!("    - 包含 {} 个属性", table.len());
                        for key in table.keys() {
                            println!("      • {}", key);
                        }
                    }
                }
            }

            if let Some(db_section) = doc.config_sections.get("database") {
                println!("  ✓ database 配置段存在");

                if db_section.properties.contains_key("pool") {
                    println!("  ✓ database.pool 嵌套配置存在");
                }

                if db_section.properties.contains_key("migrations") {
                    println!("  ✓ database.migrations 嵌套配置存在");
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 配置文件解析失败: {}", e);
            std::process::exit(1);
        }
    }
}
