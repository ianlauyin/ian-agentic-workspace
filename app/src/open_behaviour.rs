use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tokio::process::Command;

const WORKSPACE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
const PROJECTS_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../conf/projects.json");
const RESOLVE_LOG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/resolve");
const FILE_PATH: &str = "spec/open-behaviour.json";
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
    log: String,
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

    fs::create_dir_all(RESOLVE_LOG_DIR).map_err(|e| ResolveError::Failed(format!("failed to create {RESOLVE_LOG_DIR}: {e}")))?;
    let millis = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or_default();
    let log = format!("{RESOLVE_LOG_DIR}/{millis}.log");
    let out = File::create(&log).map_err(|e| ResolveError::Failed(format!("failed to create {log}: {e}")))?;
    let err = out.try_clone().map_err(|e| ResolveError::Failed(format!("failed to open {log}: {e}")))?;
    let mut child = Command::new(AGENT_BIN)
        .args(["-p", "--model", AGENT_MODEL, "--force", "--trust", "--workspace", WORKSPACE_DIR])
        .arg(prompt)
        .current_dir(WORKSPACE_DIR)
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(err)
        // cargo watch stops the process group of the server on a restart, and the run must continue
        .process_group(0)
        .spawn()
        .map_err(|e| ResolveError::Failed(format!("failed to start {AGENT_BIN}: {e}")))?;
    tokio::spawn(async move {
        let _ = child.wait().await;
    });
    Ok(Json(ResolveStart { log: format!("app/data/resolve/{millis}.log") }))
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
}
