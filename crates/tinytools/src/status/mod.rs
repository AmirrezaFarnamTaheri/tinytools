//! Tool-call lifecycle state and human-readable failure classification.
//!
//! - [`ToolLifecycleState`]: where a call is (queued/running/.../needs-input).
//! - [`ToolFailureClass`] / [`FailureCategory`]: what kind of failure it was.
//! - [`ClassifiedFailure`]: a class plus plain-language cause and next action.
//! - [`classify`]: the pure heuristic mapping raw tool error text to the above.
//!
//! Pure data and logic: no I/O, no global state, no persistence. Product copy
//! beyond the English defaults in [`describe`] belongs to the host.

mod ops;
mod types;
mod web_fetch;

pub use ops::{
    ToolFailureClassification, classify, classify_for_tool, describe, tool_execution_error,
};
pub use types::{
    ClassifiedFailure, FailureCategory, MODULE_FAULT_MARKER, NOT_FOUND_MARKER,
    POLICY_BLOCKED_MARKER, POLICY_DENIED_MARKER, TASK_FAILED_MARKER, ToolFailureClass,
    ToolLifecycleState, UNSUPPORTED_MARKER,
};
pub use web_fetch::{
    FetchedSiteFailure, FetchedSiteRecovery, classify_web_fetch_status, web_fetch_status,
};
