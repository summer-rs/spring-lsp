import * as vscode from 'vscode';
import { SummerApp } from '../models';
import { LanguageClientManager } from '../languageClient';

/**
 * Plugin 接口（从语言服务器返回）
 * 
 * 表示一个 Summer RS 插件
 */
export interface Plugin {
  /**
   * 插件名称
   */
  name: string;

  /**
   * 插件类型名
   */
  typeName: string;

  /**
   * 配置前缀（可选）
   */
  configPrefix?: string;

  /**
   * 源代码位置
   */
  location: {
    uri: string;
    range: {
      start: { line: number; character: number };
      end: { line: number; character: number };
    };
  };
}

/**
 * Plugins 树视图数据提供者
 * 
 * 负责显示运行中应用的插件列表
 */
export class PluginsTreeDataProvider
  implements vscode.TreeDataProvider<PluginTreeItem>
{
  /**
   * 树数据变化事件发射器
   */
  private _onDidChangeTreeData = new vscode.EventEmitter<
    PluginTreeItem | undefined
  >();

  /**
   * 树数据变化事件
   */
  readonly onDidChangeTreeData: vscode.Event<PluginTreeItem | undefined> =
    this._onDidChangeTreeData.event;

  /**
   * 静态分析的插件列表
   */
  private staticPlugins: Plugin[] = [];

  /**
   * 运行时的插件列表
   */
  private runtimePlugins: Plugin[] = [];

  /**
   * 当前选中的应用
   */
  private currentApp: SummerApp | undefined;

  /**
   * 语言客户端管理器
   */
  private readonly clientManager: LanguageClientManager;

  /**
   * 创建 PluginsTreeDataProvider 实例
   * 
   * @param clientManager 语言客户端管理器
   */
  constructor(clientManager: LanguageClientManager) {
    this.clientManager = clientManager;

    // 监听文档保存（特别是 main.rs）
    vscode.workspace.onDidSaveTextDocument(doc => {
      if (doc.languageId === 'rust' && doc.fileName.endsWith('main.rs')) {
        this.refreshStatic();
      }
    });
  }

  /**
   * 刷新静态分析结果（基于工作空间）
   */
  public async refreshStatic(): Promise<void> {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders || workspaceFolders.length === 0) {
      this.staticPlugins = [];
      this._onDidChangeTreeData.fire(undefined);
      return;
    }

    const workspacePath = workspaceFolders[0].uri.fsPath;
    await this.refreshStaticByPath(workspacePath);
  }

  /**
   * 刷新静态分析结果（基于指定路径）
   */
  private async refreshStaticByPath(appPath: string): Promise<void> {
    try {
      const response = await this.clientManager.sendRequest<{
        plugins: Plugin[];
      }>('summer/plugins', {
        appPath: appPath,
      });

      this.staticPlugins = response?.plugins || [];
      console.log(`Loaded ${this.staticPlugins.length} plugins from static analysis (${appPath})`);
      this._onDidChangeTreeData.fire(undefined);
    } catch (error) {
      console.error('Failed to load static plugins:', error);
      this.staticPlugins = [];
      this._onDidChangeTreeData.fire(undefined);
    }
  }

  /**
   * 刷新插件列表（兼容旧接口）
   * 
   * @param app 要刷新的应用（可选）
   */
  public async refresh(app?: SummerApp): Promise<void> {
    if (!app) {
      this.clearRuntime();
      return;
    }

    this.currentApp = app;

    // 先刷新静态分析（基于应用路径）
    await this.refreshStaticByPath(app.path);

    // 如果应用在运行，再刷新运行时信息
    if (app.state === 'running') {
      await this.refreshRuntime(app);
    }
  }

  /**
   * 刷新运行时信息
   */
  private async refreshRuntime(app: SummerApp): Promise<void> {
    if (!app.port) {
      return;
    }

    try {
      const response = await fetch(`http://localhost:${app.port}/_debug/plugins`);
      if (response.ok) {
        const data = await response.json() as { plugins?: Plugin[] };
        this.runtimePlugins = data.plugins || [];
        console.log(`Loaded ${this.runtimePlugins.length} plugins from runtime`);
        this._onDidChangeTreeData.fire(undefined);
      }
    } catch (error) {
      console.warn('Failed to load runtime plugins:', error);
    }
  }

  /**
   * 清除运行时信息
   */
  private clearRuntime(): void {
    this.runtimePlugins = [];
    this.currentApp = undefined;
    this._onDidChangeTreeData.fire(undefined);
  }

  /**
   * 获取树节点
   * 
   * @param element 树节点元素
   * @returns 树节点
   */
  public getTreeItem(element: PluginTreeItem): vscode.TreeItem {
    return element;
  }

  /**
   * 获取子节点
   * 
   * @param element 父节点，如果为 undefined 表示根节点
   * @returns 子节点列表
   */
  public async getChildren(element?: PluginTreeItem): Promise<PluginTreeItem[]> {
    if (element) {
      return [];
    }

    // 优先使用运行时信息，否则使用静态分析结果
    const plugins = this.runtimePlugins.length > 0 ? this.runtimePlugins : this.staticPlugins;

    if (plugins.length === 0) {
      return [];
    }

    // 根节点：显示所有插件
    return plugins.map((plugin) => new PluginTreeItem(plugin));
  }
}

/**
 * 插件树节点
 */
export class PluginTreeItem extends vscode.TreeItem {
  /**
   * 插件实例
   */
  public readonly plugin: Plugin;

  /**
   * 创建插件树节点
   * 
   * @param plugin 插件实例
   */
  constructor(plugin: Plugin) {
    super(plugin.name, vscode.TreeItemCollapsibleState.None);

    this.plugin = plugin;

    // 设置上下文值（用于菜单显示）
    this.contextValue = 'summer:plugin';

    // 设置工具提示
    this.tooltip = this.buildTooltip();

    // 设置描述
    this.description = this.getDescription();

    // 设置图标
    this.iconPath = new vscode.ThemeIcon('extensions', new vscode.ThemeColor('charts.purple'));

    // 设置命令（点击时导航到定义）
    if (plugin.location && plugin.location.uri && plugin.location.range) {
      this.command = {
        command: 'summer.plugin.navigate',
        title: 'Go to Definition',
        arguments: [plugin.location]
      };
    }
  }

  /**
   * 构建工具提示
   */
  private buildTooltip(): vscode.MarkdownString {
    const tooltip = new vscode.MarkdownString();
    tooltip.isTrusted = true;

    tooltip.appendMarkdown(`### ${this.plugin.name}\n\n`);
    tooltip.appendMarkdown(`**Type:** ${this.plugin.typeName}\n\n`);

    if (this.plugin.configPrefix) {
      tooltip.appendMarkdown(`**Config Prefix:** \`${this.plugin.configPrefix}\`\n\n`);
    }

    tooltip.appendMarkdown(`**Location:** ${this.plugin.location.uri}\n\n`);

    return tooltip;
  }

  /**
   * 获取描述
   */
  private getDescription(): string {
    return this.plugin.typeName;
  }
}
