# Schema 加载功能测试指南

本文档说明如何测试 summer-lsp 从 target 目录加载 Schema 的功能。

## 单元测试

### 运行所有测试

```bash
cd summer-lsp
cargo test
```

### 运行 Schema 相关测试

```bash
cargo test test_find_schema_in_target --lib
```

### 测试覆盖的场景

1. **test_find_schema_in_target** - 基本的 Schema 查找功能
2. **test_find_schema_in_target_multiple_profiles** - 多个 profile 时选择最新的
3. **test_find_schema_in_target_not_exists** - target 目录不存在的情况
4. **test_load_local_schema_file** - 从文件加载 Schema

## 集成测试示例

运行集成测试示例：

```bash
cargo run --example test_schema_loading
```

这个示例会：
1. 创建临时工作空间
2. 模拟 build.rs 生成的 Schema 文件结构
3. 测试查找和加载功能
4. 验证 Schema 内容

## 端到端测试

### 准备测试项目

1. 创建一个测试 summer-rs 项目：

```bash
cargo new test-spring-app
cd test-spring-app
```

2. 添加依赖到 `Cargo.toml`：

```toml
[dependencies]
spring = "0.2"
serde = { version = "1.0", features = ["derive"] }

[build-dependencies]
spring = "0.2"
```

3. 创建配置结构 `src/config.rs`：

```rust
use spring::config::Configurable;
use spring::submit_config_schema;
use serde::Deserialize;

#[derive(Debug, Configurable, Deserialize)]
#[config_prefix = "my-service"]
pub struct MyServiceConfig {
    /// Service endpoint URL
    pub endpoint: String,
    
    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    
    /// Enable retry on failure
    #[serde(default)]
    pub enable_retry: bool,
}

fn default_timeout() -> u64 {
    30
}

// 注册配置 Schema
submit_config_schema!("my-service", MyServiceConfig);
```

4. 创建 `build.rs`：

```rust
use spring::config::write_merged_schema_to_file;
use std::env;

fn main() {
    // 生成到 target 目录
    let out_dir = env::var("OUT_DIR").unwrap();
    let schema_path = format!("{}/summer-lsp.schema.json", out_dir);
    
    write_merged_schema_to_file(&schema_path)
        .expect("Failed to write schema file");
    
    println!("cargo:rerun-if-changed=src/config.rs");
    
    // 打印 Schema 路径用于调试
    println!("cargo:warning=Schema generated at: {}", schema_path);
}
```

5. 在 `src/lib.rs` 中引入配置：

```rust
pub mod config;
```

### 编译并验证

1. 编译项目：

```bash
cargo build
```

你应该看到类似的输出：
```
warning: Schema generated at: /path/to/test-spring-app/target/debug/build/test-spring-app-xxx/out/summer-lsp.schema.json
```

2. 验证 Schema 文件存在：

```bash
find target -name "summer-lsp.schema.json"
```

应该输出类似：
```
target/debug/build/test-spring-app-xxx/out/summer-lsp.schema.json
```

3. 查看 Schema 内容：

```bash
cat $(find target -name "summer-lsp.schema.json" | head -1) | jq .
```

应该看到：
```json
{
  "properties": {
    "my-service": {
      "properties": {
        "endpoint": {
          "description": "Service endpoint URL",
          "type": "string"
        },
        "timeout": {
          "default": 30,
          "description": "Connection timeout in seconds",
          "type": "integer"
        },
        "enable-retry": {
          "default": false,
          "description": "Enable retry on failure",
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

### 测试 LSP 加载

1. 启动 summer-lsp 服务器（在 summer-lsp 目录）：

```bash
cd /path/to/summer-lsp
cargo run
```

2. 在另一个终端，使用 LSP 客户端连接并测试（或使用 VSCode 扩展）

3. 创建配置文件 `config/app.toml`：

```toml
[my-service]
endpoint = "https://api.example.com"
timeout = 60
enable-retry = true
```

4. 在编辑器中打开 `config/app.toml`，应该看到：
   - ✅ 智能补全提示 `endpoint`、`timeout`、`enable-retry`
   - ✅ 悬停显示字段描述
   - ✅ 类型验证（例如 timeout 必须是数字）
   - ✅ 必填字段验证（endpoint 是必需的）

### 测试多 Profile 场景

1. 编译 debug 版本：

```bash
cargo build
```

2. 编译 release 版本：

```bash
cargo build --release
```

3. 检查两个 Schema 文件：

```bash
find target -name "summer-lsp.schema.json"
```

应该看到两个文件：
```
target/debug/build/test-spring-app-xxx/out/summer-lsp.schema.json
target/release/build/test-spring-app-xxx/out/summer-lsp.schema.json
```

4. LSP 应该加载最新的那个（通过文件修改时间判断）

### 测试配置变化时自动更新

1. 修改 `src/config.rs`，添加新字段：

```rust
#[derive(Debug, Configurable, Deserialize)]
#[config_prefix = "my-service"]
pub struct MyServiceConfig {
    pub endpoint: String,
    pub timeout: u64,
    pub enable_retry: bool,
    
    /// 新增字段：最大重试次数
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_max_retries() -> u32 {
    3
}
```

2. 重新编译：

```bash
cargo build
```

3. 验证 Schema 已更新：

```bash
cat $(find target -name "summer-lsp.schema.json" | head -1) | jq '.properties."my-service".properties | keys'
```

应该看到新字段：
```json
[
  "enable-retry",
  "endpoint",
  "max-retries",
  "timeout"
]
```

4. 重启 LSP 服务器，在编辑器中应该能看到新字段的补全提示

## 故障排查

### Schema 文件未生成

**检查：**
```bash
cargo clean
cargo build --verbose
```

查看是否有 `cargo:warning=Schema generated at:` 输出。

**可能原因：**
- build.rs 未正确配置
- spring 依赖未添加到 build-dependencies
- submit_config_schema! 未调用

### LSP 未加载 Schema

**检查 LSP 日志：**
```bash
RUST_LOG=summer_lsp=debug cargo run
```

查找类似的日志：
```
[INFO] Loading schema from target directory: ...
[INFO] Loaded 1 local schemas from target
```

**可能原因：**
- target 目录不存在（未编译过）
- Schema 文件路径不正确
- LSP 工作空间路径配置错误

### Schema 内容不正确

**验证 Schema 格式：**
```bash
cat $(find target -name "summer-lsp.schema.json" | head -1) | jq .
```

**检查：**
- 是否有 `properties` 字段
- 配置前缀是否正确
- 字段类型是否正确

## 性能测试

### 测试大型项目

创建一个包含多个配置的项目：

```rust
// 定义 10 个配置结构
#[derive(Configurable, Deserialize)]
#[config_prefix = "service-1"]
pub struct Service1Config { /* ... */ }

submit_config_schema!("service-1", Service1Config);

// ... 重复 10 次
```

编译并测量时间：

```bash
time cargo build
```

检查 Schema 文件大小：

```bash
ls -lh $(find target -name "summer-lsp.schema.json" | head -1)
```

### 测试 LSP 启动时间

```bash
time cargo run -- --version
```

## 自动化测试

### CI/CD 集成

在 `.github/workflows/test.yml` 中添加：

```yaml
name: Test Schema Loading

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run unit tests
        run: cargo test test_find_schema_in_target --lib
        working-directory: summer-lsp
      
      - name: Run integration test
        run: cargo run --example test_schema_loading
        working-directory: summer-lsp
      
      - name: Build test project
        run: |
          cargo new test-app
          cd test-app
          # 添加配置和 build.rs
          cargo build
          # 验证 Schema 文件存在
          find target -name "summer-lsp.schema.json" | grep -q .
```

## 总结

测试 Schema 加载功能的完整流程：

1. ✅ 运行单元测试：`cargo test test_find_schema_in_target`
2. ✅ 运行集成测试：`cargo run --example test_schema_loading`
3. ✅ 创建测试项目并验证 Schema 生成
4. ✅ 启动 LSP 并验证 Schema 加载
5. ✅ 在编辑器中测试智能补全和验证功能

所有测试通过即表示功能正常工作！
