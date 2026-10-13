#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;
use crate::agent_policy::ToolPolicyEngine;
use crate::{PermissionLevel, Tool, ToolResult};
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};

struct PromptTestTool {
    name: String,
    permission: PermissionLevel,
}

#[async_trait]
impl Tool for PromptTestTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.name
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({"type":"object"})
    }

    async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<ToolResult> {
        Ok(ToolResult::success("ok"))
    }

    fn permission_level(&self) -> PermissionLevel {
        self.permission
    }
}

#[test]
fn render_prompt_boundary_lists_allowed_and_restricted_summary() {
    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(PromptTestTool {
            name: "read_notes".into(),
            permission: PermissionLevel::ReadOnly,
        }),
        Box::new(PromptTestTool {
            name: "write_notes".into(),
            permission: PermissionLevel::Write,
        }),
    ];
    let mut permissions = HashMap::new();
    permissions.insert("web".to_string(), "read_only".to_string());
    let session = ToolPolicyEngine::build_session(
        "orchestrator",
        "web",
        "chat",
        &permissions,
        &tools,
        &HashSet::new(),
    );

    let rendered = render_tool_policy_boundary(&session, 2048).expect("boundary");

    assert!(rendered.contains("## Tool Policy Boundary"));
    assert!(rendered.contains("Agent: orchestrator"));
    assert!(rendered.contains("Allowed tools: 1 (the tools in your list)"));
    assert!(
        !rendered.contains("read_notes"),
        "names ride on the schemas, not here"
    );
    assert!(rendered.contains("Restricted tools: 1 omitted by policy"));
    assert!(!rendered.contains("write_notes"));
}

#[test]
fn render_prompt_boundary_is_bounded() {
    let tools: Vec<Box<dyn Tool>> = (0..80)
        .map(|idx| {
            Box::new(PromptTestTool {
                name: format!("long_tool_name_{idx}_with_extra_context"),
                permission: PermissionLevel::Write,
            }) as Box<dyn Tool>
        })
        .collect();
    let mut permissions = HashMap::new();
    permissions.insert("web".to_string(), "read_only".to_string());
    let session = ToolPolicyEngine::build_session(
        "orchestrator",
        "web",
        "chat",
        &permissions,
        &tools,
        &HashSet::new(),
    );

    let rendered = render_tool_policy_boundary(&session, 192).expect("boundary");

    assert!(rendered.len() <= 192);
    assert!(rendered.is_char_boundary(rendered.len()));
}

#[test]
fn empty_policy_session_renders_none() {
    let session = ToolPolicyEngine::build_session(
        "orchestrator",
        "web",
        "chat",
        &HashMap::new(),
        &[],
        &HashSet::new(),
    );

    assert!(render_tool_policy_boundary(&session, 2048).is_none());
}

#[test]
fn unrestricted_policy_session_renders_none() {
    let tools: Vec<Box<dyn Tool>> = vec![Box::new(PromptTestTool {
        name: "write_notes".into(),
        permission: PermissionLevel::Write,
    })];
    let session = ToolPolicyEngine::build_session(
        "orchestrator",
        "web",
        "chat",
        &HashMap::new(),
        &tools,
        &HashSet::new(),
    );

    assert!(render_tool_policy_boundary(&session, 2048).is_none());
}

fn restricted_session() -> ToolPolicySession {
    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(PromptTestTool {
            name: "read_notes".into(),
            permission: PermissionLevel::ReadOnly,
        }),
        Box::new(PromptTestTool {
            name: "write_notes".into(),
            permission: PermissionLevel::Write,
        }),
    ];
    let mut permissions = HashMap::new();
    permissions.insert("web".to_string(), "read_only".to_string());
    ToolPolicyEngine::build_session(
        "orchestrator",
        "web",
        "chat",
        &permissions,
        &tools,
        &HashSet::new(),
    )
}

#[test]
fn boundary_truncates_to_the_byte_budget_with_a_marker() {
    let session = restricted_session();
    let full = render_tool_policy_boundary(&session, 4096).expect("boundary");
    let budget = full.len() - 10;
    let rendered = render_tool_policy_boundary(&session, budget).expect("boundary");
    assert!(rendered.len() <= budget);
    assert!(rendered.ends_with("[...truncated]"));
}

#[test]
fn boundary_with_zero_budget_is_empty() {
    let session = restricted_session();
    assert_eq!(render_tool_policy_boundary(&session, 0).unwrap(), "");
}

#[test]
fn boundary_smaller_than_the_marker_is_cut_without_it() {
    let session = restricted_session();
    let rendered = render_tool_policy_boundary(&session, 5).unwrap();
    assert!(rendered.len() <= 5);
    assert!(!rendered.contains("truncated"));
}

#[test]
fn risk_levels_display_and_map_from_permission() {
    use crate::agent_policy::TaskRiskLevel;
    let cases = [
        (PermissionLevel::None, "low"),
        (PermissionLevel::ReadOnly, "low"),
        (PermissionLevel::Write, "medium"),
        (PermissionLevel::Execute, "high"),
        (PermissionLevel::Dangerous, "critical"),
    ];
    for (permission, text) in cases {
        assert_eq!(
            TaskRiskLevel::from_allowed_permission(permission).to_string(),
            text
        );
    }
}

#[test]
fn blocks_execution_covers_actions_and_the_permission_ceiling() {
    use crate::agent_policy::{ToolPolicyAction, ToolPolicyDecision};
    let decision = |action, required| ToolPolicyDecision {
        tool_name: "t".into(),
        action,
        required_permission: required,
        allowed_permission: PermissionLevel::ReadOnly,
    };
    assert!(decision(ToolPolicyAction::Deny, None).blocks_execution());
    assert!(decision(ToolPolicyAction::RequireApproval, None).blocks_execution());
    assert!(!decision(ToolPolicyAction::HideFromPrompt, None).blocks_execution());
    assert!(
        decision(
            ToolPolicyAction::HideFromPrompt,
            Some(PermissionLevel::Write)
        )
        .blocks_execution()
    );
    assert!(!decision(ToolPolicyAction::Allow, Some(PermissionLevel::ReadOnly)).blocks_execution());
}

#[test]
fn deny_moves_a_tool_to_blocked_and_ignores_unknown_names() {
    let mut session = restricted_session();
    assert!(session.is_allowed("read_notes"));
    session.deny(["read_notes", "unknown_tool"]);
    assert!(!session.is_allowed("read_notes"));
    assert!(session.blocked_tool_names.contains("read_notes"));
    assert!(session.decision_for("read_notes").is_denied());
    assert!(session.decision_for("unknown_tool").is_denied());
}

#[test]
fn prompt_visibility_uses_a_sentinel_when_nothing_is_allowed() {
    let mut session = restricted_session();
    session.deny(["read_notes"]);
    let visible = session.visible_tool_names_for_prompt();
    assert_eq!(visible.len(), 1);
    assert!(!visible.contains("read_notes"));

    let open =
        ToolPolicyEngine::build_session("a", "web", "chat", &HashMap::new(), &[], &HashSet::new());
    assert!(open.visible_tool_names_for_prompt().is_empty());
}
