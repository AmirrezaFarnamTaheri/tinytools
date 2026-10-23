//! Tool: `run_linter` — run linting tools for the Critic archetype.

use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;
use tinytools::ToolRunContext;
use tinytools::{PermissionLevel, Tool, ToolCallOptions, ToolResult};

/// Runs linters (cargo clippy, eslint) and returns structured findings.
#[derive(Debug)]
pub struct RunLinterTool {
    workspace_dir: PathBuf,
}

impl RunLinterTool {
    #[must_use]
    /// Create the tool.
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self { workspace_dir }
    }

    fn workspace_dir_for_context(&self, context: Option<&dyn ToolRunContext>) -> PathBuf {
        if let Some(workspace) = context.and_then(|ctx| ctx.workspace()) {
            tracing::debug!(
                workspace_root = %workspace.root.display(),
                policy_id = %workspace.policy_id,
                "[run_linter] using TinyAgents workspace descriptor as workspace dir"
            );
            return workspace.root.clone();
        }
        self.workspace_dir.clone()
    }
}

#[async_trait]
impl Tool for RunLinterTool {
    fn exposure(&self) -> tinytools::ToolExposure {
        tinytools::ToolExposure::Deferred
    }

    fn name(&self) -> &'static str {
        "run_linter"
    }

    fn description(&self) -> &'static str {
        "Run linting tools on the codebase. Supports 'clippy' for Rust and 'eslint' for \
         TypeScript/JavaScript. Returns warnings and errors."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "linter": {
                    "type": "string",
                    "enum": ["clippy", "eslint", "auto"],
                    "description": "Which linter to run. 'auto' detects from project files.",
                    "default": "auto"
                },
                "path": {
                    "type": "string",
                    "description": "Limit linting to a specific path."
                }
            }
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Execute
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        self.execute_with_context(args, ToolCallOptions::default(), None)
            .await
    }

    async fn execute_with_context(
        &self,
        args: serde_json::Value,
        _options: ToolCallOptions,
        context: Option<&dyn ToolRunContext>,
    ) -> anyhow::Result<ToolResult> {
        let workspace_dir = self.workspace_dir_for_context(context);
        let linter = args
            .get("linter")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");

        let linter = if linter == "auto" {
            if workspace_dir.join("Cargo.toml").exists() {
                "clippy"
            } else if workspace_dir.join("package.json").exists() {
                "eslint"
            } else {
                return Ok(ToolResult::error(
                    "Could not detect project type for linting.",
                ));
            }
        } else {
            linter
        };

        let mut cmd = match linter {
            "clippy" => {
                let mut cmd = tokio::process::Command::new("cargo");
                cmd.args([
                    "clippy",
                    "--message-format=short",
                    "--",
                    "-W",
                    "clippy::all",
                ])
                .current_dir(&workspace_dir);
                cmd
            }
            "eslint" => {
                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
                if path.starts_with('/') || path.contains("..") {
                    return Ok(ToolResult::error(
                        "path must be a relative path within the workspace \
                             (no absolute paths or '..')",
                    ));
                }
                // A value starting with `-` would reach eslint as an option
                // (`--output-file=<f>` writes a file). No path needs one.
                if path.starts_with('-') {
                    return Ok(ToolResult::error("path must be a path, not an option"));
                }
                let mut cmd = tokio::process::Command::new("npx");
                cmd.args(["eslint", "--format", "compact", path])
                    .current_dir(&workspace_dir);
                cmd
            }
            other => {
                return Ok(ToolResult::error(format!("Unknown linter: {other}")));
            }
        };
        crate::timeout::hide_console_window(cmd.as_std_mut());
        let output = cmd.output().await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let combined = if stdout.is_empty() {
            stderr.to_string()
        } else {
            format!("{stdout}\n{stderr}")
        };

        if output.status.success() {
            Ok(ToolResult::success(combined))
        } else {
            Ok(ToolResult::error(format!(
                "Linter exited with code {:?}\n{}",
                output.status.code(),
                combined
            )))
        }
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod test;
