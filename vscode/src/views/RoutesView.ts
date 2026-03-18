/**
 * Routes 视图提供器
 * 
 * 显示项目中的所有路由（带有 #[get], #[post] 等宏的函数）
 * 支持静态分析和运行时信息两种模式
 */

import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';

/**
 * 路由信息
 */
export interface Route {
    method: string;
    path: string;
    handler: string;
    isOpenapi: boolean;
    location: {
        uri: string;
        range: {
            start: { line: number; character: number };
            end: { line: number; character: number };
        };
    };
    // 运行时信息（可选）
    requestCount?: number;
    avgResponseTime?: number;
    errorCount?: number;
}

/**
 * 路由来源
 */
export enum RouteSource {
    Static = 'static',      // 静态分析
    Runtime = 'runtime'     // 运行时
}

/**
 * HTTP 方法分组树项
 */
export class MethodGroupTreeItem extends vscode.TreeItem {
    constructor(
        public readonly method: string,
        public readonly routes: Route[],
        public readonly source: RouteSource
    ) {
        super(method, vscode.TreeItemCollapsibleState.Expanded);

        this.tooltip = `${routes.length} ${method} route(s)`;
        this.description = `(${routes.length})`;
        this.iconPath = this.getMethodIcon(method);
        this.contextValue = 'method-group';
    }

    private getMethodIcon(method: string): vscode.ThemeIcon {
        const iconMap: Record<string, string> = {
            'GET': 'arrow-down',
            'POST': 'add',
            'PUT': 'edit',
            'DELETE': 'trash',
            'PATCH': 'diff-modified',
            'HEAD': 'eye',
            'OPTIONS': 'settings-gear'
        };

        const icon = iconMap[method] || 'symbol-method';
        return new vscode.ThemeIcon(icon);
    }
}

/**
 * 路由树项
 */
export class RouteTreeItem extends vscode.TreeItem {
    constructor(
        public readonly route: Route,
        public readonly source: RouteSource
    ) {
        super(route.path, vscode.TreeItemCollapsibleState.None);

        this.tooltip = this.buildTooltip();
        this.description = this.buildDescription();
        this.iconPath = this.getIcon();
        this.contextValue = this.buildContextValue();

        // 点击时跳转到处理器函数
        this.command = {
            command: 'summer.route.navigate',
            title: 'Go to Handler',
            arguments: [this.route]
        };
    }

    private buildTooltip(): vscode.MarkdownString {
        const md = new vscode.MarkdownString();
        md.appendMarkdown(`**${this.route.method} ${this.route.path}**\n\n`);
        md.appendMarkdown(`Handler: \`${this.route.handler}\`\n\n`);

        if (this.route.isOpenapi) {
            md.appendMarkdown('📄 OpenAPI documented\n\n');
        }

        if (this.source === RouteSource.Runtime) {
            md.appendMarkdown('✅ **Runtime Statistics**\n\n');
            if (this.route.requestCount !== undefined) {
                md.appendMarkdown(`Requests: ${this.route.requestCount}\n\n`);
            }
            if (this.route.avgResponseTime !== undefined) {
                md.appendMarkdown(`Avg Response Time: ${this.route.avgResponseTime}ms\n\n`);
            }
            if (this.route.errorCount !== undefined) {
                md.appendMarkdown(`Errors: ${this.route.errorCount}\n\n`);
            }
        } else {
            md.appendMarkdown('📝 **Static Analysis**\n\n');
            md.appendMarkdown('_Start the application to see runtime statistics_\n\n');
        }

        return md;
    }

    private buildDescription(): string {
        if (this.source === RouteSource.Runtime && this.route.requestCount !== undefined) {
            return `(${this.route.requestCount} requests)`;
        }
        if (this.source === RouteSource.Static) {
            return '(static)';
        }
        return '';
    }

    private getIcon(): vscode.ThemeIcon {
        if (this.route.isOpenapi) {
            return new vscode.ThemeIcon('book', new vscode.ThemeColor('charts.purple'));
        }

        const color = this.source === RouteSource.Runtime
            ? new vscode.ThemeColor('charts.green')
            : new vscode.ThemeColor('charts.blue');

        return new vscode.ThemeIcon('symbol-method', color);
    }

    private buildContextValue(): string {
        const parts = ['route', this.source];
        
        // 添加 HTTP 方法到 context value，用于条件显示命令
        parts.push(this.route.method);

        // GET 请求可以在浏览器中打开
        if (this.route.method === 'GET') {
            parts.push('openable');
        }

        return parts.join('-');
    }
}

/**
 * Routes 视图数据提供器
 */
export class RoutesDataProvider implements vscode.TreeDataProvider<vscode.TreeItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<vscode.TreeItem | undefined>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    private staticRoutes: Route[] = [];
    private runtimeRoutes: Route[] = [];
    private currentWorkspacePath: string | undefined;
    private currentPort: number | undefined;

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
            this.staticRoutes = [];
            this._onDidChangeTreeData.fire(undefined);
            return;
        }

        const workspacePath = workspaceFolders[0].uri.fsPath;
        this.currentWorkspacePath = workspacePath;

        try {
            const response = await this.languageClient.sendRequest<{ routes: Route[] }>(
                'summer/routes',
                { appPath: workspacePath }
            );

            this.staticRoutes = response.routes || [];
            console.log(`Loaded ${this.staticRoutes.length} routes from static analysis`);
            this._onDidChangeTreeData.fire(undefined);
        } catch (error) {
            console.error('Failed to load routes:', error);
            this.staticRoutes = [];
            this._onDidChangeTreeData.fire(undefined);
        }
    }

    /**
     * 刷新运行时信息
     * 
     * @param port 应用运行的端口
     */
    public async refreshRuntime(port: number): Promise<void> {
        this.currentPort = port;

        try {
            const response = await fetch(`http://localhost:${port}/_debug/routes`);
            if (response.ok) {
                const data = await response.json() as { routes?: Route[] };
                this.runtimeRoutes = data.routes || [];
                console.log(`Loaded ${this.runtimeRoutes.length} routes from runtime`);
                this._onDidChangeTreeData.fire(undefined);
            }
        } catch (error) {
            console.warn('Failed to load runtime routes:', error);
        }
    }

    /**
     * 清除运行时信息
     */
    public clearRuntime(): void {
        this.runtimeRoutes = [];
        this.currentPort = undefined;
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
            // 根节点：按 HTTP 方法分组
            return this.getMethodGroups();
        }

        if (element instanceof MethodGroupTreeItem) {
            // 方法分组：显示该方法的所有路由
            return this.getRoutesForMethod(element.method, element.source);
        }

        return [];
    }

    /**
     * 获取 HTTP 方法分组
     */
    private getMethodGroups(): vscode.TreeItem[] {
        const routes = this.runtimeRoutes.length > 0
            ? this.runtimeRoutes
            : this.staticRoutes;

        const source = this.runtimeRoutes.length > 0
            ? RouteSource.Runtime
            : RouteSource.Static;

        if (routes.length === 0) {
            const item = new vscode.TreeItem('No routes found');
            item.iconPath = new vscode.ThemeIcon('info');
            item.contextValue = 'empty';
            return [item];
        }

        // 按 HTTP 方法分组
        const grouped = this.groupByMethod(routes);

        // 按方法顺序排序
        const methodOrder = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'];
        const sortedMethods = Object.keys(grouped).sort((a, b) => {
            const indexA = methodOrder.indexOf(a);
            const indexB = methodOrder.indexOf(b);
            if (indexA === -1 && indexB === -1) return a.localeCompare(b);
            if (indexA === -1) return 1;
            if (indexB === -1) return -1;
            return indexA - indexB;
        });

        return sortedMethods.map(method =>
            new MethodGroupTreeItem(method, grouped[method], source)
        );
    }

    /**
     * 获取指定方法的所有路由
     */
    private getRoutesForMethod(method: string, source: RouteSource): vscode.TreeItem[] {
        const routes = source === RouteSource.Runtime
            ? this.runtimeRoutes
            : this.staticRoutes;

        return routes
            .filter(route => route.method === method)
            .sort((a, b) => a.path.localeCompare(b.path))
            .map(route => new RouteTreeItem(route, source));
    }

    /**
     * 按 HTTP 方法分组路由
     */
    private groupByMethod(routes: Route[]): Record<string, Route[]> {
        const grouped: Record<string, Route[]> = {};

        for (const route of routes) {
            if (!grouped[route.method]) {
                grouped[route.method] = [];
            }
            grouped[route.method].push(route);
        }

        return grouped;
    }

    /**
     * 检查是否有运行时信息
     */
    public hasRuntimeInfo(): boolean {
        return this.runtimeRoutes.length > 0;
    }

    /**
     * 获取当前端口
     */
    public getCurrentPort(): number | undefined {
        return this.currentPort;
    }
}
