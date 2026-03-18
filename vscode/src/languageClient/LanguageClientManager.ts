import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

/**
 * 语言客户端管理器
 * 
 * 负责启动、管理和与 summer-rs 语言服务器通信
 */
export class LanguageClientManager implements vscode.Disposable {
  /**
   * 语言客户端实例
   */
  private client: LanguageClient | undefined;

  /**
   * 输出通道（用于扩展日志）
   */
  private outputChannel: vscode.OutputChannel;

  /**
   * 扩展上下文
   */
  private readonly context: vscode.ExtensionContext;

  /**
   * 创建 LanguageClientManager 实例
   * 
   * @param context 扩展上下文
   * @param outputChannel 输出通道（用于扩展日志）
   */
  constructor(context: vscode.ExtensionContext, outputChannel: vscode.OutputChannel) {
    this.context = context;
    this.outputChannel = outputChannel;
  }

  /**
   * 启动语言服务器
   */
  public async start(): Promise<void> {
    try {
      // 查找语言服务器可执行文件
      const serverPath = await this.findServerExecutable();

      if (!serverPath) {
        this.showServerNotFoundError();
        return;
      }

      this.outputChannel.appendLine(`Found Summer LSP server at: ${serverPath}`);

      // 配置服务器选项
      const serverOptions: ServerOptions = {
        command: serverPath,
        args: ['--stdio'],
        transport: TransportKind.stdio,
      };

      // 配置客户端选项
      const clientOptions: LanguageClientOptions = {
        documentSelector: [
          { scheme: 'file', language: 'toml', pattern: '**/.summer-lsp.toml' },
          { scheme: 'file', language: 'toml', pattern: '**/config/app*.toml' },
          { scheme: 'file', language: 'rust' },
        ],
        synchronize: {
          fileEvents: vscode.workspace.createFileSystemWatcher(
            '**/{*.toml,*.rs,Cargo.toml}'
          ),
        },
        // 不指定 outputChannel，让 LSP 客户端自动创建
        // 这样会创建一个名为 "Summer RS" 的输出通道用于语言服务器日志
      };

      // 创建语言客户端
      this.client = new LanguageClient(
        'summer-rs',
        'Summer RS',
        serverOptions,
        clientOptions
      );

      // 启动客户端
      await this.client.start();

      this.outputChannel.appendLine('Summer LSP server started successfully');
    } catch (error) {
      this.outputChannel.appendLine(`Failed to start Summer LSP server: ${error}`);
      vscode.window.showErrorMessage(
        `Failed to start Summer LSP server: ${error}`,
        'Open Settings',
        'View Documentation'
      ).then(selection => {
        if (selection === 'Open Settings') {
          vscode.commands.executeCommand(
            'workbench.action.openSettings',
            'summer-rs.serverPath'
          );
        } else if (selection === 'View Documentation') {
          vscode.env.openExternal(
            vscode.Uri.parse('https://summer-rs.github.io/')
          );
        }
      });
    }
  }

  /**
   * 停止语言服务器
   */
  public async stop(): Promise<void> {
    if (this.client) {
      this.outputChannel.appendLine('Stopping Summer LSP server...');
      await this.client.stop();
      this.client = undefined;
      this.outputChannel.appendLine('Summer LSP server stopped');
    }
  }

  /**
   * 发送自定义请求到语言服务器
   * 
   * @param method 请求方法名
   * @param params 请求参数
   * @param timeout 超时时间（毫秒），默认 30 秒
   * @returns 响应结果
   */
  public async sendRequest<T>(
    method: string,
    params: any,
    timeout: number = 30000
  ): Promise<T | null> {
    if (!this.client) {
      this.outputChannel.appendLine('Language client not initialized');
      return null;
    }

    try {
      const result = await Promise.race([
        this.client.sendRequest<T>(method, params),
        new Promise<null>((_, reject) =>
          setTimeout(() => reject(new Error('Request timeout')), timeout)
        ),
      ]);

      return result;
    } catch (error) {
      this.outputChannel.appendLine(`LSP request failed: ${method} - ${error}`);
      
      vscode.window.showWarningMessage(
        `Request to language server timed out: ${method}`,
        'Retry'
      ).then(selection => {
        if (selection === 'Retry') {
          this.sendRequest<T>(method, params, timeout);
        }
      });

      return null;
    }
  }

  /**
   * 获取语言客户端实例
   * 
   * @returns 语言客户端实例，如果未初始化返回 undefined
   */
  public getClient(): LanguageClient | undefined {
    return this.client;
  }

  /**
   * 检查语言服务器是否正在运行
   * 
   * @returns 如果正在运行返回 true
   */
  public isRunning(): boolean {
    return this.client !== undefined;
  }

  /**
   * 查找语言服务器可执行文件
   * 
   * 按以下顺序查找：
   * 1. 配置中指定的路径
   * 2. 扩展目录的 bin/ 子目录（根据平台选择）
   * 3. 系统 PATH
   * 
   * @returns 服务器可执行文件路径，如果未找到返回 undefined
   */
  private async findServerExecutable(): Promise<string | undefined> {
    // 1. 检查配置中指定的路径
    const config = vscode.workspace.getConfiguration('summer-rs');
    const configPath = config.get<string>('serverPath');

    if (configPath) {
      if (fs.existsSync(configPath)) {
        return configPath;
      } else {
        this.outputChannel.appendLine(
          `Configured server path does not exist: ${configPath}`
        );
      }
    }

    // 2. 检查扩展目录中的二进制文件（根据平台选择）
    const extensionPath = this.context.extensionPath;
    const binaryName = this.getPlatformBinaryName();
    const binaryPath = path.join(extensionPath, 'bin', binaryName);

    if (fs.existsSync(binaryPath)) {
      // 确保有执行权限（Unix 系统）
      if (process.platform !== 'win32') {
        try {
          fs.chmodSync(binaryPath, 0o755);
        } catch (error) {
          this.outputChannel.appendLine(`Failed to set execute permission: ${error}`);
        }
      }
      return binaryPath;
    }

    // 3. 检查系统 PATH（使用通用名称 summer-lsp）
    const pathResult = await this.findInPath('summer-lsp');
    if (pathResult) {
      return pathResult;
    }

    return undefined;
  }

  /**
   * 获取当前平台的二进制文件名
   * 
   * @returns 平台特定的二进制文件名
   */
  private getPlatformBinaryName(): string {
    const platform = process.platform;
    const arch = process.arch;

    if (platform === 'win32') {
      return 'summer-lsp-win32-x64.exe';
    } else if (platform === 'darwin') {
      return arch === 'arm64' 
        ? 'summer-lsp-darwin-arm64' 
        : 'summer-lsp-darwin-x64';
    } else {
      // Linux 和其他 Unix 系统
      return 'summer-lsp-linux-x64';
    }
  }

  /**
   * 在系统 PATH 中查找可执行文件
   * 
   * @param binaryName 可执行文件名
   * @returns 完整路径，如果未找到返回 undefined
   */
  private async findInPath(binaryName: string): Promise<string | undefined> {
    try {
      const { exec } = require('child_process');
      const command = process.platform === 'win32' ? 'where' : 'which';

      return new Promise<string | undefined>((resolve) => {
        exec(`${command} ${binaryName}`, (error: any, stdout: string) => {
          if (error) {
            resolve(undefined);
          } else {
            const path = stdout.trim().split('\n')[0];
            resolve(path || undefined);
          }
        });
      });
    } catch {
      return undefined;
    }
  }

  /**
   * 显示服务器未找到错误
   */
  private showServerNotFoundError(): void {
    const message = 'Summer LSP server not found. Please install it or configure the path.';
    
    vscode.window.showErrorMessage(
      message,
      'Open Settings',
      'View Documentation',
      'Install Guide'
    ).then(selection => {
      if (selection === 'Open Settings') {
        vscode.commands.executeCommand(
          'workbench.action.openSettings',
          'summer-rs.serverPath'
        );
      } else if (selection === 'View Documentation') {
        vscode.env.openExternal(
          vscode.Uri.parse('https://summer-rs.github.io/')
        );
      } else if (selection === 'Install Guide') {
        vscode.env.openExternal(
          vscode.Uri.parse('https://summer-rs.github.io/')
        );
      }
    });

    this.outputChannel.appendLine(message);
    this.outputChannel.appendLine('Search paths:');
    this.outputChannel.appendLine(`  1. Configuration: summer-rs.serverPath`);
    this.outputChannel.appendLine(`  2. Extension directory: ${this.context.extensionPath}/bin/`);
    this.outputChannel.appendLine(`  3. System PATH`);
  }

  /**
   * 清理资源
   */
  public dispose(): void {
    this.stop();
    this.outputChannel.dispose();
  }
}
