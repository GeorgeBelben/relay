//! The fixed, complete set of emulated systems Relay supports (REL-134) -- ported from the
//! Electron MVP's `db/seed/systems.ts`. This is source code, not seeded database rows: the set of
//! systems a console supports is a product decision, not user-editable data, so there's no
//! `create`/`update`/`delete` for it -- only `get`/`all` lookups. `roms.system_id` stores one of
//! these `id`s as a plain string with no DB-level foreign key.
//!
//! Extensions are matched case-insensitively without a leading dot (see
//! `ingestion::scan::walk_system_folder`). CD-based systems (psx, saturn, dreamcast) deliberately
//! omit "bin" -- it's a data track referenced BY the .cue/.gdi wrapper, not launchable on its own,
//! so listing it would double-index every single-disc CD game.
//!
//! `retroarch_core`/`standalone_binary`: exactly one is set per system (see
//! `emulator::command::SystemLaunchConfig`). These are sensible defaults, not verified against
//! real hardware -- swap a system's core here if a different one turns out to fit better once
//! this runs on the actual device.

pub struct SystemDef {
    pub id: &'static str,
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    pub retroarch_core: Option<&'static str>,
    pub standalone_binary: Option<&'static str>,
}

pub const ALL: &[SystemDef] = &[
    SystemDef { id: "nes", name: "NES", extensions: &["nes"], retroarch_core: Some("nestopia"), standalone_binary: None },
    SystemDef {
        id: "snes",
        name: "SNES",
        extensions: &["sfc", "smc"],
        retroarch_core: Some("snes9x"),
        standalone_binary: None,
    },
    SystemDef {
        id: "n64",
        name: "Nintendo 64",
        extensions: &["n64", "z64", "v64"],
        retroarch_core: Some("mupen64plus_next"),
        standalone_binary: None,
    },
    SystemDef {
        id: "gb",
        name: "Game Boy",
        extensions: &["gb"],
        retroarch_core: Some("gambatte"),
        standalone_binary: None,
    },
    SystemDef {
        id: "gbc",
        name: "Game Boy Color",
        extensions: &["gbc"],
        retroarch_core: Some("gambatte"),
        standalone_binary: None,
    },
    SystemDef {
        id: "gba",
        name: "Game Boy Advance",
        extensions: &["gba"],
        retroarch_core: Some("mgba"),
        standalone_binary: None,
    },
    SystemDef {
        id: "nds",
        name: "Nintendo DS",
        extensions: &["nds"],
        // melonDS isn't packaged in Ubuntu's default repos (checked against the real device,
        // REL-136) -- desmume is, and is the standard fallback libretro NDS core.
        retroarch_core: Some("desmume"),
        standalone_binary: None,
    },
    SystemDef {
        id: "gamecube",
        name: "GameCube",
        extensions: &["iso", "gcm", "rvz"],
        retroarch_core: None,
        standalone_binary: Some("dolphin-emu"),
    },
    SystemDef {
        id: "wii",
        name: "Wii",
        extensions: &["iso", "rvz", "wbfs"],
        retroarch_core: None,
        standalone_binary: Some("dolphin-emu"),
    },
    SystemDef {
        id: "psx",
        name: "PlayStation",
        extensions: &["cue", "chd", "pbp"],
        // pcsx_rearmed isn't packaged in Ubuntu's default repos (checked against the real
        // device, REL-136) -- mednafen_psx ("Beetle PSX", package libretro-beetle-psx) is.
        retroarch_core: Some("mednafen_psx"),
        standalone_binary: None,
    },
    SystemDef {
        id: "ps2",
        name: "PlayStation 2",
        extensions: &["iso", "chd"],
        retroarch_core: None,
        standalone_binary: Some("pcsx2"),
    },
    SystemDef {
        id: "psp",
        name: "PSP",
        extensions: &["iso", "cso"],
        retroarch_core: Some("ppsspp"),
        standalone_binary: None,
    },
    SystemDef {
        id: "mastersystem",
        name: "Sega Master System",
        extensions: &["sms"],
        retroarch_core: Some("genesis_plus_gx"),
        standalone_binary: None,
    },
    SystemDef {
        id: "gamegear",
        name: "Sega Game Gear",
        extensions: &["gg"],
        retroarch_core: Some("genesis_plus_gx"),
        standalone_binary: None,
    },
    SystemDef {
        id: "megadrive",
        name: "Sega Genesis / Mega Drive",
        extensions: &["md", "bin", "gen"],
        retroarch_core: Some("genesis_plus_gx"),
        standalone_binary: None,
    },
    SystemDef {
        id: "saturn",
        name: "Sega Saturn",
        extensions: &["cue", "chd"],
        // mednafen_saturn has no packaged libretro core in Ubuntu's default repos (checked
        // against the real device, REL-136) -- yabause-qt is the closest packaged standalone.
        retroarch_core: None,
        standalone_binary: Some("yabause-qt"),
    },
    SystemDef {
        id: "dreamcast",
        name: "Dreamcast",
        extensions: &["cdi", "chd", "gdi"],
        retroarch_core: Some("flycast"),
        standalone_binary: None,
    },
    // Original Xbox emulation isn't covered by any mature libretro core -- xemu is the standalone
    // project for it, though notably less battle-tested than Dolphin/PCSX2 for the other
    // standalone systems here.
    SystemDef {
        id: "xbox",
        name: "Xbox",
        extensions: &["iso", "xiso"],
        retroarch_core: None,
        standalone_binary: Some("xemu"),
    },
    // mame2003_plus over the full "mame" core -- lighter weight, and the more common choice for
    // the older ROM sets a MAME folder is likely to actually contain.
    SystemDef {
        id: "mame",
        name: "Arcade (MAME)",
        extensions: &["zip", "chd"],
        retroarch_core: Some("mame2003_plus"),
        standalone_binary: None,
    },
    SystemDef {
        id: "ngpc",
        name: "Neo Geo Pocket / Color",
        extensions: &["ngp", "ngc"],
        retroarch_core: Some("mednafen_ngp"),
        standalone_binary: None,
    },
    SystemDef {
        id: "wonderswan",
        name: "WonderSwan / Color",
        extensions: &["ws", "wsc"],
        retroarch_core: Some("mednafen_wswan"),
        standalone_binary: None,
    },
];

pub fn get(id: &str) -> Option<&'static SystemDef> {
    ALL.iter().find(|system| system.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_system_has_exactly_one_launch_method() {
        for system in ALL {
            assert!(
                system.retroarch_core.is_some() ^ system.standalone_binary.is_some(),
                "{} must set exactly one of retroarch_core/standalone_binary",
                system.id
            );
        }
    }

    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<&str> = ALL.iter().map(|s| s.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ALL.len());
    }

    #[test]
    fn get_finds_a_known_system_and_rejects_an_unknown_one() {
        assert_eq!(get("snes").unwrap().name, "SNES");
        assert!(get("dreamcast").is_some());
        assert!(get("not-a-real-system").is_none());
    }
}
