# Shortcuts: general, per system, per game

Status: agreed with the user on 2026-11-06. Steps 1, 2 and 3 built.

**The DS was removed on 2026-11-07** ([two-devices.md](two-devices.md), phase 5): its mentions
below are history.

## What the user asked for

- Many shortcuts belong to one system or a few. They should work there and nowhere else,
  and the menu should show them that way: General, then a page per system, like Screen.
- When a shortcut and a game input share a button, the game input wins every time, so a
  game stays playable.
- Shortcuts join the per-game settings. A niche one (Sunlight, used by a couple of games)
  must not block every other game, and can still be set up in the games that use it.
- A shortcut whose feature is switched off on the Gameplay page (fast forward at speed 0,
  rewind off) does nothing: its keys aren't listened for at all.

Trigger: the DS's Touch bound to R3, which Sunlight Up already had, so R3 did nothing in
the game.

## 1. The game's input wins (built)

In a game, a shortcut is left out when it would take a press the playing system's
controls need (`input::collides`): the same button in the same layer. Plain buttons and
two-button chords are one layer, MENU chords another, SELECT chords a third. START leads
its chords, so a START chord takes only its partner. The shortcut still works on the
shelf and in menus, and the controls and shortcut pages flag the clash.

This replaces the DS-only rule that let its controls win over shortcuts on L2 and R2. It
also gives the WonderSwan its shoulders back from Fast Forward (R2) and Rewind (L2).

Fast forward at speed 0, and rewind switched off or not recorded (a DS game), aren't
listened for at all: their keys are the game's, with no refusal.

The rule adds no input lag: the list of shortcuts was already rebuilt every frame.

## 2. Shortcuts per system (built)

| Group | Shortcuts |
|---|---|
| General | Save State, Load State, Reset, Rewind (not the DS), Fast Forward, Fast Forward Toggle, Screenshot, Guide, Brightness Up/Down, Blue Light Up/Down, Shader Next/Back |
| Game Boy | Colour Correction, Palette Back/Next, Stretch On/Off |
| Game Boy Color | Colour Correction, Stretch On/Off |
| Game Boy Advance | Colour Correction, Sunlight Up/Down |
| Game Gear, NeoGeo Pocket, TurboGrafx-16, WonderSwan, Lynx | Stretch On/Off |

- Controls > Shortcuts lists General, then the systems with shortcuts of their own, in the
  Controls page's order (GB, GBC, GBA, WS, Lynx, GG, NGP, TG16), each opening its page.
- Each system page has its own keys for its shortcuts, kept in
  `Config/shortcuts/<system>.txt` with only the rows that differ from where they start:
  the keys `shortcuts.txt` has for them (upstream's file, so an upstream card carries over).
  The GBA on a frame shares the GBA's. Code: `slot_store::shortcuts`.
- A system shortcut is listened for only in that system's games (`input::Table`), and a
  clash is flagged only between rows one system has.
- Y's reset on the General page resets the general shortcuts only; on a system page, that
  system's.
- Sunlight ships unbound.

## Shortcuts follow a system's features (2026-11-07)

Found by the user: a system feature added later (the WonderSwan's and the TurboGrafx-16's
palettes) didn't bring its shortcuts, because the systems of each shortcut and the
shortcut pages were lists written by hand.

- `slot_store::features` says what each system can do that a shortcut drives (colour
  correction, palettes, stretch, sunlight), from the same facts the Screen rows use. A
  shortcut belongs to a feature (`shortcuts::feature_of`), and a system has it when it has
  the feature.
- Controls > Shortcuts has a page for every system, the DS's too; a system with no
  shortcuts of its own has its page hidden. A system that gains a feature shows its page
  and the feature's rows with no change to the menu.
- A system's keys start from `shortcuts.txt`'s, but a feature a system gained later
  starts unbound there (`features::starts_bound`): the WonderSwan's and the
  TurboGrafx-16's Palette Back/Next.
- Palette Back/Next in a WonderSwan (mono) or TurboGrafx-16 game step the game's own
  palette, as the in-game Palette row does, and the toast names it.
- The DS has Stretch On/Off too (asked for on 2026-11-07, after the invariant tests showed
  its Stretch row had none), unbound until set. They act where its Stretch row is: in a
  game in the One Screen layout on a tall panel, as the game's own.
- Found on the device the same day: the system pages showed each other's names (the
  WonderSwan's page was labelled TurboGrafx-16, the Game Gear's WonderSwan). Reordering
  the pages on 2026-11-06 had moved them in `QuickRow::ALL` but not in the enum, and a
  row's label is found by its enum place. Fixed, and a test now checks every row's place.

## 3. Per game (built)

- A fourth row of the in-game Settings (Screen, Gameplay, Controls, Shortcuts) opens one
  page: the playing system's own shortcuts first, then the general ones.
- It edits this game's own keys over the system's and the general ones; Y puts a row back
  to them. It shows the general shortcuts and the playing system's.
- The game's keys live in its own file with its screen settings and controls,
  `Config/games/<system>/<file name>.txt` ([game-overrides.md](game-overrides.md)).

## Later

- SELECT is held back up to 600 ms in a game when SELECT chords exist (it may be the first
  key of one), so a game sees SELECT late or on release. Worth revisiting.
