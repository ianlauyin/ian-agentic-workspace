use std::fs;
use std::path::{Path, PathBuf};

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::{Map, Value, json};

const PROJECTS_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../conf/projects.json");
const FILE_NAME: &str = "open-behaviour.json";

#[derive(Serialize)]
pub struct Tab {
    id: String,
    name: String,
    groups: Vec<Value>,
    errors: Vec<String>,
}

pub struct LoadError(String);

impl IntoResponse for LoadError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": self.0 }))).into_response()
    }
}

pub async fn load() -> Result<Json<Vec<Tab>>, LoadError> {
    let home = std::env::var("HOME").unwrap_or_default();
    load_tabs(Path::new(PROJECTS_FILE), &home).map(Json).map_err(LoadError)
}

fn load_tabs(projects_file: &Path, home: &str) -> Result<Vec<Tab>, String> {
    let name = projects_file.display();
    let text = fs::read_to_string(projects_file).map_err(|e| format!("failed to read {name}: {e}"))?;
    let projects: Map<String, Value> = serde_json::from_str(&text).map_err(|e| format!("failed to parse {name}: {e}"))?;
    Ok(projects.iter().map(|(id, project)| project_tab(id, project, home)).collect())
}

fn project_tab(id: &str, project: &Value, home: &str) -> Tab {
    let mut tab = Tab {
        id: id.to_string(),
        name: id.to_string(),
        groups: Vec::new(),
        errors: Vec::new(),
    };
    let Some(path) = project["path"].as_str() else {
        tab.errors.push(format!("{id} has no path in conf/projects.json"));
        return tab;
    };
    let root = match path.strip_prefix("~/") {
        Some(rest) => Path::new(home).join(rest),
        None => PathBuf::from(path),
    };
    for (package, source) in open_behaviour_files(&root) {
        match read_groups(&root.join(&source), &package, &source) {
            Ok(groups) => tab.groups.extend(groups),
            Err(e) => tab.errors.push(e),
        }
    }
    tab
}

// the project file comes first, then the package files in name order
fn open_behaviour_files(root: &Path) -> Vec<(String, String)> {
    let spec = root.join("spec");
    let mut files = Vec::new();
    if spec.join(FILE_NAME).is_file() {
        files.push((String::new(), format!("spec/{FILE_NAME}")));
    }
    let Ok(dirs) = fs::read_dir(&spec) else {
        return files;
    };
    let mut packages: Vec<String> = dirs
        .flatten()
        .filter(|dir| dir.path().join(FILE_NAME).is_file())
        .map(|dir| dir.file_name().to_string_lossy().into_owned())
        .collect();
    packages.sort();
    files.extend(packages.into_iter().map(|package| {
        let source = format!("spec/{package}/{FILE_NAME}");
        (package, source)
    }));
    files
}

fn read_groups(file: &Path, package: &str, source: &str) -> Result<Vec<Value>, String> {
    let text = fs::read_to_string(file).map_err(|e| format!("failed to read {source}: {e}"))?;
    let entries: Vec<Map<String, Value>> = serde_json::from_str(&text).map_err(|e| format!("failed to parse {source}: {e}"))?;
    Ok(entries
        .into_iter()
        .map(|mut entry| {
            entry.insert("package".to_string(), json!(package));
            entry.insert("source".to_string(), json!(source));
            Value::Object(entry)
        })
        .collect())
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
    fn reads_project_file_then_package_files() {
        let home = temp_home("order");
        write(&home.join("my-app/spec/open-behaviour.json"), r#"[{"spec":"01_a","items":[]}]"#);
        write(&home.join("my-app/spec/web/open-behaviour.json"), r#"[{"spec":"01_b","items":[]}]"#);
        write(&home.join("my-app/spec/api/open-behaviour.json"), r#"[{"spec":"01_c","items":[]}]"#);
        write(&home.join("my-app/spec/docs/readme.md"), "");
        let file = projects(&home, r#"{"my-app":{"path":"~/my-app"}}"#);

        let tabs = load_tabs(&file, home.to_str().unwrap()).unwrap();
        assert_eq!((tabs.len(), tabs[0].id.as_str(), tabs[0].name.as_str()), (1, "my-app", "my-app"));
        let sources: Vec<(&str, &str, &str)> = tabs[0]
            .groups
            .iter()
            .map(|g| {
                (
                    g["package"].as_str().unwrap(),
                    g["source"].as_str().unwrap(),
                    g["spec"].as_str().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            sources,
            [
                ("", "spec/open-behaviour.json", "01_a"),
                ("api", "spec/api/open-behaviour.json", "01_c"),
                ("web", "spec/web/open-behaviour.json", "01_b"),
            ]
        );
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
    fn bad_file_gives_a_tab_error_and_keeps_the_other_files() {
        let home = temp_home("bad");
        write(&home.join("my-app/spec/open-behaviour.json"), "not json");
        write(&home.join("my-app/spec/web/open-behaviour.json"), r#"[{"spec":"01_b","items":[]}]"#);
        let file = projects(&home, r#"{"my-app":{"path":"~/my-app"},"other":{}}"#);

        let tabs = load_tabs(&file, home.to_str().unwrap()).unwrap();
        assert_eq!(tabs[0].groups.len(), 1);
        assert_eq!(tabs[0].errors.len(), 1);
        assert!(tabs[0].errors[0].starts_with("failed to parse spec/open-behaviour.json"));
        assert_eq!(tabs[1].errors, ["other has no path in conf/projects.json"]);
    }

    #[test]
    fn missing_projects_file_is_an_error() {
        let home = temp_home("missing");
        assert!(load_tabs(&home.join("projects.json"), home.to_str().unwrap()).is_err());
    }
}
