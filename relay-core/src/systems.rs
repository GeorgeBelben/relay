pub struct System {
    /// Slug used in directory names and the library protocol (e.g. "snes").
    pub id: &'static str,
    /// Human-readable name for UI display.
    pub display_name: &'static str,
    /// File extensions recognized as ROMs for this system.
    pub extensions: &'static [&'static str],
    /// RetroArch libretro core to use, if this system is RetroArch-backed.
    /// None for systems handled by a standalone emulator instead.
    pub retroarch_core: Option<&'static str>,
}

// NOTE: retroarch_core values below are placeholders based on common
// libretro core names — verify against what's actually installed on
// the EliteDesk before relying on them; I haven't confirmed these
// against your box.
pub const ALL_SYSTEMS: &[System] = &[
    System {
        id: "nes",
        display_name: "NES",
        extensions: &["nes"],
        retroarch_core: Some("nestopia"),
    },
    System {
        id: "snes",
        display_name: "SNES",
        extensions: &["sfc", "smc"],
        retroarch_core: Some("snes9x"),
    },
    System {
        id: "genesis",
        display_name: "Genesis",
        extensions: &["md", "bin", "gen"],
        retroarch_core: Some("genesis_plus_gx"),
    },
    System {
        id: "n64",
        display_name: "N64",
        extensions: &["n64", "z64"],
        retroarch_core: Some("mupen64plus_next"),
    },
    System {
        id: "gb",
        display_name: "Game Boy",
        extensions: &["gb", "gbc"],
        retroarch_core: Some("gambatte"),
    },
    System {
        id: "gba",
        display_name: "Game Boy Advance",
        extensions: &["gba"],
        retroarch_core: Some("mgba"),
    },
    System {
        id: "psx",
        display_name: "PlayStation",
        extensions: &["bin", "cue", "chd"],
        retroarch_core: Some("pcsx_rearmed"),
    },
    System {
        id: "ps2",
        display_name: "PlayStation 2",
        extensions: &["iso", "chd"],
        retroarch_core: None,
    },
    System {
        id: "gamecube",
        display_name: "GameCube",
        extensions: &["iso", "rvz"],
        retroarch_core: None,
    },
    System {
        id: "wii",
        display_name: "Wii",
        extensions: &["iso", "rvz", "wbfs"],
        retroarch_core: None,
    },
    System {
        id: "psp",
        display_name: "PSP",
        extensions: &["iso", "cso"],
        retroarch_core: None,
    },
];

pub fn find(id: &str) -> Option<&'static System> {
    ALL_SYSTEMS.iter().find(|s| s.id == id)
}
