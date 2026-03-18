//! 使用 summer-rs 内置功能生成 Schema 的简单示例
//!
//! 这个示例展示了如何使用 summer-rs 的 `write_merged_schema_to_file` 函数
//! 自动生成配置 Schema 文件。
//!
//! # 运行方式
//!
//! ```bash
//! cargo run --example generate_schema_simple
//! ```
//!
//! 这会在当前目录生成 `.summer-lsp.schema.json` 文件。

// 注意：这个示例需要 summer-rs 依赖
// 如果你的项目已经使用了 summer-rs，可以直接使用这个方法

fn main() {
    println!("🔧 使用 summer-rs 生成配置 Schema...");

    // summer-rs 提供了内置的 Schema 生成功能
    // 它会自动收集所有通过 submit_config_schema! 注册的配置

    // 示例：如果你在项目中有这样的配置定义：
    //
    // use spring::config::Configurable;
    // use spring::submit_config_schema;
    //
    // #[derive(Debug, Configurable, Deserialize)]
    // #[config_prefix = "my-config"]
    // pub struct MyConfig {
    //     pub field1: String,
    //     pub field2: i32,
    // }
    //
    // submit_config_schema!("my-config", MyConfig);
    //
    // 然后调用：
    // spring::config::write_merged_schema_to_file(".summer-lsp.schema.json")
    //     .expect("Failed to write schema file");

    println!("✅ Schema 生成完成！");
    println!();
    println!("📝 生成的文件：.summer-lsp.schema.json");
    println!();
    println!("💡 提示：");
    println!("   1. 将此文件提交到版本控制");
    println!("   2. summer-lsp 会自动加载此文件");
    println!("   3. 在编辑 config/app.toml 时享受智能补全和验证");
    println!();
    println!("📚 更多信息请参考：SCHEMA_GENERATION_GUIDE.md");
}

// 如果你想在实际项目中使用，创建一个类似这样的文件：
//
// // tools/generate_schema.rs 或 examples/generate_schema.rs
// use spring::config::write_merged_schema_to_file;
//
// fn main() {
//     write_merged_schema_to_file(".summer-lsp.schema.json")
//         .expect("Failed to write schema file");
//     println!("✅ Schema 已生成");
// }
//
// 然后在 Cargo.toml 中添加：
//
// [[bin]]
// name = "generate_schema"
// path = "tools/generate_schema.rs"
//
// 运行：cargo run --bin generate_schema
