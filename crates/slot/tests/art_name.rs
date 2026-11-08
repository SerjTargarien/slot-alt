//! Slot² (2026-11-08): a cart whose checksum isn't in the art index (a translation, a
//! hack) finds its label by name.

use std::collections::HashMap;

use slot::art::index::ArtIndex;
use slot::art::jobs::{fetch_missing_with, ArtEvent, ART_BASE};
use slot::art::make::Kind;
use slot::art::name::key;
use slot::update::UpdateFail;
use slot_store::Platform;

#[test]
fn a_file_name_and_its_no_intro_name_share_a_key() {
    for (file, listed) in [
        (
            "Cave Noire (Japan) (En) (v1.0) (Aeon Genesis)",
            "Cave Noire (Japan)",
        ),
        (
            "Dicing Knight. (Japan) [T-En by Aeon Genesis v1.00]",
            "Dicing Knight. (Japan)",
        ),
        (
            "The Final Fantasy Legend",
            "Final Fantasy Legend, The (USA)",
        ),
        ("Pokémon Picross", "Pokemon Picross (Japan) (Proto)"),
        ("Mario & Luigi", "Mario and Luigi (USA)"),
        ("Gargoyles Quest", "Gargoyle's Quest (USA, Europe)"),
    ] {
        assert_eq!(key(file), key(listed), "{file} and {listed}");
    }
    assert_ne!(
        key("Tetris"),
        key("Tetris DX (World)"),
        "a near name matched"
    );
    assert_eq!(key("(Japan) [b]"), "", "tags alone made a name");
}

fn index() -> ArtIndex {
    ArtIndex::parse(
        r#"{
        "00000001":{"name":"Cave Noire (Japan)","platform":"GB","support-texture":"sq/GB/support-texture/1.png"},
        "00000002":{"name":"Denki Blocks! (Japan)","platform":"GBA","support-texture":"jp.png"},
        "00000003":{"name":"Denki Blocks! (Europe) (En,Fr,De,Es,It)","platform":"GBA","support-texture":"eu.png"},
        "00000004":{"name":"Denki Blocks! (USA) (En,Es)","platform":"GBA","support-texture":"us.png"},
        "00000005":{"name":"Denki Blocks! (USA) (Proto)","platform":"GBA","support-texture":"proto.png"},
        "00000006":{"name":"Tetris (World)","platform":"GB","support-texture":"tetris.png"},
        "00000007":{"support-texture":"GB/support-texture/7.png"},
        "metroid-ii-dx":{"name":"Metroid II - Return of Samus DX","aka":["Metroid II DX"],"platform":"GBC","crcs":["00000008"],"wheel":"m2.png"}
    }"#,
    )
    .unwrap()
}

fn real(entry: Option<&slot::art::index::Entry>) -> Option<&str> {
    entry.and_then(|e| e.real.as_deref().or(e.logo.as_deref()))
}

#[test]
fn a_translation_finds_the_game_it_was_made_from() {
    let ix = index();
    let found = ix.named(
        Platform::Gb,
        "Cave Noire (Japan) (En) (v1.0) (Aeon Genesis)",
    );
    assert_eq!(real(found), Some("sq/GB/support-texture/1.png"));
}

#[test]
fn of_several_regions_the_files_own_comes_first_then_usa() {
    let ix = index();
    assert_eq!(
        real(ix.named(Platform::Gba, "Denki Blocks!")),
        Some("us.png")
    );
    assert_eq!(
        real(ix.named(Platform::Gba, "Denki Blocks! (Japan) [T-En]")),
        Some("jp.png")
    );
    assert_eq!(
        real(ix.named(Platform::Gba, "Denki Blocks! (Europe) (Hack)")),
        Some("eu.png")
    );
}

#[test]
fn a_name_stays_on_its_own_system() {
    let ix = index();
    assert!(
        ix.named(Platform::Gba, "Cave Noire").is_none(),
        "a GBA cart took a Game Boy label"
    );
    assert_eq!(
        real(ix.named(Platform::Gbc, "Cave Noire")),
        Some("sq/GB/support-texture/1.png"),
        "a Game Boy game kept as .gbc missed"
    );
}

#[test]
fn an_aka_is_a_name_too() {
    let ix = index();
    assert_eq!(
        real(ix.named(Platform::Gbc, "Metroid II DX")),
        Some("m2.png")
    );
}

#[test]
fn the_checksum_wins_over_the_name() {
    let ix = index();
    let entry = ix.for_cart(0x0000_0006, Platform::Gba, "Denki Blocks!");
    assert_eq!(real(entry), Some("tetris.png"));
    let entry = ix.for_cart(0xDEAD_BEEF, Platform::Gba, "Denki Blocks!");
    assert_eq!(real(entry), Some("us.png"));
}

#[test]
fn an_index_without_names_finds_nothing_by_name() {
    let ix = ArtIndex::parse(r#"{"00000007":{"support-texture":"x.png"}}"#).unwrap();
    assert!(ix.named(Platform::Gb, "Anything").is_none());
}

fn png(w: u32, h: u32) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header()
            .unwrap()
            .write_image_data(&[90, 60, 200, 255].repeat((w * h) as usize))
            .unwrap();
    }
    out
}

#[test]
fn fetch_art_labels_a_patched_game_by_its_name() {
    let d = tempfile::tempdir().unwrap();
    let games = d.path().join("Games/GB");
    std::fs::create_dir_all(&games).unwrap();
    std::fs::write(
        games.join("Cave Noire (Japan) (En) (v1.0) (Aeon Genesis).gb"),
        vec![7u8; 0x200],
    )
    .unwrap();
    let served = HashMap::from([(
        format!("{ART_BASE}sq/GB/support-texture/1.png"),
        png(400, 352),
    )]);
    let fetch = |url: &str| served.get(url).cloned().ok_or(UpdateFail::Download);
    let mut events = Vec::new();
    fetch_missing_with(
        d.path(),
        Kind::Real,
        &index(),
        &fetch,
        &|| false,
        &mut |e| events.push(e),
    );
    assert!(
        matches!(events.last(), Some(ArtEvent::Done { landed: 1, .. })),
        "{events:?}"
    );
    assert!(d
        .path()
        .join("Labels/GB/Cave Noire (Japan) (En) (v1.0) (Aeon Genesis).png")
        .is_file());
}
