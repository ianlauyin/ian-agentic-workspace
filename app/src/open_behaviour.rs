use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tokio::process::Command;

const WORKSPACE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
const PROJECTS_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../conf/projects.json");
const FILE_PATH: &str = "spec/open-behaviour.json";
const ORCA_BIN: &str = "orca";
const AGENT_MODEL: &str = "claude-opus-5-5-medium";

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
    terminal: String,
}

struct OrcaOutput {
    status_ok: bool,
    stdout: String,
    stderr: String,
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
    let terminal = launch_terminal(&workspace, &prompt).await.map_err(ResolveError::Failed)?;
    Ok(Json(ResolveStart { terminal }))
}

fn launch_args(workspace: &Path, prompt: &str) -> Vec<String> {
    vec![
        "agent".to_string(),
        "launch".to_string(),
        "--agent".to_string(),
        "cursor".to_string(),
        "--surface".to_string(),
        "terminal".to_string(),
        "--model".to_string(),
        AGENT_MODEL.to_string(),
        "--worktree".to_string(),
        format!("path:{}", workspace.display()),
        "--prompt".to_string(),
        prompt.to_string(),
        "--json".to_string(),
    ]
}

async fn launch_terminal(workspace: &Path, prompt: &str) -> Result<String, String> {
    let args = launch_args(workspace, prompt);
    let output = run_orca(&args, workspace).await?;
    if !orca_not_running(&output) {
        return terminal_handle(&output);
    }
    // A closed Orca reports runtime_unavailable. Open it once, then launch again.
    let open = run_orca(&["open".to_string(), "--json".to_string()], workspace).await?;
    if orca_not_running(&open) || !open.status_ok {
        return Err(command_error("failed to open Orca", &open));
    }
    terminal_handle(&run_orca(&args, workspace).await?)
}

async fn run_orca(args: &[String], workspace: &Path) -> Result<OrcaOutput, String> {
    let output = Command::new(ORCA_BIN)
        .args(args)
        .current_dir(workspace)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("failed to start {ORCA_BIN}: {e}"))?;
    Ok(OrcaOutput {
        status_ok: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn orca_not_running(output: &OrcaOutput) -> bool {
    parse_orca(&output.stdout).is_some_and(|value| value["error"]["code"] == "runtime_unavailable")
        || output.stdout.contains("Orca is not running")
        || output.stderr.contains("Orca is not running")
}

fn terminal_handle(output: &OrcaOutput) -> Result<String, String> {
    let Some(value) = parse_orca(&output.stdout) else {
        return Err(command_error("failed to start the Orca terminal", output));
    };
    if value["ok"] != true {
        return Err(format!("failed to start the Orca terminal: {}", failure_text(&value)));
    }
    let outcome = &value["result"]["outcome"];
    if outcome["kind"] != "terminal" {
        return Err("Orca did not start the agent in a terminal.".to_string());
    }
    let Some(handle) = outcome["handle"].as_str().filter(|handle| !handle.is_empty()) else {
        return Err("Orca did not return a terminal.".to_string());
    };
    if value["result"]["prompt"]["outcome"] != "handed-to-terminal" {
        return Err(format!("Orca started terminal {handle}, but the agent did not receive the decision."));
    }
    Ok(handle.to_string())
}

fn parse_orca(stdout: &str) -> Option<Value> {
    serde_json::from_str(stdout.trim()).ok()
}

fn failure_text(value: &Value) -> String {
    let message = value["error"]["message"].as_str().unwrap_or("").trim();
    let code = value["error"]["code"].as_str().unwrap_or("");
    let step = value["error"]["data"]["nextSteps"].as_array().and_then(|steps| {
        steps.iter().find_map(|step| step.as_str().map(str::trim).filter(|step| !step.is_empty()))
    });
    if !message.is_empty() && message != code {
        return message.to_string();
    }
    step.map(str::to_string).unwrap_or_else(|| "Orca did not start the terminal.".to_string())
}

fn command_error(prefix: &str, output: &OrcaOutput) -> String {
    if let Some(value) = parse_orca(&output.stdout) {
        if value["ok"] != true {
            return format!("{prefix}: {}", failure_text(&value));
        }
    }
    let detail = first_line(&output.stderr).or_else(|| first_line(&output.stdout)).unwrap_or("orca failed");
    format!("{prefix}: {detail}")
}

fn first_line(text: &str) -> Option<&str> {
    text.lines().map(str::trim).find(|line| !line.is_empty())
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

    fn orca_output(stdout: &str) -> OrcaOutput {
        OrcaOutput { status_ok: true, stdout: stdout.to_string(), stderr: String::new() }
    }

    fn started(handle: &str) -> String {
        json!({
            "ok": true,
            "result": {
                "outcome": { "kind": "terminal", "handle": handle },
                "prompt": { "delivery": "submit", "outcome": "handed-to-terminal" }
            }
        })
        .to_string()
    }

    #[test]
    fn launch_args_start_cursor_in_a_new_terminal() {
        let args = launch_args(Path::new("/code/workspace"), "Resolve it.");
        let expected = [
            "agent",
            "launch",
            "--agent",
            "cursor",
            "--surface",
            "terminal",
            "--model",
            "claude-opus-5-5-medium",
            "--worktree",
            "path:/code/workspace",
            "--prompt",
            "Resolve it.",
            "--json",
        ];
        assert_eq!(args, expected.map(str::to_string).to_vec());
    }

    #[test]
    fn terminal_handle_reads_a_started_terminal() {
        let handle = terminal_handle(&orca_output(&started("term_1"))).unwrap();
        assert_eq!(handle, "term_1");
    }

    #[test]
    fn terminal_handle_rejects_a_chat_or_a_missed_prompt() {
        let chat = json!({
            "ok": true,
            "result": {
                "outcome": { "kind": "structured", "handle": "agent_1", "sessionId": "cursor_1" },
                "prompt": { "outcome": "journaled", "messageId": "msg_1" }
            }
        });
        assert_eq!(terminal_handle(&orca_output(&chat.to_string())).unwrap_err(), "Orca did not start the agent in a terminal.");
        let missed = json!({
            "ok": true,
            "result": {
                "outcome": { "kind": "terminal", "handle": "term_1" },
                "prompt": { "outcome": "not-delivered" }
            }
        });
        assert!(terminal_handle(&orca_output(&missed.to_string())).unwrap_err().contains("term_1"));
    }

    #[test]
    fn a_closed_orca_is_not_a_terminal_error() {
        let closed = json!({ "ok": false, "error": { "code": "runtime_unavailable", "message": "runtime_unavailable" } });
        assert!(orca_not_running(&orca_output(&closed.to_string())));
        let text = OrcaOutput { status_ok: false, stdout: String::new(), stderr: "Orca is not running. Run 'orca open' first.\n".to_string() };
        assert!(orca_not_running(&text));
        assert!(!orca_not_running(&orca_output(&started("term_1"))));
    }

    #[test]
    fn a_launch_error_uses_the_next_step_when_the_message_is_the_code() {
        let missing = json!({
            "ok": false,
            "error": {
                "code": "selector_not_found",
                "message": "selector_not_found",
                "data": { "nextSteps": ["No Orca workspace matched the worktree selector."] }
            }
        });
        let err = terminal_handle(&orca_output(&missing.to_string())).unwrap_err();
        assert_eq!(err, "failed to start the Orca terminal: No Orca workspace matched the worktree selector.");
    }
}
