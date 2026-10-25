//! Budget, metadata-failure, file-state and paging-marker tests for `file_read`.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use super::*;
use crate::filesystem::test_support::{TestGate, WorkspaceContext, WrapGate};
use tempfile::TempDir;

#[test]
fn reads_are_concurrency_safe() {
    let dir = TempDir::new().unwrap();
    let tool = FileReadTool::new(TestGate::supervised(dir.path().to_path_buf()));
    assert!(tool.is_concurrency_safe(&json!({})));
}

#[tokio::test]
async fn exhausted_action_budget_is_refused() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("a.txt"), "x").unwrap();
    let gate = WrapGate::new(TestGate::supervised(dir.path().to_path_buf()))
        .no_budget()
        .arc();
    let r = FileReadTool::new(gate)
        .execute(json!({"path": "a.txt"}))
        .await
        .unwrap();
    assert!(r.output().contains("action budget exhausted"));
}

#[tokio::test]
async fn missing_metadata_is_reported() {
    let dir = TempDir::new().unwrap();
    let gate = WrapGate::new(TestGate::supervised(dir.path().to_path_buf()))
        .fixed_path(dir.path().join("vanished.txt"))
        .arc();
    let r = FileReadTool::new(gate)
        .execute(json!({"path": "vanished.txt"}))
        .await
        .unwrap();
    assert!(r.is_error);
    assert!(r.output().contains("Failed to read file metadata"));
}

#[tokio::test]
async fn a_read_under_an_agent_id_records_file_state() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("a.txt"), "tracked").unwrap();
    let tool = FileReadTool::new(TestGate::supervised(dir.path().to_path_buf()));
    let r = crate::file_state::with_file_state_agent_id(
        "agent-file-read-extra".to_string(),
        tool.execute(json!({"path": "a.txt"})),
    )
    .await
    .unwrap();
    assert_eq!(r.output(), "tracked");
}

#[tokio::test]
async fn execute_with_context_delegates() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("a.txt"), "ctx").unwrap();
    let tool = FileReadTool::new(TestGate::supervised(dir.path().to_path_buf()));
    let ctx = WorkspaceContext::at(dir.path());
    let r = tool
        .execute_with_context(
            json!({"path": "a.txt"}),
            ToolCallOptions::default(),
            Some(&ctx),
        )
        .await
        .unwrap();
    assert_eq!(r.output(), "ctx");
}

#[tokio::test]
async fn an_oversized_path_drops_out_of_the_continuation_marker() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("big.txt");
    std::fs::write(&file, "a".repeat(40 * 1024)).unwrap();
    let gate = WrapGate::new(TestGate::supervised(dir.path().to_path_buf()))
        .fixed_path(file)
        .arc();
    let long_path = "p".repeat(20 * 1024);
    let r = FileReadTool::new(gate)
        .execute(json!({"path": long_path}))
        .await
        .unwrap();
    assert!(!r.is_error);
    let out = r.output();
    assert!(
        out.contains("continue with file_read at \"offset\":"),
        "marker missing"
    );
    assert!(!out.contains("ppppp"));
    assert!(out.len() <= 16 * 1024);
}

// A paged read must arm the partial-read guard before a full overwrite.
#[tokio::test]
async fn a_paged_read_is_recorded_as_partial() {
    crate::file_state::init_global(true);
    let dir = TempDir::new().unwrap();
    let target = dir.path().canonicalize().unwrap().join("big.txt");
    std::fs::write(&target, "x".repeat(40 * 1024)).unwrap();
    let tool = FileReadTool::new(TestGate::supervised(dir.path().to_path_buf()));
    let agent = format!("partial-{}", dir.path().display());
    let r = crate::file_state::with_file_state_agent_id(
        agent.clone(),
        tool.execute(json!({"path": "big.txt"})),
    )
    .await
    .unwrap();
    assert!(
        r.output().contains("[file_read page:"),
        "not paged: test premise"
    );
    assert!(
        crate::file_state::check_partial_read(&agent, &target).is_some(),
        "a paged read was recorded as a full read"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn a_fifo_is_refused_not_read() {
    let dir = TempDir::new().unwrap();
    let fifo = dir.path().join("pipe");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    let tool = FileReadTool::new(TestGate::supervised(dir.path().to_path_buf()));
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tool.execute(json!({"path": "pipe"})),
    )
    .await;
    // Give a stuck reader EOF so a failing run can still shut down.
    std::thread::spawn(move || drop(std::fs::OpenOptions::new().write(true).open(fifo)));
    let result = result.expect("file_read blocked on a FIFO").unwrap();
    assert!(result.is_error);
    assert!(result.output().contains("not a regular file"));
}
