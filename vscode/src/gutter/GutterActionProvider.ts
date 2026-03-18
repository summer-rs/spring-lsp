import * as vscode from 'vscode';

/**
 * Gutter 快速操作提供者
 * 提供点击 Gutter 图标时的快速操作菜单
 */
export class GutterActionProvider {
  constructor(private context: vscode.ExtensionContext) {}

  /**
   * 显示组件快速操作菜单
   */
  public async showComponentActions(document: vscode.TextDocument, line: number): Promise<void> {
    const structName = this.findStructName(document, line);
    if (!structName) {
      vscode.window.showInformationMessage('Could not find component name');
      return;
    }

    const actions = [
      {
        label: '$(go-to-file) Go to Definition',
        description: `Jump to ${structName} definition`,
        action: 'goToDefinition',
      },
      {
        label: '$(references) Find All References',
        description: `Find all usages of ${structName}`,
        action: 'findReferences',
      },
      {
        label: '$(type-hierarchy-sub) Show Dependencies',
        description: `Show components that ${structName} depends on`,
        action: 'showDependencies',
      },
    ];

    const selected = await vscode.window.showQuickPick(actions, {
      placeHolder: `Actions for component: ${structName}`,
    });

    if (selected) {
      await this.executeAction(selected.action, document, line, structName);
    }
  }

  /**
   * 显示路由快速操作菜单
   */
  public async showRouteActions(document: vscode.TextDocument, line: number): Promise<void> {
    const functionName = this.findFunctionName(document, line);
    const routeInfo = this.extractRouteInfo(document, line);

    if (!functionName) {
      vscode.window.showInformationMessage('Could not find route handler');
      return;
    }

    const actions = [
      {
        label: '$(go-to-file) Go to Handler',
        description: `Jump to ${functionName} function`,
        action: 'goToDefinition',
      },
      {
        label: '$(references) Find All References',
        description: `Find all references to ${functionName}`,
        action: 'findReferences',
      },
    ];

    // 如果是 GET 请求，添加在浏览器中打开的选项
    if (routeInfo && routeInfo.method === 'GET') {
      actions.push({
        label: '$(globe) Open in Browser',
        description: `Open ${routeInfo.path} in browser`,
        action: 'openInBrowser',
      });
    }

    const selected = await vscode.window.showQuickPick(actions, {
      placeHolder: `Actions for route: ${routeInfo?.method} ${routeInfo?.path}`,
    });

    if (selected) {
      await this.executeAction(selected.action, document, line, functionName, routeInfo);
    }
  }

  /**
   * 显示任务快速操作菜单
   */
  public async showJobActions(document: vscode.TextDocument, line: number): Promise<void> {
    const functionName = this.findFunctionName(document, line);
    const scheduleInfo = this.extractScheduleInfo(document, line);

    if (!functionName) {
      vscode.window.showInformationMessage('Could not find job function');
      return;
    }

    const actions = [
      {
        label: '$(go-to-file) Go to Function',
        description: `Jump to ${functionName} function`,
        action: 'goToDefinition',
      },
      {
        label: '$(references) Find All References',
        description: `Find all references to ${functionName}`,
        action: 'findReferences',
      },
      {
        label: '$(info) Show Schedule Info',
        description: scheduleInfo || 'View scheduling details',
        action: 'showScheduleInfo',
      },
    ];

    const selected = await vscode.window.showQuickPick(actions, {
      placeHolder: `Actions for job: ${functionName}`,
    });

    if (selected) {
      await this.executeAction(selected.action, document, line, functionName, { schedule: scheduleInfo });
    }
  }

  /**
   * 显示配置结构快速操作菜单 - 新增
   */
  public async showConfigurationActions(document: vscode.TextDocument, line: number): Promise<void> {
    const structName = this.findStructName(document, line);
    const configPrefix = this.findConfigPrefix(document, line);

    if (!structName) {
      vscode.window.showInformationMessage('Could not find configuration struct name');
      return;
    }

    const actions = [
      {
        label: '$(go-to-file) Go to Definition',
        description: `Jump to ${structName} definition`,
        action: 'goToDefinition',
      },
      {
        label: '$(references) Find All References',
        description: `Find all usages of ${structName}`,
        action: 'findReferences',
      },
      {
        label: '$(file-code) Copy Config Example',
        description: configPrefix ? `Generate example for [${configPrefix}]` : 'Generate configuration example',
        action: 'copyConfigExample',
      },
      {
        label: '$(list-tree) Show in Configurations View',
        description: 'Reveal in Configurations tree view',
        action: 'showInConfigView',
      },
    ];

    const selected = await vscode.window.showQuickPick(actions, {
      placeHolder: `Actions for configuration: ${structName}${configPrefix ? ` [${configPrefix}]` : ''}`,
    });

    if (selected) {
      await this.executeAction(selected.action, document, line, structName, { 
        configPrefix,
        structName 
      });
    }
  }

  /**
   * 执行操作
   */
  private async executeAction(
    action: string,
    document: vscode.TextDocument,
    line: number,
    name: string,
    extra?: any
  ): Promise<void> {
    const position = new vscode.Position(line, 0);

    switch (action) {
      case 'goToDefinition':
        // 跳转到定义
        await vscode.commands.executeCommand('editor.action.revealDefinition', document.uri, position);
        break;

      case 'findReferences':
        // 查找所有引用
        await vscode.commands.executeCommand('editor.action.goToReferences', document.uri, position);
        break;

      case 'showDependencies':
        // 显示依赖关系（通过 Components 视图）
        await vscode.commands.executeCommand('summer.component.showDependencies', { name });
        break;

      case 'openInBrowser':
        // 在浏览器中打开路由
        if (extra && extra.path) {
          await vscode.commands.executeCommand('summer.route.open', { path: extra.path });
        }
        break;

      case 'showScheduleInfo':
        // 显示调度信息
        if (extra && extra.schedule) {
          vscode.window.showInformationMessage(`Schedule: ${extra.schedule}`);
        }
        break;

      case 'copyConfigExample':
        // 复制配置示例 - 新增
        if (extra && extra.structName) {
          await vscode.commands.executeCommand('summer.configuration.copyExample', {
            name: extra.structName,
            prefix: extra.configPrefix || extra.structName.toLowerCase()
          });
        }
        break;

      case 'showInConfigView':
        // 在配置视图中显示 - 新增
        await vscode.commands.executeCommand('summer.configuration.refresh');
        vscode.window.showInformationMessage('Configurations view refreshed');
        break;

      default:
        vscode.window.showWarningMessage(`Unknown action: ${action}`);
    }
  }

  /**
   * 查找结构体名称
   */
  private findStructName(document: vscode.TextDocument, startLine: number): string | null {
    for (let i = startLine + 1; i < Math.min(startLine + 5, document.lineCount); i++) {
      const line = document.lineAt(i).text.trim();
      const match = line.match(/^(?:pub\s+)?struct\s+(\w+)/);
      if (match) {
        return match[1];
      }
    }
    return null;
  }

  /**
   * 查找配置前缀 - 新增
   */
  private findConfigPrefix(document: vscode.TextDocument, startLine: number): string | null {
    // 向上查找 #[config_prefix = "..."]
    for (let i = startLine - 1; i >= Math.max(0, startLine - 5); i--) {
      const line = document.lineAt(i).text.trim();
      const match = line.match(/^#\[config_prefix\s*=\s*"([^"]+)"\]/);
      if (match) {
        return match[1];
      }
    }
    // 向下查找
    for (let i = startLine + 1; i < Math.min(startLine + 5, document.lineCount); i++) {
      const line = document.lineAt(i).text.trim();
      const match = line.match(/^#\[config_prefix\s*=\s*"([^"]+)"\]/);
      if (match) {
        return match[1];
      }
    }
    return null;
  }

  /**
   * 查找函数名称
   */
  private findFunctionName(document: vscode.TextDocument, startLine: number): string | null {
    for (let i = startLine + 1; i < Math.min(startLine + 5, document.lineCount); i++) {
      const line = document.lineAt(i).text.trim();
      const match = line.match(/^(?:pub\s+)?(?:async\s+)?fn\s+(\w+)/);
      if (match) {
        return match[1];
      }
    }
    return null;
  }

  /**
   * 提取路由信息
   */
  private extractRouteInfo(document: vscode.TextDocument, line: number): { method: string; path: string } | null {
    const lineText = document.lineAt(line).text.trim();

    // 提取方法
    const methodMatch = lineText.match(/^#\[(get|post|put|delete|patch|route)\(/);
    if (!methodMatch) {
      return null;
    }

    // 提取路径
    const pathMatch = lineText.match(/^#\[(?:get|post|put|delete|patch|route)\("([^"]+)"/);
    if (!pathMatch) {
      return null;
    }

    return {
      method: methodMatch[1].toUpperCase(),
      path: pathMatch[1],
    };
  }

  /**
   * 提取调度信息
   */
  private extractScheduleInfo(document: vscode.TextDocument, line: number): string | null {
    const lineText = document.lineAt(line).text.trim();

    // cron
    let match = lineText.match(/^#\[cron\("([^"]+)"/);
    if (match) {
      return `Cron: ${match[1]}`;
    }

    // fix_delay
    match = lineText.match(/^#\[fix_delay\((\d+)\)/);
    if (match) {
      return `Fixed Delay: ${match[1]} seconds`;
    }

    // fix_rate
    match = lineText.match(/^#\[fix_rate\((\d+)\)/);
    if (match) {
      return `Fixed Rate: ${match[1]} seconds`;
    }

    return null;
  }
}
