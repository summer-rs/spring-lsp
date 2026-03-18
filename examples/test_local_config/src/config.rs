use serde::Deserialize;

// 模拟 spring 的 Configurable trait
pub trait Configurable {}

// 模拟 derive macro
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct OpenListConfig {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub use_origin: bool,
}

// 手动实现 Configurable（模拟 derive macro 的效果）
impl Configurable for OpenListConfig {}

// 添加 config_prefix 属性（这个需要在实际的 derive macro 中处理）
// 这里我们用注释来模拟
// #[config_prefix = "web-dav-client"]
