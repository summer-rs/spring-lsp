import * as vscode from 'vscode';
import { SummerApp } from '../models';
import { LanguageClientManager } from '../languageClient';
import { Job, JobsResponse } from '../types';
import { navigateToLocation } from '../utils';

/**
 * Jobs 树视图数据提供者
 * 
 * 负责显示运行中应用的定时任务列表
 */
export class JobsTreeDataProvider
  implements vscode.TreeDataProvider<JobTreeItem>
{
  /**
   * 树数据变化事件发射器
   */
  private _onDidChangeTreeData = new vscode.EventEmitter<
    JobTreeItem | undefined
  >();

  /**
   * 树数据变化事件
   */
  readonly onDidChangeTreeData: vscode.Event<JobTreeItem | undefined> =
    this._onDidChangeTreeData.event;

  /**
   * 静态分析的任务列表
   */
  private staticJobs: Job[] = [];

  /**
   * 运行时的任务列表
   */
  private runtimeJobs: Job[] = [];

  /**
   * 当前选中的应用
   */
  private currentApp: SummerApp | undefined;

  /**
   * 语言客户端管理器
   */
  private readonly clientManager: LanguageClientManager;

  /**
   * 扩展上下文（用于获取资源路径）
   */
  private readonly context: vscode.ExtensionContext;

  /**
   * 创建 JobsTreeDataProvider 实例
   * 
   * @param clientManager 语言客户端管理器
   * @param context 扩展上下文
   */
  constructor(clientManager: LanguageClientManager, context: vscode.ExtensionContext) {
    this.clientManager = clientManager;
    this.context = context;

    // 监听文档保存
    vscode.workspace.onDidSaveTextDocument(doc => {
      if (doc.languageId === 'rust') {
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
      this.staticJobs = [];
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
      const response = await this.clientManager.sendRequest<JobsResponse>(
        'summer/jobs',
        { appPath }
      );

      this.staticJobs = response?.jobs || [];
      console.log(`Loaded ${this.staticJobs.length} jobs from static analysis (${appPath})`);
      this._onDidChangeTreeData.fire(undefined);
    } catch (error) {
      console.error('Failed to load static jobs:', error);
      this.staticJobs = [];
      this._onDidChangeTreeData.fire(undefined);
    }
  }

  /**
   * 刷新任务列表（兼容旧接口）
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
      const response = await fetch(`http://localhost:${app.port}/_debug/jobs`);
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

  /**
   * 清除运行时信息
   */
  private clearRuntime(): void {
    this.runtimeJobs = [];
    this.currentApp = undefined;
    this._onDidChangeTreeData.fire(undefined);
  }

  /**
   * 获取树节点
   * 
   * @param element 树节点元素
   * @returns 树节点
   */
  public getTreeItem(element: JobTreeItem): vscode.TreeItem {
    return element;
  }

  /**
   * 获取子节点
   * 
   * @param element 父节点，如果为 undefined 表示根节点
   * @returns 子节点列表
   */
  public async getChildren(element?: JobTreeItem): Promise<JobTreeItem[]> {
    if (element) {
      return [];
    }

    // 优先使用运行时信息，否则使用静态分析结果
    const jobs = this.runtimeJobs.length > 0 ? this.runtimeJobs : this.staticJobs;

    if (jobs.length === 0) {
      return [];
    }

    // 根节点：显示所有任务
    return jobs.map((job) => new JobTreeItem(job, this.context));
  }

  /**
   * 获取任务定义位置
   * 
   * @param job 任务实例
   * @returns 位置信息
   */
  public getJobLocation(job: Job): vscode.Location | undefined {
    if (!job.location) {
      return undefined;
    }

    const uri = vscode.Uri.parse(job.location.uri);
    const range = new vscode.Range(
      job.location.range.start.line,
      job.location.range.start.character,
      job.location.range.end.line,
      job.location.range.end.character
    );

    return new vscode.Location(uri, range);
  }
}

/**
 * 任务树节点
 */
export class JobTreeItem extends vscode.TreeItem {
  /**
   * 任务实例
   */
  public readonly job: Job;

  /**
   * 扩展上下文
   */
  private readonly context: vscode.ExtensionContext;

  /**
   * 创建任务树节点
   * 
   * @param job 任务实例
   * @param context 扩展上下文
   */
  constructor(job: Job, context: vscode.ExtensionContext) {
    super(job.name, vscode.TreeItemCollapsibleState.None);

    this.job = job;
    this.context = context;

    // 设置上下文值（用于菜单显示）
    this.contextValue = 'summer:job';

    // 设置工具提示
    this.tooltip = this.buildTooltip();

    // 设置描述
    this.description = this.getDescription();

    // 设置图标（使用 SVG 文件）
    this.iconPath = this.getIcon();

    // 设置点击命令（跳转到处理器）
    this.command = {
      command: 'summer.job.navigate',
      title: 'Go to Handler',
      arguments: [job.location],
    };
  }

  /**
   * 构建工具提示
   */
  private buildTooltip(): vscode.MarkdownString {
    const tooltip = new vscode.MarkdownString();
    tooltip.isTrusted = true;

    tooltip.appendMarkdown(`### ${this.job.name}\n\n`);
    tooltip.appendMarkdown(`**Type:** ${this.job.jobType}\n\n`);
    tooltip.appendMarkdown(`**Schedule:** \`${this.job.schedule}\`\n\n`);

    if (this.job.jobType === 'Cron') {
      tooltip.appendMarkdown(`\n*Cron expression*\n`);
    } else if (this.job.jobType === 'FixDelay') {
      tooltip.appendMarkdown(`\n*Runs after completion*\n`);
    } else if (this.job.jobType === 'FixRate') {
      tooltip.appendMarkdown(`\n*Runs at fixed rate*\n`);
    }

    tooltip.appendMarkdown(`\n*Click to go to handler*`);

    return tooltip;
  }

  /**
   * 获取描述
   */
  private getDescription(): string {
    return `${this.job.jobType} • ${this.job.schedule}`;
  }

  /**
   * 获取图标（使用 SVG 文件）
   */
  private getIcon(): vscode.Uri {
    // 使用 job.svg 图标文件
    return vscode.Uri.joinPath(
      this.context.extensionUri,
      'resources',
      'icons',
      'job.svg'
    );
  }
}
