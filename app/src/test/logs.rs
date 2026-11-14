use serde_json::{Value, json};

use crate::logs::*;

fn events(count: usize) -> Vec<Value> {
    (0..count).map(|n| json!({ "n": n })).collect()
}

#[test]
fn page_of_gives_the_newest_events_first() {
    let page = page_of(events(120), &[], 1, 50);
    assert_eq!((page.total, page.events.len()), (120, 50));
    assert_eq!(page.events[0], json!({ "n": 119 }));
    let last = page_of(events(120), &[], 3, 50);
    assert_eq!(last.events.len(), 20);
    assert_eq!(last.events[19], json!({ "n": 0 }));
    assert!(page_of(events(120), &[], 4, 50).events.is_empty());
}

#[test]
fn page_of_uses_the_page_size_and_the_filters() {
    assert_eq!(page_of(events(120), &[], 2, 100).events.len(), 20);
    let filters = [
        Filter {
            field: "info.code".to_string(),
            value: "5".to_string(),
        },
        Filter {
            field: "level".to_string(),
            value: "warn".to_string(),
        },
    ];
    let all = vec![
        json!({ "level": "warn", "info": { "code": 5 } }),
        json!({ "level": "info", "info": { "code": 5 } }),
        json!({ "level": "warn", "info": { "code": 6 } }),
        json!({ "level": "warn" }),
        json!({ "level": "warn", "info": { "code": 5, "body.query.limit": "48" } }),
    ];
    let page = page_of(all.clone(), &filters, 1, 50);
    assert_eq!((page.total, page.events.len()), (2, 2));
    let dotted = [Filter {
        field: "info.body.query.limit".to_string(),
        value: "48".to_string(),
    }];
    assert_eq!(page_of(all, &dotted, 1, 50).total, 1);
    let text = vec![
        json!({ "info": "{\"code\":5}" }),
        json!({ "info": "not json" }),
        json!({ "info": { "code": null } }),
    ];
    assert_eq!(page_of(text, &filters[..1], 1, 50).total, 1);
}

#[test]
fn valid_date_checks_the_format() {
    assert!(valid_date("2026-10-31"));
    for date in ["2026-1-31", "2026/10/31", "20261031xx", "../../etc", ""] {
        assert!(!valid_date(date), "{date}");
    }
}

#[test]
fn read_env_value_finds_the_name() {
    let path = std::env::temp_dir().join("agentic-workspace-test-logs.env");
    std::fs::write(&path, "OTHER=1\nexport APP_KEY=\"abc\"\nAPP_KEY_2=x\n").unwrap();
    assert_eq!(read_env_value(&path, "APP_KEY").unwrap(), "abc");
    assert!(read_env_value(&path, "MISSING").is_err());
    std::fs::write(&path, "APP_KEY=\nAPP_KEY_2=''\n").unwrap();
    assert!(read_env_value(&path, "APP_KEY").is_err());
    assert!(read_env_value(&path, "APP_KEY_2").is_err());
}

#[test]
fn read_config_reads_each_server_in_file_order() {
    let path = std::env::temp_dir().join("agentic-workspace-test-logs-config.json");
    std::fs::write(
        &path,
        r#"{
          "one": {
            "base_url": "https://one.example/",
            "route": "GET /log/{app}",
            "api_key": "env:APP_KEY",
            "apps": { "Alpha": "alpha", "Skip": 1, "Beta": "beta" }
          },
          "two": {
            "base_url": "https://two.example",
            "api_key": "env:OTHER_KEY",
            "apps": {}
          }
        }"#,
    )
    .unwrap();
    let config = read_config(&path).unwrap();
    assert_eq!(config.servers.len(), 2);
    assert_eq!(config.servers[0].name, "one");
    assert_eq!(config.servers[0].base_url, "https://one.example/");
    assert_eq!(config.servers[0].key_name, "APP_KEY");
    assert_eq!(
        config.servers[0].apps,
        vec![
            ("Alpha".to_string(), "alpha".to_string()),
            ("Beta".to_string(), "beta".to_string())
        ]
    );
    assert_eq!(config.servers[1].name, "two");
    assert_eq!(config.servers[1].key_name, "OTHER_KEY");
    assert!(config.servers[1].apps.is_empty());
}

#[test]
fn read_config_rejects_a_bad_file() {
    let path = std::env::temp_dir().join("agentic-workspace-test-logs-config-bad.json");
    for text in [
        "[]",
        r#"{"one":"no"}"#,
        r#"{"one":{"base_url":"https://one.example/","api_key":"APP_KEY","apps":{}}}"#,
        r#"{"one":{"base_url":"https://one.example/","api_key":"env:","apps":{}}}"#,
        r#"{"one":{"api_key":"env:APP_KEY","apps":{}}}"#,
        r#"{"one":{"base_url":"https://one.example/","api_key":"env:APP_KEY"}}"#,
    ] {
        std::fs::write(&path, text).unwrap();
        assert!(read_config(&path).is_err(), "{text}");
    }
}

#[test]
fn parse_events_needs_the_events_list() {
    assert_eq!(parse_events(r#"{"events":[{"a":1}]}"#).unwrap().len(), 1);
    assert!(parse_events(r#"{"x":1}"#).is_err());
    assert!(parse_events("not json").is_err());
}
