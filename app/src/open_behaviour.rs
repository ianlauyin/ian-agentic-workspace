use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::Json;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tokio::process::Command;

pub const PROJECTS_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../conf/projects.json");
const FILE_PATH: &str = "spec/open-behaviour.json";
const LOG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/resolve");
const TOOL_DETAILS: [&str; 6] = ["command", "path", "pattern", "globPattern", "query", "url"];
const LOCAL_BIN: &str = "cursor-agent";
const LOCAL_MODEL: &str = "claude-sonnet-5-5-medium";
const GROK_MODEL: &str = "grok-4.7-high";
pub(crate) const GROK_START: &str = "The local agent failed. A local Grok agent takes over now.";
const FOLLOWUP_MARK: &str = "\"type\":\"followup\"";

static LOCAL_LOCK: Mutex<()> = Mutex::new(());

#[derive(Serialize)]
pub struct Tab {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) groups: Vec<Value>,
    pub(crate) errors: Vec<String>,
}

#[derive(Deserialize)]
pub struct ResolveRequest {
    pub(crate) tab: String,
    pub(crate) spec: String,
    pub(crate) id: u64,
    pub(crate) decision: String,
}

#[derive(Serialize)]
pub struct ResolveStart {
    chat: String,
}

impl ResolveStart {
    pub fn new(chat: String) -> Self {
        Self { chat }
    }
}

#[derive(Deserialize)]
pub struct ProgressRequest {
    chat: String,
}

#[derive(Deserialize)]
pub struct FollowupRequest {
    chat: String,
    text: String,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Progress {
    pub(crate) status: &'static str,
    pub(crate) steps: Vec<Step>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Step {
    pub(crate) kind: &'static str,
    pub(crate) text: String,
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
    let prompt = local_prompt(&req, &root, &item, decision);
    let chat = start_agent(&root, &prompt).map_err(ResolveError::Failed)?;
    Ok(Json(ResolveStart { chat }))
}

pub async fn progress(Query(req): Query<ProgressRequest>) -> Result<Json<Progress>, ResolveError> {
    if !valid_chat(&req.chat) {
        return Err(ResolveError::BadItem("Bad chat id.".to_string()));
    }
    let path = Path::new(LOG_DIR).join(format!("{}.log", req.chat));
    if !path.is_file() {
        return Err(ResolveError::BadItem("The chat has no log.".to_string()));
    }
    Ok(Json(read_progress(&read_log(&path))))
}

pub async fn followup(Json(req): Json<FollowupRequest>) -> Result<Json<Progress>, ResolveError> {
    let bad = |msg: &str| ResolveError::BadItem(msg.to_string());
    if !valid_chat(&req.chat) {
        return Err(bad("Bad chat id."));
    }
    let text = req.text.trim();
    if text.is_empty() {
        return Err(bad("The follow-up has no text."));
    }
    let path = Path::new(LOG_DIR).join(format!("{}.log", req.chat));
    if !path.is_file() {
        return Err(bad("The chat has no log."));
    }
    followup_local(&path, text)?;
    Ok(Json(read_progress(&read_log(&path))))
}

pub(crate) fn followup_line(text: &str) -> String {
    json!({ "type": "followup", "text": text }).to_string()
}

fn failed_end(text: &str) -> String {
    json!({ "type": "result", "is_error": true, "result": text }).to_string()
}

// the lock keeps one local agent for each chat when two follow-ups arrive at the same time
fn followup_local(log_path: &Path, text: &str) -> Result<(), ResolveError> {
    let failed = |msg: String| ResolveError::Failed(msg);
    let _guard = LOCAL_LOCK.lock().map_err(|_| failed("failed to lock the local agent".to_string()))?;
    let log = read_log(log_path);
    if !log_has_end(last_run(&log)) {
        return Err(ResolveError::BadItem("The agent is still working.".to_string()));
    }
    let session = local_session(&log).ok_or_else(|| failed("The local agent has no session.".to_string()))?;
    let model = if log.contains(GROK_START) { GROK_MODEL } else { LOCAL_MODEL };
    let root_path = log_path.with_extension("root");
    let root = fs::read_to_string(&root_path).map_err(|e| failed(format!("failed to read {}: {e}", root_path.display())))?;
    append_line(log_path, &followup_line(text)).map_err(failed)?;
    if let Err(error) = spawn_local(text, Path::new(root.trim()), log_path, model, Some(&session), false) {
        append_line(log_path, &error).map_err(failed)?;
        append_end(log_path, &failed_end("The follow-up failed.")).map_err(failed)?;
    }
    Ok(())
}

// the session of the Grok agent is after its start line
pub(crate) fn local_session(log: &str) -> Option<String> {
    let agent_log = log.rsplit_once(GROK_START).map_or(log, |(_, rest)| rest);
    agent_log
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter_map(|event| event["session_id"].as_str().map(str::to_string))
        .rfind(|id| valid_chat(id))
}

// when the first agent cannot start, the Grok agent starts; the start fails only when both cannot start
pub fn start_agent(root: &Path, prompt: &str) -> Result<String, String> {
    let chat = new_chat()?;
    fs::create_dir_all(LOG_DIR).map_err(|e| format!("failed to create {LOG_DIR}: {e}"))?;
    let log_path = Path::new(LOG_DIR).join(format!("{chat}.log"));
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&log_path)
        .map_err(|e| format!("failed to create {}: {e}", log_path.display()))?;
    let prompt_path = log_path.with_extension("prompt");
    fs::write(&prompt_path, prompt).map_err(|e| format!("failed to create {}: {e}", prompt_path.display()))?;
    let root_path = log_path.with_extension("root");
    fs::write(&root_path, root.to_string_lossy().as_bytes()).map_err(|e| format!("failed to create {}: {e}", root_path.display()))?;
    if let Err(error) = spawn_local(prompt, root, &log_path, LOCAL_MODEL, None, true) {
        append_line(&log_path, &error)?;
        if !start_local(&log_path, GROK_START, GROK_MODEL) {
            return Err(error);
        }
    }
    Ok(chat)
}

fn new_chat() -> Result<String, String> {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| format!("failed to read the time: {e}"))?;
    Ok(format!("local-{}", time.as_nanos()))
}

pub(crate) fn valid_chat(chat: &str) -> bool {
    !chat.is_empty() && chat.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

// the lock and the start line in the log keep one agent for each stage, also after a server restart
fn start_local(log_path: &Path, start: &str, model: &str) -> bool {
    let Ok(_guard) = LOCAL_LOCK.lock() else { return false };
    if read_log(log_path).contains(start) {
        return true;
    }
    if append_line(log_path, start).is_err() {
        return false;
    }
    let Ok(prompt) = fs::read_to_string(log_path.with_extension("prompt")) else { return false };
    let Ok(root) = fs::read_to_string(log_path.with_extension("root")) else { return false };
    match spawn_local(&prompt, Path::new(root.trim()), log_path, model, None, false) {
        Ok(()) => true,
        Err(error) => {
            let _ = append_line(log_path, &error);
            false
        }
    }
}

// a session id resumes the conversation of an earlier local agent
// fallback starts the Grok agent when this agent fails
fn spawn_local(prompt: &str, root: &Path, log_path: &Path, model: &str, session: Option<&str>, fallback: bool) -> Result<(), String> {
    let root = fs::canonicalize(root).map_err(|e| format!("failed to open {}: {e}", root.display()))?;
    let log = fs::OpenOptions::new().append(true).open(log_path).map_err(|e| format!("failed to open {}: {e}", log_path.display()))?;
    let err_log = log.try_clone().map_err(|e| format!("failed to open {}: {e}", log_path.display()))?;
    let mut command = Command::new(LOCAL_BIN);
    command
        .args(["--print", "--output-format", "stream-json", "--force", "--trust", "--model", model, "--workspace"])
        .arg(&root);
    if let Some(session) = session {
        command.args(["--resume", session]);
    }
    let mut child = command
        .arg(prompt)
        .current_dir(&root)
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
        if fallback {
            start_grok_after_fail(&log_path);
        }
    });
    Ok(())
}

// a follow-up run does not start the Grok agent
pub(crate) fn start_grok_after_fail(log_path: &Path) {
    let log = read_log(log_path);
    if !log.contains(FOLLOWUP_MARK) && local_failed(last_run(&log)) && !start_local(log_path, GROK_START, GROK_MODEL) {
        let _ = append_end(log_path, &failed_end("The agent failed."));
    }
}

// the first end of the run tells: a failed result, or an exit with an error code before a result
pub(crate) fn local_failed(run: &str) -> bool {
    for event in run.lines().filter_map(|line| serde_json::from_str::<Value>(line).ok()) {
        match event["type"].as_str() {
            Some("result") => return event["is_error"] == true,
            Some("exit") => return event["code"].as_i64().is_some_and(|code| code != 0),
            _ => {}
        }
    }
    false
}

pub(crate) fn read_log(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

pub(crate) fn log_has_end(log: &str) -> bool {
    log.contains("\"type\":\"result\"") || log.contains("\"type\": \"result\"") || log.contains("\"type\":\"exit\"") || log.contains("\"type\": \"exit\"")
}

// a follow-up and the Grok agent start a new run, so only the lines after the last start tell if the agent ended
pub(crate) fn last_run(log: &str) -> &str {
    let starts = [FOLLOWUP_MARK, GROK_START];
    match starts.iter().filter_map(|start| log.rfind(start)).max() {
        Some(at) => log[at..].split_once('\n').map_or("", |(_, rest)| rest),
        None => log,
    }
}

fn append_end(path: &Path, line: &str) -> Result<(), String> {
    if log_has_end(last_run(&read_log(path))) {
        return Ok(());
    }
    append_line(path, line)
}

fn append_line(path: &Path, line: &str) -> Result<(), String> {
    let mut file = fs::OpenOptions::new().create(true).append(true).open(path).map_err(|e| format!("failed to open {}: {e}", path.display()))?;
    writeln!(file, "{line}").map_err(|e| format!("failed to write {}: {e}", path.display()))
}

// a line that is not JSON is stderr text of the agent
// the first agent writes its own failed end, so that end waits for the Grok agent
pub(crate) fn read_progress(log: &str) -> Progress {
    let mut status = "running";
    let mut steps = Vec::new();
    let mut waits_grok = true;
    let mut local_failed = false;
    for line in log.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line == GROK_START {
            status = "running";
            waits_grok = false;
            local_failed = false;
        }
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
            (Some("followup"), _) => {
                status = "running";
                waits_grok = false;
                steps.push(Step { kind: "reply", text: event["text"].as_str().unwrap_or("").to_string() });
            }
            (Some("result"), _) if event["is_error"] == true => {
                if waits_grok {
                    local_failed = true;
                } else {
                    status = "failed";
                }
                if let Some(text) = event["result"].as_str() {
                    steps.push(Step { kind: "output", text: text.to_string() });
                }
            }
            (Some("result"), _) => status = "finished",
            (Some("exit"), _) if status == "running" => {
                let error_code = event["code"].as_i64().is_some_and(|code| code != 0);
                if !(waits_grok && (local_failed || error_code)) {
                    status = "stopped";
                }
            }
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

pub(crate) fn load_tabs(projects_file: &Path, home: &str) -> Result<Vec<Tab>, String> {
    let projects = read_projects(projects_file)?;
    Ok(projects.iter().map(|(id, project)| project_tab(id, project, home)).collect())
}

pub fn read_projects(projects_file: &Path) -> Result<Map<String, Value>, String> {
    let name = projects_file.display();
    let text = fs::read_to_string(projects_file).map_err(|e| format!("failed to read {name}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("failed to parse {name}: {e}"))
}

pub fn project_root(project: &Value, home: &str) -> Option<PathBuf> {
    let path = project["path"].as_str()?;
    Some(match path.strip_prefix("~/") {
        Some(rest) => Path::new(home).join(rest),
        None => PathBuf::from(path),
    })
}

pub(crate) fn find_item(projects: &Map<String, Value>, home: &str, req: &ResolveRequest) -> Result<(PathBuf, Value), ResolveError> {
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

pub(crate) fn local_prompt(req: &ResolveRequest, root: &Path, item: &Value, decision: &str) -> String {
    let item = serde_json::to_string_pretty(item).unwrap_or_default();
    format!(
        "Resolve one open-behaviour item of the project `{tab}`. The project is at `{root}`.\n\n\
         - File: `{FILE_PATH}`\n\
         - Spec: `{spec}`\n\
         - Item: #{id}\n\n\
         The item:\n\n```json\n{item}\n```\n\n\
         The final decision of the user:\n\n{decision}\n\n\
         Resolve the item as the final decision tells. When the item is resolved, remove the whole item #{id} object \
         from the `items` list of the `{spec}` entry in `{FILE_PATH}`. Do not keep the item with a resolved status or a note. \
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
