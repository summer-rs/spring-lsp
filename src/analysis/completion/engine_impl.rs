//! 智能补全引擎模块

use lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, InsertTextFormat, MarkupContent, MarkupKind,
    Position, Range,
};

use crate::analysis::rust::macro_analyzer::SummerMacro;
use crate::analysis::toml::toml_analyzer::{TomlAnalyzer, TomlDocument};
use crate::core::schema::SchemaProvider;

/// 补全上下文
///
/// 提供补全请求的上下文信息，用于确定补全类型
#[derive(Debug, Clone)]
pub enum CompletionContext {
    /// TOML 配置文件补全
    Toml,
    /// Rust 宏补全
    Macro,
    /// 未知上下文
    Unknown,
}

/// 补全引擎
///
/// 提供智能补全功能，支持 TOML 配置文件和 Rust 宏的补全
pub struct CompletionEngine {
    /// TOML 分析器
    toml_analyzer: TomlAnalyzer,
}

impl CompletionEngine {
    /// 创建新的补全引擎
    ///
    /// # 参数
    ///
    /// * `schema_provider` - Schema 提供者，用于 TOML 配置补全
    pub fn new(schema_provider: SchemaProvider) -> Self {
        Self {
            toml_analyzer: TomlAnalyzer::new(schema_provider),
        }
    }

    /// 提供补全
    ///
    /// 根据文档类型和位置提供相应的补全项
    ///
    /// # 参数
    ///
    /// * `context` - 补全上下文，指示补全类型
    /// * `position` - 光标位置
    /// * `toml_doc` - TOML 文档（可选，用于 TOML 补全）
    /// * `macro_info` - 宏信息（可选，用于宏补全）
    ///
    /// # 返回
    ///
    /// 补全项列表
    pub fn complete(
        &self,
        context: CompletionContext,
        position: Position,
        toml_doc: Option<&TomlDocument>,
        macro_info: Option<&SummerMacro>,
    ) -> Vec<CompletionItem> {
        match context {
            CompletionContext::Toml => {
                if let Some(doc) = toml_doc {
                    self.complete_toml(doc, position)
                } else {
                    Vec::new()
                }
            }
            CompletionContext::Macro => {
                if let Some(macro_info) = macro_info {
                    self.complete_macro(macro_info, None)
                } else {
                    Vec::new()
                }
            }
            CompletionContext::Unknown => Vec::new(),
        }
    }

    /// TOML 配置补全（公共方法）
    ///
    /// 为 TOML 配置文件提供补全，支持：
    /// - 配置前缀补全（在 `[` 后）
    /// - 配置项补全（在配置节内）
    /// - 枚举值补全
    /// - 环境变量补全（在 `${` 后）
    ///
    /// # 参数
    ///
    /// * `doc` - TOML 文档
    /// * `position` - 光标位置
    ///
    /// # 返回
    ///
    /// 补全项列表
    pub fn complete_toml_document(
        &self,
        doc: &TomlDocument,
        position: Position,
    ) -> Vec<CompletionItem> {
        self.complete_toml(doc, position)
    }

    /// TOML 配置补全（内部方法）
    ///
    /// 为 TOML 配置文件提供补全，支持：
    /// - 配置前缀补全（在 `[` 后）
    /// - 配置项补全（在配置节内）
    /// - 枚举值补全
    /// - 环境变量补全（在 `${` 后）
    ///
    /// # 参数
    ///
    /// * `doc` - TOML 文档
    /// * `position` - 光标位置
    ///
    /// # 返回
    ///
    /// 补全项列表
    fn complete_toml(&self, doc: &TomlDocument, position: Position) -> Vec<CompletionItem> {
        // 1. 检查是否在配置前缀位置（[之后）
        if self.is_prefix_position(doc, position) {
            return self.complete_config_prefix();
        }

        // 2. 检查是否在环境变量位置（${之后）
        if self.is_env_var_position(doc, position) {
            return self.complete_env_var();
        }

        // 3. 检查是否在配置节内
        if let Some(section) = self.find_section_at_position(doc, position) {
            // 3.1 检查是否在属性值位置，需要补全枚举值
            if let Some(property_name) = self.find_property_at_position(section, position) {
                // 获取属性的 Schema 信息
                let schema_provider = self.toml_analyzer.schema_provider();
                if let Some(plugin_schema) = schema_provider.get_plugin(&section.prefix) {
                    if let Some(property_schema) = plugin_schema.properties.get(&property_name) {
                        // 检查是否有枚举值
                        if let crate::schema::TypeInfo::String {
                            enum_values: Some(enum_vals),
                            ..
                        } = &property_schema.type_info
                        {
                            return self.complete_enum_values(enum_vals);
                        }
                    }
                }
            }

            // 3.2 否则提供配置项补全
            return self.complete_config_properties(section);
        }

        Vec::new()
    }

    /// 检查是否在配置前缀位置
    ///
    /// 判断光标是否在 `[` 字符之后，需要补全配置前缀
    fn is_prefix_position(&self, doc: &TomlDocument, position: Position) -> bool {
        // 获取光标所在行的内容
        let lines: Vec<&str> = doc.content.lines().collect();

        if position.line as usize >= lines.len() {
            return false;
        }

        let line = lines[position.line as usize];
        let char_pos = position.character as usize;

        // 光标位置必须在行内或行尾
        if char_pos > line.len() {
            return false;
        }

        // 检查光标前的字符
        let before_cursor = if char_pos > 0 { &line[..char_pos] } else { "" };

        // 如果光标前是 `[` 或 `[` 后跟一些字符，则认为是前缀位置
        // 但必须确保还没有闭合括号
        let trimmed = before_cursor.trim_start();

        // 只有在输入 `[` 后且还没有完成节名输入时才提供前缀补全
        // 如果已经有完整的节名（包含 `]`），则不是前缀位置
        trimmed.starts_with('[') && !trimmed.contains(']') && !line.contains(']')
    }

    /// 检查是否在环境变量位置
    ///
    /// 判断光标是否在 `${` 之后，需要补全环境变量名
    fn is_env_var_position(&self, _doc: &TomlDocument, _position: Position) -> bool {
        // 简化实现：这里需要检查光标前的字符是否是 `${`
        // 在实际实现中，应该解析文档内容来判断
        false
    }

    /// 查找光标所在的配置节
    ///
    /// 根据光标位置查找对应的配置节
    fn find_section_at_position<'a>(
        &self,
        doc: &'a TomlDocument,
        position: Position,
    ) -> Option<&'a crate::toml_analyzer::ConfigSection> {
        doc.config_sections
            .values()
            .find(|&section| self.position_in_range(position, section.range))
    }

    /// 查找光标所在的属性名
    ///
    /// 在配置节中查找光标位置对应的属性名（用于枚举值补全）
    fn find_property_at_position(
        &self,
        section: &crate::toml_analyzer::ConfigSection,
        position: Position,
    ) -> Option<String> {
        for (key, property) in &section.properties {
            // 检查位置是否在属性值范围内
            if self.position_in_range(position, property.range) {
                return Some(key.clone());
            }
        }
        None
    }

    /// 检查位置是否在范围内
    fn position_in_range(&self, position: Position, range: Range) -> bool {
        if position.line < range.start.line || position.line > range.end.line {
            return false;
        }
        if position.line == range.start.line && position.character < range.start.character {
            return false;
        }
        if position.line == range.end.line && position.character > range.end.character {
            return false;
        }
        true
    }

    /// 补全配置前缀
    ///
    /// 提供所有可用的配置前缀（插件名称）
    fn complete_config_prefix(&self) -> Vec<CompletionItem> {
        let prefixes = self.toml_analyzer.schema_provider().get_all_prefixes();

        prefixes
            .into_iter()
            .map(|prefix: String| {
                let description = format!("{} 插件配置", prefix);

                CompletionItem {
                    label: prefix.clone(),
                    kind: Some(CompletionItemKind::MODULE),
                    detail: Some(format!("[{}] 配置节", prefix)),
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!(
                            "**{}** 插件配置节\n\n{}\n\n\
                             **使用方式**:\n\
                             ```toml\n\
                             [{}]\n\
                             # 配置项...\n\
                             ```",
                            prefix, description, prefix
                        ),
                    })),
                    insert_text: Some(format!("[{}]\n", prefix)),
                    insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
                    ..Default::default()
                }
            })
            .collect()
    }

    /// 补全配置项
    ///
    /// 在配置节内提供配置项补全，自动去重已存在的配置项
    /// 补全配置项
    ///
    /// 在配置节内提供配置项补全，自动去重已存在的配置项
    fn complete_config_properties(
        &self,
        section: &crate::toml_analyzer::ConfigSection,
    ) -> Vec<CompletionItem> {
        let prefix = &section.prefix;

        // 从 Schema Provider 中获取插件信息
        let schema_provider = self.toml_analyzer.schema_provider();
        let plugin_schema = match schema_provider.get_plugin(prefix) {
            Some(schema) => schema,
            None => return Vec::new(),
        };

        // 获取已存在的属性名（用于去重）
        let existing_keys: std::collections::HashSet<String> =
            section.properties.keys().cloned().collect();

        // 为每个未使用的属性创建补全项
        let mut completions = Vec::new();

        for (key, property_schema) in &plugin_schema.properties {
            // 跳过已存在的属性
            if existing_keys.contains(key) {
                continue;
            }

            // 使用新的辅助方法生成类型提示和默认值
            let type_hint = self.type_info_to_hint(&property_schema.type_info);
            let default_value = if let Some(default) = &property_schema.default {
                self.value_to_string(default)
            } else {
                self.type_info_to_default(&property_schema.type_info)
            };

            // 构建插入文本：key = value  # type
            let insert_text = format!("{} = {}  # {}", key, default_value, type_hint);

            // 构建文档
            let mut doc_parts = Vec::new();

            if !property_schema.description.is_empty() {
                doc_parts.push(property_schema.description.clone());
            }

            doc_parts.push(format!("**类型**: `{}`", type_hint));

            if let Some(default) = &property_schema.default {
                doc_parts.push(format!("**默认值**: `{}`", self.value_to_string(default)));
            }

            if property_schema.required {
                doc_parts.push("**必需**: 是".to_string());
            }

            if let Some(deprecated_msg) = &property_schema.deprecated {
                doc_parts.push(format!("**已废弃**: {}", deprecated_msg));
            }

            let documentation = Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc_parts.join("\n\n"),
            });

            completions.push(CompletionItem {
                label: key.clone(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some(format!("{}: {}", key, type_hint)),
                documentation: Some(documentation),
                insert_text: Some(insert_text),
                insert_text_format: Some(lsp_types::InsertTextFormat::PLAIN_TEXT),
                deprecated: property_schema.deprecated.is_some().then_some(true),
                ..Default::default()
            });
        }

        completions
    }

    /// 将 JSON 值转换为 TOML 字符串
    #[allow(dead_code)]
    fn json_value_to_toml_string(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => format!("\"{}\"", s),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Array(_) => "[]".to_string(),
            serde_json::Value::Object(_) => "{}".to_string(),
            serde_json::Value::Null => "null".to_string(),
        }
    }

    /// 根据类型名称返回默认值
    #[allow(dead_code)]
    fn type_to_default_value(&self, type_name: &str) -> String {
        match type_name {
            "string" => "\"\"".to_string(),
            "integer" | "number" => "0".to_string(),
            "boolean" => "false".to_string(),
            "array" => "[]".to_string(),
            "object" => "{}".to_string(),
            _ => "\"\"".to_string(),
        }
    }

    /// 补全枚举值
    ///
    /// 为具有枚举类型的配置项提供值补全
    fn complete_enum_values(&self, values: &[String]) -> Vec<CompletionItem> {
        values
            .iter()
            .map(|value| CompletionItem {
                label: value.clone(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some(format!("枚举值: {}", value)),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("枚举值 `{}`", value),
                })),
                insert_text: Some(format!("\"{}\"", value)),
                insert_text_format: Some(lsp_types::InsertTextFormat::PLAIN_TEXT),
                ..Default::default()
            })
            .collect()
    }

    /// 补全环境变量
    ///
    /// 提供常见的环境变量名称补全
    pub fn complete_env_var(&self) -> Vec<CompletionItem> {
        let common_vars = vec![
            ("HOST", "主机地址"),
            ("PORT", "端口号"),
            ("DATABASE_URL", "数据库连接 URL"),
            ("REDIS_URL", "Redis 连接 URL"),
            ("LOG_LEVEL", "日志级别"),
            ("ENV", "运行环境"),
            ("DEBUG", "调试模式"),
        ];

        common_vars
            .into_iter()
            .map(|(name, description)| CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some(description.to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "**{}**\n\n{}\n\n\
                         **使用方式**:\n\
                         ```toml\n\
                         value = \"${{{}:default}}\"\n\
                         ```",
                        name, description, name
                    ),
                })),
                insert_text: Some(format!("{}:${{1:default}}}}", name)),
                insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
                ..Default::default()
            })
            .collect()
    }

    /// 将类型信息转换为类型提示字符串
    fn type_info_to_hint(&self, type_info: &crate::schema::TypeInfo) -> String {
        match type_info {
            crate::schema::TypeInfo::String {
                enum_values: Some(values),
                ..
            } => {
                format!("enum: {:?}", values)
            }
            crate::schema::TypeInfo::String { .. } => "string".to_string(),
            crate::schema::TypeInfo::Integer { min, max } => {
                if let (Some(min), Some(max)) = (min, max) {
                    format!("integer ({} - {})", min, max)
                } else {
                    "integer".to_string()
                }
            }
            crate::schema::TypeInfo::Float { .. } => "float".to_string(),
            crate::schema::TypeInfo::Boolean => "boolean".to_string(),
            crate::schema::TypeInfo::Array { .. } => "array".to_string(),
            crate::schema::TypeInfo::Object { .. } => "object".to_string(),
        }
    }

    /// 将类型信息转换为默认值字符串
    fn type_info_to_default(&self, type_info: &crate::schema::TypeInfo) -> String {
        match type_info {
            crate::schema::TypeInfo::String {
                enum_values: Some(values),
                ..
            } => {
                if let Some(first) = values.first() {
                    format!("\"{}\"", first)
                } else {
                    "\"\"".to_string()
                }
            }
            crate::schema::TypeInfo::String { .. } => "\"\"".to_string(),
            crate::schema::TypeInfo::Integer { .. } => "0".to_string(),
            crate::schema::TypeInfo::Float { .. } => "0.0".to_string(),
            crate::schema::TypeInfo::Boolean => "false".to_string(),
            crate::schema::TypeInfo::Array { .. } => "[]".to_string(),
            crate::schema::TypeInfo::Object { .. } => "{}".to_string(),
        }
    }

    /// 将 Schema 值转换为字符串
    fn value_to_string(&self, value: &crate::schema::Value) -> String {
        match value {
            crate::schema::Value::String(s) => format!("\"{}\"", s),
            crate::schema::Value::Integer(i) => i.to_string(),
            crate::schema::Value::Float(f) => f.to_string(),
            crate::schema::Value::Boolean(b) => b.to_string(),
            crate::schema::Value::Array(_) => "[]".to_string(),
            crate::schema::Value::Table(_) => "{}".to_string(),
        }
    }

    /// 为宏参数提供补全
    ///
    /// 根据宏的类型提供相应的参数补全项
    ///
    /// # Arguments
    ///
    /// * `macro_info` - 宏信息
    /// * `cursor_position` - 光标位置（用于上下文感知补全）
    ///
    /// # Returns
    ///
    /// 返回补全项列表
    pub fn complete_macro(
        &self,
        macro_info: &SummerMacro,
        _cursor_position: Option<&str>,
    ) -> Vec<CompletionItem> {
        match macro_info {
            SummerMacro::DeriveService(_) => self.complete_service_macro(),
            SummerMacro::Component(_) => self.complete_component_macro(),
            SummerMacro::Inject(_) => self.complete_inject_macro(),
            SummerMacro::AutoConfig(_) => self.complete_auto_config_macro(),
            SummerMacro::Route(_) => self.complete_route_macro(),
            SummerMacro::Job(_) => self.complete_job_macro(),
        }
    }

    /// 为 Component 宏提供补全
    ///
    /// 提供 name 参数的补全
    fn complete_component_macro(&self) -> Vec<CompletionItem> {
        vec![CompletionItem {
            label: "name".to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some("插件名称".to_string()),
            documentation: Some(Documentation::String(
                "指定自定义的插件名称，默认使用组件类型名 + \"Plugin\"".to_string(),
            )),
            insert_text: Some("name = \"$1\"".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        }]
    }

    /// 为 Service 宏提供补全
    ///
    /// 提供 inject 属性的参数补全
    fn complete_service_macro(&self) -> Vec<CompletionItem> {
        vec![
            // inject(component) 补全
            CompletionItem {
                label: "inject(component)".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("注入组件".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "从应用上下文中注入已注册的组件实例。\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[inject(component)]\n\
                            db: ConnectPool,\n\
                            ```"
                        .to_string(),
                })),
                insert_text: Some("inject(component)".to_string()),
                ..Default::default()
            },
            // inject(component = "name") 补全
            CompletionItem {
                label: "inject(component = \"name\")".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("注入指定名称的组件".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "使用指定名称从应用上下文中注入组件，适用于多实例场景（如多数据源）。\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[inject(component = \"primary\")]\n\
                            primary_db: ConnectPool,\n\
                            ```"
                        .to_string(),
                })),
                insert_text: Some("inject(component = \"$1\")".to_string()),
                insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            // inject(config) 补全
            CompletionItem {
                label: "inject(config)".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("注入配置".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "从配置文件中加载配置项。配置项通过 `#[config_prefix]` 指定的前缀从 `config/app.toml` 中读取。\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[inject(config)]\n\
                            config: MyConfig,\n\
                            ```"
                        .to_string(),
                })),
                insert_text: Some("inject(config)".to_string()),
                ..Default::default()
            },
        ]
    }

    /// 为 Inject 宏提供补全
    ///
    /// 提供注入类型的补全（component, config）
    fn complete_inject_macro(&self) -> Vec<CompletionItem> {
        vec![
            // component 补全
            CompletionItem {
                label: "component".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("注入组件".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "从应用上下文中注入已注册的组件实例。\n\n\
                            **使用方式**:\n\
                            - `#[inject(component)]` - 按类型自动查找\n\
                            - `#[inject(component = \"name\")]` - 按名称查找"
                        .to_string(),
                })),
                insert_text: Some("component".to_string()),
                ..Default::default()
            },
            // config 补全
            CompletionItem {
                label: "config".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("注入配置".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "从配置文件中加载配置项。\n\n\
                            **使用方式**:\n\
                            - `#[inject(config)]` - 从 config/app.toml 加载配置"
                        .to_string(),
                })),
                insert_text: Some("config".to_string()),
                ..Default::default()
            },
        ]
    }

    /// 为 AutoConfig 宏提供补全
    ///
    /// 提供常见的配置器类型补全
    fn complete_auto_config_macro(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "WebConfigurator".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Web 路由配置器".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "自动注册 Web 路由处理器。\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[auto_config(WebConfigurator)]\n\
                            #[tokio::main]\n\
                            async fn main() {\n\
                                App::new().add_plugin(WebPlugin).run().await\n\
                            }\n\
                            ```"
                    .to_string(),
                })),
                insert_text: Some("WebConfigurator".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "JobConfigurator".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("任务调度配置器".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "自动注册定时任务。\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[auto_config(JobConfigurator)]\n\
                            #[tokio::main]\n\
                            async fn main() {\n\
                                App::new().add_plugin(JobPlugin).run().await\n\
                            }\n\
                            ```"
                    .to_string(),
                })),
                insert_text: Some("JobConfigurator".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "StreamConfigurator".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("流处理配置器".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "自动注册流消费者。\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[auto_config(StreamConfigurator)]\n\
                            #[tokio::main]\n\
                            async fn main() {\n\
                                App::new().add_plugin(StreamPlugin).run().await\n\
                            }\n\
                            ```"
                    .to_string(),
                })),
                insert_text: Some("StreamConfigurator".to_string()),
                ..Default::default()
            },
        ]
    }

    /// 为路由宏提供补全
    ///
    /// 提供 HTTP 方法和路径参数的补全
    fn complete_route_macro(&self) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // HTTP 方法补全
        let methods = vec![
            ("GET", "获取资源"),
            ("POST", "创建资源"),
            ("PUT", "更新资源（完整）"),
            ("DELETE", "删除资源"),
            ("PATCH", "更新资源（部分）"),
            ("HEAD", "获取资源头信息"),
            ("OPTIONS", "获取支持的方法"),
        ];

        for (method, description) in methods {
            completions.push(CompletionItem {
                label: method.to_string(),
                kind: Some(CompletionItemKind::CONSTANT),
                detail: Some(description.to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "HTTP {} 方法\n\n\
                         **示例**:\n\
                         ```rust\n\
                         #[{}(\"/path\")]\n\
                         async fn handler() -> impl IntoResponse {{\n\
                         }}\n\
                         ```",
                        method,
                        method.to_lowercase()
                    ),
                })),
                insert_text: Some(method.to_string()),
                ..Default::default()
            });
        }

        // 路径参数模板补全
        completions.push(CompletionItem {
            label: "{id}".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("路径参数".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "路径参数占位符，用于捕获 URL 中的动态部分。\n\n\
                        **示例**:\n\
                        ```rust\n\
                        #[get(\"/users/{id}\")]\n\
                        async fn get_user(Path(id): Path<i64>) -> impl IntoResponse {\n\
                        }\n\
                        ```"
                .to_string(),
            })),
            insert_text: Some("{${1:id}}".to_string()),
            insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
            ..Default::default()
        });

        completions
    }

    /// 为任务调度宏提供补全
    ///
    /// 提供 cron 表达式、延迟和频率值的补全
    fn complete_job_macro(&self) -> Vec<CompletionItem> {
        vec![
            // Cron 表达式示例
            CompletionItem {
                label: "0 0 * * * *".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("每小时执行".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Cron 表达式：每小时的第 0 分 0 秒执行\n\n\
                            **格式**: 秒 分 时 日 月 星期\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[cron(\"0 0 * * * *\")]\n\
                            async fn hourly_job() {\n\
                            }\n\
                            ```"
                    .to_string(),
                })),
                insert_text: Some("\"0 0 * * * *\"".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "0 0 0 * * *".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("每天午夜执行".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Cron 表达式：每天午夜 00:00:00 执行\n\n\
                            **格式**: 秒 分 时 日 月 星期\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[cron(\"0 0 0 * * *\")]\n\
                            async fn daily_job() {\n\
                            }\n\
                            ```"
                    .to_string(),
                })),
                insert_text: Some("\"0 0 0 * * *\"".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "0 */5 * * * *".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("每 5 分钟执行".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Cron 表达式：每 5 分钟执行一次\n\n\
                            **格式**: 秒 分 时 日 月 星期\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[cron(\"0 */5 * * * *\")]\n\
                            async fn every_five_minutes() {\n\
                            }\n\
                            ```"
                    .to_string(),
                })),
                insert_text: Some("\"0 */5 * * * *\"".to_string()),
                ..Default::default()
            },
            // fix_delay 值示例
            CompletionItem {
                label: "5".to_string(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some("延迟 5 秒".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "任务完成后延迟 5 秒再次执行\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[fix_delay(5)]\n\
                            async fn delayed_job() {\n\
                            }\n\
                            ```"
                    .to_string(),
                })),
                insert_text: Some("5".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "10".to_string(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some("延迟/频率 10 秒".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "延迟或频率为 10 秒\n\n\
                            **fix_delay 示例**:\n\
                            ```rust\n\
                            #[fix_delay(10)]\n\
                            async fn delayed_job() {\n\
                            }\n\
                            ```\n\n\
                            **fix_rate 示例**:\n\
                            ```rust\n\
                            #[fix_rate(10)]\n\
                            async fn periodic_job() {\n\
                            }\n\
                            ```"
                    .to_string(),
                })),
                insert_text: Some("10".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "60".to_string(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some("延迟/频率 60 秒（1 分钟）".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "延迟或频率为 60 秒（1 分钟）\n\n\
                            **fix_delay 示例**:\n\
                            ```rust\n\
                            #[fix_delay(60)]\n\
                            async fn delayed_job() {\n\
                            }\n\
                            ```\n\n\
                            **fix_rate 示例**:\n\
                            ```rust\n\
                            #[fix_rate(60)]\n\
                            async fn periodic_job() {\n\
                            }\n\
                            ```"
                    .to_string(),
                })),
                insert_text: Some("60".to_string()),
                ..Default::default()
            },
        ]
    }
}

impl Default for CompletionEngine {
    fn default() -> Self {
        Self::new(SchemaProvider::default())
    }
}

#[cfg(test)]
mod tests;
