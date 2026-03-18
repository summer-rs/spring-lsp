# Change Log

All notable changes to the "Summer LSP for Rust" extension will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned Features
- Gutter icons for components, routes, and jobs
- Code snippets for common summer-rs patterns
- Multi-workspace support improvements
- Performance optimizations for large projects
- Remote development enhancements

## [0.1.0] - 2024-01-XX

### Added

#### Application Management
- Auto-detection of summer-rs applications in workspace
- Run and debug applications with one click
- Profile selection support (dev, prod, custom profiles)
- Automatic port detection from `config/app.toml`
- Open running applications in browser (integrated or external)
- Batch operations (run/stop multiple apps)
- Application state tracking (inactive, launching, running, stopping)

#### Real-time Application Insights
- **Apps View** - List all detected summer-rs applications
  - Display app name, version, state, and port
  - Click to open configuration file
  - Context menu with run/debug/stop actions
  
- **Components View** - Show registered components (when app is running)
  - Display component name, type, and scope
  - Navigate to component definitions
  - Show component dependencies
  
- **Routes View** - Browse HTTP endpoints (when app is running)
  - Group routes by HTTP method
  - Display path, handler, and method
  - Navigate to handler functions
  - Open GET routes in browser
  
- **Jobs View** - Display scheduled tasks (when app is running)
  - Show job name and cron expression
  - Navigate to job definitions
  
- **Plugins View** - Inspect loaded plugins (when app is running)
  - Display plugin name and version
  - Show plugin configurations

#### Dependency Graph Visualization
- Interactive dependency graph using D3.js
- Click nodes to navigate to component definitions
- Visual indication of circular dependencies
- Force-directed layout for clear visualization

#### Language Server Integration
- LSP client for summer-lsp language server
- Automatic server discovery (config, extension bundle, system PATH)
- TOML configuration file support
- Rust source file support
- LSP communication tracing (off, messages, verbose)

#### Commands
- `Summer RS: Refresh Apps` - Refresh application list
- `Summer RS: Run` - Run selected application
- `Summer RS: Debug` - Debug selected application
- `Summer RS: Stop` - Stop running application
- `Summer RS: Open in Browser` - Open application in browser
- `Summer RS: Run with Profile...` - Run with profile selection
- `Summer RS: Debug with Profile...` - Debug with profile selection
- `Summer RS: Run Multiple Apps...` - Batch run applications
- `Summer RS: Stop Multiple Apps...` - Batch stop applications
- `Summer RS: Go to Definition` - Navigate to component/route definition
- `Summer RS: Show Dependencies` - Show dependency graph
- `Summer RS: Show Welcome Page` - Display welcome page
- `Summer RS: Open Documentation` - Open summer-rs documentation

#### Configuration
- `summer-lsp.serverPath` - Custom language server path
- `summer-lsp.openWith` - Browser selection (integrated/external)
- `summer-lsp.openUrl` - URL template with placeholders
- `summer-lsp.enableGutter` - Toggle gutter icons (not yet implemented)
- `summer-lsp.env` - Environment variables for running apps
- `summer-lsp.trace.server` - LSP communication tracing level

#### Developer Experience
- Comprehensive test suite (310+ test cases)
- TypeScript with strict type checking
- ESLint and Prettier for code quality
- Detailed development documentation
- Project structure documentation
- Task completion tracking

### Technical Details
- Built with TypeScript 5.0
- Uses vscode-languageclient 9.0 for LSP communication
- Mocha + @vscode/test-electron for testing
- D3.js for dependency graph visualization
- @iarna/toml for TOML parsing

### Known Limitations
- Gutter icons feature not yet implemented
- Code snippets not yet available
- Multi-workspace support is basic
- Remote development support is limited
- No configuration hot-reload for some settings

## [0.0.1] - 2024-01-XX (Initial Development)

### Added
- Initial project setup
- Basic extension structure
- Core data models (SummerApp, AppState)
- LocalAppManager for application detection
- LocalAppController for application lifecycle
- Basic views (Apps, Components, Routes)

---

## Release Notes Format

### Version Number Guidelines
- **Major (X.0.0)**: Breaking changes, major new features
- **Minor (0.X.0)**: New features, backward compatible
- **Patch (0.0.X)**: Bug fixes, minor improvements

### Change Categories
- **Added**: New features
- **Changed**: Changes in existing functionality
- **Deprecated**: Soon-to-be removed features
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Security**: Security fixes

---

## Upgrade Guide

### From 0.0.x to 0.1.0

This is the first official release. No upgrade steps required.

**New Features:**
- Complete application management system
- Real-time application insights
- Dependency graph visualization
- Full LSP integration

**Configuration Changes:**
- Extension ID changed from `summer-rs-lsp` to `summer-lsp`
- Configuration prefix changed from `summer-rs-lsp.*` to `summer-lsp.*`

If you were using the development version, update your settings:

```json
// Old (0.0.x)
{
  "summer-rs-lsp.serverPath": "/path/to/server",
  "summer-rs-lsp.trace.server": "verbose"
}

// New (0.1.0)
{
  "summer-lsp.serverPath": "/path/to/server",
  "summer-lsp.trace.server": "verbose"
}
```

---

## Contributing

See [CONTRIBUTING.md](https://github.com/summer-rs/summer-lsp/blob/main/CONTRIBUTING.md) for details on how to contribute to this project.

## Links

- [GitHub Repository](https://github.com/summer-rs/summer-lsp)
- [Issue Tracker](https://github.com/summer-rs/summer-lsp/issues)
- [summer-rs Documentation](https://summer-rs.github.io/)
- [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=summer-rs.summer-lsp)
