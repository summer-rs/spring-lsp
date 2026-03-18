/**
 * 应用状态枚举
 * 
 * 定义 Summer RS 应用的所有可能状态
 */
export enum AppState {
  /**
   * 未激活状态 - 应用未运行
   */
  INACTIVE = 'inactive',

  /**
   * 启动中状态 - 应用正在启动
   */
  LAUNCHING = 'launching',

  /**
   * 运行中状态 - 应用正在运行
   */
  RUNNING = 'running',

  /**
   * 停止中状态 - 应用正在停止
   */
  STOPPING = 'stopping',
}

/**
 * 检查状态是否为活动状态（非 INACTIVE）
 */
export function isActiveState(state: AppState): boolean {
  return state !== AppState.INACTIVE;
}

/**
 * 检查状态是否为过渡状态（LAUNCHING 或 STOPPING）
 */
export function isTransitionState(state: AppState): boolean {
  return state === AppState.LAUNCHING || state === AppState.STOPPING;
}

/**
 * 获取状态的显示文本
 */
export function getStateDisplayText(state: AppState): string {
  switch (state) {
    case AppState.INACTIVE:
      return 'Inactive';
    case AppState.LAUNCHING:
      return 'Launching...';
    case AppState.RUNNING:
      return 'Running';
    case AppState.STOPPING:
      return 'Stopping...';
    default:
      return 'Unknown';
  }
}

/**
 * 获取状态对应的图标名称（VSCode ThemeIcon）
 */
export function getStateIcon(state: AppState): string {
  switch (state) {
    case AppState.INACTIVE:
      return 'circle-outline';
    case AppState.LAUNCHING:
      return 'loading~spin';
    case AppState.RUNNING:
      return 'debug-start';
    case AppState.STOPPING:
      return 'debug-stop';
    default:
      return 'circle-outline';
  }
}
