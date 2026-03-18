/**
 * 视图模式枚举
 */
export enum ViewMode {
  /**
   * List 模式（默认）
   * 直接显示所有组件/路由/配置的扁平列表
   */
  List = 'list',

  /**
   * Tree 模式
   * 按文件路径组织，显示文件树结构
   */
  Tree = 'tree',
}

/**
 * 各个视图的模式配置键
 */
export const VIEW_MODE_KEYS = {
  components: 'summer-lsp.componentsViewMode',
  routes: 'summer-lsp.routesViewMode',
  configurations: 'summer-lsp.configurationsViewMode',
} as const;
