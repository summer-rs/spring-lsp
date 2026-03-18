# VSCode 扩展调试指南

本指南详细说明如何调试 Summer LSP 的 VSCode 扩展。

## 目录

- [前置要求](#前置要求)
- [快速开始](#快速开始)
- [调试配置说明](#调试配置说明)
- [调试步骤](#调试步骤)
- [常见调试场景](#常见调试场景)
- [调试技巧](#调试技巧)
- [常见问题](#常见问题)

## 前置要求

### 必需工具

1. **Node.js** (v18+)
   ```bash
   node --version  # 应该 >= 18.0.0
   ```

2. **npm** (v9+)
   ```bash
   npm --version  # 应该 >= 9.0.0
   ```

3. **VSCode** (v1.75+)
   ```bash
   code --version  # 应该 >= 1.75.0
   ```

4. **TypeScript** (v5.0+)
   ```bash
   npx tsc --version  # 应该 >= 5.0.0
   ```

### 可选工具

- **Rust** (v1.70+) - 如果需要同时调试语言服务器
- **CodeLLDB** 扩展 - 用于调试 Rust 代码

## 快速开始

### 1. 安装依赖

```bash
cd summer-lsp/vscode
npm install
```

### 2. 编译 TypeScript

```bash
# 一次性编译
npm run compile

# 或者启动监听模式（推荐）
npm run watch
```

### 3. 启动调试

在 VSCode 中：
1. 打开 `summer-lsp/vscode` 目录
2. 按 `F5` 或点击 "Run and Debug" 面板的绿色播放按钮
3. 选择 "Run Extension" 配置
4. 新的 VSCode 窗口（Extension Development Host）会打开

### 4. 测试扩展

在新打开的窗口中：
1. 打开一个 summer-rs 项目
2. 查看 "Summer RS" 侧边栏
3. 测试各种功能（应用管理、配置补全等）

## 调试配置说明

### 配置 1: Run Extension

**用途**: 调试扩展的基本功能

```json
{
  "name": "Run Extension",
  "type": "extensionHost",
  "request": "launch",
  "args": [
    "--extensionDevelopmentPath=${workspaceFolder}"
  ],
  "outFiles": [
    "${workspaceFolder}/out/**/*.js"
  ],
  "preLaunchTask": "${defaultBuildTask}"
}
```

**特点**:
- 自动编译 TypeScript（通过 `preLaunchTask`）
- 在空白工作空间中启动
- 适合测试扩展的基本功能

**使用场景**:
- 测试扩展激活逻辑
- 测试命令注册
- 测试视图创建

### 配置 2: Run Extension (with test project)

**用途**: 在特定项目中调试扩展

```json
{
  "name": "Run Extension (with test project)",
  "type": "extensionHost",
  "request": "launch",
  "args": [
    "--extensionDevelopmentPath=${workspaceFolder}",
    "/Users/holmofy/rust/autowds/autowds-backend"  // 测试项目路径
  ],
  "outFiles": [
    "${workspaceFolder}/out/**/*.js"
  ],
  "preLaunchTask": "${defaultBuildTask}"
}
```

**特点**:
- 自动打开指定的测试项目
- 适合测试扩展在真实项目中的行为

**使用场景**:
- 测试应用检测逻辑
- 测试配置文件解析
- 测试路由和组件分析

**配置方法**:
1. 修改 `args` 数组中的路径为你的测试项目路径
2. 或者创建多个配置，每个对应不同的测试项目

### 配置 3: Extension Tests

**用途**: 运行扩展的自动化测试

```json
{
  "name": "Extension Tests",
  "type": "extensionHost",
  "request": "launch",
  "args": [
    "--extensionDevelopmentPath=${workspaceFolder}",
    "--extensionTestsPath=${workspaceFolder}/out/test/suite/index"
  ],
  "outFiles": [
    "${workspaceFolder}/out/test/**/*.js"
  ],
  "preLaunchTask": "${defaultBuildTask}"
}
```

**特点**:
- 运行 `test/suite/` 目录下的所有测试
- 自动编译测试代码

**使用场景**:
- 运行单元测试
- 运行集成测试
- 调试测试失败

## 调试步骤

### 方法 1: 使用 F5 快捷键（推荐）

1. **打开扩展项目**
   ```bash
   cd summer-lsp/vscode
   code .
   ```

2. **启动监听模式**（可选但推荐）
   ```bash
   npm run watch
   ```
   这样修改代码后会自动重新编译。

3. **按 F5 启动调试**
   - VSCode 会自动运行 `preLaunchTask`（编译 TypeScript）
   - 新窗口会打开（Extension Development Host）

4. **设置断点**
   - 在 `src/extension.ts` 或其他文件中点击行号左侧设置断点
   - 红点表示断点已设置

5. **触发断点**
   - 在新窗口中执行相关操作
   - 调试器会在断点处暂停

6. **查看变量和调用栈**
   - 左侧面板显示变量、监视、调用栈等信息
   - 可以在 "Debug Console" 中执行表达式

### 方法 2: 使用调试面板

1. **打开调试面板**
   - 点击左侧的 "Run and Debug" 图标
   - 或按 `Ctrl+Shift+D` (Windows/Linux) / `Cmd+Shift+D` (macOS)

2. **选择调试配置**
   - 在顶部下拉菜单中选择配置（如 "Run Extension"）

3. **点击绿色播放按钮**
   - 或按 `F5`

4. **开始调试**
   - 按照方法 1 的步骤 4-6 继续

### 方法 3: 使用命令面板

1. **打开命令面板**
   - 按 `Ctrl+Shift+P` (Windows/Linux) / `Cmd+Shift+P` (macOS)

2. **输入并选择**
   ```
   Debug: Select and Start Debugging
   ```

3. **选择配置**
   - 选择 "Run Extension" 或其他配置

4. **开始调试**

## 常见调试场景

### 场景 1: 调试扩展激活

**目标**: 确保扩展在正确的时机激活

**步骤**:
1. 在 `src/extension.ts` 的 `activate()` 函数开头设置断点
   ```typescript
   export async function activate(context: vscode.ExtensionContext): Promise<void> {
     console.log('Summer LSP extension is now activating...');  // 在这里设置断点
     // ...
   }
   ```

2. 按 `F5` 启动调试

3. 在新窗口中打开一个 summer-rs 项目

4. 断点应该被触发

**检查点**:
- `context` 对象是否正确
- `activationEvents` 是否按预期触发
- 扩展是否在正确的时机激活

### 场景 2: 调试应用检测

**目标**: 确保扩展能正确检测 summer-rs 应用

**步骤**:
1. 在 `src/controllers/LocalAppManager.ts` 的 `scanWorkspace()` 方法设置断点
   ```typescript
   private async scanWorkspace() {
     const cargoFiles = await vscode.workspace.findFiles(  // 在这里设置断点
       '**/Cargo.toml',
       '**/target/**'
     );
     // ...
   }
   ```

2. 使用 "Run Extension (with test project)" 配置启动调试

3. 观察 `cargoFiles` 变量的值

4. 单步执行，查看应用检测逻辑

**检查点**:
- `Cargo.toml` 文件是否被正确找到
- 依赖解析是否正确
- 应用是否被正确识别为 summer-rs 应用

### 场景 3: 调试命令执行

**目标**: 调试应用启动/停止等命令

**步骤**:
1. 在 `src/controllers/LocalAppController.ts` 的 `runApp()` 方法设置断点
   ```typescript
   public async runApp(app: SummerApp, debug: boolean = false, profile?: string) {
     if (app.state !== AppState.INACTIVE) {  // 在这里设置断点
       // ...
     }
     // ...
   }
   ```

2. 启动调试

3. 在新窗口的 "Summer RS" 侧边栏中点击应用的 "Run" 按钮

4. 断点被触发

**检查点**:
- `app` 对象的状态是否正确
- 调试配置是否正确生成
- 启动命令是否正确执行

### 场景 4: 调试 LSP 通信

**目标**: 调试扩展与语言服务器的通信

**步骤**:
1. 在 `src/languageClient/LanguageClientManager.ts` 设置断点
   ```typescript
   public async sendRequest<T>(method: string, params: any): Promise<T | undefined> {
     if (!this.client) {  // 在这里设置断点
       return undefined;
     }
     // ...
   }
   ```

2. 启动调试

3. 触发需要 LSP 通信的操作（如刷新组件视图）

4. 观察请求和响应

**检查点**:
- LSP 客户端是否已启动
- 请求参数是否正确
- 响应数据是否符合预期

**额外技巧**:
- 在 `settings.json` 中启用 LSP 追踪：
  ```json
  {
    "summer-rs.trace.server": "verbose"
  }
  ```
- 查看 "Output" 面板的 "Summer LSP (Language Server)" 通道

### 场景 5: 调试视图刷新

**目标**: 确保视图在正确的时机刷新

**步骤**:
1. 在 `src/views/ComponentsTreeDataProvider.ts` 设置断点
   ```typescript
   public async refresh(app: SummerApp): Promise<void> {
     console.log(`Refreshing components for app: ${app.name}`);  // 在这里设置断点
     // ...
   }
   ```

2. 启动调试

3. 在新窗口中选择一个应用

4. 观察刷新逻辑

**检查点**:
- `app` 参数是否正确
- LSP 请求是否成功
- 树视图是否正确更新

### 场景 6: 调试配置文件解析

**目标**: 调试 TOML 配置文件的解析

**步骤**:
1. 在 `src/controllers/LocalAppManager.ts` 的 `parseCargoToml()` 方法设置断点
   ```typescript
   private async parseCargoToml(file: vscode.Uri): Promise<SummerApp | null> {
     try {
       const content = await vscode.workspace.fs.readFile(file);  // 在这里设置断点
       // ...
     }
   }
   ```

2. 启动调试

3. 打开或修改 `Cargo.toml` 文件

4. 观察解析过程

**检查点**:
- 文件内容是否正确读取
- TOML 解析是否成功
- 依赖提取是否正确

## 调试技巧

### 1. 使用 Console 输出

在代码中添加 `console.log()` 语句：

```typescript
console.log('App detected:', app);
console.log('Components:', components);
console.error('Failed to start app:', error);
```

查看输出：
- **开发窗口**: 按 `Ctrl+Shift+I` 打开开发者工具
- **扩展宿主窗口**: 在 "Help" -> "Toggle Developer Tools"

### 2. 使用 Output Channel

创建输出通道：

```typescript
const outputChannel = vscode.window.createOutputChannel('Summer LSP');
outputChannel.appendLine('Extension activated');
outputChannel.show();
```

查看输出：
- "View" -> "Output"
- 选择 "Summer LSP" 通道

### 3. 使用条件断点

右键点击断点，选择 "Edit Breakpoint"：

```typescript
// 只在特定条件下暂停
app.name === 'my-app'

// 只在特定次数后暂停
hitCount > 5
```

### 4. 使用日志点（Logpoint）

右键点击行号，选择 "Add Logpoint"：

```typescript
// 不暂停执行，只输出日志
App name: {app.name}, state: {app.state}
```

### 5. 使用监视表达式

在 "Watch" 面板添加表达式：

```typescript
app.state
app.dependencies.length
this.apps.size
```

### 6. 使用调用栈

在 "Call Stack" 面板：
- 查看函数调用链
- 点击栈帧跳转到对应代码
- 查看每个栈帧的变量

### 7. 使用 Debug Console

在 "Debug Console" 中执行表达式：

```typescript
// 查看变量
app

// 调用方法
app.reset()

// 执行复杂表达式
this.apps.values().filter(a => a.state === 'running')
```

### 8. 热重载（Hot Reload）

修改代码后：
1. 保存文件（`Ctrl+S`）
2. 在调试工具栏点击 "Restart" 按钮（绿色圆形箭头）
3. 或按 `Ctrl+Shift+F5`

**注意**: 需要先启动 `npm run watch` 才能自动编译。

### 9. 使用 Source Maps

确保 `tsconfig.json` 中启用了 source maps：

```json
{
  "compilerOptions": {
    "sourceMap": true
  }
}
```

这样可以在 TypeScript 源码中设置断点，而不是编译后的 JavaScript。

### 10. 调试多个进程

如果需要同时调试扩展和语言服务器：

1. 启动扩展调试（F5）
2. 在扩展宿主窗口中，打开 summer-rs 项目
3. 在主 VSCode 窗口中，切换到 Rust 项目目录
4. 使用 "Attach to LSP Server" 配置附加到语言服务器进程

## 常见问题

### Q1: 按 F5 后没有反应

**可能原因**:
- TypeScript 编译失败
- 没有选择正确的调试配置

**解决方法**:
1. 检查 "Problems" 面板是否有编译错误
2. 手动运行 `npm run compile` 查看错误
3. 确保在 "Run and Debug" 面板选择了正确的配置

### Q2: 断点显示灰色（未绑定）

**可能原因**:
- Source maps 未正确生成
- 文件路径不匹配

**解决方法**:
1. 确保 `tsconfig.json` 中 `sourceMap: true`
2. 重新编译：`npm run compile`
3. 重启调试会话

### Q3: 扩展未激活

**可能原因**:
- `activationEvents` 配置不正确
- 工作空间不满足激活条件

**解决方法**:
1. 检查 `package.json` 中的 `activationEvents`
2. 确保测试项目包含 `Cargo.toml` 或 `.summer-lsp.toml`
3. 手动触发激活：在命令面板执行扩展的命令

### Q4: 语言服务器未启动

**可能原因**:
- 服务器路径配置不正确
- 服务器二进制文件不存在

**解决方法**:
1. 检查 `summer-rs.serverPath` 配置
2. 确保语言服务器已编译：
   ```bash
   cd summer-lsp
   cargo build --release
   ```
3. 查看 "Output" 面板的错误信息

### Q5: 修改代码后没有生效

**可能原因**:
- 没有重新编译
- 没有重启调试会话

**解决方法**:
1. 确保 `npm run watch` 正在运行
2. 或手动运行 `npm run compile`
3. 重启调试会话（`Ctrl+Shift+F5`）

### Q6: 无法查看变量值

**可能原因**:
- 变量被优化掉了
- 在错误的作用域中

**解决方法**:
1. 在 `tsconfig.json` 中禁用优化（开发时）：
   ```json
   {
     "compilerOptions": {
       "target": "ES2020",
       "module": "commonjs"
     }
   }
   ```
2. 使用 "Debug Console" 手动查询变量

### Q7: 测试失败

**可能原因**:
- 测试环境配置不正确
- 测试代码有 bug

**解决方法**:
1. 使用 "Extension Tests" 配置调试测试
2. 在测试代码中设置断点
3. 查看测试输出和错误信息

## 调试工作流

### 日常开发流程

1. **启动监听模式**
   ```bash
   npm run watch
   ```

2. **启动调试**
   - 按 `F5`

3. **修改代码**
   - 编辑 TypeScript 文件
   - 保存（自动编译）

4. **重启调试**
   - 按 `Ctrl+Shift+F5`
   - 或点击调试工具栏的重启按钮

5. **测试功能**
   - 在扩展宿主窗口中测试

6. **查看日志**
   - 检查 "Output" 面板
   - 检查 "Debug Console"

### 修复 Bug 流程

1. **重现问题**
   - 在扩展宿主窗口中重现 bug

2. **定位代码**
   - 根据错误信息找到相关代码
   - 或使用 "Go to Symbol" (`Ctrl+Shift+O`)

3. **设置断点**
   - 在可能出问题的地方设置断点

4. **重新触发**
   - 重启调试并重现问题

5. **分析原因**
   - 查看变量值
   - 单步执行代码
   - 检查调用栈

6. **修复代码**
   - 修改代码
   - 保存并重启调试

7. **验证修复**
   - 确认问题已解决
   - 运行相关测试

### 添加新功能流程

1. **编写代码**
   - 实现新功能

2. **添加日志**
   - 在关键位置添加 `console.log()`

3. **测试功能**
   - 启动调试并测试

4. **调试问题**
   - 如果有问题，设置断点调试

5. **编写测试**
   - 在 `test/suite/` 中添加测试

6. **运行测试**
   - 使用 "Extension Tests" 配置

7. **清理日志**
   - 移除或注释掉调试日志

## 性能分析

### 使用 Chrome DevTools

1. 在扩展宿主窗口中打开开发者工具：
   - "Help" -> "Toggle Developer Tools"

2. 切换到 "Performance" 标签

3. 点击 "Record" 按钮

4. 执行要分析的操作

5. 停止录制并分析结果

### 使用 VSCode 性能分析器

1. 打开命令面板（`Ctrl+Shift+P`）

2. 执行：
   ```
   Developer: Show Running Extensions
   ```

3. 查看扩展的性能指标：
   - 激活时间
   - CPU 使用率
   - 内存使用量

## 相关资源

- [VSCode Extension API](https://code.visualstudio.com/api)
- [VSCode Extension Samples](https://github.com/microsoft/vscode-extension-samples)
- [Debugging Extensions](https://code.visualstudio.com/api/working-with-extensions/testing-extension)
- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)

## 总结

调试 VSCode 扩展的关键点：

1. ✅ 使用 `npm run watch` 自动编译
2. ✅ 使用 F5 快速启动调试
3. ✅ 善用断点和日志
4. ✅ 使用 Debug Console 查询变量
5. ✅ 使用 Output Channel 输出日志
6. ✅ 使用 Chrome DevTools 分析性能
7. ✅ 编写测试并使用测试配置调试

祝调试顺利！🎉
