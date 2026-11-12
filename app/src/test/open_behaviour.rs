use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::open_behaviour::*;

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

#[test]
fn valid_chat_accepts_letters_digits_and_dash() {
    assert!(valid_chat("local-1"));
    for chat in ["", "../app", "a.b", "a b", "chat/1"] {
        assert!(!valid_chat(chat), "{chat}");
    }
}

#[test]
fn project_root_expands_a_home_path_and_keeps_an_other_path() {
    let home = "/home/user";
    assert_eq!(
        project_root(&json!({ "path": "~/my-app" }), home).unwrap(),
        Path::new(home).join("my-app")
    );
    assert_eq!(
        project_root(&json!({ "path": "/work/my-app" }), home).unwrap(),
        Path::new("/work/my-app")
    );
    assert!(project_root(&json!({}), home).is_none());
}

fn step(kind: &'static str, text: &str) -> Step {
    Step {
        kind,
        text: text.to_string(),
    }
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
        [
            step("tool", "read: /code/a.md"),
            step("tool", "todo"),
            step("tool", "mcp_search"),
            step("message", "Item #3 is resolved.")
        ]
    );
}

#[test]
fn read_progress_gives_the_status_of_the_end() {
    assert_eq!(
        read_progress(r#"{"type":"result","is_error":false,"result":"Done."}"#).status,
        "finished"
    );
    assert_eq!(
        read_progress("{\"type\":\"result\",\"is_error\":false}\n{\"type\":\"exit\",\"code\":0}")
            .status,
        "finished"
    );
    assert_eq!(
        read_progress(r#"{"type":"exit","code":null}"#).status,
        "stopped"
    );

    let failed = read_progress(&format!(
        "{GROK_START}\n{}",
        r#"{"type":"result","is_error":true,"result":"Model error."}"#
    ));
    assert_eq!(
        failed,
        Progress {
            status: "failed",
            steps: vec![step("output", GROK_START), step("output", "Model error.")]
        }
    );
}

#[test]
fn read_progress_runs_again_after_a_follow_up() {
    let ended = format!(
        "{}\n{}",
        r#"{"type":"result","is_error":false,"result":"Done."}"#,
        followup_line("Also fix the test.")
    );
    let progress = read_progress(&ended);
    assert_eq!(
        progress,
        Progress {
            status: "running",
            steps: vec![step("reply", "Also fix the test.")]
        }
    );
    assert!(!log_has_end(last_run(&ended)));

    let again = format!(
        "{ended}\n{}",
        r#"{"type":"result","is_error":false,"result":"Fixed."}"#
    );
    assert_eq!(read_progress(&again).status, "finished");
    assert!(log_has_end(last_run(&again)));
}

#[test]
fn read_progress_waits_for_grok_after_the_first_agent_fails() {
    let failed_first = [
        r#"{"type":"result","is_error":true,"result":"Model error."}"#,
        r#"{"type":"exit","code":1}"#,
    ]
    .join("\n");
    assert_eq!(read_progress(&failed_first).status, "running");
    assert_eq!(
        read_progress(r#"{"type":"exit","code":2}"#).status,
        "running"
    );

    let grok = format!("{failed_first}\n{GROK_START}");
    assert_eq!(read_progress(&grok).status, "running");
    assert_eq!(last_run(&grok), "");

    let grok_failed = format!(
        "{grok}\n{}",
        r#"{"type":"result","is_error":true,"result":"Grok error."}"#
    );
    assert_eq!(read_progress(&grok_failed).status, "failed");
}

#[test]
fn a_failed_grok_start_ends_the_log() {
    let dir = temp_home("grok-start");
    let log = dir.join("local-1.log");
    write(
        &log,
        "{\"type\":\"result\",\"is_error\":true}\n{\"type\":\"exit\",\"code\":1}\n",
    );
    start_grok_after_fail(&log);
    let text = read_log(&log);
    assert!(text.contains(GROK_START), "{text}");
    assert_eq!(read_progress(&text).status, "failed");
}

#[test]
fn a_failed_follow_up_starts_no_grok_agent() {
    let dir = temp_home("grok-followup");
    let log = dir.join("local-1.log");
    let ended = [
        r#"{"type":"result","is_error":false,"result":"Done."}"#.to_string(),
        followup_line("More."),
        r#"{"type":"result","is_error":true,"result":"Model error."}"#.to_string(),
    ]
    .join("\n");
    write(&log, &format!("{ended}\n"));
    start_grok_after_fail(&log);
    let text = read_log(&log);
    assert!(!text.contains(GROK_START));
    assert_eq!(read_progress(&text).status, "failed");
}

#[test]
fn local_session_reads_the_last_session_of_the_current_agent() {
    let first = [
        r#"{"type":"system","subtype":"init","session_id":"s-1"}"#,
        r#"{"type":"result","is_error":false,"session_id":"s-2"}"#,
    ]
    .join("\n");
    assert_eq!(local_session(&first).as_deref(), Some("s-2"));
    let bad_id = format!("{first}\n{}", r#"{"type":"system","session_id":"../x"}"#);
    assert_eq!(local_session(&bad_id).as_deref(), Some("s-2"));
    assert_eq!(local_session(&format!("{first}\n{GROK_START}")), None);
    let grok = format!(
        "{first}\n{GROK_START}\n{}",
        r#"{"type":"system","session_id":"g-1"}"#
    );
    assert_eq!(local_session(&grok).as_deref(), Some("g-1"));
}

#[test]
fn local_failed_reads_the_first_end_of_the_run() {
    assert!(local_failed(r#"{"type":"result","is_error":true}"#));
    assert!(local_failed(r#"{"type":"exit","code":1}"#));
    assert!(!local_failed(
        "{\"type\":\"result\",\"is_error\":false}\n{\"type\":\"exit\",\"code\":1}"
    ));
    assert!(!local_failed(r#"{"type":"exit","code":null}"#));
}

#[test]
fn read_progress_keeps_a_line_that_is_not_json() {
    let progress = read_progress(&format!(
        "{GROK_START}\nError: not logged in\n\n{{\"type\":\"exit\",\"code\":1}}"
    ));
    assert_eq!(
        progress,
        Progress {
            status: "stopped",
            steps: vec![
                step("output", GROK_START),
                step("output", "Error: not logged in")
            ]
        }
    );
}
