// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared process boundary for invoking the installed AgilePlus CLI.
//!
//! The desktop and tray must resolve the CLI portably and enforce a real
//! timeout. They never invoke a shell, so user-provided arguments are passed as
//! discrete process arguments rather than interpolated command text.

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const CLI_PATH_ENV: &str = "AGILEPLUS_BIN";
const POLL_INTERVAL: Duration = Duration::from_millis(20);
const ERROR_OUTPUT_LIMIT: usize = 2_000;

#[derive(Debug, PartialEq, Eq)]
pub struct CliProcessOutput {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Locate the AgilePlus CLI without embedding a machine-specific path.
///
/// Resolution order:
/// 1. explicit `AGILEPLUS_BIN` path;
/// 2. a sibling binary next to the running desktop executable;
/// 3. the process `PATH`.
pub fn resolve_agileplus_binary() -> Result<PathBuf, String> {
    let configured = env::var_os(CLI_PATH_ENV);
    let current_exe = env::current_exe().ok();
    let search_path = env::var_os("PATH");
    resolve_agileplus_binary_from(
        configured.as_deref(),
        current_exe.as_deref(),
        search_path.as_deref(),
    )
}

fn resolve_agileplus_binary_from(
    configured: Option<&OsStr>,
    current_exe: Option<&Path>,
    search_path: Option<&OsStr>,
) -> Result<PathBuf, String> {
    if let Some(configured) = configured {
        let candidate = PathBuf::from(configured);
        return executable_candidate(candidate).ok_or_else(|| {
            format!(
                "{CLI_PATH_ENV} points to a missing or non-executable file: {}",
                Path::new(configured).display()
            )
        });
    }

    let binary_name = default_binary_name();
    if let Some(candidate) = current_exe
        .and_then(Path::parent)
        .map(|parent| parent.join(binary_name))
        .and_then(executable_candidate)
    {
        return Ok(candidate);
    }

    if let Some(candidate) = search_path
        .into_iter()
        .flat_map(env::split_paths)
        .map(|directory| directory.join(binary_name))
        .find_map(executable_candidate)
    {
        return Ok(candidate);
    }

    Err(format!(
        "Unable to locate the AgilePlus CLI. Install '{binary_name}' beside the desktop app or on PATH, or set {CLI_PATH_ENV} to its full path."
    ))
}

fn default_binary_name() -> &'static str {
    if cfg!(windows) {
        "agileplus.exe"
    } else {
        "agileplus"
    }
}

fn executable_candidate(path: PathBuf) -> Option<PathBuf> {
    if !is_executable_file(&path) {
        return None;
    }
    Some(fs::canonicalize(&path).unwrap_or(path))
}

fn is_executable_file(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    {
        true
    }
}

/// Execute the installed AgilePlus CLI in one selected project root.
pub fn run_agileplus(
    args: &[String],
    repo_path: &Path,
    timeout: Duration,
) -> Result<CliProcessOutput, String> {
    if timeout.is_zero() {
        return Err("AgilePlus CLI timeout must be greater than zero".into());
    }
    if !repo_path.is_dir() {
        return Err(format!(
            "Selected AgilePlus project root is not a directory: {}",
            repo_path.display()
        ));
    }

    let binary = resolve_agileplus_binary()?;
    run_binary(&binary, args, repo_path, timeout)
}

fn run_binary(
    binary: &Path,
    args: &[String],
    repo_path: &Path,
    timeout: Duration,
) -> Result<CliProcessOutput, String> {
    let mut child = Command::new(binary)
        .args(args)
        .current_dir(repo_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            format!(
                "Failed to start AgilePlus CLI '{}': {error}",
                binary.display()
            )
        })?;

    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err("AgilePlus CLI stdout pipe was unavailable".into());
        }
    };
    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err("AgilePlus CLI stderr pipe was unavailable".into());
        }
    };

    let stdout_reader = spawn_reader(stdout);
    let stderr_reader = spawn_reader(stderr);
    let started = Instant::now();

    let outcome = loop {
        match child.try_wait() {
            Ok(Some(status)) => break WaitOutcome::Exited(status),
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                break WaitOutcome::TimedOut;
            }
            Ok(None) => {
                let remaining = timeout.saturating_sub(started.elapsed());
                thread::sleep(std::cmp::min(POLL_INTERVAL, remaining));
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                break WaitOutcome::Failed(error.to_string());
            }
        }
    };

    let stdout = finish_reader(stdout_reader, "stdout")?;
    let stderr = finish_reader(stderr_reader, "stderr")?;
    let stdout = String::from_utf8_lossy(&stdout).into_owned();
    let stderr = String::from_utf8_lossy(&stderr).into_owned();

    match outcome {
        WaitOutcome::Exited(status) => Ok(CliProcessOutput {
            code: status.code().unwrap_or(-1),
            stdout,
            stderr,
        }),
        WaitOutcome::TimedOut => Err(format!(
            "AgilePlus CLI timed out after {} ms. stdout: {} stderr: {}",
            timeout.as_millis(),
            output_preview(&stdout),
            output_preview(&stderr)
        )),
        WaitOutcome::Failed(error) => Err(format!(
            "Failed while waiting for AgilePlus CLI: {error}. stdout: {} stderr: {}",
            output_preview(&stdout),
            output_preview(&stderr)
        )),
    }
}

enum WaitOutcome {
    Exited(ExitStatus),
    TimedOut,
    Failed(String),
}

fn spawn_reader<R>(mut reader: R) -> thread::JoinHandle<std::io::Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Ok(bytes)
    })
}

fn finish_reader(
    reader: thread::JoinHandle<std::io::Result<Vec<u8>>>,
    stream_name: &str,
) -> Result<Vec<u8>, String> {
    reader
        .join()
        .map_err(|_| format!("AgilePlus CLI {stream_name} reader panicked"))?
        .map_err(|error| format!("Failed reading AgilePlus CLI {stream_name}: {error}"))
}

fn output_preview(output: &str) -> String {
    let preview: String = output.chars().take(ERROR_OUTPUT_LIMIT).collect();
    if output.chars().count() > ERROR_OUTPUT_LIMIT {
        format!("{preview}…")
    } else {
        preview
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let suffix = COUNTER.fetch_add(1, Ordering::SeqCst);
            let path = env::temp_dir().join(format!(
                "agileplus-cli-runtime-{name}-{}-{suffix}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create scratch directory");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn executable(&self, body: &str) -> PathBuf {
            let path = self.path().join(default_binary_name());
            fs::write(&path, body).expect("write executable fixture");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
                    .expect("mark fixture executable");
            }
            path
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn configured_binary_is_authoritative_and_must_exist() {
        let scratch = Scratch::new("configured-missing");
        let configured = scratch.path().join("missing-agileplus");
        let error = resolve_agileplus_binary_from(
            Some(configured.as_os_str()),
            None,
            Some(scratch.path().as_os_str()),
        )
        .expect_err("invalid explicit configuration must fail closed");
        assert!(error.contains(CLI_PATH_ENV));
        assert!(error.contains("missing or non-executable"));
    }

    #[test]
    fn sibling_binary_precedes_path_search() {
        let sibling_dir = Scratch::new("sibling");
        let path_dir = Scratch::new("path");
        let sibling = sibling_dir.executable("");
        let _path_candidate = path_dir.executable("");
        let current_exe = sibling_dir.path().join("agileplus-desktop-test");

        let resolved = resolve_agileplus_binary_from(
            None,
            Some(&current_exe),
            Some(path_dir.path().as_os_str()),
        )
        .expect("resolve sibling binary");
        assert_eq!(resolved, fs::canonicalize(sibling).unwrap());
    }

    #[test]
    fn path_search_finds_installed_cli() {
        let scratch = Scratch::new("path-search");
        let expected = scratch.executable("");
        let resolved = resolve_agileplus_binary_from(None, None, Some(scratch.path().as_os_str()))
            .expect("resolve PATH binary");
        assert_eq!(resolved, fs::canonicalize(expected).unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn process_output_and_exit_code_are_preserved() {
        let scratch = Scratch::new("output");
        let binary =
            scratch.executable("#!/bin/sh\nprintf 'hello-out'\nprintf 'hello-err' >&2\nexit 7\n");
        let output = run_binary(&binary, &[], scratch.path(), Duration::from_secs(2))
            .expect("run executable fixture");
        assert_eq!(output.code, 7);
        assert_eq!(output.stdout, "hello-out");
        assert_eq!(output.stderr, "hello-err");
    }

    #[cfg(unix)]
    #[test]
    fn timeout_terminates_the_cli() {
        let scratch = Scratch::new("timeout");
        let binary = scratch.executable("#!/bin/sh\nexec sleep 5\n");
        let started = Instant::now();
        let error = run_binary(&binary, &[], scratch.path(), Duration::from_millis(50))
            .expect_err("long-running process must time out");
        assert!(error.contains("timed out"));
        assert!(started.elapsed() < Duration::from_secs(2));
    }
}
