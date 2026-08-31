// Date & Time settings, ported from the Electron MVP's datetime.service.ts / lib/datetime/
// timedatectl.ts. Same shape as network.rs's nmcli wrapper and bluetooth.rs's bluetoothctl
// wrapper: parsing logic is pure and fully unit-tested; the binary name is threaded through as a
// parameter (not hardcoded) so tests can point it at `sh` to fabricate stdout/stderr/exit-code
// scenarios instead of needing a real timedatectl on the machine running them.

use std::process::Stdio;
use std::time::Duration;

use serde::Serialize;
use tokio::process::Command;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DateTimeStatus {
    pub timezone: String,
    // Whether automatic sync is turned on at all (`timedatectl set-ntp`'s own state).
    pub ntp_enabled: bool,
    // Whether it has actually succeeded in syncing at least once, which can lag behind
    // ntp_enabled right after toggling it on (no network, or the sync just hasn't landed yet) --
    // distinct enough that callers shouldn't collapse them into one boolean.
    pub ntp_synchronized: bool,
}

async fn run_timedatectl(timedatectl_bin: &str, args: &[&str], timeout: Duration) -> Result<std::process::Output, String> {
    let output_fut = Command::new(timedatectl_bin).args(args).stdin(Stdio::null()).output();
    match tokio::time::timeout(timeout, output_fut).await {
        Ok(Ok(output)) => Ok(output),
        // A missing binary means systemd itself isn't set up right -- an environment problem, not
        // a "try again" outcome.
        Ok(Err(e)) if e.kind() == std::io::ErrorKind::NotFound => Err("timedatectl not found -- is systemd installed?".to_string()),
        Ok(Err(e)) => Err(e.to_string()),
        Err(_elapsed) => Err("timedatectl timed out".to_string()),
    }
}

fn require_success(output: &std::process::Output) -> Result<(), String> {
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// `--property=<names> --value` (documented under `man timedatectl`'s -p/--value flags, the same
/// systemd "show" convention `systemctl show` uses) prints just the requested values, one per
/// line, in the order requested -- locale-independent and doesn't need regex over the
/// human-readable table `timedatectl` prints with no arguments.
pub fn parse_status(stdout: &str) -> DateTimeStatus {
    let mut lines = stdout.lines();
    DateTimeStatus {
        timezone: lines.next().unwrap_or_default().to_string(),
        ntp_enabled: lines.next() == Some("yes"),
        ntp_synchronized: lines.next() == Some("yes"),
    }
}

pub async fn get_status(timedatectl_bin: &str) -> Result<DateTimeStatus, String> {
    let output = run_timedatectl(timedatectl_bin, &["show", "--property=Timezone,NTP,NTPSynchronized", "--value"], COMMAND_TIMEOUT).await?;
    require_success(&output)?;
    Ok(parse_status(&String::from_utf8_lossy(&output.stdout)))
}

/// One IANA zone name per line -- the full list the timezone picker searches/filters over. This
/// never changes at runtime (it's the OS's own tzdata), so nothing here needs to react to it
/// changing mid-session.
pub async fn list_timezones(timedatectl_bin: &str) -> Result<Vec<String>, String> {
    let output = run_timedatectl(timedatectl_bin, &["list-timezones"], COMMAND_TIMEOUT).await?;
    require_success(&output)?;
    Ok(String::from_utf8_lossy(&output.stdout).lines().map(str::trim).filter(|l| !l.is_empty()).map(String::from).collect())
}

/// The timezone picker only ever offers a name that came from `list_timezones` above, so an
/// invalid zone reaching this function would mean an app bug, not an everyday user-facing outcome
/// worth a tagged result -- a plain error string is the right shape here.
pub async fn set_timezone(timedatectl_bin: &str, timezone: &str) -> Result<(), String> {
    let output = run_timedatectl(timedatectl_bin, &["set-timezone", timezone], COMMAND_TIMEOUT).await?;
    require_success(&output)
}

/// Fails if no time-sync service (systemd-timesyncd or equivalent) is installed/enabled on the
/// device -- an environment/provisioning problem, not a branch the UI needs to react to
/// differently.
pub async fn set_ntp_enabled(timedatectl_bin: &str, enabled: bool) -> Result<(), String> {
    let output = run_timedatectl(timedatectl_bin, &["set-ntp", if enabled { "true" } else { "false" }], COMMAND_TIMEOUT).await?;
    require_success(&output)
}

/// `date_time` must be a format `timedatectl set-time` accepts, e.g. "2026-08-27 21:15:00". The
/// frontend only shows this control when NTP is already off, so the one documented failure mode
/// this hits in practice -- "Automatic time synchronization is enabled" -- shouldn't come up in
/// normal use; a plain error string covers that and a malformed string alike.
pub async fn set_time(timedatectl_bin: &str, date_time: &str) -> Result<(), String> {
    let output = run_timedatectl(timedatectl_bin, &["set-time", date_time], COMMAND_TIMEOUT).await?;
    require_success(&output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_status_reads_the_three_requested_values_in_order() {
        let status = parse_status("Europe/London\nyes\nyes");
        assert_eq!(status, DateTimeStatus { timezone: "Europe/London".into(), ntp_enabled: true, ntp_synchronized: true });
    }

    #[test]
    fn parse_status_treats_ntp_off_or_not_yet_synchronized_as_false() {
        let status = parse_status("America/New_York\nno\nno");
        assert_eq!(status, DateTimeStatus { timezone: "America/New_York".into(), ntp_enabled: false, ntp_synchronized: false });
    }

    #[tokio::test]
    async fn require_success_passes_through_stdout_status_and_captures_stderr_on_failure() {
        let ok = run_timedatectl("sh", &["-c", "exit 0"], COMMAND_TIMEOUT).await.unwrap();
        assert!(require_success(&ok).is_ok());

        let failed = run_timedatectl("sh", &["-c", "echo boom >&2; exit 1"], COMMAND_TIMEOUT).await.unwrap();
        assert_eq!(require_success(&failed), Err("boom".to_string()));
    }

    #[tokio::test]
    async fn run_timedatectl_captures_stdout_on_success() {
        let output = run_timedatectl("sh", &["-c", "echo hello"], COMMAND_TIMEOUT).await.unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
    }

    #[tokio::test]
    async fn run_timedatectl_captures_stderr_on_a_nonzero_exit() {
        let output = run_timedatectl("sh", &["-c", "echo boom >&2; exit 1"], COMMAND_TIMEOUT).await.unwrap();
        assert!(!output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stderr).trim(), "boom");
    }

    #[tokio::test]
    async fn run_timedatectl_reports_a_clear_error_for_a_missing_binary() {
        let err = run_timedatectl("definitely-not-a-real-binary-xyz", &[], COMMAND_TIMEOUT).await.unwrap_err();
        assert!(err.contains("not found"));
    }

    #[tokio::test]
    async fn run_timedatectl_times_out_rather_than_hanging_forever() {
        let err = run_timedatectl("sh", &["-c", "sleep 5"], Duration::from_millis(50)).await.unwrap_err();
        assert!(err.contains("timed out"));
    }
}
