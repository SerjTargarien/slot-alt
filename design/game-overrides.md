# Game overrides: system defaults, a game's own values

Status: agreed with the user on 2026-11-06; decisions of the same day below ("Decided").
Steps 1 (screen) and 2 (shortcuts, and Gameplay with them) built; steps 3 and 4 not yet.

## Decided (2026-11-06, after the shortcut pages)

- **One file per game**, `Config/games/<system>/<file name>.txt` (the ROM's file name
  without its extension; `<system>` as `Config/controls/`), holding only what that game
  changed: its screen settings, its shortcut keys and its controls. No file means the game
  follows its system. This replaces the `game_screen.txt` and `controls/game/` of the
  model below.
- **Games are found by system and file name**, as saves, states and the other per-game
  files are. A renamed ROM loses its settings until its file is renamed too (the card is
  reachable through Gamebit); an updated ROM under the same name keeps them. Not by hash
  (an update changes it) nor by header ID (not every system has one, and a hack shares
  its base game's).
- **Controls per game now**, not later: in a game, Screen, Controls and Shortcuts all edit
  the game, and Y puts a row back to the system's. The shelf's pages set the systems'
  defaults.
- **A new in-game Shortcuts page**, a fourth row of the in-game Settings (Screen,
  Gameplay, Controls, Shortcuts): the general shortcuts and the playing system's, as this
  game's keys.
- Build order, each checked on the device before the next: the per-game file and the
  Screen overrides; then Shortcuts; then Controls; then the shelf's system first.

## Step 1 as built

- `slot_store::games`: `Screen` (one system's screen settings, read and set on
  `SlotState`), `ScreenOverrides` (a game's own values, `None` follows the system) and the
  game's file, which keeps lines a later build writes (shortcuts, controls) as they were.
- The app loads the game's file when its core starts (`load_game`, from the session, and
  `follow_game` for a game already in the slot), and builds `game_view`: `slot.state` with
  the game's values over its system's. Every screen value is read through `view()`, which
  is that view while the game is in the slot and `slot.state` otherwise, so the shelf's
  pages show the systems' values.
- A change in a game runs through `edit_game`: the usual code changes `state` while it
  holds the game's view, and what changed on the game's system goes to the game's file;
  `slot.state` isn't written. That covers the in-game Screen rows (Shader, Color
  Correction, Palette, Bezel, Scale, Stretch, Frame Blending) and the Shader, Palette,
  Colour Correction and Stretch shortcuts.
- A value set back to the system's follows the system again (and leaves the file), so the
  file holds only what differs. Y on a row with the game's own value does the same; the
  footer offers Y (Default) only there.
- The game's own value is drawn in `GAME_INK` (amber, `slot_ui::quick_menu`), a multiply
  over its pale face (`Draw::Tinted`). The user checked it on the device on 2026-11-06 and
  kept it; the shelf's system first takes the same ink.
- A row whose footer spreads three legends (Back, Default, Change) shows the full slot
  bar behind them, as binding rows do (`spreads_legends`).
## Step 2 as built: shortcuts and Gameplay

- **Gameplay** (asked for with step 2): Fast Forward, its sound, Rewind, Rumble, Load
  State on Insert and Save State on Eject take a game's own value over the shelf's one
  value (`PlayOverrides`), with the same view, Y and amber as Screen. The shelf's Game
  page sets the shelf's value. Nothing in Base² reads these (the OS only provides the
  vibration motor, which Slot² drives). Two are read outside the view: Load State on
  Insert from the inserted cart's file before it is in the slot (`loads_on_insert`), and
  Save State on Eject before eject clears the slot (power-off goes the same way).
- **Shortcuts:** a fourth in-game Settings row, Shortcuts, opens one page: the playing
  system's own shortcuts first, then the general ones (`QuickRow::PLAY_SHORTCUTS`, the
  page every shortcut row is on; the shelf shows General or one system's by hiding the
  rest). In a game, A sets the game's own key, X leaves it unbound in this game, Y puts
  it back to the shelf's; a key of its own is amber. The game listens for its own keys
  over the system's and the general ones (`App::shortcut_keys`), in `shortcut.<name>=`
  lines of its file (`none` for unbound in this game). A key set back to the shelf's
  follows the shelf again and leaves the file, as a Screen value does. A `shortcut.` line
  naming no shortcut or no key is dropped (the game follows the shelf's key) and leaves
  the file when it is next written.

- Left as they were: `video_mode.txt`, upstream's per-game stretch on 4:3 panels, where
  the stretch has no system value to follow; and `nds_view.txt`, the DS's screens, layout
  and stylus screen, already per game.

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
shelf, the game in the slot): the amber of a game's own values.

## What it needs in the code

- `slot-store`: the override file (read, write, remove one setting), and a resolved
  "settings for this game" view the session and the renderer read instead of `slot.state`
  directly.
- `slot`: the in-game rows write overrides; the shelf rows keep writing `slot.state`.
- `slot-ui`: a page whose order can change at run time (today a page is a fixed list:
  `QuickRow::page`, `shown`, `up`/`down`, `quick_window` all read it), and tinted faces for
  labels and values (a third face beside the dim and lit ones).

## Building it

1. The per-game file, and overrides for the Screen settings, with Y to clear and the tint
   on values.
2. The in-game Shortcuts page, with the game's own keys; and Gameplay per game.
3. Controls per game: the in-game Controls page edits the game.
4. The shelf's system first and tinted, on Screen and Controls > In-Game.

Each step is tested on the device before the next.
