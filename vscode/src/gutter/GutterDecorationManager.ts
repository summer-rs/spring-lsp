import * as vscode from 'vscode';
import { GutterActionProvider } from './GutterActionProvider';

/**
 * Gutter 装饰管理器
 * 负责在编辑器行号旁显示图标，标识组件、路由和任务
 */
export class GutterDecorationManager {
  private componentClassDecorationType: vscode.TextEditorDecorationType;
  private componentFunctionDecorationType: vscode.TextEditorDecorationType;
  private configDecorationType: vscode.TextEditorDecorationType;
  private routeDecorationType: vscode.TextEditorDecorationType;
  private routeOpenapiDecorationType: vscode.TextEditorDecorationType;
  private cronDecorationType: vscode.TextEditorDecorationType;
  private disposables: vscode.Disposable[] = [];
  private enabled: boolean = false;
  private actionProvider: GutterActionProvider;
  private decorationMap: Map<string, { line: number; type: 'component' | 'config' | 'route' | 'job' }> = new Map();

  constructor(private context: vscode.ExtensionContext) {
    this.actionProvider = new GutterActionProvider(context);

    // 创建装饰类型（使用与视图相同的 SVG 图标）
    this.componentClassDecorationType = this.createDecorationType('component-class', 'symbolIcon.classForeground');
    this.componentFunctionDecorationType = this.createDecorationType('component-function', 'symbolIcon.methodForeground');
    this.configDecorationType = this.createDecorationType('config', 'symbolIcon.structForeground');
    this.routeDecorationType = this.createDecorationType('route', 'symbolIcon.methodForeground');
    this.routeOpenapiDecorationType = this.createDecorationType('route-openapi', 'charts.purple');
    this.cronDecorationType = this.createDecorationType('job', 'charts.blue');

    // 检查配置并初始化
    this.updateConfiguration();

    // 监听配置变化
    this.disposables.push(
      vscode.workspace.onDidChangeConfiguration((e) => {
        if (e.affectsConfiguration('summer-rs.enableGutter')) {
          this.updateConfiguration();
        }
      })
    );

    // 监听活动编辑器变化
    this.disposables.push(
      vscode.window.onDidChangeActiveTextEditor((editor) => {
        if (this.enabled && editor) {
          this.updateDecorations(editor);
        }
      })
    );

    // 监听文档变化
    this.disposables.push(
      vscode.workspace.onDidChangeTextDocument((event) => {
        const editor = vscode.window.activeTextEditor;
        if (this.enabled && editor && event.document === editor.document) {
          // 使用防抖避免频繁更新
          this.scheduleUpdate(editor);
        }
      })
    );

    // 初始化当前编辑器
    if (this.enabled && vscode.window.activeTextEditor) {
      this.updateDecorations(vscode.window.activeTextEditor);
    }
  }

  /**
   * 创建装饰类型
   * 使用 SVG 图标文件，与视图保持一致
   */
  private createDecorationType(
    iconName: string,
    _themeColor: string
  ): vscode.TextEditorDecorationType {
    // 使用 SVG 文件路径
    const iconPath = vscode.Uri.joinPath(
      this.context.extensionUri,
      'resources',
      'icons',
      `${iconName}.svg`
    );
    
    return vscode.window.createTextEditorDecorationType({
      gutterIconPath: iconPath,
      gutterIconSize: 'contain',
    });
  }

  /**
   * 更新配置
   */
  private updateConfiguration(): void {
    const config = vscode.workspace.getConfiguration('summer-rs');
    const gutterOption = config.get<string>('enableGutter', 'on');
    const wasEnabled = this.enabled;
    this.enabled = gutterOption === 'on';

    if (wasEnabled && !this.enabled) {
      // 禁用时清除所有装饰
      this.clearAllDecorations();
    } else if (!wasEnabled && this.enabled) {
      // 启用时更新当前编辑器
      const editor = vscode.window.activeTextEditor;
      if (editor) {
        this.updateDecorations(editor);
      }
    }
  }

  private updateTimeout: NodeJS.Timeout | undefined;

  /**
   * 调度更新（防抖）
   */
  private scheduleUpdate(editor: vscode.TextEditor): void {
    if (this.updateTimeout) {
      clearTimeout(this.updateTimeout);
    }
    this.updateTimeout = setTimeout(() => {
      this.updateDecorations(editor);
    }, 300);
  }

  /**
   * 更新装饰
   */
  private updateDecorations(editor: vscode.TextEditor): void {
    if (!this.enabled) {
      return;
    }

    // 只处理 Rust 文件
    if (editor.document.languageId !== 'rust') {
      return;
    }

    const text = editor.document.getText();
    const componentClassDecorations: vscode.DecorationOptions[] = [];
    const componentFunctionDecorations: vscode.DecorationOptions[] = [];
    const configDecorations: vscode.DecorationOptions[] = [];
    const routeDecorations: vscode.DecorationOptions[] = [];
    const routeOpenapiDecorations: vscode.DecorationOptions[] = [];
    const cronDecorations: vscode.DecorationOptions[] = [];

    // 分析代码并找到需要装饰的行
    this.analyzeCode(text, editor.document, componentClassDecorations, componentFunctionDecorations, configDecorations, routeDecorations, routeOpenapiDecorations, cronDecorations);

    // 应用装饰
    editor.setDecorations(this.componentClassDecorationType, componentClassDecorations);
    editor.setDecorations(this.componentFunctionDecorationType, componentFunctionDecorations);
    editor.setDecorations(this.configDecorationType, configDecorations);
    editor.setDecorations(this.routeDecorationType, routeDecorations);
    editor.setDecorations(this.routeOpenapiDecorationType, routeOpenapiDecorations);
    editor.setDecorations(this.cronDecorationType, cronDecorations);
  }

  /**
   * 分析代码
   */
  private analyzeCode(
    text: string,
    document: vscode.TextDocument,
    componentClassDecorations: vscode.DecorationOptions[],
    componentFunctionDecorations: vscode.DecorationOptions[],
    configDecorations: vscode.DecorationOptions[],
    routeDecorations: vscode.DecorationOptions[],
    routeOpenapiDecorations: vscode.DecorationOptions[],
    cronDecorations: vscode.DecorationOptions[]
  ): void {
    const lines = text.split('\n');
    let inComment = false;
    let inString = false;

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const trimmedLine = line.trim();

      // 跳过注释和字符串
      if (trimmedLine.startsWith('//')) {
        continue;
      }

      // 检查多行注释
      if (trimmedLine.includes('/*')) {
        inComment = true;
      }
      if (inComment) {
        if (trimmedLine.includes('*/')) {
          inComment = false;
        }
        continue;
      }

      // 检查 #[derive(Service)] - 使用 class 图标
      if (this.isServiceDerive(trimmedLine)) {
        const range = new vscode.Range(i, 0, i, line.length);
        const structName = this.findStructName(lines, i);
        componentClassDecorations.push({
          range,
          hoverMessage: new vscode.MarkdownString(
            `**Summer Component**\n\n${structName ? `Struct: \`${structName}\`` : 'This struct is registered as a component'}\n\n🔵 _Service derive macro_\n\nClick to see quick actions`
          ),
        });
      }

      // 检查 #[component] 宏 - 使用 function 图标
      if (this.isComponentMacro(trimmedLine)) {
        const range = new vscode.Range(i, 0, i, line.length);
        const functionName = this.findFunctionName(lines, i);
        componentFunctionDecorations.push({
          range,
          hoverMessage: new vscode.MarkdownString(
            `**Summer Component**\n\n${functionName ? `Function: \`${functionName}\`` : 'This function is registered as a component'}\n\n🟣 _Component function macro_\n\nClick to see quick actions`
          ),
        });
      }

      // 检查 #[derive(Configurable)] - 使用专用的 config 装饰
      if (this.isConfigurableDerive(trimmedLine)) {
        const range = new vscode.Range(i, 0, i, line.length);
        const structName = this.findStructName(lines, i);
        const configPrefix = this.findConfigPrefix(lines, i);
        configDecorations.push({
          range,
          hoverMessage: new vscode.MarkdownString(
            `**Configuration Struct**\n\n${structName ? `Struct: \`${structName}\`` : 'Configuration structure'}${configPrefix ? `\n\nPrefix: \`[${configPrefix}]\`` : ''}\n\nClick to see quick actions`
          ),
        });
      }

      // 检查路由宏
      if (this.isRouteMacro(trimmedLine)) {
        const range = new vscode.Range(i, 0, i, line.length);
        const method = this.extractRouteMethod(trimmedLine);
        const path = this.extractRoutePath(trimmedLine);
        const functionName = this.findFunctionName(lines, i);
        const isOpenapi = this.isOpenapiRouteMacro(trimmedLine);
        
        const decoration = {
          range,
          hoverMessage: new vscode.MarkdownString(
            `**HTTP Route${isOpenapi ? ' (OpenAPI)' : ''}**\n\n\`${method} ${path}\`\n\n${functionName ? `Handler: \`${functionName}\`` : ''}${isOpenapi ? '\n\n📖 *OpenAPI documented route*' : ''}\n\nClick to see quick actions`
          ),
        };
        
        if (isOpenapi) {
          routeOpenapiDecorations.push(decoration);
        } else {
          routeDecorations.push(decoration);
        }
      }

      // 检查 #[cron] 或 #[fix_delay] 或 #[fix_rate]
      if (this.isCronMacro(trimmedLine) || this.isFixDelayMacro(trimmedLine) || this.isFixRateMacro(trimmedLine)) {
        const range = new vscode.Range(i, 0, i, line.length);
        const schedule = this.extractScheduleInfo(trimmedLine);
        const functionName = this.findFunctionName(lines, i);
        cronDecorations.push({
          range,
          hoverMessage: new vscode.MarkdownString(
            `**Scheduled Job**\n\n${schedule}\n\n${functionName ? `Function: \`${functionName}\`` : ''}\n\nClick to see quick actions`
          ),
        });
      }
    }
  }

  /**
   * 检查是否是 Service derive
   */
  private isServiceDerive(line: string): boolean {
    return /^#\[derive\([^)]*Service[^)]*\)\]/.test(line);
  }

  /**
   * 检查是否是 component 宏
   */
  private isComponentMacro(line: string): boolean {
    return /^#\[component(?:\(|$)/.test(line);
  }

  /**
   * 检查是否是 Configurable derive - 新增
   */
  private isConfigurableDerive(line: string): boolean {
    return /^#\[derive\([^)]*Configurable[^)]*\)\]/.test(line);
  }

  /**
   * 查找配置前缀 - 新增
   */
  private findConfigPrefix(lines: string[], startLine: number): string | null {
    // 向上查找 #[config_prefix = "..."]
    for (let i = startLine - 1; i >= Math.max(0, startLine - 5); i--) {
      const line = lines[i].trim();
      const match = line.match(/^#\[config_prefix\s*=\s*"([^"]+)"\]/);
      if (match) {
        return match[1];
      }
    }
    // 向下查找
    for (let i = startLine + 1; i < Math.min(startLine + 5, lines.length); i++) {
      const line = lines[i].trim();
      const match = line.match(/^#\[config_prefix\s*=\s*"([^"]+)"\]/);
      if (match) {
        return match[1];
      }
    }
    return null;
  }

  /**
   * 检查是否是路由宏（包括普通路由和 OpenAPI 路由）
   */
  private isRouteMacro(line: string): boolean {
    return /^#\[(get|post|put|delete|patch|route|get_api|post_api|put_api|delete_api|patch_api)\(/.test(line);
  }

  /**
   * 检查是否是 OpenAPI 路由宏
   */
  private isOpenapiRouteMacro(line: string): boolean {
    return /^#\[(get_api|post_api|put_api|delete_api|patch_api)\(/.test(line);
  }

  /**
   * 检查是否是 cron 宏
   */
  private isCronMacro(line: string): boolean {
    return /^#\[cron\(/.test(line);
  }

  /**
   * 检查是否是 fix_delay 宏
   */
  private isFixDelayMacro(line: string): boolean {
    return /^#\[fix_delay\(/.test(line);
  }

  /**
   * 检查是否是 fix_rate 宏
   */
  private isFixRateMacro(line: string): boolean {
    return /^#\[fix_rate\(/.test(line);
  }

  /**
   * 提取路由方法
   */
  private extractRouteMethod(line: string): string {
    const match = line.match(/^#\[(get|post|put|delete|patch|route|get_api|post_api|put_api|delete_api|patch_api)\(/);
    if (match) {
      // 移除 _api 后缀
      return match[1].replace('_api', '').toUpperCase();
    }
    return 'UNKNOWN';
  }

  /**
   * 提取路由路径
   */
  private extractRoutePath(line: string): string {
    const match = line.match(/^#\[(?:get|post|put|delete|patch|route|get_api|post_api|put_api|delete_api|patch_api)\("([^"]+)"/);
    if (match) {
      return match[1];
    }
    return '/';
  }

  /**
   * 提取调度信息
   */
  private extractScheduleInfo(line: string): string {
    // cron
    let match = line.match(/^#\[cron\("([^"]+)"/);
    if (match) {
      return `Cron: \`${match[1]}\``;
    }

    // fix_delay
    match = line.match(/^#\[fix_delay\((\d+)\)/);
    if (match) {
      return `Fixed Delay: ${match[1]} seconds`;
    }

    // fix_rate
    match = line.match(/^#\[fix_rate\((\d+)\)/);
    if (match) {
      return `Fixed Rate: ${match[1]} seconds`;
    }

    return 'Schedule: unknown';
  }

  /**
   * 查找结构体名称
   */
  private findStructName(lines: string[], startLine: number): string | null {
    // 向下查找最近的 struct 定义
    for (let i = startLine + 1; i < Math.min(startLine + 5, lines.length); i++) {
      const line = lines[i].trim();
      const match = line.match(/^(?:pub\s+)?struct\s+(\w+)/);
      if (match) {
        return match[1];
      }
    }
    return null;
  }

  /**
   * 查找函数名称
   */
  private findFunctionName(lines: string[], startLine: number): string | null {
    // 向下查找最近的函数定义
    for (let i = startLine + 1; i < Math.min(startLine + 5, lines.length); i++) {
      const line = lines[i].trim();
      const match = line.match(/^(?:pub\s+)?(?:async\s+)?fn\s+(\w+)/);
      if (match) {
        return match[1];
      }
    }
    return null;
  }

  /**
   * 清除所有装饰
   */
  private clearAllDecorations(): void {
    const editor = vscode.window.activeTextEditor;
    if (editor) {
      editor.setDecorations(this.componentClassDecorationType, []);
      editor.setDecorations(this.componentFunctionDecorationType, []);
      editor.setDecorations(this.configDecorationType, []);
      editor.setDecorations(this.routeDecorationType, []);
      editor.setDecorations(this.routeOpenapiDecorationType, []);
      editor.setDecorations(this.cronDecorationType, []);
    }
  }

  /**
   * 释放资源
   */
  public dispose(): void {
    this.componentClassDecorationType.dispose();
    this.componentFunctionDecorationType.dispose();
    this.configDecorationType.dispose();
    this.routeDecorationType.dispose();
    this.routeOpenapiDecorationType.dispose();
    this.cronDecorationType.dispose();
    this.disposables.forEach((d) => d.dispose());
    if (this.updateTimeout) {
      clearTimeout(this.updateTimeout);
    }
  }

  /**
   * 处理 Gutter 点击
   * 注意：VSCode 不直接支持 gutter 点击事件，
   * 我们通过命令和快捷键来模拟这个功能
   */
  public async handleGutterClick(editor: vscode.TextEditor, line: number): Promise<void> {
    if (!this.enabled) {
      return;
    }

    const document = editor.document;
    const lineText = document.lineAt(line).text.trim();

    // 检查这一行是什么类型的装饰
    if (this.isServiceDerive(lineText) || this.isComponentMacro(lineText)) {
      await this.actionProvider.showComponentActions(document, line);
    } else if (this.isConfigurableDerive(lineText)) {
      // 新增：处理配置结构的点击
      await this.actionProvider.showConfigurationActions(document, line);
    } else if (this.isRouteMacro(lineText)) {
      await this.actionProvider.showRouteActions(document, line);
    } else if (this.isCronMacro(lineText) || this.isFixDelayMacro(lineText) || this.isFixRateMacro(lineText)) {
      await this.actionProvider.showJobActions(document, line);
    }
  }

  /**
   * 注册命令
   */
  public registerCommands(): void {
    // 注册快速操作命令
    this.disposables.push(
      vscode.commands.registerCommand('summer-rs.gutter.showActions', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          return;
        }

        const line = editor.selection.active.line;
        await this.handleGutterClick(editor, line);
      })
    );
  }
}
