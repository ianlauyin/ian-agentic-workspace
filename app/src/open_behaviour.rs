use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Mutex;

use axum::Json;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tokio::process::Command;

const WORKSPACE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
const PROJECTS_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../conf/projects.json");
const FILE_PATH: &str = "spec/open-behaviour.json";
const LOG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/resolve");
const ENV_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../.env");
const API_AGENTS: &str = "https://api.cursor.com/v1/agents";
const API_KEY_VAR: &str = "CURSOR_API_KEY";
const AGENT_MODEL: &str = "claude-opus-5-5";
const SUBAGENT: &str = "target-repo";
const TOOL_DETAILS: [&str; 6] = ["command", "path", "pattern", "globPattern", "query", "url"];
const LOCAL_BIN: &str = "cursor-agent";
const LOCAL_MODEL: &str = "claude-opus-5-5-medium";
const LOCAL_START: &str = "The cloud agent failed. A local agent resolves the item now.";

static LOCAL_LOCK: Mutex<()> = Mutex::new(());

#[derive(Serialize)]
pub struct Tab {
    id: String,
    name: String,
    groups: Vec<Value>,
    errors: Vec<String>,
}

#[derive(Deserialize)]
pub struct ResolveRequest {
    tab: String,
    spec: String,
    id: u64,
    decision: String,
}

#[derive(Serialize)]
pub struct ResolveStart {
    chat: String,
}

#[derive(Deserialize)]
pub struct ProgressRequest {
    chat: String,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Progress {
    status: &'static str,
    steps: Vec<Step>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Step {
    kind: &'static str,
    text: String,
}

pub struct LoadError(String);

impl IntoResponse for LoadError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": self.0 }))).into_response()
    }
}

pub enum ResolveError {
    BadItem(String),
    Failed(String),
}

impl IntoResponse for ResolveError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ResolveError::BadItem(e) => (StatusCode::BAD_REQUEST, e),
            ResolveError::Failed(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

pub async fn load() -> Result<Json<Vec<Tab>>, LoadError> {
    let home = std::env::var("HOME").unwrap_or_default();
    load_tabs(Path::new(PROJECTS_FILE), &home).map(Json).map_err(LoadError)
}

pub async fn resolve(Json(req): Json<ResolveRequest>) -> Result<Json<ResolveStart>, ResolveError> {
    let home = std::env::var("HOME").unwrap_or_default();
    let decision = req.decision.trim();
    if decision.is_empty() {
        return Err(ResolveError::BadItem("The item has no final decision.".to_string()));
    }
    let projects = read_projects(Path::new(PROJECTS_FILE)).map_err(ResolveError::Failed)?;
    let (root, item) = find_item(&projects, &home, &req)?;
    let workspace = fs::canonicalize(WORKSPACE_DIR).map_err(|e| ResolveError::Failed(format!("failed to open the workspace: {e}")))?;
    let chat = start_agent(&workspace, &root, &req, &item, decision).await.map_err(ResolveError::Failed)?;
    Ok(Json(ResolveStart { chat }))
}

pub async fn progress(Query(req): Query<ProgressRequest>) -> Result<Json<Progress>, ResolveError> {
    if !valid_chat(&req.chat) {
        return Err(ResolveError::BadItem("Bad chat id.".to_string()));
    }
    let path = Path::new(LOG_DIR).join(format!("{}.log", req.chat));
    let log = read_log(&path);
    if req.chat.starts_with("bc-")
        && !log_has_end(&log)
        && !log.contains(LOCAL_START)
        && let Some((status, text)) = cloud_run(&req.chat).await
    {
        end_cloud_run(&path, &status, &text).map_err(ResolveError::Failed)?;
    }
    if !path.is_file() {
        return Err(ResolveError::BadItem("The chat has no log.".to_string()));
    }
    Ok(Json(read_progress(&read_log(&path))))
}

async fn start_agent(workspace: &Path, target: &Path, req: &ResolveRequest, item: &Value, decision: &str) -> Result<String, String> {
    let key = api_key()?;
    let workspace_repo = repo_remote(workspace).await?;
    let target_repo = repo_remote(target).await?;
    let body = create_body(req, item, decision, &workspace_repo, &target_repo);
    let created = post_json(&http()?, &key, API_AGENTS, &body).await?;
    let chat = created["agent"]["id"]
        .as_str()
        .filter(|id| valid_chat(id))
        .ok_or_else(|| "failed to start the Cursor agent: no agent id".to_string())?;
    let run = created["run"]["id"].as_str().unwrap_or("").to_string();
    let chat = chat.to_string();
    fs::create_dir_all(LOG_DIR).map_err(|e| format!("failed to create {LOG_DIR}: {e}"))?;
    let log_path = Path::new(LOG_DIR).join(format!("{chat}.log"));
    fs::File::create(&log_path).map_err(|e| format!("failed to create {}: {e}", log_path.display()))?;
    let prompt_path = log_path.with_extension("prompt");
    fs::write(&prompt_path, local_prompt(req, target, item, decision)).map_err(|e| format!("failed to create {}: {e}", prompt_path.display()))?;
    if valid_chat(&run) {
        let stream_chat = chat.clone();
        let stream_key = key.clone();
        tokio::spawn(async move {
            if let Err(error) = stream_run(&stream_key, &stream_chat, &run, &log_path).await {
                let error: String = safe_text(&stream_key, &error).chars().take(300).collect();
                let _ = append_line(&log_path, &error);
            }
        });
    }
    Ok(chat)
}

fn valid_chat(chat: &str) -> bool {
    !chat.is_empty() && chat.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn create_body(req: &ResolveRequest, item: &Value, decision: &str, workspace: &(String, String), target: &(String, String)) -> Value {
    let mut repos = vec![json!({ "url": workspace.0, "startingRef": workspace.1 })];
    if target.0 != workspace.0 {
        repos.push(json!({ "url": target.0, "startingRef": target.1 }));
    }
    let name: String = format!("Resolve {} {} #{}", req.tab, req.spec, req.id).chars().take(100).collect();
    json!({
        "name": name,
        "prompt": { "text": resolve_prompt(req, item, decision, &workspace.0, &target.0) },
        "model": { "id": AGENT_MODEL, "params": [{ "id": "effort", "value": "medium" }] },
        "repos": repos,
        "autoCreatePR": false,
        "customSubagents": [{
            "name": SUBAGENT,
            "description": "Edits the target repository. Always use for the code and spec change that resolves the open item.",
            "prompt": subagent_prompt(&workspace.0, &target.0),
            "model": "inherit"
        }]
    })
}

fn subagent_prompt(workspace_url: &str, target_url: &str) -> String {
    format!(
        "You edit only the target repository `{target_url}`.\n\
         Do not edit the workspace repository `{workspace_url}`.\n\
         Obey `AGENTS.md` in the target repository.\n\
         Change the code and `{FILE_PATH}` in the same change.\n\
         When an open item is resolved, delete that item from `{FILE_PATH}`."
    )
}

async fn repo_remote(repo: &Path) -> Result<(String, String), String> {
    let remote = git(repo, &["remote", "get-url", "origin"]).await?;
    let url = https_repo_url(&remote)?;
    let head = git(repo, &["rev-parse", "--abbrev-ref", "origin/HEAD"]).await?;
    let branch = branch_name(&head);
    if url.is_empty() || branch.is_empty() {
        return Err(format!("failed to read the origin remote of {}", repo.display()));
    }
    Ok((url, branch))
}

async fn git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .await
        .map_err(|e| format!("failed to read the origin remote of {}: {e}", repo.display()))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        let detail = detail.lines().map(str::trim).find(|line| !line.is_empty()).unwrap_or("git failed");
        return Err(format!("failed to read the origin remote of {}: {detail}", repo.display()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn https_repo_url(remote: &str) -> Result<String, String> {
    let remote = remote.trim().trim_end_matches('/').trim_end_matches(".git");
    if let Some(rest) = remote.strip_prefix("git@") {
        let Some((host, path)) = rest.split_once(':') else {
            return Err(format!("failed to read the origin remote: {remote}"));
        };
        return Ok(format!("https://{host}/{path}"));
    }
    if let Some(rest) = remote.strip_prefix("ssh://git@") {
        return Ok(format!("https://{rest}"));
    }
    if remote.starts_with("https://") || remote.starts_with("http://") {
        return Ok(remote.to_string());
    }
    Err(format!("failed to read the origin remote: {remote}"))
}

fn branch_name(value: &str) -> String {
    let value = value.trim();
    value.strip_prefix("origin/").unwrap_or(value).to_string()
}

fn api_key() -> Result<String, String> {
    if let Ok(value) = std::env::var(API_KEY_VAR) {
        let value = value.trim().to_string();
        if !value.is_empty() {
            return Ok(value);
        }
    }
    if let Ok(text) = fs::read_to_string(ENV_FILE)
        && let Some(value) = env_file_value(&text, API_KEY_VAR)
    {
        return Ok(value);
    }
    Err(format!("{API_KEY_VAR} is not set."))
}

fn env_file_value(text: &str, name: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((key, value)) = line.split_once('=') else { continue };
        if key.trim() != name {
            continue;
        }
        let value = unquote(value.trim());
        if !value.is_empty() {
            return Some(value);
        }
    }
    None
}

fn unquote(value: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() >= 2 && (bytes[0] == b'"' || bytes[0] == b'\'') && bytes[0] == bytes[bytes.len() - 1] {
        return value[1..value.len() - 1].to_string();
    }
    value.to_string()
}

fn http() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("failed to start the Cursor agent: {e}"))
}

async fn post_json(client: &reqwest::Client, key: &str, url: &str, body: &Value) -> Result<Value, String> {
    let response = client
        .post(url)
        .bearer_auth(key)
        .json(body)
        .send()
        .await
        .map_err(|e| api_fail(key, &e.to_string()))?;
    let status = response.status();
    let text = response.text().await.map_err(|e| api_fail(key, &e.to_string()))?;
    if !status.is_success() {
        return Err(api_fail(key, &format!("{status} {}", api_message(&text))));
    }
    serde_json::from_str(&text).map_err(|_| "failed to start the Cursor agent: bad response".to_string())
}

async fn get_json(client: &reqwest::Client, key: &str, url: &str) -> Result<Value, String> {
    let response = client.get(url).bearer_auth(key).send().await.map_err(|e| e.to_string())?;
    let status = response.status();
    let text = response.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(status.to_string());
    }
    serde_json::from_str(&text).map_err(|_| "bad response".to_string())
}

fn api_message(body: &str) -> String {
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return String::new();
    };
    let message = value["error"]["message"].as_str().or_else(|| value["message"].as_str()).unwrap_or("");
    message.chars().take(300).collect()
}

fn api_fail(key: &str, detail: &str) -> String {
    format!("failed to start the Cursor agent: {}", safe_text(key, detail))
}

fn safe_text(key: &str, text: &str) -> String {
    if key.is_empty() { text.to_string() } else { text.replace(key, "***") }
}

async fn cloud_run(chat: &str) -> Option<(String, String)> {
    let key = api_key().ok()?;
    let client = http().ok()?;
    let agent = get_json(&client, &key, &format!("{API_AGENTS}/{chat}")).await.ok()?;
    let run_id = agent["latestRunId"].as_str()?;
    if !valid_chat(run_id) {
        return None;
    }
    let run = get_json(&client, &key, &format!("{API_AGENTS}/{chat}/runs/{run_id}")).await.ok()?;
    Some((run["status"].as_str().unwrap_or("").to_string(), run["result"].as_str().unwrap_or("").to_string()))
}

fn end_cloud_run(log_path: &Path, status: &str, text: &str) -> Result<(), String> {
    if status == "ERROR" && start_local(log_path) {
        return Ok(());
    }
    match end_line(status, text) {
        Some(line) => append_end(log_path, &line),
        None => Ok(()),
    }
}

// the lock and the start line in the log keep one local agent for each chat, also after a server restart
fn start_local(log_path: &Path) -> bool {
    let Ok(_guard) = LOCAL_LOCK.lock() else { return false };
    if read_log(log_path).contains(LOCAL_START) {
        return true;
    }
    let Ok(prompt) = fs::read_to_string(log_path.with_extension("prompt")) else { return false };
    match spawn_local(&prompt, log_path) {
        Ok(()) => true,
        Err(error) => {
            let _ = append_line(log_path, &error);
            false
        }
    }
}

fn spawn_local(prompt: &str, log_path: &Path) -> Result<(), String> {
    let workspace = fs::canonicalize(WORKSPACE_DIR).map_err(|e| format!("failed to open the workspace: {e}"))?;
    append_line(log_path, LOCAL_START)?;
    let log = fs::OpenOptions::new().append(true).open(log_path).map_err(|e| format!("failed to open {}: {e}", log_path.display()))?;
    let err_log = log.try_clone().map_err(|e| format!("failed to open {}: {e}", log_path.display()))?;
    let mut child = Command::new(LOCAL_BIN)
        .args(["--print", "--output-format", "stream-json", "--force", "--trust", "--model", LOCAL_MODEL, "--workspace"])
        .arg(&workspace)
        .arg(prompt)
        .current_dir(&workspace)
        .stdin(Stdio::null())
        .stdout(log)
        .stderr(err_log)
        // a new process group keeps the agent alive when Ctrl+C stops the server
        .process_group(0)
        .spawn()
        .map_err(|e| format!("failed to start {LOCAL_BIN}: {e}"))?;
    let log_path = log_path.to_path_buf();
    tokio::spawn(async move {
        let code = child.wait().await.ok().and_then(|status| status.code());
        let _ = append_line(&log_path, &json!({ "type": "exit", "code": code }).to_string());
    });
    Ok(())
}

fn end_line(status: &str, text: &str) -> Option<String> {
    let text = if status == "ERROR" && text.is_empty() { "The agent failed." } else { text };
    match status {
        "FINISHED" => Some(json!({ "type": "result", "is_error": false, "result": text }).to_string()),
        "ERROR" => Some(json!({ "type": "result", "is_error": true, "result": text }).to_string()),
        "CANCELLED" | "EXPIRED" => Some(json!({ "type": "exit", "code": null }).to_string()),
        _ => None,
    }
}

async fn stream_run(key: &str, chat: &str, run: &str, log_path: &Path) -> Result<(), String> {
    let response = reqwest::Client::new()
        .get(format!("{API_AGENTS}/{chat}/runs/{run}/stream"))
        .bearer_auth(key)
        .header("Accept", "text/event-stream")
        .send()
        .await
        .map_err(|e| format!("failed to read the progress stream: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("failed to read the progress stream: {}", response.status()));
    }
    let mut response = response;
    let mut pending = String::new();
    let mut assistant = String::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| format!("failed to read the progress stream: {e}"))? {
        pending.push_str(&String::from_utf8_lossy(&chunk));
        pending = pending.replace("\r\n", "\n");
        while let Some(at) = pending.find("\n\n") {
            let block = pending[..at].to_string();
            pending.drain(..at + 2);
            apply_stream_block(&block, log_path, &mut assistant)?;
        }
    }
    flush_assistant(log_path, &mut assistant)
}

fn apply_stream_block(block: &str, log_path: &Path, assistant: &mut String) -> Result<(), String> {
    let (event, data) = sse_parts(block);
    let Some(event) = event else { return Ok(()) };
    let value: Value = serde_json::from_str(&data).unwrap_or(Value::Null);
    match event {
        "assistant" => {
            if let Some(text) = value["text"].as_str() {
                assistant.push_str(text);
            }
        }
        "tool_call" if value["status"] == "running" => {
            flush_assistant(log_path, assistant)?;
            let text = cloud_tool_text(&value);
            append_line(log_path, &json!({ "type": "tool_call", "subtype": "started", "tool_call": { "function": { "name": text } } }).to_string())?;
        }
        "result" => {
            flush_assistant(log_path, assistant)?;
            end_cloud_run(log_path, value["status"].as_str().unwrap_or(""), value["text"].as_str().unwrap_or(""))?;
        }
        "error" => {
            flush_assistant(log_path, assistant)?;
            if let Some(message) = value["message"].as_str() {
                append_line(log_path, message)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn sse_parts(block: &str) -> (Option<&str>, String) {
    let mut event = None;
    let mut data = String::new();
    for line in block.lines() {
        if let Some(rest) = line.strip_prefix("event:") {
            event = Some(rest.trim());
        } else if let Some(rest) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(rest.trim_start());
        }
    }
    (event, data)
}

fn cloud_tool_text(data: &Value) -> String {
    let name = data["name"].as_str().unwrap_or("tool");
    match TOOL_DETAILS.iter().find_map(|field| data["args"][*field].as_str()) {
        Some(detail) => format!("{name}: {detail}"),
        None => name.to_string(),
    }
}

fn flush_assistant(log_path: &Path, assistant: &mut String) -> Result<(), String> {
    let text = assistant.trim().to_string();
    assistant.clear();
    if text.is_empty() {
        return Ok(());
    }
    append_line(log_path, &json!({ "type": "assistant", "message": { "content": [{ "type": "text", "text": text }] } }).to_string())
}

fn read_log(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn log_has_end(log: &str) -> bool {
    log.contains("\"type\":\"result\"") || log.contains("\"type\": \"result\"") || log.contains("\"type\":\"exit\"") || log.contains("\"type\": \"exit\"")
}

fn append_end(path: &Path, line: &str) -> Result<(), String> {
    if log_has_end(&read_log(path)) {
        return Ok(());
    }
    append_line(path, line)
}

fn append_line(path: &Path, line: &str) -> Result<(), String> {
    let mut file = fs::OpenOptions::new().create(true).append(true).open(path).map_err(|e| format!("failed to open {}: {e}", path.display()))?;
    writeln!(file, "{line}").map_err(|e| format!("failed to write {}: {e}", path.display()))
}

// a line that is not JSON is stderr text of the agent
fn read_progress(log: &str) -> Progress {
    let mut status = "running";
    let mut steps = Vec::new();
    for line in log.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let Ok(event) = serde_json::from_str::<Value>(line) else {
            steps.push(Step { kind: "output", text: line.to_string() });
            continue;
        };
        match (event["type"].as_str(), event["subtype"].as_str()) {
            (Some("assistant"), _) => {
                let text = message_text(&event);
                if !text.is_empty() {
                    steps.push(Step { kind: "message", text });
                }
            }
            (Some("tool_call"), Some("started")) => steps.push(Step { kind: "tool", text: tool_text(&event["tool_call"]) }),
            (Some("result"), _) if event["is_error"] == true => {
                status = "failed";
                if let Some(text) = event["result"].as_str() {
                    steps.push(Step { kind: "output", text: text.to_string() });
                }
            }
            (Some("result"), _) => status = "finished",
            (Some("exit"), _) if status == "running" => status = "stopped",
            _ => {}
        }
    }
    Progress { status, steps }
}

fn message_text(event: &Value) -> String {
    let blocks = event["message"]["content"].as_array().into_iter().flatten();
    let texts: Vec<&str> = blocks.filter(|block| block["type"] == "text").filter_map(|block| block["text"].as_str()).collect();
    texts.concat().trim().to_string()
}

fn tool_text(call: &Value) -> String {
    let tool = call.as_object().and_then(|call| call.iter().find(|(key, _)| key.ends_with("ToolCall")));
    let Some((key, tool)) = tool else {
        return call["function"]["name"].as_str().unwrap_or("tool").to_string();
    };
    let name = key.trim_end_matches("ToolCall");
    match TOOL_DETAILS.iter().find_map(|field| tool["args"][*field].as_str()) {
        Some(detail) => format!("{name}: {detail}"),
        None => name.to_string(),
    }
}

fn load_tabs(projects_file: &Path, home: &str) -> Result<Vec<Tab>, String> {
    let projects = read_projects(projects_file)?;
    Ok(projects.iter().map(|(id, project)| project_tab(id, project, home)).collect())
}

fn read_projects(projects_file: &Path) -> Result<Map<String, Value>, String> {
    let name = projects_file.display();
    let text = fs::read_to_string(projects_file).map_err(|e| format!("failed to read {name}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("failed to parse {name}: {e}"))
}

fn project_root(project: &Value, home: &str) -> Option<PathBuf> {
    let path = project["path"].as_str()?;
    Some(match path.strip_prefix("~/") {
        Some(rest) => Path::new(home).join(rest),
        None => PathBuf::from(path),
    })
}

fn find_item(projects: &Map<String, Value>, home: &str, req: &ResolveRequest) -> Result<(PathBuf, Value), ResolveError> {
    let bad = |msg: &str| ResolveError::BadItem(msg.to_string());
    let root = projects.get(&req.tab).and_then(|project| project_root(project, home)).ok_or_else(|| bad("Unknown project."))?;
    let groups = read_groups(&root).map_err(ResolveError::Failed)?;
    let item = groups
        .iter()
        .filter(|group| group["spec"] == req.spec.as_str())
        .flat_map(|group| group["items"].as_array().into_iter().flatten())
        .find(|item| item["id"].as_u64() == Some(req.id))
        .ok_or_else(|| bad("Unknown item."))?;
    Ok((root, item.clone()))
}

fn resolve_prompt(req: &ResolveRequest, item: &Value, decision: &str, workspace_url: &str, target_url: &str) -> String {
    let item = serde_json::to_string_pretty(item).unwrap_or_default();
    format!(
        "You are the agent for this workspace repository `{workspace_url}`.\n\
         Call the `{SUBAGENT}` subagent to resolve one open-behaviour item in the target repository `{target_url}`.\n\
         Do not edit the target repository yourself.\n\n\
         - File: `{FILE_PATH}`\n\
         - Spec: `{spec}`\n\
         - Item: #{id}\n\
         - Project: `{tab}`\n\n\
         The item:\n\n```json\n{item}\n```\n\n\
         The final decision of the user:\n\n{decision}\n\n\
         The subagent resolves the item as the final decision tells. When the item is resolved, the subagent deletes it from `{FILE_PATH}`. \
         The subagent obeys `AGENTS.md` in the target repository.",
        spec = req.spec,
        id = req.id,
        tab = req.tab,
    )
}

fn local_prompt(req: &ResolveRequest, root: &Path, item: &Value, decision: &str) -> String {
    let item = serde_json::to_string_pretty(item).unwrap_or_default();
    format!(
        "Resolve one open-behaviour item of the project `{tab}`. The project is at `{root}`.\n\n\
         - File: `{FILE_PATH}`\n\
         - Spec: `{spec}`\n\
         - Item: #{id}\n\n\
         The item:\n\n```json\n{item}\n```\n\n\
         The final decision of the user:\n\n{decision}\n\n\
         Resolve the item as the final decision tells. When the item is resolved, delete it from `{FILE_PATH}`. \
         Obey `AGENTS.md`.",
        tab = req.tab,
        root = root.display(),
        spec = req.spec,
        id = req.id,
    )
}

fn project_tab(id: &str, project: &Value, home: &str) -> Tab {
    let mut tab = Tab {
        id: id.to_string(),
        name: id.to_string(),
        groups: Vec::new(),
        errors: Vec::new(),
    };
    let Some(root) = project_root(project, home) else {
        tab.errors.push(format!("{id} has no path in conf/projects.json"));
        return tab;
    };
    match read_groups(&root) {
        Ok(groups) => tab.groups = groups,
        Err(e) => tab.errors.push(e),
    }
    tab
}

fn read_groups(root: &Path) -> Result<Vec<Value>, String> {
    let file = root.join(FILE_PATH);
    if !file.is_file() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&file).map_err(|e| format!("failed to read {FILE_PATH}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("failed to parse {FILE_PATH}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_home(name: &str) -> PathBuf {
        let home = std::env::temp_dir().join(format!("agentic-workspace-open-behaviour-{name}"));
        let _ = fs::remove_dir_all(&home);
        fs::create_dir_all(&home).unwrap();
        home
    }

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn projects(home: &Path, text: &str) -> PathBuf {
        let file = home.join("projects.json");
        write(&file, text);
        file
    }

    #[test]
    fn reads_only_the_spec_file_in_its_order() {
        let home = temp_home("order");
        write(
            &home.join("my-app/spec/open-behaviour.json"),
            r#"[{"spec":"web/01_b","items":[]},{"spec":"api/01_a","items":[]}]"#,
        );
        write(&home.join("my-app/spec/web/open-behaviour.json"), r#"[{"spec":"01_c","items":[]}]"#);
        let file = projects(&home, r#"{"my-app":{"path":"~/my-app"}}"#);

        let tabs = load_tabs(&file, home.to_str().unwrap()).unwrap();
        assert_eq!((tabs.len(), tabs[0].id.as_str(), tabs[0].name.as_str()), (1, "my-app", "my-app"));
        let specs: Vec<&str> = tabs[0].groups.iter().map(|g| g["spec"].as_str().unwrap()).collect();
        assert_eq!(specs, ["web/01_b", "api/01_a"]);
        assert!(tabs[0].errors.is_empty());
    }

    #[test]
    fn keeps_the_order_of_the_projects_file() {
        let home = temp_home("projects");
        let file = projects(&home, r#"{"zeta":{"path":"~/zeta"},"alpha":{"path":"~/alpha"}}"#);
        let tabs = load_tabs(&file, home.to_str().unwrap()).unwrap();
        let ids: Vec<&str> = tabs.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, ["zeta", "alpha"]);
        assert!(tabs.iter().all(|t| t.groups.is_empty() && t.errors.is_empty()));
    }

    #[test]
    fn bad_file_gives_a_tab_error_and_keeps_the_other_tabs() {
        let home = temp_home("bad");
        write(&home.join("my-app/spec/open-behaviour.json"), "not json");
        write(&home.join("good/spec/open-behaviour.json"), r#"[{"spec":"web/01_b","items":[]}]"#);
        let file = projects(&home, r#"{"my-app":{"path":"~/my-app"},"other":{},"good":{"path":"~/good"}}"#);

        let tabs = load_tabs(&file, home.to_str().unwrap()).unwrap();
        assert!(tabs[0].groups.is_empty());
        assert_eq!(tabs[0].errors.len(), 1);
        assert!(tabs[0].errors[0].starts_with("failed to parse spec/open-behaviour.json"));
        assert_eq!(tabs[1].errors, ["other has no path in conf/projects.json"]);
        assert_eq!((tabs[2].groups.len(), tabs[2].errors.len()), (1, 0));
    }

    #[test]
    fn missing_projects_file_is_an_error() {
        let home = temp_home("missing");
        assert!(load_tabs(&home.join("projects.json"), home.to_str().unwrap()).is_err());
    }

    fn resolve_request(tab: &str, spec: &str, id: u64) -> ResolveRequest {
        ResolveRequest {
            tab: tab.to_string(),
            spec: spec.to_string(),
            id,
            decision: "Use option 1.".to_string(),
        }
    }

    fn resolve_home(name: &str) -> (PathBuf, Map<String, Value>) {
        let home = temp_home(name);
        write(
            &home.join("my-app/spec/open-behaviour.json"),
            r#"[{"spec":"web/01_login","items":[{"id":3,"item":"Sign-in"}]}]"#,
        );
        let projects = read_projects(&projects(&home, r#"{"my-app":{"path":"~/my-app"}}"#)).unwrap();
        (home, projects)
    }

    #[test]
    fn find_item_returns_the_root_and_the_item() {
        let (home, projects) = resolve_home("find");
        let req = resolve_request("my-app", "web/01_login", 3);
        let Ok((root, item)) = find_item(&projects, home.to_str().unwrap(), &req) else {
            panic!("item not found");
        };
        assert_eq!(root, home.join("my-app"));
        assert_eq!(item, json!({ "id": 3, "item": "Sign-in" }));
    }

    #[test]
    fn find_item_rejects_an_unknown_project_spec_or_item() {
        let (home, projects) = resolve_home("find-bad");
        let home = home.to_str().unwrap();
        for req in [
            resolve_request("other", "web/01_login", 3),
            resolve_request("my-app", "web/02_pay", 3),
            resolve_request("my-app", "web/01_login", 4),
        ] {
            assert!(matches!(find_item(&projects, home, &req), Err(ResolveError::BadItem(_))), "{} {}", req.tab, req.spec);
        }
    }

    #[test]
    fn resolve_prompt_has_the_item_and_the_decision() {
        let req = resolve_request("my-app", "web/01_login", 3);
        let prompt = resolve_prompt(&req, &json!({ "id": 3 }), "Use option 1.", "https://github.com/acme/workspace", "https://github.com/acme/my-app");
        for part in [
            "`my-app`",
            "https://github.com/acme/workspace",
            "https://github.com/acme/my-app",
            "`spec/open-behaviour.json`",
            "`web/01_login`",
            "#3",
            "\"id\": 3",
            "Use option 1.",
            "AGENTS.md",
            "target-repo",
            "Do not edit the target repository yourself.",
        ] {
            assert!(prompt.contains(part), "{part}");
        }
    }

    #[test]
    fn create_body_puts_this_repo_first_and_the_target_subagent() {
        let req = resolve_request("my-app", "web/01_login", 3);
        let body = create_body(
            &req,
            &json!({ "id": 3 }),
            "Use option 1.",
            &("https://github.com/acme/workspace".to_string(), "main".to_string()),
            &("https://github.com/acme/my-app".to_string(), "main".to_string()),
        );
        assert_eq!(body["repos"][0]["url"], "https://github.com/acme/workspace");
        assert_eq!(body["repos"][1]["url"], "https://github.com/acme/my-app");
        assert_eq!(body["model"]["id"], AGENT_MODEL);
        assert_eq!(body["model"]["params"][0]["id"], "effort");
        assert_eq!(body["model"]["params"][0]["value"], "medium");
        assert_eq!(body["autoCreatePR"], false);
        assert_eq!(body["customSubagents"][0]["name"], "target-repo");
        assert!(body["customSubagents"][0]["prompt"].as_str().unwrap().contains("https://github.com/acme/my-app"));
    }

    #[test]
    fn create_body_sends_one_repo_when_the_urls_match() {
        let req = resolve_request("my-app", "web/01_login", 3);
        let repo = ("https://github.com/acme/my-app".to_string(), "main".to_string());
        let body = create_body(&req, &json!({ "id": 3 }), "Use option 1.", &repo, &repo);
        assert_eq!(body["repos"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn https_repo_url_accepts_git_and_https_remotes() {
        assert_eq!(https_repo_url("git@github.com:acme/my-app.git").unwrap(), "https://github.com/acme/my-app");
        assert_eq!(https_repo_url("https://github.com/acme/my-app.git").unwrap(), "https://github.com/acme/my-app");
        assert_eq!(https_repo_url("ssh://git@github.com/acme/my-app.git").unwrap(), "https://github.com/acme/my-app");
        assert!(https_repo_url("acme/my-app").is_err());
    }

    #[test]
    fn end_line_maps_the_cloud_run_status() {
        assert!(end_line("RUNNING", "").is_none());
        assert_eq!(end_line("FINISHED", "Done.").unwrap(), r#"{"type":"result","is_error":false,"result":"Done."}"#);
        assert!(end_line("ERROR", "").unwrap().contains("The agent failed."));
        assert_eq!(end_line("CANCELLED", "").unwrap(), r#"{"type":"exit","code":null}"#);
    }

    #[test]
    fn end_cloud_run_ends_the_log_when_no_local_agent_can_start() {
        let dir = temp_home("cloud-end");
        let failed = dir.join("bc-1.log");
        write(&failed, "");
        end_cloud_run(&failed, "ERROR", "").unwrap();
        assert_eq!(read_progress(&read_log(&failed)).status, "failed");

        let finished = dir.join("bc-2.log");
        write(&finished, "");
        write(&finished.with_extension("prompt"), "Resolve it.");
        end_cloud_run(&finished, "FINISHED", "Done.").unwrap();
        assert_eq!(read_progress(&read_log(&finished)).status, "finished");
    }

    #[test]
    fn end_cloud_run_keeps_a_started_local_agent() {
        let dir = temp_home("cloud-local");
        let log = dir.join("bc-1.log");
        write(&log, &format!("{LOCAL_START}\n"));
        write(&log.with_extension("prompt"), "Resolve it.");
        end_cloud_run(&log, "ERROR", "").unwrap();
        let progress = read_progress(&read_log(&log));
        assert_eq!(progress, Progress { status: "running", steps: vec![step("output", LOCAL_START)] });
    }

    #[test]
    fn local_prompt_has_the_project_path_and_the_decision() {
        let req = resolve_request("my-app", "web/01_login", 3);
        let prompt = local_prompt(&req, Path::new("/code/my-app"), &json!({ "id": 3 }), "Use option 1.");
        for part in ["`my-app`", "`/code/my-app`", "`web/01_login`", "#3", "\"id\": 3", "Use option 1.", "AGENTS.md"] {
            assert!(prompt.contains(part), "{part}");
        }
    }

    #[test]
    fn env_file_value_reads_one_named_value() {
        let text = "OTHER=secret\n# CURSOR_API_KEY=no\nexport CURSOR_API_KEY=\"cursor_test\"\n";
        assert_eq!(env_file_value(text, "CURSOR_API_KEY").as_deref(), Some("cursor_test"));
        assert_eq!(env_file_value("CURSOR_API_KEY=\n", "CURSOR_API_KEY"), None);
    }

    #[tokio::test]
    async fn repo_remote_reads_origin() {
        let dir = temp_home("remote");
        git_ok(&dir, &["init", "-b", "main", "--template="]);
        git_ok(&dir, &["remote", "add", "origin", "git@github.com:acme/my-app.git"]);
        git_ok(&dir, &["-c", "user.email=test@example.com", "-c", "user.name=test", "commit", "--allow-empty", "-m", "init"]);
        git_ok(&dir, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
        git_ok(&dir, &["symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/main"]);
        let (url, branch) = repo_remote(&dir).await.unwrap();
        assert_eq!(url, "https://github.com/acme/my-app");
        assert_eq!(branch, "main");
    }

    fn git_ok(dir: &Path, args: &[&str]) {
        let status = std::process::Command::new("git").args(args).current_dir(dir).status().unwrap();
        assert!(status.success(), "{args:?}");
    }

    fn step(kind: &'static str, text: &str) -> Step {
        Step { kind, text: text.to_string() }
    }

    #[test]
    fn read_progress_gives_the_messages_and_the_started_tools() {
        let log = [
            r#"{"type":"system","subtype":"init"}"#,
            r#"{"type":"user","message":{"content":[{"type":"text","text":"Resolve it."}]}}"#,
            r#"{"type":"tool_call","subtype":"started","tool_call":{"readToolCall":{"args":{"path":"/code/a.md"}}}}"#,
            r#"{"type":"tool_call","subtype":"completed","tool_call":{"readToolCall":{"args":{"path":"/code/a.md"},"result":{}}}}"#,
            r#"{"type":"tool_call","subtype":"started","tool_call":{"todoToolCall":{"args":{}}}}"#,
            r#"{"type":"tool_call","subtype":"started","tool_call":{"function":{"name":"mcp_search"}}}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Item #3 "},{"type":"text","text":"is resolved."}]}}"#,
        ]
        .join("\n");
        let progress = read_progress(&log);
        assert_eq!(progress.status, "running");
        assert_eq!(
            progress.steps,
            [step("tool", "read: /code/a.md"), step("tool", "todo"), step("tool", "mcp_search"), step("message", "Item #3 is resolved.")]
        );
    }

    #[test]
    fn read_progress_gives_the_status_of_the_end() {
        assert_eq!(read_progress(r#"{"type":"result","is_error":false,"result":"Done."}"#).status, "finished");
        assert_eq!(read_progress("{\"type\":\"result\",\"is_error\":false}\n{\"type\":\"exit\",\"code\":0}").status, "finished");
        assert_eq!(read_progress(r#"{"type":"exit","code":null}"#).status, "stopped");

        let failed = read_progress(r#"{"type":"result","is_error":true,"result":"Model error."}"#);
        assert_eq!(failed, Progress { status: "failed", steps: vec![step("output", "Model error.")] });
    }

    #[test]
    fn read_progress_keeps_a_line_that_is_not_json() {
        let progress = read_progress("Error: not logged in\n\n{\"type\":\"exit\",\"code\":1}");
        assert_eq!(progress, Progress { status: "stopped", steps: vec![step("output", "Error: not logged in")] });
    }
}
