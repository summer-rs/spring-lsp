/**
 * Jobs 视图提供器
 * 
 * 显示项目中的所有定时任务（带有 #[cron], #[fix_delay], #[fix_rate] 的函数）
 * 支持静态分析和运行时信息两种模式
 */

import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';

/**
 * 任务类型
 */
export type JobType = 'Cron' | 'FixDelay' | 'FixRate';

/**
 * 任务信息
 */
export interface Job {
    name: string;
    jobType: JobType;
    schedule: string;
    location: {
        uri: string;
        range: {
            start: { line: number; character: number };
            end: { line: number; character: number };
        };
    };
    // 运行时信息（可选）
    executionCount?: number;
    lastExecutionTime?: string;
    nextExecutionTime?: string;
    avgExecutionTime?: number;
    errorCount?: number;
}

/**
 * 任务来源
 */
export enum JobSource {
    Static = 'static',
    Runtime = 'runtime'
}

/**
 * 任务树项
 */
export class JobTreeItem extends vscode.TreeItem {
    constructor(
        public readonly job: Job,
        public readonly source: JobSource
    ) {
        super(job.name, vscode.TreeItemCollapsibleState.None);

        this.tooltip = this.buildTooltip();
        this.description = this.buildDescription();
        this.iconPath = this.getIcon();
        this.contextValue = `job-${source}-${job.jobType}`;

        // 点击时跳转到定义
        this.command = {
            command: 'summer.job.navigate',
            title: 'Go to Definition',
            arguments: [this.job]
        };
    }

    private buildTooltip(): vscode.MarkdownString {
        const md = new vscode.MarkdownString();
        md.appendMarkdown(`**${this.job.name}**\n\n`);
        md.appendMarkdown(`Type: ${this.job.jobType}\n\n`);
        md.appendMarkdown(`Schedule: \`${this.job.schedule}\`\n\n`);

        if (this.source === JobSource.Runtime) {
            md.appendMarkdown('✅ **Runtime Statistics**\n\n');
            if (this.job.executionCount !== undefined) {
                md.appendMarkdown(`Executions: ${this.job.executionCount}\n\n`);
            }
            if (this.job.avgExecutionTime !== undefined) {
                md.appendMarkdown(`Avg Execution Time: ${this.job.avgExecutionTime}ms\n\n`);
            }
            if (this.job.lastExecutionTime) {
                md.appendMarkdown(`Last Run: ${this.job.lastExecutionTime}\n\n`);
            }
            if (this.job.nextExecutionTime) {
                md.appendMarkdown(`Next Run: ${this.job.nextExecutionTime}\n\n`);
            }
            if (this.job.errorCount !== undefined && this.job.errorCount > 0) {
                md.appendMarkdown(`⚠️ Errors: ${this.job.errorCount}\n\n`);
            }
        } else {
            md.appendMarkdown('📝 **Static Analysis**\n\n');
            md.appendMarkdown('_Start the application to see runtime statistics_\n\n');
        }

        return md;
    }

    private buildDescription(): string {
        if (this.source === JobSource.Runtime) {
            if (this.job.executionCount !== undefined) {
                return `(${this.job.executionCount} runs)`;
            }
            return '(runtime)';
        }
        return `${this.job.schedule}`;
    }

    private getIcon(): vscode.ThemeIcon {
        const iconMap: Record<JobType, string> = {
            'Cron': 'clock',
            'FixDelay': 'watch',
            'FixRate': 'pulse'
        };

        const icon = iconMap[this.job.jobType] || 'symbol-event';
        const color = this.source === JobSource.Runtime
            ? new vscode.ThemeColor('charts.green')
            : new vscode.ThemeColor('charts.blue');

        return new vscode.ThemeIcon(icon, color);
    }
}

/**
 * Jobs 视图数据提供器
 */
export class JobsDataProvider implements vscode.TreeDataProvider<vscode.TreeItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<vscode.TreeItem | undefined>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    private staticJobs: Job[] = [];
    private runtimeJobs: Job[] = [];

    constructor(private languageClient: LanguageClient) {
        // 监听文档保存
        vscode.workspace.onDidSaveTextDocument(doc => {
            if (doc.languageId === 'rust') {
                this.refreshStatic();
            }
        });

        // 初始加载
        this.refreshStatic();
    }

    public async refreshStatic(): Promise<void> {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders || workspaceFolders.length === 0) {
            this.staticJobs = [];
            this._onDidChangeTreeData.fire(undefined);
            return;
        }

        const workspacePath = workspaceFolders[0].uri.fsPath;

        try {
            const response = await this.languageClient.sendRequest<{ jobs: Job[] }>(
                'summer/jobs',
                { appPath: workspacePath }
            );

            this.staticJobs = response.jobs || [];
            console.log(`Loaded ${this.staticJobs.length} jobs from static analysis`);
            this._onDidChangeTreeData.fire(undefined);
        } catch (error) {
            console.error('Failed to load jobs:', error);
            this.staticJobs = [];
            this._onDidChangeTreeData.fire(undefined);
        }
    }

    public async refreshRuntime(port: number): Promise<void> {
        try {
            const response = await fetch(`http://localhost:${port}/_debug/jobs`);
            if (response.ok) {
                const data = await response.json() as { jobs?: Job[] };
                this.runtimeJobs = data.jobs || [];
                console.log(`Loaded ${this.runtimeJobs.length} jobs from runtime`);
                this._onDidChangeTreeData.fire(undefined);
            }
        } catch (error) {
            console.warn('Failed to load runtime jobs:', error);
        }
    }

    public clearRuntime(): void {
        this.runtimeJobs = [];
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

        const jobs = this.runtimeJobs.length > 0
            ? this.runtimeJobs
            : this.staticJobs;

        const source = this.runtimeJobs.length > 0
            ? JobSource.Runtime
            : JobSource.Static;

        if (jobs.length === 0) {
            const item = new vscode.TreeItem('No jobs found');
            item.iconPath = new vscode.ThemeIcon('info');
            item.contextValue = 'empty';
            return [item];
        }

        return jobs
            .sort((a, b) => a.name.localeCompare(b.name))
            .map(job => new JobTreeItem(job, source));
    }

    public hasRuntimeInfo(): boolean {
        return this.runtimeJobs.length > 0;
    }
}
