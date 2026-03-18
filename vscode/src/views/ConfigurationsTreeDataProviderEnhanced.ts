import * as vscode from 'vscode';
import { SummerApp } from '../models';
import { LanguageClientManager } from '../languageClient';
import { ConfigurationStruct, ConfigurationsResponse } from '../types';
import { ViewMode, VIEW_MODE_KEYS } from '../types/viewMode';
import { FileTreeNode, createFileTreeNodes } from './BaseTreeDataProvider';

/**
 * 增强版 Configurations 树视图数据提供者
 * 
 * 支持两种视图模式：
 * - List 模式：按配置节分组显示
 * - Tree 模式：按文件组织显示
 */
export class ConfigurationsTreeDataProviderEnhanced
  implements vscode.TreeDataProvider<TreeNode>
{
  private _onDidChangeTreeData = new vscode.EventEmitter<TreeNode | undefined>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  private configurations: ConfigurationStruct[] = [];
  private currentApp: SummerApp | undefined;
  private viewMode: ViewMode = ViewMode.List;

  constructor(
    private readonly clientManager: LanguageClientManager,
    private readonly context: vscode.ExtensionContext
  ) {
    this.loadViewMode();

    vscode.workspace.onDidChangeConfiguration(e => {
      if (e.affectsConfiguration(VIEW_MODE_KEYS.configurations)) {
        this.loadViewMode();
        this._onDidChangeTreeData.fire(undefined);
      }
    });

    vscode.workspace.onDidSaveTextDocument(doc => {
      if (doc.languageId === 'toml') {
        this.refreshStatic();
      }
    });

    vscode.workspace.onDidChangeWorkspaceFolders(() => {
      this.refreshStatic();
    });

    this.refreshStatic();
  }

  private loadViewMode(): void {
    const config = vscode.workspace.getConfiguration();
    const mode = config.get<string>(VIEW_MODE_KEYS.configurations, ViewMode.List);
    this.viewMode = mode as ViewMode;
    console.log(`[ConfigurationsTreeDataProvider] View mode: ${this.viewMode}`);
  }

  /**
   * 选择视图模式（通过快速选择）
   */
  public async selectViewMode(): Promise<void> {
    const items: vscode.QuickPickItem[] = [
      {
        label: '$(list-flat) List',
        description: 'Flat list view',
        detail: 'Show all configuration structs in a flat list',
        picked: this.viewMode === ViewMode.List
      },
      {
        label: '$(list-tree) Tree',
        description: 'Group by file',
        detail: 'Organize configuration structs by file',
        picked: this.viewMode === ViewMode.Tree
      }
    ];

    const selected = await vscode.window.showQuickPick(items, {
      placeHolder: 'Select view mode for Configurations',
      title: 'Configurations View Mode'
    });

    if (!selected) {
      return;
    }

    const newMode = selected.label.includes('List') ? ViewMode.List : ViewMode.Tree;
    
    if (newMode !== this.viewMode) {
      await vscode.workspace.getConfiguration().update(
        VIEW_MODE_KEYS.configurations,
        newMode,
        vscode.ConfigurationTarget.Workspace
      );
      this.viewMode = newMode;
      this._onDidChangeTreeData.fire(undefined);
      
      vscode.window.showInformationMessage(
        `Configurations view: ${newMode === ViewMode.List ? 'List' : 'Tree'} mode`
      );
    }
  }

  /**
   * 切换视图模式（快速切换）
   */
  public async toggleViewMode(): Promise<void> {
    const newMode = this.viewMode === ViewMode.List ? ViewMode.Tree : ViewMode.List;
    await vscode.workspace.getConfiguration().update(
      VIEW_MODE_KEYS.configurations,
      newMode,
      vscode.ConfigurationTarget.Workspace
    );
    this.viewMode = newMode;
    this._onDidChangeTreeData.fire(undefined);
    
    vscode.window.showInformationMessage(
      `Configurations view: ${newMode === ViewMode.List ? 'List' : 'Tree'} mode`
    );
  }

  public async refreshStatic(): Promise<void> {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders || workspaceFolders.length === 0) {
      this.configurations = [];
      this._onDidChangeTreeData.fire(undefined);
      return;
    }

    const workspacePath = workspaceFolders[0].uri.fsPath;
    await this.refreshStaticByPath(workspacePath);
  }

  private async refreshStaticByPath(appPath: string): Promise<void> {
    try {
      const response = await this.clientManager.sendRequest<ConfigurationsResponse>(
        'summer/configurations',
        { appPath }
      );

      this.configurations = response?.configurations || [];
      console.log(`[ConfigurationsTreeDataProvider] Loaded ${this.configurations.length} configurations`);
      this._onDidChangeTreeData.fire(undefined);
    } catch (error) {
      console.error('Failed to load configurations:', error);
      this.configurations = [];
      this._onDidChangeTreeData.fire(undefined);
    }
  }

  public async refresh(app?: SummerApp): Promise<void> {
    if (!app) {
      this.configurations = [];
      this.currentApp = undefined;
      this._onDidChangeTreeData.fire(undefined);
      return;
    }

    this.currentApp = app;
    await this.refreshStaticByPath(app.path);
  }

  public getTreeItem(element: TreeNode): vscode.TreeItem {
    return element;
  }

  public async getChildren(element?: TreeNode): Promise<TreeNode[]> {
    if (this.configurations.length === 0) {
      return [];
    }

    // 根节点
    if (!element) {
      if (this.viewMode === ViewMode.Tree) {
        return createFileTreeNodes(this.configurations, 'Configurations');
      } else {
        // List 模式：平铺显示所有配置结构体
        return this.getListNodes();
      }
    }

    // 文件节点的子节点
    if (element instanceof FileTreeNode) {
      return element.items.map(
        config => new ConfigStructTreeNode(config as ConfigurationStruct, this.context)
      );
    }

    return [];
  }

  /**
   * 获取平铺列表节点（List 模式）
   */
  private getListNodes(): TreeNode[] {
    // 按名称排序
    const sortedConfigs = [...this.configurations].sort((a, b) => 
      a.name.localeCompare(b.name)
    );
    
    return sortedConfigs.map(
      config => new ConfigStructTreeNode(config, this.context)
    );
  }
}

type TreeNode = FileTreeNode | ConfigStructTreeNode;

/**
 * 配置结构体树节点
 */
class ConfigStructTreeNode extends vscode.TreeItem {
  constructor(
    public readonly configStruct: ConfigurationStruct,
    private readonly context: vscode.ExtensionContext
  ) {
    super(configStruct.name, vscode.TreeItemCollapsibleState.None);

    this.contextValue = 'summer:configStruct';
    this.description = `[${configStruct.prefix}]`;
    this.tooltip = this.buildTooltip();
    
    // 使用专用的 config.svg 图标
    this.iconPath = {
      light: vscode.Uri.joinPath(context.extensionUri, 'resources', 'icons', 'config.svg'),
      dark: vscode.Uri.joinPath(context.extensionUri, 'resources', 'icons', 'config.svg')
    };

    if (configStruct.location) {
      this.command = {
        command: 'summer.configuration.navigate',
        title: 'Go to Definition',
        arguments: [configStruct.location],
      };
    }
  }

  private buildTooltip(): vscode.MarkdownString {
    const tooltip = new vscode.MarkdownString();
    tooltip.isTrusted = true;

    tooltip.appendMarkdown(`### ${this.configStruct.name}\n\n`);
    tooltip.appendMarkdown(`**Prefix:** \`[${this.configStruct.prefix}]\`\n\n`);

    if (this.configStruct.fields.length > 0) {
      tooltip.appendMarkdown(`**Fields:**\n\n`);
      for (const field of this.configStruct.fields) {
        const optional = field.optional ? ' (optional)' : '';
        tooltip.appendMarkdown(`- \`${field.name}: ${field.type}\`${optional}\n`);
        if (field.description) {
          tooltip.appendMarkdown(`  - ${field.description}\n`);
        }
      }
    }

    tooltip.appendMarkdown(`\n*Click to go to definition*`);

    return tooltip;
  }
}
