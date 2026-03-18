# summer-lsp

A Language Server Protocol (LSP) implementation for the [summer-rs](https://github.com/summer-rs/summer-rs) framework, providing intelligent IDE support for Rust applications built with summer-rs.

## Features

### 🎯 TOML Configuration Support
- **Smart completion** for configuration sections and properties
- **Real-time validation** with detailed error messages
- **Hover documentation** with type information and examples
- **Environment variable** support (`${VAR:default}` syntax)
- **Schema-based validation** with automatic schema loading

### 🔧 Rust Macro Analysis
- **Macro recognition** for summer-rs macros (`#[derive(Service)]`, `#[inject]`, route macros, job macros)
- **Macro expansion** with readable generated code
- **Parameter validation** and error reporting
- **Hover tooltips** with macro documentation and usage examples
- **Smart completion** for macro parameters

### 🌐 Route Management
- **Route detection** for all HTTP method macros (`#[get]`, `#[post]`, etc.)
- **Path parameter parsing** and validation
- **Conflict detection** for duplicate routes
- **Route navigation** and search capabilities
- **RESTful style validation**

### 🔍 Advanced Features
- **Dependency injection validation** with circular dependency detection
- **Component registration verification**
- **Performance monitoring** and server status queries
- **Configurable diagnostics** with custom filtering
- **Error recovery** with graceful degradation
- **Multi-document workspace** support

## Installation

### Prerequisites
- Rust 1.70+ 
- A compatible editor with LSP support (VS Code, Neovim, Emacs, etc.)

### From Source
```bash
git clone https://github.com/summer-rs/summer-lsp
cd summer-lsp
cargo build --release
```

The binary will be available at `target/release/summer-lsp`.

### From crates.io
```bash
cargo install summer-lsp
```

### Pre-built Binaries
Download pre-built binaries from the [releases page](https://github.com/summer-rs/summer-lsp/releases):

- Linux x86_64 (glibc and musl)
- macOS x86_64 and ARM64
- Windows x86_64

## Editor Setup

### VS Code

#### Option 1: Install the Extension (Recommended)

Install the official Summer RS LSP extension:

1. From VSCode Marketplace (coming soon)
2. Or install from VSIX:
   ```bash
   cd vscode
   npm install
   npm run package
   code --install-extension summer-rs-lsp-0.1.0.vsix
   ```

The extension will automatically detect and start the language server.

#### Option 2: Manual Configuration

If you prefer manual setup, add to your `settings.json`:

```json
{
  "summer-rs-lsp.enable": true,
  "summer-rs-lsp.serverPath": "/path/to/summer-lsp",
  "summer-rs-lsp.trace.server": "verbose"
}
```

See [vscode/README.md](vscode/README.md) for more details.

### Neovim (with nvim-lspconfig)
```lua
require'lspconfig'.summer_lsp.setup{
  cmd = {"/path/to/summer-lsp"},
  filetypes = {"toml", "rust"},
  root_dir = require'lspconfig'.util.root_pattern("Cargo.toml", ".summer-lsp.toml"),
}
```

### Emacs (with lsp-mode)
```elisp
(add-to-list 'lsp-language-id-configuration '(toml-mode . "toml"))
(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection "/path/to/summer-lsp")
                  :major-modes '(toml-mode rust-mode)
                  :server-id 'summer-lsp))
```

## Configuration

Create a `.summer-lsp.toml` file in your project root:

```toml
[completion]
trigger_characters = ["[", ".", "$", "{", "#", "("]

[schema]
url = "https://summer-rs.github.io/config-schema.json"

[diagnostics]
disabled = ["deprecated-config"]

[logging]
level = "info"
verbose = false
```

## Usage

### Local Configuration Schema Generation

For projects using summer-rs, you can generate a local configuration schema for enhanced LSP support:

#### Quick Start

1. **Define your configuration**:
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
}

fn default_timeout() -> u64 { 30 }

// Register the schema
submit_config_schema!("my-service", MyServiceConfig);
```

2. **Add build script**:
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
    
    // 当配置文件变化时重新生成
    println!("cargo:rerun-if-changed=src/config.rs");
}
```

3. **Build your project**:
```bash
cargo build  # Schema 自动生成到 target 目录
```

The generated schema will be automatically discovered by summer-lsp, providing:
- ✅ Smart completion for your custom configurations
- ✅ Type validation and error checking
- ✅ Hover documentation from your doc comments
- ✅ Support for all serde attributes

#### Multi-Crate Workspace Support

In a Cargo workspace with multiple crates, summer-lsp automatically:
- 🔍 Discovers all schema files from different crates
- 🔄 Merges them into a single unified schema
- ✨ Provides completion for all crate configurations

Example workspace structure:
```
my-workspace/
├── service-a/  # Generates schema for service-a configs
├── service-b/  # Generates schema for service-b configs
└── config/
    └── app.toml  # Can use configs from both crates
```

For detailed instructions, see [SCHEMA_GENERATION_GUIDE.md](SCHEMA_GENERATION_GUIDE.md).

### TOML Configuration Files
summer-lsp automatically provides intelligent support for `config/app.toml` and related configuration files:

```toml
# Smart completion for configuration sections
[web]
host = "0.0.0.0"  # Hover for documentation
port = 8080       # Type validation

# Environment variable support
[database]
url = "${DATABASE_URL:postgresql://localhost/mydb}"

# Validation and error reporting
[redis]
url = "redis://localhost:6379"
pool_size = 10    # Range validation
```

### Rust Code Analysis
summer-lsp analyzes your Rust code for summer-rs specific patterns:

```rust
// Service macro with dependency injection
#[derive(Clone, Service)]
struct UserService {
    #[inject(component)]
    db: ConnectPool,
    
    #[inject(config)]
    config: UserConfig,
}

// Route macros with validation
#[get("/users/{id}")]
async fn get_user(
    Path(id): Path<i64>,
    Component(service): Component<UserService>
) -> Result<Json<User>> {
    // Implementation
}

// Job scheduling macros
#[cron("0 0 * * * *")]
async fn cleanup_job() {
    // Hourly cleanup task
}
```

## Performance

summer-lsp is designed for high performance:

- **Startup time**: < 2 seconds
- **Completion response**: < 100ms
- **Diagnostic updates**: < 200ms
- **Memory usage**: < 50MB for typical projects
- **Concurrent documents**: 100+ supported

## Supported Features

| Feature | TOML | Rust | Status |
|---------|------|------|--------|
| Syntax highlighting | ✅ | ✅ | Complete |
| Completion | ✅ | ✅ | Complete |
| Hover documentation | ✅ | ✅ | Complete |
| Diagnostics | ✅ | ✅ | Complete |
| Go to definition | ⚠️ | ⚠️ | Partial |
| Document symbols | ⚠️ | ⚠️ | Planned |
| Workspace symbols | ⚠️ | ⚠️ | Planned |
| Code actions | ❌ | ❌ | Planned |
| Formatting | ❌ | ❌ | Planned |

## Architecture

summer-lsp follows a modular architecture:

```
┌─────────────────────────────────────────────────────────┐
│                    LSP Protocol Layer                    │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                   Server Core Layer                      │
│         (Message Dispatch, State Management)             │
└─────────────────────────────────────────────────────────┘
                            ↓
┌──────────────┬──────────────┬──────────────┬────────────┐
│   Config     │    Macro     │   Routing    │ Diagnostic │
│   Analysis   │   Analysis   │   Analysis   │   Engine   │
└──────────────┴──────────────┴──────────────┴────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                   Foundation Layer                       │
│      (Schema, Document, Index, Completion)              │
└─────────────────────────────────────────────────────────┘
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup
```bash
git clone https://github.com/summer-rs/summer-lsp
cd summer-lsp

# 运行测试
cargo test

# 启动服务器
cargo run
```

### Running Tests
```bash
# Unit tests
cargo test --lib

# Integration tests  
cargo test --tests

# Property-based tests
cargo test --release

# Performance tests
cargo test --release performance
```

## Documentation

- [VSCode Extension Guide](vscode/README.md) - VSCode extension usage
- [API Documentation](https://docs.rs/summer-lsp) - Rust API docs
- [Contributing Guide](CONTRIBUTING.md) - Development guidelines

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release history.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

- [summer-rs](https://github.com/summer-rs/summer-rs) - The amazing Rust application framework
- [taplo](https://github.com/tamasfe/taplo) - TOML parsing and analysis
- [lsp-server](https://github.com/rust-lang/rust-analyzer/tree/master/lib/lsp-server) - LSP protocol implementation
- [rust-analyzer](https://github.com/rust-lang/rust-analyzer) - Inspiration for LSP architecture

---

**summer-lsp** - Intelligent IDE support for summer-rs applications