//! Schema 生成工具
//!
//! 用户可以在项目中运行这个工具来生成配置 Schema
//!
//! 使用方法：
//! 1. 在 Cargo.toml 中添加 schemars 依赖
//! 2. 为配置结构体添加 #[derive(JsonSchema)]
//! 3. 运行这个工具生成 schema.json

use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

// 示例配置结构
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
struct WebDavClientConfig {
    /// WebDAV 用户名
    username: String,
    /// WebDAV 密码
    password: String,
    /// 是否使用原始资源
    #[serde(default)]
    use_origin: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct CustomDatabaseConfig {
    /// 数据库主机
    host: String,
    /// 数据库端口
    #[serde(default = "default_port")]
    port: u16,
    /// 连接超时（秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout: Option<u64>,
}

fn default_port() -> u16 {
    5432
}

fn main() {
    // 生成各个配置结构的 Schema
    let mut schemas = HashMap::new();

    // WebDAV 客户端配置
    let web_dav_schema = schema_for!(WebDavClientConfig);
    schemas.insert("web-dav-client", web_dav_schema);

    // 自定义数据库配置
    let custom_db_schema = schema_for!(CustomDatabaseConfig);
    schemas.insert("custom-db", custom_db_schema);

    // 构建完整的 Schema 文档
    let mut properties = serde_json::Map::new();
    for (prefix, schema) in schemas {
        properties.insert(prefix.to_string(), serde_json::to_value(schema).unwrap());
    }

    let full_schema = serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": properties,
        "description": "Local configuration schema"
    });

    // 写入文件
    let output = serde_json::to_string_pretty(&full_schema).unwrap();
    fs::write(".summer-lsp.schema.json", output).expect("Failed to write schema file");

    println!("✅ Schema 已生成到 .summer-lsp.schema.json");
    println!("📋 包含 {} 个配置结构", properties.len());
}
