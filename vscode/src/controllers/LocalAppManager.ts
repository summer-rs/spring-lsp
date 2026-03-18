import * as vscode from 'vscode';
import * as path from 'path';
import * as toml from '@iarna/toml';
import { SummerApp, AppState } from '../models';
import { debounce } from '../utils/debounce';

/**
 * 本地应用管理器
 * 
 * 负责检测和管理工作空间中的 Summer RS 应用
 */
export class LocalAppManager implements vscode.Disposable {
  /**
   * 应用列表（key 为应用路径）
   */
  private apps: Map<string, SummerApp> = new Map();

  /**
   * 文件系统监听器
   */
  private watcher: vscode.FileSystemWatcher | undefined;

  /**
   * 防抖的刷新函数
   */
  private refreshDebounced: () => void;

  /**
   * 应用列表变化事件发射器
   */
  private readonly _onDidChangeApps = new vscode.EventEmitter<SummerApp | undefined>();

  /**
   * 应用列表变化事件
   * 
   * 当应用列表发生变化时触发，用于通知视图刷新
   */
  public readonly onDidChangeApps: vscode.Event<SummerApp | undefined> =
    this._onDidChangeApps.event;

  /**
   * 创建 LocalAppManager 实例
   */
  constructor() {
    // 创建防抖的刷新函数（500ms 延迟）
    this.refreshDebounced = debounce(() => {
      this.scanWorkspace();
    }, 500);
  }

  /**
   * 初始化应用管理器
   * 
   * 启动工作空间扫描和文件监听
   */
  public async initialize(): Promise<void> {
    await this.startAppListSynchronization();
  }

  /**
   * 获取所有应用列表
   * 
   * @returns 应用列表，按名称排序
   */
  public getAppList(): SummerApp[] {
    return Array.from(this.apps.values()).sort((a, b) =>
      a.name.toLowerCase().localeCompare(b.name.toLowerCase())
    );
  }

  /**
   * 根据路径获取应用
   * 
   * @param path 应用路径
   * @returns 应用实例，如果不存在返回 undefined
   */
  public getAppByPath(path: string): SummerApp | undefined {
    return this.apps.get(path);
  }

  /**
   * 根据调试会话查找应用
   * 
   * @param session 调试会话
   * @returns 应用实例，如果不存在返回 undefined
   */
  public getAppBySession(session: vscode.DebugSession): SummerApp | undefined {
    return this.getAppList().find(
      (app) =>
        app.activeSessionName === session.name ||
        app.name === session.configuration.name
    );
  }

  /**
   * 根据 PID 查找应用
   * 
   * @param pid 进程 ID
   * @returns 应用实例，如果不存在返回 undefined
   */
  public getAppByPid(pid: number): SummerApp | undefined {
    return this.getAppList().find((app) => app.pid === pid);
  }

  /**
   * 触发应用列表变化事件
   * 
   * @param element 变化的应用，如果为 undefined 表示整个列表变化
   */
  public fireDidChangeApps(element: SummerApp | undefined): void {
    this._onDidChangeApps.fire(element);
  }

  /**
   * 启动应用列表同步
   * 
   * 执行初始扫描并设置文件监听
   */
  private async startAppListSynchronization(): Promise<void> {
    // 执行初始扫描
    await this.scanWorkspace();

    // 创建文件监听器，监听 Cargo.toml 文件的变化
    this.watcher = vscode.workspace.createFileSystemWatcher('**/Cargo.toml');

    // 监听文件创建
    this.watcher.onDidCreate(() => {
      this.refreshDebounced();
    });

    // 监听文件修改
    this.watcher.onDidChange(() => {
      this.refreshDebounced();
    });

    // 监听文件删除
    this.watcher.onDidDelete(() => {
      this.refreshDebounced();
    });
  }

  /**
   * 扫描工作空间查找 Summer RS 应用
   * 
   * 查找所有 Cargo.toml 文件并解析，识别 Summer RS 应用
   */
  private async scanWorkspace(): Promise<void> {
    try {
      // 使用 vscode.workspace.findFiles 查找所有 Cargo.toml 文件
      // 排除 target 和 node_modules 目录
      const cargoFiles = await vscode.workspace.findFiles(
        '**/Cargo.toml',
        '**/{target,node_modules}/**'
      );

      // 创建新的应用映射
      const newApps = new Map<string, SummerApp>();

      // 解析每个 Cargo.toml 文件
      for (const file of cargoFiles) {
        const app = await this.parseCargoToml(file);
        
        if (app && this.isSummerRsApp(app)) {
          // 保留现有应用的运行时状态
          const existing = this.apps.get(app.path);
          if (existing) {
            // 保留状态信息
            app.state = existing.state;
            app.pid = existing.pid;
            app.port = existing.port;
            app.profile = existing.profile;
            app.activeSessionName = existing.activeSessionName;
            app.contextPath = existing.contextPath;
          }
          
          newApps.set(app.path, app);
        }
      }

      // 更新应用列表
      this.apps = newApps;

      // 触发变化事件
      this.fireDidChangeApps(undefined);
    } catch (error) {
      console.error('Failed to scan workspace:', error);
      vscode.window.showErrorMessage(
        `Failed to scan workspace for Summer RS apps: ${error}`
      );
    }
  }

  /**
   * 解析 Cargo.toml 文件
   * 
   * @param file Cargo.toml 文件的 URI
   * @returns SummerApp 实例，如果解析失败或不是有效的应用返回 null
   */
  private async parseCargoToml(file: vscode.Uri): Promise<SummerApp | null> {
    try {
      // 读取文件内容
      const content = await vscode.workspace.fs.readFile(file);
      const contentStr = Buffer.from(content).toString('utf8');

      // 解析 TOML
      const cargoToml = toml.parse(contentStr) as any;

      // 检查是否有 package 部分
      if (!cargoToml.package || !cargoToml.package.name) {
        return null;
      }

      // 提取应用信息
      const packageName = cargoToml.package.name as string;
      const version = (cargoToml.package.version as string) || '0.1.0';
      const dirPath = path.dirname(file.fsPath);

      // 提取依赖
      const dependencies = this.extractDependencies(cargoToml);

      // 检查是否是可执行 crate
      if (!this.isExecutableCrate(cargoToml, dirPath)) {
        return null;
      }

      // 创建 SummerApp 实例
      return new SummerApp(dirPath, packageName, version, dependencies, AppState.INACTIVE);
    } catch (error) {
      console.error(`Failed to parse ${file.fsPath}:`, error);
      return null;
    }
  }

  /**
   * 从 Cargo.toml 提取依赖列表
   * 
   * @param cargoToml 解析后的 Cargo.toml 对象
   * @returns 依赖名称列表
   */
  private extractDependencies(cargoToml: any): string[] {
    const deps: string[] = [];

    // 提取 dependencies
    if (cargoToml.dependencies) {
      deps.push(...Object.keys(cargoToml.dependencies));
    }

    // 提取 dev-dependencies
    if (cargoToml['dev-dependencies']) {
      deps.push(...Object.keys(cargoToml['dev-dependencies']));
    }

    // 提取 build-dependencies
    if (cargoToml['build-dependencies']) {
      deps.push(...Object.keys(cargoToml['build-dependencies']));
    }

    return deps;
  }

  /**
   * 检查是否是可执行 crate
   * 
   * 检查是否有 [[bin]] 部分或 src/main.rs 文件
   * 
   * @param cargoToml 解析后的 Cargo.toml 对象
   * @param dirPath 应用目录路径
   * @returns 如果是可执行 crate 返回 true
   */
  private isExecutableCrate(cargoToml: any, dirPath: string): boolean {
    // 检查是否有 [[bin]] 部分
    if (cargoToml.bin && Array.isArray(cargoToml.bin) && cargoToml.bin.length > 0) {
      return true;
    }

    // 检查是否只有 [lib] 且没有 [[bin]]
    if (cargoToml.lib && !cargoToml.bin) {
      // 只有 library，不是可执行应用
      return false;
    }

    // 检查是否存在 src/main.rs
    try {
      const mainRsPath = path.join(dirPath, 'src', 'main.rs');
      const fs = require('fs');
      return fs.existsSync(mainRsPath);
    } catch {
      return false;
    }
  }

  /**
   * 检查应用是否是 Summer RS 应用
   * 
   * 通过检查依赖来判断
   * 
   * @param app SummerApp 实例
   * @returns 如果是 Summer RS 应用返回 true
   */
  private isSummerRsApp(app: SummerApp): boolean {
    return app.isSummerRsApp();
  }

  /**
   * 清理资源
   */
  public dispose(): void {
    this.watcher?.dispose();
    this._onDidChangeApps.dispose();
    this.apps.clear();
  }
}
