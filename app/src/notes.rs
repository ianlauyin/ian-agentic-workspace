use std::path::Path;

use axum::Json;
use serde::{Deserialize, Serialize};

use crate::open_behaviour::{PROJECTS_FILE, ResolveError, ResolveStart, project_root, read_projects, start_agent};

#[derive(Serialize)]
pub struct Project {
    id: String,
    name: String,
}

#[derive(Deserialize)]
pub struct AssignRequest {
    tab: String,
    text: String,
}

pub async fn projects() -> Result<Json<Vec<Project>>, ResolveError> {
    let projects = read_projects(Path::new(PROJECTS_FILE)).map_err(ResolveError::Failed)?;
    Ok(Json(projects.keys().map(|id| Project { id: id.clone(), name: id.clone() }).collect()))
}

pub async fn assign(Json(req): Json<AssignRequest>) -> Result<Json<ResolveStart>, ResolveError> {
    let bad = |msg: &str| ResolveError::BadItem(msg.to_string());
    let home = std::env::var("HOME").unwrap_or_default();
    let text = req.text.trim();
    if text.is_empty() {
        return Err(bad("The note has no text."));
    }
    let projects = read_projects(Path::new(PROJECTS_FILE)).map_err(ResolveError::Failed)?;
    let root = projects.get(&req.tab).and_then(|project| project_root(project, &home)).ok_or_else(|| bad("Unknown project."))?;
    let chat = start_agent(&root, &task_prompt(&req.tab, &root, text)).map_err(ResolveError::Failed)?;
    Ok(Json(ResolveStart::new(chat)))
}

pub(crate) fn task_prompt(tab: &str, root: &Path, text: &str) -> String {
    format!(
        "Do one task in the project `{tab}`. The project is at `{root}`.\n\n\
         The task, from the notes of the user:\n\n{text}\n\n\
         Obey `AGENTS.md`.",
        root = root.display(),
    )
}
