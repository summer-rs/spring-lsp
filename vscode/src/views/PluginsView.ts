/**
 * Plugins 视图提供器
 * 
 * 显示项目中的所有插件（.add_plugin() 调用）
 * 支持静态分析和运行时信息两种模式
 */

import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';

/**
 * 插件信息
 */
export interface Plugin {
    name: string;
    typeName: string;
    configPrefix?: string;
    location: {
        uri: string;
        range: {
            start: { line: number; character: number };
            end: { line: number; character: number };
        };
    };
    // 运行时信息（可选）
    status?: 'active' | 'inactive' | 'error';
    version?: string;
    dependencies?: string[];
}

/**
 * 插件来源
 */
export enum PluginSource {
    Static = 'static',
    Runtime = 'runtime'
}

/**
 * 插件树项
 */
export class PluginTreeItem extends vscode.TreeItem {
    constructor(
        public readonly plugin: Plugin,
        public readonly source: PluginSource
    ) {
        super(plugin.name, vscode.TreeItemCollapsibleState.None);

        this.tooltip = this.buildTooltip();
        this.description = this.buildDescription();
        this.iconPath = this.getIcon();
        this.contextValue = `plugin-${source}`;

        // 点击时跳转到定义
        this.command = {
            command: 'summer.plugin.navigate',
            title: 'Go to Definition',
            arguments: [this.plugin]
        };
    }

    private buildTooltip(): vscode.MarkdownString {
        const md = new vscode.MarkdownString();
        md.appendMarkdown(`**${this.plugin.name}**\n\n`);
        md.appendMarkdown(`Type: \`${this.plugin.typeName}\`\n\n`);

        if (this.plugin.configPrefix) {
            md.appendMarkdown(`Config Prefix: \`[${this.plugin.configPrefix}]\`\n\n`);
        }

        if (this.source === PluginSource.Runtime) {
            md.appendMarkdown('✅ **Runtime Information**\n\n');
            if (this.plugin.status) {
                const statusIcon = this.plugin.status === 'active' ? '✅' :
                                  this.plugin.status === 'error' ? '❌' : '⚠️';
                md.appendMarkdown(`Status: ${statusIcon} ${this.plugin.status}\n\n`);
            }
            if (this.plugin.version) {
                md.appendMarkdown(`Version: ${this.plugin.version}\n\n`);
            }
            if (this.plugin.dependencies && this.plugin.dependencies.length > 0) {
                md.appendMarkdown(`**Dependencies:**\n`);
                this.plugin.dependencies.forEach(dep => {
                    md.appendMarkdown(`- ${dep}\n`);
                });
            }
        } else {
            md.appendMarkdown('📝 **Static Analysis**\n\n');
            md.appendMarkdown('_Start the application to see runtime information_\n\n');
        }

        return md;
    }

    private buildDescription(): string {
        if (this.source === PluginSource.Runtime && this.plugin.status) {
            return `(${this.plugin.status})`;
        }
        if (this.plugin.configPrefix) {
            return `[${this.plugin.configPrefix}]`;
        }
        return '';
    }

    private getIcon(): vscode.ThemeIcon {
        if (this.source === PluginSource.Runtime) {
            if (this.plugin.status === 'active') {
                return new vscode.ThemeIcon('extensions', new vscode.ThemeColor('charts.green'));
            } else if (this.plugin.status === 'error') {
                return new vscode.ThemeIcon('extensions', new vscode.ThemeColor('charts.red'));
            } else {
                return new vscode.ThemeIcon('extensions', new vscode.ThemeColor('charts.yellow'));
            }
        }

        return new vscode.ThemeIcon('extensions', new vscode.ThemeColor('charts.blue'));
    }
}

/**
 * Plugins 视图数据提供器
 */
export class PluginsDataProvider implements vscode.TreeDataProvider<vscode.TreeItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<vscode.TreeItem | undefined>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    private staticPlugins: Plugin[] = [];
    private runtimePlugins: Plugin[] = [];

    constructor(private languageClient: LanguageClient) {
        // 监听文档保存
        vscode.workspace.onDidSaveTextDocument(doc => {
            if (doc.languageId === 'rust' && doc.fileName.endsWith('main.rs')) {
                this.refreshStatic();
            }
        });

        // 初始加载
        this.refreshStatic();
    }

    public async refreshStatic(): Promise<void> {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders || workspaceFolders.length === 0) {
            this.staticPlugins = [];
            this._onDidChangeTreeData.fire(undefined);
            return;
        }

        const workspacePath = workspaceFolders[0].uri.fsPath;

        try {
            const response = await this.languageClient.sendRequest<{ plugins: Plugin[] }>(
                'summer/plugins',
                { appPath: workspacePath }
            );

            this.staticPlugins = response.plugins || [];
            console.log(`Loaded ${this.staticPlugins.length} plugins from static analysis`);
            this._onDidChangeTreeData.fire(undefined);
        } catch (error) {
            console.error('Failed to load plugins:', error);
            this.staticPlugins = [];
            this._onDidChangeTreeData.fire(undefined);
        }
    }

    public async refreshRuntime(port: number): Promise<void> {
        try {
            const response = await fetch(`http://localhost:${port}/_debug/plugins`);
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

    public clearRuntime(): void {
        this.runtimePlugins = [];
        this._onDidChangeTreeData.fire(undefined);
    }

    public refresh(): void {
        this.refreshStatic();
    }

    getTreeItem(element: vscode.TreeItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: vscode.TreeItem): Promise<vscode.TreeItem[]> {
        if (element) {
            return [];
        }

        const plugins = this.runtimePlugins.length > 0
            ? this.runtimePlugins
            : this.staticPlugins;

        const source = this.runtimePlugins.length > 0
            ? PluginSource.Runtime
            : PluginSource.Static;

        if (plugins.length === 0) {
            const item = new vscode.TreeItem('No plugins found');
            item.iconPath = new vscode.ThemeIcon('info');
            item.contextValue = 'empty';
            return [item];
        }

        return plugins
            .sort((a, b) => a.name.localeCompare(b.name))
            .map(plugin => new PluginTreeItem(plugin, source));
    }

    public hasRuntimeInfo(): boolean {
        return this.runtimePlugins.length > 0;
    }
}
