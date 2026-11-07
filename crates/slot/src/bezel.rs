use std::path::{Path, PathBuf};

use slot_store::Platform;

pub const BEZELS: &str = "System/bezels";
pub const GAME_BEZELS: &str = "Bezels";

/// A platform's own bezels: its PNGs.
pub fn list(root: &Path, platform: Platform) -> Vec<String> {
    let dir = root.join(GAME_BEZELS).join(platform.hardware().dir_name());
    let mut files: Vec<String> = std::fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| !slot_store::is_hidden(p))
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("png")))
        .filter_map(|p| Some(p.file_name()?.to_str()?.to_owned()))
        .collect();
    files.sort_by(|a, b| {
        a.to_lowercase()
            .cmp(&b.to_lowercase())
            .then_with(|| a.cmp(b))
    });
    files
}

/// The PNG a picked bezel draws.
pub fn file(root: &Path, platform: Platform, name: &str) -> PathBuf {
    root.join(GAME_BEZELS)
        .join(platform.hardware().dir_name())
        .join(name)
}

fn dims(path: &Path) -> Option<(u32, u32)> {
    if !path.extension()?.eq_ignore_ascii_case("png") {
        return None;
    }
    let (w, h) = path.file_stem()?.to_str()?.split_once('x')?;
    let (w, h) = (w.parse::<u32>().ok()?, h.parse::<u32>().ok()?);
    (w > 0 && h > 0).then_some((w, h))
}

pub fn pick(root: &Path, panel: (u32, u32)) -> Option<PathBuf> {
    let mut fits: Vec<((u32, u32), PathBuf)> = std::fs::read_dir(root.join(BEZELS))
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| !slot_store::is_hidden(p))
        .filter_map(|p| Some((dims(&p)?, p)))
        .filter(|((w, h), _)| {
            u64::from(*w) * u64::from(panel.1) == u64::from(*h) * u64::from(panel.0)
        })
        .collect();
    fits.sort_by_key(|((w, _), p)| (w.abs_diff(panel.0), p.clone()));
    fits.into_iter().next().map(|(_, p)| p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folders_are_not_bezels() {
        let d = tempfile::tempdir().unwrap();
        let tg16 = d.path().join(GAME_BEZELS).join("TG16");
        std::fs::create_dir_all(tg16.join("Set")).unwrap();
        std::fs::write(tg16.join("Mine.png"), b"").unwrap();
        assert_eq!(list(d.path(), Platform::Tg16), ["Mine.png"]);
    }
}
