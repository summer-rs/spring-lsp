# 本地配置结构支持

## 概述

summer-lsp 支持三种方式来识别和验证本地配置结构：

1. **推荐方式**：使用 summer-rs 内置的 Schema 生成功能（精确、完整、零配置）
2. **备选方式**：手动创建 `.summer-lsp.schema.json` 文件
3. **Fallback 方式**：自动扫描 Rust 代码（简单、但类型推断有限）

## 方式一：使用 summer-rs 内置 Schema 生成（推荐）

### 快速开始

如果你的项目使用了 summer-rs，只需三步：

#### 1. 定义配置结构

```rust
use spring::config::Configurable;
use spring::submit_config_schema;
use serde::Deserialize;

/// Web DAV 客户端配置
#[derive(Debug, Configurable, Deserialize)]
#[config_prefix = "web-dav-client"]
pub struct WebDavClientConfig {
    /// WebDAV 用户名
    pub username: String,
    /// WebDAV 密码
    pub password: String,
    /// 是否使用原始资源
    #[serde(default)]
    pub use_origin: bool,
}

// 注册配置 Schema
submit_config_schema!("web-dav-client", WebDavClientConfig);
```

#### 2. 添加 build.rs 脚本

```rust
// build.rs
use spring::config::write_merged_schema_to_file;
use std::env;

fn main() {
    // 生成到 target 目录
    let out_dir = env::var("OUT_DIR").unwrap();
    let schema_path = format!("{}/summer-lsp.schema.json", out_dir);
    
    write_merged_schema_to_file(&schema_path)
        .expect("Failed to write schema file");
    
    println!("cargo:rerun-if-changed=src/config.rs");
}
```

#### 3. 编译项目

```bash
cargo build  # Schema 自动生成到 target 目录
```

完成！summer-lsp 会自动加载 Schema 文件。

### 多 Crate 工作空间支持

在 Cargo workspace 中，每个 crate 可能都有自己的配置结构。summer-lsp 会自动：

1. **查找所有 Schema 文件**：扫描 `target/*/build/*/out/summer-lsp.schema.json`
2. **自动合并**：将所有 crate 的配置合并为一个完整的 Schema
3. **冲突处理**：如果多个 crate 定义了相同的配置前缀，后编译的会覆盖先编译的

#### 示例：多 Crate 工作空间

```
my-workspace/
├── Cargo.toml          # workspace 配置
├── service-a/          # crate 1
│   ├── Cargo.toml
│   ├── build.rs        # 生成 service-a 的 Schema
│   └── src/
│       └── config.rs   # 定义 service-a 的配置
└── service-b/          # crate 2
    ├── Cargo.toml
    ├── build.rs        # 生成 service-b 的 Schema
    └── src/
        └── config.rs   # 定义 service-b 的配置
```

编译后，summer-lsp 会自动合并两个 crate 的配置：

```bash
cargo build
# 生成：
# - target/debug/build/service-a-xxx/out/summer-lsp.schema.json
# - target/debug/build/service-b-xxx/out/summer-lsp.schema.json
# summer-lsp 自动合并为一个完整的 Schema
```

在 `config/app.toml` 中可以使用所有 crate 的配置：

```toml
[service-a]
endpoint = "http://localhost:8080"

[service-b]
port = 9090
```

### 工作原理

summer-rs 使用 `inventory` crate 在编译时自动收集所有配置：

1. **注册阶段**：`submit_config_schema!` 宏注册配置
2. **收集阶段**：`inventory::iter` 遍历所有注册的配置
3. **生成阶段**：使用 `schemars` 为每个配置生成 Schema
4. **合并阶段**：将所有 Schema 合并为一个 JSON 对象

### 优势

- ✅ **零配置**：summer-rs 已内置 Schema 生成功能
- ✅ **自动收集**：无需手动维护配置列表
- ✅ **完整类型信息**：包括格式、约束、枚举值等
- ✅ **支持所有 serde 属性**：`rename`、`default`、`skip` 等
- ✅ **编译时验证**：类型安全

详细文档请参考：[SCHEMA_GENERATION_GUIDE.md](./SCHEMA_GENERATION_GUIDE.md)

## 方式二：手动创建 Schema 文件

如果你不使用 summer-rs 的插件系统，可以手动创建 `.summer-lsp.schema.json`：

```json
{
  "properties": {
    "web-dav-client": {
      "type": "object",
      "properties": {
        "username": {
          "type": "string",
          "description": "WebDAV 用户名"
        },
        "password": {
          "type": "string",
          "description": "WebDAV 密码"
        },
        "use-origin": {
          "type": "boolean",
          "default": false,
          "description": "是否使用原始资源"
        }
      },
      "required": ["username", "password"]
    }
  }
}
```

## 方式三：自动扫描（Fallback）

如果没有 `.summer-lsp.schema.json` 文件，LSP 会自动扫描项目中的 Rust 代码。

### 功能特性

### 1. 自动扫描本地配置

LSP 会自动扫描项目中所有带有 `#[derive(Configurable)]` 的结构体：

```rust
use serde::Deserialize;

/// Web DAV 客户端配置
#[derive(Debug, Configurable, Deserialize)]
#[config_prefix = "web-dav-client"]
pub struct OpenListConfig {
    /// WebDAV 用户名
    pub username: String,
    /// WebDAV 密码
    pub password: String,
    /// 是否使用原始资源
    #[serde(default)]
    pub use_origin: bool,
}
```

### 2. Schema 自动生成

扫描到的配置结构会自动转换为 JSON Schema：

- 提取 `#[config_prefix]` 作为配置节名称
- 提取字段名称和类型
- 提取文档注释作为描述
- 识别 `Option<T>` 类型作为可选字段

### 3. 与远程 Schema 合并

本地扫描的 Schema 会与远程 Schema（https://summer-rs.github.io/config-schema.json）合并：

- 远程 Schema：summer-rs 框架的官方插件配置
- 本地 Schema：项目中自定义的配置结构
- 合并策略：本地 Schema 优先（可以覆盖远程 Schema）

### 4. 完整的验证支持

本地配置享受与官方插件相同的验证支持：

- ✅ 类型验证
- ✅ 必需字段检查
- ✅ 文档提示（Hover）
- ✅ 未定义字段提示

## 使用示例

### 定义配置结构

```rust
// src/config.rs

use serde::Deserialize;

/// 自定义数据库配置
#[derive(Debug, Configurable, Deserialize)]
#[config_prefix = "custom-db"]
pub struct CustomDatabaseConfig {
    /// 数据库主机
    pub host: String,
    /// 数据库端口
    pub port: u16,
    /// 连接超时（秒）
    pub timeout: Option<u64>,
}
```

### 在配置文件中使用

```toml
# config/app.toml

[custom-db]
host = "localhost"
port = 5432
timeout = 30  # 可选字段
```

### LSP 验证效果

- ✅ `host`, `port`, `timeout` 字段被识别
- ✅ 类型错误会被检测（如 `port = "abc"`）
- ✅ 悬停时显示文档注释
- 💡 未定义的字段产生提示（如 `unknown_field = "test"`）

## 类型映射

Rust 类型会自动映射到 JSON Schema 类型：

| Rust 类型 | JSON Schema 类型 |
|-----------|-----------------|
| `String`, `&str` | `string` |
| `bool` | `boolean` |
| `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64` | `integer` |
| `f32`, `f64` | `number` |
| `Vec<T>` | `array` |
| `HashMap<K, V>`, `BTreeMap<K, V>` | `object` |
| `Option<T>` | 可选的 T 类型 |

## API 使用

### 在代码中使用

```rust
use summer_lsp::schema::SchemaProvider;
use std::path::Path;

// 加载 Schema（包含本地扫描）
let workspace_path = Path::new(".");
let schema_provider = SchemaProvider::load_with_workspace(workspace_path)
    .await?;

// 使用 schema_provider 进行验证
let analyzer = TomlAnalyzer::new(schema_provider);
```

### 仅加载远程 Schema

```rust
// 如果不需要本地扫描，使用原来的方法
let schema_provider = SchemaProvider::load().await?;
```

## 性能考虑

- 扫描过程会遍历项目中的所有 `.rs` 文件
- 自动跳过 `target` 目录
- 解析失败的文件会被跳过（不影响其他文件）
- 扫描结果会被缓存在 SchemaProvider 中

## 限制和注意事项

1. **需要 `#[config_prefix]` 属性**
   - 必须明确指定配置前缀
   - 格式：`#[config_prefix = "your-prefix"]`

2. **类型推断限制**
   - 复杂的泛型类型可能无法完全识别
   - 自定义类型会被映射为 `string`

3. **文档注释**
   - 只支持 `///` 风格的文档注释
   - `//` 普通注释不会被提取

4. **扫描范围**
   - 只扫描当前工作空间
   - 不扫描依赖库中的配置

## 示例项目

查看 `test_project/` 目录中的示例：

```bash
# 运行示例
cargo run --example test_local_schema
cargo run --example validate_with_local_schema
```

## 集成到 LSP 服务器

在 LSP 服务器初始化时，使用 `load_with_workspace` 方法：

```rust
// 在服务器初始化时
let workspace_folders = initialization_params.workspace_folders;
let workspace_path = workspace_folders
    .first()
    .map(|f| Path::new(&f.uri.path()))
    .unwrap_or(Path::new("."));

let schema_provider = SchemaProvider::load_with_workspace(workspace_path)
    .await?;
```

## 优势

1. **零配置**：只需添加 `#[derive(Configurable)]`，无需手动维护 Schema
2. **类型安全**：配置结构和验证规则保持同步
3. **文档集成**：文档注释自动成为 LSP 提示
4. **扩展性**：支持任意自定义配置结构

## 未来改进

1. 支持更多的 serde 属性（如 `#[serde(rename)]`）
2. 支持嵌套配置结构
3. 支持枚举类型的值验证
4. 增量扫描（只扫描变化的文件）
5. 缓存扫描结果到磁盘
