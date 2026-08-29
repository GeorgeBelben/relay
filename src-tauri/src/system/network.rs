// WiFi settings (REL-110), ported from the Electron MVP's network.service.ts / lib/network/
// nmcli.ts. Parsing/classification logic is pure and fully unit-tested; the actual `nmcli`
// execution (list_wifi_networks/connect_to_wifi_network) is thin glue over it and, like
// bluetoothctl (REL-111) and the controller-input work (REL-91), needs real Linux hardware with
// NetworkManager to verify end to end -- there's no WiFi adapter or nmcli in this dev environment.

use std::process::Stdio;
use std::time::Duration;

use serde::Serialize;
use tokio::process::Command;

// nmcli's own default association wait is 90s (`man nmcli`) -- far too long to leave the Settings
// UI spinning on a wrong password. --wait caps nmcli's own wait; the timeout below is a hard
// backstop in case nmcli doesn't respect it (e.g. it hangs pre-association).
const CONNECT_WAIT_SECONDS: u64 = 20;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(CONNECT_WAIT_SECONDS + 5);
// A scan is a real over-the-air operation, not a local lookup -- nmcli itself only guarantees the
// AP list is "no older than 30s" for a plain `device wifi list`, so --rescan yes (used below) can
// take a few seconds; this is just a ceiling for a hung radio/driver.
const SCAN_TIMEOUT: Duration = Duration::from_secs(15);

// nmcli's terse (-t) mode escapes literal ':' and '\' inside field values as '\:' and '\\' (see
// `man nmcli`'s -e/--escape, on by default in terse mode) -- SSIDs are free text and can
// legitimately contain a colon, so a naive `line.split(':')` would silently corrupt those. Walks
// the string once, splitting on an unescaped ':' and un-escaping each field.
pub fn parse_terse_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            ':' => fields.push(std::mem::take(&mut current)),
            _ => current.push(c),
        }
    }
    fields.push(current);
    fields
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: i64,
    pub secured: bool,
    pub in_use: bool,
}

// Parses `nmcli -t -f IN-USE,SSID,SIGNAL,SECURITY device wifi list` terse output into one row per
// network name, sorted strongest-first.
pub fn parse_wifi_list(stdout: &str) -> Vec<WifiNetwork> {
    let mut by_ssid: std::collections::HashMap<String, WifiNetwork> = std::collections::HashMap::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = parse_terse_line(line);
        let ssid = fields.get(1).cloned().unwrap_or_default();
        if ssid.is_empty() {
            continue; // a hidden network broadcasts an empty SSID -- nothing to show or select
        }
        let network = WifiNetwork {
            ssid: ssid.clone(),
            signal: fields.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
            secured: fields.get(3).is_some_and(|s| !s.is_empty()),
            in_use: fields.first().map(String::as_str) == Some("*"),
        };

        // A single SSID can appear once per physical AP in range (mesh/repeaters, dual-band) --
        // one row per network name, keeping whichever is strongest, except the in-use AP always
        // wins regardless of signal (it's the one actually worth calling "connected").
        let should_replace = match by_ssid.get(&ssid) {
            None => true,
            Some(existing) => network.in_use || (!existing.in_use && network.signal > existing.signal),
        };
        if should_replace {
            by_ssid.insert(ssid, network);
        }
    }

    let mut networks: Vec<WifiNetwork> = by_ssid.into_values().collect();
    networks.sort_by(|a, b| b.signal.cmp(&a.signal));
    networks
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "reason", rename_all = "kebab-case")]
pub enum WifiConnectError {
    WrongPassword { message: String },
    Unreachable { message: String },
    Unknown { message: String },
}

// Confirmed error text from NetworkManager bug reports/forum threads for a rejected PSK -- not
// verified against real hardware from this repo (no WiFi adapter in the dev environment).
// "unreachable" similarly best-effort matches nmcli's own "no network with SSID" wording for an
// AP that's out of range or was never actually seen in the last scan. Anything else falls through
// to "unknown" with nmcli's own message rather than guessing further.
pub fn classify_connect_failure(stderr: &str) -> WifiConnectError {
    if stderr.contains("Secrets were required") {
        return WifiConnectError::WrongPassword { message: "Incorrect password".to_string() };
    }
    if stderr.to_lowercase().contains("no network with ssid") {
        return WifiConnectError::Unreachable { message: "Network not found -- it may be out of range".to_string() };
    }
    WifiConnectError::Unknown { message: stderr.trim().to_string() }
}

async fn run_nmcli(nmcli_bin: &str, args: &[&str], timeout: Duration) -> Result<std::process::Output, String> {
    let output_fut = Command::new(nmcli_bin).args(args).stdin(Stdio::null()).output();
    match tokio::time::timeout(timeout, output_fut).await {
        Ok(Ok(output)) => Ok(output),
        // A missing binary means NetworkManager itself isn't installed/set up right -- an
        // environment problem, not a "try again" outcome.
        Ok(Err(e)) if e.kind() == std::io::ErrorKind::NotFound => {
            Err("nmcli not found -- is NetworkManager installed?".to_string())
        }
        Ok(Err(e)) => Err(e.to_string()),
        Err(_elapsed) => Err("nmcli timed out".to_string()),
    }
}

// --rescan yes forces a fresh over-the-air scan rather than serving nmcli's own <=30s-old cache --
// this is always invoked from an explicit user action (a "Scan for Networks" button), never a
// background poll, so paying for a real scan every time is the right default.
pub async fn list_wifi_networks(nmcli_bin: &str) -> Result<Vec<WifiNetwork>, String> {
    let output = run_nmcli(
        nmcli_bin,
        &["-t", "-f", "IN-USE,SSID,SIGNAL,SECURITY", "device", "wifi", "list", "--rescan", "yes"],
        SCAN_TIMEOUT,
    )
    .await?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(parse_wifi_list(&String::from_utf8_lossy(&output.stdout)))
}

pub async fn connect_to_wifi_network(nmcli_bin: &str, ssid: &str, password: Option<&str>) -> Result<(), WifiConnectError> {
    let mut args: Vec<String> = vec!["device".into(), "wifi".into(), "connect".into(), ssid.into()];
    if let Some(pw) = password {
        args.push("password".into());
        args.push(pw.into());
    }
    args.push("--wait".into());
    args.push(CONNECT_WAIT_SECONDS.to_string());
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();

    let output = run_nmcli(nmcli_bin, &arg_refs, CONNECT_TIMEOUT)
        .await
        .map_err(|message| WifiConnectError::Unknown { message })?;

    if output.status.success() {
        return Ok(());
    }
    Err(classify_connect_failure(&String::from_utf8_lossy(&output.stderr)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_terse_line_splits_on_unescaped_colons() {
        assert_eq!(parse_terse_line("*:HomeWifi:80:WPA2"), vec!["*", "HomeWifi", "80", "WPA2"]);
    }

    #[test]
    fn parse_terse_line_unescapes_a_colon_inside_a_field() {
        assert_eq!(parse_terse_line(r"*:Joe\:s Wifi:80:WPA2"), vec!["*", "Joe:s Wifi", "80", "WPA2"]);
    }

    #[test]
    fn parse_terse_line_unescapes_a_literal_backslash() {
        assert_eq!(parse_terse_line(r"*:Back\\slash:80:"), vec!["*", r"Back\slash", "80", ""]);
    }

    #[test]
    fn parse_wifi_list_skips_hidden_networks_with_empty_ssid() {
        let networks = parse_wifi_list("*::80:WPA2\n");
        assert!(networks.is_empty());
    }

    #[test]
    fn parse_wifi_list_keeps_the_strongest_ap_for_a_duplicate_ssid() {
        let stdout = ":Office:40:WPA2\n:Office:75:WPA2\n:Office:60:WPA2\n";
        let networks = parse_wifi_list(stdout);
        assert_eq!(networks.len(), 1);
        assert_eq!(networks[0].signal, 75);
    }

    #[test]
    fn parse_wifi_list_prefers_the_in_use_ap_even_if_weaker() {
        let stdout = ":Office:90:WPA2\n*:Office:20:WPA2\n";
        let networks = parse_wifi_list(stdout);
        assert_eq!(networks.len(), 1);
        assert!(networks[0].in_use);
        assert_eq!(networks[0].signal, 20);
    }

    #[test]
    fn parse_wifi_list_sorts_strongest_first_and_reports_secured() {
        let stdout = ":Weak:10:\n:Strong:90:WPA2\n";
        let networks = parse_wifi_list(stdout);
        assert_eq!(networks[0].ssid, "Strong");
        assert!(networks[0].secured);
        assert_eq!(networks[1].ssid, "Weak");
        assert!(!networks[1].secured);
    }

    #[test]
    fn classify_connect_failure_recognizes_a_rejected_password() {
        let err = classify_connect_failure("Error: Secrets were required, but not provided.");
        assert!(matches!(err, WifiConnectError::WrongPassword { .. }));
    }

    #[test]
    fn classify_connect_failure_recognizes_an_out_of_range_network_case_insensitively() {
        let err = classify_connect_failure("Error: No network with SSID 'Foo' found.");
        assert!(matches!(err, WifiConnectError::Unreachable { .. }));
    }

    #[test]
    fn classify_connect_failure_falls_back_to_unknown_with_the_raw_message() {
        let err = classify_connect_failure("some other nmcli error\n");
        assert_eq!(err, WifiConnectError::Unknown { message: "some other nmcli error".to_string() });
    }

    #[tokio::test]
    async fn run_nmcli_captures_stdout_on_success() {
        let output = run_nmcli("sh", &["-c", "echo hello"], Duration::from_secs(5)).await.unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
    }

    #[tokio::test]
    async fn run_nmcli_captures_stderr_on_a_nonzero_exit() {
        let output = run_nmcli("sh", &["-c", "echo boom >&2; exit 1"], Duration::from_secs(5)).await.unwrap();
        assert!(!output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stderr).trim(), "boom");
    }

    #[tokio::test]
    async fn run_nmcli_reports_a_clear_error_for_a_missing_binary() {
        let err = run_nmcli("definitely-not-a-real-binary-xyz", &[], Duration::from_secs(5)).await.unwrap_err();
        assert!(err.contains("not found"));
    }

    #[tokio::test]
    async fn run_nmcli_times_out_rather_than_hanging_forever() {
        let err = run_nmcli("sh", &["-c", "sleep 5"], Duration::from_millis(50)).await.unwrap_err();
        assert!(err.contains("timed out"));
    }
}
