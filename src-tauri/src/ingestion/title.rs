use std::sync::LazyLock;

use regex::Regex;

// Strips No-Intro-style parenthetical/bracketed tags (region, language, revision, disc number)
// from a filename/folder stem -- e.g. "Super Mario World (USA) (Rev 1)" -> "Super Mario World".
// Ported from the Electron MVP's scanner/title.ts; a placeholder until real metadata matching
// (identify stage) lands, so expect rough edges.
static TAG_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*[(\[][^)\]]*[)\]]\s*").unwrap());

pub fn title_from_filename(stem: &str) -> String {
    let without_tags = TAG_PATTERN.replace_all(stem, " ");
    let without_underscores = without_tags.replace('_', " ");
    without_underscores.split_whitespace().collect::<Vec<_>>().join(" ")
}
