use std::process::{ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use serde::Serialize;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;

use crate::db::time::now_unix;

// Pushed to the frontend as the emulator process starts, runs, and exits. "launching" covers the
// spawn call itself; "running" fires once the OS confirms the process actually started (spawn()
// succeeding), not just that it was called. Originally mirrored the Electron MVP's
// shared/types.ts#LauncherStatus exactly (which only had "exited", firing on any exit regardless
// of code -- REL-36 in the old project never distinguished a clean quit from a crash), but
// REL-88 splits that into "exited" (a zero exit code -- the emulator's own quit path) and
// "crashed" (anything else: a nonzero exit code, or terminated by a signal on Unix) so the UI can
// show a real error state instead of silently returning to Home either way, which is what "don't
// leave the kiosk frozen" actually requires -- a frozen *process* is caught by "running" simply
// never transitioning, but a process that's already gone with no thrown JS error (the old
// Electron model) needs this distinction to be visible at all.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum LauncherStatus {
    Idle,
    Launching,
    Running,
    Exited,
    Crashed { exit_code: Option<i32>, signal: Option<i32> },
    Error { message: String },
}

// The MVP never captured this at all (`spawn(command, args, { stdio: "ignore" })`) -- a RetroArch
// core-load failure or similar just vanished. Kept as discrete, sourced, timestamped lines
// (rather than raw undifferentiated bytes) so a caller can log/display them meaningfully; this
// deliberately doesn't try to pattern-match specific failure strings (e.g. a particular core's
// "failed to load" wording) since that's fragile and unverified without a real RetroArch install
// to check the actual text against -- "surfacing" a core-load failure here means the operator can
// now *see* it at all, not that it's classified automatically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LogLine {
    pub stream: LogStream,
    pub line: String,
    pub timestamp: i64,
}

async fn pump_lines<R: AsyncRead + Unpin>(reader: R, stream: LogStream, on_log: &impl Fn(LogLine)) {
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        on_log(LogLine { stream, line, timestamp: now_unix() });
    }
}

#[cfg(unix)]
fn signal_of(status: &ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.signal()
}

#[cfg(not(unix))]
fn signal_of(_status: &ExitStatus) -> Option<i32> {
    None
}

#[derive(Debug)]
pub enum LaunchError {
    AlreadyRunning,
    Spawn(std::io::Error),
    Wait(std::io::Error),
}

impl std::fmt::Display for LaunchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRunning => write!(f, "a game is already running"),
            Self::Spawn(e) => write!(f, "failed to spawn process: {e}"),
            Self::Wait(e) => write!(f, "failed to wait for process exit: {e}"),
        }
    }
}

impl std::error::Error for LaunchError {}

#[derive(Debug)]
pub enum KillError {
    NotRunning,
    Signal(std::io::Error),
}

impl std::fmt::Display for KillError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotRunning => write!(f, "no game is currently running"),
            Self::Signal(e) => write!(f, "failed to signal process: {e}"),
        }
    }
}

impl std::error::Error for KillError {}

/// Sends SIGTERM to the process tracked in `active_pid` (0 means none), if any -- RetroArch/
/// PCSX2/Dolphin all handle a graceful terminate; nothing here escalates to SIGKILL, since that
/// risks corrupting a save the emulator hasn't finished flushing. Deliberately signals by raw PID
/// (via `libc::kill`) rather than holding the `tokio::process::Child` behind a shared lock that
/// both this and `launch`'s `.wait()` would need -- `wait()` holds its lock for the entire time
/// the process is running, which would make a concurrent `kill()` block until the process exits
/// on its own, defeating the point. New Linear issue if this ever needs to target Windows (no
/// signals there).
#[cfg(unix)]
pub fn kill(active_pid: &AtomicU32) -> Result<(), KillError> {
    let pid = active_pid.load(Ordering::SeqCst);
    if pid == 0 {
        return Err(KillError::NotRunning);
    }
    // SAFETY: libc::kill with a pid read from our own tracked child and a standard signal number
    // is safe to call; a pid that's already exited (a racing, already-cleared slot) just returns
    // ESRCH, surfaced as a normal Err rather than UB.
    let result = unsafe { libc::kill(pid as libc::pid_t, libc::SIGTERM) };
    if result == 0 {
        Ok(())
    } else {
        Err(KillError::Signal(std::io::Error::last_os_error()))
    }
}

/// Baseline emulator launch: spawn `command`/`args` via `tokio::process`, capture stdout/stderr
/// as structured log lines and the exit code once it quits. Only one launch may be in flight at a
/// time -- unlike `ingestion::pipeline::rescan`'s overlap guard (which silently no-ops a
/// duplicate call), `running` here returns `Err(AlreadyRunning)` instead, since the caller (the
/// "Launch" button) needs to know the request was rejected, matching the MVP's `throw new
/// Error("A game is already running")`. `active_pid` is set to the child's OS PID once spawned
/// (and cleared back to 0 on exit) so a concurrent call to `kill()` can reach it. Ported from
/// `launcher.service.ts#launchGame`'s baseline spawn/exit handling -- minus RetroAchievements
/// config injection (no RA integration exists in this rewrite yet).
pub async fn launch(
    command: &str,
    args: &[String],
    running: &AtomicBool,
    active_pid: &AtomicU32,
    mut on_status: impl FnMut(LauncherStatus),
    on_log: impl Fn(LogLine),
) -> Result<ExitStatus, LaunchError> {
    if running.swap(true, Ordering::SeqCst) {
        return Err(LaunchError::AlreadyRunning);
    }

    on_status(LauncherStatus::Launching);

    let spawned = Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match spawned {
        Ok(child) => child,
        Err(e) => {
            running.store(false, Ordering::SeqCst);
            on_status(LauncherStatus::Error { message: format!("Couldn't start {command}: {e}") });
            return Err(LaunchError::Spawn(e));
        }
    };

    active_pid.store(child.id().unwrap_or(0), Ordering::SeqCst);
    on_status(LauncherStatus::Running);

    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");

    let (_, _, result) = tokio::join!(
        pump_lines(stdout, LogStream::Stdout, &on_log),
        pump_lines(stderr, LogStream::Stderr, &on_log),
        child.wait(),
    );
    running.store(false, Ordering::SeqCst);
    active_pid.store(0, Ordering::SeqCst);

    match result {
        Ok(exit_status) => {
            if exit_status.success() {
                on_status(LauncherStatus::Exited);
            } else {
                on_status(LauncherStatus::Crashed {
                    exit_code: exit_status.code(),
                    signal: signal_of(&exit_status),
                });
            }
            Ok(exit_status)
        }
        Err(e) => {
            on_status(LauncherStatus::Error { message: format!("Error waiting for {command} to exit: {e}") });
            Err(LaunchError::Wait(e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn reports_launching_then_running_then_exited_on_a_clean_zero_exit() {
        let running = AtomicBool::new(false);
        let active_pid = AtomicU32::new(0);
        let mut statuses = Vec::new();

        let exit_status = launch(
            "sh",
            &["-c".to_string(), "exit 0".to_string()],
            &running,
            &active_pid,
            |s| statuses.push(s),
            |_| {},
        )
        .await
        .unwrap();

        assert_eq!(statuses, vec![LauncherStatus::Launching, LauncherStatus::Running, LauncherStatus::Exited]);
        assert_eq!(exit_status.code(), Some(0));
        assert!(!running.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn nonzero_exit_code_is_reported_as_crashed() {
        let running = AtomicBool::new(false);
        let active_pid = AtomicU32::new(0);
        let mut statuses = Vec::new();

        let exit_status = launch(
            "sh",
            &["-c".to_string(), "exit 42".to_string()],
            &running,
            &active_pid,
            |s| statuses.push(s),
            |_| {},
        )
        .await
        .unwrap();

        assert_eq!(
            statuses,
            vec![
                LauncherStatus::Launching,
                LauncherStatus::Running,
                LauncherStatus::Crashed { exit_code: Some(42), signal: None },
            ]
        );
        assert_eq!(exit_status.code(), Some(42));
    }

    #[tokio::test]
    async fn signal_terminated_process_is_reported_as_crashed_with_the_signal() {
        let running = AtomicBool::new(false);
        let active_pid = AtomicU32::new(0);
        let mut statuses = Vec::new();

        // Sends SIGKILL to itself -- deterministic, no real crash needed to prove the plumbing.
        launch("sh", &["-c".to_string(), "kill -9 $$".to_string()], &running, &active_pid, |s| statuses.push(s), |_| {})
            .await
            .unwrap();

        assert_eq!(statuses.len(), 3);
        assert_eq!(statuses[2], LauncherStatus::Crashed { exit_code: None, signal: Some(9) });
    }

    #[tokio::test]
    async fn returns_already_running_without_touching_status_when_a_launch_is_in_flight() {
        let running = AtomicBool::new(true); // simulate an in-flight launch
        let active_pid = AtomicU32::new(0);
        let mut statuses = Vec::new();

        let err = launch("sh", &["-c".to_string(), "exit 0".to_string()], &running, &active_pid, |s| statuses.push(s), |_| {})
            .await
            .unwrap_err();

        assert!(matches!(err, LaunchError::AlreadyRunning));
        assert!(statuses.is_empty());
        assert!(running.load(Ordering::SeqCst)); // untouched, still true
    }

    #[tokio::test]
    async fn missing_binary_reports_launching_then_error_and_clears_the_running_flag() {
        let running = AtomicBool::new(false);
        let active_pid = AtomicU32::new(0);
        let mut statuses = Vec::new();

        let err = launch("definitely-not-a-real-binary-xyz", &[], &running, &active_pid, |s| statuses.push(s), |_| {}).await.unwrap_err();

        assert!(matches!(err, LaunchError::Spawn(_)));
        assert_eq!(statuses.len(), 2);
        assert_eq!(statuses[0], LauncherStatus::Launching);
        assert!(matches!(&statuses[1], LauncherStatus::Error { .. }));
        assert!(!running.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn captures_stdout_and_stderr_as_structured_log_lines() {
        use std::sync::Mutex;

        let running = AtomicBool::new(false);
        let active_pid = AtomicU32::new(0);
        let logs = Mutex::new(Vec::new());

        launch(
            "sh",
            &["-c".to_string(), "echo out-line; echo err-line >&2".to_string()],
            &running,
            &active_pid,
            |_| {},
            |line| logs.lock().unwrap().push(line),
        )
        .await
        .unwrap();

        let logs = logs.into_inner().unwrap();
        assert_eq!(logs.len(), 2);
        assert!(logs.iter().any(|l| l.stream == LogStream::Stdout && l.line == "out-line"));
        assert!(logs.iter().any(|l| l.stream == LogStream::Stderr && l.line == "err-line"));
    }

    #[tokio::test]
    async fn kill_returns_not_running_when_nothing_is_active() {
        let active_pid = AtomicU32::new(0);
        assert!(matches!(kill(&active_pid), Err(KillError::NotRunning)));
    }

    #[tokio::test]
    async fn kill_sends_sigterm_and_the_process_can_observe_and_act_on_it() {
        let running = AtomicBool::new(false);
        let active_pid = AtomicU32::new(0);
        let mut statuses = Vec::new();

        // Traps TERM and exits with a distinct code -- proves the signal was actually delivered
        // and handled, not just that the process happened to die around the same time.
        let args = vec!["-c".to_string(), "trap 'exit 15' TERM; sleep 5".to_string()];
        let (exit_result, _) = tokio::join!(
            launch("sh", &args, &running, &active_pid, |s| statuses.push(s), |_| {}),
            async {
                // Give the shell a moment to install the trap before signaling it.
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                kill(&active_pid).unwrap();
            },
        );

        assert_eq!(exit_result.unwrap().code(), Some(15));
        assert_eq!(active_pid.load(Ordering::SeqCst), 0);
    }
}
