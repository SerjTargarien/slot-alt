# Controls: per-system, several buttons per input

Status: steps 1-3 built and checked on the device (2026-11-06); step 4, defaults per
system, is next.

## What the user asked for

- Each system has its own mapping, on its own page, like the Screen pages.
- An emulated input can take **up to three** physical buttons.
- Fix the input model globally first; settle each system's defaults after.
- Trigger: on the WonderSwan, Dicing Knight's items sit on the Y pad, which is only
  reachable by holding L or R.

## How it works today

1. **Physical buttons.** evdev events become slot's `Btn`s (D-pad, A, B, X, Y, L1, R1, L2,
   R2, Select, Start, Menu). The left stick can steer like the D-pad (a menu switch); the
   right stick isn't read.
2. **One shared set of 12 game buttons** (`GameButton`: D-pad, A, B, Select, Start, L, R,
   Turbo A, Turbo B) in `Config/controls.txt`, each bound to **one** physical button or a
   chord (Menu or Select plus a button). Every system uses this set.
3. **`Pad::mask`** turns held buttons into the core's RetroPad bits: A, B, X, Y, L, R,
   L2, R2, Select, Start, D-pad. Turbo A and B ride on X and Y, and the emulator turns them
   into auto-fire.
4. **Each system then rewrites the bits in code** (`session.rs`, `slot-retro` `core.rs`):
   the WonderSwan turns L and R into hold-to-shift keys for its Y pad and rotates its
   pads when a game is played upright; the Lynx and Game Gear drop Select; systems
   without shoulders drop L and R.

So the menu shows Game Boy names for every system, a system's real buttons can only be
reached through fixed tricks, and one physical button can drive only one input.

## The model

Three layers, each with one job.

### 1. Physical inputs

What the device has: Up, Down, Left, Right, A, B, X, Y, L1, R1, L2, R2, Select, Start,
plus each stick's four directions as buttons (LS Up/Down/Left/Right, RS
Up/Down/Left/Right; a direction counts as pressed past about half travel, with hysteresis
so it doesn't chatter). Menu stays slot's (menus and chords) and isn't bindable. The
sticks also click in (L3 and R3).

A **binding** is one physical input, optionally held with a modifier: Menu, Select, or any
other physical input (a chord, such as L1+Up). That's today's `Binding`, plus the stick
directions.

### 2. System inputs

Each platform declares its own inputs, in its own names, and which core bit each drives:

| System | Inputs |
|---|---|
| Game Boy, Game Boy Color | Up, Down, Left, Right, A, B, Select, Start, Turbo A, Turbo B |
| Game Boy Advance | the same, plus L and R |
| Game Gear | Up, Down, Left, Right, 1, 2, Start, Turbo 1, Turbo 2 |
| NeoGeo Pocket | Up, Down, Left, Right, A, B, Option, Turbo A, Turbo B |
| TurboGrafx-16 | Up, Down, Left, Right, I, II, Select, Run, Turbo I, Turbo II |
| Atari Lynx | Up, Down, Left, Right, A, B, Option 1, Option 2, Pause, Turbo A, Turbo B |
| WonderSwan | X1, X2, X3, X4, Y1, Y2, Y3, Y4, A, B, Start, Turbo A, Turbo B |
| Nintendo DS | Up, Down, Left, Right, A, B, X, Y, L, R, Select, Start, and slot's DS controls: Swap Screens, Next Layout, Stylus |

The core-bit mapping of each input (Cygne's Y pad on R2, R, L2 and L, for instance) lives
with the platform, checked against each core when built.

### 3. Bindings per system

For each system, each input has **up to three bindings**. A press reaches the core when
any of an input's bindings is held. Several inputs may share a physical button only if the
user binds them so; the page warns about a clash, as Shortcuts do today.

The systems' rewrites in code go away where bindings can say the same thing: the
WonderSwan's hold-L-for-the-Y-pad becomes the bindings `Y1 = L1+Up`, `Y2 = L1+Right`...
What stays in code is what bindings can't express:
- **Orientation**: a WonderSwan game played upright, or a Lynx held sideways, turns which
  of the system's pads the D-pad drives. That stays a transform after bindings, as now.
- **Turbo**: auto-fire, as now.
- Inputs a core must never see (Select resets the Game Gear): simply not offered.

## Storage

One file per system, `Config/controls/<system>.txt` (`gb`, `gbc`, `gba`, `gg`, `ngp`,
`tg16`, `lynx`, `ws`, `nds`): `input=binding[,binding[,binding]]`, for example

```
y1=rs_up,l1+up
a=a
```

A missing file or line means the system's default. Upstream's `Config/controls.txt` stays
as it is; on first use, its values seed the Game Boy, Game Boy Color and Game Boy Advance
files, so nothing a user set is lost.

## Menus

- Settings > Controls > In-Game lists the systems; each opens its page. In a game, the
  in-game Controls page is the playing system's.
- A row shows the input and its bindings: `Y1      RS Up · L1+Up`.
- On a row: **A** adds a binding (press the buttons to bind; with three already, it
  replaces the oldest), **X** removes the last one, **Y** puts the row back to its default.
  The legend at the bottom says so.
- A clash (the same binding on two inputs, or on an input and a shortcut active in that
  system) shows the same toast as today.

## Shortcuts and the shelf

Shortcuts stay one global page. A shortcut only acts where it applies (the Stretch keys
already skip the GBA and DS), and the binding pages warn when an input and a shortcut
share a binding. The shelf's own keys are unchanged.

## Defaults

Phase one keeps **today's behaviour exactly** for every system, written as bindings. The
better defaults per system (the WonderSwan's Y pad on the right stick or on L2/R2/X/Y, the
DS stylus on the right stick...) are decided afterwards, one system at a time.

## Building it

1. **Model and storage**: physical inputs with stick directions, system inputs, up to three
   bindings per input, per-system files, defaults equal to today. `Pad::mask` works from
   the playing system's bindings; the WonderSwan's shift code becomes default bindings.
   Tests: every system produces today's core bits for today's presses.
2. **Menus**: the per-system pages and editing several bindings.
3. **Sticks**: reading both sticks as directions (the left stick's "steer like the D-pad"
   switch becomes default bindings).
4. **Defaults**: one system at a time, with the user.

Each step is tested on the device before the next.

## Decisions (2026-11-06)

- **Editing a row:** A adds a binding (with three already, it replaces the **oldest**), X
  removes the last one, Y puts the row back to its default.
- **The left stick's "steer like the D-pad" switch goes.** Sticks are plain bindable
  inputs, and every system's defaults bind movement to both the D-pad and the left stick.
- **The DS's slot controls (Swap Screens, Next Layout, Stylus) are inputs on its page.**
  The principle: as little hard-coded input as possible. (Stretch already moved from fixed
  L1/R1 to shortcuts.) Orientation stays as it is for now.

## Built (step 1a and 2, 2026-11-06)

- **Pages:** Settings > Controls > In-Game lists the nine systems; each opens that system's
  page. In a game, Controls is the playing system's page. B goes back to the system's row.
- **One row order for every system** (Up, Down, Left, Right, Y1-Y4, A, B, X, Y, Select,
  Start, L, R, Turbo A, Turbo B), each system showing only its own rows. Where a system's
  names would read out of order, its rows drive other core buttons (`controls::drives`):
  the Game Gear's Button 1 and the Pocket's A are the core's B; the WonderSwan's X1-X4
  run up, right, down, left. The defaults follow, so every system still presses today's
  core buttons for today's presses.
- **Not offered:** Select on the Game Gear (it resets), the NeoGeo Pocket and the
  WonderSwan.
- **Capture** waits for a second button or the release on every row, so any row takes a
  chord.
- **Rows keep their names everywhere** except the clash toast, which still uses the
  standard name ("Same as Console A Button").

## Built (steps 1b and 3, 2026-11-06)

- **The DS's controls are inputs on its page:** Swap Screens (L2), Next Layout
  (SELECT+L2), Stylus (R2, held) and Touch (A). While the stylus is out, the D-pad's
  inputs move it and Touch's buttons touch, reaching the game for nothing else. In a DS
  game they win over a shortcut on the same keys, as L2 and R2 always did.
- **Both sticks are read as directions**, from each axis's range as the kernel reports it,
  pressing past half travel and letting go under a third. In a game they're LS and RS
  directions for bindings; everywhere else the left stick moves like the D-pad. The
  "Left Stick as D-Pad" switch and the `LEFT_STICK` description key are gone.
- **SELECT chords on game rows** reach the game: the gesture layer sends them as
  themselves, as it does MENU chords, instead of holding SELECT back.
