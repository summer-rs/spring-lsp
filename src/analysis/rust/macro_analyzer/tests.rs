//! 宏分析器测试

use super::*;
use lsp_types::{Position, Range};

/// 创建测试用的 Range
fn test_range() -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 10,
        },
    }
}

#[test]
fn test_http_method_from_str() {
    assert_eq!(HttpMethod::parse_method("GET"), Some(HttpMethod::Get));
    assert_eq!(HttpMethod::parse_method("get"), Some(HttpMethod::Get));
    assert_eq!(HttpMethod::parse_method("POST"), Some(HttpMethod::Post));
    assert_eq!(HttpMethod::parse_method("post"), Some(HttpMethod::Post));
    assert_eq!(HttpMethod::parse_method("PUT"), Some(HttpMethod::Put));
    assert_eq!(HttpMethod::parse_method("DELETE"), Some(HttpMethod::Delete));
    assert_eq!(HttpMethod::parse_method("PATCH"), Some(HttpMethod::Patch));
    assert_eq!(HttpMethod::parse_method("HEAD"), Some(HttpMethod::Head));
    assert_eq!(
        HttpMethod::parse_method("OPTIONS"),
        Some(HttpMethod::Options)
    );
    assert_eq!(
        HttpMethod::parse_method("CONNECT"),
        Some(HttpMethod::Connect)
    );
    assert_eq!(HttpMethod::parse_method("TRACE"), Some(HttpMethod::Trace));
    assert_eq!(HttpMethod::parse_method("INVALID"), None);
}

#[test]
fn test_http_method_as_str() {
    assert_eq!(HttpMethod::Get.as_str(), "GET");
    assert_eq!(HttpMethod::Post.as_str(), "POST");
    assert_eq!(HttpMethod::Put.as_str(), "PUT");
    assert_eq!(HttpMethod::Delete.as_str(), "DELETE");
    assert_eq!(HttpMethod::Patch.as_str(), "PATCH");
    assert_eq!(HttpMethod::Head.as_str(), "HEAD");
    assert_eq!(HttpMethod::Options.as_str(), "OPTIONS");
    assert_eq!(HttpMethod::Connect.as_str(), "CONNECT");
    assert_eq!(HttpMethod::Trace.as_str(), "TRACE");
}

#[test]
fn test_inject_type() {
    let component = InjectType::Component;
    let config = InjectType::Config;

    assert_eq!(component, InjectType::Component);
    assert_eq!(config, InjectType::Config);
    assert_ne!(component, config);
}

#[test]
fn test_inject_macro_creation() {
    let inject = InjectMacro {
        inject_type: InjectType::Component,
        component_name: Some("my_component".to_string()),
        range: test_range(),
    };

    assert_eq!(inject.inject_type, InjectType::Component);
    assert_eq!(inject.component_name, Some("my_component".to_string()));
}

#[test]
fn test_service_macro_creation() {
    let service = ServiceMacro {
        struct_name: "MyService".to_string(),
        fields: vec![Field {
            name: "db".to_string(),
            type_name: "ConnectPool".to_string(),
            inject: Some(InjectMacro {
                inject_type: InjectType::Component,
                component_name: None,
                range: test_range(),
            }),
        }],
        scope: ServiceScope::Singleton,
        range: test_range(),
    };

    assert_eq!(service.struct_name, "MyService");
    assert_eq!(service.fields.len(), 1);
    assert_eq!(service.fields[0].name, "db");
    assert_eq!(service.fields[0].type_name, "ConnectPool");
    assert!(service.fields[0].inject.is_some());
}

#[test]
fn test_route_macro_creation() {
    let route = RouteMacro {
        path: "/users/{id}".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec!["AuthMiddleware".to_string()],
        handler_name: "get_user".to_string(),
        is_openapi: false,
        range: test_range(),
    };

    assert_eq!(route.path, "/users/{id}");
    assert_eq!(route.methods.len(), 1);
    assert_eq!(route.methods[0], HttpMethod::Get);
    assert_eq!(route.middlewares.len(), 1);
    assert_eq!(route.handler_name, "get_user");
}

#[test]
fn test_auto_config_macro_creation() {
    let auto_config = AutoConfigMacro {
        configurator_type: "WebConfigurator".to_string(),
        range: test_range(),
    };

    assert_eq!(auto_config.configurator_type, "WebConfigurator");
}

#[test]
fn test_job_macro_cron() {
    let job = JobMacro::Cron {
        expression: "0 0 * * * *".to_string(),
        range: test_range(),
    };

    match job {
        JobMacro::Cron { expression, .. } => {
            assert_eq!(expression, "0 0 * * * *");
        }
        _ => panic!("Expected Cron variant"),
    }
}

#[test]
fn test_job_macro_fix_delay() {
    let job = JobMacro::FixDelay {
        seconds: 5,
        range: test_range(),
    };

    match job {
        JobMacro::FixDelay { seconds, .. } => {
            assert_eq!(seconds, 5);
        }
        _ => panic!("Expected FixDelay variant"),
    }
}

#[test]
fn test_job_macro_fix_rate() {
    let job = JobMacro::FixRate {
        seconds: 10,
        range: test_range(),
    };

    match job {
        JobMacro::FixRate { seconds, .. } => {
            assert_eq!(seconds, 10);
        }
        _ => panic!("Expected FixRate variant"),
    }
}

#[test]
fn test_summer_macro_variants() {
    let service = SummerMacro::DeriveService(ServiceMacro {
        struct_name: "MyService".to_string(),
        fields: vec![],
        scope: ServiceScope::Singleton,
        range: test_range(),
    });

    let inject = SummerMacro::Inject(InjectMacro {
        inject_type: InjectType::Component,
        component_name: None,
        range: test_range(),
    });

    let route = SummerMacro::Route(RouteMacro {
        path: "/test".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "test_handler".to_string(),
        range: test_range(),
        is_openapi: false,
    });

    let auto_config = SummerMacro::AutoConfig(AutoConfigMacro {
        configurator_type: "WebConfigurator".to_string(),
        range: test_range(),
    });

    let job = SummerMacro::Job(JobMacro::Cron {
        expression: "0 0 * * * *".to_string(),
        range: test_range(),
    });

    // 验证所有变体都可以创建
    match service {
        SummerMacro::DeriveService(_) => {}
        _ => panic!("Expected DeriveService variant"),
    }

    match inject {
        SummerMacro::Inject(_) => {}
        _ => panic!("Expected Inject variant"),
    }

    match route {
        SummerMacro::Route(_) => {}
        _ => panic!("Expected Route variant"),
    }

    match auto_config {
        SummerMacro::AutoConfig(_) => {}
        _ => panic!("Expected AutoConfig variant"),
    }

    match job {
        SummerMacro::Job(_) => {}
        _ => panic!("Expected Job variant"),
    }
}

#[test]
fn test_rust_document_creation() {
    let uri = Url::parse("file:///test.rs").unwrap();
    let doc = RustDocument {
        uri: uri.clone(),
        content: "fn main() {}".to_string(),
        macros: vec![],
    };

    assert_eq!(doc.uri, uri);
    assert_eq!(doc.content, "fn main() {}");
    assert_eq!(doc.macros.len(), 0);
}

#[test]
fn test_field_without_inject() {
    let field = Field {
        name: "name".to_string(),
        type_name: "String".to_string(),
        inject: None,
    };

    assert_eq!(field.name, "name");
    assert_eq!(field.type_name, "String");
    assert!(field.inject.is_none());
}

#[test]
fn test_route_macro_multiple_methods() {
    let route = RouteMacro {
        path: "/api/resource".to_string(),
        methods: vec![HttpMethod::Get, HttpMethod::Post, HttpMethod::Put],
        middlewares: vec![],
        handler_name: "handle_resource".to_string(),
        range: test_range(),
        is_openapi: false,
    };

    assert_eq!(route.methods.len(), 3);
    assert!(route.methods.contains(&HttpMethod::Get));
    assert!(route.methods.contains(&HttpMethod::Post));
    assert!(route.methods.contains(&HttpMethod::Put));
}

#[test]
fn test_route_macro_multiple_middlewares() {
    let route = RouteMacro {
        path: "/protected".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec![
            "AuthMiddleware".to_string(),
            "LogMiddleware".to_string(),
            "RateLimitMiddleware".to_string(),
        ],
        handler_name: "protected_handler".to_string(),
        range: test_range(),
        is_openapi: false,
    };

    assert_eq!(route.middlewares.len(), 3);
    assert_eq!(route.middlewares[0], "AuthMiddleware");
    assert_eq!(route.middlewares[1], "LogMiddleware");
    assert_eq!(route.middlewares[2], "RateLimitMiddleware");
}

// ============ MacroAnalyzer 测试 ============

#[test]
fn test_macro_analyzer_creation() {
    let analyzer = MacroAnalyzer::new();
    // 验证分析器可以创建
    let _ = analyzer;
}

#[test]
fn test_macro_analyzer_default() {
    let _analyzer = MacroAnalyzer;
    // 验证单元结构体创建（不需要 default()）
}

#[test]
fn test_parse_empty_rust_file() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = "".to_string();

    let result = analyzer.parse(uri.clone(), content.clone());
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.uri, uri);
    assert_eq!(doc.content, content);
    assert_eq!(doc.macros.len(), 0);
}

#[test]
fn test_parse_simple_rust_file() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        fn main() {
            println!("Hello, world!");
        }
    "#
    .to_string();

    let result = analyzer.parse(uri.clone(), content.clone());
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.uri, uri);
    assert_eq!(doc.content, content);
}

#[test]
fn test_parse_invalid_rust_syntax() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = "fn main( {".to_string(); // 语法错误

    let result = analyzer.parse(uri, content);
    assert!(result.is_err());
}

#[test]
fn test_parse_struct_definition() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        struct MyStruct {
            field1: String,
            field2: i32,
        }
    "#
    .to_string();

    let result = analyzer.parse(uri, content);
    assert!(result.is_ok());
}

#[test]
fn test_parse_with_attributes() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        #[derive(Debug, Clone)]
        struct MyStruct {
            field: String,
        }
    "#
    .to_string();

    let result = analyzer.parse(uri, content);
    assert!(result.is_ok());
}

#[test]
fn test_extract_macros_empty_document() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = "fn main() {}".to_string();

    let doc = RustDocument {
        uri,
        content,
        macros: vec![],
    };

    let result = analyzer.extract_macros(doc);
    assert!(result.is_ok());

    let extracted_doc = result.unwrap();
    // 空文档不应该有宏
    assert_eq!(extracted_doc.macros.len(), 0);
}

#[test]
fn test_extract_macros_with_derive() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        #[derive(Service)]
        struct MyService {
            #[inject(component)]
            db: ConnectPool,
        }
    "#
    .to_string();

    let doc = RustDocument {
        uri,
        content,
        macros: vec![],
    };

    let result = analyzer.extract_macros(doc);
    assert!(result.is_ok());

    // 现在应该能够识别 Service 宏
    let extracted_doc = result.unwrap();
    assert_eq!(extracted_doc.macros.len(), 1);

    // 验证识别的宏类型
    match &extracted_doc.macros[0] {
        SummerMacro::DeriveService(service) => {
            assert_eq!(service.struct_name, "MyService");
            assert_eq!(service.fields.len(), 1);
            assert_eq!(service.fields[0].name, "db");
            assert_eq!(service.fields[0].type_name, "ConnectPool");
            assert!(service.fields[0].inject.is_some());

            if let Some(inject) = &service.fields[0].inject {
                assert_eq!(inject.inject_type, InjectType::Component);
            }
        }
        _ => panic!("Expected DeriveService macro"),
    }
}

#[test]
fn test_extract_macros_invalid_syntax() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = "fn main( {".to_string(); // 语法错误

    let doc = RustDocument {
        uri,
        content,
        macros: vec![],
    };

    let result = analyzer.extract_macros(doc);
    assert!(result.is_err());
}

#[test]
fn test_parse_and_extract_workflow() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        #[derive(Service)]
        struct MyService {
            field: String,
        }
    "#
    .to_string();

    // 先解析
    let parse_result = analyzer.parse(uri, content);
    assert!(parse_result.is_ok());

    let doc = parse_result.unwrap();

    // 再提取宏
    let extract_result = analyzer.extract_macros(doc);
    assert!(extract_result.is_ok());
}

// ============ 宏识别功能测试 ============

#[test]
fn test_recognize_service_macro() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        #[derive(Clone, Service)]
        struct UserService {
            #[inject(component)]
            db: ConnectPool,
            
            #[inject(config)]
            config: UserConfig,
        }
    "#
    .to_string();

    let doc = analyzer.parse(uri, content).unwrap();
    let result = analyzer.extract_macros(doc).unwrap();

    assert_eq!(result.macros.len(), 1);

    match &result.macros[0] {
        SummerMacro::DeriveService(service) => {
            assert_eq!(service.struct_name, "UserService");
            assert_eq!(service.fields.len(), 2);

            // 验证第一个字段
            assert_eq!(service.fields[0].name, "db");
            assert_eq!(service.fields[0].type_name, "ConnectPool");
            assert!(service.fields[0].inject.is_some());
            if let Some(inject) = &service.fields[0].inject {
                assert_eq!(inject.inject_type, InjectType::Component);
                assert_eq!(inject.component_name, None);
            }

            // 验证第二个字段
            assert_eq!(service.fields[1].name, "config");
            assert_eq!(service.fields[1].type_name, "UserConfig");
            assert!(service.fields[1].inject.is_some());
            if let Some(inject) = &service.fields[1].inject {
                assert_eq!(inject.inject_type, InjectType::Config);
            }
        }
        _ => panic!("Expected DeriveService macro"),
    }
}

#[test]
fn test_recognize_get_route_macro() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        #[get("/users/{id}")]
        async fn get_user(id: i64) -> Result<Json<User>> {
            Ok(Json(User::default()))
        }
    "#
    .to_string();

    let doc = analyzer.parse(uri, content).unwrap();
    let result = analyzer.extract_macros(doc).unwrap();

    assert_eq!(result.macros.len(), 1);

    match &result.macros[0] {
        SummerMacro::Route(route) => {
            assert_eq!(route.path, "/users/{id}");
            assert_eq!(route.methods.len(), 1);
            assert_eq!(route.methods[0], HttpMethod::Get);
            assert_eq!(route.handler_name, "get_user");
        }
        _ => panic!("Expected Route macro"),
    }
}

#[test]
fn test_recognize_post_route_macro() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        #[post("/users")]
        async fn create_user(user: Json<User>) -> Result<Json<User>> {
            Ok(user)
        }
    "#
    .to_string();

    let doc = analyzer.parse(uri, content).unwrap();
    let result = analyzer.extract_macros(doc).unwrap();

    assert_eq!(result.macros.len(), 1);

    match &result.macros[0] {
        SummerMacro::Route(route) => {
            assert_eq!(route.path, "/users");
            assert_eq!(route.methods.len(), 1);
            assert_eq!(route.methods[0], HttpMethod::Post);
            assert_eq!(route.handler_name, "create_user");
        }
        _ => panic!("Expected Route macro"),
    }
}

#[test]
fn test_recognize_multiple_http_methods() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        #[route("/api/resource", method = "GET", method = "POST")]
        async fn handle_resource() -> String {
            "OK".to_string()
        }
    "#
    .to_string();

    let doc = analyzer.parse(uri, content).unwrap();
    let result = analyzer.extract_macros(doc).unwrap();

    assert_eq!(result.macros.len(), 1);

    match &result.macros[0] {
        SummerMacro::Route(route) => {
            assert_eq!(route.path, "/api/resource");
            assert_eq!(route.methods.len(), 2);
            assert!(route.methods.contains(&HttpMethod::Get));
            assert!(route.methods.contains(&HttpMethod::Post));
            assert_eq!(route.handler_name, "handle_resource");
        }
        _ => panic!("Expected Route macro"),
    }
}

#[test]
fn test_recognize_auto_config_macro() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        #[auto_config(WebConfigurator)]
        #[tokio::main]
        async fn main() {
            App::new().run().await
        }
    "#
    .to_string();

    let doc = analyzer.parse(uri, content).unwrap();
    let result = analyzer.extract_macros(doc).unwrap();

    assert_eq!(result.macros.len(), 1);

    match &result.macros[0] {
        SummerMacro::AutoConfig(auto_config) => {
            assert_eq!(auto_config.configurator_type, "WebConfigurator");
        }
        _ => panic!("Expected AutoConfig macro"),
    }
}

#[test]
fn test_recognize_cron_job_macro() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        #[cron("0 0 * * * *")]
        async fn hourly_job() {
            println!("Running hourly job");
        }
    "#
    .to_string();

    let doc = analyzer.parse(uri, content).unwrap();
    let result = analyzer.extract_macros(doc).unwrap();

    assert_eq!(result.macros.len(), 1);

    match &result.macros[0] {
        SummerMacro::Job(JobMacro::Cron { expression, .. }) => {
            assert_eq!(expression, "0 0 * * * *");
        }
        _ => panic!("Expected Cron job macro"),
    }
}

#[test]
fn test_recognize_fix_delay_job_macro() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        #[fix_delay(5)]
        async fn delayed_job() {
            println!("Running delayed job");
        }
    "#
    .to_string();

    let doc = analyzer.parse(uri, content).unwrap();
    let result = analyzer.extract_macros(doc).unwrap();

    assert_eq!(result.macros.len(), 1);

    match &result.macros[0] {
        SummerMacro::Job(JobMacro::FixDelay { seconds, .. }) => {
            assert_eq!(*seconds, 5);
        }
        _ => panic!("Expected FixDelay job macro"),
    }
}

#[test]
fn test_recognize_fix_rate_job_macro() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        #[fix_rate(10)]
        async fn periodic_job() {
            println!("Running periodic job");
        }
    "#
    .to_string();

    let doc = analyzer.parse(uri, content).unwrap();
    let result = analyzer.extract_macros(doc).unwrap();

    assert_eq!(result.macros.len(), 1);

    match &result.macros[0] {
        SummerMacro::Job(JobMacro::FixRate { seconds, .. }) => {
            assert_eq!(*seconds, 10);
        }
        _ => panic!("Expected FixRate job macro"),
    }
}

#[test]
fn test_recognize_multiple_macros_in_file() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        #[derive(Service)]
        struct MyService {
            #[inject(component)]
            db: ConnectPool,
        }
        
        #[get("/users")]
        async fn get_users() -> String {
            "users".to_string()
        }
        
        #[post("/users")]
        async fn create_user() -> String {
            "created".to_string()
        }
        
        #[cron("0 0 * * * *")]
        async fn cleanup_job() {
            println!("Cleanup");
        }
    "#
    .to_string();

    let doc = analyzer.parse(uri, content).unwrap();
    let result = analyzer.extract_macros(doc).unwrap();

    // 应该识别 4 个宏：1 个 Service，2 个 Route，1 个 Job
    assert_eq!(result.macros.len(), 4);

    // 验证宏类型
    let mut service_count = 0;
    let mut route_count = 0;
    let mut job_count = 0;

    for macro_item in &result.macros {
        match macro_item {
            SummerMacro::DeriveService(_) => service_count += 1,
            SummerMacro::Route(_) => route_count += 1,
            SummerMacro::Job(_) => job_count += 1,
            _ => {}
        }
    }

    assert_eq!(service_count, 1);
    assert_eq!(route_count, 2);
    assert_eq!(job_count, 1);
}

// ============ 宏展开功能测试 ============

#[test]
fn test_expand_service_macro() {
    let service = ServiceMacro {
        struct_name: "UserService".to_string(),
        fields: vec![
            Field {
                name: "db".to_string(),
                type_name: "ConnectPool".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: None,
                    range: test_range(),
                }),
            },
            Field {
                name: "config".to_string(),
                type_name: "UserConfig".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Config,
                    component_name: None,
                    range: test_range(),
                }),
            },
        ],
        scope: ServiceScope::Singleton,
        range: test_range(),
    };

    let analyzer = MacroAnalyzer::new();
    let expanded = analyzer.expand_macro(&SummerMacro::DeriveService(service));

    // 验证展开的代码包含关键元素
    assert!(expanded.contains("UserService"));
    assert!(expanded.contains("impl UserService"));
    assert!(expanded.contains("pub fn build"));
    assert!(expanded.contains("app.get_component::<ConnectPool>()"));
    assert!(expanded.contains("app.get_config::<UserConfig>()"));
    assert!(expanded.contains("db"));
    assert!(expanded.contains("config"));
}

#[test]
fn test_expand_service_macro_with_named_component() {
    let service = ServiceMacro {
        struct_name: "MultiDbService".to_string(),
        fields: vec![
            Field {
                name: "primary_db".to_string(),
                type_name: "ConnectPool".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: Some("primary".to_string()),
                    range: test_range(),
                }),
            },
            Field {
                name: "secondary_db".to_string(),
                type_name: "ConnectPool".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: Some("secondary".to_string()),
                    range: test_range(),
                }),
            },
        ],
        scope: ServiceScope::Singleton,
        range: test_range(),
    };

    let analyzer = MacroAnalyzer::new();
    let expanded = analyzer.expand_macro(&SummerMacro::DeriveService(service));

    // 验证展开的代码包含命名组件
    assert!(expanded.contains("MultiDbService"));
    assert!(expanded.contains("get_component::<ConnectPool>(\"primary\")"));
    assert!(expanded.contains("get_component::<ConnectPool>(\"secondary\")"));
    assert!(expanded.contains("primary_db"));
    assert!(expanded.contains("secondary_db"));
}

#[test]
fn test_expand_service_macro_without_inject() {
    let service = ServiceMacro {
        struct_name: "SimpleService".to_string(),
        fields: vec![Field {
            name: "name".to_string(),
            type_name: "String".to_string(),
            inject: None,
        }],
        scope: ServiceScope::Singleton,
        range: test_range(),
    };

    let analyzer = MacroAnalyzer::new();
    let expanded = analyzer.expand_macro(&SummerMacro::DeriveService(service));

    // 验证展开的代码包含默认初始化
    assert!(expanded.contains("SimpleService"));
    assert!(expanded.contains("Default::default()"));
    assert!(expanded.contains("name"));
}

#[test]
fn test_expand_inject_macro_component() {
    let inject = InjectMacro {
        inject_type: InjectType::Component,
        component_name: None,
        range: test_range(),
    };

    let analyzer = MacroAnalyzer::new();
    let expanded = analyzer.expand_macro(&SummerMacro::Inject(inject));

    // 验证展开的代码包含注入说明
    assert!(expanded.contains("Inject 属性展开"));
    assert!(expanded.contains("注入类型: 组件"));
    assert!(expanded.contains("app.get_component::<T>()"));
}

#[test]
fn test_expand_inject_macro_component_with_name() {
    let inject = InjectMacro {
        inject_type: InjectType::Component,
        component_name: Some("my_component".to_string()),
        range: test_range(),
    };

    let analyzer = MacroAnalyzer::new();
    let expanded = analyzer.expand_macro(&SummerMacro::Inject(inject));

    // 验证展开的代码包含组件名称
    assert!(expanded.contains("组件名称: \"my_component\""));
    assert!(expanded.contains("app.get_component::<T>(\"my_component\")"));
}

#[test]
fn test_expand_inject_macro_config() {
    let inject = InjectMacro {
        inject_type: InjectType::Config,
        component_name: None,
        range: test_range(),
    };

    let analyzer = MacroAnalyzer::new();
    let expanded = analyzer.expand_macro(&SummerMacro::Inject(inject));

    // 验证展开的代码包含配置注入说明
    assert!(expanded.contains("注入类型: 配置"));
    assert!(expanded.contains("app.get_config::<T>()"));
}

#[test]
fn test_expand_auto_config_macro() {
    let auto_config = AutoConfigMacro {
        configurator_type: "WebConfigurator".to_string(),
        range: test_range(),
    };

    let analyzer = MacroAnalyzer::new();
    let expanded = analyzer.expand_macro(&SummerMacro::AutoConfig(auto_config));

    // 验证展开的代码包含配置器信息
    assert!(expanded.contains("AutoConfig 宏展开"));
    assert!(expanded.contains("配置器类型: WebConfigurator"));
    assert!(expanded.contains("WebConfigurator::new()"));
    assert!(expanded.contains("configurator.configure"));
}

#[test]
fn test_expand_route_macro_get() {
    let route = RouteMacro {
        path: "/users/{id}".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "get_user".to_string(),
        is_openapi: false,
        range: test_range(),
    };

    let analyzer = MacroAnalyzer::new();
    let expanded = analyzer.expand_macro(&SummerMacro::Route(route));

    // 验证展开的代码包含路由信息
    assert!(expanded.contains("路由宏展开"));
    assert!(expanded.contains("路由路径: /users/{id}"));
    assert!(expanded.contains("HTTP 方法: GET"));
    assert!(expanded.contains("router.route"));
    assert!(expanded.contains("get_user"));
}

#[test]
fn test_expand_route_macro_multiple_methods() {
    let route = RouteMacro {
        path: "/api/resource".to_string(),
        methods: vec![HttpMethod::Get, HttpMethod::Post],
        middlewares: vec![],
        handler_name: "handle_resource".to_string(),
        range: test_range(),
        is_openapi: false,
    };

    let analyzer = MacroAnalyzer::new();
    let expanded = analyzer.expand_macro(&SummerMacro::Route(route));

    // 验证展开的代码包含多个方法
    assert!(expanded.contains("HTTP 方法: GET, POST"));
    assert!(expanded.contains("handle_resource"));
}

#[test]
fn test_expand_route_macro_with_middlewares() {
    let route = RouteMacro {
        path: "/protected".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec!["AuthMiddleware".to_string(), "LogMiddleware".to_string()],
        handler_name: "protected_handler".to_string(),
        range: test_range(),
        is_openapi: false,
    };

    let analyzer = MacroAnalyzer::new();
    let expanded = analyzer.expand_macro(&SummerMacro::Route(route));

    // 验证展开的代码包含中间件信息
    assert!(expanded.contains("中间件: AuthMiddleware, LogMiddleware"));
    assert!(expanded.contains("应用中间件"));
    assert!(expanded.contains(".layer(AuthMiddleware)"));
    assert!(expanded.contains(".layer(LogMiddleware)"));
}

#[test]
fn test_expand_cron_job_macro() {
    let job = JobMacro::Cron {
        expression: "0 0 * * * *".to_string(),
        range: test_range(),
    };

    let analyzer = MacroAnalyzer::new();
    let expanded = analyzer.expand_macro(&SummerMacro::Job(job));

    // 验证展开的代码包含 Cron 任务信息
    assert!(expanded.contains("任务调度宏展开"));
    assert!(expanded.contains("任务类型: Cron"));
    assert!(expanded.contains("Cron 表达式: 0 0 * * * *"));
    assert!(expanded.contains("CronJob::new"));
}

#[test]
fn test_expand_fix_delay_job_macro() {
    let job = JobMacro::FixDelay {
        seconds: 5,
        range: test_range(),
    };

    let analyzer = MacroAnalyzer::new();
    let expanded = analyzer.expand_macro(&SummerMacro::Job(job));

    // 验证展开的代码包含 FixDelay 任务信息
    assert!(expanded.contains("任务类型: FixDelay"));
    assert!(expanded.contains("延迟秒数: 5"));
    assert!(expanded.contains("任务完成后延迟指定秒数再次执行"));
    assert!(expanded.contains("FixDelayJob::new(5"));
}

#[test]
fn test_expand_fix_rate_job_macro() {
    let job = JobMacro::FixRate {
        seconds: 10,
        range: test_range(),
    };

    let analyzer = MacroAnalyzer::new();
    let expanded = analyzer.expand_macro(&SummerMacro::Job(job));

    // 验证展开的代码包含 FixRate 任务信息
    assert!(expanded.contains("任务类型: FixRate"));
    assert!(expanded.contains("频率秒数: 10"));
    assert!(expanded.contains("每隔指定秒数执行一次任务"));
    assert!(expanded.contains("FixRateJob::new(10"));
}

#[test]
fn test_expand_macro_produces_valid_syntax() {
    // 测试所有宏展开都生成语法正确的代码（至少是有效的注释）
    let analyzer = MacroAnalyzer::new();

    let service = SummerMacro::DeriveService(ServiceMacro {
        struct_name: "TestService".to_string(),
        fields: vec![],
        scope: ServiceScope::Singleton,
        range: test_range(),
    });

    let expanded = analyzer.expand_macro(&service);

    // 验证生成的代码不为空
    assert!(!expanded.is_empty());

    // 验证生成的代码包含注释标记
    assert!(expanded.contains("//"));
}

#[test]
fn test_expand_empty_service() {
    let service = ServiceMacro {
        struct_name: "EmptyService".to_string(),
        fields: vec![],
        scope: ServiceScope::Singleton,
        range: test_range(),
    };

    let analyzer = MacroAnalyzer::new();
    let expanded = analyzer.expand_macro(&SummerMacro::DeriveService(service));

    // 验证空服务也能正确展开
    assert!(expanded.contains("EmptyService"));
    assert!(expanded.contains("impl EmptyService"));
    assert!(expanded.contains("pub fn build"));
}

#[test]
fn test_expand_all_macro_types() {
    let analyzer = MacroAnalyzer::new();

    // 测试所有宏类型都能展开
    let macros = vec![
        SummerMacro::DeriveService(ServiceMacro {
            struct_name: "TestService".to_string(),
            fields: vec![],
            scope: ServiceScope::Singleton,
            range: test_range(),
        }),
        SummerMacro::Inject(InjectMacro {
            inject_type: InjectType::Component,
            component_name: None,
            range: test_range(),
        }),
        SummerMacro::AutoConfig(AutoConfigMacro {
            configurator_type: "TestConfigurator".to_string(),
            range: test_range(),
        }),
        SummerMacro::Route(RouteMacro {
            path: "/test".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "test_handler".to_string(),
            range: test_range(),
            is_openapi: false,
        }),
        SummerMacro::Job(JobMacro::Cron {
            expression: "0 0 * * * *".to_string(),
            range: test_range(),
        }),
    ];

    for macro_item in macros {
        let expanded = analyzer.expand_macro(&macro_item);
        // 所有宏都应该能生成非空的展开代码
        assert!(!expanded.is_empty());
    }
}

#[test]
fn test_expand_macro_comprehensive_example() {
    // 综合测试：展示完整的宏展开功能
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();

    // 创建一个包含多种宏的复杂示例
    let content = r#"
        #[derive(Clone, Service)]
        struct UserService {
            #[inject(component = "primary")]
            db: ConnectPool,
            
            #[inject(config)]
            config: UserConfig,
        }
        
        #[get("/users/{id}")]
        #[middlewares(AuthMiddleware, LogMiddleware)]
        async fn get_user(id: i64) -> Result<Json<User>> {
            Ok(Json(User::default()))
        }
        
        #[auto_config(WebConfigurator)]
        #[tokio::main]
        async fn main() {
            App::new().run().await
        }
        
        #[cron("0 0 * * * *")]
        async fn hourly_cleanup() {
            println!("Cleanup");
        }
    "#
    .to_string();

    // 解析并提取宏
    let doc = analyzer.parse(uri, content).unwrap();
    let result = analyzer.extract_macros(doc).unwrap();

    // 应该识别到 4 个宏
    assert_eq!(result.macros.len(), 4);

    // 展开所有宏并验证
    for macro_item in &result.macros {
        let expanded = analyzer.expand_macro(macro_item);

        // 验证展开的代码不为空
        assert!(!expanded.is_empty());

        // 验证展开的代码包含注释
        assert!(expanded.contains("//"));

        // 根据宏类型验证特定内容
        match macro_item {
            SummerMacro::DeriveService(service) => {
                assert!(expanded.contains(&service.struct_name));
                assert!(expanded.contains("impl"));
                assert!(expanded.contains("pub fn build"));
            }
            SummerMacro::Route(route) => {
                assert!(expanded.contains(&route.path));
                assert!(expanded.contains(&route.handler_name));
            }
            SummerMacro::AutoConfig(auto_config) => {
                assert!(expanded.contains(&auto_config.configurator_type));
            }
            SummerMacro::Job(_) => {
                assert!(expanded.contains("任务调度宏展开"));
            }
            _ => {}
        }
    }
}

#[test]
fn test_expand_macro_readability() {
    // 测试展开的代码是否易读
    let analyzer = MacroAnalyzer::new();

    let service = ServiceMacro {
        struct_name: "MyService".to_string(),
        fields: vec![Field {
            name: "db".to_string(),
            type_name: "ConnectPool".to_string(),
            inject: Some(InjectMacro {
                inject_type: InjectType::Component,
                component_name: None,
                range: test_range(),
            }),
        }],
        scope: ServiceScope::Singleton,
        range: test_range(),
    };

    let expanded = analyzer.expand_macro(&SummerMacro::DeriveService(service));

    // 验证代码包含清晰的注释
    assert!(expanded.contains("// 原始定义"));
    assert!(expanded.contains("// 展开后的代码"));

    // 验证代码格式良好（包含换行和缩进）
    assert!(expanded.contains("\n"));
    assert!(expanded.contains("    ")); // 缩进

    // 验证代码包含有意义的说明
    assert!(expanded.contains("从应用上下文构建服务实例"));
}

#[test]
fn test_recognize_all_http_method_macros() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        #[get("/get")]
        async fn get_handler() {}
        
        #[post("/post")]
        async fn post_handler() {}
        
        #[put("/put")]
        async fn put_handler() {}
        
        #[delete("/delete")]
        async fn delete_handler() {}
        
        #[patch("/patch")]
        async fn patch_handler() {}
        
        #[head("/head")]
        async fn head_handler() {}
        
        #[options("/options")]
        async fn options_handler() {}
    "#
    .to_string();

    let doc = analyzer.parse(uri, content).unwrap();
    let result = analyzer.extract_macros(doc).unwrap();

    assert_eq!(result.macros.len(), 7);

    // 验证所有 HTTP 方法都被识别
    let methods: Vec<HttpMethod> = result
        .macros
        .iter()
        .filter_map(|m| match m {
            SummerMacro::Route(route) => Some(route.methods[0].clone()),
            _ => None,
        })
        .collect();

    assert!(methods.contains(&HttpMethod::Get));
    assert!(methods.contains(&HttpMethod::Post));
    assert!(methods.contains(&HttpMethod::Put));
    assert!(methods.contains(&HttpMethod::Delete));
    assert!(methods.contains(&HttpMethod::Patch));
    assert!(methods.contains(&HttpMethod::Head));
    assert!(methods.contains(&HttpMethod::Options));
}

#[test]
fn test_service_without_inject() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        #[derive(Service)]
        struct SimpleService {
            name: String,
            count: i32,
        }
    "#
    .to_string();

    let doc = analyzer.parse(uri, content).unwrap();
    let result = analyzer.extract_macros(doc).unwrap();

    assert_eq!(result.macros.len(), 1);

    match &result.macros[0] {
        SummerMacro::DeriveService(service) => {
            assert_eq!(service.struct_name, "SimpleService");
            assert_eq!(service.fields.len(), 2);

            // 验证字段没有 inject 属性
            assert!(service.fields[0].inject.is_none());
            assert!(service.fields[1].inject.is_none());
        }
        _ => panic!("Expected DeriveService macro"),
    }
}

#[test]
fn test_inject_with_component_name() {
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        #[derive(Service)]
        struct MultiDbService {
            #[inject(component = "primary")]
            primary_db: ConnectPool,
            
            #[inject(component = "secondary")]
            secondary_db: ConnectPool,
        }
    "#
    .to_string();

    let doc = analyzer.parse(uri, content).unwrap();
    let result = analyzer.extract_macros(doc).unwrap();

    assert_eq!(result.macros.len(), 1);

    match &result.macros[0] {
        SummerMacro::DeriveService(service) => {
            assert_eq!(service.fields.len(), 2);

            // 验证第一个字段的组件名称
            if let Some(inject) = &service.fields[0].inject {
                assert_eq!(inject.component_name, Some("primary".to_string()));
            } else {
                panic!("Expected inject attribute");
            }

            // 验证第二个字段的组件名称
            if let Some(inject) = &service.fields[1].inject {
                assert_eq!(inject.component_name, Some("secondary".to_string()));
            } else {
                panic!("Expected inject attribute");
            }
        }
        _ => panic!("Expected DeriveService macro"),
    }
}

// ============ 宏悬停提示功能测试 ============

#[test]
fn test_hover_service_macro() {
    let analyzer = MacroAnalyzer::new();
    let service = ServiceMacro {
        struct_name: "UserService".to_string(),
        fields: vec![
            Field {
                name: "db".to_string(),
                type_name: "ConnectPool".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: None,
                    range: test_range(),
                }),
            },
            Field {
                name: "config".to_string(),
                type_name: "UserConfig".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Config,
                    component_name: None,
                    range: test_range(),
                }),
            },
        ],
        scope: ServiceScope::Singleton,
        range: test_range(),
    };

    let hover = analyzer.hover_macro(&SummerMacro::DeriveService(service));

    // 验证悬停提示包含关键信息
    assert!(hover.contains("# Service 派生宏"));
    assert!(hover.contains("UserService"));
    assert!(hover.contains("注入字段"));
    assert!(hover.contains("db"));
    assert!(hover.contains("ConnectPool"));
    assert!(hover.contains("config"));
    assert!(hover.contains("UserConfig"));
    assert!(hover.contains("展开后的代码"));
    assert!(hover.contains("```rust"));
    assert!(hover.contains("impl UserService"));
}

#[test]
fn test_hover_service_macro_with_named_component() {
    let analyzer = MacroAnalyzer::new();
    let service = ServiceMacro {
        struct_name: "MultiDbService".to_string(),
        fields: vec![Field {
            name: "primary_db".to_string(),
            type_name: "ConnectPool".to_string(),
            inject: Some(InjectMacro {
                inject_type: InjectType::Component,
                component_name: Some("primary".to_string()),
                range: test_range(),
            }),
        }],
        scope: ServiceScope::Singleton,
        range: test_range(),
    };

    let hover = analyzer.hover_macro(&SummerMacro::DeriveService(service));

    // 验证悬停提示包含组件名称
    assert!(hover.contains("MultiDbService"));
    assert!(hover.contains("primary_db"));
    assert!(hover.contains("注入组件 `\"primary\"`"));
}

#[test]
fn test_hover_service_macro_empty_fields() {
    let analyzer = MacroAnalyzer::new();
    let service = ServiceMacro {
        struct_name: "EmptyService".to_string(),
        fields: vec![],
        scope: ServiceScope::Singleton,
        range: test_range(),
    };

    let hover = analyzer.hover_macro(&SummerMacro::DeriveService(service));

    // 验证空服务也能生成悬停提示
    assert!(hover.contains("# Service 派生宏"));
    assert!(hover.contains("EmptyService"));
    assert!(hover.contains("展开后的代码"));
}

#[test]
fn test_hover_inject_macro_component() {
    let analyzer = MacroAnalyzer::new();
    let inject = InjectMacro {
        inject_type: InjectType::Component,
        component_name: None,
        range: test_range(),
    };

    let hover = analyzer.hover_macro(&SummerMacro::Inject(inject));

    // 验证悬停提示包含注入信息
    assert!(hover.contains("# Inject 属性宏"));
    assert!(hover.contains("注入类型"));
    assert!(hover.contains("组件 (Component)"));
    assert!(hover.contains("app.get_component::<T>()"));
    assert!(hover.contains("使用示例"));
    assert!(hover.contains("```rust"));
}

#[test]
fn test_hover_inject_macro_component_with_name() {
    let analyzer = MacroAnalyzer::new();
    let inject = InjectMacro {
        inject_type: InjectType::Component,
        component_name: Some("my_component".to_string()),
        range: test_range(),
    };

    let hover = analyzer.hover_macro(&SummerMacro::Inject(inject));

    // 验证悬停提示包含组件名称
    assert!(hover.contains("组件名称"));
    assert!(hover.contains("my_component"));
    assert!(hover.contains("app.get_component::<T>(\"my_component\")"));
    assert!(hover.contains("多实例场景"));
}

#[test]
fn test_hover_inject_macro_config() {
    let analyzer = MacroAnalyzer::new();
    let inject = InjectMacro {
        inject_type: InjectType::Config,
        component_name: None,
        range: test_range(),
    };

    let hover = analyzer.hover_macro(&SummerMacro::Inject(inject));

    // 验证悬停提示包含配置注入信息
    assert!(hover.contains("# Inject 属性宏"));
    assert!(hover.contains("配置 (Config)"));
    assert!(hover.contains("app.get_config::<T>()"));
    assert!(hover.contains("config/app.toml"));
    assert!(hover.contains("#[config_prefix]"));
}

#[test]
fn test_hover_auto_config_macro() {
    let analyzer = MacroAnalyzer::new();
    let auto_config = AutoConfigMacro {
        configurator_type: "WebConfigurator".to_string(),
        range: test_range(),
    };

    let hover = analyzer.hover_macro(&SummerMacro::AutoConfig(auto_config));

    // 验证悬停提示包含配置器信息
    assert!(hover.contains("# AutoConfig 属性宏"));
    assert!(hover.contains("WebConfigurator"));
    assert!(hover.contains("展开后的代码"));
}

#[test]
fn test_hover_route_macro() {
    let analyzer = MacroAnalyzer::new();
    let route = RouteMacro {
        path: "/users/{id}".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "get_user".to_string(),
        is_openapi: false,
        range: test_range(),
    };

    let hover = analyzer.hover_macro(&SummerMacro::Route(route));

    // 验证悬停提示包含路由信息
    assert!(hover.contains("# 路由宏"));
    assert!(hover.contains("路由路径"));
    assert!(hover.contains("/users/{id}"));
    assert!(hover.contains("HTTP 方法"));
    assert!(hover.contains("`GET`"));
    assert!(hover.contains("处理器函数"));
    assert!(hover.contains("get_user"));
}

#[test]
fn test_hover_route_macro_multiple_methods() {
    let analyzer = MacroAnalyzer::new();
    let route = RouteMacro {
        path: "/api/resource".to_string(),
        methods: vec![HttpMethod::Get, HttpMethod::Post],
        middlewares: vec![],
        handler_name: "handle_resource".to_string(),
        range: test_range(),
        is_openapi: false,
    };

    let hover = analyzer.hover_macro(&SummerMacro::Route(route));

    // 验证悬停提示包含多个方法
    assert!(hover.contains("`GET`"));
    assert!(hover.contains("`POST`"));
}

#[test]
fn test_hover_route_macro_with_middlewares() {
    let analyzer = MacroAnalyzer::new();
    let route = RouteMacro {
        path: "/protected".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec!["AuthMiddleware".to_string(), "LogMiddleware".to_string()],
        handler_name: "protected_handler".to_string(),
        range: test_range(),
        is_openapi: false,
    };

    let hover = analyzer.hover_macro(&SummerMacro::Route(route));

    // 验证悬停提示包含中间件信息
    assert!(hover.contains("中间件"));
    assert!(hover.contains("AuthMiddleware"));
    assert!(hover.contains("LogMiddleware"));
}

#[test]
fn test_hover_cron_job_macro() {
    let analyzer = MacroAnalyzer::new();
    let job = JobMacro::Cron {
        expression: "0 0 * * * *".to_string(),
        range: test_range(),
    };

    let hover = analyzer.hover_macro(&SummerMacro::Job(job));

    // 验证悬停提示包含 Cron 任务信息
    assert!(hover.contains("# 任务调度宏"));
    assert!(hover.contains("定时任务"));
    assert!(hover.contains("Cron 表达式"));
    assert!(hover.contains("0 0 * * * *"));
    assert!(hover.contains("秒 分 时 日 月 星期"));
}

#[test]
fn test_hover_fix_delay_job_macro() {
    let analyzer = MacroAnalyzer::new();
    let job = JobMacro::FixDelay {
        seconds: 5,
        range: test_range(),
    };

    let hover = analyzer.hover_macro(&SummerMacro::Job(job));

    // 验证悬停提示包含 FixDelay 任务信息
    assert!(hover.contains("# 任务调度宏"));
    assert!(hover.contains("固定延迟任务"));
    assert!(hover.contains("延迟秒数"));
    assert!(hover.contains("5"));
}

#[test]
fn test_hover_fix_rate_job_macro() {
    let analyzer = MacroAnalyzer::new();
    let job = JobMacro::FixRate {
        seconds: 10,
        range: test_range(),
    };

    let hover = analyzer.hover_macro(&SummerMacro::Job(job));

    // 验证悬停提示包含 FixRate 任务信息
    assert!(hover.contains("# 任务调度宏"));
    assert!(hover.contains("固定频率任务"));
    assert!(hover.contains("频率秒数"));
    assert!(hover.contains("10"));
}

#[test]
fn test_hover_all_macro_types() {
    let analyzer = MacroAnalyzer::new();

    // 测试所有宏类型都能生成悬停提示
    let macros = vec![
        SummerMacro::DeriveService(ServiceMacro {
            struct_name: "TestService".to_string(),
            fields: vec![],
            scope: ServiceScope::Singleton,
            range: test_range(),
        }),
        SummerMacro::Inject(InjectMacro {
            inject_type: InjectType::Component,
            component_name: None,
            range: test_range(),
        }),
        SummerMacro::AutoConfig(AutoConfigMacro {
            configurator_type: "TestConfigurator".to_string(),
            range: test_range(),
        }),
        SummerMacro::Route(RouteMacro {
            path: "/test".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "test_handler".to_string(),
            range: test_range(),
            is_openapi: false,
        }),
        SummerMacro::Job(JobMacro::Cron {
            expression: "0 0 * * * *".to_string(),
            range: test_range(),
        }),
    ];

    for macro_item in macros {
        let hover = analyzer.hover_macro(&macro_item);

        // 所有宏都应该能生成非空的悬停提示
        assert!(!hover.is_empty());

        // 所有悬停提示都应该包含标题
        assert!(hover.contains("#"));

        // 所有悬停提示都应该包含代码块
        assert!(hover.contains("```"));
    }
}

#[test]
fn test_hover_markdown_format() {
    let analyzer = MacroAnalyzer::new();
    let service = ServiceMacro {
        struct_name: "MyService".to_string(),
        fields: vec![Field {
            name: "db".to_string(),
            type_name: "ConnectPool".to_string(),
            inject: Some(InjectMacro {
                inject_type: InjectType::Component,
                component_name: None,
                range: test_range(),
            }),
        }],
        scope: ServiceScope::Singleton,
        range: test_range(),
    };

    let hover = analyzer.hover_macro(&SummerMacro::DeriveService(service));

    // 验证 Markdown 格式
    assert!(hover.contains("# ")); // 标题
    assert!(hover.contains("**")); // 粗体
    assert!(hover.contains("`")); // 代码
    assert!(hover.contains("```rust")); // 代码块
    assert!(hover.contains("- ")); // 列表
}

#[test]
fn test_hover_comprehensive_example() {
    // 综合测试：验证完整的悬停提示功能
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();

    let content = r#"
        #[derive(Clone, Service)]
        struct UserService {
            #[inject(component = "primary")]
            db: ConnectPool,
            
            #[inject(config)]
            config: UserConfig,
        }
        
        #[get("/users/{id}")]
        async fn get_user(id: i64) -> Result<Json<User>> {
            Ok(Json(User::default()))
        }
    "#
    .to_string();

    // 解析并提取宏
    let doc = analyzer.parse(uri, content).unwrap();
    let result = analyzer.extract_macros(doc).unwrap();

    // 为所有宏生成悬停提示
    for macro_item in &result.macros {
        let hover = analyzer.hover_macro(macro_item);

        // 验证悬停提示不为空
        assert!(!hover.is_empty());

        // 验证悬停提示包含标题
        assert!(hover.starts_with("#"));

        // 验证悬停提示包含代码块
        assert!(hover.contains("```rust"));

        // 根据宏类型验证特定内容
        match macro_item {
            SummerMacro::DeriveService(service) => {
                assert!(hover.contains(&service.struct_name));
                assert!(hover.contains("注入字段"));
            }
            SummerMacro::Route(route) => {
                assert!(hover.contains(&route.path));
                assert!(hover.contains("HTTP 方法"));
            }
            _ => {}
        }
    }
}

#[test]
fn test_hover_readability() {
    // 测试悬停提示的可读性
    let analyzer = MacroAnalyzer::new();

    let inject = InjectMacro {
        inject_type: InjectType::Component,
        component_name: Some("my_db".to_string()),
        range: test_range(),
    };

    let hover = analyzer.hover_macro(&SummerMacro::Inject(inject));

    // 验证悬停提示包含清晰的说明
    assert!(hover.contains("标记字段从应用上下文中自动注入依赖"));

    // 验证悬停提示包含使用示例
    assert!(hover.contains("使用示例"));

    // 验证悬停提示格式良好
    assert!(hover.contains("\n\n")); // 段落分隔

    // 验证悬停提示包含有意义的标签
    assert!(hover.contains("**注入类型**"));
    assert!(hover.contains("**组件名称**"));
    assert!(hover.contains("**注入代码**"));
}

#[test]
fn test_hover_service_with_mixed_fields() {
    // 测试包含不同类型注入的服务
    let analyzer = MacroAnalyzer::new();
    let service = ServiceMacro {
        struct_name: "MixedService".to_string(),
        fields: vec![
            Field {
                name: "db".to_string(),
                type_name: "ConnectPool".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: None,
                    range: test_range(),
                }),
            },
            Field {
                name: "cache".to_string(),
                type_name: "RedisPool".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: Some("redis".to_string()),
                    range: test_range(),
                }),
            },
            Field {
                name: "config".to_string(),
                type_name: "AppConfig".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Config,
                    component_name: None,
                    range: test_range(),
                }),
            },
            Field {
                name: "name".to_string(),
                type_name: "String".to_string(),
                inject: None,
            },
        ],
        scope: ServiceScope::Singleton,
        range: test_range(),
    };

    let hover = analyzer.hover_macro(&SummerMacro::DeriveService(service));

    // 验证所有字段都在悬停提示中
    assert!(hover.contains("db"));
    assert!(hover.contains("cache"));
    assert!(hover.contains("config"));
    assert!(hover.contains("name"));

    // 验证不同的注入类型都被正确标识
    assert!(hover.contains("注入组件"));
    assert!(hover.contains("注入组件 `\"redis\"`"));
    assert!(hover.contains("注入配置"));
}

// ============ 宏参数验证功能测试 ============

#[test]
fn test_validate_service_macro_valid() {
    let analyzer = MacroAnalyzer::new();
    let service = ServiceMacro {
        struct_name: "UserService".to_string(),
        fields: vec![Field {
            name: "db".to_string(),
            type_name: "ConnectPool".to_string(),
            inject: Some(InjectMacro {
                inject_type: InjectType::Component,
                component_name: None,
                range: test_range(),
            }),
        }],
        scope: ServiceScope::Singleton,
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::DeriveService(service));

    // 有效的 Service 宏不应该产生诊断
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_validate_service_macro_empty_component_name() {
    let analyzer = MacroAnalyzer::new();
    let service = ServiceMacro {
        struct_name: "UserService".to_string(),
        fields: vec![Field {
            name: "db".to_string(),
            type_name: "ConnectPool".to_string(),
            inject: Some(InjectMacro {
                inject_type: InjectType::Component,
                component_name: Some("".to_string()), // 空字符串
                range: test_range(),
            }),
        }],
        scope: ServiceScope::Singleton,
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::DeriveService(service));

    // 应该产生错误诊断
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].severity,
        Some(lsp_types::DiagnosticSeverity::ERROR)
    );
    assert!(diagnostics[0].message.contains("组件名称不能为空字符串"));
}

#[test]
fn test_validate_inject_macro_config_with_name() {
    let analyzer = MacroAnalyzer::new();
    let inject = InjectMacro {
        inject_type: InjectType::Config,
        component_name: Some("my_config".to_string()), // Config 不应该有组件名称
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Inject(inject));

    // 应该产生错误诊断
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].severity,
        Some(lsp_types::DiagnosticSeverity::ERROR)
    );
    assert!(diagnostics[0]
        .message
        .contains("配置注入 (config) 不应该指定组件名称"));
}

#[test]
fn test_validate_inject_macro_component_valid() {
    let analyzer = MacroAnalyzer::new();
    let inject = InjectMacro {
        inject_type: InjectType::Component,
        component_name: Some("my_component".to_string()),
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Inject(inject));

    // 有效的 Component 注入不应该产生诊断
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_validate_inject_macro_config_valid() {
    let analyzer = MacroAnalyzer::new();
    let inject = InjectMacro {
        inject_type: InjectType::Config,
        component_name: None,
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Inject(inject));

    // 有效的 Config 注入不应该产生诊断
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_validate_auto_config_macro_empty_type() {
    let analyzer = MacroAnalyzer::new();
    let auto_config = AutoConfigMacro {
        configurator_type: "".to_string(), // 空字符串
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::AutoConfig(auto_config));

    // 应该产生错误诊断
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].severity,
        Some(lsp_types::DiagnosticSeverity::ERROR)
    );
    assert!(diagnostics[0].message.contains("必须指定配置器类型"));
}

#[test]
fn test_validate_auto_config_macro_valid() {
    let analyzer = MacroAnalyzer::new();
    let auto_config = AutoConfigMacro {
        configurator_type: "WebConfigurator".to_string(),
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::AutoConfig(auto_config));

    // 有效的 AutoConfig 宏不应该产生诊断
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_validate_route_macro_empty_path() {
    let analyzer = MacroAnalyzer::new();
    let route = RouteMacro {
        path: "".to_string(), // 空路径
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "handler".to_string(),
        range: test_range(),
        is_openapi: false,
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Route(route));

    // 应该产生错误诊断
    assert!(!diagnostics.is_empty());
    assert!(diagnostics
        .iter()
        .any(|d| d.message.contains("路由路径不能为空")));
}

#[test]
fn test_validate_route_macro_path_without_slash() {
    let analyzer = MacroAnalyzer::new();
    let route = RouteMacro {
        path: "users".to_string(), // 不以 / 开头
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "handler".to_string(),
        range: test_range(),
        is_openapi: false,
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Route(route));

    // 应该产生错误诊断
    assert!(!diagnostics.is_empty());
    assert!(diagnostics
        .iter()
        .any(|d| d.message.contains("必须以 '/' 开头")));
}

#[test]
fn test_validate_route_macro_no_methods() {
    let analyzer = MacroAnalyzer::new();
    let route = RouteMacro {
        path: "/users".to_string(),
        methods: vec![], // 没有方法
        middlewares: vec![],
        handler_name: "handler".to_string(),
        range: test_range(),
        is_openapi: false,
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Route(route));

    // 应该产生错误诊断
    assert!(!diagnostics.is_empty());
    assert!(diagnostics
        .iter()
        .any(|d| d.message.contains("至少指定一个 HTTP 方法")));
}

#[test]
fn test_validate_route_macro_empty_handler_name() {
    let analyzer = MacroAnalyzer::new();
    let route = RouteMacro {
        path: "/users".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "".to_string(), // 空处理器名称
        is_openapi: false,
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Route(route));

    // 应该产生错误诊断
    assert!(!diagnostics.is_empty());
    assert!(diagnostics
        .iter()
        .any(|d| d.message.contains("处理器函数名称不能为空")));
}

#[test]
fn test_validate_route_macro_valid() {
    let analyzer = MacroAnalyzer::new();
    let route = RouteMacro {
        path: "/users/{id}".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "get_user".to_string(),
        is_openapi: false,
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Route(route));

    // 有效的路由宏不应该产生诊断
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_validate_route_macro_nested_braces() {
    let analyzer = MacroAnalyzer::new();
    let route = RouteMacro {
        path: "/users/{{id}}".to_string(), // 嵌套的大括号
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "handler".to_string(),
        is_openapi: false,
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Route(route));

    // 应该产生错误诊断
    assert!(!diagnostics.is_empty());
    assert!(diagnostics.iter().any(|d| d.message.contains("不能嵌套")));
}

#[test]
fn test_validate_route_macro_unmatched_closing_brace() {
    let analyzer = MacroAnalyzer::new();
    let route = RouteMacro {
        path: "/users/id}".to_string(), // 没有匹配的开括号
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "handler".to_string(),
        is_openapi: false,
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Route(route));

    // 应该产生错误诊断
    assert!(!diagnostics.is_empty());
    assert!(diagnostics.iter().any(|d| d.message.contains("缺少开括号")));
}

#[test]
fn test_validate_route_macro_unclosed_brace() {
    let analyzer = MacroAnalyzer::new();
    let route = RouteMacro {
        path: "/users/{id".to_string(), // 没有闭括号
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "handler".to_string(),
        range: test_range(),
        is_openapi: false,
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Route(route));

    // 应该产生错误诊断
    assert!(!diagnostics.is_empty());
    assert!(diagnostics.iter().any(|d| d.message.contains("缺少闭括号")));
}

#[test]
fn test_validate_route_macro_empty_param_name() {
    let analyzer = MacroAnalyzer::new();
    let route = RouteMacro {
        path: "/users/{}".to_string(), // 空参数名称
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "handler".to_string(),
        is_openapi: false,
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Route(route));

    // 应该产生错误诊断
    assert!(!diagnostics.is_empty());
    assert!(diagnostics
        .iter()
        .any(|d| d.message.contains("参数名称不能为空")));
}

#[test]
fn test_validate_route_macro_invalid_param_name() {
    let analyzer = MacroAnalyzer::new();
    let route = RouteMacro {
        path: "/users/{id-name}".to_string(), // 包含非法字符
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "handler".to_string(),
        is_openapi: false,
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Route(route));

    // 应该产生错误诊断
    assert!(!diagnostics.is_empty());
    assert!(diagnostics
        .iter()
        .any(|d| d.message.contains("只能包含字母、数字和下划线")));
}

#[test]
fn test_validate_route_macro_valid_param_names() {
    let analyzer = MacroAnalyzer::new();
    let route = RouteMacro {
        path: "/users/{user_id}/posts/{post_id}".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "handler".to_string(),
        is_openapi: false,
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Route(route));

    // 有效的参数名称不应该产生诊断
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_validate_cron_job_empty_expression() {
    let analyzer = MacroAnalyzer::new();
    let job = JobMacro::Cron {
        expression: "".to_string(), // 空表达式
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Job(job));

    // 应该产生错误诊断
    assert!(!diagnostics.is_empty());
    assert!(diagnostics
        .iter()
        .any(|d| d.message.contains("Cron 表达式不能为空")));
}

#[test]
fn test_validate_cron_job_invalid_parts() {
    let analyzer = MacroAnalyzer::new();
    let job = JobMacro::Cron {
        expression: "0 0 *".to_string(), // 只有 3 个部分，应该有 6 个
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Job(job));

    // 应该产生错误诊断
    assert!(!diagnostics.is_empty());
    assert!(diagnostics
        .iter()
        .any(|d| d.message.contains("应该包含 6 个部分")));
}

#[test]
fn test_validate_cron_job_valid() {
    let analyzer = MacroAnalyzer::new();
    let job = JobMacro::Cron {
        expression: "0 0 * * * *".to_string(), // 有效的 cron 表达式
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Job(job));

    // 有效的 cron 表达式不应该产生诊断
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_validate_fix_delay_job_zero_seconds() {
    let analyzer = MacroAnalyzer::new();
    let job = JobMacro::FixDelay {
        seconds: 0, // 0 秒延迟
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Job(job));

    // 应该产生警告诊断
    assert!(!diagnostics.is_empty());
    assert_eq!(
        diagnostics[0].severity,
        Some(lsp_types::DiagnosticSeverity::WARNING)
    );
    assert!(diagnostics[0].message.contains("延迟秒数为 0"));
}

#[test]
fn test_validate_fix_delay_job_valid() {
    let analyzer = MacroAnalyzer::new();
    let job = JobMacro::FixDelay {
        seconds: 5,
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Job(job));

    // 有效的 FixDelay 任务不应该产生诊断
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_validate_fix_rate_job_zero_seconds() {
    let analyzer = MacroAnalyzer::new();
    let job = JobMacro::FixRate {
        seconds: 0, // 0 秒频率
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Job(job));

    // 应该产生错误诊断
    assert!(!diagnostics.is_empty());
    assert_eq!(
        diagnostics[0].severity,
        Some(lsp_types::DiagnosticSeverity::ERROR)
    );
    assert!(diagnostics[0].message.contains("频率秒数不能为 0"));
}

#[test]
fn test_validate_fix_rate_job_valid() {
    let analyzer = MacroAnalyzer::new();
    let job = JobMacro::FixRate {
        seconds: 10,
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Job(job));

    // 有效的 FixRate 任务不应该产生诊断
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_validate_all_macro_types() {
    let analyzer = MacroAnalyzer::new();

    // 测试所有宏类型的验证功能
    let macros = vec![
        SummerMacro::DeriveService(ServiceMacro {
            struct_name: "TestService".to_string(),
            fields: vec![],
            scope: ServiceScope::Singleton,
            range: test_range(),
        }),
        SummerMacro::Inject(InjectMacro {
            inject_type: InjectType::Component,
            component_name: None,
            range: test_range(),
        }),
        SummerMacro::AutoConfig(AutoConfigMacro {
            configurator_type: "TestConfigurator".to_string(),
            range: test_range(),
        }),
        SummerMacro::Route(RouteMacro {
            path: "/test".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "test_handler".to_string(),
            range: test_range(),
            is_openapi: false,
        }),
        SummerMacro::Job(JobMacro::Cron {
            expression: "0 0 * * * *".to_string(),
            range: test_range(),
        }),
    ];

    for macro_item in macros {
        let diagnostics = analyzer.validate_macro(&macro_item);
        // 所有有效的宏都不应该产生诊断
        assert_eq!(diagnostics.len(), 0);
    }
}

#[test]
fn test_validate_comprehensive_example() {
    // 综合测试：验证完整的宏验证功能
    let analyzer = MacroAnalyzer::new();
    let uri = Url::parse("file:///test.rs").unwrap();

    let content = r#"
        #[derive(Clone, Service)]
        struct UserService {
            #[inject(component = "primary")]
            db: ConnectPool,
            
            #[inject(config)]
            config: UserConfig,
        }
        
        #[get("/users/{id}")]
        async fn get_user(id: i64) -> Result<Json<User>> {
            Ok(Json(User::default()))
        }
        
        #[auto_config(WebConfigurator)]
        #[tokio::main]
        async fn main() {
            App::new().run().await
        }
        
        #[cron("0 0 * * * *")]
        async fn hourly_cleanup() {
            println!("Cleanup");
        }
    "#
    .to_string();

    // 解析并提取宏
    let doc = analyzer.parse(uri, content).unwrap();
    let result = analyzer.extract_macros(doc).unwrap();

    // 验证所有宏
    for macro_item in &result.macros {
        let diagnostics = analyzer.validate_macro(macro_item);

        // 所有有效的宏都不应该产生诊断
        assert_eq!(diagnostics.len(), 0);
    }
}

#[test]
fn test_validate_multiple_errors() {
    // 测试一个宏可以产生多个错误
    let analyzer = MacroAnalyzer::new();
    let route = RouteMacro {
        path: "users".to_string(), // 不以 / 开头
        methods: vec![],           // 没有方法
        middlewares: vec![],
        handler_name: "".to_string(), // 空处理器名称
        is_openapi: false,
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::Route(route));

    // 应该产生多个错误诊断
    assert!(diagnostics.len() >= 3);
}

#[test]
fn test_validate_diagnostic_structure() {
    // 测试诊断信息的结构
    let analyzer = MacroAnalyzer::new();
    let auto_config = AutoConfigMacro {
        configurator_type: "".to_string(),
        range: test_range(),
    };

    let diagnostics = analyzer.validate_macro(&SummerMacro::AutoConfig(auto_config));

    assert_eq!(diagnostics.len(), 1);

    let diagnostic = &diagnostics[0];

    // 验证诊断信息的各个字段
    assert_eq!(
        diagnostic.severity,
        Some(lsp_types::DiagnosticSeverity::ERROR)
    );
    assert!(diagnostic.code.is_some());
    assert_eq!(diagnostic.source, Some("summer-lsp".to_string()));
    assert!(!diagnostic.message.is_empty());
}

#[test]
fn test_validate_error_codes() {
    // 测试不同错误有不同的错误代码
    let analyzer = MacroAnalyzer::new();

    let route1 = RouteMacro {
        path: "".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "handler".to_string(),
        range: test_range(),
        is_openapi: false,
    };

    let route2 = RouteMacro {
        path: "users".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "handler".to_string(),
        range: test_range(),
        is_openapi: false,
    };

    let diagnostics1 = analyzer.validate_macro(&SummerMacro::Route(route1));
    let diagnostics2 = analyzer.validate_macro(&SummerMacro::Route(route2));

    // 不同的错误应该有不同的错误代码
    assert!(!diagnostics1.is_empty());
    assert!(!diagnostics2.is_empty());

    if let (Some(code1), Some(code2)) = (&diagnostics1[0].code, &diagnostics2[0].code) {
        assert_ne!(code1, code2);
    }
}
