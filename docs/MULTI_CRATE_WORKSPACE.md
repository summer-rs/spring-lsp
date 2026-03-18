# 多 Crate 工作空间支持

## 概述

summer-lsp 完全支持 Cargo workspace 中的多 crate 项目。每个 crate 可以定义自己的配置结构，summer-lsp 会自动发现、合并并提供智能补全。

## 功能特性

### 1. 自动发现

summer-lsp 会扫描整个 workspace 的 target 目录，查找所有的 Schema 文件：

```
target/
├── debug/
│   └── build/
│       ├── service-a-xxx/
│       │   └── out/
│       │       └── summer-lsp.schema.json  ✓ 发现
│       ├── service-b-xxx/
│       │   └── out/
│       │       └── summer-lsp.schema.json  ✓ 发现
│       └── service-c-xxx/
│           └── out/
│               └── summer-lsp.schema.json  ✓ 发现
└── release/
    └── build/
        └── ...  ✓ 也会扫描
```

### 2. 智能合并

找到多个 Schema 文件后，summer-lsp 会：

1. **按修改时间排序**：最新编译的 crate 优先
2. **合并所有配置**：将所有 crate 的配置合并为一个完整的 Schema
3. **处理冲突**：如果多个 crate 定义了相同的配置前缀，后编译的会覆盖先编译的
4. **生成临时文件**：合并后的 Schema 保存到 `/tmp/summer-lsp-merged.schema.json`

### 3. 统一体验

合并后，在 `config/app.toml` 中可以使用所有 crate 的配置：

```toml
# service-a 的配置
[service-a]
endpoint = "http://localhost:8080"
timeout = 30

# service-b 的配置
[service-b]
port = 9090
workers = 4

# service-c 的配置
[service-c]
cache_size = 1000
```

所有配置都享受相同的 LSP 功能：
- ✅ 智能补全
- ✅ 类型验证
- ✅ 悬停文档
- ✅ 错误诊断

## 使用示例

### 工作空间结构

```
my-workspace/
├── Cargo.toml          # workspace 配置
│   [workspace]
│   members = ["service-a", "service-b", "shared"]
│
├── service-a/          # 微服务 A
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/
│       └── config.rs
│
├── service-b/          # 微服务 B
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/
│       └── config.rs
│
├── shared/             # 共享库
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/
│       └── config.rs
│
└── config/             # 统一配置目录
    ├── app.toml        # 主配置文件
    ├── app-dev.toml    # 开发环境配置
    └── app-prod.toml   # 生产环境配置
```

### service-a/src/config.rs

```rust
use spring::config::Configurable;
use spring::submit_config_schema;
use serde::Deserialize;

#[derive(Debug, Configurable, Deserialize)]
#[config_prefix = "service-a"]
pub struct ServiceAConfig {
    /// API endpoint URL
    pub endpoint: String,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_timeout() -> u64 { 30 }

submit_config_schema!("service-a", ServiceAConfig);
```

### service-b/src/config.rs

```rust
use spring::config::Configurable;
use spring::submit_config_schema;
use serde::Deserialize;

#[derive(Debug, Configurable, Deserialize)]
#[config_prefix = "service-b"]
pub struct ServiceBConfig {
    /// HTTP server port
    pub port: u16,
    /// Number of worker threads
    #[serde(default = "default_workers")]
    pub workers: usize,
}

fn default_workers() -> usize { 4 }

submit_config_schema!("service-b", ServiceBConfig);
```

### shared/src/config.rs

```rust
use spring::config::Configurable;
use spring::submit_config_schema;
use serde::Deserialize;

#[derive(Debug, Configurable, Deserialize)]
#[config_prefix = "cache"]
pub struct CacheConfig {
    /// Cache size in MB
    pub size_mb: u64,
    /// Cache TTL in seconds
    #[serde(default = "default_ttl")]
    pub ttl: u64,
}

fn default_ttl() -> u64 { 3600 }

submit_config_schema!("cache", CacheConfig);
```

### 统一的 build.rs

所有 crate 使用相同的 build.rs：

```rust
// build.rs
use spring::config::write_merged_schema_to_file;
use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let schema_path = format!("{}/summer-lsp.schema.json", out_dir);
    
    write_merged_schema_to_file(&schema_path)
        .expect("Failed to write schema file");
    
    println!("cargo:rerun-if-changed=src/config.rs");
}
```

### config/app.toml

```toml
#:schema https://summer-rs.github.io/config-schema.json

# service-a 配置
[service-a]
endpoint = "http://api.example.com"
timeout = 60

# service-b 配置
[service-b]
port = 8080
workers = 8

# 共享缓存配置
[cache]
size_mb = 512
ttl = 7200
```

## 编译和使用

### 编译整个 workspace

```bash
# 编译所有 crate
cargo build

# 或者只编译特定 crate
cargo build -p service-a
cargo build -p service-b
```

### 查看生成的 Schema 文件

```bash
# 查找所有生成的 Schema 文件
find target -name "summer-lsp.schema.json"

# 输出示例：
# target/debug/build/service-a-xxx/out/summer-lsp.schema.json
# target/debug/build/service-b-xxx/out/summer-lsp.schema.json
# target/debug/build/shared-xxx/out/summer-lsp.schema.json
```

### 查看合并后的 Schema

```bash
# summer-lsp 会将合并后的 Schema 保存到临时目录
cat /tmp/summer-lsp-merged.schema.json | jq .
```

## 日志和调试

### 启用详细日志

```bash
# 设置日志级别
export RUST_LOG=summer_lsp=debug

# 启动 LSP 服务器
summer-lsp
```

### 日志输出示例

```
INFO summer_lsp::core::schema: Loading schema from target directory: /path/to/workspace
DEBUG summer_lsp::core::schema: Found 3 schema files, merging...
DEBUG summer_lsp::core::schema: Merging schema from: target/debug/build/service-a-xxx/out/summer-lsp.schema.json
DEBUG summer_lsp::core::schema: Merging schema from: target/debug/build/service-b-xxx/out/summer-lsp.schema.json
DEBUG summer_lsp::core::schema: Merging schema from: target/debug/build/shared-xxx/out/summer-lsp.schema.json
INFO summer_lsp::core::schema: Merged 3 schema files (6 configs) into: /tmp/summer-lsp-merged.schema.json
INFO summer_lsp::core::schema: Loaded 6 local schemas from file
```

## 最佳实践

### 1. 使用唯一的配置前缀

每个 crate 应该使用唯一的配置前缀，避免冲突：

```rust
// ✅ 好的做法
#[config_prefix = "service-a"]  // service-a crate
#[config_prefix = "service-b"]  // service-b crate
#[config_prefix = "cache"]      // shared crate

// ❌ 避免这样做
#[config_prefix = "config"]     // 太通用，容易冲突
#[config_prefix = "settings"]   // 太通用，容易冲突
```

### 2. 共享配置结构

如果多个 crate 需要相同的配置，考虑提取到共享 crate：

```rust
// shared/src/database.rs
#[derive(Debug, Configurable, Deserialize)]
#[config_prefix = "database"]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
}

submit_config_schema!("database", DatabaseConfig);

// service-a 和 service-b 都可以使用
use shared::DatabaseConfig;
```

### 3. 统一 build.rs

所有 crate 使用相同的 build.rs 模板，保持一致性。

### 4. 配置文件组织

使用环境特定的配置文件：

```
config/
├── app.toml          # 基础配置
├── app-dev.toml      # 开发环境覆盖
├── app-test.toml     # 测试环境覆盖
└── app-prod.toml     # 生产环境覆盖
```

### 5. 文档注释

充分利用文档注释，它们会出现在 LSP 的悬停提示中：

```rust
#[derive(Configurable, Deserialize)]
#[config_prefix = "service-a"]
pub struct ServiceAConfig {
    /// API endpoint URL
    ///
    /// Format: `http://host:port/path`
    ///
    /// # Example
    ///
    /// ```toml
    /// [service-a]
    /// endpoint = "http://api.example.com/v1"
    /// ```
    pub endpoint: String,
}
```

## 故障排查

### Schema 文件未被发现

1. **检查 build.rs 是否正确执行**：
   ```bash
   cargo clean
   cargo build -vv  # 查看详细输出
   ```

2. **检查 Schema 文件是否生成**：
   ```bash
   find target -name "summer-lsp.schema.json"
   ```

3. **检查 LSP 日志**：
   ```bash
   RUST_LOG=summer_lsp=debug summer-lsp
   ```

### 配置未被识别

1. **检查配置前缀是否匹配**：
   - Rust 代码：`#[config_prefix = "service-a"]`
   - TOML 文件：`[service-a]`

2. **检查 Schema 内容**：
   ```bash
   cat /tmp/summer-lsp-merged.schema.json | jq '.properties | keys'
   ```

3. **重新编译**：
   ```bash
   cargo clean
   cargo build
   ```

### 配置冲突

如果多个 crate 定义了相同的配置前缀：

1. **检查所有 crate 的配置前缀**：
   ```bash
   rg "#\[config_prefix" --type rust
   ```

2. **重命名冲突的前缀**：
   ```rust
   // 从
   #[config_prefix = "config"]
   // 改为
   #[config_prefix = "service-a-config"]
   ```

3. **更新配置文件**：
   ```toml
   # 从
   [config]
   # 改为
   [service-a-config]
   ```

## 测试

### 单元测试

summer-lsp 包含完整的单元测试：

```bash
# 运行所有测试
cargo test --lib

# 运行多 crate 相关的测试
cargo test --lib test_merge_schema_files
cargo test --lib test_find_schema_in_target_multiple_crates
```

### 集成测试

创建测试 workspace：

```bash
# 创建测试 workspace
mkdir test-workspace
cd test-workspace

# 创建 workspace Cargo.toml
cat > Cargo.toml << 'EOF'
[workspace]
members = ["crate-a", "crate-b"]
EOF

# 创建 crate-a
cargo new crate-a --lib
# 添加配置和 build.rs

# 创建 crate-b
cargo new crate-b --lib
# 添加配置和 build.rs

# 编译
cargo build

# 验证 Schema 合并
find target -name "summer-lsp.schema.json"
```

## 性能考虑

### Schema 文件大小

- 每个 crate 的 Schema 通常在 1-10 KB
- 合并后的 Schema 通常在 10-100 KB
- 对 LSP 性能影响可忽略不计

### 扫描性能

- 扫描 target 目录通常在 10-100ms
- 合并 Schema 文件通常在 1-10ms
- 只在 LSP 启动时执行一次

### 内存使用

- Schema 加载到内存后占用很小（通常 < 1 MB）
- 不会随着 crate 数量线性增长

## 参考资源

- [SCHEMA_GENERATION_GUIDE.md](../SCHEMA_GENERATION_GUIDE.md) - Schema 生成详细指南
- [LOCAL_CONFIG_SUPPORT.md](../LOCAL_CONFIG_SUPPORT.md) - 本地配置支持文档
- [TESTING_GUIDE.md](./TESTING_GUIDE.md) - 测试指南
- [summer-rs 文档](https://summer-rs.github.io/)

## 常见问题

### Q: 为什么需要多 crate 支持？

A: 在微服务架构中，每个服务通常是一个独立的 crate，但它们可能共享同一个配置文件。多 crate 支持让你可以在一个配置文件中管理所有服务的配置。

### Q: 合并后的 Schema 保存在哪里？

A: 合并后的 Schema 保存在系统临时目录：`/tmp/summer-lsp-merged.schema.json`（Linux/macOS）或 `%TEMP%\summer-lsp-merged.schema.json`（Windows）。

### Q: 如果两个 crate 定义了相同的配置前缀会怎样？

A: 后编译的 crate 会覆盖先编译的。建议使用唯一的配置前缀避免冲突。

### Q: 是否支持跨 workspace 的 Schema 合并？

A: 目前只支持单个 workspace 内的合并。如果你有多个 workspace，每个 workspace 会有自己的合并 Schema。

### Q: 如何强制重新生成 Schema？

A: 运行 `cargo clean && cargo build` 会重新生成所有 Schema 文件。

### Q: 是否支持增量编译？

A: 是的。只有修改了配置相关代码的 crate 会重新生成 Schema，其他 crate 的 Schema 会被复用。

### Q: 如何在 CI 中使用？

A: 在 CI 中正常编译项目即可，Schema 会自动生成。如果需要验证 Schema，可以添加测试：

```bash
# .github/workflows/ci.yml
- name: Build and generate schemas
  run: cargo build

- name: Verify schemas exist
  run: |
    if [ -z "$(find target -name 'summer-lsp.schema.json')" ]; then
      echo "No schema files found!"
      exit 1
    fi
```
