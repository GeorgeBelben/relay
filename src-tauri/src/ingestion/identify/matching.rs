// Standard Jaro-Winkler string similarity, 0 (no match) to 1 (identical) -- ported from the
// Electron MVP's metadata/jaroWinkler.ts, which itself notes this is the same approach Cove uses
// for its TMDB matcher. No crate for the algorithm itself; only NFD normalization is delegated.
use unicode_normalization::UnicodeNormalization;

fn jaro(a: &[char], b: &[char]) -> f64 {
    if a == b {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let match_distance = (a.len().max(b.len()) / 2).saturating_sub(1);
    let mut a_matches = vec![false; a.len()];
    let mut b_matches = vec![false; b.len()];

    let mut matches = 0usize;
    for i in 0..a.len() {
        let start = i.saturating_sub(match_distance);
        let end = (i + match_distance + 1).min(b.len());
        for j in start..end {
            if b_matches[j] || a[i] != b[j] {
                continue;
            }
            a_matches[i] = true;
            b_matches[j] = true;
            matches += 1;
            break;
        }
    }
    if matches == 0 {
        return 0.0;
    }

    let mut transpositions = 0usize;
    let mut k = 0;
    for i in 0..a.len() {
        if !a_matches[i] {
            continue;
        }
        while !b_matches[k] {
            k += 1;
        }
        if a[i] != b[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let matches_f = matches as f64;
    (matches_f / a.len() as f64 + matches_f / b.len() as f64 + (matches_f - transpositions as f64 / 2.0) / matches_f) / 3.0
}

// Boosts the Jaro score for strings sharing a common prefix (up to 4 chars), which is what turns
// "jaro" into "jaro-winkler" -- titles that start the same way are much likelier to be the same
// game than the raw Jaro score alone would suggest.
pub fn jaro_winkler(a: &str, b: &str) -> f64 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let score = jaro(&a, &b);

    let max_prefix = 4.min(a.len()).min(b.len());
    let mut prefix_length = 0;
    while prefix_length < max_prefix && a[prefix_length] == b[prefix_length] {
        prefix_length += 1;
    }

    score + prefix_length as f64 * 0.1 * (1.0 - score)
}

const COMBINING_MARKS_START: u32 = 0x0300;
const COMBINING_MARKS_END: u32 = 0x036F;

// Lowercase, strip everything but letters/digits/spaces, collapse whitespace -- so "Mario Kart:
// 64!" and "mario kart 64" compare as equal inputs rather than being penalized for punctuation
// neither title's source actually disagrees about.
//
// The NFD-decompose-then-strip-diacritics step runs first and matters on its own: without it, an
// accented character (e.g. the é in "Pokémon") falls through the alphanumeric filter below and
// gets replaced with a bare space, splitting the word in two ("pok mon") instead of folding it to
// its unaccented letter ("pokemon") -- which then also throws off the prefix bonus above, since
// "pok" is a much weaker shared prefix than the full "pokemon" every candidate actually shares.
pub fn normalize_for_match(title: &str) -> String {
    let decomposed: String = title
        .nfd()
        .filter(|c| !(COMBINING_MARKS_START..=COMBINING_MARKS_END).contains(&(*c as u32)))
        .collect();

    let lowered = decomposed.to_lowercase();
    let filtered: String = lowered
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect();

    filtered.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub struct MatchResult<'a, T> {
    pub candidate: &'a T,
    pub score: f64,
}

// `early_return_threshold`: once a candidate's score clears this, it's returned immediately
// rather than continuing to scan for something that scores even higher. Matters because
// candidates normally arrive in the source API's own relevance order, and blindly maximizing
// Jaro-Winkler across all of them can rank a same-franchise sequel/remake above the actual best
// match -- a title that's a superset of the query (extra words inserted mid-string) can still
// score deceptively high in practice: for the query "Pokemon Ruby", SteamGridDB's own search
// correctly ranks "Pokémon Ruby Version" first, but naive score-maximization across all results
// would pick "Pokémon Omega Ruby" (a much later remake) instead, since it happens to score higher
// under pure character similarity. Pass `f64::INFINITY` for "always find the true best across
// every candidate," when the caller has no meaningfully-ordered list to respect.
pub fn best_match<'a, T>(
    query: &str,
    candidates: &'a [T],
    get_name: impl Fn(&T) -> &str,
    early_return_threshold: f64,
) -> Option<MatchResult<'a, T>> {
    let normalized_query = normalize_for_match(query);
    let mut best: Option<MatchResult<'a, T>> = None;

    for candidate in candidates {
        let score = jaro_winkler(&normalized_query, &normalize_for_match(get_name(candidate)));
        if best.as_ref().is_none_or(|b| score > b.score) {
            best = Some(MatchResult { candidate, score });
        }
        if score >= early_return_threshold {
            return Some(MatchResult { candidate, score });
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jaro_winkler_scores_identical_strings_as_one() {
        assert_eq!(jaro_winkler("mario kart 64", "mario kart 64"), 1.0);
    }

    #[test]
    fn jaro_winkler_scores_completely_different_strings_low() {
        assert!(jaro_winkler("mario kart 64", "final fantasy vii") < 0.6);
    }

    #[test]
    fn jaro_winkler_scores_a_shared_prefix_higher_than_raw_jaro() {
        assert!(jaro_winkler("mario kart 64", "mario kart") > 0.9);
    }

    #[test]
    fn jaro_winkler_handles_empty_strings_without_panicking() {
        assert_eq!(jaro_winkler("", "mario"), 0.0);
        assert_eq!(jaro_winkler("", ""), 1.0);
    }

    #[test]
    fn normalize_strips_punctuation_and_normalizes_case_whitespace() {
        assert_eq!(normalize_for_match("Mario Kart: 64!"), "mario kart 64");
        assert_eq!(normalize_for_match("  Zelda -- Ocarina of Time  "), "zelda ocarina of time");
    }

    // Found via a real mismatch in the Electron MVP: "Pokémon" fell through the alphanumeric
    // filter as two separate "words" (é became a bare space) without NFD decomposition first.
    #[test]
    fn normalize_folds_accented_characters_instead_of_splitting_on_them() {
        assert_eq!(normalize_for_match("Pokémon Ruby Version"), "pokemon ruby version");
        assert_eq!(normalize_for_match("café"), "cafe");
    }

    struct Candidate {
        name: &'static str,
    }

    #[test]
    fn best_match_picks_the_highest_scoring_candidate_when_no_threshold_given() {
        let candidates = [
            Candidate { name: "Final Fantasy VII" },
            Candidate { name: "Super Mario Kart" },
            Candidate { name: "Mario Kart 64" },
        ];
        let result = best_match("Mario Kart 64", &candidates, |c| c.name, f64::INFINITY).unwrap();
        assert_eq!(result.candidate.name, "Mario Kart 64");
        assert!((result.score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn best_match_returns_none_for_an_empty_candidate_list() {
        let candidates: [Candidate; 0] = [];
        assert!(best_match("Mario Kart 64", &candidates, |c| c.name, f64::INFINITY).is_none());
    }

    // Regression test for the real "Pokemon Ruby" -> "Pokémon Omega Ruby" mismatch: SteamGridDB's
    // own search already ranks the correct "Pokémon Ruby Version" first, but pure score
    // maximization across every candidate picked "Pokémon Omega Ruby" instead.
    #[test]
    fn best_match_prefers_an_earlier_candidate_that_clears_the_threshold() {
        let candidates = [
            Candidate { name: "Pokémon Ruby Version" },
            Candidate { name: "Pokémon Ruby Cross" },
            Candidate { name: "Pokémon Omega Ruby" },
        ];
        let result = best_match("Pokemon Ruby", &candidates, |c| c.name, 0.82).unwrap();
        assert_eq!(result.candidate.name, "Pokémon Ruby Version");
    }

    #[test]
    fn best_match_falls_back_to_best_scoring_candidate_when_none_clear_threshold() {
        let candidates = [
            Candidate { name: "Pokémon Ruby Version" },
            Candidate { name: "Pokémon Omega Ruby" },
        ];
        let result = best_match("Pokemon Ruby", &candidates, |c| c.name, 0.99).unwrap();
        assert_eq!(result.candidate.name, "Pokémon Omega Ruby");
        assert!(result.score < 0.99);
    }
}
