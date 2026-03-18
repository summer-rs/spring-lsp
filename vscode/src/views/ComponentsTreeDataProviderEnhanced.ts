import * as vscode from 'vscode';
import * as path from 'path';
import { SummerApp } from '../models';
import { LanguageClientManager } from '../languageClient';
import { Component, ComponentsResponse, ComponentSource, DataSource } from '../types';
import { ViewMode, VIEW_MODE_KEYS } from '../types/viewMode';
import { FileTreeNode } from './BaseTreeDataProvider';

/**
 * 增强版 Components 树视图数据提供者
 * 
 * 支持两种视图模式：
 * - List 模式：扁平列表，直接显示所有组件
 * - Tree 模式：按文件组织，显示文件树结构
 */
export class ComponentsTreeDataProviderEnhanced
  implements vscode.TreeDataProvider<TreeNode>
{
  private _onDidChangeTreeData = new vscode.EventEmitter<TreeNode | undefined>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  private staticComponents: Map<string, Component> = new Map();
  private runtimeComponents: Map<string, Component> = new Map();
  private currentApp: SummerApp | undefined;
  private viewMode: ViewMode = ViewMode.List;

  constructor(
    private readonly clientManager: LanguageClientManager,
    private readonly context: vscode.ExtensionContext
  ) {
    // 读取配置的视图模式
    this.loadViewMode();

    // 监听配置变化
    vscode.workspace.onDidChangeConfiguration(e => {
      if (e.affectsConfiguration(VIEW_MODE_KEYS.components)) {
        this.loadViewMode();
        this._onDidChangeTreeData.fire(undefined);
      }
    });

    // 监听文档保存
    vscode.workspace.onDidSaveTextDocument(doc => {
      if (doc.languageId === 'rust') {
        this.refreshStatic();
      }
    });

    // 监听工作空间变化
    vscode.workspace.onDidChangeWorkspaceFolders(() => {
      this.refreshStatic();
    });

    // 初始加载
    this.refreshStatic();
  }

  /**
   * 加载视图模式配置
   */
  private loadViewMode(): void {
    const config = vscode.workspace.getConfiguration();
    const mode = config.get<string>(VIEW_MODE_KEYS.components, ViewMode.List);
    this.viewMode = mode as ViewMode;
    console.log(`[ComponentsTreeDataProvider] View mode: ${this.viewMode}`);
  }

  /**
   * 切换视图模式（通过快速选择）
   */
  public async selectViewMode(): Promise<void> {
    const items: vscode.QuickPickItem[] = [
      {
        label: '$(list-flat) List',
        description: 'Flat list view',
        detail: 'Show all components in a flat list',
        picked: this.viewMode === ViewMode.List
      },
      {
        label: '$(list-tree) Tree',
        description: 'Group by file',
        detail: 'Organize components by file structure',
        picked: this.viewMode === ViewMode.Tree
      }
    ];

    const selected = await vscode.window.showQuickPick(items, {
      placeHolder: 'Select view mode for Components',
      title: 'Components View Mode'
    });

    if (!selected) {
      return;
    }

    const newMode = selected.label.includes('List') ? ViewMode.List : ViewMode.Tree;
    
    if (newMode !== this.viewMode) {
      await vscode.workspace.getConfiguration().update(
        VIEW_MODE_KEYS.components,
        newMode,
        vscode.ConfigurationTarget.Workspace
      );
      this.viewMode = newMode;
      this._onDidChangeTreeData.fire(undefined);
      
      vscode.window.showInformationMessage(
        `Components view: ${newMode === ViewMode.List ? 'List' : 'Tree'} mode`
      );
    }
  }

  /**
   * 切换视图模式（快速切换，用于快捷键）
   */
  public async toggleViewMode(): Promise<void> {
    const newMode = this.viewMode === ViewMode.List ? ViewMode.Tree : ViewMode.List;
    await vscode.workspace.getConfiguration().update(
      VIEW_MODE_KEYS.components,
      newMode,
      vscode.ConfigurationTarget.Workspace
    );
    this.viewMode = newMode;
    this._onDidChangeTreeData.fire(undefined);
    
    vscode.window.showInformationMessage(
      `Components view: ${newMode === ViewMode.List ? 'List' : 'Tree'} mode`
    );
  }

  /**
   * 刷新静态分析结果
   */
  public async refreshStatic(): Promise<void> {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders || workspaceFolders.length === 0) {
      this.staticComponents.clear();
      this._onDidChangeTreeData.fire(undefined);
      return;
    }

    const workspacePath = workspaceFolders[0].uri.fsPath;
    await this.refreshStaticByPath(workspacePath);
  }

  /**
   * 刷新指定路径的静态分析结果
   */
  private async refreshStaticByPath(appPath: string): Promise<void> {
    console.log(`[ComponentsTreeDataProvider] Refreshing for: ${appPath}`);
    
    try {
      const response = await this.clientManager.sendRequest<ComponentsResponse>(
        'summer/components',
        { appPath }
      );

      this.staticComponents.clear();
      if (response && response.components) {
        response.components.forEach((component) => {
          this.staticComponents.set(component.name, component);
        });
        console.log(`[ComponentsTreeDataProvider] Loaded ${this.staticComponents.size} components`);
      }
      
      this._onDidChangeTreeData.fire(undefined);
    } catch (error) {
      console.error('Failed to load components:', error);
      this.staticComponents.clear();
      this._onDidChangeTreeData.fire(undefined);
    }
  }

  /**
   * 刷新（兼容接口）
   */
  public async refresh(app?: SummerApp): Promise<void> {
    if (!app) {
      this.clearRuntime();
      return;
    }

    this.currentApp = app;
    await this.refreshStaticByPath(app.path);

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
      const response = await fetch(`http://localhost:${app.port}/_debug/components`);
      if (response.ok) {
        const data = await response.json() as { components?: Component[] };
        this.runtimeComponents.clear();
        if (data.components) {
          data.components.forEach((component: Component) => {
            this.runtimeComponents.set(component.name, component);
          });
        }
        this._onDidChangeTreeData.fire(undefined);
      }
    } catch (error) {
      console.warn('Failed to load runtime components:', error);
    }
  }

  /**
   * 清除运行时信息
   */
  private clearRuntime(): void {
    this.runtimeComponents.clear();
    this.currentApp = undefined;
    this._onDidChangeTreeData.fire(undefined);
  }

  /**
   * 获取树节点
   */
  public getTreeItem(element: TreeNode): vscode.TreeItem {
    return element;
  }

  /**
   * 获取子节点
   */
  public async getChildren(element?: TreeNode): Promise<TreeNode[]> {
    const components = this.runtimeComponents.size > 0
      ? this.runtimeComponents
      : this.staticComponents;

    const source = this.runtimeComponents.size > 0
      ? DataSource.Runtime
      : DataSource.Static;

    if (components.size === 0) {
      return [];
    }

    // 根节点
    if (!element) {
      if (this.viewMode === ViewMode.Tree) {
        return this.getFileTreeNodes(components, source);
      } else {
        return this.getListNodes(components, source);
      }
    }

    // 文件节点的子节点
    if (element instanceof FileTreeNode) {
      return element.items.map(
        comp => new ComponentTreeNode(comp as Component, components, this.context, source)
      );
    }

    // 组件节点的子节点（依赖）
    if (element instanceof ComponentTreeNode) {
      return this.getDependencyNodes(element.component, components, source);
    }

    return [];
  }

  /**
   * 获取 List 模式的节点
   */
  private getListNodes(
    components: Map<string, Component>,
    source: DataSource
  ): TreeNode[] {
    return Array.from(components.values()).map(
      comp => new ComponentTreeNode(comp, components, this.context, source)
    );
  }

  /**
   * 获取 Tree 模式的节点（按文件组织）
   */
  private getFileTreeNodes(
    components: Map<string, Component>,
    source: DataSource
  ): TreeNode[] {
    // 按文件路径分组
    const fileMap = new Map<string, Component[]>();
    
    for (const component of components.values()) {
      if (!component.location) {
        continue;
      }

      const fileUri = component.location.uri;
      if (!fileMap.has(fileUri)) {
        fileMap.set(fileUri, []);
      }
      fileMap.get(fileUri)!.push(component);
    }

    // 创建文件节点
    const fileNodes: FileTreeNode[] = [];
    for (const [fileUri, comps] of fileMap.entries()) {
      fileNodes.push(new FileTreeNode(fileUri, comps, 'Components'));
    }

    // 按文件路径排序
    fileNodes.sort((a, b) => a.filePath.localeCompare(b.filePath));

    return fileNodes;
  }

  /**
   * 获取依赖节点
   */
  private getDependencyNodes(
    component: Component,
    allComponents: Map<string, Component>,
    source: DataSource
  ): TreeNode[] {
    if (component.dependencies.length === 0) {
      return [];
    }

    const dependencyNodes: TreeNode[] = [];
    
    for (const depTypeName of component.dependencies) {
      let depComponent = allComponents.get(depTypeName);
      
      if (!depComponent) {
        for (const comp of allComponents.values()) {
          if (comp.typeName === depTypeName || comp.name === depTypeName) {
            depComponent = comp;
            break;
          }
        }
      }
      
      if (depComponent) {
        dependencyNodes.push(
          new ComponentTreeNode(depComponent, allComponents, this.context, source)
        );
      } else {
        dependencyNodes.push(new PlaceholderTreeNode(depTypeName));
      }
    }
    
    return dependencyNodes;
  }
}

/**
 * 树节点基类
 */
type TreeNode = FileTreeNode | ComponentTreeNode | PlaceholderTreeNode;

/**
 * 组件树节点
 */
class ComponentTreeNode extends vscode.TreeItem {
  public readonly component: Component;

  constructor(
    component: Component,
    private readonly allComponents: Map<string, Component>,
    private readonly context: vscode.ExtensionContext,
    private readonly source: DataSource
  ) {
    super(
      component.name,
      component.dependencies.length > 0
        ? vscode.TreeItemCollapsibleState.Collapsed
        : vscode.TreeItemCollapsibleState.None
    );

    this.component = component;

    // 设置上下文值
    this.contextValue = `summer:component-${source}`;

    // 设置工具提示
    this.tooltip = this.buildTooltip();

    // 设置描述
    this.description = this.getDescription();

    // 设置图标
    this.iconPath = this.getIcon();

    // 设置点击命令
    if (component.location) {
      this.command = {
        command: 'summer.component.navigate',
        title: 'Go to Definition',
        arguments: [component.location],
      };
    }
  }

  private buildTooltip(): vscode.MarkdownString {
    const tooltip = new vscode.MarkdownString();
    tooltip.isTrusted = true;

    tooltip.appendMarkdown(`### ${this.component.name}\n\n`);
    tooltip.appendMarkdown(`**Type:** \`${this.component.typeName}\`\n\n`);
    tooltip.appendMarkdown(`**Scope:** ${this.component.scope}\n\n`);

    if (this.component.source === ComponentSource.Component) {
      tooltip.appendMarkdown(`**Defined with:** \`#[component]\` 🟣\n\n`);
    } else {
      tooltip.appendMarkdown(`**Defined with:** \`#[derive(Service)]\` 🔵\n\n`);
    }

    if (this.source === DataSource.Runtime) {
      tooltip.appendMarkdown('✅ **Runtime Information**\n\n');
    } else {
      tooltip.appendMarkdown('📝 **Static Analysis**\n\n');
    }

    if (this.component.dependencies.length > 0) {
      tooltip.appendMarkdown(`**Dependencies:**\n`);
      this.component.dependencies.forEach((dep) => {
        tooltip.appendMarkdown(`- ${dep}\n`);
      });
    }

    return tooltip;
  }

  private getDescription(): string {
    const parts: string[] = [];
    parts.push(this.component.scope);
    if (this.component.dependencies.length > 0) {
      parts.push(`${this.component.dependencies.length} deps`);
    }
    return parts.join(' • ');
  }

  private getIcon(): vscode.ThemeIcon | vscode.Uri {
    // 根据组件定义方式使用不同颜色
    let color: vscode.ThemeColor;
    let iconName: string;
    
    if (this.component.source === ComponentSource.Component) {
      // #[component] 宏：使用紫色/品红色
      color = new vscode.ThemeColor('charts.purple');
      iconName = 'symbol-method'; // 函数图标
    } else {
      // #[derive(Service)] 宏：使用蓝色
      color = new vscode.ThemeColor('charts.blue');
      iconName = 'symbol-class'; // 类图标
    }
    
    // 如果是运行时信息，使用绿色
    if (this.source === DataSource.Runtime) {
      color = new vscode.ThemeColor('charts.green');
    }

    // 尝试使用 SVG 图标，如果不存在则使用主题图标
    try {
      const iconFileName = this.component.source === ComponentSource.Component
        ? 'component-function.svg' 
        : 'component-class.svg';
      
      return vscode.Uri.joinPath(
        this.context.extensionUri,
        'resources',
        'icons',
        iconFileName
      );
    } catch {
      return new vscode.ThemeIcon(iconName, color);
    }
  }
}

/**
 * 占位符树节点
 */
class PlaceholderTreeNode extends vscode.TreeItem {
  constructor(typeName: string) {
    super(typeName, vscode.TreeItemCollapsibleState.None);

    this.contextValue = 'summer:dependency:external';
    this.description = 'external';
    this.iconPath = new vscode.ThemeIcon(
      'symbol-interface',
      new vscode.ThemeColor('symbolIcon.interfaceForeground')
    );

    this.tooltip = new vscode.MarkdownString(
      `**External Dependency**\n\n` +
      `Type: \`${typeName}\`\n\n` +
      `This is an external dependency.`
    );
  }
}
