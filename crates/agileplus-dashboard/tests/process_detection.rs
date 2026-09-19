// SPDX-License-Identifier: MIT OR Apache-2.0
//! Live-process coverage for `process_detector::detect_agents`.
//!
//! `detect_agents` walks the real process table with `sysinfo`, so the only way
//! to assert what it does is to put real processes in that table. Each test
//! builds a stub executable whose *file name* matches (or deliberately fails to
//! match) an agent pattern, starts it with agent-shaped argv, and then asserts
//! what the detector reports for that exact pid.
//!
//! This exercises the whole detection pipeline end to end: the pattern scan and
//! `break` per process, `process.name()`, argv collection, `--cwd` extraction,
//! `WP` task extraction, display-name formatting, start-time formatting, and the
//! pid sort. The existing unit tests only cover the pure helpers.
//!
//! Nothing here reads or writes the operator's configuration, and every stub is
//! built inside a private temp directory.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use agileplus_dashboard::process_detector::{DetectedAgent, detect_agents};

/// Mirrors the pattern list inside `detect_agents`.
const AGENT_PATTERNS: [&str; 7] = [
    "claude", "gemini", "codex", "cursor", "windsurf", "aider", "cline",
];

const STUB_C: &str = r#"
#include <unistd.h>
int main(void) {
    for (;;) {
        sleep(1);
    }
}
"#;

const STUB_RS: &str = r#"
fn main() {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
"#;

fn probe_root() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!(
            "agileplus-dashboard-process-detection-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("create probe directory");
        dir
    })
}

/// Build `probe_root()/<name>` from the C stub, falling back to `rustc` when no
/// C compiler is installed. The executable name is what `sysinfo` reports as the
/// process name, so it is the whole point of the fixture.
fn build_probe(name: &str) -> PathBuf {
    let directory = probe_root();
    let binary = directory.join(name);
    if binary.exists() {
        return binary;
    }

    let source = directory.join("probe-stub.c");
    std::fs::write(&source, STUB_C).expect("write C stub");

    for compiler in ["cc", "clang", "gcc"] {
        let output = Command::new(compiler)
            .arg("-o")
            .arg(&binary)
            .arg(&source)
            .output();
        if let Ok(output) = output
            && output.status.success()
        {
            return binary;
        }
    }

    // No usable C compiler: rustc is present by definition when this test runs.
    let rust_source = directory.join("probe-stub.rs");
    std::fs::write(&rust_source, STUB_RS).expect("write Rust stub");
    let output = Command::new("rustc")
        .arg("-o")
        .arg(&binary)
        .arg(&rust_source)
        .output()
        .expect("rustc must be available to build the probe stub");
    assert!(
        output.status.success(),
        "rustc failed to build the probe stub: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    binary
}

/// A spawned stub that is killed when the test ends, including on panic.
struct Probe(Child);

impl Probe {
    fn spawn(binary: &Path, args: &[&str]) -> Self {
        let child = Command::new(binary)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn probe process");
        Self(child)
    }

    fn pid(&self) -> u32 {
        self.0.id()
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Poll `detect_agents` until `pid` appears, returning the whole snapshot that
/// contained it (so sibling assertions see one consistent view).
fn snapshot_containing(pid: u32, timeout: Duration) -> Vec<DetectedAgent> {
    let deadline = Instant::now() + timeout;
    loop {
        let agents = detect_agents();
        if agents.iter().any(|agent| agent.pid == pid) {
            return agents;
        }
        assert!(
            Instant::now() < deadline,
            "detect_agents never reported pid {pid} within {timeout:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

#[test]
fn detect_agents_reports_a_real_agent_process_with_its_argv_context() {
    let probe = build_probe("claude-probe");
    let child = Probe::spawn(
        &probe,
        &["--cwd", "/repos/AgilePlus-wtrees/my-feature", "WP13"],
    );

    let agents = snapshot_containing(child.pid(), Duration::from_secs(10));
    let detected = agents
        .iter()
        .find(|agent| agent.pid == child.pid())
        .expect("the probe pid must be reported");

    // The process name comes from the real executable, and the display name is
    // derived from it plus the last path segment of `--cwd`.
    assert_eq!(detected.process_name, "claude-probe");
    assert_eq!(detected.name, "claude-my-feature");
    assert_eq!(
        detected.worktree,
        Some("/repos/AgilePlus-wtrees/my-feature".to_string())
    );
    assert_eq!(detected.current_task, "Task: WP13");

    // A process created a moment ago has consumed no measurable CPU, and
    // `detect_agents` builds a fresh `System`, so its first sample is always
    // zero: the documented threshold classifies the probe as idle.
    assert_eq!(
        detected.status, "idle",
        "a freshly started idle process must not be reported as running"
    );

    // Start time is read from the live process table and formatted as elapsed.
    let started_at = detected
        .started_at
        .as_deref()
        .expect("a live process must report a start time");
    assert!(
        started_at.ends_with('s') && started_at.len() <= 3,
        "a process started seconds ago must report seconds, got {started_at:?}"
    );

    // Every reported process must genuinely match a pattern, and the list is
    // ordered by pid.
    for agent in &agents {
        let process_name = agent.process_name.to_lowercase();
        assert!(
            AGENT_PATTERNS
                .iter()
                .any(|pattern| process_name.contains(pattern)),
            "detect_agents reported a process that matches no agent pattern: {process_name}"
        );
    }
    assert!(
        agents.windows(2).all(|pair| pair[0].pid <= pair[1].pid),
        "detect_agents must sort by pid"
    );
}

#[test]
fn detect_agents_skips_processes_that_match_no_agent_pattern() {
    // Both stubs are compiled from the same source and differ only in file name,
    // so the only thing that can explain the difference in reporting is the
    // agent-pattern filter inside `detect_agents`.
    let agent_probe = build_probe("gemini-probe");
    let plain_probe = build_probe("plain-probe");

    let agent_child = Probe::spawn(&agent_probe, &["--cwd", "/repos/AgilePlus-wtrees/other"]);
    let plain_child = Probe::spawn(&plain_probe, &["--cwd", "/repos/AgilePlus-wtrees/other"]);

    let agents = snapshot_containing(agent_child.pid(), Duration::from_secs(10));

    let detected = agents
        .iter()
        .find(|agent| agent.pid == agent_child.pid())
        .expect("the gemini probe must be reported");
    assert_eq!(detected.process_name, "gemini-probe");
    assert_eq!(detected.name, "gemini-other");

    assert!(
        !agents.iter().any(|agent| agent.pid == plain_child.pid()),
        "a live process whose name matches no pattern must never be reported"
    );
}
