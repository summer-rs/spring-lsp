//! 组件扫描器模块
//!
//! 扫描项目中的所有组件定义：
//! - 带有 #[derive(Service)] 的结构体
//! - 带有 #[component] 的函数

use crate::analysis::rust::macro_analyzer::{MacroAnalyzer, ServiceScope, SummerMacro};
use crate::protocol::types::{LocationResponse, PositionResponse, RangeResponse};
use lsp_types::Url;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// 组件扫描器
pub struct ComponentScanner {
    macro_analyzer: MacroAnalyzer,
}

impl ComponentScanner {
    /// 创建新的组件扫描器
    pub fn new() -> Self {
        Self {
            macro_analyzer: MacroAnalyzer::new(),
        }
    }

    /// 扫描项目中的所有组件
    ///
    /// # Arguments
    ///
    /// * `project_path` - 项目根目录路径（可以是 workspace 根目录或具体项目目录）
    ///
    /// # Returns
    ///
    /// 返回扫描到的所有组件信息
    pub fn scan_components(
        &self,
        project_path: &Path,
    ) -> Result<Vec<ComponentInfoResponse>, ScanError> {
        tracing::info!("Starting component scan in: {:?}", project_path);

        let mut components = Vec::new();

        // 检查是否直接是一个 summer-rs 项目（有 src 目录）
        let src_path = project_path.join("src");

        if src_path.exists() && src_path.is_dir() {
            // 直接扫描这个项目
            tracing::info!("Found src directory, scanning single project");
            components.extend(self.scan_single_project(project_path)?);
        } else {
            // 可能是 workspace 根目录，递归查找所有 summer-rs 项目
            tracing::info!("No src directory found, searching for summer-rs projects in workspace");
            components.extend(self.scan_workspace(project_path)?);
        }

        tracing::info!("Total components found: {}", components.len());
        Ok(components)
    }

    /// 扫描单个项目
    fn scan_single_project(
        &self,
        project_path: &Path,
    ) -> Result<Vec<ComponentInfoResponse>, ScanError> {
        let mut components = Vec::new();

        // 查找 src 目录
        let src_path = project_path.join("src");
        tracing::info!("Looking for src directory: {:?}", src_path);

        if !src_path.exists() {
            tracing::error!("src directory not found at: {:?}", src_path);
            return Err(ScanError::InvalidProject(
                "src directory not found".to_string(),
            ));
        }

        tracing::info!("Found src directory, starting file scan...");
        let mut file_count = 0;
        let mut parsed_count = 0;
        let mut macro_count = 0;

        // 遍历所有 Rust 文件
        for entry in WalkDir::new(&src_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            file_count += 1;
            let file_path = entry.path();
            tracing::info!("Scanning file {}: {:?}", file_count, file_path);

            // 读取文件内容
            let content = match fs::read_to_string(file_path) {
                Ok(content) => {
                    tracing::info!("Successfully read file, size: {} bytes", content.len());
                    content
                }
                Err(e) => {
                    tracing::warn!("Failed to read file {:?}: {}", file_path, e);
                    continue;
                }
            };

            // 解析文件
            let file_url = match Url::from_file_path(file_path) {
                Ok(url) => {
                    tracing::info!("Converted to URL: {}", url);
                    url
                }
                Err(_) => {
                    tracing::warn!("Failed to convert path to URL: {:?}", file_path);
                    continue;
                }
            };

            let rust_doc = match self.macro_analyzer.parse(file_url.clone(), content) {
                Ok(doc) => {
                    parsed_count += 1;
                    tracing::info!("Successfully parsed file");
                    doc
                }
                Err(e) => {
                    tracing::warn!("Failed to parse file {:?}: {}", file_path, e);
                    continue;
                }
            };

            // 提取宏信息
            let rust_doc = match self.macro_analyzer.extract_macros(rust_doc) {
                Ok(doc) => {
                    tracing::info!("Extracted {} macros from file", doc.macros.len());
                    macro_count += doc.macros.len();
                    doc
                }
                Err(e) => {
                    tracing::warn!("Failed to extract macros from {:?}: {}", file_path, e);
                    continue;
                }
            };

            // 提取组件信息
            for summer_macro in &rust_doc.macros {
                match summer_macro {
                    // 处理 #[derive(Service)] 宏
                    SummerMacro::DeriveService(service_macro) => {
                        tracing::info!(
                            "Found Service component: {} in {:?}",
                            service_macro.struct_name,
                            file_path
                        );

                        components.push(ComponentInfoResponse {
                            name: service_macro.struct_name.clone(),
                            type_name: service_macro.struct_name.clone(),
                            scope: match service_macro.scope {
                                ServiceScope::Singleton => ComponentScope::Singleton,
                                ServiceScope::Prototype => ComponentScope::Prototype,
                            },
                            source: ComponentSource::Service,
                            dependencies: service_macro
                                .fields
                                .iter()
                                .filter_map(|field| {
                                    // 只包含带有 inject 标注的字段
                                    field.inject.as_ref().map(|_| field.type_name.clone())
                                })
                                .collect(),
                            location: LocationResponse {
                                uri: file_url.to_string(),
                                range: RangeResponse {
                                    start: PositionResponse {
                                        line: service_macro.range.start.line,
                                        character: service_macro.range.start.character,
                                    },
                                    end: PositionResponse {
                                        line: service_macro.range.end.line,
                                        character: service_macro.range.end.character,
                                    },
                                },
                            },
                        });
                    }
                    // 处理 #[component] 宏
                    SummerMacro::Component(component_macro) => {
                        tracing::info!(
                            "Found Component function: {} -> {} in {:?}",
                            component_macro.function_name,
                            component_macro.component_type,
                            file_path
                        );

                        components.push(ComponentInfoResponse {
                            name: component_macro.component_type.clone(),
                            type_name: component_macro.component_type.clone(),
                            scope: ComponentScope::Singleton, // summer-rs 默认是单例
                            source: ComponentSource::Component,
                            dependencies: component_macro
                                .dependencies
                                .iter()
                                .map(|dep| dep.type_name.clone())
                                .collect(),
                            location: LocationResponse {
                                uri: file_url.to_string(),
                                range: RangeResponse {
                                    start: PositionResponse {
                                        line: component_macro.range.start.line,
                                        character: component_macro.range.start.character,
                                    },
                                    end: PositionResponse {
                                        line: component_macro.range.end.line,
                                        character: component_macro.range.end.character,
                                    },
                                },
                            },
                        });
                    }
                    _ => {}
                }
            }
        }

        tracing::info!(
            "Scanned {} Rust files, parsed {} files, found {} macros, extracted {} components",
            file_count,
            parsed_count,
            macro_count,
            components.len()
        );
        Ok(components)
    }

    /// 扫描 workspace 中的所有 summer-rs 项目
    fn scan_workspace(
        &self,
        workspace_path: &Path,
    ) -> Result<Vec<ComponentInfoResponse>, ScanError> {
        let mut all_components = Vec::new();
        let mut project_count = 0;

        // 递归查找所有包含 Cargo.toml 的目录
        for entry in WalkDir::new(workspace_path)
            .max_depth(5) // 限制递归深度
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "Cargo.toml")
        {
            let cargo_toml_path = entry.path();
            let project_dir = cargo_toml_path.parent().unwrap();

            // 检查是否有 src 目录
            let src_dir = project_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            // 检查是否是 summer-rs 项目
            if !self.is_summer_rs_project(cargo_toml_path) {
                continue;
            }

            tracing::info!("Found summer-rs project: {:?}", project_dir);
            project_count += 1;

            // 扫描这个项目
            match self.scan_single_project(project_dir) {
                Ok(components) => {
                    tracing::info!("Found {} components in {:?}", components.len(), project_dir);
                    all_components.extend(components);
                }
                Err(e) => {
                    tracing::warn!("Failed to scan project {:?}: {}", project_dir, e);
                }
            }
        }

        tracing::info!("Scanned {} summer-rs projects in workspace", project_count);
        Ok(all_components)
    }

    /// 检查是否是 summer-rs 项目
    fn is_summer_rs_project(&self, cargo_toml_path: &Path) -> bool {
        // 读取 Cargo.toml
        let content = match fs::read_to_string(cargo_toml_path) {
            Ok(content) => content,
            Err(_) => return false,
        };

        // 简单检查是否包含 summer 依赖
        content.contains("summer")
            && (content.contains("summer-web")
                || content.contains("summer-sqlx")
                || content.contains("summer-redis")
                || content.contains("\"summer\""))
    }
}

impl Default for ComponentScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// 组件作用域
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentScope {
    /// 单例（默认）
    Singleton,
    /// 原型（每次注入创建新实例）
    Prototype,
}

/// 组件定义方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentSource {
    /// 使用 #[derive(Service)] 定义
    #[serde(rename = "service")]
    Service,
    /// 使用 #[component] 定义
    #[serde(rename = "component")]
    Component,
}

/// 组件信息响应（用于 JSON 序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfoResponse {
    /// 组件名称
    pub name: String,
    /// 组件类型名
    #[serde(rename = "typeName")]
    pub type_name: String,
    /// 作用域
    pub scope: ComponentScope,
    /// 组件来源（Service 或 Component）
    pub source: ComponentSource,
    /// 依赖列表
    pub dependencies: Vec<String>,
    /// 源代码位置
    pub location: LocationResponse,
}

/// summer/components 请求参数
#[derive(Debug, Deserialize)]
pub struct ComponentsRequest {
    /// 应用路径
    #[serde(rename = "appPath")]
    pub app_path: String,
}

/// summer/components 响应
#[derive(Debug, Serialize)]
pub struct ComponentsResponse {
    /// 组件列表
    pub components: Vec<ComponentInfoResponse>,
}

/// 扫描错误
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("Failed to read file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Invalid project structure: {0}")]
    InvalidProject(String),

    #[error("No components found")]
    NoComponents,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_scanner_new() {
        let _scanner = ComponentScanner::new();
        // 验证扫描器创建成功（不会 panic）
    }

    #[test]
    fn test_component_scanner_default() {
        let _scanner = ComponentScanner::default();
        // 验证默认扫描器创建成功（不会 panic）
    }
}
