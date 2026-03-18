//! 依赖注入验证模块
//!
//! 提供依赖注入的验证功能，包括：
//! - 组件注册验证
//! - 组件类型存在性验证
//! - 组件名称匹配验证
//! - 循环依赖检测
//! - 配置注入验证

use crate::analysis::rust::macro_analyzer::{InjectMacro, InjectType, RustDocument, SummerMacro};
use crate::analysis::toml::toml_analyzer::TomlDocument;
use crate::core::index::IndexManager;
use lsp_types::{Diagnostic, DiagnosticSeverity, Location, NumberOrString};
use std::collections::{HashMap, HashSet};

/// 依赖注入验证器
pub struct DependencyInjectionValidator {
    /// 索引管理器
    index_manager: IndexManager,
}

impl DependencyInjectionValidator {
    /// 创建新的依赖注入验证器
    pub fn new(index_manager: IndexManager) -> Self {
        Self { index_manager }
    }

    /// 验证依赖注入
    ///
    /// 验证 Rust 文档中的所有依赖注入，包括：
    /// - 组件注册验证
    /// - 组件类型存在性验证
    /// - 组件名称匹配验证
    /// - 循环依赖检测
    /// - 配置注入验证
    ///
    /// # Arguments
    ///
    /// * `rust_docs` - Rust 文档列表
    /// * `toml_docs` - TOML 配置文档列表（包含 URI 和文档内容）
    ///
    /// # Returns
    ///
    /// 返回诊断信息列表
    ///
    /// # Requirements
    ///
    /// - 11.1: 组件注册验证
    /// - 11.2: 组件类型存在性验证
    /// - 11.3: 组件名称匹配验证
    /// - 11.4: 循环依赖检测
    /// - 11.5: 配置注入验证
    pub fn validate(
        &self,
        rust_docs: &[RustDocument],
        toml_docs: &[(lsp_types::Url, TomlDocument)],
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // 提取所有服务和注入信息
        let services = self.extract_services(rust_docs);

        // 验证每个服务的依赖注入
        for (service_name, service_info) in &services {
            for field in &service_info.fields {
                if let Some(inject) = &field.inject {
                    match inject.inject_type {
                        InjectType::Component => {
                            // 验证组件注入
                            diagnostics.extend(self.validate_component_injection(
                                service_name,
                                field,
                                inject,
                                &service_info.location,
                            ));
                        }
                        InjectType::Config => {
                            // 验证配置注入
                            diagnostics.extend(self.validate_config_injection(
                                field,
                                inject,
                                toml_docs,
                                &service_info.location,
                            ));
                        }
                    }
                }
            }
        }

        // 检测循环依赖
        diagnostics.extend(self.detect_circular_dependencies(&services));

        diagnostics
    }

    /// 验证组件注入
    ///
    /// # Requirements
    ///
    /// - 11.1: 验证组件是否已注册
    /// - 11.2: 验证组件类型是否存在
    /// - 11.3: 验证组件名称是否匹配
    fn validate_component_injection(
        &self,
        _service_name: &str,
        field: &FieldInfo,
        inject: &InjectMacro,
        location: &Location,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // 获取组件名称（如果指定）
        let component_name = inject.component_name.as_deref().unwrap_or(&field.type_name);

        // 验证组件是否已注册（需求 11.1）
        if let Some(component_info) = self.index_manager.find_component(component_name) {
            // 组件已注册，验证类型是否匹配
            if component_info.type_name != field.type_name {
                diagnostics.push(Diagnostic {
                    range: location.range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String(
                        "component-type-mismatch".to_string(),
                    )),
                    message: format!(
                        "组件 '{}' 的类型不匹配。期望类型: {}，实际类型: {}",
                        component_name, field.type_name, component_info.type_name
                    ),
                    source: Some("summer-lsp".to_string()),
                    ..Default::default()
                });
            }
        } else {
            // 组件未注册，检查类型是否存在（需求 11.2）
            let symbols = self.index_manager.find_symbol(&field.type_name);
            if symbols.is_empty() {
                diagnostics.push(Diagnostic {
                    range: location.range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String(
                        "component-type-not-found".to_string(),
                    )),
                    message: format!(
                        "组件类型 '{}' 不存在。请确保该类型已定义。",
                        field.type_name
                    ),
                    source: Some("summer-lsp".to_string()),
                    ..Default::default()
                });
            } else {
                // 类型存在但组件未注册（需求 11.1）
                diagnostics.push(Diagnostic {
                    range: location.range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String(
                        "component-not-registered".to_string(),
                    )),
                    message: format!(
                        "组件 '{}' 未注册。请确保该组件已通过插件注册。",
                        component_name
                    ),
                    source: Some("summer-lsp".to_string()),
                    ..Default::default()
                });
            }
        }

        // 如果指定了组件名称，验证名称是否匹配（需求 11.3）
        if let Some(specified_name) = &inject.component_name {
            if let Some(component_info) = self.index_manager.find_component(specified_name) {
                // 组件存在，验证类型是否匹配
                if component_info.type_name != field.type_name {
                    // 获取所有可用的同类型组件
                    let available_components =
                        self.get_available_components_by_type(&field.type_name);

                    let suggestion = if available_components.is_empty() {
                        String::new()
                    } else {
                        format!(
                            "\n可用的 {} 类型组件: {}",
                            field.type_name,
                            available_components.join(", ")
                        )
                    };

                    diagnostics.push(Diagnostic {
                        range: location.range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String(
                            "component-name-mismatch".to_string(),
                        )),
                        message: format!(
                            "组件名称 '{}' 的类型不匹配。期望类型: {}，实际类型: {}{}",
                            specified_name, field.type_name, component_info.type_name, suggestion
                        ),
                        source: Some("summer-lsp".to_string()),
                        ..Default::default()
                    });
                }
            } else {
                // 指定的组件名称不存在
                let available_components = self.get_available_components_by_type(&field.type_name);

                let suggestion = if available_components.is_empty() {
                    String::new()
                } else {
                    format!(
                        "\n可用的 {} 类型组件: {}",
                        field.type_name,
                        available_components.join(", ")
                    )
                };

                diagnostics.push(Diagnostic {
                    range: location.range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String(
                        "component-name-not-found".to_string(),
                    )),
                    message: format!("组件名称 '{}' 不存在。{}", specified_name, suggestion),
                    source: Some("summer-lsp".to_string()),
                    ..Default::default()
                });
            }
        }

        diagnostics
    }

    /// 验证配置注入
    ///
    /// # Requirements
    ///
    /// - 11.5: 验证配置项是否存在
    fn validate_config_injection(
        &self,
        field: &FieldInfo,
        _inject: &InjectMacro,
        toml_docs: &[(lsp_types::Url, TomlDocument)],
        location: &Location,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // 从类型名称中提取配置前缀
        // 例如：UserConfig -> user, DatabaseConfig -> database
        let config_prefix = self.extract_config_prefix(&field.type_name);

        // 检查配置是否存在
        let mut config_found = false;
        let mut config_file_uri = None;

        for (uri, toml_doc) in toml_docs {
            if toml_doc.config_sections.contains_key(&config_prefix) {
                config_found = true;
                config_file_uri = Some(uri.clone());
                break;
            }
        }

        if !config_found {
            let message = if let Some(uri) = &config_file_uri {
                format!(
                    "配置项 '{}' 不存在。请在配置文件 {} 中添加 [{}] 配置节。",
                    config_prefix, uri, config_prefix
                )
            } else {
                format!(
                    "配置项 '{}' 不存在。请在配置文件中添加 [{}] 配置节。",
                    config_prefix, config_prefix
                )
            };

            diagnostics.push(Diagnostic {
                range: location.range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("config-not-found".to_string())),
                message,
                source: Some("summer-lsp".to_string()),
                related_information: config_file_uri.map(|uri| {
                    vec![lsp_types::DiagnosticRelatedInformation {
                        location: Location {
                            uri,
                            range: lsp_types::Range {
                                start: lsp_types::Position {
                                    line: 0,
                                    character: 0,
                                },
                                end: lsp_types::Position {
                                    line: 0,
                                    character: 0,
                                },
                            },
                        },
                        message: format!("在此文件中添加 [{}] 配置节", config_prefix),
                    }]
                }),
                ..Default::default()
            });
        }

        diagnostics
    }

    /// 检测循环依赖
    ///
    /// # Requirements
    ///
    /// - 11.4: 检测循环依赖并建议使用 LazyComponent
    fn detect_circular_dependencies(
        &self,
        services: &HashMap<String, ServiceInfo>,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // 构建依赖图
        let mut dependency_graph: HashMap<String, Vec<String>> = HashMap::new();
        for (service_name, service_info) in services {
            let mut dependencies = Vec::new();
            for field in &service_info.fields {
                if let Some(inject) = &field.inject {
                    if inject.inject_type == InjectType::Component {
                        // 检查是否是 LazyComponent
                        if !field.type_name.contains("LazyComponent") {
                            dependencies.push(field.type_name.clone());
                        }
                    }
                }
            }
            dependency_graph.insert(service_name.clone(), dependencies);
        }

        // 使用 DFS 检测循环
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for service_name in services.keys() {
            if !visited.contains(service_name) {
                if let Some(cycle) = self.detect_cycle_dfs(
                    service_name,
                    &dependency_graph,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                ) {
                    // 找到循环依赖
                    if let Some(service_info) = services.get(service_name) {
                        diagnostics.push(Diagnostic {
                            range: service_info.location.range,
                            severity: Some(DiagnosticSeverity::WARNING),
                            code: Some(NumberOrString::String("circular-dependency".to_string())),
                            message: format!(
                                "检测到循环依赖: {}。建议使用 LazyComponent<T> 打破循环。",
                                cycle.join(" -> ")
                            ),
                            source: Some("summer-lsp".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        diagnostics
    }

    /// DFS 检测循环依赖
    fn detect_cycle_dfs(
        &self,
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if let Some(cycle) =
                        self.detect_cycle_dfs(neighbor, graph, visited, rec_stack, path)
                    {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(neighbor) {
                    // 找到循环
                    let cycle_start = path.iter().position(|n| n == neighbor).unwrap();
                    let mut cycle = path[cycle_start..].to_vec();
                    cycle.push(neighbor.to_string());
                    return Some(cycle);
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
        None
    }

    /// 提取服务信息
    fn extract_services(&self, rust_docs: &[RustDocument]) -> HashMap<String, ServiceInfo> {
        let mut services = HashMap::new();

        for doc in rust_docs {
            for summer_macro in &doc.macros {
                if let SummerMacro::DeriveService(service_macro) = summer_macro {
                    let service_info = ServiceInfo {
                        name: service_macro.struct_name.clone(),
                        fields: service_macro
                            .fields
                            .iter()
                            .map(|f| FieldInfo {
                                name: f.name.clone(),
                                type_name: f.type_name.clone(),
                                inject: f.inject.clone(),
                            })
                            .collect(),
                        location: Location {
                            uri: doc.uri.clone(),
                            range: service_macro.range,
                        },
                    };
                    services.insert(service_macro.struct_name.clone(), service_info);
                }
            }
        }

        services
    }

    /// 提取配置前缀
    ///
    /// 从类型名称中提取配置前缀
    /// 例如：UserConfig -> user, DatabaseConfig -> database
    fn extract_config_prefix(&self, type_name: &str) -> String {
        // 移除 "Config" 后缀
        let prefix = type_name.strip_suffix("Config").unwrap_or(type_name);

        // 转换为小写并用连字符分隔
        // 例如：UserProfile -> user-profile
        let mut result = String::new();
        for (i, ch) in prefix.chars().enumerate() {
            if i > 0 && ch.is_uppercase() {
                result.push('-');
            }
            result.push(ch.to_lowercase().next().unwrap());
        }

        result
    }

    /// 获取指定类型的所有可用组件
    fn get_available_components_by_type(&self, _type_name: &str) -> Vec<String> {
        // TODO: 实现从索引中查找所有指定类型的组件
        // 当前返回空列表
        Vec::new()
    }
}

/// 服务信息
struct ServiceInfo {
    /// 服务名称
    #[allow(dead_code)]
    name: String,
    /// 字段列表
    fields: Vec<FieldInfo>,
    /// 位置
    location: Location,
}

/// 字段信息
struct FieldInfo {
    /// 字段名称
    #[allow(dead_code)]
    name: String,
    /// 字段类型
    type_name: String,
    /// 注入宏
    inject: Option<InjectMacro>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_config_prefix() {
        let validator = DependencyInjectionValidator::new(IndexManager::new());

        assert_eq!(validator.extract_config_prefix("UserConfig"), "user");
        assert_eq!(
            validator.extract_config_prefix("DatabaseConfig"),
            "database"
        );
        assert_eq!(
            validator.extract_config_prefix("UserProfileConfig"),
            "user-profile"
        );
        assert_eq!(validator.extract_config_prefix("Config"), "");
        assert_eq!(validator.extract_config_prefix("User"), "user");
    }

    #[test]
    fn test_dependency_injection_validator_new() {
        let index_manager = IndexManager::new();
        let validator = DependencyInjectionValidator::new(index_manager);

        // 验证可以创建验证器
        let diagnostics = validator.validate(&[], &[]);
        assert_eq!(diagnostics.len(), 0);
    }
}
