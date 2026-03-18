# Schema 生成工作流程

本文档说明 summer-lsp 如何与 summer-rs 的 Schema 生成功能集成。

## 概述

summer-lsp 支持三层 Schema 加载策略：

```
┌─────────────────────────────────────────────────────────┐
│  1. 远程 Schema (summer-rs 官方插件)                     │
│     https://summer-rs.github.io/config-schema.json      │
└─────────────────────────────────────────────────────────┘
                         ↓ 合并
┌─────────────────────────────────────────────────────────┐
│  2. 本地 Schema 文件 (推荐)                              │
│     .summer-lsp.schema.json                             │
│     由 summer-rs 的 write_merged_schema_to_file 生成     │
└─────────────────────────────────────────────────────────┘
                         ↓ Fallback
┌─────────────────────────────────────────────────────────┐
│  3. 自动扫描 (Fallback)                                  │
│     扫描 #[derive(Configurable)] 结构体                  │
└─────────────────────────────────────────────────────────┘
```

## summer-rs 的 Schema 生成实现

### 核心代码位置

`summer-rs/spring/src/config/mod.rs` (第 102 行)

### 关键函数

```rust
/// 获取所有注册的 schemas
pub fn auto_config_schemas() -> Vec<(String, Schema)> {
    inventory::iter::<ConfigSchema>
        .into_iter()
        .map(|c| (c.prefix.to_string(), (c.schema)()))
        .collect()
}

/// 合并所有配置 schemas 为一个 JSON schema
pub fn merge_all_schemas() -> serde_json::Value {
    let mut properties = serde_json::Map::new();

    for (prefix, schema) in auto_config_schemas() {
        properties.insert(prefix, serde_json::to_value(schema).unwrap());
    }

    json!({
        "type": "object",
        "properties": properties
    })
}

/// 将合并的 JSON schema 写入文件
pub fn write_merged_schema_to_file(path: &str) -> std::io::Result<()> {
    let merged = merge_all_schemas();
    std::fs::write(path, serde_json::to_string_pretty(&merged).unwrap())
}
```

### 工作原理

1. **注册阶段**：
   ```rust
   submit_config_schema!("my-config", MyConfig);
   ```
   使用 `inventory` crate 在编译时注册配置

2. **收集阶段**：
   ```rust
   inventory::iter::<ConfigSchema>
   ```
   遍历所有注册的配置

3. **生成阶段**：
   ```rust
   (c.schema)()  // 调用 schema_for!(MyConfig)
   ```
   使用 `schemars` 为每个配置生成 Schema

4. **合并阶段**：
   ```rust
   properties.insert(prefix, schema);
   ```
   将所有 Schema 合并为一个 JSON 对象

## summer-lsp 的 Schema 加载实现

### 核心代码位置

`summer-lsp/src/core/schema.rs`

### 关键函数

```rust
/// 从 URL 加载 Schema 并合并本地 Schema 文件
pub async fn load_with_workspace(workspace_path: &Path) -> anyhow::Result<Self> {
    // 1. 加载远程 Schema
    let mut provider = Self::load().await?;
    
    // 2. 尝试加载本地 Schema 文件
    let local_schema_path = workspace_path.join(".summer-lsp.schema.json");
    if local_schema_path.exists() {
        match Self::load_local_schema_file(&local_schema_path) {
            Ok(local_schemas) => {
                for (prefix, schema) in local_schemas {
                    provider.schema.plugins.insert(prefix, schema);
                }
                return Ok(provider);
            }
            Err(e) => {
                tracing::warn!("Failed to load local schema file: {}", e);
            }
        }
    }
    
    // 3. Fallback: 扫描 Rust 代码生成 Schema
    let scanner = ConfigScanner::new();
    match scanner.scan_configurations(workspace_path) {
        Ok(configurations) => {
            for config in configurations {
                let schema_json = Self::configuration_to_schema(&config);
                provider.schema.plugins.insert(config.prefix.clone(), schema_json);
            }
        }
        Err(e) => {
            tracing::warn!("Failed to scan local configurations: {}", e);
        }
    }
    
    Ok(provider)
}
```

### 加载策略

1. **优先级 1：本地 Schema 文件**
   - 文件名：`.summer-lsp.schema.json`
   - 位置：工作空间根目录
   - 格式：summer-rs 生成的标准格式
   - 优势：精确、完整、快速

2. **优先级 2：自动扫描**
   - 扫描 `#[derive(Configurable)]` 结构体
   - 提取字段名称和类型
   - 推断基本的 Schema
   - 优势：零配置、自动发现
   - 劣势：类型推断有限

## 完整工作流程

### 开发阶段

```mermaid
graph TD
    A[定义配置结构] --> B[添加 #derive Configurable]
    B --> C[使用 submit_config_schema! 注册]
    C --> D[创建 Schema 生成工具]
    D --> E[运行 cargo run --bin generate_schema]
    E --> F[生成 .summer-lsp.schema.json]
    F --> G[提交到版本控制]
```

### 运行时阶段

```mermaid
graph TD
    A[启动 LSP] --> B[加载远程 Schema]
    B --> C{存在 .summer-lsp.schema.json?}
    C -->|是| D[加载本地 Schema 文件]
    C -->|否| E[扫描 Rust 代码]
    D --> F[合并 Schema]
    E --> F
    F --> G[提供 LSP 功能]
    G --> H[智能补全]
    G --> I[类型验证]
    G --> J[悬停文档]
```

## 示例项目结构

```
my-spring-app/
├── Cargo.toml
├── .summer-lsp.schema.json      # 生成的 Schema 文件
├── src/
│   ├── main.rs
│   └── config.rs                # 配置定义
│       ├── #[derive(Configurable)]
│       └── submit_config_schema!()
├── tools/
│   └── generate_schema.rs       # Schema 生成工具
│       └── write_merged_schema_to_file()
└── config/
    └── app.toml                 # 配置文件（享受 LSP 支持）
```

## 配置示例

### 1. 定义配置 (src/config.rs)

```rust
use spring::config::Configurable;
use spring::submit_config_schema;
use serde::Deserialize;

#[derive(Debug, Configurable, Deserialize)]
#[config_prefix = "my-service"]
pub struct MyServiceConfig {
    /// 服务端点 URL
    pub endpoint: String,
    
    /// 连接超时（秒）
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    
    /// 是否启用重试
    #[serde(default)]
    pub enable_retry: bool,
}

fn default_timeout() -> u64 {
    30
}

// 注册配置 Schema
submit_config_schema!("my-service", MyServiceConfig);
```

### 2. 生成 Schema (tools/generate_schema.rs)

```rust
use spring::config::write_merged_schema_to_file;

fn main() {
    write_merged_schema_to_file(".summer-lsp.schema.json")
        .expect("Failed to write schema file");
    
    println!("✅ Schema 已生成到 .summer-lsp.schema.json");
}
```

### 3. 生成的 Schema (.summer-lsp.schema.json)

```json
{
  "properties": {
    "my-service": {
      "properties": {
        "endpoint": {
          "description": "服务端点 URL",
          "type": "string"
        },
        "timeout": {
          "default": 30,
          "description": "连接超时（秒）",
          "type": "integer"
        },
        "enable-retry": {
          "default": false,
          "description": "是否启用重试",
          "type": "boolean"
        }
      },
      "required": ["endpoint"],
      "type": "object"
    }
  },
  "type": "object"
}
```

### 4. 配置文件 (config/app.toml)

```toml
#:schema https://summer-rs.github.io/config-schema.json

[my-service]
endpoint = "https://api.example.com"  # ✅ LSP 提供补全和验证
timeout = 60                          # ✅ 类型检查
enable-retry = true                   # ✅ 悬停显示文档
```

## 集成到构建流程

### 方法 1：build.rs（推荐）

```rust
// build.rs
use spring::config::write_merged_schema_to_file;

fn main() {
    // 生成 Schema
    write_merged_schema_to_file(".summer-lsp.schema.json")
        .expect("Failed to generate schema");
    
    // 当配置文件变化时重新生成
    println!("cargo:rerun-if-changed=src/config.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
}
```

**优势：**
- ✅ 编译时自动生成，无需手动运行
- ✅ 配置变化时自动更新
- ✅ 团队成员编译时自动获得最新 Schema
- ✅ 减少人为遗忘更新的风险

**使用：**
```bash
cargo build  # Schema 自动生成
cargo run    # 编译时自动生成 Schema
```

### 方法 2：独立工具

```rust
// tools/generate_schema.rs
use spring::config::write_merged_schema_to_file;

fn main() {
    write_merged_schema_to_file(".summer-lsp.schema.json")
        .expect("Failed to write schema file");
    
    println!("✅ Schema 已生成到 .summer-lsp.schema.json");
}
```

在 `Cargo.toml` 中添加：
```toml
[[bin]]
name = "generate_schema"
path = "tools/generate_schema.rs"
```

**使用：**
```bash
cargo run --bin generate_schema
```

### 方法 3：Makefile

```makefile
.PHONY: schema dev

schema:
	cargo run --bin generate_schema

dev: schema
	cargo run
```

### 方法 3：Makefile

```makefile
.PHONY: schema dev

schema:
	cargo run --bin generate_schema

dev: schema
	cargo run
```

### 方法 4：just

```justfile
# justfile
generate-schema:
    cargo run --bin generate_schema

dev: generate-schema
    cargo run

test: generate-schema
    cargo test
```

### 方法 4：cargo-make

```toml
# Makefile.toml
[tasks.generate-schema]
command = "cargo"
args = ["run", "--bin", "generate_schema"]

[tasks.dev]
dependencies = ["generate-schema"]
command = "cargo"
args = ["run"]
```

## CI/CD 集成

### GitHub Actions

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Generate schema
        run: cargo run --bin generate_schema
      
      - name: Check schema is up to date
        run: |
          git diff --exit-code .summer-lsp.schema.json || \
          (echo "Schema is out of date. Run 'cargo run --bin generate_schema'" && exit 1)
      
      - name: Run tests
        run: cargo test
```

## 最佳实践

### 1. 提交 Schema 文件到版本控制

```gitignore
# 不要忽略 Schema 文件
!.summer-lsp.schema.json
```

**原因**：
- 团队成员获得一致的 LSP 体验
- CI 环境也能使用 LSP 功能
- 不需要每次都重新生成

### 2. 定期更新 Schema

当添加或修改配置时：

```bash
cargo run --bin generate_schema
git add .summer-lsp.schema.json
git commit -m "Update configuration schema"
```

### 3. 在 CI 中验证 Schema

确保 Schema 文件是最新的：

```yaml
- name: Verify schema
  run: |
    cargo run --bin generate_schema
    git diff --exit-code .summer-lsp.schema.json
```

### 4. 充分利用文档注释

```rust
#[derive(Configurable, Deserialize)]
#[config_prefix = "database"]
pub struct DatabaseConfig {
    /// 数据库连接 URL
    ///
    /// 支持的格式：
    /// - PostgreSQL: `postgres://user:pass@host:port/db`
    /// - MySQL: `mysql://user:pass@host:port/db`
    ///
    /// # 示例
    ///
    /// ```toml
    /// [database]
    /// url = "postgres://localhost:5432/mydb"
    /// ```
    pub url: String,
}
```

文档注释会出现在：
- LSP 的悬停提示中
- 生成的 Schema 的 `description` 字段中
- IDE 的自动补全提示中

## 故障排查

### Schema 文件未被加载

**症状**：自定义配置没有补全和验证

**解决方案**：
1. 确认文件名为 `.summer-lsp.schema.json`
2. 确认文件在工作空间根目录
3. 检查 LSP 日志：`RUST_LOG=summer_lsp=debug`
4. 重启 LSP 服务器

### Schema 格式错误

**症状**：LSP 启动失败或功能异常

**解决方案**：
```bash
# 验证 JSON 格式
cat .summer-lsp.schema.json | jq .

# 检查 properties 字段
cat .summer-lsp.schema.json | jq '.properties'
```

### 配置未注册

**症状**：运行 `generate_schema` 后配置未出现在 Schema 中

**解决方案**：
1. 确认 `submit_config_schema!` 在模块级别调用
2. 确认配置模块被编译（没有被 `#[cfg]` 排除）
3. 清理并重新构建：`cargo clean && cargo build`

## 参考资源

- [summer-rs 配置文档](https://summer-rs.github.io/docs/plugins/plugin-by-self/)
- [summer-rs 源码 - config/mod.rs](https://github.com/summer-rs/summer-rs/blob/master/spring/src/config/mod.rs)
- [SCHEMA_GENERATION_GUIDE.md](../SCHEMA_GENERATION_GUIDE.md) - 详细的 Schema 生成指南
- [LOCAL_CONFIG_SUPPORT.md](../LOCAL_CONFIG_SUPPORT.md) - 本地配置支持文档
- [schemars 文档](https://docs.rs/schemars/)
- [inventory crate](https://docs.rs/inventory/)
