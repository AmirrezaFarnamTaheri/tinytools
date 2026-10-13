//! Tool-registry vocabulary: discovery entries, policy diagnostics, and the
//! bounded buffer of recent policy denials a host reports through them.

#![allow(missing_docs)]

pub mod denials;
mod types;

pub use types::{
    CapabilityProviderDiagnostics, McpAllowlistDiagnostics, McpServerAllowlistSummary,
    McpWriteAuditHealth, RecentPolicyDenial, ToolPolicyDiagnostics, ToolPolicyPosture,
    ToolRegistryEntry, ToolRegistryHealth, ToolRegistryList, ToolRegistryTransport,
};
