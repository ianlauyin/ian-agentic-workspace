use std::path::Path;

use crate::notes::*;

#[test]
fn task_prompt_has_the_project_path_and_the_text() {
    let text = task_prompt("my-app", Path::new("/work/my-app"), "Add a health route.");
    assert!(text.contains("`my-app`") && text.contains("/work/my-app") && text.contains("Add a health route."), "{text}");
    assert!(text.contains("AGENTS.md"));
}
