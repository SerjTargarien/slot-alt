# WonderSwan orientation

Status: agreed with the user on 2026-11-06. Step 1 built; step 2 not yet.

## What the user asked for

Klonoa (tried 2026-11-06) can't be turned and doesn't turn by itself. A WonderSwan game
should start the way it is played, follow the game when it changes orientation, and be
turnable by hand.

## What there was (2026-11-06)

- **The core** (Beetle Cygne `4b01295`, upstream's newest) turns only by hand:
  `wswan_rotate_display` is Manual (SELECT toggles), Landscape or Portrait. Its reading of
  the cart header's orientation is commented out (`#if 0`). When it turns it tells the
  frontend (SET_ROTATION, and the picture's new shape).
- **The hardware's signal:** a game writes the LCD icon port (`0x15`): bit 1 lights the
  console's "vertical" icon, bit 2 "horizontal", to tell the player how to hold it. The
  emulator keeps the byte (`LCDIcons`, `mednafen/wswan/gfx.c`) and never reads it.
- **Slot²** read the header bit once (`slot_store::ws_vertical`). For a horizontal game it
  held SELECT back, so nothing could turn it (Klonoa's header says horizontal: footer byte
  6 is `0x04`). For a vertical game it passed SELECT to the core, which turned it, and the
  picture followed; but the buttons followed the header, not the turn.

## The plan

### Step 1: Slot² follows and sets the turn (built)

- The buttons follow the picture: drawn tall, the game has the upright map (the Y pad on
  the D-pad); drawn wide, the horizontal map, or the sideways one for a vertical game.
- A game starts in its own saved orientation, else the header's: Slot² sets the core's
  `wswan_rotate_display` to Portrait or Landscape when the game opens. SELECT never
  reaches the core (the WonderSwan has no SELECT).
- **Rotate**, a WonderSwan input on its controls page (SELECT by default), turns any game
  by hand. The orientation a game is left in is its own, saved in its file
  (`Config/games/ws/<file name>.txt`, `ws_orientation=portrait|landscape`).
- Code: `Input::Rotate` (a slot input, like the DS's), `Session::sync_ws` and `ws_layout`,
  `App::keep_ws_portrait`, and the core option set as the game opens (`spawn_core`).

### Step 2: following the game

- A small patch to Beetle Cygne exposes the LCD icon byte (a memory region of its own).
  The first core Slot² builds itself; slotPlus's build stays as the fallback, and the core
  map is updated.
- Slot² reads it each frame: when the game lights the vertical or the horizontal icon, the
  picture turns to match. Rotate still turns by hand in between.
