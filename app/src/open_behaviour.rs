use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;

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
const AGENT_BIN: &str = "cursor-agent";
const AGENT_MODEL: &str = "claude-opus-5-5-medium";
const TOOL_DETAILS: [&str; 6] = ["command", "path", "pattern", "globPattern", "query", "url"];

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
    let prompt = resolve_prompt(&req, &root, &item, decision);
    let workspace = fs::canonicalize(WORKSPACE_DIR).map_err(|e| ResolveError::Failed(format!("failed to open the workspace: {e}")))?;
    let chat = start_agent(&workspace, &prompt).await.map_err(ResolveError::Failed)?;
    Ok(Json(ResolveStart { chat }))
}

pub async fn progress(Query(req): Query<ProgressRequest>) -> Result<Json<Progress>, ResolveError> {
    if !valid_chat(&req.chat) {
        return Err(ResolveError::BadItem("Bad chat id.".to_string()));
    }
    let path = Path::new(LOG_DIR).join(format!("{}.log", req.chat));
    let log = fs::read_to_string(&path).map_err(|_| ResolveError::BadItem("The chat has no log.".to_string()))?;
    Ok(Json(read_progress(&log)))
}

async fn start_agent(workspace: &Path, prompt: &str) -> Result<String, String> {
    let output = Command::new(AGENT_BIN)
        .arg("create-chat")
        .current_dir(workspace)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|e| format!("failed to start {AGENT_BIN}: {e}"))?;
    let chat = chat_id(
        output.status.success(),
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    )?;
    fs::create_dir_all(LOG_DIR).map_err(|e| format!("failed to create {LOG_DIR}: {e}"))?;
    let log_path = Path::new(LOG_DIR).join(format!("{chat}.log"));
    let log = fs::File::create(&log_path).map_err(|e| format!("failed to create {}: {e}", log_path.display()))?;
    let err_log = log.try_clone().map_err(|e| format!("failed to open {}: {e}", log_path.display()))?;
    let mut child = Command::new(AGENT_BIN)
        .args(agent_args(workspace, &chat, prompt))
        .current_dir(workspace)
        .stdin(Stdio::null())
        .stdout(log)
        .stderr(err_log)
        // A new process group keeps the agent alive when Ctrl+C stops the server.
        .process_group(0)
        .spawn()
        .map_err(|e| format!("failed to start {AGENT_BIN}: {e}"))?;
    // the exit line tells the page that an agent stopped without a result event
    tokio::spawn(async move {
        let code = child.wait().await.ok().and_then(|status| status.code());
        if let Ok(mut file) = fs::OpenOptions::new().append(true).open(&log_path) {
            let _ = writeln!(file, "{}", json!({ "type": "exit", "code": code }));
        }
    });
    Ok(chat)
}

fn chat_id(status_ok: bool, stdout: &str, stderr: &str) -> Result<String, String> {
    let chat = stdout.trim();
    if status_ok && valid_chat(chat) {
        return Ok(chat.to_string());
    }
    let detail = first_line(stderr).or_else(|| first_line(stdout)).unwrap_or("no chat id");
    Err(format!("failed to create the Cursor chat: {detail}"))
}

fn valid_chat(chat: &str) -> bool {
    !chat.is_empty() && chat.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn agent_args(workspace: &Path, chat: &str, prompt: &str) -> Vec<String> {
    vec![
        "--print".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--force".to_string(),
        "--trust".to_string(),
        "--model".to_string(),
        AGENT_MODEL.to_string(),
        "--workspace".to_string(),
        workspace.display().to_string(),
        "--resume".to_string(),
        chat.to_string(),
        prompt.to_string(),
    ]
}

fn first_line(text: &str) -> Option<&str> {
    text.lines().map(str::trim).find(|line| !line.is_empty())
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

fn resolve_prompt(req: &ResolveRequest, root: &Path, item: &Value, decision: &str) -> String {
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
        let prompt = resolve_prompt(&req, Path::new("/code/my-app"), &json!({ "id": 3 }), "Use option 1.");
        for part in ["`my-app`", "`/code/my-app`", "`spec/open-behaviour.json`", "`web/01_login`", "#3", "\"id\": 3", "Use option 1.", "AGENTS.md"] {
            assert!(prompt.contains(part), "{part}");
        }
    }

    #[test]
    fn agent_args_run_cursor_without_approval_in_the_chat() {
        let args = agent_args(Path::new("/code/workspace"), "chat-1", "Resolve it.");
        let expected = [
            "--print",
            "--output-format",
            "stream-json",
            "--force",
            "--trust",
            "--model",
            "claude-opus-5-5-medium",
            "--workspace",
            "/code/workspace",
            "--resume",
            "chat-1",
            "Resolve it.",
        ];
        assert_eq!(args, expected.map(str::to_string).to_vec());
    }

    #[test]
    fn chat_id_reads_the_created_chat() {
        let chat = chat_id(true, "46798133-e840-455f-8bd1-0fe36eab3bfe\n", "").unwrap();
        assert_eq!(chat, "46798133-e840-455f-8bd1-0fe36eab3bfe");
    }

    #[test]
    fn chat_id_rejects_a_failure_or_a_bad_id() {
        let failed = chat_id(false, "", "Not logged in.\nRun login.\n").unwrap_err();
        assert_eq!(failed, "failed to create the Cursor chat: Not logged in.");
        assert!(chat_id(true, "", "").is_err());
        assert!(chat_id(true, "../chat 1", "").is_err());
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
