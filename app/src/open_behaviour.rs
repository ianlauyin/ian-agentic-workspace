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
const LOG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/resolve");
const AGENT_BIN: &str = "cursor-agent";
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
    chat: String,
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
    tokio::spawn(async move {
        let _ = child.wait().await;
    });
    Ok(chat)
}

fn chat_id(status_ok: bool, stdout: &str, stderr: &str) -> Result<String, String> {
    let chat = stdout.trim();
    let valid = !chat.is_empty() && chat.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    if status_ok && valid {
        return Ok(chat.to_string());
    }
    let detail = first_line(stderr).or_else(|| first_line(stdout)).unwrap_or("no chat id");
    Err(format!("failed to create the Cursor chat: {detail}"))
}

fn agent_args(workspace: &Path, chat: &str, prompt: &str) -> Vec<String> {
    vec![
        "--print".to_string(),
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
}
