/**
 * DependencyGraphView - 依赖图 WebView 视图
 * 
 * 使用 D3.js 可视化组件依赖关系图
 */

import * as vscode from 'vscode';
import { LanguageClientManager } from '../languageClient/LanguageClientManager';
import { SummerApp } from '../models/SummerApp';

/**
 * 依赖图节点
 */
export interface DependencyNode {
  /** 节点 ID（组件名称） */
  id: string;
  /** 显示名称 */
  name: string;
  /** 组件类型 */
  type: string;
  /** 是否有错误（循环依赖等） */
  hasError: boolean;
  /** 错误消息 */
  errorMessage?: string;
  /** 源码位置 */
  location?: {
    uri: string;
    range: {
      start: { line: number; character: number };
      end: { line: number; character: number };
    };
  };
}

/**
 * 依赖图边
 */
export interface DependencyEdge {
  /** 源节点 ID */
  source: string;
  /** 目标节点 ID */
  target: string;
  /** 依赖类型（component, config, lazy） */
  dependencyType: string;
}

/**
 * 依赖图数据
 */
export interface DependencyGraph {
  /** 节点列表 */
  nodes: DependencyNode[];
  /** 边列表 */
  edges: DependencyEdge[];
}

/**
 * LSP 请求参数
 */
interface DependencyGraphRequest {
  appPath: string;
}

/**
 * LSP 响应
 */
interface DependencyGraphResponse {
  nodes: DependencyNode[];
  edges: DependencyEdge[];
}

/**
 * WebView 消息类型
 */
interface WebViewMessage {
  command: string;
  componentName?: string;
}

/**
 * 依赖图 WebView 视图
 */
export class DependencyGraphView {
  private panel: vscode.WebviewPanel | undefined;
  private currentApp: SummerApp | undefined;

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly languageClient: LanguageClientManager
  ) {}

  /**
   * 显示依赖图
   */
  public async show(app: SummerApp): Promise<void> {
    this.currentApp = app;

    // 如果面板已存在，则显示并更新
    if (this.panel) {
      this.panel.reveal(vscode.ViewColumn.One);
      await this.updateGraph(app);
      return;
    }

    // 创建新的 WebView 面板
    this.panel = vscode.window.createWebviewPanel(
      'summerDependencyGraph',
      `Dependency Graph - ${app.name}`,
      vscode.ViewColumn.One,
      {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [
          vscode.Uri.joinPath(this.context.extensionUri, 'resources')
        ]
      }
    );

    // 设置图标
    this.panel.iconPath = {
      light: vscode.Uri.joinPath(
        this.context.extensionUri,
        'resources',
        'light',
        'dependency-graph.svg'
      ),
      dark: vscode.Uri.joinPath(
        this.context.extensionUri,
        'resources',
        'dark',
        'dependency-graph.svg'
      )
    };

    // 处理面板关闭
    this.panel.onDidDispose(() => {
      this.panel = undefined;
      this.currentApp = undefined;
    });

    // 处理 WebView 消息
    this.panel.webview.onDidReceiveMessage(
      async (message: WebViewMessage) => {
        await this.handleMessage(message);
      }
    );

    // 加载并显示依赖图
    await this.updateGraph(app);
  }

  /**
   * 更新依赖图
   */
  private async updateGraph(app: SummerApp): Promise<void> {
    if (!this.panel) {
      return;
    }

    try {
      // 获取依赖图数据
      const graph = await this.fetchDependencyGraph(app);

      // 更新 WebView 内容
      this.panel.webview.html = this.getHtmlContent(graph, app);
    } catch (error) {
      vscode.window.showErrorMessage(
        `Failed to load dependency graph: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  /**
   * 从语言服务器获取依赖图数据
   */
  private async fetchDependencyGraph(app: SummerApp): Promise<DependencyGraph> {
    const response = await this.languageClient.sendRequest<DependencyGraphResponse>(
      'summer/dependencyGraph',
      {
        appPath: app.path
      }
    );

    if (!response) {
      throw new Error('No response from language server');
    }

    return {
      nodes: response.nodes || [],
      edges: response.edges || []
    };
  }

  /**
   * 处理 WebView 消息
   */
  private async handleMessage(message: WebViewMessage): Promise<void> {
    switch (message.command) {
      case 'navigateToComponent':
        if (message.componentName) {
          await this.navigateToComponent(message.componentName);
        }
        break;

      case 'refresh':
        if (this.currentApp) {
          await this.updateGraph(this.currentApp);
        }
        break;

      default:
        console.warn('Unknown message command:', message.command);
    }
  }

  /**
   * 跳转到组件定义
   */
  private async navigateToComponent(componentName: string): Promise<void> {
    if (!this.currentApp) {
      return;
    }

    try {
      // 从语言服务器获取组件位置
      const location = await this.languageClient.sendRequest<
        { uri: string; range: any } | null
      >('summer/componentLocation', {
        appPath: this.currentApp.path,
        componentName
      });

      if (!location) {
        vscode.window.showWarningMessage(
          `Could not find location for component: ${componentName}`
        );
        return;
      }

      // 打开文档并跳转到位置
      const uri = vscode.Uri.parse(location.uri);
      const range = new vscode.Range(
        location.range.start.line,
        location.range.start.character,
        location.range.end.line,
        location.range.end.character
      );

      await vscode.window.showTextDocument(uri, {
        selection: range,
        preview: false
      });
    } catch (error) {
      vscode.window.showErrorMessage(
        `Failed to navigate to component: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  /**
   * 生成 WebView HTML 内容
   */
  private getHtmlContent(graph: DependencyGraph, app: SummerApp): string {
    const graphData = JSON.stringify(graph);
    const appName = app.name;

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Dependency Graph - ${appName}</title>
  <style>
    body {
      margin: 0;
      padding: 20px;
      font-family: var(--vscode-font-family);
      background-color: var(--vscode-editor-background);
      color: var(--vscode-editor-foreground);
    }

    #toolbar {
      margin-bottom: 10px;
      display: flex;
      gap: 10px;
      align-items: center;
    }

    button {
      background-color: var(--vscode-button-background);
      color: var(--vscode-button-foreground);
      border: none;
      padding: 6px 14px;
      cursor: pointer;
      border-radius: 2px;
    }

    button:hover {
      background-color: var(--vscode-button-hoverBackground);
    }

    #graph {
      width: 100%;
      height: calc(100vh - 100px);
      border: 1px solid var(--vscode-panel-border);
      background-color: var(--vscode-editor-background);
    }

    .node {
      cursor: pointer;
      stroke: var(--vscode-editor-foreground);
      stroke-width: 2px;
    }

    .node:hover {
      opacity: 0.8;
      stroke-width: 3px;
    }

    .node-error {
      fill: var(--vscode-errorForeground);
    }

    .node-normal {
      fill: var(--vscode-charts-blue);
    }

    .link {
      stroke: var(--vscode-editor-foreground);
      stroke-opacity: 0.3;
      stroke-width: 2px;
      fill: none;
    }

    .link-lazy {
      stroke-dasharray: 5, 5;
    }

    .label {
      font-size: 12px;
      fill: var(--vscode-editor-foreground);
      pointer-events: none;
      user-select: none;
    }

    .tooltip {
      position: absolute;
      padding: 8px;
      background-color: var(--vscode-editorHoverWidget-background);
      border: 1px solid var(--vscode-editorHoverWidget-border);
      border-radius: 3px;
      pointer-events: none;
      opacity: 0;
      transition: opacity 0.2s;
      font-size: 12px;
      max-width: 300px;
    }

    .legend {
      margin-top: 10px;
      padding: 10px;
      border: 1px solid var(--vscode-panel-border);
      border-radius: 3px;
    }

    .legend-item {
      display: flex;
      align-items: center;
      gap: 8px;
      margin: 5px 0;
    }

    .legend-color {
      width: 20px;
      height: 20px;
      border-radius: 50%;
      border: 2px solid var(--vscode-editor-foreground);
    }
  </style>
</head>
<body>
  <div id="toolbar">
    <button id="refreshBtn">Refresh</button>
    <button id="zoomInBtn">Zoom In</button>
    <button id="zoomOutBtn">Zoom Out</button>
    <button id="resetBtn">Reset View</button>
    <span style="margin-left: auto;">
      Nodes: <strong id="nodeCount">0</strong> | 
      Edges: <strong id="edgeCount">0</strong>
    </span>
  </div>

  <div id="graph"></div>
  <div class="tooltip" id="tooltip"></div>

  <div class="legend">
    <div class="legend-item">
      <div class="legend-color node-normal"></div>
      <span>Normal Component</span>
    </div>
    <div class="legend-item">
      <div class="legend-color node-error"></div>
      <span>Component with Error (e.g., circular dependency)</span>
    </div>
    <div class="legend-item">
      <svg width="40" height="2">
        <line x1="0" y1="1" x2="40" y2="1" stroke="currentColor" stroke-width="2"/>
      </svg>
      <span>Direct Dependency</span>
    </div>
    <div class="legend-item">
      <svg width="40" height="2">
        <line x1="0" y1="1" x2="40" y2="1" stroke="currentColor" stroke-width="2" stroke-dasharray="5,5"/>
      </svg>
      <span>Lazy Dependency</span>
    </div>
  </div>

  <script src="https://d3js.org/d3.v7.min.js"></script>
  <script>
    const vscode = acquireVsCodeApi();
    const graphData = ${graphData};

    // 更新统计信息
    document.getElementById('nodeCount').textContent = graphData.nodes.length;
    document.getElementById('edgeCount').textContent = graphData.edges.length;

    // 创建 SVG
    const container = document.getElementById('graph');
    const width = container.clientWidth;
    const height = container.clientHeight;

    const svg = d3.select('#graph')
      .append('svg')
      .attr('width', width)
      .attr('height', height);

    const g = svg.append('g');

    // 缩放行为
    let currentZoom = 1;
    const zoom = d3.zoom()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        g.attr('transform', event.transform);
        currentZoom = event.transform.k;
      });

    svg.call(zoom);

    // 力导向图模拟
    const simulation = d3.forceSimulation(graphData.nodes)
      .force('link', d3.forceLink(graphData.edges)
        .id(d => d.id)
        .distance(100))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(30));

    // 绘制边
    const link = g.append('g')
      .selectAll('path')
      .data(graphData.edges)
      .enter()
      .append('path')
      .attr('class', d => \`link \${d.dependencyType === 'lazy' ? 'link-lazy' : ''}\`);

    // 绘制节点
    const node = g.append('g')
      .selectAll('circle')
      .data(graphData.nodes)
      .enter()
      .append('circle')
      .attr('r', 15)
      .attr('class', d => \`node \${d.hasError ? 'node-error' : 'node-normal'}\`)
      .on('click', (event, d) => {
        vscode.postMessage({
          command: 'navigateToComponent',
          componentName: d.name
        });
      })
      .on('mouseover', (event, d) => {
        showTooltip(event, d);
      })
      .on('mouseout', () => {
        hideTooltip();
      })
      .call(d3.drag()
        .on('start', dragStarted)
        .on('drag', dragged)
        .on('end', dragEnded));

    // 添加标签
    const label = g.append('g')
      .selectAll('text')
      .data(graphData.nodes)
      .enter()
      .append('text')
      .attr('class', 'label')
      .text(d => d.name)
      .attr('dx', 20)
      .attr('dy', 4);

    // 更新位置
    simulation.on('tick', () => {
      link.attr('d', d => {
        const dx = d.target.x - d.source.x;
        const dy = d.target.y - d.source.y;
        return \`M\${d.source.x},\${d.source.y} L\${d.target.x},\${d.target.y}\`;
      });

      node
        .attr('cx', d => d.x)
        .attr('cy', d => d.y);

      label
        .attr('x', d => d.x)
        .attr('y', d => d.y);
    });

    // 拖拽处理
    function dragStarted(event, d) {
      if (!event.active) simulation.alphaTarget(0.3).restart();
      d.fx = d.x;
      d.fy = d.y;
    }

    function dragged(event, d) {
      d.fx = event.x;
      d.fy = event.y;
    }

    function dragEnded(event, d) {
      if (!event.active) simulation.alphaTarget(0);
      d.fx = null;
      d.fy = null;
    }

    // 工具提示
    const tooltip = document.getElementById('tooltip');

    function showTooltip(event, d) {
      let content = \`<strong>\${d.name}</strong><br>\`;
      content += \`Type: \${d.type}<br>\`;
      if (d.hasError) {
        content += \`<span style="color: var(--vscode-errorForeground)">⚠ \${d.errorMessage || 'Has error'}</span>\`;
      }

      tooltip.innerHTML = content;
      tooltip.style.left = (event.pageX + 10) + 'px';
      tooltip.style.top = (event.pageY + 10) + 'px';
      tooltip.style.opacity = 1;
    }

    function hideTooltip() {
      tooltip.style.opacity = 0;
    }

    // 工具栏按钮
    document.getElementById('refreshBtn').addEventListener('click', () => {
      vscode.postMessage({ command: 'refresh' });
    });

    document.getElementById('zoomInBtn').addEventListener('click', () => {
      svg.transition().call(zoom.scaleBy, 1.3);
    });

    document.getElementById('zoomOutBtn').addEventListener('click', () => {
      svg.transition().call(zoom.scaleBy, 0.7);
    });

    document.getElementById('resetBtn').addEventListener('click', () => {
      svg.transition().call(
        zoom.transform,
        d3.zoomIdentity.translate(0, 0).scale(1)
      );
    });
  </script>
</body>
</html>`;
  }

  /**
   * 关闭视图
   */
  public dispose(): void {
    this.panel?.dispose();
    this.panel = undefined;
    this.currentApp = undefined;
  }
}
