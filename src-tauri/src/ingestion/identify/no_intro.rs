//! CRC32-exact matching against libretro-database's No-Intro DAT mirrors -- an exact hash match
//! beats guessing a title from a filename (catalog numbers, GoodTools/TOSEC tags, plain garbage).
//! Ported from the Electron MVP's `lib/identify/{parseNoIntroDat,noIntroDats}.ts` (that project's
//! REL-38).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use regex::Regex;

const DEFAULT_BASE_URL: &str = "https://raw.githubusercontent.com/libretro/libretro-database/master/metadat/no-intro/";

// Splitting on the literal "game (" line (rather than trying to balance parens, since the nested
// `rom(...)` has its own closing paren before the block's own) means each resulting chunk
// contains exactly one game's fields and nothing from its neighbors -- no need to track nesting
// at all, just pull the first `name "..."` and the `crc` token out of each chunk.
static GAME_BLOCK_DELIMITER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^game \(\s*$").unwrap());
static NAME_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?m)^\s*name\s+"([^"]*)""#).unwrap());
static CRC_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bcrc\s+([0-9A-Fa-f]{8})\b").unwrap());

/// crc32 (uppercase hex, matching `probe::probe_file`'s own format) -> canonical DAT name, still
/// carrying its region/language tags (e.g. "(USA)") -- callers run that back through
/// `title_from_filename` to strip them, same as a raw filename.
pub fn parse_no_intro_dat(text: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();

    // The first chunk is the file's own `clrmamepro ( ... )` header, not a game block.
    let blocks: Vec<&str> = GAME_BLOCK_DELIMITER.split(text).collect();
    for block in blocks.into_iter().skip(1) {
        let name = NAME_PATTERN.captures(block).map(|c| c[1].to_string());
        let crc = CRC_PATTERN.captures(block).map(|c| c[1].to_uppercase());
        if let (Some(name), Some(crc)) = (name, crc) {
            result.insert(crc, name);
        }
    }

    result
}

// One or more libretro-database DAT filenames per system -- ngpc/wonderswan need two each (mono
// and Color are separate No-Intro DAT sets upstream, merged into one lookup here since Relay's
// own system list doesn't split them). Scoped to exactly the systems whose whole-file CRC32
// (ingestion::probe) matches a DAT rom hash directly -- disc-based systems' CRC32 is either the
// .cue/.gdi wrapper's own hash or a possibly-compressed container's, neither of which matches
// Redump's raw-track hash, so they're deliberately excluded (same reasoning as the MVP's own
// RetroAchievements hash.ts).
fn dat_files_for(system_id: &str) -> Option<&'static [&'static str]> {
    match system_id {
        "nes" => Some(&["Nintendo - Nintendo Entertainment System.dat"]),
        "snes" => Some(&["Nintendo - Super Nintendo Entertainment System.dat"]),
        "n64" => Some(&["Nintendo - Nintendo 64.dat"]),
        "gb" => Some(&["Nintendo - Game Boy.dat"]),
        "gbc" => Some(&["Nintendo - Game Boy Color.dat"]),
        "gba" => Some(&["Nintendo - Game Boy Advance.dat"]),
        "nds" => Some(&["Nintendo - Nintendo DS.dat"]),
        "mastersystem" => Some(&["Sega - Master System - Mark III.dat"]),
        "gamegear" => Some(&["Sega - Game Gear.dat"]),
        "megadrive" => Some(&["Sega - Mega Drive - Genesis.dat"]),
        "ngpc" => Some(&["SNK - Neo Geo Pocket.dat", "SNK - Neo Geo Pocket Color.dat"]),
        "wonderswan" => Some(&["Bandai - WonderSwan.dat", "Bandai - WonderSwan Color.dat"]),
        _ => None,
    }
}

/// Fetches, parses, and caches (in memory for the process lifetime, and on disk at
/// `<cache_dir>/<system_id>.json`) No-Intro DAT files, keyed by system. DAT files are a few MB
/// and change rarely, so a lookup only ever hits the network once per system per cache directory.
pub struct NoIntroDatLookup {
    http: reqwest::Client,
    base_url: reqwest::Url,
    cache_dir: PathBuf,
    memory: Mutex<HashMap<String, HashMap<String, String>>>,
}

impl NoIntroDatLookup {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self::with_base_url(cache_dir, DEFAULT_BASE_URL)
    }

    /// Exposed so tests can point the client at a local mock server instead of the real API.
    pub fn with_base_url(cache_dir: PathBuf, base_url: &str) -> Self {
        let mut base_url = reqwest::Url::parse(base_url).expect("invalid No-Intro DAT base URL");
        // A trailing slash leaves an empty final path segment; `fetch_and_parse`'s
        // `path_segments_mut().push()` would then append *after* it, producing "..//filename.dat"
        // -- which GitHub's raw file server 404s on. Popping it here makes `push()` correct
        // regardless of whether the caller's base URL string happened to end with "/".
        base_url.path_segments_mut().unwrap().pop_if_empty();

        Self {
            http: reqwest::Client::new(),
            base_url,
            cache_dir,
            memory: Mutex::new(HashMap::new()),
        }
    }

    /// `None` means "couldn't identify" (unsupported system, DAT unavailable, or this exact dump
    /// just isn't in No-Intro's set) -- callers fall back to the filename-derived title either
    /// way, same as if this didn't exist.
    pub async fn lookup(&self, system_id: &str, crc32: &str) -> Option<String> {
        let map = self.dat_map(system_id).await?;
        map.get(&crc32.to_uppercase()).cloned()
    }

    async fn dat_map(&self, system_id: &str) -> Option<HashMap<String, String>> {
        let dat_files = dat_files_for(system_id)?;

        if let Some(cached) = self.memory.lock().unwrap().get(system_id) {
            return Some(cached.clone());
        }

        if let Some(from_disk) = self.load_from_disk(system_id).await {
            self.memory.lock().unwrap().insert(system_id.to_string(), from_disk.clone());
            return Some(from_disk);
        }

        let fetched = self.fetch_and_parse(dat_files).await;
        // A failed fetch/parse returns empty -- don't cache that to disk, so the next lookup gets
        // to retry instead of being stuck with a permanently-empty cache from a network hiccup.
        if fetched.is_empty() {
            return None;
        }

        self.memory.lock().unwrap().insert(system_id.to_string(), fetched.clone());
        if let Err(e) = self.save_to_disk(system_id, &fetched).await {
            eprintln!("identify: failed to cache DAT for \"{system_id}\": {e}");
        }
        Some(fetched)
    }

    fn cache_path(&self, system_id: &str) -> PathBuf {
        self.cache_dir.join(format!("{system_id}.json"))
    }

    async fn load_from_disk(&self, system_id: &str) -> Option<HashMap<String, String>> {
        let raw = tokio::fs::read_to_string(self.cache_path(system_id)).await.ok()?;
        serde_json::from_str(&raw).ok()
    }

    async fn save_to_disk(&self, system_id: &str, map: &HashMap<String, String>) -> std::io::Result<()> {
        tokio::fs::create_dir_all(&self.cache_dir).await?;
        let json = serde_json::to_string(map).unwrap_or_default();
        tokio::fs::write(self.cache_path(system_id), json).await
    }

    async fn fetch_and_parse(&self, filenames: &[&str]) -> HashMap<String, String> {
        let mut merged = HashMap::new();
        for filename in filenames {
            let mut url = self.base_url.clone();
            url.path_segments_mut().unwrap().push(filename);

            match self.http.get(url).send().await {
                Ok(res) if res.status().is_success() => match res.text().await {
                    Ok(text) => merged.extend(parse_no_intro_dat(&text)),
                    Err(e) => eprintln!("identify: failed to read DAT \"{filename}\": {e}"),
                },
                Ok(res) => eprintln!("identify: failed to fetch DAT \"{filename}\" ({})", res.status()),
                Err(e) => eprintln!("identify: failed to fetch DAT \"{filename}\": {e}"),
            }
        }
        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Verbatim excerpt (trimmed) from the real "Nintendo - Game Boy Advance.dat" fetched from
    // github.com/libretro/libretro-database, not a hand-constructed fixture -- proves the parser
    // against the actual shape, not an assumption of it.
    const REAL_EXCERPT: &str = r#"clrmamepro (
	name "Nintendo - Game Boy Advance"
	description "Nintendo - Game Boy Advance"
	version "2026.08.01"
	homepage "http://github.com/robloach/libretro-dats"
)

game (
	name "007 - Everything or Nothing (Japan)"
	region "Japan"
	serial "BJBJ"
	rom ( name "007 - Everything or Nothing (Japan).gba" size 8388608 crc CAF2E99F md5 55354D9E3BC9C1FA682B5110E5ED1544 sha1 6E4E9BE9A07580EF267BE9C2EA1BD0730B3BE44A serial "BJBJ" )
)
game (
	name "007 - Everything or Nothing (USA, Europe) (En,Fr,De)"
	region "USA"
	serial "BJBE"
	rom ( name "007 - Everything or Nothing (USA, Europe) (En,Fr,De).gba" size 8388608 crc 9D4F1E18 md5 B63B2244EDC2385AE1EAB9C8EE448C6F sha1 FC6163F99B71B05C10686A0D29010B31274E1DC4 serial "BJBE" )
)
"#;

    #[test]
    fn indexes_each_roms_canonical_name_by_its_crc32_uppercased() {
        let result = parse_no_intro_dat(REAL_EXCERPT);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("CAF2E99F").map(String::as_str), Some("007 - Everything or Nothing (Japan)"));
        assert_eq!(
            result.get("9D4F1E18").map(String::as_str),
            Some("007 - Everything or Nothing (USA, Europe) (En,Fr,De)")
        );
    }

    #[test]
    fn normalizes_a_lowercase_crc_from_the_source_file_to_uppercase() {
        let lowered = REAL_EXCERPT.replace("9D4F1E18", "9d4f1e18");
        let result = parse_no_intro_dat(&lowered);
        assert_eq!(
            result.get("9D4F1E18").map(String::as_str),
            Some("007 - Everything or Nothing (USA, Europe) (En,Fr,De)")
        );
    }

    #[test]
    fn doesnt_mistake_the_files_own_header_block_for_a_game() {
        let result = parse_no_intro_dat(REAL_EXCERPT);
        assert!(!result.values().any(|v| v == "Nintendo - Game Boy Advance"));
    }

    #[test]
    fn skips_a_game_block_with_no_rom_crc_rather_than_panicking() {
        let malformed = "game (\n\tname \"No ROM Line\"\n)\n";
        assert_eq!(parse_no_intro_dat(malformed).len(), 0);
    }

    #[test]
    fn returns_an_empty_map_for_empty_or_unrecognized_input() {
        assert_eq!(parse_no_intro_dat("").len(), 0);
        assert_eq!(parse_no_intro_dat("not a dat file at all").len(), 0);
    }

    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const SAMPLE_DAT: &str = "clrmamepro (\n\tname \"Nintendo - Game Boy Advance\"\n)\n\ngame (\n\tname \"Super Mario Advance 4 - Super Mario Bros. 3 (USA) (En,Fr,De,Es,It)\"\n\trom ( name \"Super Mario Advance 4.gba\" size 8388608 crc 1A2B3C4D md5 X sha1 Y )\n)\n";

    #[tokio::test]
    async fn lookup_fetches_parses_and_returns_the_canonical_name_for_a_matching_crc32() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/Nintendo%20-%20Game%20Boy%20Advance.dat"))
            .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_DAT))
            .mount(&server)
            .await;

        let cache_dir = tempfile::tempdir().unwrap();
        let lookup = NoIntroDatLookup::with_base_url(cache_dir.path().to_path_buf(), &format!("{}/", server.uri()));

        let title = lookup.lookup("gba", "1A2B3C4D").await;
        assert_eq!(title.as_deref(), Some("Super Mario Advance 4 - Super Mario Bros. 3 (USA) (En,Fr,De,Es,It)"));
    }

    #[tokio::test]
    async fn lookup_is_case_insensitive_on_the_crc32_argument() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_DAT))
            .mount(&server)
            .await;

        let cache_dir = tempfile::tempdir().unwrap();
        let lookup = NoIntroDatLookup::with_base_url(cache_dir.path().to_path_buf(), &format!("{}/", server.uri()));

        assert!(lookup.lookup("gba", "1a2b3c4d").await.is_some());
    }

    #[tokio::test]
    async fn lookup_returns_none_for_a_crc32_not_present_in_the_dat() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_DAT))
            .mount(&server)
            .await;

        let cache_dir = tempfile::tempdir().unwrap();
        let lookup = NoIntroDatLookup::with_base_url(cache_dir.path().to_path_buf(), &format!("{}/", server.uri()));

        assert_eq!(lookup.lookup("gba", "DEADBEEF").await, None);
    }

    #[tokio::test]
    async fn lookup_returns_none_for_a_system_with_no_dat_mapping() {
        let cache_dir = tempfile::tempdir().unwrap();
        // Base URL deliberately unreachable -- a disc-based system should never even attempt a
        // fetch, so this proves that rather than merely tolerating a failed one.
        let lookup = NoIntroDatLookup::with_base_url(cache_dir.path().to_path_buf(), "http://127.0.0.1:1/");

        assert_eq!(lookup.lookup("psx", "1A2B3C4D").await, None);
    }

    #[tokio::test]
    async fn lookup_caches_the_parsed_dat_to_disk_and_reuses_it_across_instances() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_DAT))
            .expect(1) // exactly once: the second lookup below must come from the disk cache
            .mount(&server)
            .await;

        let cache_dir = tempfile::tempdir().unwrap();
        let base_url = format!("{}/", server.uri());

        let first = NoIntroDatLookup::with_base_url(cache_dir.path().to_path_buf(), &base_url);
        assert!(first.lookup("gba", "1A2B3C4D").await.is_some());

        let cache_file = cache_dir.path().join("gba.json");
        assert!(cache_file.exists());

        // A fresh instance (simulating a new process) reads the disk cache instead of fetching.
        let second = NoIntroDatLookup::with_base_url(cache_dir.path().to_path_buf(), &base_url);
        assert_eq!(
            second.lookup("gba", "1A2B3C4D").await.as_deref(),
            Some("Super Mario Advance 4 - Super Mario Bros. 3 (USA) (En,Fr,De,Es,It)")
        );
    }

    #[tokio::test]
    async fn lookup_does_not_cache_a_failed_fetch() {
        let server = MockServer::start().await;
        Mock::given(method("GET")).respond_with(ResponseTemplate::new(500)).mount(&server).await;

        let cache_dir = tempfile::tempdir().unwrap();
        let lookup = NoIntroDatLookup::with_base_url(cache_dir.path().to_path_buf(), &format!("{}/", server.uri()));

        assert_eq!(lookup.lookup("gba", "1A2B3C4D").await, None);
        assert!(!cache_dir.path().join("gba.json").exists());
    }
}
