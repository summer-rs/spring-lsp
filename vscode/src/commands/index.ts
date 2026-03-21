/**
 * Commands module
 * 
 * 统一管理所有扩展命令的注册和处理
 */

import * as vscode from 'vscode';
import { LocalAppManager } from '../controllers/LocalAppManager';
import { LocalAppController } from '../controllers/LocalAppController';
import { LanguageClientManager } from '../languageClient/LanguageClientManager';
import { DependencyGraphView } from '../views/DependencyGraphView';
import { ConfigurationStruct } from '../types';
import { ComponentsTreeDataProviderEnhanced } from '../views/ComponentsTreeDataProviderEnhanced';
import { RoutesTreeDataProviderEnhanced } from '../views/RoutesTreeDataProviderEnhanced';
import { ConfigurationsTreeDataProviderEnhanced } from '../views/ConfigurationsTreeDataProviderEnhanced';
import { SpringApp } from '../models/SpringApp';

/**
 * 命令 ID 常量
 */
export const Commands = {
  // 应用操作命令
  REFRESH: 'spring-rs.refresh',
  APP_RUN: 'spring-rs.app.run',
  APP_DEBUG: 'spring-rs.app.debug',
  APP_STOP: 'spring-rs.app.stop',
  APP_OPEN: 'spring-rs.app.open',
  APP_RUN_WITH_PROFILE: 'spring-rs.app.runWithProfile',
  APP_DEBUG_WITH_PROFILE: 'spring-rs.app.debugWithProfile',
  APP_RUN_MULTIPLE: 'spring-rs.app.runMultiple',
  APP_STOP_MULTIPLE: 'spring-rs.app.stopMultiple',

  // 导航命令
  COMPONENT_NAVIGATE: 'spring.component.navigate',
  COMPONENT_SHOW_DEPENDENCIES: 'spring.component.showDependencies',
  ROUTE_NAVIGATE: 'spring.route.navigate',
  ROUTE_OPEN: 'spring.route.open',
  JOB_NAVIGATE: 'spring.job.navigate',
  PLUGIN_NAVIGATE: 'spring.plugin.navigate',
  CONFIGURATION_REFRESH: 'spring.configuration.refresh',
  CONFIGURATION_NAVIGATE: 'spring.configuration.navigate',
  CONFIGURATION_COPY_EXAMPLE: 'spring.configuration.copyExample',

  // 视图模式切换命令
  COMPONENTS_SELECT_VIEW_MODE: 'spring.components.selectViewMode',
  COMPONENTS_TOGGLE_VIEW_MODE: 'spring.components.toggleViewMode',
  COMPONENTS_SWITCH_TO_LIST_VIEW: 'spring.components.switchToListView',
  ROUTES_SELECT_VIEW_MODE: 'spring.routes.selectViewMode',
  ROUTES_TOGGLE_VIEW_MODE: 'spring.routes.toggleViewMode',
  ROUTES_SWITCH_TO_LIST_VIEW: 'spring.routes.switchToListView',
  CONFIGURATIONS_SELECT_VIEW_MODE: 'spring.configurations.selectViewMode',
  CONFIGURATIONS_TOGGLE_VIEW_MODE: 'spring.configurations.toggleViewMode',
  CONFIGURATIONS_SWITCH_TO_LIST_VIEW: 'spring.configurations.switchToListView',

  // 文档和帮助命令
  OPEN_DOCUMENTATION: 'spring-rs.openDocumentation',
  SHOW_WELCOME: 'spring-rs.showWelcome',

  // 内部命令（用于测试和调试）
  _GET_APPS: '_spring.getApps'
} as const;

/**
 * 命令处理器类
 * 
 * 负责注册和处理所有扩展命令
 */
export class CommandManager {
  private disposables: vscode.Disposable[] = [];
  private dependencyGraphView: DependencyGraphView | undefined;
  private configurationsProvider: ConfigurationsTreeDataProviderEnhanced | undefined;
  private componentsProvider: ComponentsTreeDataProviderEnhanced | undefined;
  private routesProvider: RoutesTreeDataProviderEnhanced | undefined;

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly appManager: LocalAppManager,
    private readonly appController: LocalAppController,
    private readonly languageClient: LanguageClientManager
  ) {}

  /**
   * 设置配置视图提供者（在视图注册后调用）
   */
  public setConfigurationsProvider(provider: ConfigurationsTreeDataProviderEnhanced): void {
    this.configurationsProvider = provider;
  }

  /**
   * 设置组件视图提供者（在视图注册后调用）
   */
  public setComponentsProvider(provider: ComponentsTreeDataProviderEnhanced): void {
    this.componentsProvider = provider;
  }

  /**
   * 设置路由视图提供者（在视图注册后调用）
   */
  public setRoutesProvider(provider: RoutesTreeDataProviderEnhanced): void {
    this.routesProvider = provider;
  }

  /**
   * 注册所有命令
   */
  public registerCommands(): void {
    // 注册应用操作命令
    this.registerAppCommands();

    // 注册导航命令
    this.registerNavigationCommands();

    // 注册配置视图命令
    this.registerConfigurationCommands();

    // 注册视图模式切换命令
    this.registerViewModeCommands();

    // 注册文档和帮助命令
    this.registerDocumentationCommands();

    // 注册内部命令
    this.registerInternalCommands();
  }

  /**
   * 注册应用操作命令
   */
  private registerAppCommands(): void {
    // 刷新应用列表
    this.register(Commands.REFRESH, () => {
      this.appManager.fireDidChangeApps(undefined);
      vscode.window.showInformationMessage('Spring RS apps refreshed');
    });

    // 运行应用
    this.register(Commands.APP_RUN, async (item?: any) => {
      const app = this.extractApp(item);
      if (!app) {
        const selected = await this.selectApp('Select an app to run');
        if (selected) {
          await this.appController.runApp(selected, false);
        }
      } else {
        await this.appController.runApp(app, false);
      }
    });

    // 调试应用
    this.register(Commands.APP_DEBUG, async (item?: any) => {
      const app = this.extractApp(item);
      if (!app) {
        const selected = await this.selectApp('Select an app to debug');
        if (selected) {
          await this.appController.runApp(selected, true);
        }
      } else {
        await this.appController.runApp(app, true);
      }
    });

    // 停止应用
    this.register(Commands.APP_STOP, async (item?: any) => {
      const app = this.extractApp(item);
      if (!app) {
        const selected = await this.selectApp('Select an app to stop', app => app.state !== 'inactive');
        if (selected) {
          await this.appController.stopApp(selected);
        }
      } else {
        await this.appController.stopApp(app);
      }
    });

    // 在浏览器中打开
    this.register(Commands.APP_OPEN, async (item?: any) => {
      const app = this.extractApp(item);
      if (!app) {
        const selected = await this.selectApp('Select an app to open', app => app.state === 'running');
        if (selected) {
          await this.appController.openApp(selected);
        }
      } else {
        await this.appController.openApp(app);
      }
    });

    // 使用 Profile 运行
    this.register(Commands.APP_RUN_WITH_PROFILE, async (item?: any) => {
      const app = this.extractApp(item);
      if (!app) {
        const selected = await this.selectApp('Select an app to run with profile');
        if (selected) {
          await this.appController.runAppWithProfile(selected, false);
        }
      } else {
        await this.appController.runAppWithProfile(app, false);
      }
    });

    // 使用 Profile 调试
    this.register(Commands.APP_DEBUG_WITH_PROFILE, async (item?: any) => {
      const app = this.extractApp(item);
      if (!app) {
        const selected = await this.selectApp('Select an app to debug with profile');
        if (selected) {
          await this.appController.runAppWithProfile(selected, true);
        }
      } else {
        await this.appController.runAppWithProfile(app, true);
      }
    });

    // 批量运行应用
    this.register(Commands.APP_RUN_MULTIPLE, async () => {
      await this.appController.runApps(false);
    });

    // 批量停止应用
    this.register(Commands.APP_STOP_MULTIPLE, async () => {
      await this.appController.stopApps();
    });
  }

  /**
   * 注册导航命令
   */
  private registerNavigationCommands(): void {
    // 导航到组件定义
    this.register(Commands.COMPONENT_NAVIGATE, async (itemOrLocation?: any) => {
      if (!itemOrLocation) {
        vscode.window.showWarningMessage('No location provided');
        return;
      }

      try {
        // 提取 location 对象
        // 可能是直接的 location 对象，也可能是 TreeItem（包含 component.location）
        let location = itemOrLocation;
        if (itemOrLocation.component && itemOrLocation.component.location) {
          location = itemOrLocation.component.location;
        } else if (itemOrLocation.location) {
          location = itemOrLocation.location;
        }

        if (!location || !location.uri || !location.range) {
          vscode.window.showWarningMessage('Invalid location object');
          return;
        }

        const uri = vscode.Uri.parse(location.uri);
        const range = new vscode.Range(
          location.range.start.line,
          location.range.start.character,
          location.range.end.line,
          location.range.end.character
        );

        // 打开文档并选中范围
        const editor = await vscode.window.showTextDocument(uri, {
          selection: range,
          preview: false,
          viewColumn: vscode.ViewColumn.One
        });

        // 确保范围可见并居中显示
        editor.revealRange(range, vscode.TextEditorRevealType.InCenter);
      } catch (error) {
        vscode.window.showErrorMessage(
          `Failed to navigate: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    });

    // 显示组件依赖关系
    this.register(Commands.COMPONENT_SHOW_DEPENDENCIES, async (app?: SpringApp) => {
      if (!app) {
        app = await this.selectApp('Select an app to show dependencies', app => app.state === 'running');
      }
      if (app) {
        if (!this.dependencyGraphView) {
          this.dependencyGraphView = new DependencyGraphView(
            this.context,
            this.languageClient
          );
        }
        await this.dependencyGraphView.show(app);
      }
    });

    // 导航到路由处理器
    this.register(Commands.ROUTE_NAVIGATE, async (location?: any) => {
      if (!location) {
        vscode.window.showWarningMessage('No location provided');
        return;
      }

      try {
        const uri = vscode.Uri.parse(location.uri);
        const range = new vscode.Range(
          location.range.start.line,
          location.range.start.character,
          location.range.end.line,
          location.range.end.character
        );

        // 打开文档并选中范围
        const editor = await vscode.window.showTextDocument(uri, {
          selection: range,
          preview: false,
          viewColumn: vscode.ViewColumn.One
        });

        // 确保范围可见并居中显示
        editor.revealRange(range, vscode.TextEditorRevealType.InCenter);
      } catch (error) {
        vscode.window.showErrorMessage(
          `Failed to navigate: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    });

    // 在浏览器中打开路由
    this.register(Commands.ROUTE_OPEN, async (itemOrRoute?: any) => {
      console.log('[ROUTE_OPEN] Received argument:', itemOrRoute);
      
      // 提取路由和应用信息
      let route: { path: string } | undefined;
      let app: SpringApp | undefined;

      if (!itemOrRoute) {
        vscode.window.showWarningMessage('No route or app provided');
        return;
      }

      // 如果是 RouteItem（从树视图点击）
      if (itemOrRoute.route && itemOrRoute.app) {
        route = itemOrRoute.route;
        app = itemOrRoute.app;
      }
      // 如果是直接传递的对象（包含 path 和 app）
      else if (itemOrRoute.path && itemOrRoute.app) {
        route = itemOrRoute;
        app = itemOrRoute.app;
      }
      // 无法提取
      else {
        vscode.window.showWarningMessage('Invalid route or app object');
        console.error('[ROUTE_OPEN] Invalid argument structure:', itemOrRoute);
        return;
      }

      if (!app) {
        vscode.window.showWarningMessage('No app provided');
        return;
      }

      if (!route) {
        vscode.window.showWarningMessage('No route provided');
        return;
      }

      if (app.state !== 'running') {
        vscode.window.showWarningMessage('App is not running');
        return;
      }

      try {
        // 获取端口
        const port = app.port || await this.appController['detectPort'](app);
        if (!port) {
          vscode.window.showErrorMessage("Couldn't determine port");
          return;
        }

        // 构建 URL
        const contextPath = app.contextPath || '';
        const url = `http://localhost:${port}${contextPath}${route.path}`;

        console.log('[ROUTE_OPEN] Opening URL:', url);

        // 打开浏览器
        const config = vscode.workspace.getConfiguration('spring-rs');
        const openWith = config.get<string>('openWith', 'integrated');
        const command = openWith === 'external' ? 'vscode.open' : 'simpleBrowser.api.open';

        let uri = vscode.Uri.parse(url);
        uri = await vscode.env.asExternalUri(uri);
        await vscode.commands.executeCommand(command, uri);
      } catch (error) {
        vscode.window.showErrorMessage(
          `Failed to open route: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    });

    // 导航到任务定义
    this.register(Commands.JOB_NAVIGATE, async (location?: any) => {
      if (!location) {
        vscode.window.showWarningMessage('No location provided');
        return;
      }

      try {
        const uri = vscode.Uri.parse(location.uri);
        const range = new vscode.Range(
          location.range.start.line,
          location.range.start.character,
          location.range.end.line,
          location.range.end.character
        );

        // 打开文档并选中范围
        const editor = await vscode.window.showTextDocument(uri, {
          selection: range,
          preview: false,
          viewColumn: vscode.ViewColumn.One
        });

        // 确保范围可见并居中显示
        editor.revealRange(range, vscode.TextEditorRevealType.InCenter);
      } catch (error) {
        vscode.window.showErrorMessage(
          `Failed to navigate: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    });

    // 导航到插件定义
    this.register(Commands.PLUGIN_NAVIGATE, async (itemOrLocation?: any) => {
      console.log('[PLUGIN_NAVIGATE] Received argument:', itemOrLocation);
      
      if (!itemOrLocation) {
        vscode.window.showWarningMessage('No location provided');
        return;
      }

      try {
        // 提取 location 对象
        // 可能是直接的 location 对象，也可能是 TreeItem（包含 plugin.location）
        let location = itemOrLocation;
        
        // 如果是 TreeItem（从右键菜单）
        if (itemOrLocation.plugin && itemOrLocation.plugin.location) {
          console.log('[PLUGIN_NAVIGATE] Extracting from TreeItem.plugin.location');
          location = itemOrLocation.plugin.location;
        }
        // 如果有 location 属性
        else if (itemOrLocation.location) {
          console.log('[PLUGIN_NAVIGATE] Extracting from item.location');
          location = itemOrLocation.location;
        }
        
        console.log('[PLUGIN_NAVIGATE] Final location object:', JSON.stringify(location, null, 2));
        
        if (!location || !location.uri) {
          vscode.window.showWarningMessage('Location missing uri');
          console.error('[PLUGIN_NAVIGATE] Invalid location:', location);
          return;
        }
        
        if (!location.range) {
          vscode.window.showWarningMessage('Location missing range');
          return;
        }
        
        if (!location.range.start || !location.range.end) {
          vscode.window.showWarningMessage('Location range missing start or end');
          return;
        }

        const uri = vscode.Uri.parse(location.uri);
        const range = new vscode.Range(
          location.range.start.line,
          location.range.start.character,
          location.range.end.line,
          location.range.end.character
        );

        // 打开文档并选中范围
        const editor = await vscode.window.showTextDocument(uri, {
          selection: range,
          preview: false,
          viewColumn: vscode.ViewColumn.One
        });

        // 确保范围可见并居中显示
        editor.revealRange(range, vscode.TextEditorRevealType.InCenter);
      } catch (error) {
        console.error('[PLUGIN_NAVIGATE] Error:', error);
        vscode.window.showErrorMessage(
          `Failed to navigate: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    });
  }

  /**
   * 注册配置视图命令
   */
  private registerConfigurationCommands(): void {
    // 刷新配置列表
    this.register(Commands.CONFIGURATION_REFRESH, async () => {
      if (this.configurationsProvider) {
        await this.configurationsProvider.refresh();
        vscode.window.showInformationMessage('配置列表已刷新');
      }
    });

    // 导航到配置结构定义
    this.register(Commands.CONFIGURATION_NAVIGATE, async (itemOrLocation?: any) => {
      if (!itemOrLocation) {
        vscode.window.showWarningMessage('未提供位置信息');
        return;
      }

      try {
        // 提取 location 对象
        let location = itemOrLocation;
        if (itemOrLocation.config && itemOrLocation.config.location) {
          location = itemOrLocation.config.location;
        } else if (itemOrLocation.location) {
          location = itemOrLocation.location;
        }

        if (!location || !location.uri || !location.range) {
          vscode.window.showWarningMessage('无效的位置对象');
          return;
        }

        const uri = vscode.Uri.parse(location.uri);
        const range = new vscode.Range(
          location.range.start.line,
          location.range.start.character,
          location.range.end.line,
          location.range.end.character
        );

        // 打开文档并选中范围
        const editor = await vscode.window.showTextDocument(uri, {
          selection: range,
          preview: false,
          viewColumn: vscode.ViewColumn.One
        });

        // 确保范围可见并居中显示
        editor.revealRange(range, vscode.TextEditorRevealType.InCenter);
      } catch (error) {
        vscode.window.showErrorMessage(
          `导航失败: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    });

    // 复制配置示例
    this.register(Commands.CONFIGURATION_COPY_EXAMPLE, async (item?: any) => {
      if (!item) {
        vscode.window.showWarningMessage('未提供配置信息');
        return;
      }

      try {
        // 提取配置结构
        let config: ConfigurationStruct | undefined;
        if (item.config) {
          config = item.config;
        } else if (item.name && item.prefix && item.fields) {
          config = item as ConfigurationStruct;
        }

        if (!config) {
          vscode.window.showWarningMessage('无效的配置对象');
          return;
        }

        // 生成配置示例
        const example = this.generateConfigExample(config);

        // 复制到剪贴板
        await vscode.env.clipboard.writeText(example);
        
        // 显示成功消息，并提供选项
        const action = await vscode.window.showInformationMessage(
          `已复制配置示例到剪贴板`,
          '粘贴到配置文件',
          '查看示例'
        );

        if (action === '粘贴到配置文件') {
          // 打开或创建配置文件
          const configPath = await this.findOrCreateConfigFile();
          if (configPath) {
            const document = await vscode.workspace.openTextDocument(configPath);
            const editor = await vscode.window.showTextDocument(document);
            
            // 在文档末尾插入
            const lastLine = document.lineCount - 1;
            const lastLineText = document.lineAt(lastLine).text;
            const position = new vscode.Position(
              lastLine,
              lastLineText.length
            );
            
            await editor.edit(editBuilder => {
              editBuilder.insert(position, '\n\n' + example);
            });
          }
        } else if (action === '查看示例') {
          // 在新文档中显示示例
          const doc = await vscode.workspace.openTextDocument({
            language: 'toml',
            content: example
          });
          await vscode.window.showTextDocument(doc, {
            preview: true,
            viewColumn: vscode.ViewColumn.Beside
          });
        }
      } catch (error) {
        vscode.window.showErrorMessage(
          `复制配置示例失败: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    });
  }

  /**
   * 生成配置示例
   */
  private generateConfigExample(config: ConfigurationStruct): string {
    const lines: string[] = [];
    
    // 添加注释说明
    lines.push(`# ${config.name} 配置`);
    lines.push(`# 配置前缀: [${config.prefix}]`);
    lines.push('');
    
    // 添加配置节
    lines.push(`[${config.prefix}]`);
    
    // 添加字段
    for (const field of config.fields) {
      // 添加字段描述
      if (field.description) {
        lines.push(`# ${field.description}`);
      }
      
      // 添加字段类型和是否必需
      const required = field.optional ? '可选' : '必需';
      lines.push(`# 类型: ${field.type} (${required})`);
      
      // 添加字段示例值
      const exampleValue = this.getExampleValue(field.type);
      if (field.optional) {
        lines.push(`# ${field.name} = ${exampleValue}`);
      } else {
        lines.push(`${field.name} = ${exampleValue}`);
      }
      lines.push('');
    }
    
    return lines.join('\n');
  }

  /**
   * 根据类型获取示例值
   */
  private getExampleValue(type: string): string {
    // 移除 Option<T> 包装
    const innerType = type.replace(/^Option<(.+)>$/, '$1');
    
    // 基本类型
    if (innerType === 'String' || innerType === 'str' || innerType.includes('String')) {
      return '"example"';
    }
    if (innerType === 'bool') {
      return 'true';
    }
    if (innerType.match(/^(i|u)(8|16|32|64|128|size)$/)) {
      return '0';
    }
    if (innerType.match(/^f(32|64)$/)) {
      return '0.0';
    }
    
    // 集合类型
    if (innerType.startsWith('Vec<')) {
      return '[]';
    }
    if (innerType.startsWith('HashMap<') || innerType.startsWith('BTreeMap<')) {
      return '{}';
    }
    
    // 默认
    return '"TODO: 填写配置值"';
  }

  /**
   * 查找或创建配置文件
   */
  private async findOrCreateConfigFile(): Promise<vscode.Uri | undefined> {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders || workspaceFolders.length === 0) {
      vscode.window.showWarningMessage('未打开工作空间');
      return undefined;
    }

    // 查找现有的配置文件
    const configFiles = await vscode.workspace.findFiles(
      '**/config/app.toml',
      '**/target/**'
    );

    if (configFiles.length > 0) {
      // 如果有多个，让用户选择
      if (configFiles.length > 1) {
        const selected = await vscode.window.showQuickPick(
          configFiles.map(uri => ({
            label: vscode.workspace.asRelativePath(uri),
            uri
          })),
          { placeHolder: '选择配置文件' }
        );
        return selected?.uri;
      }
      return configFiles[0];
    }

    // 没有找到，询问是否创建
    const create = await vscode.window.showInformationMessage(
      '未找到配置文件，是否创建 config/app.toml？',
      '创建',
      '取消'
    );

    if (create === '创建') {
      // 选择工作空间文件夹
      let folder = workspaceFolders[0];
      if (workspaceFolders.length > 1) {
        const selected = await vscode.window.showWorkspaceFolderPick({
          placeHolder: '选择工作空间文件夹'
        });
        if (selected) {
          folder = selected;
        }
      }

      // 创建配置文件
      const configUri = vscode.Uri.joinPath(folder.uri, 'config', 'app.toml');
      const configDir = vscode.Uri.joinPath(folder.uri, 'config');
      
      try {
        // 创建目录
        await vscode.workspace.fs.createDirectory(configDir);
        
        // 创建文件
        const content = new TextEncoder().encode(
          '# Spring RS 配置文件\n' +
          '#:schema https://spring-rs.github.io/config-schema.json\n\n'
        );
        await vscode.workspace.fs.writeFile(configUri, content);
        
        return configUri;
      } catch (error) {
        vscode.window.showErrorMessage(
          `创建配置文件失败: ${error instanceof Error ? error.message : String(error)}`
        );
        return undefined;
      }
    }

    return undefined;
  }

  /**
   * 注册视图模式切换命令
   */
  private registerViewModeCommands(): void {
    // Components 视图模式选择
    this.register(Commands.COMPONENTS_SELECT_VIEW_MODE, async () => {
      if (this.componentsProvider) {
        await this.componentsProvider.selectViewMode();
      }
    });

    // Components 视图模式切换（List -> Tree）
    this.register(Commands.COMPONENTS_TOGGLE_VIEW_MODE, async () => {
      if (this.componentsProvider) {
        await this.componentsProvider.toggleViewMode();
      }
    });

    // Components 切换到 List 视图（Tree -> List）
    this.register(Commands.COMPONENTS_SWITCH_TO_LIST_VIEW, async () => {
      if (this.componentsProvider) {
        await this.componentsProvider.toggleViewMode();
      }
    });

    // Routes 视图模式选择
    this.register(Commands.ROUTES_SELECT_VIEW_MODE, async () => {
      if (this.routesProvider) {
        await this.routesProvider.selectViewMode();
      }
    });

    // Routes 视图模式切换（List -> Tree）
    this.register(Commands.ROUTES_TOGGLE_VIEW_MODE, async () => {
      if (this.routesProvider) {
        await this.routesProvider.toggleViewMode();
      }
    });

    // Routes 切换到 List 视图（Tree -> List）
    this.register(Commands.ROUTES_SWITCH_TO_LIST_VIEW, async () => {
      if (this.routesProvider) {
        await this.routesProvider.toggleViewMode();
      }
    });

    // Configurations 视图模式选择
    this.register(Commands.CONFIGURATIONS_SELECT_VIEW_MODE, async () => {
      if (this.configurationsProvider && 'selectViewMode' in this.configurationsProvider) {
        await this.configurationsProvider.selectViewMode();
      }
    });

    // Configurations 视图模式切换（List -> Tree）
    this.register(Commands.CONFIGURATIONS_TOGGLE_VIEW_MODE, async () => {
      if (this.configurationsProvider && 'toggleViewMode' in this.configurationsProvider) {
        await this.configurationsProvider.toggleViewMode();
      }
    });

    // Configurations 切换到 List 视图（Tree -> List）
    this.register(Commands.CONFIGURATIONS_SWITCH_TO_LIST_VIEW, async () => {
      if (this.configurationsProvider && 'toggleViewMode' in this.configurationsProvider) {
        await this.configurationsProvider.toggleViewMode();
      }
    });
  }

  /**
   * 注册文档和帮助命令
   */
  private registerDocumentationCommands(): void {
    // 打开文档
    this.register(Commands.OPEN_DOCUMENTATION, async () => {
      const url = 'https://spring-rs.github.io/';
      await vscode.env.openExternal(vscode.Uri.parse(url));
    });

    // 显示欢迎页面
    this.register(Commands.SHOW_WELCOME, async () => {
      const panel = vscode.window.createWebviewPanel(
        'springWelcome',
        'Welcome to Spring LSP',
        vscode.ViewColumn.One,
        {
          enableScripts: false
        }
      );

      panel.webview.html = this.getWelcomeHtml();
    });
  }

  /**
   * 注册内部命令（用于测试和调试）
   */
  private registerInternalCommands(): void {
    // 获取应用列表（内部命令）
    this.register(Commands._GET_APPS, () => {
      return this.appManager.getAppList();
    });
  }

  /**
   * 注册单个命令
   */
  private register(
    command: string,
    callback: (...args: any[]) => any
  ): void {
    const disposable = vscode.commands.registerCommand(command, callback);
    this.disposables.push(disposable);
    this.context.subscriptions.push(disposable);
  }

  /**
   * 从参数中提取 SpringApp 对象
   * 
   * 支持以下类型的参数：
   * - SpringApp 实例
   * - AppTreeItem 实例（包含 app 属性）
   * - 其他对象（尝试提取 app 属性）
   * 
   * @param item 参数对象
   * @returns SpringApp 实例，如果无法提取返回 undefined
   */
  private extractApp(item?: any): SpringApp | undefined {
    if (!item) {
      return undefined;
    }

    // 如果已经是 SpringApp 实例
    if (item instanceof SpringApp) {
      return item;
    }

    // 如果有 app 属性（AppTreeItem）
    if (item.app && item.app instanceof SpringApp) {
      return item.app;
    }

    // 无法提取
    return undefined;
  }

  /**
   * 选择应用
   */
  private async selectApp(
    placeHolder: string,
    filter?: (app: SpringApp) => boolean
  ): Promise<SpringApp | undefined> {
    let apps = this.appManager.getAppList();

    if (filter) {
      apps = apps.filter(filter);
    }

    if (apps.length === 0) {
      vscode.window.showInformationMessage('No apps available');
      return undefined;
    }

    if (apps.length === 1) {
      return apps[0];
    }

    const selected = await vscode.window.showQuickPick(
      apps.map(app => ({
        label: app.name,
        description: `${app.path} - ${app.state}`,
        app
      })),
      { placeHolder }
    );

    return selected?.app;
  }

  /**
   * 生成欢迎页面 HTML
   */
  private getWelcomeHtml(): string {
    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Welcome to Spring LSP</title>
  <style>
    body {
      font-family: var(--vscode-font-family);
      padding: 20px;
      color: var(--vscode-foreground);
      background-color: var(--vscode-editor-background);
    }
    h1 {
      color: var(--vscode-textLink-foreground);
    }
    h2 {
      margin-top: 30px;
      border-bottom: 1px solid var(--vscode-panel-border);
      padding-bottom: 5px;
    }
    ul {
      line-height: 1.8;
    }
    code {
      background-color: var(--vscode-textCodeBlock-background);
      padding: 2px 6px;
      border-radius: 3px;
    }
    .feature {
      margin: 15px 0;
    }
    .feature-title {
      font-weight: bold;
      color: var(--vscode-textLink-foreground);
    }
  </style>
</head>
<body>
  <h1>🚀 Welcome to Spring LSP for Rust</h1>
  <h2>spring-rs has been renamed to summer-rs</h2>
  <p>
    Spring LSP 为 <strong>spring-rs</strong> 框架提供完整的 IDE 支持，
    帮助你更高效地开发 Rust 应用。
  </p>

  <h2>✨ 主要功能</h2>

  <div class="feature">
    <div class="feature-title">📦 应用管理</div>
    <ul>
      <li>自动检测工作空间中的 spring-rs 应用</li>
      <li>一键启动、停止和调试应用</li>
      <li>支持 Profile 选择和环境配置</li>
      <li>批量操作多个应用</li>
    </ul>
  </div>

  <div class="feature">
    <div class="feature-title">🔍 实时信息</div>
    <ul>
      <li><strong>Components 视图</strong>：查看所有注册的组件和依赖关系</li>
      <li><strong>Routes 视图</strong>：查看所有 HTTP 路由和端点</li>
      <li><strong>Jobs 视图</strong>：查看定时任务和调度信息</li>
      <li><strong>Plugins 视图</strong>：查看已加载的插件</li>
      <li><strong>依赖图</strong>：可视化组件依赖关系</li>
    </ul>
  </div>

  <div class="feature">
    <div class="feature-title">⚡ 智能支持</div>
    <ul>
      <li>TOML 配置文件的智能补全和验证</li>
      <li>代码导航和跳转</li>
      <li>实时诊断和错误提示</li>
      <li>代码片段和模板</li>
    </ul>
  </div>

  <h2>🚀 快速开始</h2>

  <ol>
    <li>打开一个包含 spring-rs 应用的工作空间</li>
    <li>在活动栏点击 <strong>Spring RS</strong> 图标</li>
    <li>在 <strong>Apps</strong> 视图中查看检测到的应用</li>
    <li>右键点击应用，选择 <strong>Run</strong> 或 <strong>Debug</strong></li>
    <li>应用启动后，查看 <strong>Components</strong>、<strong>Routes</strong> 等视图</li>
  </ol>

  <h2>⚙️ 配置</h2>

  <p>在 VSCode 设置中搜索 <code>spring-rs</code> 可以配置：</p>
  <ul>
    <li><code>spring-rs.serverPath</code>：语言服务器路径</li>
    <li><code>spring-rs.openWith</code>：浏览器打开方式（integrated/external）</li>
    <li><code>spring-rs.openUrl</code>：URL 模板</li>
    <li><code>spring-rs.env</code>：环境变量</li>
  </ul>

  <h2>📚 资源</h2>

  <ul>
    <li><a href="https://spring-rs.github.io/">Spring RS 官方文档</a></li>
    <li><a href="https://github.com/spring-rs/spring-rs">Spring RS GitHub</a></li>
    <li><a href="https://github.com/spring-rs/spring-lsp">Spring LSP GitHub</a></li>
  </ul>

  <h2>💡 提示</h2>

  <ul>
    <li>使用 <code>Ctrl+Shift+P</code> (Windows/Linux) 或 <code>Cmd+Shift+P</code> (macOS) 打开命令面板</li>
    <li>输入 <code>Spring</code> 查看所有可用命令</li>
    <li>右键点击视图中的项目查看可用操作</li>
  </ul>

  <p style="margin-top: 40px; text-align: center; color: var(--vscode-descriptionForeground);">
    Happy coding with Spring RS! 🎉
  </p>
</body>
</html>`;
  }

  /**
   * 清理资源
   */
  public dispose(): void {
    this.disposables.forEach(d => d.dispose());
    this.disposables = [];
    this.dependencyGraphView?.dispose();
  }
}
