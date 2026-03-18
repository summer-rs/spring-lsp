import * as vscode from 'vscode';
import * as path from 'path';
import { ViewMode } from '../types/viewMode';

/**
 * 带位置信息的项目接口
 */
export interface ItemWithLocation {
  name: string;
  location?: {
    uri: string;
    range: {
      start: { line: number; character: number };
      end: { line: number; character: number };
    };
  };
}

/**
 * 文件树节点
 */
export class FileTreeNode extends vscode.TreeItem {
  public readonly filePath: string;
  public readonly items: ItemWithLocation[];

  constructor(
    fileUri: string,
    items: ItemWithLocation[],
    itemTypeName: string = 'items'
  ) {
    const uri = vscode.Uri.parse(fileUri);
    const fileName = path.basename(uri.fsPath);
    const dirName = path.basename(path.dirname(uri.fsPath));
    
    super(fileName, vscode.TreeItemCollapsibleState.Expanded);

    this.filePath = uri.fsPath;
    this.items = items;

    // 设置描述
    this.description = dirName;

    // 设置上下文值
    this.contextValue = 'summer:file';

    // 设置工具提示
    this.tooltip = new vscode.MarkdownString(
      `**File:** \`${fileName}\`\n\n` +
      `**Path:** ${uri.fsPath}\n\n` +
      `**${itemTypeName}:** ${items.length}`
    );

    // 设置图标
    this.iconPath = new vscode.ThemeIcon('file-code');

    // 设置点击命令
    this.command = {
      command: 'vscode.open',
      title: 'Open File',
      arguments: [uri],
    };
  }
}

/**
 * 按文件组织项目的辅助函数
 */
export function groupByFile<T extends ItemWithLocation>(
  items: T[]
): Map<string, T[]> {
  const fileMap = new Map<string, T[]>();
  
  for (const item of items) {
    if (!item.location) {
      continue;
    }

    const fileUri = item.location.uri;
    if (!fileMap.has(fileUri)) {
      fileMap.set(fileUri, []);
    }
    fileMap.get(fileUri)!.push(item);
  }

  return fileMap;
}

/**
 * 创建文件树节点列表
 */
export function createFileTreeNodes<T extends ItemWithLocation>(
  items: T[],
  itemTypeName: string = 'items'
): FileTreeNode[] {
  const fileMap = groupByFile(items);
  const fileNodes: FileTreeNode[] = [];

  for (const [fileUri, fileItems] of fileMap.entries()) {
    fileNodes.push(new FileTreeNode(fileUri, fileItems, itemTypeName));
  }

  // 按文件路径排序
  fileNodes.sort((a, b) => a.filePath.localeCompare(b.filePath));

  return fileNodes;
}
