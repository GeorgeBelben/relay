use std::process::{ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tokio::process::Command;

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

/// Baseline emulator launch: spawn `command`/`args` via `tokio::process`, capture the exit code
/// once it quits. Only one launch may be in flight at a time -- unlike
/// `ingestion::pipeline::rescan`'s overlap guard (which silently no-ops a duplicate call),
/// `running` here returns `Err(AlreadyRunning)` instead, since the caller (the "Launch" button)
/// needs to know the request was rejected, matching the MVP's `throw new Error("A game is
/// already running")`. Ported from `launcher.service.ts#launchGame`'s baseline spawn/exit
/// handling -- minus RetroAchievements config injection (no RA integration exists in this
/// rewrite yet) and minus stdout/stderr capture (REL-89).
pub async fn launch(
    command: &str,
    args: &[String],
    running: &AtomicBool,
    mut on_status: impl FnMut(LauncherStatus),
) -> Result<ExitStatus, LaunchError> {
    if running.swap(true, Ordering::SeqCst) {
        return Err(LaunchError::AlreadyRunning);
    }

    on_status(LauncherStatus::Launching);

    let spawned = Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    let mut child = match spawned {
        Ok(child) => child,
        Err(e) => {
            running.store(false, Ordering::SeqCst);
            on_status(LauncherStatus::Error { message: format!("Couldn't start {command}: {e}") });
            return Err(LaunchError::Spawn(e));
        }
    };

    on_status(LauncherStatus::Running);

    let result = child.wait().await;
    running.store(false, Ordering::SeqCst);

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
        let mut statuses = Vec::new();

        let exit_status = launch(
            "sh",
            &["-c".to_string(), "exit 0".to_string()],
            &running,
            |s| statuses.push(s),
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
        let mut statuses = Vec::new();

        let exit_status = launch(
            "sh",
            &["-c".to_string(), "exit 42".to_string()],
            &running,
            |s| statuses.push(s),
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
        let mut statuses = Vec::new();

        // Sends SIGKILL to itself -- deterministic, no real crash needed to prove the plumbing.
        launch("sh", &["-c".to_string(), "kill -9 $$".to_string()], &running, |s| statuses.push(s))
            .await
            .unwrap();

        assert_eq!(statuses.len(), 3);
        assert_eq!(statuses[2], LauncherStatus::Crashed { exit_code: None, signal: Some(9) });
    }

    #[tokio::test]
    async fn returns_already_running_without_touching_status_when_a_launch_is_in_flight() {
        let running = AtomicBool::new(true); // simulate an in-flight launch
        let mut statuses = Vec::new();

        let err = launch("sh", &["-c".to_string(), "exit 0".to_string()], &running, |s| statuses.push(s))
            .await
            .unwrap_err();

        assert!(matches!(err, LaunchError::AlreadyRunning));
        assert!(statuses.is_empty());
        assert!(running.load(Ordering::SeqCst)); // untouched, still true
    }

    #[tokio::test]
    async fn missing_binary_reports_launching_then_error_and_clears_the_running_flag() {
        let running = AtomicBool::new(false);
        let mut statuses = Vec::new();

        let err = launch("definitely-not-a-real-binary-xyz", &[], &running, |s| statuses.push(s)).await.unwrap_err();

        assert!(matches!(err, LaunchError::Spawn(_)));
        assert_eq!(statuses.len(), 2);
        assert_eq!(statuses[0], LauncherStatus::Launching);
        assert!(matches!(&statuses[1], LauncherStatus::Error { .. }));
        assert!(!running.load(Ordering::SeqCst));
    }
}
