# Screen settings: system defaults, game overrides

Status: agreed with the user on 2026-11-06; not built yet.

## What the user asked for

- The shelf sets each system's defaults; a game can set its own values on top. Overrides
  without extra menus.
- On a system's shelf (not Recents or Favorites), that system's rows in Screen and in
  Controls > In-Game move to the top of their list and are tinted, while every other
  system stays a press away.
- A game's own values are tinted too, so they stand out from the defaults.

## How it works today

Every Screen setting has one home, per system, in `slot.state`; the shelf's
Settings > Screen > (system) page and the in-game Screen page both edit it ("one shared
value", 2026-11-05). Nothing on the Screen page is per game. Per game today: the core
(`selected_core.txt`), the cart's look (`cart_shell.txt`), the DS's screens
(`nds_view.txt`), the Lynx's hold (`lynx_hold.txt`), and upstream's stretch on panels
other than the square one (`video_mode.txt`).

## The model

Two layers, read top first:

1. **The game**, `Config/game_screen.txt`: one line per game that changed something,
   holding only what it changed, for example `Tetris=shader:lcd,scale:fit`.
2. **The system**, `slot.state`, as now.

A game without a line, or without a setting on its line, follows its system, so a shelf
change reaches every game that hasn't chosen otherwise.

Settings that take an override: Shader, Color Correction, Palette, Bezel, Scale, Stretch,
Frame Blending. Brightness, Blue Light, Saturation and Contrast stay one global value:
they're about the room, not the game.

Controls can take the same game layer later (`Config/controls/game/<stem>.txt`), with the
same rules.

## Menus

- **The shelf's pages edit only the defaults**, and show them even while a game is in the
  slot.
- **The in-game Screen page edits the game.** Changing a row writes the game's own value.
- **Y on an in-game row puts it back to the system's value** (removes the override), as Y
  puts a controls row back to its default. The legend shows "Default" only on a row the
  game has changed.
- **The Stretch shortcuts (MENU+L1/R1) write the game's value**, since they act in a game.
- **Tint:** a game's own value is drawn in the accent ink; a value that follows the system
  is drawn as now.

## The shelf's system first

When the shelf shows one system:

- Settings > Screen lists General first, then that system, then the rest in the usual
  order. Controls > In-Game lists that system first.
- That system's row label is drawn in the accent ink.
- On Recents and Favorites, both lists keep their usual order and no tint.

One ink means one thing: **this is about what you're looking at** (the system on the
shelf, the game in the slot). It needs to read on every theme and over a game; pick it
with the user on the device.

## What it needs in the code

- `slot-store`: the override file (read, write, remove one setting), and a resolved
  "settings for this game" view the session and the renderer read instead of `slot.state`
  directly.
- `slot`: the in-game rows write overrides; the shelf rows keep writing `slot.state`;
  `video_mode.txt` folds into the new file for the systems that stretch.
- `slot-ui`: a page whose order can change at run time (today a page is a fixed list:
  `QuickRow::page`, `shown`, `up`/`down`, `quick_window` all read it), and tinted faces for
  labels and values (a third face beside the dim and lit ones).

## Building it

1. Overrides for the Screen settings, with Y to clear and the tint on values.
2. The shelf's system first and tinted, on Screen and Controls > In-Game.
3. Later: game overrides for controls.

Each step is tested on the device before the next.
