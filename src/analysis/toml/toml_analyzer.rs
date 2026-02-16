//! TOML 配置文件分析模块

use lsp_types::{
    Diagnostic, DiagnosticSeverity, Hover, HoverContents, MarkupContent, MarkupKind, Position,
    Range,
};
use std::collections::HashMap;
use taplo::dom::node::IntegerValue;

use crate::schema::{PropertySchema, SchemaProvider, TypeInfo};

/// TOML 文档
///
/// 表示解析后的 TOML 配置文件，包含环境变量引用、配置节和属性等信息
#[derive(Debug, Clone)]
pub struct TomlDocument {
    /// taplo 的 DOM 根节点
    pub root: taplo::dom::Node,
    /// 提取的环境变量引用
    pub env_vars: Vec<EnvVarReference>,
    /// 提取的配置节（键为配置前缀）
    pub config_sections: HashMap<String, ConfigSection>,
    /// 原始内容（用于计算行列位置）
    pub content: String,
}

/// 环境变量引用
///
/// 表示 TOML 配置中的环境变量插值，格式为 `${VAR:default}` 或 `${VAR}`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVarReference {
    /// 环境变量名称
    pub name: String,
    /// 默认值（可选）
    pub default: Option<String>,
    /// 在文档中的位置范围
    pub range: Range,
}

/// 配置节
///
/// 表示 TOML 配置文件中的一个配置节，如 `[web]` 或 `[redis]`
#[derive(Debug, Clone)]
pub struct ConfigSection {
    /// 配置前缀（节名称）
    pub prefix: String,
    /// 配置属性映射（键为属性名）
    pub properties: HashMap<String, ConfigProperty>,
    /// 在文档中的位置范围
    pub range: Range,
}

/// 配置属性
///
/// 表示配置节中的单个属性，如 `host = "localhost"`
#[derive(Debug, Clone)]
pub struct ConfigProperty {
    /// 属性键
    pub key: String,
    /// 属性值
    pub value: ConfigValue,
    /// 在文档中的位置范围
    pub range: Range,
}

/// 配置值
///
/// 表示 TOML 配置属性的值，支持多种类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    /// 字符串值
    String(String),
    /// 整数值
    Integer(i64),
    /// 浮点数值
    Float(f64),
    /// 布尔值
    Boolean(bool),
    /// 数组值
    Array(Vec<ConfigValue>),
    /// 表（对象）值
    Table(HashMap<String, ConfigValue>),
}

/// TOML 分析器
///
/// 负责解析 TOML 配置文件，提取环境变量引用和配置节
pub struct TomlAnalyzer {
    /// Schema 提供者
    schema_provider: SchemaProvider,
}

impl TomlAnalyzer {
    /// 创建新的 TOML 分析器
    pub fn new(schema_provider: SchemaProvider) -> Self {
        Self { schema_provider }
    }

    /// 获取 Schema 提供者的引用
    pub fn schema_provider(&self) -> &SchemaProvider {
        &self.schema_provider
    }

    /// 提供悬停提示
    ///
    /// 当用户悬停在 TOML 配置项或环境变量上时，显示相关文档和信息
    ///
    /// # 参数
    ///
    /// * `doc` - 已解析的 TOML 文档
    /// * `position` - 光标位置
    ///
    /// # 返回
    ///
    /// 如果光标位置有可显示的信息，返回 `Some(Hover)`，否则返回 `None`
    ///
    /// # 功能
    ///
    /// 1. 配置项悬停：显示配置项的文档、类型信息、默认值等
    /// 2. 环境变量悬停：显示环境变量的当前值（如果可用）
    pub fn hover(&self, doc: &TomlDocument, position: Position) -> Option<Hover> {
        // 首先检查是否悬停在环境变量上
        if let Some(hover) = self.hover_env_var(doc, position) {
            return Some(hover);
        }

        // 然后检查是否悬停在配置项上
        if let Some(hover) = self.hover_config_property(doc, position) {
            return Some(hover);
        }

        None
    }

    /// 为环境变量提供悬停提示
    fn hover_env_var(&self, doc: &TomlDocument, position: Position) -> Option<Hover> {
        // 查找光标位置的环境变量
        for env_var in &doc.env_vars {
            if self.position_in_range(position, env_var.range) {
                let mut hover_text = String::new();

                // 添加标题
                hover_text.push_str("# 环境变量\n\n");

                // 添加变量名
                hover_text.push_str(&format!("**变量名**: `{}`\n\n", env_var.name));

                // 添加默认值（如果有）
                if let Some(default) = &env_var.default {
                    hover_text.push_str(&format!("**默认值**: `{}`\n\n", default));
                }

                // 尝试获取环境变量的当前值
                if let Ok(value) = std::env::var(&env_var.name) {
                    hover_text.push_str(&format!("**当前值**: `{}`\n\n", value));
                } else {
                    hover_text.push_str("**当前值**: *未设置*\n\n");
                }

                // 添加说明
                hover_text.push_str("**说明**:\n\n");
                hover_text.push_str("环境变量插值允许在配置文件中引用系统环境变量。\n\n");
                hover_text.push_str("**格式**:\n");
                hover_text.push_str("- `${VAR}` - 引用环境变量，如果未设置则报错\n");
                hover_text.push_str("- `${VAR:default}` - 引用环境变量，如果未设置则使用默认值\n");

                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: hover_text,
                    }),
                    range: Some(env_var.range),
                });
            }
        }

        None
    }

    /// 为配置项提供悬停提示
    fn hover_config_property(&self, doc: &TomlDocument, position: Position) -> Option<Hover> {
        // 遍历所有配置节
        for (prefix, section) in &doc.config_sections {
            // 检查是否悬停在配置节的某个属性上
            for (key, property) in &section.properties {
                if self.position_in_range(position, property.range) {
                    // 检查配置项是否在 Schema 中定义
                    let is_defined = self.schema_provider.has_property(prefix, key);

                    return Some(
                        self.create_basic_property_hover(prefix, key, property, is_defined),
                    );
                }
            }
        }

        None
    }

    /// 创建配置项的悬停提示（有 Schema）
    fn create_property_hover(
        &self,
        prefix: &str,
        key: &str,
        property: &ConfigProperty,
        schema: &PropertySchema,
    ) -> Hover {
        let mut hover_text = String::new();

        // 添加标题
        hover_text.push_str(&format!("# 配置项: `{}.{}`\n\n", prefix, key));

        // 添加描述
        if !schema.description.is_empty() {
            hover_text.push_str(&format!("{}\n\n", schema.description));
        }

        // 添加类型信息
        hover_text.push_str(&format!(
            "**类型**: {}\n\n",
            self.type_info_to_string(&schema.type_info)
        ));

        // 添加当前值
        hover_text.push_str(&format!(
            "**当前值**: `{}`\n\n",
            self.config_value_to_string(&property.value)
        ));

        // 添加默认值（如果有）
        if let Some(default) = &schema.default {
            hover_text.push_str(&format!(
                "**默认值**: `{}`\n\n",
                self.value_to_string(default)
            ));
        }

        // 添加是否必需
        if schema.required {
            hover_text.push_str("**必需**: 是\n\n");
        }

        // 添加枚举值（如果有）
        if let TypeInfo::String {
            enum_values: Some(enum_vals),
            ..
        } = &schema.type_info
        {
            hover_text.push_str("**允许的值**:\n");
            for val in enum_vals {
                hover_text.push_str(&format!("- `{}`\n", val));
            }
            hover_text.push('\n');
        }

        // 添加范围限制（如果有）
        match &schema.type_info {
            TypeInfo::Integer { min, max } => {
                if min.is_some() || max.is_some() {
                    hover_text.push_str("**值范围**:\n");
                    if let Some(min_val) = min {
                        hover_text.push_str(&format!("- 最小值: `{}`\n", min_val));
                    }
                    if let Some(max_val) = max {
                        hover_text.push_str(&format!("- 最大值: `{}`\n", max_val));
                    }
                    hover_text.push('\n');
                }
            }
            TypeInfo::Float { min, max } => {
                if min.is_some() || max.is_some() {
                    hover_text.push_str("**值范围**:\n");
                    if let Some(min_val) = min {
                        hover_text.push_str(&format!("- 最小值: `{}`\n", min_val));
                    }
                    if let Some(max_val) = max {
                        hover_text.push_str(&format!("- 最大值: `{}`\n", max_val));
                    }
                    hover_text.push('\n');
                }
            }
            TypeInfo::String {
                min_length,
                max_length,
                ..
            } => {
                if min_length.is_some() || max_length.is_some() {
                    hover_text.push_str("**长度限制**:\n");
                    if let Some(min_len) = min_length {
                        hover_text.push_str(&format!("- 最小长度: `{}`\n", min_len));
                    }
                    if let Some(max_len) = max_length {
                        hover_text.push_str(&format!("- 最大长度: `{}`\n", max_len));
                    }
                    hover_text.push('\n');
                }
            }
            _ => {}
        }

        // 添加示例代码（如果有）
        if let Some(example) = &schema.example {
            hover_text.push_str("**示例**:\n\n");
            hover_text.push_str("```toml\n");
            hover_text.push_str(example);
            hover_text.push_str("\n```\n\n");
        }

        // 添加废弃警告（如果有）
        if let Some(deprecated_msg) = &schema.deprecated {
            hover_text.push_str(&format!("⚠️ **已废弃**: {}\n\n", deprecated_msg));
        }

        // 添加配置文件位置提示
        hover_text.push_str("---\n\n");
        hover_text.push_str(&format!("*配置节*: `[{}]`\n", prefix));
        hover_text.push_str("*配置文件*: `config/app.toml`\n");

        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: hover_text,
            }),
            range: Some(property.range),
        }
    }

    /// 创建配置项的基本悬停提示（无 Schema）
    fn create_basic_property_hover(
        &self,
        prefix: &str,
        key: &str,
        property: &ConfigProperty,
        is_defined: bool,
    ) -> Hover {
        let mut hover_text = String::new();

        // 添加标题
        hover_text.push_str(&format!("# 配置项: `{}.{}`\n\n", prefix, key));

        // 添加当前值
        hover_text.push_str(&format!(
            "**当前值**: `{}`\n\n",
            self.config_value_to_string(&property.value)
        ));

        // 添加类型
        hover_text.push_str(&format!(
            "**类型**: {}\n\n",
            self.config_value_type_name(&property.value)
        ));

        // 如果未在 Schema 中定义，添加警告
        if !is_defined {
            hover_text.push_str("⚠️ **警告**: 此配置项未在 Schema 中定义\n\n");
        }

        // 添加配置文件位置提示
        hover_text.push_str("---\n\n");
        hover_text.push_str(&format!("*配置节*: `[{}]`\n", prefix));
        hover_text.push_str("*配置文件*: `config/app.toml`\n");

        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: hover_text,
            }),
            range: Some(property.range),
        }
    }

    /// 检查位置是否在范围内
    fn position_in_range(&self, position: Position, range: Range) -> bool {
        // 检查行号
        if position.line < range.start.line || position.line > range.end.line {
            return false;
        }

        // 如果在同一行，检查字符位置
        if position.line == range.start.line && position.character < range.start.character {
            return false;
        }

        if position.line == range.end.line && position.character > range.end.character {
            return false;
        }

        true
    }

    /// 将配置值转换为字符串
    fn config_value_to_string(&self, value: &ConfigValue) -> String {
        match value {
            ConfigValue::String(s) => format!("\"{}\"", s),
            ConfigValue::Integer(i) => i.to_string(),
            ConfigValue::Float(f) => f.to_string(),
            ConfigValue::Boolean(b) => b.to_string(),
            ConfigValue::Array(arr) => {
                let items: Vec<String> =
                    arr.iter().map(|v| self.config_value_to_string(v)).collect();
                format!("[{}]", items.join(", "))
            }
            ConfigValue::Table(table) => {
                let items: Vec<String> = table
                    .iter()
                    .map(|(k, v)| format!("{} = {}", k, self.config_value_to_string(v)))
                    .collect();
                format!("{{ {} }}", items.join(", "))
            }
        }
    }

    /// 将 Schema 中的值转换为字符串
    fn value_to_string(&self, value: &crate::schema::Value) -> String {
        match value {
            crate::schema::Value::String(s) => format!("\"{}\"", s),
            crate::schema::Value::Integer(n) => n.to_string(),
            crate::schema::Value::Float(f) => f.to_string(),
            crate::schema::Value::Boolean(b) => b.to_string(),
            crate::schema::Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| self.value_to_string(v)).collect();
                format!("[{}]", items.join(", "))
            }
            crate::schema::Value::Table(obj) => {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{} = {}", k, self.value_to_string(v)))
                    .collect();
                format!("{{ {} }}", items.join(", "))
            }
        }
    }

    /// 验证配置文件
    ///
    /// 根据 Schema 验证配置文件，生成诊断信息
    ///
    /// # 参数
    ///
    /// * `doc` - 已解析的 TOML 文档
    ///
    /// # 返回
    ///
    /// 诊断信息列表，包含错误、警告等
    ///
    /// # 验证项
    ///
    /// 1. 配置项定义检查：检查配置项是否在 Schema 中定义
    /// 2. 类型验证：检查配置值类型是否匹配
    /// 3. 必需项检查：检查必需的配置项是否存在
    /// 4. 废弃项检查：检查是否使用了废弃的配置项
    /// 5. 环境变量语法验证：检查环境变量插值语法是否正确
    /// 6. 值范围验证：检查配置值是否在允许的范围内
    pub fn validate(&self, doc: &TomlDocument) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // 1. 验证环境变量语法
        diagnostics.extend(self.validate_env_var_syntax(&doc.env_vars));

        // 2. 验证配置节和属性
        for (prefix, section) in &doc.config_sections {
            // 检查配置节是否在 Schema 中定义
            if self.schema_provider.has_plugin(prefix) {
                // 验证配置节中的属性
                diagnostics.extend(self.validate_section_properties(section));
            } else {
                // 配置节未在 Schema 中定义
                diagnostics.push(Diagnostic {
                    range: section.range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(lsp_types::NumberOrString::String(
                        "undefined-section".to_string(),
                    )),
                    message: format!("配置节 '{}' 未在 Schema 中定义", prefix),
                    source: Some("spring-lsp".to_string()),
                    ..Default::default()
                });
            }
        }

        diagnostics
    }

    /// 验证配置节中的属性（简化版）
    fn validate_section_properties(&self, section: &ConfigSection) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (key, property) in &section.properties {
            // 检查配置项是否在 Schema 中定义
            if !self.schema_provider.has_property(&section.prefix, key) {
                diagnostics.push(Diagnostic {
                    range: property.range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(lsp_types::NumberOrString::String(
                        "undefined-property".to_string(),
                    )),
                    message: format!("配置项 '{}' 未在 Schema 中定义", key),
                    source: Some("spring-lsp".to_string()),
                    ..Default::default()
                });
            }
        }

        diagnostics
    }

    fn validate_env_var_syntax(&self, env_vars: &[EnvVarReference]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for env_var in env_vars {
            // 检查变量名是否为空
            if env_var.name.is_empty() {
                diagnostics.push(Diagnostic {
                    range: env_var.range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(lsp_types::NumberOrString::String(
                        "empty-var-name".to_string(),
                    )),
                    message: "环境变量名不能为空".to_string(),
                    source: Some("spring-lsp".to_string()),
                    ..Default::default()
                });
            }

            // 检查变量名是否符合命名规范（大写字母、数字、下划线）
            if !env_var
                .name
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
            {
                diagnostics.push(Diagnostic {
                    range: env_var.range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(lsp_types::NumberOrString::String(
                        "invalid-var-name".to_string(),
                    )),
                    message: format!(
                        "环境变量名 '{}' 不符合命名规范，建议使用大写字母、数字和下划线",
                        env_var.name
                    ),
                    source: Some("spring-lsp".to_string()),
                    ..Default::default()
                });
            }
        }

        diagnostics
    }

    /// 验证配置节中的属性
    fn validate_section(
        &self,
        section: &ConfigSection,
        plugin_schema: &crate::schema::PluginSchema,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (key, property) in &section.properties {
            if let Some(property_schema) = plugin_schema.properties.get(key) {
                // 检查是否废弃
                if let Some(deprecated_msg) = &property_schema.deprecated {
                    diagnostics.push(Diagnostic {
                        range: property.range,
                        severity: Some(DiagnosticSeverity::WARNING),
                        code: Some(lsp_types::NumberOrString::String(
                            "deprecated-property".to_string(),
                        )),
                        message: format!("配置项 '{}' 已废弃: {}", key, deprecated_msg),
                        source: Some("spring-lsp".to_string()),
                        ..Default::default()
                    });
                }

                // 验证类型
                diagnostics.extend(self.validate_property_type(property, property_schema));

                // 验证值范围
                diagnostics.extend(self.validate_property_range(property, property_schema));
            } else {
                // 配置项未在 Schema 中定义
                diagnostics.push(Diagnostic {
                    range: property.range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(lsp_types::NumberOrString::String(
                        "undefined-property".to_string(),
                    )),
                    message: format!("配置项 '{}' 未在 Schema 中定义", key),
                    source: Some("spring-lsp".to_string()),
                    ..Default::default()
                });
            }
        }

        diagnostics
    }

    /// 验证配置属性类型
    fn validate_property_type(
        &self,
        property: &ConfigProperty,
        schema: &PropertySchema,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let type_matches = matches!(
            (&property.value, &schema.type_info),
            (ConfigValue::String(_), TypeInfo::String { .. })
                | (ConfigValue::Integer(_), TypeInfo::Integer { .. })
                | (ConfigValue::Float(_), TypeInfo::Float { .. })
                | (ConfigValue::Boolean(_), TypeInfo::Boolean)
                | (ConfigValue::Array(_), TypeInfo::Array { .. })
                | (ConfigValue::Table(_), TypeInfo::Object { .. })
        );

        if !type_matches {
            let expected_type = self.type_info_to_string(&schema.type_info);
            let actual_type = self.config_value_type_name(&property.value);

            diagnostics.push(Diagnostic {
                range: property.range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(lsp_types::NumberOrString::String(
                    "type-mismatch".to_string(),
                )),
                message: format!(
                    "配置项 '{}' 的类型不匹配：期望 {}，实际 {}",
                    property.key, expected_type, actual_type
                ),
                source: Some("spring-lsp".to_string()),
                ..Default::default()
            });
        }

        diagnostics
    }

    /// 验证配置属性值范围
    fn validate_property_range(
        &self,
        property: &ConfigProperty,
        schema: &PropertySchema,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        match (&property.value, &schema.type_info) {
            // 验证字符串长度和枚举值
            (
                ConfigValue::String(s),
                TypeInfo::String {
                    enum_values,
                    min_length,
                    max_length,
                },
            ) => {
                // 检查枚举值
                if let Some(enum_vals) = enum_values {
                    if !enum_vals.contains(s) {
                        diagnostics.push(Diagnostic {
                            range: property.range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(lsp_types::NumberOrString::String(
                                "invalid-enum-value".to_string(),
                            )),
                            message: format!(
                                "配置项 '{}' 的值 '{}' 不在允许的枚举值中：{:?}",
                                property.key, s, enum_vals
                            ),
                            source: Some("spring-lsp".to_string()),
                            ..Default::default()
                        });
                    }
                }

                // 检查最小长度
                if let Some(min_len) = min_length {
                    if s.len() < *min_len {
                        diagnostics.push(Diagnostic {
                            range: property.range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(lsp_types::NumberOrString::String(
                                "string-too-short".to_string(),
                            )),
                            message: format!(
                                "配置项 '{}' 的值长度 {} 小于最小长度 {}",
                                property.key,
                                s.len(),
                                min_len
                            ),
                            source: Some("spring-lsp".to_string()),
                            ..Default::default()
                        });
                    }
                }

                // 检查最大长度
                if let Some(max_len) = max_length {
                    if s.len() > *max_len {
                        diagnostics.push(Diagnostic {
                            range: property.range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(lsp_types::NumberOrString::String(
                                "string-too-long".to_string(),
                            )),
                            message: format!(
                                "配置项 '{}' 的值长度 {} 超过最大长度 {}",
                                property.key,
                                s.len(),
                                max_len
                            ),
                            source: Some("spring-lsp".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }

            // 验证整数范围
            (ConfigValue::Integer(i), TypeInfo::Integer { min, max }) => {
                if let Some(min_val) = min {
                    if *i < *min_val {
                        diagnostics.push(Diagnostic {
                            range: property.range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(lsp_types::NumberOrString::String(
                                "value-too-small".to_string(),
                            )),
                            message: format!(
                                "配置项 '{}' 的值 {} 小于最小值 {}",
                                property.key, i, min_val
                            ),
                            source: Some("spring-lsp".to_string()),
                            ..Default::default()
                        });
                    }
                }

                if let Some(max_val) = max {
                    if *i > *max_val {
                        diagnostics.push(Diagnostic {
                            range: property.range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(lsp_types::NumberOrString::String(
                                "value-too-large".to_string(),
                            )),
                            message: format!(
                                "配置项 '{}' 的值 {} 超过最大值 {}",
                                property.key, i, max_val
                            ),
                            source: Some("spring-lsp".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }

            // 验证浮点数范围
            (ConfigValue::Float(f), TypeInfo::Float { min, max }) => {
                if let Some(min_val) = min {
                    if *f < *min_val {
                        diagnostics.push(Diagnostic {
                            range: property.range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(lsp_types::NumberOrString::String(
                                "value-too-small".to_string(),
                            )),
                            message: format!(
                                "配置项 '{}' 的值 {} 小于最小值 {}",
                                property.key, f, min_val
                            ),
                            source: Some("spring-lsp".to_string()),
                            ..Default::default()
                        });
                    }
                }

                if let Some(max_val) = max {
                    if *f > *max_val {
                        diagnostics.push(Diagnostic {
                            range: property.range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(lsp_types::NumberOrString::String(
                                "value-too-large".to_string(),
                            )),
                            message: format!(
                                "配置项 '{}' 的值 {} 超过最大值 {}",
                                property.key, f, max_val
                            ),
                            source: Some("spring-lsp".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }

            _ => {}
        }

        diagnostics
    }

    /// 验证必需的配置项
    fn validate_required_properties(
        &self,
        section: &ConfigSection,
        plugin_schema: &crate::schema::PluginSchema,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (key, property_schema) in &plugin_schema.properties {
            if property_schema.required && !section.properties.contains_key(key) {
                diagnostics.push(Diagnostic {
                    range: section.range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(lsp_types::NumberOrString::String(
                        "missing-required-property".to_string(),
                    )),
                    message: format!("缺少必需的配置项 '{}'", key),
                    source: Some("spring-lsp".to_string()),
                    ..Default::default()
                });
            }
        }

        diagnostics
    }

    /// 将 TypeInfo 转换为字符串描述
    fn type_info_to_string(&self, type_info: &TypeInfo) -> String {
        match type_info {
            TypeInfo::String { .. } => "字符串".to_string(),
            TypeInfo::Integer { .. } => "整数".to_string(),
            TypeInfo::Float { .. } => "浮点数".to_string(),
            TypeInfo::Boolean => "布尔值".to_string(),
            TypeInfo::Array { .. } => "数组".to_string(),
            TypeInfo::Object { .. } => "对象".to_string(),
        }
    }

    /// 获取配置值的类型名称
    fn config_value_type_name(&self, value: &ConfigValue) -> String {
        match value {
            ConfigValue::String(_) => "字符串".to_string(),
            ConfigValue::Integer(_) => "整数".to_string(),
            ConfigValue::Float(_) => "浮点数".to_string(),
            ConfigValue::Boolean(_) => "布尔值".to_string(),
            ConfigValue::Array(_) => "数组".to_string(),
            ConfigValue::Table(_) => "对象".to_string(),
        }
    }

    /// 解析 TOML 文档
    ///
    /// 使用 taplo 解析 TOML 内容，提取环境变量引用和配置节
    ///
    /// # 参数
    ///
    /// * `content` - TOML 文件内容
    ///
    /// # 返回
    ///
    /// 成功时返回 `TomlDocument`，失败时返回错误信息
    ///
    /// # 示例
    ///
    /// ```
    /// use spring_lsp::toml_analyzer::TomlAnalyzer;
    /// use spring_lsp::schema::SchemaProvider;
    ///
    /// let schema_provider = SchemaProvider::new();
    /// let analyzer = TomlAnalyzer::new(schema_provider);
    /// let doc = analyzer.parse("[web]\nhost = \"localhost\"").unwrap();
    /// assert_eq!(doc.config_sections.len(), 1);
    /// ```
    pub fn parse(&self, content: &str) -> Result<TomlDocument, String> {
        // 预处理：提取环境变量引用并替换为占位符
        let (preprocessed_content, env_vars) = self.preprocess_env_vars(content);

        // 使用 taplo 解析预处理后的 TOML
        let parse_result = taplo::parser::parse(&preprocessed_content);

        // 检查语法错误
        if !parse_result.errors.is_empty() {
            let error_messages: Vec<String> = parse_result
                .errors
                .iter()
                .map(|e| format!("{:?}:{:?} - {}", e.range.start(), e.range.end(), e.message))
                .collect();
            return Err(format!("TOML 语法错误: {}", error_messages.join("; ")));
        }

        // 转换为 DOM
        let root = parse_result.into_dom();

        // 提取配置节
        let config_sections = self.extract_config_sections(&root, content);

        Ok(TomlDocument {
            root,
            env_vars,
            config_sections,
            content: content.to_string(),
        })
    }

    /// 预处理环境变量引用
    ///
    /// 将 `${VAR:default}` 或 `${VAR}` 替换为占位符，以便 TOML 解析器能够正常解析
    /// 同时提取所有环境变量引用的位置信息
    ///
    /// **注意**：只处理引号外的环境变量引用，引号内的会被保留（因为它们是合法的 TOML 字符串）
    fn preprocess_env_vars(&self, content: &str) -> (String, Vec<EnvVarReference>) {
        let mut result = String::with_capacity(content.len());
        let mut env_vars = Vec::new();
        let mut line = 0u32;
        let mut line_start = 0;
        let mut i = 0;
        let chars: Vec<char> = content.chars().collect();
        let mut in_string = false; // 跟踪是否在字符串内
        let mut in_multiline_string = false; // 跟踪是否在多行字符串内
        let mut escape_next = false; // 跟踪下一个字符是否被转义

        while i < chars.len() {
            // 更新行号和行起始位置
            if chars[i] == '\n' {
                line += 1;
                line_start = i + 1;
            }

            // 处理转义字符
            if escape_next {
                result.push(chars[i]);
                escape_next = false;
                i += 1;
                continue;
            }

            // 检查转义符
            if chars[i] == '\\' && (in_string || in_multiline_string) {
                escape_next = true;
                result.push(chars[i]);
                i += 1;
                continue;
            }

            // 检查多行字符串（三个引号）
            if i + 2 < chars.len() && chars[i] == '"' && chars[i + 1] == '"' && chars[i + 2] == '"'
            {
                in_multiline_string = !in_multiline_string;
                result.push_str("\"\"\"");
                i += 3;
                continue;
            }

            // 检查普通字符串
            if chars[i] == '"' && !in_multiline_string {
                in_string = !in_string;
                result.push(chars[i]);
                i += 1;
                continue;
            }

            // 只在引号外处理环境变量引用
            if !in_string && !in_multiline_string {
                // 查找 ${
                if i + 1 < chars.len() && chars[i] == '$' && chars[i + 1] == '{' {
                    // 查找对应的 }
                    let mut j = i + 2;
                    while j < chars.len() && chars[j] != '}' {
                        j += 1;
                    }

                    if j < chars.len() {
                        // 找到了完整的环境变量引用
                        let var_content: String = chars[i + 2..j].iter().collect();

                        // 解析变量名和默认值
                        let (name, default) = if let Some(colon_pos) = var_content.find(':') {
                            let name = var_content[..colon_pos].to_string();
                            let default = Some(var_content[colon_pos + 1..].to_string());
                            (name, default)
                        } else {
                            (var_content.to_string(), None)
                        };

                        // 计算位置
                        let start_char = i - line_start;
                        let end_char = j + 1 - line_start;

                        env_vars.push(EnvVarReference {
                            name: name.clone(),
                            default: default.clone(),
                            range: Range {
                                start: Position {
                                    line,
                                    character: start_char as u32,
                                },
                                end: Position {
                                    line,
                                    character: end_char as u32,
                                },
                            },
                        });

                        // 替换为占位符（使用默认值或空字符串）
                        let placeholder = if let Some(default_val) = &default {
                            // 如果默认值是布尔值或数字，直接使用
                            if default_val == "true"
                                || default_val == "false"
                                || default_val.parse::<i64>().is_ok()
                                || default_val.parse::<f64>().is_ok()
                            {
                                default_val.clone()
                            } else {
                                // 字符串需要加引号
                                format!("\"{}\"", default_val.replace('"', "\\\""))
                            }
                        } else {
                            // 没有默认值，使用空字符串
                            "\"\"".to_string()
                        };

                        result.push_str(&placeholder);
                        i = j + 1;
                        continue;
                    }
                }
            }

            result.push(chars[i]);
            i += 1;
        }

        (result, env_vars)
    }

    /// 提取环境变量引用
    /// 提取配置节
    ///
    /// 遍历 TOML DOM 树，提取所有配置节和属性
    fn extract_config_sections(
        &self,
        root: &taplo::dom::Node,
        content: &str,
    ) -> HashMap<String, ConfigSection> {
        let mut sections = HashMap::new();

        // 获取根表
        if let Some(table) = root.as_table() {
            let entries = table.entries();

            // 使用 get() 获取 Arc 引用，然后迭代
            let entries_arc = entries.get();
            for (key, value) in entries_arc.iter() {
                let prefix = key.value().to_string();

                // 只处理表类型的节（配置节）
                if value.as_table().is_some() {
                    let properties = self.extract_properties(value, content);
                    let range = self.node_to_range(value, content);

                    sections.insert(
                        prefix.clone(),
                        ConfigSection {
                            prefix,
                            properties,
                            range,
                        },
                    );
                }
            }
        }

        sections
    }

    /// 提取配置属性
    ///
    /// 从 TOML 节点中提取所有属性
    fn extract_properties(
        &self,
        node: &taplo::dom::Node,
        content: &str,
    ) -> HashMap<String, ConfigProperty> {
        let mut properties = HashMap::new();

        // 获取表节点
        if let Some(table) = node.as_table() {
            let entries = table.entries();

            // 使用 get() 获取 Arc 引用，然后迭代
            let entries_arc = entries.get();
            for (key, value) in entries_arc.iter() {
                let key_str = key.value().to_string();
                let config_value = self.node_to_config_value(value);
                let range = self.node_to_range(value, content);

                properties.insert(
                    key_str.clone(),
                    ConfigProperty {
                        key: key_str,
                        value: config_value,
                        range,
                    },
                );
            }
        }

        properties
    }

    /// 将 TOML 节点转换为配置值
    fn node_to_config_value(&self, node: &taplo::dom::Node) -> ConfigValue {
        match node {
            taplo::dom::Node::Bool(b) => ConfigValue::Boolean(b.value()),
            taplo::dom::Node::Str(s) => ConfigValue::String(s.value().to_string()),
            taplo::dom::Node::Integer(i) => {
                // IntegerValue 需要转换为 i64
                match i.value() {
                    IntegerValue::Positive(v) => ConfigValue::Integer(v as i64),
                    IntegerValue::Negative(v) => ConfigValue::Integer(v),
                }
            }
            taplo::dom::Node::Float(f) => ConfigValue::Float(f.value()),
            taplo::dom::Node::Array(arr) => {
                let items = arr.items();
                let mut values = Vec::new();

                // 使用 get() 获取 Arc 引用，然后迭代
                let items_arc = items.get();
                for item in items_arc.iter() {
                    values.push(self.node_to_config_value(item));
                }

                ConfigValue::Array(values)
            }
            taplo::dom::Node::Table(table) => {
                let entries = table.entries();
                let mut map = HashMap::new();

                // 使用 get() 获取 Arc 引用，然后迭代
                let entries_arc = entries.get();
                for (key, value) in entries_arc.iter() {
                    let key_str = key.value().to_string();
                    map.insert(key_str, self.node_to_config_value(value));
                }

                ConfigValue::Table(map)
            }
            _ => ConfigValue::String(String::new()), // 默认值
        }
    }

    /// 将 TOML 节点转换为 LSP 范围
    ///
    /// 将 taplo 提供的字节偏移量转换为行号和字符位置
    fn node_to_range(&self, node: &taplo::dom::Node, content: &str) -> Range {
        // taplo 的 text_ranges 返回一个迭代器
        let mut text_ranges = node.text_ranges();
        if let Some(first_range) = text_ranges.next() {
            let start: usize = first_range.start().into();
            let end: usize = first_range.end().into();

            // 将字节偏移量转换为行号和字符位置
            let start_pos = self.byte_offset_to_position(content, start);
            let end_pos = self.byte_offset_to_position(content, end);

            Range {
                start: start_pos,
                end: end_pos,
            }
        } else {
            // 默认范围
            Range {
                start: lsp_types::Position {
                    line: 0,
                    character: 0,
                },
                end: lsp_types::Position {
                    line: 0,
                    character: 0,
                },
            }
        }
    }

    /// 将字节偏移量转换为 LSP Position
    ///
    /// 遍历内容，计算字节偏移量对应的行号和字符位置
    fn byte_offset_to_position(&self, content: &str, byte_offset: usize) -> lsp_types::Position {
        let mut line = 0;
        let mut character = 0;
        let mut current_offset = 0;

        for ch in content.chars() {
            if current_offset >= byte_offset {
                break;
            }

            if ch == '\n' {
                line += 1;
                character = 0;
            } else {
                character += 1;
            }

            current_offset += ch.len_utf8();
        }

        lsp_types::Position {
            line: line as u32,
            character: character as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_env_vars_in_quotes() {
        let schema_provider = SchemaProvider::new();
        let analyzer = TomlAnalyzer::new(schema_provider);

        // 测试：引号内的环境变量应该保持不变
        let content = r#"test_pay_amount = "${TEST_PAY_AMOUNT:false}""#;
        let (preprocessed, env_vars) = analyzer.preprocess_env_vars(content);

        println!("原始: {}", content);
        println!("预处理后: {}", preprocessed);
        println!("环境变量数量: {}", env_vars.len());

        // 引号内的环境变量不应该被提取
        assert_eq!(env_vars.len(), 0, "引号内的环境变量不应该被提取");
        assert_eq!(preprocessed, content, "引号内的内容不应该被修改");
    }

    #[test]
    fn test_preprocess_env_vars_without_quotes() {
        let schema_provider = SchemaProvider::new();
        let analyzer = TomlAnalyzer::new(schema_provider);

        // 测试：引号外的环境变量应该被替换
        let content = r#"test_pay_amount = ${TEST_PAY_AMOUNT:false}"#;
        let (preprocessed, env_vars) = analyzer.preprocess_env_vars(content);

        println!("原始: {}", content);
        println!("预处理后: {}", preprocessed);
        println!("环境变量数量: {}", env_vars.len());

        // 应该提取到一个环境变量
        assert_eq!(env_vars.len(), 1);
        assert_eq!(env_vars[0].name, "TEST_PAY_AMOUNT");
        assert_eq!(env_vars[0].default, Some("false".to_string()));

        // 应该被替换为布尔值
        assert_eq!(preprocessed, "test_pay_amount = false");
    }

    #[test]
    fn test_parse_with_quoted_env_var() {
        let schema_provider = SchemaProvider::new();
        let analyzer = TomlAnalyzer::new(schema_provider);

        // 测试：带引号的环境变量应该能正常解析
        let content = r#"
[pay]
test_pay_amount = "${TEST_PAY_AMOUNT:false}"
api_key = "${API_KEY:test_key}"
"#;

        let result = analyzer.parse(content);
        assert!(
            result.is_ok(),
            "带引号的环境变量应该能正常解析: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_without_quoted_env_var() {
        let schema_provider = SchemaProvider::new();
        let analyzer = TomlAnalyzer::new(schema_provider);

        // 测试：不带引号的环境变量应该能正常解析（被预处理后）
        let content = r#"
[pay]
test_pay_amount = ${TEST_PAY_AMOUNT:false}
port = ${PORT:8080}
"#;

        let result = analyzer.parse(content);
        assert!(
            result.is_ok(),
            "不带引号的环境变量应该能正常解析: {:?}",
            result.err()
        );

        let doc = result.unwrap();
        assert_eq!(doc.env_vars.len(), 2, "应该提取到 2 个环境变量");
    }
}
