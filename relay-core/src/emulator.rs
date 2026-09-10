use std::process::Child;

/// Common interface every emulator integration implements.
///
/// Different backends (RetroArch cores, standalone emulators like PCSX2)
/// all launch differently under the hood, but the daemon only needs to
/// know "give me a rom path, get back a running process" — it shouldn't
/// need to care which backend it's talking to.
pub trait EmulatorBackend {
    /// Launch the given ROM and return the spawned OS process.
    ///
    /// The daemon takes ownership of the returned `Child` — tracking it
    /// and noticing when it exits is the daemon's job, not the backend's.
    fn launch(&self, rom_path: &str) -> Result<Child, String>;
}
