//! 测试 Schema 从 target 目录加载的示例
//!
//! 这个示例展示如何验证 summer-lsp 能够正确从 target 目录加载 Schema
//!
//! # 运行方式
//!
//! ```bash
//! cargo run --example test_schema_loading
//! ```

use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn main() {
    println!("🧪 测试 Schema 从 target 目录加载\n");

    // 创建临时工作空间
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_path = temp_dir.path();

    println!("📁 创建临时工作空间: {:?}", workspace_path);

    // 模拟 build.rs 生成的 Schema 文件结构
    let schema_dir = workspace_path.join("target/debug/build/my-app/out");
    fs::create_dir_all(&schema_dir).expect("Failed to create schema dir");

    let schema_path = schema_dir.join("summer-lsp.schema.json");

    // 创建测试 Schema
    let schema_content = serde_json::json!({
        "properties": {
            "my-service": {
                "type": "object",
                "properties": {
                    "endpoint": {
                        "type": "string",
                        "description": "Service endpoint URL"
                    },
                    "timeout": {
                        "type": "integer",
                        "default": 30,
                        "description": "Connection timeout in seconds"
                    },
                    "enable-retry": {
                        "type": "boolean",
                        "default": false,
                        "description": "Enable retry on failure"
                    }
                },
                "required": ["endpoint"]
            }
        }
    });

    fs::write(
        &schema_path,
        serde_json::to_string_pretty(&schema_content).unwrap(),
    )
    .expect("Failed to write schema file");

    println!("✅ 创建 Schema 文件: {:?}", schema_path);
    println!();

    // 测试查找功能
    println!("🔍 测试查找 Schema 文件...");
    let found = find_schema_in_target(workspace_path);

    match found {
        Some(path) => {
            println!("✅ 找到 Schema 文件: {:?}", path);
            println!();

            // 读取并验证内容
            println!("📖 读取 Schema 内容...");
            let content = fs::read_to_string(&path).expect("Failed to read schema");
            let schema: serde_json::Value =
                serde_json::from_str(&content).expect("Failed to parse schema");

            if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
                println!("✅ Schema 包含 {} 个配置节:", properties.len());
                for (key, _) in properties {
                    println!("   - {}", key);
                }
            }

            println!();
            println!("🎉 测试成功！Schema 可以正确从 target 目录加载");
        }
        None => {
            println!("❌ 未找到 Schema 文件");
            std::process::exit(1);
        }
    }
}

/// 在 target 目录中查找 Schema 文件
///
/// 这是 SchemaProvider::find_schema_in_target 的简化版本
fn find_schema_in_target(workspace_path: &Path) -> Option<std::path::PathBuf> {
    let target_dir = workspace_path.join("target");
    if !target_dir.exists() {
        return None;
    }

    let mut schema_files = Vec::new();

    if let Ok(entries) = fs::read_dir(&target_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let build_dir = path.join("build");
                if build_dir.exists() {
                    if let Ok(build_entries) = fs::read_dir(&build_dir) {
                        for build_entry in build_entries.flatten() {
                            let out_dir = build_entry.path().join("out");
                            let schema_path = out_dir.join("summer-lsp.schema.json");
                            if schema_path.exists() {
                                if let Ok(metadata) = fs::metadata(&schema_path) {
                                    if let Ok(modified) = metadata.modified() {
                                        schema_files.push((schema_path, modified));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 返回最新的 Schema 文件
    schema_files.sort_by(|a, b| b.1.cmp(&a.1));
    schema_files.first().map(|(path, _)| path.clone())
}
