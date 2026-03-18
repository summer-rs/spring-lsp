/**
 * Components 视图提供器
 * 
 * 显示项目中的所有组件（带有 #[derive(Service)] 的结构体）
 * 支持静态分析和运行时信息两种模式
 */

import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';

/**
 * 组件信息
 */
export interface Component {
    name: string;
    typeName: string;
    scope: 'Singleton' | 'Prototype';
    dependencies: string[];
    location: {
        uri: string;
        range: {
            start: { line: number; character: number };
            end: { line: number; character: number };
        };
    };
    // 运行时信息（可选）
    instanceCount?: number;
    memoryUsage?: number;
}

/**
 * 组件来源
 */
export enum ComponentSource {
    Static = 'static',      // 静态分析
    Runtime = 'runtime'     // 运行时
}

/**
 * 组件树项
 */
export class ComponentTreeItem extends vscode.TreeItem {
    constructor(
        public readonly component: Component,
        public readonly source: ComponentSource,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState
    ) {
        super(component.name, collapsibleState);

        this.tooltip = this.buildTooltip();
        this.description = this.buildDescription();
        this.iconPath = this.getIcon();
        this.contextValue = `component-${source}`;

        // 点击时跳转到定义
        this.command = {
            command: 'summer.component.navigate',
            title: 'Go to Definition',
            arguments: [this.component]
        };
    }

    private buildTooltip(): vscode.MarkdownString {
        const md = new vscode.MarkdownString();
        md.appendMarkdown(`**${this.component.name}**\n\n`);
        md.appendMarkdown(`Type: \`${this.component.typeName}\`\n\n`);
        md.appendMarkdown(`Scope: ${this.component.scope}\n\n`);

        if (this.source === ComponentSource.Runtime) {
            md.appendMarkdown('✅ **Runtime Information**\n\n');
            if (this.component.instanceCount !== undefined) {
                md.appendMarkdown(`Instances: ${this.component.instanceCount}\n\n`);
            }
            if (this.component.memoryUsage !== undefined) {
                md.appendMarkdown(`Memory: ${this.formatBytes(this.component.memoryUsage)}\n\n`);
            }
        } else {
            md.appendMarkdown('📝 **Static Analysis**\n\n');
            md.appendMarkdown('_Start the application to see runtime information_\n\n');
        }

        if (this.component.dependencies.length > 0) {
            md.appendMarkdown(`**Dependencies:**\n`);
            this.component.dependencies.forEach(dep => {
                md.appendMarkdown(`- ${dep}\n`);
            });
        }

        return md;
    }

    private buildDescription(): string {
        if (this.source === ComponentSource.Runtime) {
            return '(runtime)';
        }
        return '(static)';
    }

    private getIcon(): vscode.ThemeIcon {
        const color = this.source === ComponentSource.Runtime
            ? new vscode.ThemeColor('charts.green')
            : new vscode.ThemeColor('charts.blue');

        return new vscode.ThemeIcon('symbol-class', color);
    }

    private formatBytes(bytes: number): string {
        if (bytes < 1024) return `${bytes} B`;
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
        return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
    }
}

/**
 * 依赖树项
 */
export class DependencyTreeItem extends vscode.TreeItem {
    constructor(
        public readonly dependencyName: string,
        public readonly component: Component | undefined,
        public readonly source: ComponentSource
    ) {
        super(
            dependencyName,
            component ? vscode.TreeItemCollapsibleState.Collapsed : vscode.TreeItemCollapsibleState.None
        );

        this.tooltip = component ? `${component.typeName}` : `${dependencyName} (not found)`;
        this.iconPath = new vscode.ThemeIcon('symbol-field');
        this.contextValue = component ? 'dependency' : 'dependency-missing';

        if (component) {
            this.command = {
                command: 'summer.component.navigate',
                title: 'Go to Definition',
                arguments: [component]
            };
        }
    }
}

/**
 * Components 视图数据提供器
 */
export class ComponentsDataProvider implements vscode.TreeDataProvider<vscode.TreeItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<vscode.TreeItem | undefined>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    private staticComponents: Component[] = [];
    private runtimeComponents: Component[] = [];
    private currentWorkspacePath: string | undefined;

    constructor(private languageClient: LanguageClient) {
        // 监听文档保存，触发静态分析
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
     * 刷新静态分析结果
     */
    public async refreshStatic(): Promise<void> {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders || workspaceFolders.length === 0) {
            this.staticComponents = [];
            this._onDidChangeTreeData.fire(undefined);
            return;
        }

        // 使用第一个工作空间文件夹
        const workspacePath = workspaceFolders[0].uri.fsPath;
        this.currentWorkspacePath = workspacePath;

        try {
            const response = await this.languageClient.sendRequest<{ components: Component[] }>(
                'summer/components',
                { appPath: workspacePath }
            );

            this.staticComponents = response.components || [];
            console.log(`Loaded ${this.staticComponents.length} components from static analysis`);
            this._onDidChangeTreeData.fire(undefined);
        } catch (error) {
            console.error('Failed to load components:', error);
            this.staticComponents = [];
            this._onDidChangeTreeData.fire(undefined);
        }
    }

    /**
     * 刷新运行时信息
     * 
     * @param port 应用运行的端口
     */
    public async refreshRuntime(port: number): Promise<void> {
        try {
            // 从运行中的应用获取组件信息
            const response = await fetch(`http://localhost:${port}/_debug/components`);
            if (response.ok) {
                const data = await response.json() as { components?: Component[] };
                this.runtimeComponents = data.components || [];
                console.log(`Loaded ${this.runtimeComponents.length} components from runtime`);
                this._onDidChangeTreeData.fire(undefined);
            }
        } catch (error) {
            console.warn('Failed to load runtime components:', error);
            // 运行时信息加载失败不影响静态信息
        }
    }

    /**
     * 清除运行时信息
     */
    public clearRuntime(): void {
        this.runtimeComponents = [];
        this._onDidChangeTreeData.fire(undefined);
    }

    /**
     * 手动刷新
     */
    public refresh(): void {
        this.refreshStatic();
    }

    getTreeItem(element: vscode.TreeItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: vscode.TreeItem): Promise<vscode.TreeItem[]> {
        if (!element) {
            // 根节点：显示所有组件
            return this.getRootComponents();
        }

        if (element instanceof ComponentTreeItem) {
            // 组件节点：显示依赖
            return this.getComponentDependencies(element.component);
        }

        if (element instanceof DependencyTreeItem && element.component) {
            // 依赖节点：显示依赖的依赖
            return this.getComponentDependencies(element.component);
        }

        return [];
    }

    /**
     * 获取根组件列表
     */
    private getRootComponents(): vscode.TreeItem[] {
        // 优先使用运行时信息，否则使用静态分析结果
        const components = this.runtimeComponents.length > 0
            ? this.runtimeComponents
            : this.staticComponents;

        const source = this.runtimeComponents.length > 0
            ? ComponentSource.Runtime
            : ComponentSource.Static;

        if (components.length === 0) {
            // 显示提示信息
            const item = new vscode.TreeItem('No components found');
            item.iconPath = new vscode.ThemeIcon('info');
            item.contextValue = 'empty';
            return [item];
        }

        return components.map(component =>
            new ComponentTreeItem(
                component,
                source,
                component.dependencies.length > 0
                    ? vscode.TreeItemCollapsibleState.Collapsed
                    : vscode.TreeItemCollapsibleState.None
            )
        );
    }

    /**
     * 获取组件的依赖列表
     */
    private getComponentDependencies(component: Component): vscode.TreeItem[] {
        if (component.dependencies.length === 0) {
            return [];
        }

        const components = this.runtimeComponents.length > 0
            ? this.runtimeComponents
            : this.staticComponents;

        const source = this.runtimeComponents.length > 0
            ? ComponentSource.Runtime
            : ComponentSource.Static;

        return component.dependencies.map(depName => {
            const depComponent = components.find(c => c.name === depName || c.typeName === depName);
            return new DependencyTreeItem(depName, depComponent, source);
        });
    }

    /**
     * 检查是否有运行时信息
     */
    public hasRuntimeInfo(): boolean {
        return this.runtimeComponents.length > 0;
    }
}
