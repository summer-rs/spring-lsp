//! 配置 Schema 管理模块

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;

use crate::scanner::config::{ConfigField, ConfigScanner, ConfigurationStruct};

/// 配置 Schema
///
/// 包含所有插件的配置定义
///
/// 这个结构体用于解析 summer-rs 生成的 JSON Schema，格式为：
/// ```json
/// {
///   "properties": {
///     "web": { ... },
///     "redis": { ... }
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    /// 插件配置映射，键为配置前缀
    /// 在 JSON Schema 中对应 "properties" 字段
    #[serde(rename = "properties")]
    pub plugins: HashMap<String, serde_json::Value>,
}

/// 插件 Schema
///
/// 单个插件的配置定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSchema {
    /// 配置前缀（如 "web"、"redis" 等）
    pub prefix: String,
    /// 配置属性映射，键为属性名
    pub properties: HashMap<String, PropertySchema>,
}

/// 配置属性 Schema
///
/// 单个配置属性的定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    /// 属性名称
    pub name: String,
    /// 类型信息
    pub type_info: TypeInfo,
    /// 属性描述
    pub description: String,
    /// 默认值（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    /// 是否必需
    #[serde(default)]
    pub required: bool,
    /// 废弃信息（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<String>,
    /// 示例代码（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}

/// 类型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TypeInfo {
    /// 字符串类型
    String {
        /// 枚举值（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        enum_values: Option<Vec<String>>,
        /// 最小长度（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        min_length: Option<usize>,
        /// 最大长度（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        max_length: Option<usize>,
    },
    /// 整数类型
    Integer {
        /// 最小值（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<i64>,
        /// 最大值（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<i64>,
    },
    /// 浮点数类型
    Float {
        /// 最小值（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        /// 最大值（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
    },
    /// 布尔类型
    Boolean,
    /// 数组类型
    Array {
        /// 元素类型
        item_type: Box<TypeInfo>,
    },
    /// 对象类型（嵌套配置）
    Object {
        /// 嵌套属性
        properties: HashMap<String, PropertySchema>,
    },
}

/// 配置值
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    /// 字符串值
    String(String),
    /// 整数值
    Integer(i64),
    /// 浮点数值
    Float(f64),
    /// 布尔值
    Boolean(bool),
    /// 数组值
    Array(Vec<Value>),
    /// 对象值
    Table(HashMap<String, Value>),
}

/// Schema 提供者
///
/// 管理配置 Schema，提供配置项元数据查询
#[derive(Clone)]
pub struct SchemaProvider {
    /// Schema 数据（加载后不会改变，直接拥有即可）
    schema: ConfigSchema,
}

impl SchemaProvider {
    /// Schema URL
    const SCHEMA_URL: &'static str = "https://summer-rs.github.io/config-schema.json";

    /// 创建新的 Schema 提供者（使用空 Schema）
    pub fn new() -> Self {
        Self {
            schema: ConfigSchema {
                plugins: HashMap::new(),
            },
        }
    }

    /// 从 URL 加载 Schema
    ///
    /// 如果加载失败，使用内置的备用 Schema
    pub async fn load() -> anyhow::Result<Self> {
        // 尝试从 URL 加载 Schema
        match Self::load_from_url(Self::SCHEMA_URL).await {
            Ok(schema) => {
                tracing::info!("Successfully loaded schema from {}", Self::SCHEMA_URL);
                Ok(Self { schema })
            }
            Err(e) => {
                tracing::warn!("Failed to load schema from URL: {}, using fallback", e);
                // 使用内置备用 Schema
                Ok(Self::with_fallback_schema())
            }
        }
    }

    /// 从 URL 加载 Schema 并合并本地 Schema 文件
    ///
    /// # Arguments
    ///
    /// * `workspace_path` - 工作空间路径，用于查找本地 Schema 文件
    ///
    /// # Returns
    ///
    /// 返回合并了远程和本地 Schema 的 SchemaProvider
    ///
    /// # 本地 Schema 文件
    ///
    /// 支持以下文件（按优先级）：
    /// 1. `target/*/summer-lsp.schema.json` - build.rs 生成的 Schema 文件
    /// 2. `.summer-lsp.schema.json` - 手动生成的 Schema 文件（兼容旧版本）
    /// 3. 扫描 Rust 代码生成 Schema（fallback）
    pub async fn load_with_workspace(workspace_path: &Path) -> anyhow::Result<Self> {
        // 加载远程 Schema
        let mut provider = Self::load().await?;

        // 1. 尝试从 target 目录加载 Schema（build.rs 生成的）
        if let Some(schema_path) = Self::find_schema_in_target(workspace_path) {
            tracing::info!("Loading schema from target directory: {:?}", schema_path);
            match Self::load_local_schema_file(&schema_path) {
                Ok(local_schemas) => {
                    tracing::info!("Loaded {} local schemas from target", local_schemas.len());
                    for (prefix, schema) in local_schemas {
                        provider.schema.plugins.insert(prefix, schema);
                    }
                    return Ok(provider);
                }
                Err(e) => {
                    tracing::warn!("Failed to load schema from target: {}", e);
                }
            }
        }

        // 2. 尝试从根目录加载 Schema（兼容旧版本）
        let local_schema_path = workspace_path.join(".summer-lsp.schema.json");
        if local_schema_path.exists() {
            tracing::info!("Loading local schema from: {:?}", local_schema_path);
            match Self::load_local_schema_file(&local_schema_path) {
                Ok(local_schemas) => {
                    tracing::info!("Loaded {} local schemas from file", local_schemas.len());
                    for (prefix, schema) in local_schemas {
                        provider.schema.plugins.insert(prefix, schema);
                    }
                    return Ok(provider);
                }
                Err(e) => {
                    tracing::warn!("Failed to load local schema file: {}", e);
                }
            }
        }

        // 3. Fallback: 扫描 Rust 代码生成 Schema
        tracing::info!("Scanning local configurations in: {:?}", workspace_path);
        let scanner = ConfigScanner::new();
        match scanner.scan_configurations(workspace_path) {
            Ok(configurations) => {
                tracing::info!("Found {} local configuration structs", configurations.len());

                // 将本地配置转换为 Schema 并合并
                for config in configurations {
                    let schema_json = Self::configuration_to_schema(&config);
                    provider
                        .schema
                        .plugins
                        .insert(config.prefix.clone(), schema_json);
                    tracing::debug!("Added local configuration: {}", config.prefix);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to scan local configurations: {}", e);
            }
        }

        Ok(provider)
    }

    /// 在 target 目录中查找 Schema 文件
    ///
    /// build.rs 生成的文件位于 target/{profile}/build/{package}/out/summer-lsp.schema.json
    /// 在 workspace 中，每个 crate 都可能生成自己的 Schema 文件
    /// 我们需要找到所有的 Schema 文件并合并
    fn find_schema_in_target(workspace_path: &Path) -> Option<std::path::PathBuf> {
        let target_dir = workspace_path.join("target");
        if !target_dir.exists() {
            return None;
        }

        // 收集所有 Schema 文件
        let mut schema_files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&target_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // 检查 target/{profile}/build/*/out/summer-lsp.schema.json
                    let build_dir = path.join("build");
                    if build_dir.exists() {
                        if let Ok(build_entries) = std::fs::read_dir(&build_dir) {
                            for build_entry in build_entries.flatten() {
                                let out_dir = build_entry.path().join("out");
                                let schema_path = out_dir.join("summer-lsp.schema.json");
                                if schema_path.exists() {
                                    if let Ok(metadata) = std::fs::metadata(&schema_path) {
                                        if let Ok(modified) = metadata.modified() {
                                            schema_files.push((schema_path, modified));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if schema_files.is_empty() {
            return None;
        }

        // 如果只有一个 Schema 文件，直接返回
        if schema_files.len() == 1 {
            return Some(schema_files[0].0.clone());
        }

        // 多个 Schema 文件：合并它们
        tracing::info!("Found {} schema files, merging...", schema_files.len());

        // 按修改时间排序，优先使用最新的
        schema_files.sort_by(|a, b| b.1.cmp(&a.1));

        // 尝试合并所有 Schema 文件
        match Self::merge_schema_files(&schema_files) {
            Ok(merged_path) => Some(merged_path),
            Err(e) => {
                tracing::warn!("Failed to merge schema files: {}, using latest", e);
                // 合并失败，返回最新的
                Some(schema_files[0].0.clone())
            }
        }
    }

    /// 合并多个 Schema 文件
    ///
    /// 在 workspace 中，每个 crate 可能生成自己的 Schema
    /// 我们需要将它们合并为一个完整的 Schema
    fn merge_schema_files(
        schema_files: &[(std::path::PathBuf, std::time::SystemTime)],
    ) -> anyhow::Result<std::path::PathBuf> {
        use std::fs;

        let mut merged_properties = serde_json::Map::new();

        // 读取并合并所有 Schema 文件
        for (path, _) in schema_files {
            tracing::debug!("Merging schema from: {:?}", path);
            let content = fs::read_to_string(path)?;
            let schema: serde_json::Value = serde_json::from_str(&content)?;

            if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
                for (key, value) in properties {
                    // 如果键已存在，后面的会覆盖前面的（按时间排序，新的优先）
                    merged_properties.insert(key.clone(), value.clone());
                }
            }
        }

        // 创建合并后的 Schema
        let merged_schema = serde_json::json!({
            "type": "object",
            "properties": merged_properties
        });

        // 写入临时文件
        let temp_dir = std::env::temp_dir();
        let merged_path = temp_dir.join("summer-lsp-merged.schema.json");
        fs::write(&merged_path, serde_json::to_string_pretty(&merged_schema)?)?;

        tracing::info!(
            "Merged {} schema files ({} configs) into: {:?}",
            schema_files.len(),
            merged_properties.len(),
            merged_path
        );

        Ok(merged_path)
    }

    /// 从本地文件加载 Schema
    fn load_local_schema_file(path: &Path) -> anyhow::Result<HashMap<String, serde_json::Value>> {
        let content = std::fs::read_to_string(path)?;
        let schema: serde_json::Value = serde_json::from_str(&content)?;

        let mut schemas = HashMap::new();

        // 提取 properties 字段
        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            for (key, value) in properties {
                schemas.insert(key.clone(), value.clone());
            }
        }

        Ok(schemas)
    }

    /// 将 ConfigurationStruct 转换为 JSON Schema
    fn configuration_to_schema(config: &ConfigurationStruct) -> serde_json::Value {
        let mut properties = serde_json::Map::new();

        for field in &config.fields {
            let field_schema = Self::field_to_schema(field);
            properties.insert(field.name.clone(), field_schema);
        }

        json!({
            "type": "object",
            "properties": properties,
            "description": format!("Configuration for {}", config.name)
        })
    }

    /// 将 ConfigField 转换为 JSON Schema 属性
    fn field_to_schema(field: &ConfigField) -> serde_json::Value {
        let mut schema = serde_json::Map::new();

        // 推断类型
        let (field_type, is_optional) = if field.optional {
            // Option<T> 类型，提取内部类型
            let inner_type = field
                .type_name
                .strip_prefix("Option<")
                .and_then(|s| s.strip_suffix('>'))
                .unwrap_or(&field.type_name);
            (inner_type, true)
        } else {
            (field.type_name.as_str(), false)
        };

        // 映射 Rust 类型到 JSON Schema 类型
        let json_type = match field_type {
            "String" | "str" | "&str" => "string",
            "bool" => "boolean",
            "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128"
            | "isize" | "usize" => "integer",
            "f32" | "f64" => "number",
            t if t.starts_with("Vec<") => "array",
            t if t.starts_with("HashMap<") || t.starts_with("BTreeMap<") => "object",
            _ => "string", // 默认为字符串
        };

        schema.insert("type".to_string(), json!(json_type));

        // 添加描述
        if let Some(desc) = &field.description {
            schema.insert("description".to_string(), json!(desc));
        }

        // 如果是可选的，添加说明
        if is_optional {
            let desc = schema
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            schema.insert(
                "description".to_string(),
                json!(if desc.is_empty() {
                    "Optional field".to_string()
                } else {
                    format!("{} (optional)", desc)
                }),
            );
        }

        serde_json::Value::Object(schema)
    }

    /// 从指定 URL 加载 Schema
    async fn load_from_url(url: &str) -> anyhow::Result<ConfigSchema> {
        let response = reqwest::get(url).await?;
        let schema = response.json::<ConfigSchema>().await?;
        Ok(schema)
    }

    /// 使用内置备用 Schema
    fn with_fallback_schema() -> Self {
        let fallback_schema = Self::create_fallback_schema();
        Self {
            schema: fallback_schema,
        }
    }

    /// 创建内置备用 Schema
    ///
    /// 包含常见的 summer-rs 插件配置
    fn create_fallback_schema() -> ConfigSchema {
        let mut plugins = HashMap::new();

        // Web 插件配置
        let web_schema = json!({
            "type": "object",
            "properties": {
                "host": {
                    "type": "string",
                    "description": "Web server host address",
                    "default": "0.0.0.0"
                },
                "port": {
                    "type": "integer",
                    "description": "Web server port",
                    "default": 8080,
                    "minimum": 1,
                    "maximum": 65535
                }
            }
        });
        plugins.insert("web".to_string(), web_schema);

        // Redis 插件配置
        let redis_schema = json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "Redis connection URL",
                    "default": "redis://localhost:6379"
                }
            }
        });
        plugins.insert("redis".to_string(), redis_schema);

        ConfigSchema { plugins }
    }

    /// 获取插件 Schema
    ///
    /// 检查指定的配置前缀是否在 Schema 中定义
    pub fn has_plugin(&self, prefix: &str) -> bool {
        self.schema.plugins.contains_key(prefix)
    }

    /// 获取所有配置前缀
    ///
    /// 返回所有已注册插件的配置前缀列表
    pub fn get_all_prefixes(&self) -> Vec<String> {
        self.schema.plugins.keys().cloned().collect()
    }

    /// 获取插件的 Schema
    ///
    /// 返回指定插件的 JSON Schema
    pub fn get_plugin_schema(&self, prefix: &str) -> Option<&serde_json::Value> {
        self.schema.plugins.get(prefix)
    }

    /// 检查配置属性是否存在
    ///
    /// 查询指定插件的指定属性是否在 Schema 中定义
    pub fn has_property(&self, prefix: &str, property: &str) -> bool {
        if let Some(plugin_schema) = self.schema.plugins.get(prefix) {
            // 尝试解析为 JSON Schema 对象
            if let Some(properties) = plugin_schema.get("properties") {
                if let Some(props_obj) = properties.as_object() {
                    return props_obj.contains_key(property);
                }
            }
        }
        false
    }

    /// 获取插件的结构化 Schema
    ///
    /// 将 JSON Schema 解析为 PluginSchema 结构
    pub fn get_plugin(&self, prefix: &str) -> Option<PluginSchema> {
        let plugin_json = self.schema.plugins.get(prefix)?;

        // 获取 $defs（如果存在）
        let defs = plugin_json.get("$defs").unwrap_or(&serde_json::Value::Null);

        // 解析 properties
        let properties_json = plugin_json.get("properties")?.as_object()?;
        let mut properties = HashMap::new();

        for (key, value) in properties_json {
            if let Some(property_schema) = Self::parse_property_schema_with_defs(key, value, defs) {
                properties.insert(key.clone(), property_schema);
            }
        }

        Some(PluginSchema {
            prefix: prefix.to_string(),
            properties,
        })
    }

    /// 解析属性 Schema
    fn parse_property_schema(name: &str, value: &serde_json::Value) -> Option<PropertySchema> {
        // 处理 $ref 引用
        if value.get("$ref").is_some() {
            // 暂时跳过 $ref，因为需要上下文来解析
            // 我们将在 get_plugin 中处理这个
            return None;
        }

        let type_info = Self::parse_type_info(value)?;
        let description = value
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string();

        let default = value.get("default").and_then(Self::parse_value);
        let required = value
            .get("required")
            .and_then(|r| r.as_bool())
            .unwrap_or(false);
        let deprecated = value
            .get("deprecated")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());
        let example = value
            .get("example")
            .and_then(|e| e.as_str())
            .map(|s| s.to_string());

        Some(PropertySchema {
            name: name.to_string(),
            type_info,
            description,
            default,
            required,
            deprecated,
            example,
        })
    }

    /// 解析属性 Schema（带 $defs 上下文）
    fn parse_property_schema_with_defs(
        name: &str,
        value: &serde_json::Value,
        defs: &serde_json::Value,
    ) -> Option<PropertySchema> {
        // 处理 $ref 引用
        if let Some(ref_path) = value.get("$ref").and_then(|r| r.as_str()) {
            // 解析引用路径，例如 "#/$defs/LogLevel"
            if let Some(def_name) = ref_path.strip_prefix("#/$defs/") {
                if let Some(def_value) = defs.get(def_name) {
                    // 递归解析引用的定义
                    return Self::parse_property_schema_with_defs(name, def_value, defs);
                }
            }
            // 如果无法解析引用，返回 None
            return None;
        }

        let type_info = Self::parse_type_info_with_defs(value, defs)?;
        let description = value
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string();

        let default = value.get("default").and_then(Self::parse_value);
        let required = value
            .get("required")
            .and_then(|r| r.as_bool())
            .unwrap_or(false);
        let deprecated = value
            .get("deprecated")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());
        let example = value
            .get("example")
            .and_then(|e| e.as_str())
            .map(|s| s.to_string());

        Some(PropertySchema {
            name: name.to_string(),
            type_info,
            description,
            default,
            required,
            deprecated,
            example,
        })
    }

    /// 解析类型信息
    fn parse_type_info(value: &serde_json::Value) -> Option<TypeInfo> {
        Self::parse_type_info_with_defs(value, &serde_json::Value::Null)
    }

    /// 解析类型信息（带 $defs 上下文）
    fn parse_type_info_with_defs(
        value: &serde_json::Value,
        _defs: &serde_json::Value,
    ) -> Option<TypeInfo> {
        // 处理 oneOf（通常用于枚举）
        if let Some(one_of) = value.get("oneOf").and_then(|o| o.as_array()) {
            // 提取所有 const 值作为枚举
            let enum_values: Vec<String> = one_of
                .iter()
                .filter_map(|item| item.get("const").and_then(|c| c.as_str()))
                .map(|s| s.to_string())
                .collect();

            if !enum_values.is_empty() {
                return Some(TypeInfo::String {
                    enum_values: Some(enum_values),
                    min_length: None,
                    max_length: None,
                });
            }
        }

        // 处理顶层 enum
        if let Some(enum_array) = value.get("enum").and_then(|e| e.as_array()) {
            let enum_values: Vec<String> = enum_array
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();

            if !enum_values.is_empty() {
                return Some(TypeInfo::String {
                    enum_values: Some(enum_values),
                    min_length: None,
                    max_length: None,
                });
            }
        }

        let type_str = value.get("type")?.as_str()?;

        match type_str {
            "string" => {
                let enum_values = value.get("enum").and_then(|e| e.as_array()).map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                });
                let min_length = value
                    .get("minLength")
                    .and_then(|m| m.as_u64())
                    .map(|n| n as usize);
                let max_length = value
                    .get("maxLength")
                    .and_then(|m| m.as_u64())
                    .map(|n| n as usize);

                Some(TypeInfo::String {
                    enum_values,
                    min_length,
                    max_length,
                })
            }
            "integer" => {
                let min = value.get("minimum").and_then(|m| m.as_i64());
                let max = value.get("maximum").and_then(|m| m.as_i64());

                Some(TypeInfo::Integer { min, max })
            }
            "number" => {
                let min = value.get("minimum").and_then(|m| m.as_f64());
                let max = value.get("maximum").and_then(|m| m.as_f64());

                Some(TypeInfo::Float { min, max })
            }
            "boolean" => Some(TypeInfo::Boolean),
            "array" => {
                let items = value.get("items")?;
                let item_type = Self::parse_type_info(items)?;

                Some(TypeInfo::Array {
                    item_type: Box::new(item_type),
                })
            }
            "object" => {
                let properties_json = value.get("properties")?.as_object()?;
                let mut properties = HashMap::new();

                for (key, val) in properties_json {
                    if let Some(prop_schema) = Self::parse_property_schema(key, val) {
                        properties.insert(key.clone(), prop_schema);
                    }
                }

                Some(TypeInfo::Object { properties })
            }
            _ => None,
        }
    }

    /// 解析值
    fn parse_value(value: &serde_json::Value) -> Option<Value> {
        match value {
            serde_json::Value::String(s) => Some(Value::String(s.clone())),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Some(Value::Integer(i))
                } else {
                    n.as_f64().map(Value::Float)
                }
            }
            serde_json::Value::Bool(b) => Some(Value::Boolean(*b)),
            serde_json::Value::Array(arr) => {
                let values: Option<Vec<Value>> = arr.iter().map(Self::parse_value).collect();
                values.map(Value::Array)
            }
            serde_json::Value::Object(obj) => {
                let mut table = HashMap::new();
                for (k, v) in obj {
                    if let Some(val) = Self::parse_value(v) {
                        table.insert(k.clone(), val);
                    }
                }
                Some(Value::Table(table))
            }
            serde_json::Value::Null => None,
        }
    }
}

impl Default for SchemaProvider {
    fn default() -> Self {
        Self::with_fallback_schema()
    }
}

impl SchemaProvider {
    /// 从给定的 ConfigSchema 创建 SchemaProvider（用于测试）
    ///
    /// 这个方法主要用于属性测试，允许使用自定义的 Schema 创建提供者
    pub fn from_schema(schema: ConfigSchema) -> Self {
        Self { schema }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_real_schema() {
        // 测试加载真实的 Schema
        let provider = SchemaProvider::load().await.unwrap();

        // 验证常见的插件存在
        assert!(provider.has_plugin("logger"), "Should have logger plugin");
        assert!(provider.has_plugin("grpc"), "Should have grpc plugin");
        assert!(provider.has_plugin("web"), "Should have web plugin");
        assert!(provider.has_plugin("redis"), "Should have redis plugin");

        // 验证可以获取插件 Schema
        let grpc_schema = provider.get_plugin("grpc");
        assert!(grpc_schema.is_some(), "Should be able to get grpc schema");

        if let Some(grpc) = grpc_schema {
            // 验证 graceful 属性存在
            assert!(
                grpc.properties.contains_key("graceful"),
                "GRPC should have 'graceful' property"
            );

            // 验证 port 属性存在
            assert!(
                grpc.properties.contains_key("port"),
                "GRPC should have 'port' property"
            );
        }

        // 验证 logger 插件
        let logger_schema = provider.get_plugin("logger");
        assert!(
            logger_schema.is_some(),
            "Should be able to get logger schema"
        );

        if let Some(logger) = logger_schema {
            // 打印调试信息
            println!(
                "Logger properties found: {:?}",
                logger.properties.keys().collect::<Vec<_>>()
            );

            // 验证常见属性
            assert!(
                logger.properties.contains_key("level"),
                "Logger should have 'level' property. Found: {:?}",
                logger.properties.keys().collect::<Vec<_>>()
            );
            assert!(
                logger.properties.contains_key("format"),
                "Logger should have 'format' property"
            );
        }
    }

    #[test]
    fn test_parse_property_schema() {
        let json = serde_json::json!({
            "type": "string",
            "description": "Test property",
            "default": "test_value",
            "enum": ["value1", "value2", "value3"]
        });

        let property = SchemaProvider::parse_property_schema("test_prop", &json);
        assert!(property.is_some());

        let prop = property.unwrap();
        assert_eq!(prop.name, "test_prop");
        assert_eq!(prop.description, "Test property");

        // 验证枚举值
        if let TypeInfo::String { enum_values, .. } = &prop.type_info {
            assert!(enum_values.is_some());
            let enums = enum_values.as_ref().unwrap();
            assert_eq!(enums.len(), 3);
            assert!(enums.contains(&"value1".to_string()));
        } else {
            panic!("Expected String type");
        }

        // 验证默认值
        assert!(prop.default.is_some());
        if let Some(Value::String(s)) = &prop.default {
            assert_eq!(s, "test_value");
        } else {
            panic!("Expected String default value");
        }
    }

    #[test]
    fn test_parse_integer_type() {
        let json = serde_json::json!({
            "type": "integer",
            "minimum": 1,
            "maximum": 65535
        });

        let type_info = SchemaProvider::parse_type_info(&json);
        assert!(type_info.is_some());

        if let Some(TypeInfo::Integer { min, max }) = type_info {
            assert_eq!(min, Some(1));
            assert_eq!(max, Some(65535));
        } else {
            panic!("Expected Integer type");
        }
    }

    #[test]
    fn test_parse_boolean_type() {
        let json = serde_json::json!({
            "type": "boolean"
        });

        let type_info = SchemaProvider::parse_type_info(&json);
        assert!(type_info.is_some());
        assert!(matches!(type_info.unwrap(), TypeInfo::Boolean));
    }

    #[test]
    fn test_find_schema_in_target() {
        use std::fs;
        use tempfile::TempDir;

        // 创建临时目录结构
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();

        // 创建 target/debug/build/my-package/out/summer-lsp.schema.json
        let target_dir = workspace_path.join("target/debug/build/my-package/out");
        fs::create_dir_all(&target_dir).unwrap();

        let schema_path = target_dir.join("summer-lsp.schema.json");
        let schema_content = serde_json::json!({
            "properties": {
                "test-config": {
                    "type": "object",
                    "properties": {
                        "field1": {
                            "type": "string",
                            "description": "Test field"
                        }
                    }
                }
            }
        });
        fs::write(
            &schema_path,
            serde_json::to_string_pretty(&schema_content).unwrap(),
        )
        .unwrap();

        // 测试查找功能
        let found = SchemaProvider::find_schema_in_target(workspace_path);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), schema_path);
    }

    #[test]
    fn test_find_schema_in_target_multiple_profiles() {
        use std::fs;
        use std::thread;
        use std::time::Duration;
        use tempfile::TempDir;

        // 创建临时目录结构
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();

        // 创建多个 profile 的 Schema 文件
        let debug_dir = workspace_path.join("target/debug/build/my-package/out");
        fs::create_dir_all(&debug_dir).unwrap();
        let debug_schema = debug_dir.join("summer-lsp.schema.json");
        fs::write(
            &debug_schema,
            serde_json::json!({
                "properties": {
                    "debug-config": {
                        "type": "object"
                    }
                }
            })
            .to_string(),
        )
        .unwrap();

        // 等待一小段时间确保文件时间戳不同
        thread::sleep(Duration::from_millis(10));

        let release_dir = workspace_path.join("target/release/build/my-package/out");
        fs::create_dir_all(&release_dir).unwrap();
        let release_schema = release_dir.join("summer-lsp.schema.json");
        fs::write(
            &release_schema,
            serde_json::json!({
                "properties": {
                    "release-config": {
                        "type": "object"
                    }
                }
            })
            .to_string(),
        )
        .unwrap();

        // 应该合并两个 profile 的 Schema
        let found = SchemaProvider::find_schema_in_target(workspace_path);
        assert!(found.is_some());

        let merged_path = found.unwrap();
        let content = fs::read_to_string(&merged_path).unwrap();
        let schema: serde_json::Value = serde_json::from_str(&content).unwrap();

        let properties = schema
            .get("properties")
            .and_then(|p| p.as_object())
            .unwrap();

        // 应该包含两个 profile 的配置
        assert_eq!(properties.len(), 2);
        assert!(properties.contains_key("debug-config"));
        assert!(properties.contains_key("release-config"));
    }

    #[test]
    fn test_find_schema_in_target_not_exists() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();

        // target 目录不存在
        let found = SchemaProvider::find_schema_in_target(workspace_path);
        assert!(found.is_none());
    }

    #[test]
    fn test_load_local_schema_file() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let schema_path = temp_dir.path().join("test-schema.json");

        let schema_content = serde_json::json!({
            "properties": {
                "web": {
                    "type": "object",
                    "properties": {
                        "port": {
                            "type": "integer",
                            "default": 8080
                        }
                    }
                },
                "database": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string"
                        }
                    }
                }
            }
        });

        fs::write(
            &schema_path,
            serde_json::to_string_pretty(&schema_content).unwrap(),
        )
        .unwrap();

        let schemas = SchemaProvider::load_local_schema_file(&schema_path).unwrap();
        assert_eq!(schemas.len(), 2);
        assert!(schemas.contains_key("web"));
        assert!(schemas.contains_key("database"));
    }

    #[test]
    fn test_merge_schema_files() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // 创建第一个 Schema 文件（crate1）
        let schema1_path = temp_dir.path().join("schema1.json");
        let schema1_content = serde_json::json!({
            "properties": {
                "service-a": {
                    "type": "object",
                    "properties": {
                        "endpoint": {
                            "type": "string"
                        }
                    }
                }
            }
        });
        fs::write(
            &schema1_path,
            serde_json::to_string_pretty(&schema1_content).unwrap(),
        )
        .unwrap();

        // 创建第二个 Schema 文件（crate2）
        let schema2_path = temp_dir.path().join("schema2.json");
        let schema2_content = serde_json::json!({
            "properties": {
                "service-b": {
                    "type": "object",
                    "properties": {
                        "port": {
                            "type": "integer"
                        }
                    }
                }
            }
        });
        fs::write(
            &schema2_path,
            serde_json::to_string_pretty(&schema2_content).unwrap(),
        )
        .unwrap();

        // 模拟文件时间戳
        let time1 = std::time::SystemTime::now();
        let time2 = std::time::SystemTime::now();

        let schema_files = vec![(schema1_path, time1), (schema2_path, time2)];

        // 合并 Schema 文件
        let merged_path = SchemaProvider::merge_schema_files(&schema_files).unwrap();
        assert!(merged_path.exists());

        // 验证合并结果
        let merged_content = fs::read_to_string(&merged_path).unwrap();
        let merged_schema: serde_json::Value = serde_json::from_str(&merged_content).unwrap();

        let properties = merged_schema
            .get("properties")
            .and_then(|p| p.as_object())
            .unwrap();

        // 应该包含两个 crate 的配置
        assert_eq!(properties.len(), 2);
        assert!(properties.contains_key("service-a"));
        assert!(properties.contains_key("service-b"));
    }

    #[test]
    fn test_find_schema_in_target_multiple_crates() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();

        // 创建多个 crate 的 Schema 文件
        let crate1_dir = workspace_path.join("target/debug/build/crate1/out");
        fs::create_dir_all(&crate1_dir).unwrap();
        let schema1_path = crate1_dir.join("summer-lsp.schema.json");
        fs::write(
            &schema1_path,
            serde_json::json!({
                "properties": {
                    "service-a": {
                        "type": "object"
                    }
                }
            })
            .to_string(),
        )
        .unwrap();

        let crate2_dir = workspace_path.join("target/debug/build/crate2/out");
        fs::create_dir_all(&crate2_dir).unwrap();
        let schema2_path = crate2_dir.join("summer-lsp.schema.json");
        fs::write(
            &schema2_path,
            serde_json::json!({
                "properties": {
                    "service-b": {
                        "type": "object"
                    }
                }
            })
            .to_string(),
        )
        .unwrap();

        // 查找并合并
        let found = SchemaProvider::find_schema_in_target(workspace_path);
        assert!(found.is_some());

        let merged_path = found.unwrap();
        let content = fs::read_to_string(&merged_path).unwrap();
        let schema: serde_json::Value = serde_json::from_str(&content).unwrap();

        let properties = schema
            .get("properties")
            .and_then(|p| p.as_object())
            .unwrap();

        // 应该包含两个 crate 的配置
        assert_eq!(properties.len(), 2);
        assert!(properties.contains_key("service-a"));
        assert!(properties.contains_key("service-b"));
    }
}
