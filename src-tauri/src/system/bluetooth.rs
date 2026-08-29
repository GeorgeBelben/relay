// Bluetooth settings (REL-111), ported from the Electron MVP's bluetooth.service.ts / lib/
// bluetooth/bluetoothctl.ts. Parsing/classification logic is pure and fully unit-tested; the
// actual `bluetoothctl` execution is thin glue over it and, like network.rs's nmcli work and the
// controller-input work (REL-91), needs real Linux hardware with a Bluetooth adapter to verify
// end to end.
//
// Pairing carries a real constraint beyond app code: BlueZ needs a NoInputNoOutput ("Just Works")
// agent already registered system-wide via a `bt-agent` systemd service set up during device
// provisioning -- without one, BlueZ has nothing to ask and the pair request fails outright. This
// module doesn't register one itself, same as the MVP; every call here stays a plain one-shot
// `bluetoothctl <command> <args>` invocation.

use std::process::Stdio;
use std::time::Duration;

use serde::Serialize;
use tokio::process::Command;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);
const PAIR_TIMEOUT: Duration = Duration::from_secs(20);
const SCAN_SECONDS: u64 = 10;
const SCAN_TIMEOUT: Duration = Duration::from_secs(SCAN_SECONDS + 5);

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BluetoothDevice {
    pub address: String,
    pub name: String,
    pub paired: bool,
    pub connected: bool,
    pub battery_percent: Option<u32>,
}

// Matches a "Device XX:XX:XX:XX:XX:XX Some Name" line, with or without a leading "[NEW] "/
// "[CHG] " scan-event tag -- both `bluetoothctl devices` (plain "Device ...") and live `scan on`
// chatter (tagged) use this same core shape for a device entry.
pub fn parse_device_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    let rest = if let Some(stripped) = line.strip_prefix('[') {
        let (_, after) = stripped.split_once(']')?;
        after.trim_start()
    } else {
        line
    };
    let rest = rest.strip_prefix("Device ")?;
    let (address, name) = rest.split_once(' ')?;

    let is_mac = address.len() == 17
        && address.split(':').count() == 6
        && address.split(':').all(|group| group.len() == 2 && group.chars().all(|c| c.is_ascii_hexdigit()));
    if !is_mac || name.is_empty() {
        return None;
    }

    Some((address.to_string(), name.to_string()))
}

// Parses a `bluetoothctl info <address>` field, e.g. pulling "yes" out of a "Paired: yes" line, or
// "0x54 (84)" out of "Battery Percentage: 0x54 (84)".
fn parse_info_field<'a>(stdout: &'a str, field: &str) -> Option<&'a str> {
    let prefix = format!("{field}:");
    stdout.lines().find_map(|line| line.trim().strip_prefix(&prefix)).map(str::trim)
}

// Battery Percentage's value looks like "0x54 (84)" -- the decimal in parens is what's wanted.
fn parse_battery_percent(field_value: &str) -> Option<u32> {
    let open = field_value.find('(')?;
    let close = field_value[open..].find(')')? + open;
    field_value[open + 1..close].trim().parse().ok()
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "reason", rename_all = "kebab-case")]
pub enum BluetoothPairError {
    Rejected { message: String },
    Unreachable { message: String },
    Unknown { message: String },
}

// BlueZ's own D-Bus error names, surfaced verbatim by bluetoothctl's "Failed to pair: <error>"
// line -- not verified against real hardware from this repo (no Bluetooth adapter in the dev
// environment), same caveat network::classify_connect_failure carries for nmcli.
pub fn classify_pair_failure(output: &str) -> BluetoothPairError {
    if output.contains("AuthenticationFailed")
        || output.contains("AuthenticationRejected")
        || output.contains("AuthenticationCanceled")
    {
        return BluetoothPairError::Rejected { message: "Pairing was rejected".to_string() };
    }
    let lower = output.to_lowercase();
    if lower.contains("org.bluez.error.notready")
        || lower.contains("does not exist")
        || (lower.contains("device") && lower.contains("not available"))
    {
        return BluetoothPairError::Unreachable { message: "Device not found -- try scanning again".to_string() };
    }
    BluetoothPairError::Unknown { message: output.trim().to_string() }
}

async fn run_bluetoothctl(bin: &str, args: &[&str], timeout: Duration) -> Result<std::process::Output, String> {
    let output_fut = Command::new(bin).args(args).stdin(Stdio::null()).output();
    match tokio::time::timeout(timeout, output_fut).await {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(e)) if e.kind() == std::io::ErrorKind::NotFound => {
            Err("bluetoothctl not found -- is bluez installed?".to_string())
        }
        Ok(Err(e)) => Err(e.to_string()),
        Err(_elapsed) => Err("bluetoothctl timed out".to_string()),
    }
}

async fn get_device_state(bin: &str, address: &str) -> Result<(bool, bool, Option<u32>), String> {
    let output = run_bluetoothctl(bin, &["info", address], COMMAND_TIMEOUT).await?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let paired = parse_info_field(&stdout, "Paired") == Some("yes");
    let connected = parse_info_field(&stdout, "Connected") == Some("yes");
    let battery_percent = parse_info_field(&stdout, "Battery Percentage").and_then(parse_battery_percent);

    Ok((paired, connected, battery_percent))
}

// Scans for SCAN_SECONDS and returns whatever showed up as newly discovered -- deliberately not
// cross-checked against already-paired devices ("press sync on your controller, see it appear";
// re-selecting an already-paired device that happens to show up just re-runs pair/trust/connect,
// a harmless no-op/refresh, not worth filtering out).
pub async fn scan_for_devices(bin: &str) -> Result<Vec<BluetoothDevice>, String> {
    let output = run_bluetoothctl(bin, &["--timeout", &SCAN_SECONDS.to_string(), "scan", "on"], SCAN_TIMEOUT).await?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut by_address: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for line in stdout.lines() {
        if let Some((address, name)) = parse_device_line(line) {
            by_address.insert(address, name);
        }
    }

    Ok(by_address
        .into_iter()
        .map(|(address, name)| BluetoothDevice { address, name, paired: false, connected: false, battery_percent: None })
        .collect())
}

pub async fn list_paired_devices(bin: &str) -> Result<Vec<BluetoothDevice>, String> {
    let output = run_bluetoothctl(bin, &["devices", "Paired"], COMMAND_TIMEOUT).await?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let entries: Vec<(String, String)> = stdout.lines().filter_map(parse_device_line).collect();

    // One `info` call per paired device -- fine at the scale this screen actually deals with (a
    // handful of controllers), and it's the only way to get connected/battery state, which
    // `devices Paired` itself doesn't report.
    let mut devices = Vec::with_capacity(entries.len());
    for (address, name) in entries {
        let (paired, connected, battery_percent) = get_device_state(bin, &address).await?;
        devices.push(BluetoothDevice { address, name, paired, connected, battery_percent });
    }
    Ok(devices)
}

pub async fn pair_device(bin: &str, address: &str) -> Result<(), BluetoothPairError> {
    let output = run_bluetoothctl(bin, &["pair", address], PAIR_TIMEOUT)
        .await
        .map_err(|message| BluetoothPairError::Unknown { message })?;

    if !output.status.success() {
        let combined = format!("{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        return Err(classify_pair_failure(&combined));
    }

    // bluetoothctl's one-shot `pair` prints a result line but its own process exit code isn't
    // confirmed reliable for "did pairing actually succeed" (unverified against real hardware) --
    // re-reading Paired: yes/no via `info` right after is a source of truth that doesn't depend
    // on trusting that.
    let (paired, _connected, _battery) =
        get_device_state(bin, address).await.map_err(|message| BluetoothPairError::Unknown { message })?;
    if !paired {
        return Err(BluetoothPairError::Rejected { message: "Pairing was rejected".to_string() });
    }

    // trust (so it auto-reconnects on power-on without re-pairing) then connect -- best-effort: a
    // device that paired but won't immediately connect (e.g. a controller that needs its own sync
    // button pressed again) still counts as a successful pair.
    let _ = run_bluetoothctl(bin, &["trust", address], COMMAND_TIMEOUT).await;
    let _ = run_bluetoothctl(bin, &["connect", address], COMMAND_TIMEOUT).await;

    Ok(())
}

pub async fn remove_device(bin: &str, address: &str) -> Result<(), String> {
    run_bluetoothctl(bin, &["remove", address], COMMAND_TIMEOUT).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_device_line_matches_a_plain_device_line() {
        let parsed = parse_device_line("Device AA:BB:CC:DD:EE:FF Xbox Wireless Controller");
        assert_eq!(parsed, Some(("AA:BB:CC:DD:EE:FF".to_string(), "Xbox Wireless Controller".to_string())));
    }

    #[test]
    fn parse_device_line_matches_a_tagged_scan_event_line() {
        let parsed = parse_device_line("[NEW] Device 11:22:33:44:55:66 DualSense Controller");
        assert_eq!(parsed, Some(("11:22:33:44:55:66".to_string(), "DualSense Controller".to_string())));
    }

    #[test]
    fn parse_device_line_rejects_a_non_matching_line() {
        assert_eq!(parse_device_line("Controller AA:BB:CC:DD:EE:FF [default]"), None);
        assert_eq!(parse_device_line(""), None);
    }

    #[test]
    fn parse_battery_percent_extracts_the_decimal_value() {
        assert_eq!(parse_battery_percent("0x54 (84)"), Some(84));
    }

    #[test]
    fn parse_battery_percent_is_none_without_parens() {
        assert_eq!(parse_battery_percent("unknown"), None);
    }

    #[test]
    fn parse_info_field_finds_a_matching_field_case_sensitively() {
        let stdout = "Name: Controller\nPaired: yes\nTrusted: yes\nConnected: no\n";
        assert_eq!(parse_info_field(stdout, "Paired"), Some("yes"));
        assert_eq!(parse_info_field(stdout, "Connected"), Some("no"));
        assert_eq!(parse_info_field(stdout, "Battery Percentage"), None);
    }

    #[test]
    fn classify_pair_failure_recognizes_authentication_rejection() {
        let err = classify_pair_failure("Failed to pair: org.bluez.Error.AuthenticationRejected");
        assert!(matches!(err, BluetoothPairError::Rejected { .. }));
    }

    #[test]
    fn classify_pair_failure_recognizes_an_unreachable_device() {
        let err = classify_pair_failure("Device AA:BB:CC:DD:EE:FF not available");
        assert!(matches!(err, BluetoothPairError::Unreachable { .. }));
    }

    #[test]
    fn classify_pair_failure_falls_back_to_unknown() {
        let err = classify_pair_failure("some other bluetoothctl error\n");
        assert_eq!(err, BluetoothPairError::Unknown { message: "some other bluetoothctl error".to_string() });
    }

    #[tokio::test]
    async fn run_bluetoothctl_captures_stdout_on_success() {
        let output = run_bluetoothctl("sh", &["-c", "echo hello"], Duration::from_secs(5)).await.unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
    }

    #[tokio::test]
    async fn run_bluetoothctl_reports_a_clear_error_for_a_missing_binary() {
        let err = run_bluetoothctl("definitely-not-a-real-binary-xyz", &[], Duration::from_secs(5)).await.unwrap_err();
        assert!(err.contains("not found"));
    }

    #[tokio::test]
    async fn run_bluetoothctl_times_out_rather_than_hanging_forever() {
        let err = run_bluetoothctl("sh", &["-c", "sleep 5"], Duration::from_millis(50)).await.unwrap_err();
        assert!(err.contains("timed out"));
    }
}
