/**
 * LSP 相关类型定义
 * 
 * 定义与语言服务器通信的请求和响应类型
 */

import { Location, ComponentScope, ComponentSource, JobType } from './common';

/**
 * Component 接口
 * 
 * 表示一个 Summer RS 组件
 */
export interface Component {
  /**
   * 组件名称
   */
  name: string;

  /**
   * 组件类型（完整的 Rust 类型名）
   */
  typeName: string;

  /**
   * 作用域（Singleton 或 Prototype）
   */
  scope: ComponentScope;

  /**
   * 组件定义方式（Service 或 Component）
   */
  source: ComponentSource;

  /**
   * 依赖的组件类型名列表
   */
  dependencies: string[];

  /**
   * 组件定义位置
   */
  location: Location;
}

/**
 * Route 接口
 * 
 * 表示一个 HTTP 路由
 */
export interface Route {
  /**
   * 路由名称（用于显示，通常是 handler 名称）
   */
  name: string;

  /**
   * HTTP 方法（GET, POST, PUT, DELETE, PATCH 等）
   */
  method: string;

  /**
   * 路由路径
   */
  path: string;

  /**
   * 处理器函数名称
   */
  handler: string;

  /**
   * 是否为 OpenAPI 路由
   */
  isOpenapi: boolean;

  /**
   * 路由定义位置（可选）
   */
  location?: Location;
}

/**
 * Configuration 接口
 * 
 * 表示一个配置项
 */
export interface Configuration {
  /**
   * 配置项名称（键）
   */
  name: string;

  /**
   * 配置项值
   */
  value?: string;

  /**
   * 所属配置节（如 web, database 等）
   */
  section?: string;

  /**
   * 配置项定义位置
   */
  location?: Location;
}

/**
 * ConfigurationStruct 接口
 * 
 * 表示一个配置结构体（带有 #[derive(Configurable)] 的结构体）
 */
export interface ConfigurationStruct {
  /**
   * 结构体名称
   */
  name: string;

  /**
   * 配置前缀（从 #[config_prefix = "..."] 提取）
   */
  prefix: string;

  /**
   * 字段列表
   */
  fields: ConfigField[];

  /**
   * 定义位置
   */
  location?: Location;
}

/**
 * ConfigField 接口
 * 
 * 表示配置结构体的一个字段
 */
export interface ConfigField {
  /**
   * 字段名称
   */
  name: string;

  /**
   * 字段类型
   */
  type: string;

  /**
   * 是否可选
   */
  optional: boolean;

  /**
   * 描述（从文档注释提取）
   */
  description?: string;
}

/**
 * Job 接口
 * 
 * 表示一个定时任务
 */
export interface Job {
  /**
   * 任务名称（函数名）
   */
  name: string;

  /**
   * 任务类型（Cron, FixDelay, FixRate）
   */
  jobType: JobType;

  /**
   * 调度表达式或间隔
   */
  schedule: string;

  /**
   * 源代码位置
   */
  location: Location;
}

/**
 * Plugin 接口
 * 
 * 表示一个已加载的插件
 */
export interface Plugin {
  /**
   * 插件名称
   */
  name: string;

  /**
   * 插件版本
   */
  version?: string;

  /**
   * 插件描述
   */
  description?: string;

  /**
   * 是否已启用
   */
  enabled: boolean;

  /**
   * 插件定义位置
   */
  location?: Location;
}

/**
 * LSP 请求：获取组件列表
 */
export interface ComponentsRequest {
  /**
   * 应用路径
   */
  appPath: string;
}

/**
 * LSP 响应：组件列表
 */
export interface ComponentsResponse {
  /**
   * 组件列表
   */
  components: Component[];
}

/**
 * LSP 请求：获取路由列表
 */
export interface RoutesRequest {
  /**
   * 应用路径
   */
  appPath: string;
}

/**
 * LSP 响应：路由列表
 */
export interface RoutesResponse {
  /**
   * 路由列表
   */
  routes: Route[];
}

/**
 * LSP 请求：获取配置列表
 */
export interface ConfigurationsRequest {
  /**
   * 应用路径
   */
  appPath: string;
}

/**
 * LSP 响应：配置列表
 */
export interface ConfigurationsResponse {
  /**
   * 配置结构体列表
   */
  configurations: ConfigurationStruct[];
}

/**
 * LSP 请求：获取任务列表
 */
export interface JobsRequest {
  /**
   * 应用路径
   */
  appPath: string;
}

/**
 * LSP 响应：任务列表
 */
export interface JobsResponse {
  /**
   * 任务列表
   */
  jobs: Job[];
}

/**
 * LSP 请求：获取插件列表
 */
export interface PluginsRequest {
  /**
   * 应用路径
   */
  appPath: string;
}

/**
 * LSP 响应：插件列表
 */
export interface PluginsResponse {
  /**
   * 插件列表
   */
  plugins: Plugin[];
}

/**
 * 依赖图节点
 */
export interface DependencyNode {
  /**
   * 节点 ID（组件名称）
   */
  id: string;

  /**
   * 组件名称
   */
  name: string;

  /**
   * 组件类型
   */
  typeName: string;

  /**
   * 是否有错误（循环依赖等）
   */
  hasError: boolean;
}

/**
 * 依赖图边
 */
export interface DependencyEdge {
  /**
   * 源节点 ID
   */
  source: string;

  /**
   * 目标节点 ID
   */
  target: string;
}

/**
 * LSP 请求：获取依赖图
 */
export interface DependencyGraphRequest {
  /**
   * 应用路径
   */
  appPath: string;
}

/**
 * LSP 响应：依赖图
 */
export interface DependencyGraphResponse {
  /**
   * 节点列表
   */
  nodes: DependencyNode[];

  /**
   * 边列表
   */
  edges: DependencyEdge[];
}
