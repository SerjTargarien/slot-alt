# Checklist: adding a setting, a row, a system feature

Written on 2026-11-07 (the structure pass, HANDOFF.md). A fact here is often kept in
more than one place; this lists the places, and the test that fails when one is missed.
The tests are the safety net, not the list: run `sh tools/slotsq/test.sh` and read what
fails.

## A menu row

1. Add it once to the `quick_rows!` list in `crates/slot-ui/src/quick_menu.rs` (the enum
   and `QuickRow::ALL` come from it, in one order).
2. Put it on its page: the page constant (`GAME`, `SCREEN_WS`, …) and the matching arm of
   `QuickRow::page()`. *Caught by* `invariants::every_row_is_on_the_page_it_names`,
   `every_page_list_is_the_page_of_each_of_its_rows`, `every_row_is_on_exactly_one_page_list`.
3. Give it a label in `QuickRow::label()`. *Caught by* `every_row_has_a_label`. A page's
   label test (`quick_menu::...` listing a page's labels) needs it too.
4. If it opens a page: `child()`, `opens()`, and the page's way back in `parent()`
   (keyed by the page's first row). *Caught by* `a_rows_child_is_the_first_row_of_a_page`,
   `a_row_that_says_it_opens_a_page_has_one`, `back_from_a_page_returns_to_the_row_that_opened_it`,
   `every_row_can_be_reached_from_the_main_or_the_in_game_menu`.
5. Its value (`App::quick_value`) and what Left and Right do (`App::change_setting`).
   Both end in catch-alls, so the compiler won't ask. *Caught by*
   `every_shelf_row_with_a_value_changes_with_left_and_right`.
6. If it's hidden somewhere: `App::hidden_rows` (the shelf) or `App::in_game_hidden`.

## A setting kept in `slot.state`

1. The field on `SlotState`, its default, the line `write_slot_state` writes and the line
   `parse` reads (`crates/slot-store/src/slot_state.rs`). *Caught by*
   `invariants::every_slot_state_field_is_written_and_read`, which won't compile until the
   field is given a value there, and fails until it's written and read.
2. `durability.rs`'s test that spells out every field (it won't compile either).
3. A file without the line must read as today's behaviour (the default).

## A setting a game can have its own value for

1. Screen settings: a field on `Screen` (read and set on `SlotState` in
   `crates/slot-store/src/games.rs`) and on `ScreenOverrides` (`apply`, `record`, `has`,
   `clear`, `value`, `read`), and a `ScreenSetting`. Gameplay ones: `Play`,
   `PlayOverrides`, `PlaySetting` the same way.
2. In the app: the in-game row in `game_setting()` (`crates/slot/src/app.rs`), and, if a
   core is told of it, the pending value re-sent in `clear_game_setting`.
3. *Caught by* `invariants::a_game_with_its_own_everything_reads_back_as_itself` (store)
   and `game_overrides::every_in_game_setting_is_the_games_own_or_named_as_every_games`
   (a row that's meant to be every game's is named there).

## A system feature with shortcuts

1. `slot_store::features`: a `Feature` (and in `Feature::ALL`), `has_feature`, and
   `starts_bound` (a system that gains a feature later starts with its keys unbound).
2. Its shortcuts: `Shortcut` in `bindings.rs` (key, label, default binding) and
   `shortcuts::feature_of`; the action in the app (`App::adjust` or `Phase::Playing`).
3. The system's Screen row, if the feature has one. *Caught by*
   `invariants::a_systems_features_are_the_rows_its_screen_page_has` (palette, colour
   correction, stretch) and `every_feature_has_shortcuts_and_a_system_that_has_it`.
4. The shortcut pages follow by themselves (a system with none has its page hidden).

## A core option

1. Spell it in one function beside the others in `crates/slot/src/core.rs`
   (`blend_option`, `sprite_option`, `colour_option`), with a unit test per core: cores
   spell the same idea differently (`Enabled` and `enabled`), and some ask it inverted.
2. Set it as the core opens (`session::spawn_core`) and, if it can change in a game, live
   (a pending value the session sends).
3. If the option is new to the core (a pin bump, a patch), regenerate the snapshot:
   `python3 -I tools/slotsq/core_options.py` (`cores/options.json`). *Caught by*
   `core_options::every_option_slot_sets_is_one_its_core_declares` (a misspelt key or
   value), once the function that sets it runs in that test.
4. Check it changes live in the core's source (`check_variables` on
   `GET_VARIABLE_UPDATE`), and try it on the device first with `SLOT_CORE_OPTIONS`
   (design/core-options.md).

## Where Slot²'s own code goes

New code of Slot²'s own goes in a file of its own beside upstream's, as an `impl` block
of the same type: `crates/slot/src/app/*.rs` for `App` (`games.rs`, `game_keys.rs`,
`core_options.rs`), `crates/slot/src/session/*.rs` for `Session` (`ws.rs`). Upstream's
files then change only where they must call it, which keeps upstream's updates easy to
take and Slot²'s features easy to lift out.

## Formatting

`sh tools/slotsq/fmt.sh` formats the changed files that were formatted already, and names
the others (upstream's unformatted ones): format your lines in those by hand. Never
`cargo fmt --all`, nor `rustfmt` on one of those files.
