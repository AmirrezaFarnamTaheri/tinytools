//! Agent tool-policy profiling and enforcement.
//!
//! Keeps prompt-visible tools and runtime execution aligned with a session's
//! configured channel permission boundary. [`ToolPolicyEngine`] classifies a
//! belt against the permission a channel allows and returns an immutable
//! [`ToolPolicySession`]; [`render_tool_policy_boundary`] renders the matching
//! system-prompt section. The engine decides nothing about *how* a denial is
//! surfaced or approved: that stays with the host.

#![allow(missing_docs)]

mod engine;
mod prompt;
mod types;

pub use engine::ToolPolicyEngine;
pub use prompt::{TOOL_POLICY_BOUNDARY_HEADING, render_tool_policy_boundary};
pub use types::{
    TaskProfile, TaskRiskLevel, ToolCapability, ToolPolicyAction, ToolPolicyDecision,
    ToolPolicySession,
};
