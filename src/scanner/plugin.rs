//! 插件扫描器模块
//!
//! 扫描项目中的所有插件注册（.add_plugin() 调用）

use crate::protocol::types::{LocationResponse, PositionResponse, RangeResponse};

use lsp_types::Url;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 插件扫描器
pub struct PluginScanner;

impl PluginScanner {
    /// 创建新的插件扫描器
    pub fn new() -> Self {
        Self
    }

    /// 扫描项目中的所有插件
    ///
    /// # Arguments
    ///
    /// * `project_path` - 项目根目录路径
    ///
    /// # Returns
    ///
    /// 返回扫描到的所有插件信息
    pub fn scan_plugins(&self, project_path: &Path) -> Result<Vec<PluginInfoResponse>, ScanError> {
        let mut plugins = Vec::new();

        // 查找 main.rs
        let main_path = project_path.join("src/main.rs");
        if !main_path.exists() {
            // 如果没有 main.rs，尝试 lib.rs
            let lib_path = project_path.join("src/lib.rs");
            if !lib_path.exists() {
                return Err(ScanError::InvalidProject(
                    "Neither main.rs nor lib.rs found".to_string(),
                ));
            }
            self.scan_file(&lib_path, &mut plugins)?;
            return Ok(plugins);
        }

        self.scan_file(&main_path, &mut plugins)?;
        Ok(plugins)
    }

    /// 扫描单个文件中的插件
    fn scan_file(
        &self,
        file_path: &Path,
        plugins: &mut Vec<PluginInfoResponse>,
    ) -> Result<(), ScanError> {
        let content = fs::read_to_string(file_path)?;
        let file_url = Url::from_file_path(file_path)
            .map_err(|_| ScanError::InvalidProject("Failed to convert path to URL".to_string()))?;

        // 简单的文本搜索 .add_plugin() 调用
        // TODO: 使用 syn 进行更精确的 AST 分析
        for (line_num, line) in content.lines().enumerate() {
            if line.contains(".add_plugin(") {
                // 提取插件类型名
                if let Some(plugin_name) = self.extract_plugin_name(line) {
                    plugins.push(PluginInfoResponse {
                        name: plugin_name.clone(),
                        type_name: plugin_name,
                        config_prefix: None, // TODO: 从配置中推断
                        location: LocationResponse {
                            uri: file_url.to_string(),
                            range: RangeResponse {
                                start: PositionResponse {
                                    line: line_num as u32,
                                    character: 0,
                                },
                                end: PositionResponse {
                                    line: line_num as u32,
                                    character: line.len() as u32,
                                },
                            },
                        },
                    });
                }
            }
        }

        Ok(())
    }

    /// 从代码行中提取插件名称
    fn extract_plugin_name(&self, line: &str) -> Option<String> {
        // 查找 .add_plugin( 后面的内容
        if let Some(start) = line.find(".add_plugin(") {
            let after = &line[start + 12..]; // ".add_plugin(" 长度为 12

            // 查找第一个非空白字符
            let trimmed = after.trim_start();

            // 提取插件类型名（到第一个括号、逗号或空白）
            let end = trimmed
                .find(|c: char| c == '(' || c == ')' || c == ',' || c.is_whitespace())
                .unwrap_or(trimmed.len());

            let plugin_name = &trimmed[..end];
            if !plugin_name.is_empty() {
                return Some(plugin_name.to_string());
            }
        }
        None
    }
}

impl Default for PluginScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// 插件信息响应（用于 JSON 序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfoResponse {
    /// 插件名称
    pub name: String,
    /// 插件类型名
    #[serde(rename = "typeName")]
    pub type_name: String,
    /// 配置前缀（如果有）
    #[serde(rename = "configPrefix")]
    pub config_prefix: Option<String>,
    /// 源代码位置
    pub location: LocationResponse,
}

/// summer/plugins 请求参数
#[derive(Debug, Deserialize)]
pub struct PluginsRequest {
    /// 应用路径
    #[serde(rename = "appPath")]
    pub app_path: String,
}

/// summer/plugins 响应
#[derive(Debug, Serialize)]
pub struct PluginsResponse {
    /// 插件列表
    pub plugins: Vec<PluginInfoResponse>,
}

/// 扫描错误
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("Failed to read file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Invalid project structure: {0}")]
    InvalidProject(String),

    #[error("No plugins found")]
    NoPlugins,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_scanner_new() {
        let _scanner = PluginScanner::new();
        // 验证扫描器创建成功（不会 panic）
    }

    #[test]
    fn test_extract_plugin_name() {
        let scanner = PluginScanner::new();

        assert_eq!(
            scanner.extract_plugin_name("    .add_plugin(WebPlugin)"),
            Some("WebPlugin".to_string())
        );

        assert_eq!(
            scanner.extract_plugin_name(".add_plugin(SqlxPlugin::new())"),
            Some("SqlxPlugin::new".to_string())
        );
    }
}
