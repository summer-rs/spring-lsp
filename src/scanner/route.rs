//! 路由扫描器模块
//!
//! 扫描项目中的所有路由定义

use crate::analysis::rust::macro_analyzer::{MacroAnalyzer, SummerMacro};
use crate::protocol::types::{LocationResponse, PositionResponse, RangeResponse};
use lsp_types::Url;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// 路由扫描器
pub struct RouteScanner {
    macro_analyzer: MacroAnalyzer,
}

impl RouteScanner {
    /// 创建新的路由扫描器
    pub fn new() -> Self {
        Self {
            macro_analyzer: MacroAnalyzer::new(),
        }
    }

    /// 扫描项目中的所有路由
    ///
    /// # Arguments
    ///
    /// * `project_path` - 项目根目录路径
    ///
    /// # Returns
    ///
    /// 返回扫描到的所有路由信息
    pub fn scan_routes(&self, project_path: &Path) -> Result<Vec<RouteInfoResponse>, ScanError> {
        let mut routes = Vec::new();

        // 查找 src 目录
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Err(ScanError::InvalidProject(
                "src directory not found".to_string(),
            ));
        }

        // 遍历所有 Rust 文件
        for entry in WalkDir::new(&src_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            let file_path = entry.path();

            // 读取文件内容
            let content = match fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(e) => {
                    tracing::warn!("Failed to read file {:?}: {}", file_path, e);
                    continue;
                }
            };

            // 解析文件
            let file_url = match Url::from_file_path(file_path) {
                Ok(url) => url,
                Err(_) => {
                    tracing::warn!("Failed to convert path to URL: {:?}", file_path);
                    continue;
                }
            };

            let rust_doc = match self.macro_analyzer.parse(file_url.clone(), content) {
                Ok(doc) => doc,
                Err(e) => {
                    tracing::warn!("Failed to parse file {:?}: {}", file_path, e);
                    continue;
                }
            };

            // 提取宏信息
            let rust_doc = match self.macro_analyzer.extract_macros(rust_doc) {
                Ok(doc) => doc,
                Err(e) => {
                    tracing::warn!("Failed to extract macros from {:?}: {}", file_path, e);
                    continue;
                }
            };

            // 提取路由信息
            for summer_macro in &rust_doc.macros {
                if let SummerMacro::Route(route_macro) = summer_macro {
                    // 为每个 HTTP 方法创建独立的路由条目
                    for method in &route_macro.methods {
                        routes.push(RouteInfoResponse {
                            method: method.as_str().to_string(),
                            path: route_macro.path.clone(),
                            handler: route_macro.handler_name.clone(),
                            is_openapi: route_macro.is_openapi,
                            location: LocationResponse {
                                uri: file_url.to_string(),
                                range: RangeResponse {
                                    start: PositionResponse {
                                        line: route_macro.range.start.line,
                                        character: route_macro.range.start.character,
                                    },
                                    end: PositionResponse {
                                        line: route_macro.range.end.line,
                                        character: route_macro.range.end.character,
                                    },
                                },
                            },
                        });
                    }
                }
            }
        }

        Ok(routes)
    }
}

impl Default for RouteScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// 路由信息响应（用于 JSON 序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfoResponse {
    /// HTTP 方法
    pub method: String,
    /// 路径模式
    pub path: String,
    /// 处理器函数名
    pub handler: String,
    /// 是否为 OpenAPI 路由
    #[serde(rename = "isOpenapi")]
    pub is_openapi: bool,
    /// 源代码位置
    pub location: LocationResponse,
}

/// summer/routes 请求参数
#[derive(Debug, Deserialize)]
pub struct RoutesRequest {
    /// 应用路径
    #[serde(rename = "appPath")]
    pub app_path: String,
}

/// summer/routes 响应
#[derive(Debug, Serialize)]
pub struct RoutesResponse {
    /// 路由列表
    pub routes: Vec<RouteInfoResponse>,
}

/// 扫描错误
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("Failed to read file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Invalid project structure: {0}")]
    InvalidProject(String),

    #[error("No routes found")]
    NoRoutes,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_scanner_new() {
        let _scanner = RouteScanner::new();
        // 验证扫描器创建成功（不会 panic）
    }

    #[test]
    fn test_route_scanner_default() {
        let _scanner = RouteScanner::default();
        // 验证默认扫描器创建成功（不会 panic）
    }
}

// ============================================================================
// 路由类型定义和导航器
// ============================================================================

use lsp_types::Location;
use std::collections::HashMap;

/// 路由导航器
///
/// 提供路由相关的导航功能，如跳转到处理器定义
pub struct RouteNavigator {
    // TODO: 添加字段
}

impl RouteNavigator {
    /// 创建新的路由导航器
    pub fn new() -> Self {
        Self {}
    }

    /// 查找路由处理器的定义位置
    pub fn find_handler_location(&self, _route_path: &str) -> Option<Location> {
        // TODO: 实现查找逻辑
        None
    }
}

impl Default for RouteNavigator {
    fn default() -> Self {
        Self::new()
    }
}

/// 路由索引
///
/// 维护项目中所有路由的索引
pub struct RouteIndex {
    /// 路由映射：路径 -> 路由信息
    routes: HashMap<String, Route>,
}

impl RouteIndex {
    /// 创建新的路由索引
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    /// 添加路由
    pub fn add_route(&mut self, route: Route) {
        let key = format!("{} {}", route.method.as_str(), route.path);
        self.routes.insert(key, route);
    }

    /// 查找路由
    pub fn find_route(&self, method: HttpMethod, path: &str) -> Option<&Route> {
        let key = format!("{} {}", method.as_str(), path);
        self.routes.get(&key)
    }

    /// 获取所有路由
    pub fn all_routes(&self) -> Vec<&Route> {
        self.routes.values().collect()
    }
}

impl Default for RouteIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP 方法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
        }
    }
}

/// 路由信息
#[derive(Debug, Clone)]
pub struct Route {
    /// HTTP 方法
    pub method: HttpMethod,
    /// 路径模式
    pub path: String,
    /// 处理器函数名
    pub handler: String,
    /// 源代码位置
    pub location: Location,
}
