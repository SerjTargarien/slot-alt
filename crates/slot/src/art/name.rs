//! Slot²: a cart's label found by its file name when its checksum isn't in the index
//! (2026-11-08). Translations and hacks are patched games, so their checksums are new,
//! but most keep the original's name: "Cave Noire (Japan) (En) (Aeon Genesis)" is
//! "Cave Noire (Japan)" in the index. The art server names its entries with No-Intro's
//! names and the platform they're for.

use slot_store::Platform;

/// A title reduced to what two spellings of it share: the `(...)` and `[...]` tags
/// dropped, lowercase letters and digits joined by `-`, accents folded, `&` read as
/// "and", and "the" left out, since No-Intro moves it to the end ("Legend, The").
pub fn key(name: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    let mut word = String::new();
    let mut depth = 0u32;
    for c in name.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            _ if depth > 0 => {}
            '\'' | '\u{2019}' => {}
            '&' => {
                end(&mut words, &mut word);
                words.push("and".into());
            }
            _ => match fold(c) {
                Some(f) => word.push_str(f),
                None if c.is_ascii_alphanumeric() => word.push(c.to_ascii_lowercase()),
                None => end(&mut words, &mut word),
            },
        }
    }
    end(&mut words, &mut word);
    words.retain(|w| w != "the");
    words.join("-")
}

fn end(words: &mut Vec<String>, word: &mut String) {
    if !word.is_empty() {
        words.push(std::mem::take(word));
    }
}

fn fold(c: char) -> Option<&'static str> {
    Some(match c.to_lowercase().next()? {
        'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' => "a",
        'é' | 'è' | 'ê' | 'ë' => "e",
        'í' | 'ì' | 'î' | 'ï' => "i",
        'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ø' => "o",
        'ú' | 'ù' | 'û' | 'ü' => "u",
        'ñ' => "n",
        'ç' => "c",
        'ß' => "ss",
        _ => return None,
    })
}

/// The words inside a name's `(...)` and `[...]` tags, lowercase: "(USA, Europe)" gives
/// "usa" and "europe".
pub fn tags(name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut inside = String::new();
    let mut depth = 0u32;
    for c in name.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' if depth > 0 => {
                depth -= 1;
                out.extend(
                    inside
                        .split(|c: char| !c.is_alphanumeric())
                        .filter(|w| !w.is_empty())
                        .map(str::to_lowercase),
                );
                inside.clear();
            }
            _ if depth > 0 => inside.push(c),
            _ => {}
        }
    }
    out
}

/// The art server's platforms to look in for a cart, its own first. A Game Boy game
/// is often kept as `.gbc` and the other way round.
pub fn platforms(platform: Platform) -> &'static [&'static str] {
    match platform.hardware() {
        Platform::Gba | Platform::Gbav => &["GBA"],
        Platform::Gb => &["GB", "GBC"],
        Platform::Gbc => &["GBC", "GB"],
        Platform::Ws => &["WS"],
        Platform::Lynx => &["Lynx"],
        Platform::Gg => &["GG"],
        Platform::Ngp => &["NGP"],
        Platform::Tg16 => &["TG16"],
    }
}

const PRERELEASE: [&str; 6] = ["proto", "beta", "demo", "sample", "kiosk", "pirate"];
const REGIONS: [&str; 4] = ["world", "usa", "europe", "japan"];

/// How well an entry named `name` suits a file with `file_tags`, lowest best: a
/// release before a prototype or demo, then the file's own region, then World, USA,
/// Europe and Japan, then the shorter name (the fewest extra tags).
pub fn rank(name: &str, file_tags: &[String]) -> (bool, usize, usize) {
    let own = tags(name);
    let early = own.iter().any(|t| PRERELEASE.contains(&t.as_str()));
    let same = own
        .iter()
        .any(|t| REGIONS.contains(&t.as_str()) && file_tags.contains(t));
    let region = match same {
        true => 0,
        false => {
            1 + REGIONS
                .iter()
                .position(|r| own.iter().any(|t| t == r))
                .unwrap_or(REGIONS.len())
        }
    };
    (early, region, name.len())
}
