use std::collections::HashMap;

use serde_json::Value;
use slot_store::Platform;
use slot_ui::house_label::Band;

#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    pub ground: [u8; 3],
    pub band: Option<Band>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub real: Option<String>,
    pub logo: Option<String>,
    pub look: Option<Look>,
    /// Slot²: the entry's No-Intro name, to choose between entries of one name.
    pub name: Option<String>,
}

pub struct ArtIndex {
    entries: Vec<Entry>,
    by_crc: HashMap<u32, usize>,
    /// Slot²: entries by platform and name key (`super::name::key`), names and akas.
    by_name: HashMap<(String, String), Vec<usize>>,
}

impl ArtIndex {
    pub fn parse(json: &str) -> Option<ArtIndex> {
        let Value::Object(map) = serde_json::from_str::<Value>(json).ok()? else {
            return None;
        };
        let mut index = ArtIndex {
            entries: Vec::new(),
            by_crc: HashMap::new(),
            by_name: HashMap::new(),
        };
        for (key, raw) in &map {
            let at = index.entries.len();
            index.entries.push(entry(raw));
            let mut crcs: Vec<u32> = crc_of(key).into_iter().collect();
            if let Some(list) = raw["crcs"].as_array() {
                crcs.extend(list.iter().filter_map(|c| c.as_str().and_then(crc_of)));
            }
            for crc in crcs {
                index.by_crc.insert(crc, at);
            }
            if let Some(platform) = raw["platform"].as_str() {
                let akas = raw["aka"].as_array().into_iter().flatten();
                let names = std::iter::once(&raw["name"]).chain(akas);
                for name in names.filter_map(Value::as_str) {
                    let key = super::name::key(name);
                    if !key.is_empty() {
                        let at_name = index.by_name.entry((platform.into(), key)).or_default();
                        if !at_name.contains(&at) {
                            at_name.push(at);
                        }
                    }
                }
            }
        }
        Some(index)
    }

    pub fn get(&self, crc: u32) -> Option<&Entry> {
        self.by_crc.get(&crc).map(|&at| &self.entries[at])
    }

    /// Slot²: the entry for a file named `stem`, for when its checksum isn't here (a
    /// translation or a hack): the same name on the cart's platform, the best of several
    /// by `super::name::rank`.
    pub fn named(&self, platform: Platform, stem: &str) -> Option<&Entry> {
        let key = super::name::key(stem);
        if key.is_empty() {
            return None;
        }
        let file_tags = super::name::tags(stem);
        super::name::platforms(platform).iter().find_map(|p| {
            let found = self.by_name.get(&(p.to_string(), key.clone()))?;
            found
                .iter()
                .map(|&at| &self.entries[at])
                .min_by_key(|e| super::name::rank(e.name.as_deref().unwrap_or(""), &file_tags))
        })
    }

    /// Slot²: a cart's entry, by its checksum, else by its file name.
    pub fn for_cart(&self, crc: u32, platform: Platform, stem: &str) -> Option<&Entry> {
        self.get(crc).or_else(|| self.named(platform, stem))
    }
}

fn crc_of(key: &str) -> Option<u32> {
    match key.len() == 8 {
        true => u32::from_str_radix(key, 16).ok(),
        false => None,
    }
}

fn entry(raw: &Value) -> Entry {
    let path = |k: &str| raw[k].as_str().map(str::to_string);
    Entry {
        real: path("support-texture"),
        logo: path("wheel"),
        look: look(&raw["look"]),
        name: path("name"),
    }
}

fn rgb(v: &Value) -> Option<[u8; 3]> {
    let s = v.as_str()?;
    if s.len() != 6 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let at = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).ok();
    Some([at(0)?, at(2)?, at(4)?])
}

fn look(raw: &Value) -> Option<Look> {
    if raw["sure"].as_bool() != Some(true) {
        return None;
    }
    let ground = rgb(&raw["ground"])?;
    let b = &raw["band"];
    if b.is_null() {
        return Some(Look { ground, band: None });
    }
    let size = b["size"].as_f64()? as f32;
    if !(size > 0.0 && size < 0.5) {
        return None;
    }
    let band = Band::parse(b["edge"].as_str()?, size, &rgb(&b["colour"])?)?;
    Some(Look {
        ground,
        band: Some(band),
    })
}
