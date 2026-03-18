/**
 * 视图管理器
 * 
 * 统一管理所有视图的生命周期和状态
 */

import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';
import { ComponentsDataProvider } from './ComponentsView';
import { RoutesDataProvider } from './RoutesView';
import { JobsDataProvider } from './JobsView';
import { PluginsDataProvider } from './PluginsView';

/**
 * 应用状态
 */
export interface AppState {
    isRunning: boolean;
    port?: number;
    name?: string;
}

/**
 * 视图管理器
 */
export class ViewManager {
    private componentsProvider: ComponentsDataProvider;
    private routesProvider: RoutesDataProvider;
    private jobsProvider: JobsDataProvider;
    private pluginsProvider: PluginsDataProvider;

    private componentsView: vscode.TreeView<vscode.TreeItem>;
    private routesView: vscode.TreeView<vscode.TreeItem>;
    private jobsView: vscode.TreeView<vscode.TreeItem>;
    private pluginsView: vscode.TreeView<vscode.TreeItem>;

    private currentAppState: AppState = { isRunning: false };

    constructor(
        private context: vscode.ExtensionContext,
        private languageClient: LanguageClient
    ) {
        // 创建数据提供器
        this.componentsProvider = new ComponentsDataProvider(languageClient);
        this.routesProvider = new RoutesDataProvider(languageClient);
        this.jobsProvider = new JobsDataProvider(languageClient);
        this.pluginsProvider = new PluginsDataProvider(languageClient);

        // 创建树视图
        this.componentsView = vscode.window.createTreeView('summer.components', {
            treeDataProvider: this.componentsProvider,
            showCollapseAll: true
        });

        this.routesView = vscode.window.createTreeView('summer.routes', {
            treeDataProvider: this.routesProvider,
            showCollapseAll: true
        });

        this.jobsView = vscode.window.createTreeView('summer.jobs', {
            treeDataProvider: this.jobsProvider,
            showCollapseAll: false
        });

        this.pluginsView = vscode.window.createTreeView('summer.plugins', {
            treeDataProvider: this.pluginsProvider,
            showCollapseAll: false
        });

        // 注册到 context
        context.subscriptions.push(
            this.componentsView,
            this.routesView,
            this.jobsView,
            this.pluginsView
        );

        // 注册命令
        this.registerCommands();

        // 更新视图标题
        this.updateViewTitles();
    }

    /**
     * 注册所有命令
     */
    private registerCommands(): void {
        // 刷新命令
        this.context.subscriptions.push(
            vscode.commands.registerCommand('summer.components.refresh', () => {
                this.componentsProvider.refresh();
            }),
            vscode.commands.registerCommand('summer.routes.refresh', () => {
                this.routesProvider.refresh();
            }),
            vscode.commands.registerCommand('summer.jobs.refresh', () => {
                this.jobsProvider.refresh();
            }),
            vscode.commands.registerCommand('summer.plugins.refresh', () => {
                this.pluginsProvider.refresh();
            }),
            vscode.commands.registerCommand('summer.views.refreshAll', () => {
                this.refreshAll();
            })
        );

        // 导航命令
        this.context.subscriptions.push(
            vscode.commands.registerCommand('summer.component.navigate', (component) => {
                this.navigateToLocation(component.location);
            }),
            vscode.commands.registerCommand('summer.route.navigate', (route) => {
                this.navigateToLocation(route.location);
            }),
            vscode.commands.registerCommand('summer.job.navigate', (job) => {
                this.navigateToLocation(job.location);
            }),
            vscode.commands.registerCommand('summer.plugin.navigate', (plugin) => {
                this.navigateToLocation(plugin.location);
            })
        );

        // 路由特殊命令：在浏览器中打开
        this.context.subscriptions.push(
            vscode.commands.registerCommand('summer.route.open', async (route) => {
                await this.openRouteInBrowser(route);
            })
        );

        // 组件特殊命令：显示依赖图
        this.context.subscriptions.push(
            vscode.commands.registerCommand('summer.component.showDependencies', async (component) => {
                await this.showComponentDependencies(component);
            })
        );
    }

    /**
     * 导航到代码位置
     */
    private async navigateToLocation(location: any): Promise<void> {
        try {
            const uri = vscode.Uri.parse(location.uri);
            const range = new vscode.Range(
                location.range.start.line,
                location.range.start.character,
                location.range.end.line,
                location.range.end.character
            );

            const document = await vscode.workspace.openTextDocument(uri);
            const editor = await vscode.window.showTextDocument(document, {
                selection: range,
                preview: false
            });

            // 滚动到可见区域
            editor.revealRange(range, vscode.TextEditorRevealType.InCenter);
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to navigate: ${error}`);
        }
    }

    /**
     * 在浏览器中打开路由
     */
    private async openRouteInBrowser(route: any): Promise<void> {
        if (!this.currentAppState.isRunning || !this.currentAppState.port) {
            vscode.window.showWarningMessage('Application is not running');
            return;
        }

        const config = vscode.workspace.getConfiguration('summer-rs');
        const urlTemplate = config.get<string>('openUrl', 'http://localhost:{port}{path}');

        const url = urlTemplate
            .replace('{port}', this.currentAppState.port.toString())
            .replace('{path}', route.path);

        try {
            const uri = vscode.Uri.parse(url);
            const externalUri = await vscode.env.asExternalUri(uri);
            await vscode.env.openExternal(externalUri);
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to open URL: ${error}`);
        }
    }

    /**
     * 显示组件依赖图
     */
    private async showComponentDependencies(component: any): Promise<void> {
        // TODO: 实现依赖图可视化
        vscode.window.showInformationMessage(
            `Dependencies of ${component.name}: ${component.dependencies.join(', ') || 'none'}`
        );
    }

    /**
     * 刷新所有视图（静态分析）
     */
    public refreshAll(): void {
        console.log('Refreshing all views (static analysis)...');
        this.componentsProvider.refreshStatic();
        this.routesProvider.refreshStatic();
        this.jobsProvider.refreshStatic();
        this.pluginsProvider.refreshStatic();
    }

    /**
     * 应用启动时调用
     */
    public async onAppStarted(port: number, appName?: string): Promise<void> {
        console.log(`App started on port ${port}`);
        this.currentAppState = {
            isRunning: true,
            port,
            name: appName
        };

        // 更新上下文
        vscode.commands.executeCommand('setContext', 'summer:hasRunningApp', true);

        // 刷新运行时信息
        await this.refreshRuntime(port);

        // 更新视图标题
        this.updateViewTitles();
    }

    /**
     * 应用停止时调用
     */
    public onAppStopped(): void {
        console.log('App stopped');
        this.currentAppState = { isRunning: false };

        // 更新上下文
        vscode.commands.executeCommand('setContext', 'summer:hasRunningApp', false);

        // 清除运行时信息
        this.clearRuntime();

        // 更新视图标题
        this.updateViewTitles();
    }

    /**
     * 刷新运行时信息
     */
    private async refreshRuntime(port: number): Promise<void> {
        console.log('Refreshing runtime information...');
        
        // 并行刷新所有视图的运行时信息
        await Promise.all([
            this.componentsProvider.refreshRuntime(port),
            this.routesProvider.refreshRuntime(port),
            this.jobsProvider.refreshRuntime(port),
            this.pluginsProvider.refreshRuntime(port)
        ]);

        console.log('Runtime information refreshed');
    }

    /**
     * 清除运行时信息
     */
    private clearRuntime(): void {
        console.log('Clearing runtime information...');
        this.componentsProvider.clearRuntime();
        this.routesProvider.clearRuntime();
        this.jobsProvider.clearRuntime();
        this.pluginsProvider.clearRuntime();
    }

    /**
     * 更新视图标题
     */
    private updateViewTitles(): void {
        const suffix = this.currentAppState.isRunning
            ? ` (${this.currentAppState.name || 'running'})`
            : '';

        this.componentsView.title = `Components${suffix}`;
        this.routesView.title = `Routes${suffix}`;
        this.jobsView.title = `Jobs${suffix}`;
        this.pluginsView.title = `Plugins${suffix}`;

        // 更新描述
        const description = this.currentAppState.isRunning
            ? `Port ${this.currentAppState.port}`
            : 'Static Analysis';

        this.componentsView.description = description;
        this.routesView.description = description;
        this.jobsView.description = description;
        this.pluginsView.description = description;
    }

    /**
     * 获取当前应用状态
     */
    public getAppState(): AppState {
        return { ...this.currentAppState };
    }

    /**
     * 检查是否有运行时信息
     */
    public hasRuntimeInfo(): boolean {
        return this.componentsProvider.hasRuntimeInfo() ||
               this.routesProvider.hasRuntimeInfo() ||
               this.jobsProvider.hasRuntimeInfo() ||
               this.pluginsProvider.hasRuntimeInfo();
    }

    /**
     * 释放资源
     */
    public dispose(): void {
        // TreeView 会自动通过 context.subscriptions 释放
    }
}
