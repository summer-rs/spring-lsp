import { AppState } from './AppState';

/**
 * Summer RS 应用数据模型
 * 
 * 表示工作空间中的一个 Summer RS 应用
 */
export class SummerApp {
  /**
   * 应用目录的绝对路径
   */
  public readonly path: string;

  /**
   * 应用名称（从 Cargo.toml 的 package.name 读取）
   */
  public readonly name: string;

  /**
   * 应用版本（从 Cargo.toml 的 package.version 读取）
   */
  public readonly version: string;

  /**
   * 应用依赖列表（从 Cargo.toml 的 dependencies 读取）
   */
  public readonly dependencies: string[];

  /**
   * 当前应用状态
   */
  public state: AppState;

  /**
   * 进程 ID（仅在 RUNNING 状态时有值）
   */
  public pid?: number;

  /**
   * 监听端口（仅在 RUNNING 状态时有值）
   */
  public port?: number;

  /**
   * 当前激活的 Profile（如 dev、prod 等）
   */
  public profile?: string;

  /**
   * 调试会话名称（用于关联 VSCode 调试会话）
   */
  public activeSessionName?: string;

  /**
   * 上下文路径（如 /api）
   */
  public contextPath?: string;

  /**
   * 创建一个新的 SummerApp 实例
   * 
   * @param path 应用目录的绝对路径
   * @param name 应用名称
   * @param version 应用版本
   * @param dependencies 依赖列表
   * @param state 初始状态（默认为 INACTIVE）
   */
  constructor(
    path: string,
    name: string,
    version: string,
    dependencies: string[],
    state: AppState = AppState.INACTIVE
  ) {
    this.path = path;
    this.name = name;
    this.version = version;
    this.dependencies = dependencies;
    this.state = state;
  }

  /**
   * 重置应用状态
   * 
   * 将状态重置为 INACTIVE，并清除所有运行时信息
   */
  public reset(): void {
    this.state = AppState.INACTIVE;
    this.pid = undefined;
    this.port = undefined;
    this.profile = undefined;
    this.activeSessionName = undefined;
    // 注意：contextPath 不重置，因为它是配置的一部分
  }

  /**
   * 检查应用是否正在运行
   */
  public isRunning(): boolean {
    return this.state === AppState.RUNNING;
  }

  /**
   * 检查应用是否处于非活动状态
   */
  public isInactive(): boolean {
    return this.state === AppState.INACTIVE;
  }

  /**
   * 检查应用是否处于过渡状态（启动中或停止中）
   */
  public isTransitioning(): boolean {
    return this.state === AppState.LAUNCHING || this.state === AppState.STOPPING;
  }

  /**
   * 获取应用的显示名称
   * 
   * 如果有 profile，返回 "name (profile)"，否则返回 "name"
   */
  public getDisplayName(): string {
    if (this.profile) {
      return `${this.name} (${this.profile})`;
    }
    return this.name;
  }

  /**
   * 获取应用的完整描述
   */
  public getDescription(): string {
    const parts: string[] = [];
    
    parts.push(`v${this.version}`);
    
    if (this.state !== AppState.INACTIVE) {
      parts.push(this.state);
    }
    
    if (this.port) {
      parts.push(`port ${this.port}`);
    }
    
    return parts.join(' • ');
  }

  /**
   * 检查应用是否依赖指定的包
   * 
   * @param packageName 包名称
   * @returns 如果依赖该包返回 true
   */
  public hasDependency(packageName: string): boolean {
    return this.dependencies.some(
      (dep) => dep === packageName || dep.startsWith(`${packageName}-`)
    );
  }

  /**
   * 检查应用是否是 Summer RS 应用
   * 
   * 通过检查是否依赖 summer-rs 相关的包来判断
   */
  public isSummerRsApp(): boolean {
    const summerPackages = ['summer', 'summer-web', 'summer-sqlx', 'summer-redis'];
    return summerPackages.some((pkg) => this.hasDependency(pkg));
  }

  /**
   * 创建应用的副本
   */
  public clone(): SummerApp {
    const app = new SummerApp(this.path, this.name, this.version, [...this.dependencies], this.state);
    app.pid = this.pid;
    app.port = this.port;
    app.profile = this.profile;
    app.activeSessionName = this.activeSessionName;
    app.contextPath = this.contextPath;
    return app;
  }

  /**
   * 转换为 JSON 对象
   */
  public toJSON(): Record<string, unknown> {
    return {
      path: this.path,
      name: this.name,
      version: this.version,
      dependencies: this.dependencies,
      state: this.state,
      pid: this.pid,
      port: this.port,
      profile: this.profile,
      activeSessionName: this.activeSessionName,
      contextPath: this.contextPath,
    };
  }

  /**
   * 从 JSON 对象创建 SummerApp 实例
   */
  public static fromJSON(json: Record<string, unknown>): SummerApp {
    const app = new SummerApp(
      json.path as string,
      json.name as string,
      json.version as string,
      json.dependencies as string[],
      json.state as AppState
    );
    
    app.pid = json.pid as number | undefined;
    app.port = json.port as number | undefined;
    app.profile = json.profile as string | undefined;
    app.activeSessionName = json.activeSessionName as string | undefined;
    app.contextPath = json.contextPath as string | undefined;
    
    return app;
  }
}
