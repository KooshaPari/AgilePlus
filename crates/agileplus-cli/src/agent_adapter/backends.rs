//! Backend-specific agent dispatch implementations.
//!
//! Each backend spawns an external process, writes the prompt to its stdin,
//! waits for completion, and returns an `AgentResult`.
//!
//! Traceability: FR-013

use std::process::Output;

use agileplus_domain::error::DomainError;
use agileplus_domain::ports::agent::{AgentConfig, AgentResult, AgentTask};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};

/// Spawn a backend process (claude-cli or cheap-llm-mcp) with the given prompt.
///
/// Returns the running child and a stdin sender for follow-up instructions.
pub(crate) async fn spawn_process(
    binary: &str,
    args: &[&str],
    task: &AgentTask,
    config: &AgentConfig,
) -> Result<(Child, tokio::sync::mpsc::Sender<Vec<u8>>), DomainError> {
    let prompt_content = tokio::fs::read_to_string(&task.prompt_path)
        .await
        .map_err(|e| DomainError::Agent(format!("reading prompt: {}", e)))?;

    let mut cmd = Command::new(binary);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.current_dir(&task.worktree_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for arg in &config.extra_args {
        cmd.arg(arg);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| DomainError::Agent(format!("spawn {binary}: {}", e)))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| DomainError::Agent(format!("{binary}: no stdin handle")))?;

    let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(4);
    let _ = stdin_tx.send(prompt_content.into_bytes()).await;

    tokio::spawn(async move {
        while let Some(data) = stdin_rx.recv().await {
            if stdin.write_all(&data).await.is_err() {
                break;
            }
        }
    });

    Ok((child, stdin_tx))
}

/// Build an `AgentResult` from a completed process output.
pub(crate) fn build_result(output: Output) -> Result<AgentResult, DomainError> {
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let pr_url = extract_pr_url(&stdout);
    let commits = extract_commits(&stdout);

    Ok(AgentResult {
        success: output.status.success(),
        pr_url,
        commits,
        stdout,
        stderr,
        exit_code: output.status.code().unwrap_or(-1),
    })
}

/// Dispatch to the stub backend (no-op, for testing).
pub(crate) async fn dispatch_stub(
    task: &AgentTask,
    _config: &AgentConfig,
) -> Result<AgentResult, DomainError> {
    tracing::warn!(
        "Using stub agent backend for WP {} in {}",
        task.wp_id,
        task.worktree_path.display()
    );
    Ok(AgentResult {
        success: true,
        pr_url: None,
        commits: vec![],
        stdout: format!("[stub] dispatched WP {}", task.wp_id),
        stderr: String::new(),
        exit_code: 0,
    })
}

/// Extract PR URL from agent stdout.
fn extract_pr_url(stdout: &str) -> Option<String> {
    let re = regex::Regex::new(r"https://github\.com/[^/]+/[^/]+/pull/\d+").ok()?;
    re.find(stdout).map(|m| m.as_str().to_string())
}

/// Extract commit SHAs from agent stdout.
fn extract_commits(stdout: &str) -> Vec<String> {
    let re = regex::Regex::new(r"\b([0-9a-f]{7,40})\b").ok();
    re.as_ref()
        .map(|r| {
            r.captures_iter(stdout)
                .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_url_extraction() {
        let stdout = "Successfully created PR at https://github.com/example/repo/pull/42";
        assert_eq!(
            extract_pr_url(stdout),
            Some("https://github.com/example/repo/pull/42".to_string())
        );
        assert_eq!(extract_pr_url("No PR found"), None);
    }

    #[test]
    fn test_commit_extraction() {
        let stdout = "Committed 1234567 and a1b2c3d with message";
        let commits = extract_commits(stdout);
        assert_eq!(commits.len(), 2);
        assert!(commits.contains(&"1234567".to_string()));
        assert!(commits.contains(&"a1b2c3d".to_string()));
    }
}
