import * as vscode from 'vscode';
import { LocalAppManager } from './controllers/LocalAppManager';
import { LocalAppController } from './controllers/LocalAppController';
import { LanguageClientManager } from './languageClient/LanguageClientManager';
import { CommandManager } from './commands';
import { SummerApp } from './models/SummerApp';
import {
  AppsTreeDataProvider,
  JobsTreeDataProvider,
  PluginsTreeDataProvider
} from './views';
import { ComponentsTreeDataProviderEnhanced } from './views/ComponentsTreeDataProviderEnhanced';
import { RoutesTreeDataProviderEnhanced } from './views/RoutesTreeDataProviderEnhanced';
import { ConfigurationsTreeDataProviderEnhanced } from './views/ConfigurationsTreeDataProviderEnhanced';
import { GutterDecorationManager } from './gutter';

/**
 * 扩展激活函数
 * 
 * 当满足激活条件时，VSCode 会调用此函数
 */
export async function activate(context: vscode.ExtensionContext): Promise<void> {
  console.log('Summer LSP extension is now activating...');

  try {
    // 1. 创建输出通道（用于扩展自身的日志）
    const outputChannel = vscode.window.createOutputChannel('Summer LSP');
    context.subscriptions.push(outputChannel);
    outputChannel.appendLine('Summer LSP extension starting...');

    // 2. 创建语言客户端管理器（会创建单独的语言服务器输出通道）
    outputChannel.appendLine('Initializing language client...');
    const languageClient = new LanguageClientManager(context, outputChannel);

    // 3. 创建应用管理器
    outputChannel.appendLine('Initializing app manager...');
    const appManager = new LocalAppManager();
    context.subscriptions.push(appManager);
    
    // 初始化应用管理器（启动工作空间扫描）
    await appManager.initialize();
    outputChannel.appendLine('App manager initialized');

    // 4. 创建应用控制器
    outputChannel.appendLine('Initializing app controller...');
    const appController = new LocalAppController(appManager, context);

    // 5. 注册视图
    outputChannel.appendLine('Registering views...');
    const { configurationsProvider, componentsProvider, routesProvider, refreshInitialApp } = registerViews(context, appManager, languageClient);

    // 6. 注册命令
    outputChannel.appendLine('Registering commands...');
    const commandManager = new CommandManager(
      context,
      appManager,
      appController,
      languageClient
    );
    commandManager.registerCommands();
    
    // 设置 provider 引用（必须在 registerCommands 之后）
    setupCommandManagerProviders(
      commandManager,
      componentsProvider,
      routesProvider,
      configurationsProvider
    );
    
    context.subscriptions.push(commandManager);

    // 6.5. 初始化 Gutter 装饰管理器（可选功能）
    outputChannel.appendLine('Initializing gutter decorations...');
    const gutterManager = new GutterDecorationManager(context);
    gutterManager.registerCommands();
    context.subscriptions.push(gutterManager);

    // 7. 设置调试会话事件监听器
    outputChannel.appendLine('Setting up debug session listeners...');
    setupDebugSessionListeners(context, appController);

    // 8. 启动语言服务器
    outputChannel.appendLine('Starting language server...');
    try {
      await languageClient.start();
      outputChannel.appendLine('Language server started successfully');
      
      // 语言服务器启动后，刷新配置视图
      outputChannel.appendLine('Refreshing configurations view...');
      await configurationsProvider.refresh();
      
      // 语言服务器启动后，刷新初始应用的所有视图
      outputChannel.appendLine('Refreshing initial app views...');
      refreshInitialApp();
    } catch (error) {
      outputChannel.appendLine(
        `Warning: Language server failed to start: ${error instanceof Error ? error.message : String(error)}`
      );
      vscode.window.showWarningMessage(
        'Summer LSP language server failed to start. Some features may not be available.',
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

    // 9. 设置上下文变量
    await vscode.commands.executeCommand('setContext', 'summer:activated', true);

    // 10. 显示欢迎消息（仅首次激活）
    const hasShownWelcome = context.globalState.get<boolean>('summer.hasShownWelcome');
    if (!hasShownWelcome) {
      const selection = await vscode.window.showInformationMessage(
        'Welcome to Summer LSP for Rust! 🚀',
        'Show Welcome Page',
        'Dismiss'
      );
      if (selection === 'Show Welcome Page') {
        await vscode.commands.executeCommand('summer-rs.showWelcome');
      }
      await context.globalState.update('summer.hasShownWelcome', true);
    }

    outputChannel.appendLine('Summer LSP extension activated successfully!');
    console.log('Summer LSP extension is now active!');
  } catch (error) {
    console.error('Failed to activate Summer LSP extension:', error);
    vscode.window.showErrorMessage(
      `Failed to activate Summer LSP extension: ${error instanceof Error ? error.message : String(error)}`
    );
    throw error;
  }
}

/**
 * 注册所有视图
 */
function registerViews(
  context: vscode.ExtensionContext,
  appManager: LocalAppManager,
  languageClient: LanguageClientManager
): { 
  configurationsProvider: ConfigurationsTreeDataProviderEnhanced; 
  componentsProvider: ComponentsTreeDataProviderEnhanced;
  routesProvider: RoutesTreeDataProviderEnhanced;
  refreshInitialApp: () => void;
} {
  // 1. 注册 Apps 视图（带应用选择功能）
  const appsProvider = new AppsTreeDataProvider(appManager);
  const appsView = vscode.window.createTreeView('summer.apps', {
    treeDataProvider: appsProvider,
    showCollapseAll: false,
    canSelectMany: false  // 只能选择一个应用
  });
  context.subscriptions.push(appsView);

  // 2. 注册 Components 视图
  const componentsProvider = new ComponentsTreeDataProviderEnhanced(languageClient, context);
  const componentsView = vscode.window.createTreeView('summer.components', {
    treeDataProvider: componentsProvider,
    showCollapseAll: true
  });
  context.subscriptions.push(componentsView);

  // 3. 注册 Routes 视图
  const routesProvider = new RoutesTreeDataProviderEnhanced(languageClient, context);
  const routesView = vscode.window.createTreeView('summer.routes', {
    treeDataProvider: routesProvider,
    showCollapseAll: true
  });
  context.subscriptions.push(routesView);

  // 4. 注册 Jobs 视图
  const jobsProvider = new JobsTreeDataProvider(languageClient, context);
  const jobsView = vscode.window.createTreeView('summer.jobs', {
    treeDataProvider: jobsProvider,
    showCollapseAll: true
  });
  context.subscriptions.push(jobsView);

  // 5. 注册 Plugins 视图
  const pluginsProvider = new PluginsTreeDataProvider(languageClient);
  const pluginsView = vscode.window.createTreeView('summer.plugins', {
    treeDataProvider: pluginsProvider,
    showCollapseAll: true
  });
  context.subscriptions.push(pluginsView);

  // 6. 注册 Configurations 视图
  const configurationsProvider = new ConfigurationsTreeDataProviderEnhanced(languageClient, context);
  const configurationsView = vscode.window.createTreeView('summer.configurations', {
    treeDataProvider: configurationsProvider,
    showCollapseAll: true
  });
  context.subscriptions.push(configurationsView);

  // 监听应用选择事件，刷新所有视图（必须在监听复选框事件之前设置）
  appsProvider.onDidSelectApp((app: SummerApp) => {
    console.log(`App selected: ${app.name}, refreshing all views...`);
    
    // 刷新所有视图
    componentsProvider.refresh(app);
    routesProvider.refresh(app);
    configurationsProvider.refresh(app);
    jobsProvider.refresh(app);
    pluginsProvider.refresh(app);
    
    // 更新视图描述（显示当前应用名称）
    componentsView.description = app.name;
    routesView.description = app.name;
    configurationsView.description = app.name;
    jobsView.description = app.name;
    pluginsView.description = app.name;
  });

  // 监听复选框变化事件
  appsView.onDidChangeCheckboxState((event) => {
    for (const [item, state] of event.items) {
      if (state === vscode.TreeItemCheckboxState.Checked && 'app' in item) {
        // 用户选中了某个应用
        appsProvider.selectApp((item as any).app);
        break;
      }
    }
  });

  // 创建初始刷新函数（在语言服务器启动后调用）
  const refreshInitialApp = () => {
    const initialApp = appsProvider.getSelectedApp();
    if (initialApp) {
      console.log(`[After LSP ready] Initial app selected: ${initialApp.name}, refreshing all views...`);
      componentsProvider.refresh(initialApp);
      routesProvider.refresh(initialApp);
      configurationsProvider.refresh(initialApp);
      jobsProvider.refresh(initialApp);
      pluginsProvider.refresh(initialApp);
      
      componentsView.description = initialApp.name;
      routesView.description = initialApp.name;
      jobsView.description = initialApp.name;
      pluginsView.description = initialApp.name;
    }
  };

  // 监听应用状态变化，刷新当前选中应用的视图
  appManager.onDidChangeApps((app: SummerApp | undefined) => {
    const selectedApp = appsProvider.getSelectedApp();
    
    if (app && selectedApp && app.path === selectedApp.path) {
      // 当前选中的应用状态变化，刷新视图
      if (app.state === 'running') {
        componentsProvider.refresh(app);
        routesProvider.refresh(app);
        configurationsProvider.refresh(app);
        jobsProvider.refresh(app);
        pluginsProvider.refresh(app);
      } else if (app.state === 'inactive') {
        // 应用停止，刷新为静态模式
        componentsProvider.refresh(app);
        routesProvider.refresh(app);
        configurationsProvider.refresh(app);
        jobsProvider.refresh(app);
        pluginsProvider.refresh(app);
      }
    }
    
    // 更新上下文变量
    const hasRunningApp = appManager.getAppList().some(a => a.state === 'running');
    vscode.commands.executeCommand('setContext', 'summer:hasRunningApp', hasRunningApp);
  });

  // 初始化配置视图
  configurationsProvider.refresh();

  // 监听文档变化，刷新配置视图
  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((document) => {
      if (document.languageId === 'rust') {
        configurationsProvider.refresh();
      }
    })
  );

  return { configurationsProvider, componentsProvider, routesProvider, refreshInitialApp };
}

/**
 * 设置 CommandManager 的 provider 引用
 */
function setupCommandManagerProviders(
  commandManager: CommandManager,
  componentsProvider: ComponentsTreeDataProviderEnhanced,
  routesProvider: RoutesTreeDataProviderEnhanced,
  configurationsProvider: ConfigurationsTreeDataProviderEnhanced
): void {
  commandManager.setComponentsProvider(componentsProvider);
  commandManager.setRoutesProvider(routesProvider);
  commandManager.setConfigurationsProvider(configurationsProvider);
}

/**
 * 设置调试会话事件监听器
 */
function setupDebugSessionListeners(
  context: vscode.ExtensionContext,
  appController: LocalAppController
): void {
  // 监听调试会话启动
  const onDidStartDebugSession = vscode.debug.onDidStartDebugSession(session => {
    // 检查是否是 Rust 调试会话
    if (session.type === 'lldb' || session.type === 'rust' || session.type === 'cppdbg') {
      appController.onDidStartApp(session);
    }
  });
  context.subscriptions.push(onDidStartDebugSession);

  // 监听调试会话终止
  const onDidTerminateDebugSession = vscode.debug.onDidTerminateDebugSession(session => {
    if (session.type === 'lldb' || session.type === 'rust' || session.type === 'cppdbg') {
      appController.onDidStopApp(session);
    }
  });
  context.subscriptions.push(onDidTerminateDebugSession);
}

/**
 * 扩展停用函数
 * 
 * 当扩展被停用时，VSCode 会调用此函数
 */
export function deactivate(): void {
  console.log('Summer LSP extension is now deactivating...');
  
  // 清理上下文变量
  vscode.commands.executeCommand('setContext', 'summer:activated', false);
  vscode.commands.executeCommand('setContext', 'summer:hasRunningApp', false);
  
  // 注意：所有资源都通过 context.subscriptions 自动清理
  // 不需要手动调用 dispose
  
  console.log('Summer LSP extension deactivated');
}
