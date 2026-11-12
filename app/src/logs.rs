use std::path::Path;
use std::process::Stdio;

use axum::Json;
use axum::extract::Query;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::open_behaviour::ResolveError;

const CONFIG_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../conf/log-server.json");
const ENV_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../.env");
const PAGE_SIZES: [usize; 4] = [50, 100, 500, 1000];
const DAY_LIMIT: usize = 1000;
const FETCH_SECONDS: &str = "20";

#[derive(Serialize)]
pub struct LogApp {
    id: String,
    name: String,
}

#[derive(Deserialize)]
pub struct LogsRequest {
    app: String,
    date: String,
    page: usize,
    page_size: Option<usize>,
    filters: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct Filter {
    pub(crate) field: String,
    pub(crate) value: String,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct LogsPage {
    page: usize,
    page_size: usize,
    pub(crate) total: usize,
    pub(crate) events: Vec<Value>,
}

struct Config {
    base_url: String,
    key_name: String,
    apps: Vec<(String, String)>,
}

pub async fn apps() -> Result<Json<Vec<LogApp>>, ResolveError> {
    let config = read_config(Path::new(CONFIG_FILE)).map_err(ResolveError::Failed)?;
    Ok(Json(
        config
            .apps
            .into_iter()
            .map(|(id, _)| LogApp {
                name: id.clone(),
                id,
            })
            .collect(),
    ))
}

pub async fn load(Query(req): Query<LogsRequest>) -> Result<Json<LogsPage>, ResolveError> {
    let bad = |msg: &str| ResolveError::BadItem(msg.to_string());
    if req.page == 0 {
        return Err(bad("Bad page."));
    }
    if !valid_date(&req.date) {
        return Err(bad("Bad date."));
    }
    let page_size = req.page_size.unwrap_or(PAGE_SIZES[0]);
    if !PAGE_SIZES.contains(&page_size) {
        return Err(bad("Bad page size."));
    }
    let filters: Vec<Filter> = match &req.filters {
        Some(text) => serde_json::from_str(text).map_err(|_| bad("Bad filters."))?,
        None => Vec::new(),
    };
    let config = read_config(Path::new(CONFIG_FILE)).map_err(ResolveError::Failed)?;
    let server_app = config
        .apps
        .iter()
        .find(|(id, _)| *id == req.app)
        .map(|(_, name)| name)
        .ok_or_else(|| bad("Unknown app."))?;
    let key =
        read_env_value(Path::new(ENV_FILE), &config.key_name).map_err(ResolveError::Failed)?;
    let url = format!(
        "{}/log/{}?date={}&limit={DAY_LIMIT}",
        config.base_url.trim_end_matches('/'),
        server_app,
        req.date
    );
    let body = fetch(&url, &key).await.map_err(ResolveError::Failed)?;
    let events = parse_events(&body).map_err(ResolveError::Failed)?;
    Ok(Json(page_of(events, &filters, req.page, page_size)))
}

pub(crate) fn valid_date(date: &str) -> bool {
    date.len() == 10
        && date.char_indices().all(|(i, c)| {
            if i == 4 || i == 7 {
                c == '-'
            } else {
                c.is_ascii_digit()
            }
        })
}

fn read_config(path: &Path) -> Result<Config, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read conf/log-server.json: {e}"))?;
    let json: Value = serde_json::from_str(&text)
        .map_err(|e| format!("failed to parse conf/log-server.json: {e}"))?;
    let field = |name: &str| {
        json[name]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| format!("conf/log-server.json has no {name}"))
    };
    let key_name = field("api_key")?
        .strip_prefix("env:")
        .map(str::to_string)
        .ok_or("api_key in conf/log-server.json is not env:NAME")?;
    let apps = json["apps"]
        .as_object()
        .ok_or("conf/log-server.json has no apps")?;
    let apps = apps
        .iter()
        .filter_map(|(id, name)| Some((id.clone(), name.as_str()?.to_string())))
        .collect();
    Ok(Config {
        base_url: field("base_url")?,
        key_name,
        apps,
    })
}

pub(crate) fn read_env_value(path: &Path, name: &str) -> Result<String, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("failed to read .env: {e}"))?;
    let value = text.lines().find_map(|line| {
        let line = line.trim().trim_start_matches("export ").trim_start();
        let value = line.strip_prefix(name)?.trim_start().strip_prefix('=')?;
        Some(value.trim().trim_matches(['"', '\'']).to_string())
    });
    value
        .filter(|v| !v.is_empty())
        .ok_or_else(|| format!("{name} has no value in .env"))
}

// the key goes to curl on stdin, so it is not in the process list
async fn fetch(url: &str, key: &str) -> Result<String, String> {
    let mut child = Command::new("curl")
        .args([
            "-sS",
            "--max-time",
            FETCH_SECONDS,
            "-H",
            "@-",
            "-w",
            "\n%{http_code}",
            url,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to start curl: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(format!("Authorization: Bearer {key}\n").as_bytes())
            .await
            .map_err(|e| format!("failed to send the key to curl: {e}"))?;
    }
    let out = child
        .wait_with_output()
        .await
        .map_err(|e| format!("curl failed: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "failed to reach the log server: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let (body, status) = text
        .rsplit_once('\n')
        .ok_or("the log server gave no status")?;
    match status {
        "200" => Ok(body.to_string()),
        "401" => Err("The log server key is not correct.".to_string()),
        "403" => Err("The log server key cannot read this app.".to_string()),
        _ => Err(format!("The log server returned {status}: {}", body.trim())),
    }
}

pub(crate) fn parse_events(body: &str) -> Result<Vec<Value>, String> {
    let json: Value = serde_json::from_str(body)
        .map_err(|e| format!("failed to parse the log server response: {e}"))?;
    json["events"]
        .as_array()
        .cloned()
        .ok_or_else(|| "the log server response has no events".to_string())
}

// the log server gives the events of a day in the stored order, and the page shows the newest first
pub(crate) fn page_of(
    mut events: Vec<Value>,
    filters: &[Filter],
    page: usize,
    page_size: usize,
) -> LogsPage {
    events.reverse();
    events.retain(|event| filters.iter().all(|filter| matches(event, filter)));
    let total = events.len();
    let events = events
        .into_iter()
        .skip((page - 1) * page_size)
        .take(page_size)
        .collect();
    LogsPage {
        page,
        page_size,
        total,
        events,
    }
}

// a field is a path of keys from the event, such as `info.body.limit`, and the value is the text of the field.
// a key can hold dots, such as `body.limit` in `info`, so the path can match more than one key at each level
fn matches(event: &Value, filter: &Filter) -> bool {
    match_path(event, &filter.field, &filter.value)
}

fn match_path(value: &Value, path: &str, expected: &str) -> bool {
    if path.is_empty() {
        return match value {
            Value::Null => false,
            Value::String(text) => text == expected,
            other => {
                let text = other.to_string();
                text == expected
            }
        };
    }
    let parsed;
    let value = match value {
        Value::String(text) => match serde_json::from_str(text) {
            Ok(json) => {
                parsed = json;
                &parsed
            }
            Err(_) => return false,
        },
        other => other,
    };
    let Value::Object(map) = value else {
        return false;
    };
    map.iter().any(|(key, child)| {
        if path == key {
            return match_path(child, "", expected);
        }
        path.strip_prefix(key.as_str())
            .and_then(|rest| rest.strip_prefix('.'))
            .is_some_and(|rest| match_path(child, rest, expected))
    })
}
