import * as vscode from 'vscode';
import * as path from 'path';
import * as toml from '@iarna/toml';
import { spawn, ChildProcess } from 'child_process';
import { SummerApp, AppState } from '../models';
import { LocalAppManager } from './LocalAppManager';

/**
 * 运行中的应用进程信息
 */
interface RunningProcess {
  process: ChildProcess;
  terminal: vscode.Terminal;
  pid?: number;
}

/**
 * 本地应用控制器
 * 
 * 负责应用的启动、停止、调试和生命周期管理
 */
export class LocalAppController implements vscode.Disposable {
  /**
   * 扩展上下文
   */
  private readonly context: vscode.ExtensionContext;

  /**
   * 应用管理器
   */
  private readonly manager: LocalAppManager;

  /**
   * 运行中的进程映射（key 为应用路径）
   */
  private readonly runningProcesses: Map<string, RunningProcess> = new Map();

  /**
   * 一次性资源列表
   */
  private readonly disposables: vscode.Disposable[] = [];

  /**
   * 创建 LocalAppController 实例
   * 
   * @param manager 应用管理器
   * @param context 扩展上下文
   */
  constructor(manager: LocalAppManager, context: vscode.ExtensionContext) {
    this.manager = manager;
    this.context = context;

    // 监听终端关闭事件
    this.disposables.push(
      vscode.window.onDidCloseTerminal((terminal) => {
        this.onTerminalClosed(terminal);
      })
    );
  }

  /**
   * 获取应用列表
   */
  public getAppList(): SummerApp[] {
    return this.manager.getAppList();
  }

  /**
   * 运行应用
   * 
   * @param app 要运行的应用
   * @param debug 是否以调试模式运行（默认 false）
   * @param profile 要使用的 Profile（可选）
   */
  public async runApp(
    app: SummerApp,
    debug: boolean = false,
    profile?: string
  ): Promise<void> {
    // 检查应用状态
    if (app.state !== AppState.INACTIVE) {
      vscode.window.showWarningMessage(
        `App ${app.name} is already ${app.state}`
      );
      return;
    }

    // 更新状态为 LAUNCHING
    this.setState(app, AppState.LAUNCHING);

    try {
      // 构建环境变量
      const env = this.buildEnv(app, profile);
      
      // 构建命令参数
      const args = debug ? ['run'] : ['run', '--release'];
      
      // 创建终端用于显示输出
      const terminalName = `Summer RS: ${app.name}`;
      const writeEmitter = new vscode.EventEmitter<string>();
      const pty: vscode.Pseudoterminal = {
        onDidWrite: writeEmitter.event,
        open: () => {
          writeEmitter.fire(`\x1b[1;32m[Summer RS]\x1b[0m Starting ${app.name}...\r\n`);
          writeEmitter.fire(`\x1b[1;34m[Command]\x1b[0m cargo ${args.join(' ')}\r\n`);
          writeEmitter.fire(`\x1b[1;34m[Directory]\x1b[0m ${app.path}\r\n\r\n`);
        },
        close: () => {},
      };
      
      const terminal = vscode.window.createTerminal({
        name: terminalName,
        pty,
      });
      
      // 显示终端
      terminal.show();
      
      // 启动进程
      const childProcess = spawn('cargo', args, {
        cwd: app.path,
        env: { ...process.env, ...env },
        shell: false,
      });
      
      // 获取 PID
      const pid = childProcess.pid;
      if (pid) {
        app.pid = pid;
        writeEmitter.fire(`\x1b[1;32m[PID]\x1b[0m ${pid}\r\n\r\n`);
      }
      
      // 监听标准输出
      childProcess.stdout?.on('data', (data: Buffer) => {
        const text = data.toString();
        writeEmitter.fire(text.replace(/\n/g, '\r\n'));
        
        // 检测端口信息
        this.detectPortFromOutput(app, text);
      });
      
      // 监听标准错误
      childProcess.stderr?.on('data', (data: Buffer) => {
        const text = data.toString();
        writeEmitter.fire(`\x1b[1;31m${text.replace(/\n/g, '\r\n')}\x1b[0m`);
      });
      
      // 监听进程退出
      childProcess.on('exit', (code: number | null, signal: string | null) => {
        const exitMessage = signal 
          ? `Process terminated by signal ${signal}`
          : `Process exited with code ${code}`;
        
        writeEmitter.fire(`\r\n\x1b[1;33m[Exit]\x1b[0m ${exitMessage}\r\n`);
        
        // 清理进程信息
        this.runningProcesses.delete(app.path);
        
        // 重置应用状态
        app.reset();
        this.setState(app, AppState.INACTIVE);
        
        // 根据退出码决定是否自动关闭终端
        if (code === 0) {
          // 正常退出（用户主动停止），2秒后关闭终端
          writeEmitter.fire(`\x1b[1;32mTerminal will close in 2 seconds...\x1b[0m\r\n`);
          setTimeout(() => {
            terminal.dispose();
          }, 2000);
        } else {
          // 异常退出（编译失败、运行时错误等），保持终端打开
          writeEmitter.fire(`\x1b[1;31mTerminal will remain open. Close it manually when done.\x1b[0m\r\n`);
          // 不关闭终端，让用户查看错误信息
        }
      });
      
      // 监听进程错误
      childProcess.on('error', (error: Error) => {
        writeEmitter.fire(`\r\n\x1b[1;31m[Error]\x1b[0m ${error.message}\r\n`);
        
        // 清理进程信息
        this.runningProcesses.delete(app.path);
        
        // 重置应用状态
        app.reset();
        this.setState(app, AppState.INACTIVE);
        
        vscode.window.showErrorMessage(`Failed to start ${app.name}: ${error.message}`);
      });
      
      // 保存进程信息
      this.runningProcesses.set(app.path, {
        process: childProcess,
        terminal,
        pid,
      });
      
      // 保存会话信息
      app.activeSessionName = terminalName;
      app.profile = profile;
      
      // 注意：不在这里设置 RUNNING 状态
      // 状态会在检测到端口输出时自动更新为 RUNNING
      // 或者在进程退出时更新为 INACTIVE
      
    } catch (error) {
      this.setState(app, AppState.INACTIVE);
      vscode.window.showErrorMessage(
        `Failed to start ${app.name}: ${error}`
      );
    }
  }
  
  /**
   * 检查进程是否在运行
   */
  private isProcessRunning(pid?: number): boolean {
    if (!pid) {
      return false;
    }
    
    try {
      // 发送信号 0 检查进程是否存在
      process.kill(pid, 0);
      return true;
    } catch {
      return false;
    }
  }
  
  /**
   * 从输出中检测端口
   */
  private detectPortFromOutput(app: SummerApp, output: string): void {
    // 匹配常见的端口输出格式
    const patterns = [
      /listening on .*:(\d+)/i,
      /server.*on.*:(\d+)/i,
      /http:\/\/[^:]+:(\d+)/i,
      /0\.0\.0\.0:(\d+)/,
      /127\.0\.0\.1:(\d+)/,
      /localhost:(\d+)/,
    ];
    
    for (const pattern of patterns) {
      const match = output.match(pattern);
      if (match && match[1]) {
        const port = parseInt(match[1], 10);
        if (port > 0 && port < 65536 && !app.port) {
          app.port = port;
          
          // 检测到端口说明应用已经启动完成，更新状态为 RUNNING
          if (app.state === AppState.LAUNCHING) {
            this.setState(app, AppState.RUNNING);
          }
          
          this.manager.fireDidChangeApps(app);
          break;
        }
      }
    }
  }
  
  /**
   * 终端关闭回调
   */
  private onTerminalClosed(terminal: vscode.Terminal): void {
    // 查找对应的应用
    for (const [appPath, processInfo] of this.runningProcesses.entries()) {
      if (processInfo.terminal === terminal) {
        // 终止进程
        if (processInfo.pid && this.isProcessRunning(processInfo.pid)) {
          try {
            process.kill(processInfo.pid, 'SIGTERM');
          } catch (error) {
            console.error('Failed to kill process:', error);
          }
        }
        
        // 清理进程信息
        this.runningProcesses.delete(appPath);
        
        // 更新应用状态
        const app = this.manager.getAppByPath(appPath);
        if (app) {
          app.reset();
          this.setState(app, AppState.INACTIVE);
        }
        
        break;
      }
    }
  }

  /**
   * 使用 Profile 运行应用
   * 
   * 显示 Profile 选择菜单，然后运行应用
   * 
   * @param app 要运行的应用
   * @param debug 是否以调试模式运行（默认 false）
   */
  public async runAppWithProfile(
    app: SummerApp,
    debug: boolean = false
  ): Promise<void> {
    // 检测可用的 Profiles
    const profiles = await this.detectProfiles(app);

    if (profiles.length === 0) {
      vscode.window.showInformationMessage(
        'No profiles detected. Running with default profile.'
      );
      await this.runApp(app, debug);
      return;
    }

    // 显示 Profile 选择菜单
    const selectedProfiles = await vscode.window.showQuickPick(profiles, {
      canPickMany: true,
      title: 'Select Active Profiles',
      placeHolder: 'Will set SUMMER_ENV environment variable',
    });

    if (selectedProfiles !== undefined) {
      const profileArg = selectedProfiles.join(',');
      await this.runApp(app, debug, profileArg);
    }
  }

  /**
   * 停止应用
   * 
   * @param app 要停止的应用
   */
  public async stopApp(app: SummerApp): Promise<void> {
    if (app.state === AppState.INACTIVE) {
      return;
    }

    // 更新状态为 STOPPING
    this.setState(app, AppState.STOPPING);

    try {
      // 查找对应的进程
      const processInfo = this.runningProcesses.get(app.path);

      if (processInfo) {
        const { process: childProcess, terminal, pid } = processInfo;
        
        // 尝试优雅地终止进程
        if (pid && this.isProcessRunning(pid)) {
          try {
            // 先发送 SIGTERM
            process.kill(pid, 'SIGTERM');
            
            // 等待进程退出
            await new Promise<void>((resolve) => {
              const checkInterval = setInterval(() => {
                if (!this.isProcessRunning(pid)) {
                  clearInterval(checkInterval);
                  resolve();
                }
              }, 100);
              
              // 超时后强制终止
              setTimeout(() => {
                clearInterval(checkInterval);
                if (this.isProcessRunning(pid)) {
                  try {
                    process.kill(pid, 'SIGKILL');
                  } catch (error) {
                    console.error('Failed to force kill process:', error);
                  }
                }
                resolve();
              }, 5000);
            });
          } catch (error) {
            console.error('Failed to stop process:', error);
          }
        }
        
        // 关闭终端
        terminal.dispose();
        
        // 清理进程信息
        this.runningProcesses.delete(app.path);
      }

      // 重置应用状态
      app.reset();
      this.setState(app, AppState.INACTIVE);
      
    } catch (error) {
      vscode.window.showErrorMessage(
        `Failed to stop ${app.name}: ${error}`
      );
      // 即使失败也重置状态
      app.reset();
      this.setState(app, AppState.INACTIVE);
    }
  }

  /**
   * 在浏览器中打开应用
   * 
   * @param app 要打开的应用
   */
  public async openApp(app: SummerApp): Promise<void> {
    if (app.state !== AppState.RUNNING) {
      vscode.window.showWarningMessage('App is not running');
      return;
    }

    try {
      // 获取端口和上下文路径
      const port = app.port || (await this.detectPort(app));
      const contextPath = app.contextPath || '';

      if (!port) {
        vscode.window.showErrorMessage(
          "Couldn't determine port app is running on"
        );
        return;
      }

      // 构建 URL
      const config = vscode.workspace.getConfiguration('summer-rs');
      const urlTemplate = config.get<string>(
        'openUrl',
        'http://localhost:{port}{contextPath}'
      );

      const url = urlTemplate
        .replace('{port}', port.toString())
        .replace('{contextPath}', contextPath);

      // 打开浏览器
      const openWith = config.get<string>('openWith', 'integrated');
      const browserCommand =
        openWith === 'external' ? 'vscode.open' : 'simpleBrowser.api.open';

      let uri = vscode.Uri.parse(url);
      uri = await vscode.env.asExternalUri(uri); // 支持远程环境
      await vscode.commands.executeCommand(browserCommand, uri);
    } catch (error) {
      vscode.window.showErrorMessage(
        `Failed to open browser: ${error}`
      );
    }
  }

  /**
   * 批量运行应用
   * 
   * @param debug 是否以调试模式运行（默认 false）
   */
  public async runApps(debug: boolean = false): Promise<void> {
    const appList = this.getAppList();
    const inactiveApps = appList.filter((app) => app.state === AppState.INACTIVE);

    if (inactiveApps.length === 0) {
      vscode.window.showInformationMessage('No inactive apps to run');
      return;
    }

    if (inactiveApps.length === 1) {
      await this.runApp(inactiveApps[0], debug);
      return;
    }

    const selected = await vscode.window.showQuickPick(
      inactiveApps.map((app) => ({ label: app.name, app })),
      {
        canPickMany: true,
        placeHolder: `Select apps to ${debug ? 'debug' : 'run'}`,
      }
    );

    if (selected) {
      await Promise.all(selected.map((item) => this.runApp(item.app, debug)));
    }
  }

  /**
   * 批量停止应用
   */
  public async stopApps(): Promise<void> {
    const appList = this.getAppList();
    const runningApps = appList.filter((app) => app.state !== AppState.INACTIVE);

    if (runningApps.length === 0) {
      vscode.window.showInformationMessage('No running apps to stop');
      return;
    }

    if (runningApps.length === 1) {
      await this.stopApp(runningApps[0]);
      return;
    }

    const selected = await vscode.window.showQuickPick(
      runningApps.map((app) => ({ label: app.name, app })),
      {
        canPickMany: true,
        placeHolder: 'Select apps to stop',
      }
    );

    if (selected) {
      await Promise.all(selected.map((item) => this.stopApp(item.app)));
    }
  }

  /**
   * 调试会话启动回调（保留用于未来的真正调试支持）
   * 
   * @param session 调试会话
   */
  public onDidStartApp(session: vscode.DebugSession): void {
    // 预留给未来的调试功能
  }

  /**
   * 调试会话终止回调（保留用于未来的真正调试支持）
   * 
   * @param session 调试会话
   */
  public onDidStopApp(session: vscode.DebugSession): void {
    // 预留给未来的调试功能
  }

  /**
   * 创建调试配置
   * 
   * @param app 应用
   * @param debug 是否调试模式
   * @param profile Profile 名称
   * @returns 调试配置，如果创建失败返回 null
   */
  private async createDebugConfiguration(
    app: SummerApp,
    debug: boolean,
    profile?: string
  ): Promise<vscode.DebugConfiguration | null> {
    // 检查是否已有配置
    const existingConfig = await this.findExistingLaunchConfig(app);
    if (existingConfig) {
      return this.enhanceConfig(existingConfig, debug, profile);
    }

    // 创建新配置
    const config: vscode.DebugConfiguration = {
      type: 'lldb',
      request: 'launch',
      name: `Run ${app.name}`,
      cargo: {
        args: ['run', '--manifest-path', `${app.path}/Cargo.toml`],
      },
      cwd: app.path,
      env: this.buildEnv(app, profile),
    };

    return config;
  }

  /**
   * 增强现有配置
   * 
   * @param config 现有配置
   * @param debug 是否调试模式
   * @param profile Profile 名称
   * @returns 增强后的配置
   */
  private enhanceConfig(
    config: vscode.DebugConfiguration,
    debug: boolean,
    profile?: string
  ): vscode.DebugConfiguration {
    const enhanced = { ...config };
    enhanced.noDebug = !debug;

    if (profile) {
      enhanced.env = {
        ...enhanced.env,
        SUMMER_ENV: profile,
      };
    }

    return enhanced;
  }

  /**
   * 查找现有的 launch.json 配置
   * 
   * @param app 应用
   * @returns 配置对象，如果不存在返回 undefined
   */
  private async findExistingLaunchConfig(
    app: SummerApp
  ): Promise<vscode.DebugConfiguration | undefined> {
    const launchConfig = vscode.workspace.getConfiguration(
      'launch',
      vscode.Uri.file(app.path)
    );

    const configs: vscode.DebugConfiguration[] = launchConfig.get(
      'configurations',
      []
    );
    return configs.find(
      (c) => c.name === `Run ${app.name}` || c.name === app.name
    );
  }

  /**
   * 构建环境变量
   * 
   * @param app 应用
   * @param profile Profile 名称
   * @returns 环境变量对象
   */
  private buildEnv(
    app: SummerApp,
    profile?: string
  ): Record<string, string> {
    const env: Record<string, string> = {};

    if (profile) {
      env.SUMMER_ENV = profile;
    }

    // 从配置读取额外的环境变量
    const config = vscode.workspace.getConfiguration('summer-rs');
    const envVars = config.get<Record<string, string>>('env', {});
    Object.assign(env, envVars);

    return env;
  }

  /**
   * 检测可用的 Profiles
   * 
   * @param app 应用
   * @returns Profile 名称列表
   */
  private async detectProfiles(app: SummerApp): Promise<string[]> {
    const configDir = path.join(app.path, 'config');
    const profilePattern = /^app-(.*)\.toml$/;
    const profiles: string[] = [];

    try {
      const uri = vscode.Uri.file(configDir);
      const entries = await vscode.workspace.fs.readDirectory(uri);

      for (const [name, type] of entries) {
        if (type === vscode.FileType.File) {
          const match = profilePattern.exec(name);
          if (match) {
            profiles.push(match[1]);
          }
        }
      }
    } catch (error) {
      console.log('Failed to detect profiles:', error);
    }

    return profiles;
  }

  /**
   * 检测应用端口
   * 
   * @param app 应用
   * @returns 端口号，如果检测失败返回 undefined
   */
  private async detectPort(app: SummerApp): Promise<number | undefined> {
    const configPath = path.join(app.path, 'config', 'app.toml');

    try {
      const content = await vscode.workspace.fs.readFile(
        vscode.Uri.file(configPath)
      );
      const config = toml.parse(content.toString()) as any;
      return config.web?.port || 8080;
    } catch {
      return 8080; // 默认端口
    }
  }

  /**
   * 终止进程
   * 
   * @param pid 进程 ID
   */
  private async killProcess(pid: number): Promise<void> {
    const { exec } = require('child_process');
    const command =
      process.platform === 'win32'
        ? `taskkill /F /PID ${pid}`
        : `kill -9 ${pid}`;

    return new Promise<void>((resolve) => {
      exec(command, () => resolve());
    });
  }

  /**
   * 设置应用状态
   * 
   * @param app 应用
   * @param state 新状态
   */
  private setState(app: SummerApp, state: AppState): void {
    app.state = state;
    this.manager.fireDidChangeApps(app);
  }

  /**
   * 清理资源
   */
  public dispose(): void {
    // 停止所有运行中的应用
    for (const [appPath, processInfo] of this.runningProcesses.entries()) {
      if (processInfo.pid && this.isProcessRunning(processInfo.pid)) {
        try {
          process.kill(processInfo.pid, 'SIGTERM');
        } catch (error) {
          console.error('Failed to kill process on dispose:', error);
        }
      }
      processInfo.terminal.dispose();
    }
    
    this.runningProcesses.clear();
    this.disposables.forEach((d) => d.dispose());
  }
}
