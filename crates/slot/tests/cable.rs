use slot::cable::{decode, encode, Cable, DELAY};
use slot_retro::ButtonMask;

const A: ButtonMask = ButtonMask(ButtonMask::A);
const B: ButtonMask = ButtonMask(ButtonMask::B);

#[test]
fn a_fresh_cable_can_run_the_delay_before_the_peer_says_anything() {
    let mut c = Cable::new(0);
    for _ in 0..DELAY {
        assert!(c.ready().is_some(), "a seeded frame needed the peer");
        c.advance();
    }
    assert!(
        c.ready().is_none(),
        "it ran past the seed without the peer's masks"
    );
}

#[test]
fn a_sampled_mask_is_stamped_for_a_later_frame() {
    let mut c = Cable::new(0);
    let (frame, mask) = decode(&c.sample(A)).expect("its own packet did not parse");
    assert_eq!(frame, DELAY);
    assert_eq!(mask, A);
}

#[test]
fn each_end_reports_the_pair_in_port_order() {
    let mut p0 = Cable::new(0);
    let mut p1 = Cable::new(1);
    p1.prime();
    for _ in 0..DELAY {
        let a = p0.sample(A);
        let b = p1.sample(B);
        assert!(p0.accept(&b));
        assert!(p1.accept(&a));
        p0.advance();
        p1.advance();
    }

    assert_eq!(p0.ready(), Some((A, B)), "player 0 read the pair swapped");
    assert_eq!(
        p1.ready(),
        Some((A, B)),
        "player 1 disagreed with player 0 about port order"
    );
}

#[test]
fn a_frame_waits_rather_than_guessing_a_missing_mask() {
    let mut c = Cable::new(0);
    for _ in 0..DELAY {
        c.sample(A);
        c.advance();
    }
    c.sample(A);
    assert!(c.ready().is_none(), "it ran without the peer's mask");
    c.stall();
    c.stall();
    assert_eq!(c.stalled(), 2);

    assert!(c.accept(&encode(DELAY, B)));
    assert_eq!(c.ready(), Some((A, B)));
    c.advance();
    assert_eq!(c.stalled(), 0, "the stall count survived a frame running");
}

#[test]
fn both_ends_agree_on_every_frame() {
    let mut p0 = Cable::new(0);
    let mut p1 = Cable::new(1);
    p1.prime();
    let script = [A, B, A, ButtonMask::default(), B, B, A];

    let mut seen0 = Vec::new();
    let mut seen1 = Vec::new();
    for (i, m) in script.iter().enumerate() {
        let a = p0.sample(*m);
        let b = p1.sample(script[script.len() - 1 - i]);
        p0.accept(&b);
        p1.accept(&a);
        if let Some(pair) = p0.ready() {
            seen0.push(pair);
            p0.advance();
        }
        if let Some(pair) = p1.ready() {
            seen1.push(pair);
            p1.advance();
        }
    }
    assert_eq!(seen0, seen1, "the two ends ran different frames");
    assert!(seen0.len() >= script.len() - DELAY as usize);
}

#[test]
fn junk_and_duplicates_are_dropped_rather_than_raised() {
    let mut c = Cable::new(0);
    assert!(!c.accept(&[9, 2, 3]), "a short packet was taken");
    assert!(!c.accept(&[]), "an empty packet was taken");
    for _ in 0..DELAY {
        c.sample(B);
        c.advance();
    }
    assert!(c.accept(&encode(DELAY, A)));
    c.sample(B);
    assert_eq!(c.ready(), Some((B, A)));
    c.advance();
    assert!(
        !c.accept(&encode(DELAY, A)),
        "a mask for a frame already run was taken back in"
    );
}

#[test]
fn a_stall_does_not_add_to_the_input_delay() {
    let mut c = Cable::new(0);
    for _ in 0..DELAY {
        c.sample(A);
        c.advance();
    }
    let (stamped, _) = decode(&c.sample(B)).expect("packet");
    let ahead = stamped - c.frame();

    for _ in 0..20 {
        c.sample(ButtonMask::default());
        c.stall();
    }
    assert_eq!(c.stalled(), 20);

    let (stamped, mask) = decode(&c.sample(ButtonMask::default())).expect("packet");
    assert_eq!(
        stamped - c.frame(),
        ahead,
        "the input delay grew across a stall: masks are being stamped per present again"
    );
    assert_eq!(
        mask, B,
        "a decided mask was revised, which desyncs the pair"
    );
}

#[test]
fn a_joiner_runs_nothing_until_it_has_the_hosts_machine() {
    let mut join = Cable::new(1);
    assert!(!join.primed(), "a fresh joiner claimed to be ready");
    assert!(
        join.ready().is_none(),
        "the joiner ran a seeded frame on its own machine"
    );

    // Slot²: bytes that don't compress (states are packed, link step 4), so it still
    // takes several pieces.
    let mut x = 0x2545_f491u32;
    let state: Vec<u8> = (0..150_000)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            x as u8
        })
        .collect();
    let packets = slot::cable::state_packets(&state);
    assert!(packets.len() > 1, "a state this size must be cut up");
    for (i, p) in packets.iter().enumerate() {
        assert!(p.len() <= 65_535, "chunk {i} cannot cross the wire");
        assert!(join.accept(p));
        if i + 1 < packets.len() {
            assert!(
                join.take_state().is_none(),
                "a part was taken for the whole"
            );
        }
    }
    assert_eq!(join.take_state().as_deref(), Some(state.as_slice()));
    assert!(
        join.take_state().is_none(),
        "the state was handed over twice"
    );

    join.prime();
    assert!(join.ready().is_some(), "priming did not let the frames run");
}

#[test]
fn a_host_is_ready_from_the_start() {
    let host = Cable::new(0);
    assert!(host.primed());
    assert!(host.ready().is_some());
}

#[test]
fn a_state_chunk_is_never_read_as_a_button_mask() {
    let mut c = Cable::new(0);
    for p in slot::cable::state_packets(&[9u8; 32]) {
        assert!(decode(&p).is_none(), "a state chunk parsed as a mask");
        assert!(c.accept(&p));
    }
    assert!(c.accept(&encode(DELAY, A)));
}

#[test]
fn a_stall_during_the_swap_is_not_a_lost_peer() {
    let mut host = Cable::new(0);
    assert!(
        !host.armed(),
        "the host armed before the joiner said it was running"
    );
    for _ in 0..200 {
        host.stall();
    }
    assert!(
        !host.armed(),
        "a long swap was read as a peer that vanished"
    );

    assert!(host.accept(&slot::cable::ready_packet()));
    assert!(host.armed(), "the joiner said it was ready and was ignored");
}

#[test]
fn a_joiner_is_not_armed_until_it_has_restored() {
    let mut join = Cable::new(1);
    assert!(!join.armed(), "an unprimed joiner armed");
    join.prime();
    assert!(join.armed(), "the joiner never armed after restoring");
}

/// Slot² (link step 3): a link is quiet only once it's armed, both sides holding the game
/// (the joiner restored the host's state, the host heard its READY), counted from then.
/// Upstream counted the joiner's wait for the game as silence, and links the RGB20SX
/// hosts, whose state takes 6-16 s to arrive, ended 3 s in (2026-11-08). A start that
/// never arms ends after `START_FRAMES`.
#[test]
fn a_link_is_quiet_only_once_both_sides_have_the_game() {
    use slot::cable::{ready_packet, QUIET_FRAMES, START_FRAMES};
    // The joiner, waiting for the game.
    let mut joiner = Cable::new(1);
    for _ in 0..QUIET_FRAMES * 10 {
        joiner.stall();
    }
    assert!(!joiner.quiet(), "waiting for the game counted as silence");
    joiner.prime();
    assert!(!joiner.quiet(), "the wait carried over into the game");
    for _ in 0..QUIET_FRAMES - 1 {
        joiner.stall();
    }
    assert!(!joiner.quiet());
    joiner.stall();
    assert!(
        joiner.quiet(),
        "a running link quiet for {QUIET_FRAMES} frames"
    );

    // The host, waiting for the joiner to load the game.
    let mut host = Cable::new(0);
    for _ in 0..QUIET_FRAMES * 10 {
        host.stall();
    }
    assert!(!host.quiet(), "the joiner loading counted as silence");
    assert!(host.accept(&ready_packet()));
    assert!(!host.quiet(), "the wait carried over past READY");
    for _ in 0..QUIET_FRAMES {
        host.stall();
    }
    assert!(host.quiet());

    // A start that never arms.
    let mut lonely = Cable::new(0);
    for _ in 0..START_FRAMES {
        lonely.stall();
    }
    assert!(
        lonely.quiet(),
        "a start that never armed kept the link forever"
    );
}

/// Slot² (link step 4): the host's game crosses packed. A Game Boy state is mostly zeros
/// (Tetris's 202 KB packs to 4.5 KB with gzip); sent as it was, the 405 KB took ~3 s to
/// a dozing joiner (2026-11-08).
#[test]
fn the_hosts_game_crosses_packed_and_arrives_whole() {
    let mut state = vec![0u8; 405_580];
    for (i, b) in state.iter_mut().enumerate().step_by(7) {
        *b = (i % 251) as u8;
    }
    let packets = slot::cable::state_packets(&state);
    let sent: usize = packets.iter().map(Vec::len).sum();
    assert!(
        sent * 5 < state.len(),
        "{sent} bytes sent for a {} byte state",
        state.len()
    );
    let mut join = Cable::new(1);
    for p in &packets {
        assert!(join.accept(p));
    }
    assert_eq!(join.take_state().as_deref(), Some(state.as_slice()));
}

/// Slot² (link step 4): a stock slotPlus host sends its game as it is; a Slot² joiner
/// still reads it.
#[test]
fn a_game_sent_unpacked_still_arrives_whole() {
    let state: Vec<u8> = (0..1000u32).map(|n| (n * 7) as u8).collect();
    let mut join = Cable::new(1);
    let (first, last) = state.split_at(600);
    assert!(join.accept(&[&[1u8][..], first].concat()), "KIND_STATE");
    assert!(join.accept(&[&[2u8][..], last].concat()), "KIND_STATE_END");
    assert_eq!(join.take_state().as_deref(), Some(state.as_slice()));
}
