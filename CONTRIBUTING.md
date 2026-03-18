# 贡献指南

感谢您对 summer-lsp 项目的关注！我们欢迎各种形式的贡献。

## 如何贡献

### 报告 Bug

如果您发现了 bug，请在 GitHub Issues 中创建一个新的 issue，并包含以下信息：

- 清晰的标题和描述
- 重现步骤
- 预期行为和实际行为
- 您的环境信息（操作系统、Rust 版本、编辑器等）
- 相关的日志输出

### 提出功能请求

如果您有新功能的想法，请创建一个 issue 并描述：

- 功能的用途和价值
- 预期的行为
- 可能的实现方案

### 提交代码

1. **Fork 项目**并创建您的分支：
   ```bash
   git checkout -b feature/my-new-feature
   ```

2. **编写代码**并遵循项目的代码风格：
   - 使用 `cargo fmt` 格式化代码
   - 使用 `cargo clippy` 检查代码质量
   - 为新功能编写测试
   - 更新相关文档

3. **运行测试**确保所有测试通过：
   ```bash
   cargo test
   ```

4. **提交更改**：
   ```bash
   git commit -m "Add some feature"
   ```

5. **推送到您的 Fork**：
   ```bash
   git push origin feature/my-new-feature
   ```

6. **创建 Pull Request**

## 代码风格

- 遵循 Rust 官方代码风格指南
- 使用 `cargo fmt` 格式化代码
- 使用有意义的变量名和函数名
- 为公共 API 添加文档注释
- 保持函数简短和专注

## 测试

- 为新功能编写单元测试
- 为通用属性编写属性测试
- 确保测试覆盖率达到 80% 以上
- 测试应该快速且可靠

## 提交消息

提交消息应该清晰地描述更改：

- 使用现在时态（"Add feature" 而不是 "Added feature"）
- 第一行不超过 50 个字符
- 如果需要，添加详细描述
- 引用相关的 issue 编号

示例：
```
Add TOML completion support

- Implement configuration prefix completion
- Add property completion within sections
- Support enum value completion

Fixes #123
```

## 文档

- 为新功能更新 README.md
- 为公共 API 添加文档注释
- 更新设计文档（如果架构有变化）
- 添加使用示例

## 行为准则

请遵守我们的行为准则，尊重所有贡献者。

## 许可证

通过贡献代码，您同意您的贡献将在 MIT 或 Apache-2.0 双重许可下发布。

## 问题？

如果您有任何问题，请随时在 issue 中提问或联系维护者。

感谢您的贡献！
