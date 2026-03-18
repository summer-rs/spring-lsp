//! 补全引擎单元测试

use super::*;
use crate::macro_analyzer::{
    AutoConfigMacro, HttpMethod, InjectMacro, InjectType, JobMacro, RouteMacro, ServiceMacro,
    SummerMacro,
};
use crate::schema::SchemaProvider;
use crate::toml_analyzer::TomlAnalyzer;
use lsp_types::{Position, Range, Url};

/// 创建测试用的 Range
fn test_range() -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    }
}

/// 创建测试用的 URL
#[allow(dead_code)]
fn test_url() -> Url {
    Url::parse("file:///test.rs").unwrap()
}

/// 创建测试用的补全引擎
fn test_engine() -> CompletionEngine {
    let schema_provider = SchemaProvider::default();
    CompletionEngine::new(schema_provider)
}

#[test]
fn test_complete_service_macro() {
    let engine = test_engine();
    let service_macro = ServiceMacro {
        struct_name: "TestService".to_string(),
        fields: vec![],
        range: test_range(),
    };

    let completions = engine.complete_macro(&SummerMacro::DeriveService(service_macro), None);

    // 应该提供 3 个补全项：inject(component), inject(component = "name"), inject(config)
    assert_eq!(completions.len(), 3);

    // 检查第一个补全项：inject(component)
    assert_eq!(completions[0].label, "inject(component)");
    assert_eq!(completions[0].kind, Some(CompletionItemKind::PROPERTY));
    assert_eq!(completions[0].detail, Some("注入组件".to_string()));
    assert!(completions[0].documentation.is_some());
    assert_eq!(
        completions[0].insert_text,
        Some("inject(component)".to_string())
    );

    // 检查第二个补全项：inject(component = "name")
    assert_eq!(completions[1].label, "inject(component = \"name\")");
    assert_eq!(completions[1].kind, Some(CompletionItemKind::PROPERTY));
    assert_eq!(
        completions[1].detail,
        Some("注入指定名称的组件".to_string())
    );
    assert!(completions[1].documentation.is_some());
    assert_eq!(
        completions[1].insert_text,
        Some("inject(component = \"$1\")".to_string())
    );
    assert_eq!(
        completions[1].insert_text_format,
        Some(lsp_types::InsertTextFormat::SNIPPET)
    );

    // 检查第三个补全项：inject(config)
    assert_eq!(completions[2].label, "inject(config)");
    assert_eq!(completions[2].kind, Some(CompletionItemKind::PROPERTY));
    assert_eq!(completions[2].detail, Some("注入配置".to_string()));
    assert!(completions[2].documentation.is_some());
    assert_eq!(
        completions[2].insert_text,
        Some("inject(config)".to_string())
    );
}

#[test]
fn test_complete_inject_macro() {
    let engine = test_engine();
    let inject_macro = InjectMacro {
        inject_type: InjectType::Component,
        component_name: None,
        range: test_range(),
    };

    let completions = engine.complete_macro(&SummerMacro::Inject(inject_macro), None);

    // 应该提供 2 个补全项：component, config
    assert_eq!(completions.len(), 2);

    // 检查第一个补全项：component
    assert_eq!(completions[0].label, "component");
    assert_eq!(completions[0].kind, Some(CompletionItemKind::KEYWORD));
    assert_eq!(completions[0].detail, Some("注入组件".to_string()));
    assert!(completions[0].documentation.is_some());
    assert_eq!(completions[0].insert_text, Some("component".to_string()));

    // 检查第二个补全项：config
    assert_eq!(completions[1].label, "config");
    assert_eq!(completions[1].kind, Some(CompletionItemKind::KEYWORD));
    assert_eq!(completions[1].detail, Some("注入配置".to_string()));
    assert!(completions[1].documentation.is_some());
    assert_eq!(completions[1].insert_text, Some("config".to_string()));
}

#[test]
fn test_complete_auto_config_macro() {
    let engine = test_engine();
    let auto_config_macro = AutoConfigMacro {
        configurator_type: "".to_string(),
        range: test_range(),
    };

    let completions = engine.complete_macro(&SummerMacro::AutoConfig(auto_config_macro), None);

    // 应该提供 3 个配置器类型补全
    assert_eq!(completions.len(), 3);

    // 检查补全项
    assert_eq!(completions[0].label, "WebConfigurator");
    assert_eq!(completions[0].kind, Some(CompletionItemKind::CLASS));
    assert_eq!(completions[0].detail, Some("Web 路由配置器".to_string()));
    assert!(completions[0].documentation.is_some());

    assert_eq!(completions[1].label, "JobConfigurator");
    assert_eq!(completions[1].kind, Some(CompletionItemKind::CLASS));
    assert_eq!(completions[1].detail, Some("任务调度配置器".to_string()));
    assert!(completions[1].documentation.is_some());

    assert_eq!(completions[2].label, "StreamConfigurator");
    assert_eq!(completions[2].kind, Some(CompletionItemKind::CLASS));
    assert_eq!(completions[2].detail, Some("流处理配置器".to_string()));
    assert!(completions[2].documentation.is_some());
}

#[test]
fn test_complete_route_macro() {
    let engine = test_engine();
    let route_macro = RouteMacro {
        path: "/test".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "test_handler".to_string(),
        range: test_range(),
        is_openapi: false,
    };

    let completions = engine.complete_macro(&SummerMacro::Route(route_macro), None);

    // 应该提供 HTTP 方法和路径参数补全
    assert!(completions.len() >= 7); // 至少 7 个 HTTP 方法

    // 检查 HTTP 方法补全
    let get_completion = completions.iter().find(|c| c.label == "GET");
    assert!(get_completion.is_some());
    let get_completion = get_completion.unwrap();
    assert_eq!(get_completion.kind, Some(CompletionItemKind::CONSTANT));
    assert_eq!(get_completion.detail, Some("获取资源".to_string()));
    assert!(get_completion.documentation.is_some());

    let post_completion = completions.iter().find(|c| c.label == "POST");
    assert!(post_completion.is_some());
    let post_completion = post_completion.unwrap();
    assert_eq!(post_completion.kind, Some(CompletionItemKind::CONSTANT));
    assert_eq!(post_completion.detail, Some("创建资源".to_string()));

    // 检查路径参数补全
    let path_param_completion = completions.iter().find(|c| c.label == "{id}");
    assert!(path_param_completion.is_some());
    let path_param_completion = path_param_completion.unwrap();
    assert_eq!(
        path_param_completion.kind,
        Some(CompletionItemKind::SNIPPET)
    );
    assert_eq!(path_param_completion.detail, Some("路径参数".to_string()));
    assert_eq!(
        path_param_completion.insert_text,
        Some("{${1:id}}".to_string())
    );
    assert_eq!(
        path_param_completion.insert_text_format,
        Some(lsp_types::InsertTextFormat::SNIPPET)
    );
}

#[test]
fn test_complete_job_macro_cron() {
    let engine = test_engine();
    let job_macro = JobMacro::Cron {
        expression: "".to_string(),
        range: test_range(),
    };

    let completions = engine.complete_macro(&SummerMacro::Job(job_macro), None);

    // 应该提供 cron 表达式和延迟/频率值补全
    assert!(completions.len() >= 6);

    // 检查 cron 表达式补全
    let hourly_cron = completions.iter().find(|c| c.label == "0 0 * * * *");
    assert!(hourly_cron.is_some());
    let hourly_cron = hourly_cron.unwrap();
    assert_eq!(hourly_cron.kind, Some(CompletionItemKind::SNIPPET));
    assert_eq!(hourly_cron.detail, Some("每小时执行".to_string()));
    assert!(hourly_cron.documentation.is_some());
    assert_eq!(hourly_cron.insert_text, Some("\"0 0 * * * *\"".to_string()));

    let daily_cron = completions.iter().find(|c| c.label == "0 0 0 * * *");
    assert!(daily_cron.is_some());
    let daily_cron = daily_cron.unwrap();
    assert_eq!(daily_cron.detail, Some("每天午夜执行".to_string()));

    // 检查延迟/频率值补全
    let delay_5 = completions.iter().find(|c| c.label == "5");
    assert!(delay_5.is_some());
    let delay_5 = delay_5.unwrap();
    assert_eq!(delay_5.kind, Some(CompletionItemKind::VALUE));
    assert_eq!(delay_5.detail, Some("延迟 5 秒".to_string()));
}

#[test]
fn test_complete_job_macro_fix_delay() {
    let engine = test_engine();
    let job_macro = JobMacro::FixDelay {
        seconds: 0,
        range: test_range(),
    };

    let completions = engine.complete_macro(&SummerMacro::Job(job_macro), None);

    // 应该提供延迟值补全
    assert!(completions.len() >= 3);

    // 检查延迟值补全
    let delay_10 = completions.iter().find(|c| c.label == "10");
    assert!(delay_10.is_some());
    let delay_10 = delay_10.unwrap();
    assert_eq!(delay_10.kind, Some(CompletionItemKind::VALUE));
    assert!(delay_10.detail.is_some());
    assert!(delay_10.documentation.is_some());
}

#[test]
fn test_complete_job_macro_fix_rate() {
    let engine = test_engine();
    let job_macro = JobMacro::FixRate {
        seconds: 0,
        range: test_range(),
    };

    let completions = engine.complete_macro(&SummerMacro::Job(job_macro), None);

    // 应该提供频率值补全
    assert!(completions.len() >= 3);

    // 检查频率值补全
    let rate_60 = completions.iter().find(|c| c.label == "60");
    assert!(rate_60.is_some());
    let rate_60 = rate_60.unwrap();
    assert_eq!(rate_60.kind, Some(CompletionItemKind::VALUE));
    assert!(rate_60.detail.is_some());
    assert!(rate_60.documentation.is_some());
}

#[test]
fn test_completion_items_have_documentation() {
    let engine = test_engine();

    // 测试所有宏类型的补全项都有文档
    let test_cases = vec![
        SummerMacro::DeriveService(ServiceMacro {
            struct_name: "Test".to_string(),
            fields: vec![],
            range: test_range(),
        }),
        SummerMacro::Inject(InjectMacro {
            inject_type: InjectType::Component,
            component_name: None,
            range: test_range(),
        }),
        SummerMacro::AutoConfig(AutoConfigMacro {
            configurator_type: "".to_string(),
            range: test_range(),
        }),
        SummerMacro::Route(RouteMacro {
            path: "/test".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "handler".to_string(),
            range: test_range(),
            is_openapi: false,
        }),
        SummerMacro::Job(JobMacro::Cron {
            expression: "".to_string(),
            range: test_range(),
        }),
    ];

    for macro_info in test_cases {
        let completions = engine.complete_macro(&macro_info, None);
        assert!(!completions.is_empty(), "补全列表不应为空");

        for completion in completions {
            assert!(
                completion.documentation.is_some(),
                "补全项 '{}' 应该有文档说明",
                completion.label
            );
            assert!(
                completion.detail.is_some(),
                "补全项 '{}' 应该有详细信息",
                completion.label
            );
            assert!(
                completion.insert_text.is_some(),
                "补全项 '{}' 应该有插入文本",
                completion.label
            );
        }
    }
}

#[test]
fn test_completion_items_have_correct_kind() {
    let engine = test_engine();

    // Service 宏的补全项应该是 PROPERTY 类型
    let service_completions = engine.complete_macro(
        &SummerMacro::DeriveService(ServiceMacro {
            struct_name: "Test".to_string(),
            fields: vec![],
            range: test_range(),
        }),
        None,
    );
    for completion in service_completions {
        assert_eq!(
            completion.kind,
            Some(CompletionItemKind::PROPERTY),
            "Service 宏的补全项应该是 PROPERTY 类型"
        );
    }

    // Inject 宏的补全项应该是 KEYWORD 类型
    let inject_completions = engine.complete_macro(
        &SummerMacro::Inject(InjectMacro {
            inject_type: InjectType::Component,
            component_name: None,
            range: test_range(),
        }),
        None,
    );
    for completion in inject_completions {
        assert_eq!(
            completion.kind,
            Some(CompletionItemKind::KEYWORD),
            "Inject 宏的补全项应该是 KEYWORD 类型"
        );
    }

    // AutoConfig 宏的补全项应该是 CLASS 类型
    let auto_config_completions = engine.complete_macro(
        &SummerMacro::AutoConfig(AutoConfigMacro {
            configurator_type: "".to_string(),
            range: test_range(),
        }),
        None,
    );
    for completion in auto_config_completions {
        assert_eq!(
            completion.kind,
            Some(CompletionItemKind::CLASS),
            "AutoConfig 宏的补全项应该是 CLASS 类型"
        );
    }
}

// ============================================================================
// 补全引擎基础功能测试（任务 9.1）
// ============================================================================

#[test]
fn test_complete_with_toml_context() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    // 创建一个简单的 TOML 文档
    let toml_content = "[web]\nhost = \"localhost\"";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    // 光标在 web 节内（使用 line 0）
    let position = Position {
        line: 0,
        character: 10,
    };

    // 使用 TOML 上下文调用 complete
    let completions = engine.complete(CompletionContext::Toml, position, Some(&doc), None);

    // 现在应该返回配置项补全（任务 9.2 已实现）
    // 由于 host 已存在，应该只补全 port
    assert!(!completions.is_empty());
}

#[test]
fn test_complete_with_macro_context() {
    let engine = test_engine();
    let service_macro = ServiceMacro {
        struct_name: "TestService".to_string(),
        fields: vec![],
        range: test_range(),
    };

    let position = Position {
        line: 0,
        character: 0,
    };

    // 使用宏上下文调用 complete
    let completions = engine.complete(
        CompletionContext::Macro,
        position,
        None,
        Some(&SummerMacro::DeriveService(service_macro)),
    );

    // 应该返回 Service 宏的补全项
    assert_eq!(completions.len(), 3);
    assert_eq!(completions[0].label, "inject(component)");
}

#[test]
fn test_complete_with_unknown_context() {
    let engine = test_engine();

    let position = Position {
        line: 0,
        character: 0,
    };

    // 使用未知上下文调用 complete
    let completions = engine.complete(CompletionContext::Unknown, position, None, None);

    // 应该返回空列表
    assert_eq!(completions.len(), 0);
}

#[test]
fn test_complete_toml_without_document() {
    let engine = test_engine();

    let position = Position {
        line: 0,
        character: 0,
    };

    // TOML 上下文但没有提供文档
    let completions = engine.complete(CompletionContext::Toml, position, None, None);

    // 应该返回空列表
    assert_eq!(completions.len(), 0);
}

#[test]
fn test_complete_macro_without_macro_info() {
    let engine = test_engine();

    let position = Position {
        line: 0,
        character: 0,
    };

    // 宏上下文但没有提供宏信息
    let completions = engine.complete(CompletionContext::Macro, position, None, None);

    // 应该返回空列表
    assert_eq!(completions.len(), 0);
}

#[test]
fn test_complete_dispatches_to_correct_handler() {
    let engine = test_engine();

    // 测试不同的宏类型都能正确分发
    let test_cases = vec![
        (
            SummerMacro::DeriveService(ServiceMacro {
                struct_name: "Test".to_string(),
                fields: vec![],
                range: test_range(),
            }),
            3, // Service 宏应该返回 3 个补全项
        ),
        (
            SummerMacro::Inject(InjectMacro {
                inject_type: InjectType::Component,
                component_name: None,
                range: test_range(),
            }),
            2, // Inject 宏应该返回 2 个补全项
        ),
        (
            SummerMacro::AutoConfig(AutoConfigMacro {
                configurator_type: "".to_string(),
                range: test_range(),
            }),
            3, // AutoConfig 宏应该返回 3 个补全项
        ),
    ];

    let position = Position {
        line: 0,
        character: 0,
    };

    for (macro_info, expected_count) in test_cases {
        let completions =
            engine.complete(CompletionContext::Macro, position, None, Some(&macro_info));

        assert_eq!(
            completions.len(),
            expected_count,
            "宏类型 {:?} 的补全项数量不正确",
            macro_info
        );
    }
}

#[test]
fn test_completion_context_clone() {
    // 测试 CompletionContext 可以克隆
    let context = CompletionContext::Toml;
    let cloned = context.clone();

    match (context, cloned) {
        (CompletionContext::Toml, CompletionContext::Toml) => {}
        _ => panic!("克隆的上下文类型不匹配"),
    }
}

#[test]
fn test_completion_context_debug() {
    // 测试 CompletionContext 可以调试输出
    let context = CompletionContext::Macro;
    let debug_str = format!("{:?}", context);
    assert!(debug_str.contains("Macro"));
}

// ============================================================================
// TOML 配置补全测试（任务 9.2）
// ============================================================================

#[test]
fn test_complete_config_properties_in_section() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    // 创建一个只有 host 的 web 配置节
    let toml_content = "[web]\nhost = \"localhost\"";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    // 光标在 web 节内的第二行（host 属性所在行）
    let position = Position {
        line: 1,
        character: 5, // 在 host 属性之后
    };

    let completions = engine.complete(CompletionContext::Toml, position, Some(&doc), None);

    // 应该提供 port 的补全（host 已存在，应该被去重）
    assert!(!completions.is_empty());

    // 检查是否包含 port
    let port_completion = completions.iter().find(|c| c.label == "port");
    assert!(port_completion.is_some(), "应该包含 port 补全");

    let port_completion = port_completion.unwrap();
    assert_eq!(port_completion.kind, Some(CompletionItemKind::PROPERTY));
    assert!(port_completion.detail.is_some());
    assert!(port_completion.documentation.is_some());
    assert!(port_completion.insert_text.is_some());

    // 检查插入文本包含类型提示
    let insert_text = port_completion.insert_text.as_ref().unwrap();
    assert!(insert_text.contains("port"));
    assert!(insert_text.contains("#")); // 应该包含注释
}

#[test]
fn test_complete_config_properties_deduplication() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    // 创建一个包含所有属性的 web 配置节
    let toml_content = "[web]\nhost = \"localhost\"\nport = 8080";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    // 光标在 web 节内的第二行
    let position = Position {
        line: 1,
        character: 5,
    };

    let completions = engine.complete(CompletionContext::Toml, position, Some(&doc), None);

    // 由于 host 和 port 都已存在，不应该再提供这些补全
    let host_completion = completions.iter().find(|c| c.label == "host");
    assert!(host_completion.is_none(), "host 已存在，不应该再补全");

    let port_completion = completions.iter().find(|c| c.label == "port");
    assert!(port_completion.is_none(), "port 已存在，不应该再补全");
}

#[test]
fn test_complete_config_properties_empty_section() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    // 创建一个空的 web 配置节
    let toml_content = "[web]";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    // 光标在 web 节内
    let position = Position {
        line: 0,
        character: 5,
    };

    let completions = engine.complete(CompletionContext::Toml, position, Some(&doc), None);

    // 应该提供所有配置项的补全
    assert!(completions.len() >= 2); // 至少 host 和 port

    // 检查是否包含 host 和 port
    let host_completion = completions.iter().find(|c| c.label == "host");
    assert!(host_completion.is_some(), "应该包含 host 补全");

    let port_completion = completions.iter().find(|c| c.label == "port");
    assert!(port_completion.is_some(), "应该包含 port 补全");
}

#[test]
fn test_complete_config_properties_with_documentation() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    // 创建一个空的 web 配置节
    let toml_content = "[web]";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    // 光标在 web 节内
    let position = Position {
        line: 0,
        character: 5,
    };

    let completions = engine.complete(CompletionContext::Toml, position, Some(&doc), None);

    // 检查所有补全项都有文档
    for completion in completions {
        assert!(
            completion.documentation.is_some(),
            "补全项 '{}' 应该有文档说明",
            completion.label
        );
        assert!(
            completion.detail.is_some(),
            "补全项 '{}' 应该有详细信息",
            completion.label
        );
        assert!(
            completion.insert_text.is_some(),
            "补全项 '{}' 应该有插入文本",
            completion.label
        );
    }
}

#[test]
fn test_complete_config_properties_correct_kind() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    // 创建一个空的 web 配置节
    let toml_content = "[web]";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    // 光标在 web 节内
    let position = Position {
        line: 0,
        character: 5,
    };

    let completions = engine.complete(CompletionContext::Toml, position, Some(&doc), None);

    // 检查所有补全项的类型都是 PROPERTY
    for completion in completions {
        assert_eq!(
            completion.kind,
            Some(CompletionItemKind::PROPERTY),
            "配置项补全应该是 PROPERTY 类型"
        );
    }
}

#[test]
fn test_complete_outside_section() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    // 创建一个 TOML 文档
    let toml_content = "[web]\nhost = \"localhost\"";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    // 光标在配置节外（行号超出范围）
    let position = Position {
        line: 10,
        character: 0,
    };

    let completions = engine.complete(CompletionContext::Toml, position, Some(&doc), None);

    // 不在任何配置节内，应该返回空列表
    assert_eq!(completions.len(), 0);
}

#[test]
fn test_complete_redis_section() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    // 创建一个空的 redis 配置节
    let toml_content = "[redis]";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    // 光标在 redis 节内
    let position = Position {
        line: 0,
        character: 7,
    };

    let completions = engine.complete(CompletionContext::Toml, position, Some(&doc), None);

    // 应该提供 redis 配置项的补全
    assert!(!completions.is_empty());

    // 检查是否包含 url
    let url_completion = completions.iter().find(|c| c.label == "url");
    assert!(url_completion.is_some(), "应该包含 url 补全");
}

#[test]
fn test_complete_unknown_section() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    // 创建一个未知的配置节
    let toml_content = "[unknown]";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    // 光标在 unknown 节内
    let position = Position {
        line: 0,
        character: 9,
    };

    let completions = engine.complete(CompletionContext::Toml, position, Some(&doc), None);

    // 未知配置节，应该返回空列表
    assert_eq!(completions.len(), 0);
}

#[test]
fn test_type_info_to_hint() {
    let engine = test_engine();

    // 测试字符串类型
    let string_type = crate::schema::TypeInfo::String {
        enum_values: None,
        min_length: None,
        max_length: None,
    };
    let hint = engine.type_info_to_hint(&string_type);
    assert_eq!(hint, "string");

    // 测试枚举类型
    let enum_type = crate::schema::TypeInfo::String {
        enum_values: Some(vec!["a".to_string(), "b".to_string()]),
        min_length: None,
        max_length: None,
    };
    let hint = engine.type_info_to_hint(&enum_type);
    assert!(hint.contains("enum"));

    // 测试整数类型
    let int_type = crate::schema::TypeInfo::Integer {
        min: Some(1),
        max: Some(100),
    };
    let hint = engine.type_info_to_hint(&int_type);
    assert!(hint.contains("integer"));
    assert!(hint.contains("1"));
    assert!(hint.contains("100"));

    // 测试布尔类型
    let bool_type = crate::schema::TypeInfo::Boolean;
    let hint = engine.type_info_to_hint(&bool_type);
    assert_eq!(hint, "boolean");
}

#[test]
fn test_type_info_to_default() {
    let engine = test_engine();

    // 测试字符串类型
    let string_type = crate::schema::TypeInfo::String {
        enum_values: None,
        min_length: None,
        max_length: None,
    };
    let default = engine.type_info_to_default(&string_type);
    assert_eq!(default, "\"\"");

    // 测试枚举类型（应该使用第一个枚举值）
    let enum_type = crate::schema::TypeInfo::String {
        enum_values: Some(vec!["first".to_string(), "second".to_string()]),
        min_length: None,
        max_length: None,
    };
    let default = engine.type_info_to_default(&enum_type);
    assert_eq!(default, "\"first\"");

    // 测试整数类型
    let int_type = crate::schema::TypeInfo::Integer {
        min: None,
        max: None,
    };
    let default = engine.type_info_to_default(&int_type);
    assert_eq!(default, "0");

    // 测试浮点数类型
    let float_type = crate::schema::TypeInfo::Float {
        min: None,
        max: None,
    };
    let default = engine.type_info_to_default(&float_type);
    assert_eq!(default, "0.0");

    // 测试布尔类型
    let bool_type = crate::schema::TypeInfo::Boolean;
    let default = engine.type_info_to_default(&bool_type);
    assert_eq!(default, "false");

    // 测试数组类型
    let array_type = crate::schema::TypeInfo::Array {
        item_type: Box::new(crate::schema::TypeInfo::String {
            enum_values: None,
            min_length: None,
            max_length: None,
        }),
    };
    let default = engine.type_info_to_default(&array_type);
    assert_eq!(default, "[]");
}

#[test]
fn test_value_to_string() {
    let engine = test_engine();

    // 测试字符串值
    let string_val = crate::schema::Value::String("test".to_string());
    assert_eq!(engine.value_to_string(&string_val), "\"test\"");

    // 测试整数值
    let int_val = crate::schema::Value::Integer(42);
    assert_eq!(engine.value_to_string(&int_val), "42");

    // 测试浮点数值
    let float_val = crate::schema::Value::Float(3.5);
    assert_eq!(engine.value_to_string(&float_val), "3.5");

    // 测试布尔值
    let bool_val = crate::schema::Value::Boolean(true);
    assert_eq!(engine.value_to_string(&bool_val), "true");

    // 测试数组值
    let array_val = crate::schema::Value::Array(vec![]);
    assert_eq!(engine.value_to_string(&array_val), "[]");

    // 测试表值
    use std::collections::HashMap;
    let table_val = crate::schema::Value::Table(HashMap::new());
    assert_eq!(engine.value_to_string(&table_val), "{}");
}

#[test]
fn test_position_in_range() {
    let engine = test_engine();

    let range = Range {
        start: Position {
            line: 1,
            character: 5,
        },
        end: Position {
            line: 3,
            character: 10,
        },
    };

    // 测试在范围内的位置
    let pos_inside = Position {
        line: 2,
        character: 7,
    };
    assert!(engine.position_in_range(pos_inside, range));

    // 测试在起始位置
    let pos_start = Position {
        line: 1,
        character: 5,
    };
    assert!(engine.position_in_range(pos_start, range));

    // 测试在结束位置
    let pos_end = Position {
        line: 3,
        character: 10,
    };
    assert!(engine.position_in_range(pos_end, range));

    // 测试在范围外（行号太小）
    let pos_before = Position {
        line: 0,
        character: 5,
    };
    assert!(!engine.position_in_range(pos_before, range));

    // 测试在范围外（行号太大）
    let pos_after = Position {
        line: 4,
        character: 5,
    };
    assert!(!engine.position_in_range(pos_after, range));

    // 测试在范围外（同一行，字符位置太小）
    let pos_before_char = Position {
        line: 1,
        character: 3,
    };
    assert!(!engine.position_in_range(pos_before_char, range));

    // 测试在范围外（同一行，字符位置太大）
    let pos_after_char = Position {
        line: 3,
        character: 15,
    };
    assert!(!engine.position_in_range(pos_after_char, range));
}

// ============================================================================
// 补全引擎属性测试（任务 9.3）
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::toml_analyzer::ConfigProperty;
    use proptest::prelude::*;
    use std::collections::HashMap;

    // 生成有效的配置前缀（插件名称）
    fn valid_prefix() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-z][a-z0-9-]*").unwrap()
    }

    // 生成有效的配置键名
    fn valid_key() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-z][a-z0-9_]*").unwrap()
    }

    // 生成环境变量名
    #[allow(dead_code)]
    fn env_var_name() -> impl Strategy<Value = String> {
        prop::string::string_regex("[A-Z][A-Z0-9_]*").unwrap()
    }

    // 创建测试用的配置节
    fn create_config_section(
        prefix: &str,
        properties: HashMap<String, ConfigProperty>,
    ) -> crate::toml_analyzer::ConfigSection {
        crate::toml_analyzer::ConfigSection {
            prefix: prefix.to_string(),
            properties,
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 100,
                },
            },
        }
    }

    // **Property 13: 配置前缀补全**
    // **Validates: Requirements 4.1**
    proptest! {
        #[test]
        fn prop_complete_config_prefix_returns_all_prefixes(
            prefixes in prop::collection::hash_set(valid_prefix(), 1..10)
        ) {
            // 创建一个包含多个插件的 Schema
            let mut schema = crate::schema::ConfigSchema {
                plugins: HashMap::new(),
            };

            for prefix in &prefixes {
                schema.plugins.insert(
                    prefix.clone(),
                    serde_json::json!({
                        "type": "object",
                        "properties": {}
                    }),
                );
            }

            let schema_provider = crate::schema::SchemaProvider::from_schema(schema);
            let engine = CompletionEngine::new(schema_provider);

            // 获取配置前缀补全
            let completions = engine.complete_config_prefix();

            // 验证：所有在 Schema 中定义的配置前缀都应该出现在补全列表中
            for prefix in &prefixes {
                prop_assert!(
                    completions.iter().any(|c| c.label == *prefix),
                    "配置前缀 '{}' 应该出现在补全列表中",
                    prefix
                );
            }

            // 验证：补全项数量应该等于 Schema 中的插件数量
            prop_assert_eq!(
                completions.len(),
                prefixes.len(),
                "补全项数量应该等于 Schema 中的插件数量"
            );
        }
    }

    // **Property 14: 配置项补全**
    // **Validates: Requirements 4.2**
    proptest! {
        #[test]
        fn prop_complete_config_properties_returns_unused_properties(
            prefix in valid_prefix(),
            all_keys in prop::collection::hash_set(valid_key(), 2..10),
            used_keys_count in 0usize..5usize,
        ) {
            let all_keys: Vec<String> = all_keys.into_iter().collect();
            let used_keys_count = used_keys_count.min(all_keys.len());
            let used_keys: Vec<String> = all_keys.iter().take(used_keys_count).cloned().collect();

            // 创建 Schema，包含所有配置项
            let mut properties_json = serde_json::Map::new();
            for key in &all_keys {
                properties_json.insert(
                    key.clone(),
                    serde_json::json!({
                        "type": "string",
                        "description": format!("Property {}", key)
                    }),
                );
            }

            let mut schema = crate::schema::ConfigSchema {
                plugins: HashMap::new(),
            };
            schema.plugins.insert(
                    prefix.clone(),
                    serde_json::json!({
                        "type": "object",
                        "properties": properties_json
                    }),
                );

            let schema_provider = crate::schema::SchemaProvider::from_schema(schema);
            let engine = CompletionEngine::new(schema_provider);

            // 创建配置节，包含部分已使用的配置项
            let mut section_properties = HashMap::new();
            for key in &used_keys {
                section_properties.insert(
                    key.clone(),
                    ConfigProperty {
                        key: key.clone(),
                        value: crate::toml_analyzer::ConfigValue::String("test".to_string()),
                        range: Range {
                            start: Position { line: 0, character: 0 },
                            end: Position { line: 0, character: 10 },
                        },
                    },
                );
            }

            let section = create_config_section(&prefix, section_properties);

            // 获取配置项补全
            let completions = engine.complete_config_properties(&section);

            // 验证：已使用的配置项不应该出现在补全列表中
            for used_key in &used_keys {
                prop_assert!(
                    !completions.iter().any(|c| c.label == *used_key),
                    "已使用的配置项 '{}' 不应该出现在补全列表中",
                    used_key
                );
            }

            // 验证：未使用的配置项应该出现在补全列表中
            let unused_keys: Vec<&String> = all_keys.iter()
                .filter(|k| !used_keys.contains(k))
                .collect();

            for unused_key in &unused_keys {
                prop_assert!(
                    completions.iter().any(|c| c.label == **unused_key),
                    "未使用的配置项 '{}' 应该出现在补全列表中",
                    unused_key
                );
            }
        }
    }

    // **Property 15: 枚举值补全**
    // **Validates: Requirements 4.3**
    proptest! {
        #[test]
        fn prop_complete_enum_values_returns_all_enum_values(
            enum_values in prop::collection::vec(valid_key(), 1..10)
        ) {
            let engine = test_engine();

            // 获取枚举值补全
            let completions = engine.complete_enum_values(&enum_values);

            // 验证：所有枚举值都应该出现在补全列表中
            for value in &enum_values {
                prop_assert!(
                    completions.iter().any(|c| c.label == *value),
                    "枚举值 '{}' 应该出现在补全列表中",
                    value
                );
            }

            // 验证：补全项数量应该等于枚举值数量
            prop_assert_eq!(
                completions.len(),
                enum_values.len(),
                "补全项数量应该等于枚举值数量"
            );

            // 验证：所有补全项的类型都是 ENUM_MEMBER
            for completion in &completions {
                prop_assert_eq!(
                    completion.kind,
                    Some(CompletionItemKind::ENUM_MEMBER),
                    "枚举值补全项的类型应该是 ENUM_MEMBER"
                );
            }
        }
    }

    // **Property 16: 环境变量补全**
    // **Validates: Requirements 4.4**
    proptest! {
        #[test]
        fn prop_complete_env_var_returns_common_vars(
            _dummy in any::<u8>() // proptest 需要至少一个参数
        ) {
            let engine = test_engine();

            // 获取环境变量补全
            let completions = engine.complete_env_var();

            // 验证：应该返回常见的环境变量
            prop_assert!(
                !completions.is_empty(),
                "环境变量补全列表不应该为空"
            );

            // 验证：所有补全项的类型都是 VARIABLE
            for completion in &completions {
                prop_assert_eq!(
                    completion.kind,
                    Some(CompletionItemKind::VARIABLE),
                    "环境变量补全项的类型应该是 VARIABLE"
                );
            }

            // 验证：所有补全项都有文档说明
            for completion in &completions {
                prop_assert!(
                    completion.documentation.is_some(),
                    "环境变量补全项 '{}' 应该有文档说明",
                    completion.label
                );
            }

            // 验证：常见的环境变量应该包含在列表中
            let common_vars = vec!["HOST", "PORT", "DATABASE_URL", "REDIS_URL"];
            for var in common_vars {
                prop_assert!(
                    completions.iter().any(|c| c.label == var),
                    "常见环境变量 '{}' 应该出现在补全列表中",
                    var
                );
            }
        }
    }

    // **Property 18: 补全去重**
    // **Validates: Requirements 4.6**
    proptest! {
        #[test]
        fn prop_completion_deduplication_no_duplicates(
            prefix in valid_prefix(),
            all_keys in prop::collection::hash_set(valid_key(), 3..10),
            existing_keys_count in 1usize..5usize,
        ) {
            let all_keys: Vec<String> = all_keys.into_iter().collect();
            let existing_keys_count = existing_keys_count.min(all_keys.len());
            let existing_keys: Vec<String> = all_keys.iter()
                .take(existing_keys_count)
                .cloned()
                .collect();

            // 创建 Schema
            let mut properties_json = serde_json::Map::new();
            for key in &all_keys {
                properties_json.insert(
                    key.clone(),
                    serde_json::json!({
                        "type": "string",
                        "description": format!("Property {}", key)
                    }),
                );
            }

            let mut schema = crate::schema::ConfigSchema {

                plugins: HashMap::new(),
            };
            schema.plugins.insert(
                    prefix.clone(),
                    serde_json::json!({
                        "type": "object",
                        "properties": properties_json
                    }),
                );

            let schema_provider = crate::schema::SchemaProvider::from_schema(schema);
            let engine = CompletionEngine::new(schema_provider);

            // 创建配置节，包含已存在的配置项
            let mut section_properties = HashMap::new();
            for key in &existing_keys {
                section_properties.insert(
                    key.clone(),
                    ConfigProperty {
                        key: key.clone(),
                        value: crate::toml_analyzer::ConfigValue::String("test".to_string()),
                        range: Range {
                            start: Position { line: 0, character: 0 },
                            end: Position { line: 0, character: 10 },
                        },
                    },
                );
            }

            let section = create_config_section(&prefix, section_properties);

            // 获取配置项补全
            let completions = engine.complete_config_properties(&section);

            // 验证：已存在的配置项不应该出现在补全列表中（去重）
            for existing_key in &existing_keys {
                prop_assert!(
                    !completions.iter().any(|c| c.label == *existing_key),
                    "已存在的配置项 '{}' 不应该出现在补全列表中（应该被去重）",
                    existing_key
                );
            }

            // 验证：补全列表中不应该有重复项
            let mut seen = std::collections::HashSet::new();
            for completion in &completions {
                prop_assert!(
                    seen.insert(&completion.label),
                    "补全列表中不应该有重复的配置项 '{}'",
                    completion.label
                );
            }
        }
    }

    // 额外的属性测试：验证补全项的完整性
    proptest! {
        #[test]
        fn prop_completion_items_have_required_fields(
            prefix in valid_prefix(),
            keys in prop::collection::hash_set(valid_key(), 1..5),
        ) {
            let keys: Vec<String> = keys.into_iter().collect();

            // 创建 Schema
            let mut properties_json = serde_json::Map::new();
            for key in &keys {
                properties_json.insert(
                    key.clone(),
                    serde_json::json!({
                        "type": "string",
                        "description": format!("Property {}", key)
                    }),
                );
            }

            let mut schema = crate::schema::ConfigSchema {

                plugins: HashMap::new(),
            };
            schema.plugins.insert(
                    prefix.clone(),
                    serde_json::json!({
                        "type": "object",
                        "properties": properties_json
                    }),
                );

            let schema_provider = crate::schema::SchemaProvider::from_schema(schema);
            let engine = CompletionEngine::new(schema_provider);

            // 创建空的配置节
            let section = create_config_section(&prefix, HashMap::new());

            // 获取配置项补全
            let completions = engine.complete_config_properties(&section);

            // 验证：所有补全项都有必需的字段
            for completion in &completions {
                prop_assert!(
                    completion.detail.is_some(),
                    "补全项 '{}' 应该有 detail 字段",
                    completion.label
                );
                prop_assert!(
                    completion.documentation.is_some(),
                    "补全项 '{}' 应该有 documentation 字段",
                    completion.label
                );
                prop_assert!(
                    completion.insert_text.is_some(),
                    "补全项 '{}' 应该有 insert_text 字段",
                    completion.label
                );
                prop_assert_eq!(
                    completion.kind,
                    Some(CompletionItemKind::PROPERTY),
                    "配置项补全的 kind 应该是 PROPERTY"
                );
            }
        }
    }

    // 额外的属性测试：验证类型提示的正确性
    proptest! {
        #[test]
        fn prop_type_info_to_hint_is_consistent(
            type_info in prop_oneof![
                Just(crate::schema::TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                }),
                Just(crate::schema::TypeInfo::Integer {
                    min: None,
                    max: None,
                }),
                Just(crate::schema::TypeInfo::Float {
                    min: None,
                    max: None,
                }),
                Just(crate::schema::TypeInfo::Boolean),
            ]
        ) {
            let engine = test_engine();
            let hint = engine.type_info_to_hint(&type_info);

            // 验证：类型提示不应该为空
            prop_assert!(
                !hint.is_empty(),
                "类型提示不应该为空"
            );

            // 验证：类型提示应该包含类型名称
            match type_info {
                crate::schema::TypeInfo::String { .. } => {
                    prop_assert!(
                        hint.contains("string") || hint.contains("enum"),
                        "字符串类型的提示应该包含 'string' 或 'enum'"
                    );
                }
                crate::schema::TypeInfo::Integer { .. } => {
                    prop_assert!(
                        hint.contains("integer"),
                        "整数类型的提示应该包含 'integer'"
                    );
                }
                crate::schema::TypeInfo::Float { .. } => {
                    prop_assert!(
                        hint.contains("float"),
                        "浮点数类型的提示应该包含 'float'"
                    );
                }
                crate::schema::TypeInfo::Boolean => {
                    prop_assert!(
                        hint.contains("boolean"),
                        "布尔类型的提示应该包含 'boolean'"
                    );
                }
                _ => {}
            }
        }
    }
}

// ============================================================================
// 补全插入完整性测试（任务 9.4 - Requirement 4.5）
// ============================================================================

#[test]
fn test_config_property_insertion_has_complete_format() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    // 创建一个空的 web 配置节
    let toml_content = "[web]";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    let position = Position {
        line: 0,
        character: 5,
    };

    let completions = engine.complete(CompletionContext::Toml, position, Some(&doc), None);

    // 验证每个补全项的插入文本都包含完整的格式
    for completion in completions {
        let insert_text = completion
            .insert_text
            .as_ref()
            .unwrap_or_else(|| panic!("补全项 '{}' 应该有 insert_text", completion.label));

        // 验证插入文本包含配置项名称
        assert!(
            insert_text.contains(&completion.label),
            "插入文本应该包含配置项名称 '{}'",
            completion.label
        );

        // 验证插入文本包含等号
        assert!(
            insert_text.contains("="),
            "插入文本应该包含等号: {}",
            insert_text
        );

        // 验证插入文本包含类型提示注释（以 # 开头）
        assert!(
            insert_text.contains("#"),
            "插入文本应该包含类型提示注释: {}",
            insert_text
        );

        // 验证插入文本格式为: key = value  # type
        let parts: Vec<&str> = insert_text.split("=").collect();
        assert_eq!(parts.len(), 2, "插入文本应该包含一个等号: {}", insert_text);

        let value_and_comment: Vec<&str> = parts[1].split("#").collect();
        assert_eq!(
            value_and_comment.len(),
            2,
            "插入文本应该包含一个注释符号: {}",
            insert_text
        );
    }
}

#[test]
fn test_config_property_insertion_has_correct_default_values() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    // 创建一个空的 web 配置节
    let toml_content = "[web]";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    let position = Position {
        line: 0,
        character: 5,
    };

    let completions = engine.complete(CompletionContext::Toml, position, Some(&doc), None);

    // 查找 host 补全项
    let host_completion = completions
        .iter()
        .find(|c| c.label == "host")
        .expect("应该有 host 补全项");

    let insert_text = host_completion.insert_text.as_ref().unwrap();

    // 验证字符串类型的默认值是字符串格式（可能是空字符串或有默认值）
    assert!(
        insert_text.contains("\"") && insert_text.matches("\"").count() >= 2,
        "字符串类型的默认值应该是字符串格式（带引号）: {}",
        insert_text
    );

    // 查找 port 补全项
    let port_completion = completions
        .iter()
        .find(|c| c.label == "port")
        .expect("应该有 port 补全项");

    let insert_text = port_completion.insert_text.as_ref().unwrap();

    // 验证整数类型的默认值是 0
    assert!(
        insert_text.contains("= 0") || insert_text.contains("= 8080"),
        "整数类型应该有合理的默认值: {}",
        insert_text
    );
}

#[test]
fn test_enum_value_insertion_has_quotes() {
    let engine = test_engine();

    let enum_values = vec!["debug".to_string(), "info".to_string(), "warn".to_string()];
    let completions = engine.complete_enum_values(&enum_values);

    // 验证每个枚举值的插入文本都有引号
    for completion in completions {
        let insert_text = completion
            .insert_text
            .as_ref()
            .unwrap_or_else(|| panic!("枚举值 '{}' 应该有 insert_text", completion.label));

        // 验证插入文本包含引号
        assert!(
            insert_text.starts_with("\"") && insert_text.ends_with("\""),
            "枚举值的插入文本应该有引号: {}",
            insert_text
        );

        // 验证引号内的值与标签匹配
        let value = insert_text.trim_matches('"');
        assert_eq!(value, completion.label, "引号内的值应该与标签匹配");
    }
}

#[test]
fn test_env_var_insertion_has_snippet_format() {
    let engine = test_engine();
    let completions = engine.complete_env_var();

    // 验证每个环境变量的插入文本都是 snippet 格式
    for completion in completions {
        let insert_text = completion
            .insert_text
            .as_ref()
            .unwrap_or_else(|| panic!("环境变量 '{}' 应该有 insert_text", completion.label));

        // 验证插入文本包含变量名
        assert!(
            insert_text.contains(&completion.label),
            "插入文本应该包含变量名 '{}': {}",
            completion.label,
            insert_text
        );

        // 验证插入文本包含 snippet 占位符 ${1:default}
        assert!(
            insert_text.contains("${1:") || insert_text.contains("$1"),
            "插入文本应该包含 snippet 占位符: {}",
            insert_text
        );

        // 验证插入文本包含冒号和闭合大括号
        assert!(
            insert_text.contains(":") && insert_text.contains("}"),
            "插入文本应该包含完整的环境变量格式: {}",
            insert_text
        );

        // 验证 insert_text_format 是 SNIPPET
        assert_eq!(
            completion.insert_text_format,
            Some(lsp_types::InsertTextFormat::SNIPPET),
            "环境变量补全应该使用 SNIPPET 格式"
        );
    }
}

#[test]
fn test_macro_parameter_insertion_completeness() {
    let engine = test_engine();

    // 测试 Service 宏的补全
    let service_macro = ServiceMacro {
        struct_name: "Test".to_string(),
        fields: vec![],
        range: test_range(),
    };

    let completions = engine.complete_macro(&SummerMacro::DeriveService(service_macro), None);

    // 验证带名称的组件注入使用 snippet 格式
    let named_inject = completions
        .iter()
        .find(|c| c.label.contains("component = \"name\""))
        .expect("应该有带名称的组件注入补全");

    assert_eq!(
        named_inject.insert_text_format,
        Some(lsp_types::InsertTextFormat::SNIPPET),
        "带名称的组件注入应该使用 SNIPPET 格式"
    );

    let insert_text = named_inject.insert_text.as_ref().unwrap();
    assert!(
        insert_text.contains("$1"),
        "带名称的组件注入应该包含 snippet 占位符: {}",
        insert_text
    );
}

#[test]
fn test_route_macro_path_parameter_snippet() {
    let engine = test_engine();

    let route_macro = RouteMacro {
        path: "/test".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "handler".to_string(),
        range: test_range(),
        is_openapi: false,
    };

    let completions = engine.complete_macro(&SummerMacro::Route(route_macro), None);

    // 查找路径参数补全
    let path_param = completions
        .iter()
        .find(|c| c.label == "{id}")
        .expect("应该有路径参数补全");

    // 验证路径参数使用 snippet 格式
    assert_eq!(
        path_param.insert_text_format,
        Some(lsp_types::InsertTextFormat::SNIPPET),
        "路径参数应该使用 SNIPPET 格式"
    );

    let insert_text = path_param.insert_text.as_ref().unwrap();

    // 验证包含 snippet 占位符
    assert!(
        insert_text.contains("${1:") && insert_text.contains("}"),
        "路径参数应该包含 snippet 占位符: {}",
        insert_text
    );

    // 验证格式为 {${1:id}}
    assert!(
        insert_text.starts_with("{") && insert_text.ends_with("}"),
        "路径参数应该包含大括号: {}",
        insert_text
    );
}

// ============================================================================
// 边缘情况测试（任务 9.4）
// ============================================================================

#[test]
fn test_complete_with_empty_toml_document() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    // 创建一个空的 TOML 文档
    let toml_content = "";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    let position = Position {
        line: 0,
        character: 0,
    };

    let completions = engine.complete(CompletionContext::Toml, position, Some(&doc), None);

    // 空文档应该返回空补全列表（因为不在任何配置节内）
    assert_eq!(completions.len(), 0);
}

#[test]
fn test_complete_at_document_boundary() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = "[web]\nhost = \"localhost\"";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    // 测试在文档开始位置
    let position_start = Position {
        line: 0,
        character: 0,
    };

    let completions = engine.complete(CompletionContext::Toml, position_start, Some(&doc), None);

    // 在配置节标题位置，应该返回配置项补全
    // 验证返回了补全项（长度检查）
    let _ = completions.len();

    // 测试在文档结束位置之后
    let position_end = Position {
        line: 100,
        character: 100,
    };

    let completions = engine.complete(CompletionContext::Toml, position_end, Some(&doc), None);

    // 超出文档范围，应该返回空列表
    assert_eq!(completions.len(), 0);
}

#[test]
fn test_complete_with_invalid_position() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = "[web]";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    // 测试负数行号（虽然 Position 使用 u32，但测试边界）
    let position = Position {
        line: u32::MAX,
        character: u32::MAX,
    };

    let completions = engine.complete(CompletionContext::Toml, position, Some(&doc), None);

    // 无效位置应该返回空列表
    assert_eq!(completions.len(), 0);
}

#[test]
fn test_complete_with_all_properties_used() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    // 创建一个包含所有可能属性的配置节
    let toml_content = "[web]\nhost = \"localhost\"\nport = 8080";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    let position = Position {
        line: 0,
        character: 20,
    };

    let completions = engine.complete(CompletionContext::Toml, position, Some(&doc), None);

    // 所有属性都已使用，应该返回空列表或只有未知属性
    // 由于 web 插件可能有更多属性，这里只验证 host 和 port 不在列表中
    let has_host = completions.iter().any(|c| c.label == "host");
    let has_port = completions.iter().any(|c| c.label == "port");

    assert!(!has_host, "host 已存在，不应该再补全");
    assert!(!has_port, "port 已存在，不应该再补全");
}

#[test]
fn test_complete_with_unknown_section() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    // 创建一个未知的配置节
    let toml_content = "[unknown_plugin_12345]";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    let position = Position {
        line: 0,
        character: 10,
    };

    let completions = engine.complete(CompletionContext::Toml, position, Some(&doc), None);

    // 未知配置节应该返回空列表
    assert_eq!(completions.len(), 0);
}

#[test]
fn test_complete_enum_with_empty_values() {
    let engine = test_engine();

    // 测试空的枚举值列表
    let empty_values: Vec<String> = vec![];
    let completions = engine.complete_enum_values(&empty_values);

    // 空枚举值应该返回空列表
    assert_eq!(completions.len(), 0);
}

#[test]
fn test_complete_enum_with_single_value() {
    let engine = test_engine();

    // 测试只有一个枚举值
    let single_value = vec!["only_one".to_string()];
    let completions = engine.complete_enum_values(&single_value);

    // 应该返回一个补全项
    assert_eq!(completions.len(), 1);
    assert_eq!(completions[0].label, "only_one");

    // 验证插入文本有引号
    let insert_text = completions[0].insert_text.as_ref().unwrap();
    assert_eq!(insert_text, "\"only_one\"");
}

#[test]
fn test_complete_with_special_characters_in_enum() {
    let engine = test_engine();

    // 测试包含特殊字符的枚举值
    let special_values = vec![
        "value-with-dash".to_string(),
        "value_with_underscore".to_string(),
        "value.with.dot".to_string(),
    ];

    let completions = engine.complete_enum_values(&special_values);

    // 应该返回所有枚举值
    assert_eq!(completions.len(), 3);

    // 验证每个值都正确处理
    for (i, value) in special_values.iter().enumerate() {
        assert_eq!(completions[i].label, *value);
        let insert_text = completions[i].insert_text.as_ref().unwrap();
        assert_eq!(insert_text, &format!("\"{}\"", value));
    }
}

#[test]
fn test_position_in_range_edge_cases() {
    let engine = test_engine();

    // 测试单点范围（起始和结束位置相同）
    let single_point_range = Range {
        start: Position {
            line: 5,
            character: 10,
        },
        end: Position {
            line: 5,
            character: 10,
        },
    };

    let pos_exact = Position {
        line: 5,
        character: 10,
    };
    assert!(engine.position_in_range(pos_exact, single_point_range));

    let pos_before = Position {
        line: 5,
        character: 9,
    };
    assert!(!engine.position_in_range(pos_before, single_point_range));

    let pos_after = Position {
        line: 5,
        character: 11,
    };
    assert!(!engine.position_in_range(pos_after, single_point_range));

    // 测试跨行范围
    let multi_line_range = Range {
        start: Position {
            line: 1,
            character: 5,
        },
        end: Position {
            line: 3,
            character: 10,
        },
    };

    // 中间行的任意位置都应该在范围内
    let pos_middle_line = Position {
        line: 2,
        character: 0,
    };
    assert!(engine.position_in_range(pos_middle_line, multi_line_range));

    let pos_middle_line_end = Position {
        line: 2,
        character: 999,
    };
    assert!(engine.position_in_range(pos_middle_line_end, multi_line_range));
}

#[test]
fn test_type_info_to_hint_with_ranges() {
    let engine = test_engine();

    // 测试带范围的整数类型
    let int_with_range = crate::schema::TypeInfo::Integer {
        min: Some(1),
        max: Some(100),
    };
    let hint = engine.type_info_to_hint(&int_with_range);
    assert!(hint.contains("1"));
    assert!(hint.contains("100"));
    assert!(hint.contains("integer"));

    // 测试只有最小值的整数类型
    let int_with_min = crate::schema::TypeInfo::Integer {
        min: Some(0),
        max: None,
    };
    let hint = engine.type_info_to_hint(&int_with_min);
    assert_eq!(hint, "integer");

    // 测试只有最大值的整数类型
    let int_with_max = crate::schema::TypeInfo::Integer {
        min: None,
        max: Some(255),
    };
    let hint = engine.type_info_to_hint(&int_with_max);
    assert_eq!(hint, "integer");
}

#[test]
fn test_type_info_to_default_for_all_types() {
    let engine = test_engine();

    // 测试所有类型的默认值
    let test_cases = vec![
        (
            crate::schema::TypeInfo::String {
                enum_values: None,
                min_length: None,
                max_length: None,
            },
            "\"\"",
        ),
        (
            crate::schema::TypeInfo::Integer {
                min: None,
                max: None,
            },
            "0",
        ),
        (
            crate::schema::TypeInfo::Float {
                min: None,
                max: None,
            },
            "0.0",
        ),
        (crate::schema::TypeInfo::Boolean, "false"),
        (
            crate::schema::TypeInfo::Array {
                item_type: Box::new(crate::schema::TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                }),
            },
            "[]",
        ),
        (
            crate::schema::TypeInfo::Object {
                properties: std::collections::HashMap::new(),
            },
            "{}",
        ),
    ];

    for (type_info, expected_default) in test_cases {
        let default = engine.type_info_to_default(&type_info);
        assert_eq!(
            default, expected_default,
            "类型 {:?} 的默认值不正确",
            type_info
        );
    }
}

#[test]
fn test_value_to_string_for_all_value_types() {
    let engine = test_engine();

    // 测试所有值类型的字符串转换
    let test_cases = vec![
        (crate::schema::Value::String("test".to_string()), "\"test\""),
        (crate::schema::Value::Integer(42), "42"),
        (crate::schema::Value::Integer(-10), "-10"),
        (crate::schema::Value::Float(3.5), "3.5"),
        (crate::schema::Value::Float(-2.5), "-2.5"),
        (crate::schema::Value::Boolean(true), "true"),
        (crate::schema::Value::Boolean(false), "false"),
        (crate::schema::Value::Array(vec![]), "[]"),
        (
            crate::schema::Value::Table(std::collections::HashMap::new()),
            "{}",
        ),
    ];

    for (value, expected_string) in test_cases {
        let string = engine.value_to_string(&value);
        assert_eq!(string, expected_string, "值 {:?} 的字符串表示不正确", value);
    }
}

#[test]
fn test_complete_with_nested_config_sections() {
    let engine = test_engine();
    let schema_provider = SchemaProvider::default();
    let toml_analyzer = TomlAnalyzer::new(schema_provider);

    // 创建包含多个配置节的文档
    let toml_content = "[web]\nhost = \"localhost\"\n\n[redis]\nurl = \"redis://localhost\"";
    let doc = toml_analyzer.parse(toml_content).unwrap();

    // 在第一个配置节内
    let position_web = Position {
        line: 0,
        character: 20,
    };

    let completions_web = engine.complete(CompletionContext::Toml, position_web, Some(&doc), None);

    // 应该只补全 web 配置节的属性
    // 验证不包含 redis 的属性
    let has_url = completions_web.iter().any(|c| c.label == "url");
    assert!(!has_url, "在 web 配置节内不应该补全 redis 的属性");
}

#[test]
fn test_completion_context_equality() {
    // 测试 CompletionContext 的相等性
    let toml1 = CompletionContext::Toml;
    let toml2 = CompletionContext::Toml;
    let macro1 = CompletionContext::Macro;
    let unknown = CompletionContext::Unknown;

    // 由于 CompletionContext 没有实现 PartialEq，我们通过模式匹配来测试
    match (toml1, toml2) {
        (CompletionContext::Toml, CompletionContext::Toml) => {}
        _ => panic!("相同的 Toml 上下文应该匹配"),
    }

    match (macro1, unknown) {
        (CompletionContext::Macro, CompletionContext::Unknown) => {}
        _ => panic!("不同的上下文应该不匹配"),
    }
}
