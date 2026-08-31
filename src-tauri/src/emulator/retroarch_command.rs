//! Sends a command over RetroArch's UDP network-command interface (REL-23) -- the same interface
//! `retroarch_config.rs`'s appendconfig forces on per-launch (`network_cmd_enable`). Ported from
//! the Electron MVP's `retroarchCommand.ts` (`sendRetroArchCommand`): plain fire-and-forget UDP,
//! no ack, no response to wait for. Only meaningful while a RetroArch-core game is actually
//! running -- sending to a port nothing is listening on (no game running, or the running system
//! uses a standalone binary instead of RetroArch) is a silent no-op, not an error, so callers
//! don't need to track "is this actually a RetroArch session" before calling.
//!
//! Command strings match RetroArch's own `command.c`/`command_event.h` (e.g. "PAUSE_TOGGLE",
//! "SAVE_STATE") -- see <https://docs.libretro.com/guides/network-control-interface/>.

use std::io;
use tokio::net::UdpSocket;

use super::retroarch_config::NETWORK_CMD_PORT;

pub async fn send_command(command: &str) -> io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    socket.send_to(command.as_bytes(), ("127.0.0.1", NETWORK_CMD_PORT)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn send_command_delivers_the_exact_bytes_to_a_local_listener() {
        // Binds the real network_cmd port rather than a stand-in -- proves send_command actually
        // targets RetroArch's documented port, not just "some UDP socket".
        let listener = UdpSocket::bind(("127.0.0.1", NETWORK_CMD_PORT)).await.unwrap();

        send_command("PAUSE_TOGGLE").await.unwrap();

        let mut buf = [0u8; 64];
        let (len, _addr) = listener.recv_from(&mut buf).await.unwrap();
        assert_eq!(&buf[..len], b"PAUSE_TOGGLE");
    }
}
