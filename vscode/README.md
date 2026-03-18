# Summer RS

[![Version](https://img.shields.io/visual-studio-marketplace/v/holmofy.summer-rs)](https://marketplace.visualstudio.com/items?itemName=holmofy.summer-rs)
[![Installs](https://img.shields.io/visual-studio-marketplace/i/holmofy.summer-rs)](https://marketplace.visualstudio.com/items?itemName=holmofy.summer-rs)
[![Rating](https://img.shields.io/visual-studio-marketplace/r/holmofy.summer-rs)](https://marketplace.visualstudio.com/items?itemName=holmofy.summer-rs)

Language Server Protocol support and application management for the [summer-rs](https://summer-rs.github.io/) framework.

## Features

### 🚀 Application Management

- **Auto-detect** summer-rs applications in your workspace
- **Run/Debug** applications with one click
- **Profile support** - easily switch between development, production, and custom profiles
- **Port detection** - automatically detect and display application ports
- **Open in browser** - quickly open running applications
- **Batch operations** - run or stop multiple applications at once

### 🔍 Real-time Application Insights

When your application is running, get instant visibility into:

- **Components** - view all registered components and their dependencies
- **Routes** - browse all HTTP endpoints with methods and paths
- **Jobs** - see scheduled tasks and cron expressions
- **Plugins** - inspect loaded plugins and their configurations

### 📊 Dependency Graph Visualization

Visualize component dependencies with an interactive graph powered by D3.js:

- Click nodes to navigate to component definitions
- Identify circular dependencies
- Understand your application architecture at a glance

### 🎯 Smart TOML Configuration Support

- **Intelligent completion** for `config/app.toml` files
- **Schema validation** based on summer-rs configuration metadata
- **Environment variable** interpolation support (`${VAR:default}`)
- **Jump to definition** from configuration to Rust code

### 🛠️ Code Navigation

- Navigate from routes to handler functions
- Jump from components to their definitions
- Show component dependencies
- Quick access to documentation

## Installation

### Prerequisites

- Visual Studio Code 1.75.0 or higher
- Rust toolchain (rustc, cargo)
- [summer-lsp](https://github.com/summer-rs/summer-lsp) language server (optional, bundled with extension)

### From VS Code Marketplace

1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X / Cmd+Shift+X)
3. Search for "Summer RS"
4. Click Install

### From VSIX

1. Download the latest `.vsix` file from [Releases](https://github.com/summer-rs/summer-lsp/releases)
2. Open VS Code
3. Go to Extensions
4. Click "..." menu → "Install from VSIX..."
5. Select the downloaded file

## Getting Started

### 1. Create a summer-rs Project

```bash
cargo new my-summer-app
cd my-summer-app
```

Add summer-rs to your `Cargo.toml`:

```toml
[dependencies]
summer = "0.1"
summer-web = "0.1"
tokio = { version = "1", features = ["full"] }
```

### 2. Create Configuration

Create `config/app.toml`:

```toml
[web]
host = "0.0.0.0"
port = 8080
```

### 3. Write Your Application

```rust
use summer::App;
use summer_web::{get, WebPlugin};
use summer_web::axum::response::IntoResponse;

#[get("/")]
async fn hello() -> impl IntoResponse {
    "Hello, Summer RS!"
}

#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(WebPlugin)
        .run()
        .await
}
```

### 4. Use the Extension

1. Open your project in VS Code
2. The extension will automatically detect your summer-rs application
3. Click the Summer RS icon in the Activity Bar
4. Click the ▶️ (Run) button next to your app
5. Once running, explore Components, Routes, and other views

## Configuration

### Extension Settings

This extension contributes the following settings:

- `summer-rs.serverPath`: Path to summer-lsp executable (leave empty to use bundled version)
- `summer-rs.openWith`: Browser to use when opening apps (`integrated` or `external`)
- `summer-rs.openUrl`: URL template for opening apps (default: `http://localhost:{port}{contextPath}`)
- `summer-rs.enableGutter`: Show gutter icons in editors (`on` or `off`)
- `summer-rs.env`: Environment variables to set when running apps
- `summer-rs.trace.server`: Trace LSP communication (`off`, `messages`, or `verbose`)

### Example Configuration

```json
{
  "summer-rs.openWith": "external",
  "summer-rs.env": {
    "RUST_LOG": "debug",
    "DATABASE_URL": "postgres://localhost/mydb"
  },
  "summer-rs.trace.server": "messages"
}
```

## Usage

### Running Applications

**Single Application:**
- Click the ▶️ (Run) button in the Apps view
- Or right-click an app → "Run"

**With Profile:**
- Right-click an app → "Run with Profile..."
- Select one or more profiles (e.g., `dev`, `prod`)

**Multiple Applications:**
- Click the "Run Multiple Apps" button in the view title
- Select apps to run

### Debugging Applications

- Click the 🐛 (Debug) button in the Apps view
- Or right-click an app → "Debug"
- Set breakpoints in your Rust code
- Use VS Code's debugging features

### Viewing Application Information

Once an application is running:

1. **Components View** - See all registered components
   - Click a component to navigate to its definition
   - Right-click → "Show Dependencies" to visualize the dependency graph

2. **Routes View** - Browse all HTTP endpoints
   - Grouped by HTTP method (GET, POST, etc.)
   - Click a route to navigate to the handler function
   - Right-click GET routes → "Open in Browser"

3. **Jobs View** - See scheduled tasks
   - View cron expressions and schedules
   - Navigate to job definitions

4. **Plugins View** - Inspect loaded plugins
   - See plugin names and versions
   - View plugin configurations

### Opening in Browser

**From Apps View:**
- Click the 🌐 (Globe) button next to a running app

**From Routes View:**
- Right-click a GET route → "Open in Browser"

**Custom URL Template:**
```json
{
  "summer-rs.openUrl": "https://myapp.local:{port}{contextPath}"
}
```

## Keyboard Shortcuts

| Command | Shortcut | Description |
|---------|----------|-------------|
| Refresh Apps | - | Refresh the application list |
| Show Welcome | - | Show the welcome page |
| Open Documentation | - | Open summer-rs documentation |

You can customize shortcuts in VS Code's Keyboard Shortcuts editor.

## Troubleshooting

### Language Server Not Starting

**Problem:** Extension shows "Language server failed to start"

**Solutions:**
1. Check if `summer-lsp` is installed:
   ```bash
   summer-lsp --version
   ```

2. Specify the path manually:
   ```json
   {
     "summer-rs.serverPath": "/path/to/summer-lsp"
   }
   ```

3. Install from source:
   ```bash
   cargo install summer-lsp
   ```

### Application Not Detected

**Problem:** Your summer-rs app doesn't appear in the Apps view

**Solutions:**
1. Ensure `Cargo.toml` includes summer-rs dependencies:
   ```toml
   [dependencies]
   summer = "0.1"
   ```

2. Ensure your crate is a binary (has `src/main.rs` or `[[bin]]` section)

3. Click the refresh button in the Apps view

4. Check the Output panel (View → Output → Summer RS) for errors

### Port Detection Issues

**Problem:** Extension can't detect the application port

**Solutions:**
1. Ensure `config/app.toml` has the port configured:
   ```toml
   [web]
   port = 8080
   ```

2. The extension will use port 8080 by default if not configured

### Components/Routes Not Showing

**Problem:** Views are empty even though the app is running

**Solutions:**
1. Ensure the language server is running (check Output panel)

2. Wait a few seconds for the app to fully start

3. Click the refresh button in the view

4. Check if the app is actually running (look for the green icon)

## Known Issues

- Gutter icons feature is not yet implemented
- Code snippets are not yet available
- Multi-workspace support is limited

See the [issue tracker](https://github.com/summer-rs/summer-lsp/issues) for a complete list.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](https://github.com/summer-rs/summer-lsp/blob/main/CONTRIBUTING.md) for details.

### Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/summer-rs/summer-lsp.git
   cd summer-lsp/vscode
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Open in VS Code:
   ```bash
   code .
   ```

4. Press F5 to launch the Extension Development Host

5. Make changes and reload the window (Ctrl+R / Cmd+R)

### Debugging the Extension

For detailed debugging instructions, see [DEBUG_EXTENSION.md](DEBUG_EXTENSION.md).

**Quick Start:**
1. Start watch mode: `npm run watch`
2. Press F5 to start debugging
3. Set breakpoints in TypeScript files
4. Test in the Extension Development Host window

**Available Debug Configurations:**
- **Run Extension** - Debug basic functionality
- **Run Extension (with test project)** - Debug with a specific project
- **Extension Tests** - Debug automated tests

## Resources

- [summer-rs Documentation](https://summer-rs.github.io/)
- [summer-lsp GitHub](https://github.com/summer-rs/summer-lsp)
- [Issue Tracker](https://github.com/summer-rs/summer-lsp/issues)
- [Changelog](CHANGELOG.md)

## License

This extension is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

This extension is inspired by:
- [vscode-spring-boot-dashboard](https://github.com/microsoft/vscode-spring-boot-dashboard) - Microsoft's Spring Boot Dashboard
- [spring-tools](https://github.com/spring-projects/sts4) - Spring Tools for various IDEs

Special thanks to the summer-rs community and all contributors!

---

**Enjoy coding with Summer RS! 🚀**
