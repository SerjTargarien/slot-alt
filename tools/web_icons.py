#!/usr/bin/env python3
"""Draw the web page's handheld icons for slotPlus's systems, flat, in the style of
Slot's own GB, GBC and GBA icons. No makers' names or marks.

    tools/web_icons.py [TAG ...]

Writes web/public/platforms/<TAG>.svg for WS, Lynx, GG, NGP, TG16 and PokeMini
(all of them unless tags are named).
"""

import sys
from pathlib import Path

OUT = Path(__file__).resolve().parents[1] / 'web' / 'public' / 'platforms'


def svg(w, h, body):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" '
            f'viewBox="0 0 {w} {h}">{body}</svg>\n')


def rr(x, y, w, h, r, fill, extra=''):
    return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" fill="{fill}"{extra}/>'


def circle(cx, cy, r, fill):
    return f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}"/>'


def dpad(cx, cy, s, fill):
    t = s / 3
    return (rr(cx - s / 2, cy - t / 2, s, t, t / 4, fill) + rr(cx - t / 2, cy - s / 2, t, s, t / 4, fill))


def ws():
    # The pearl WonderSwan: landscape, the screen left of centre, the X and Y pads on the
    # left, A and B on the right.
    b = rr(4, 4, 320, 180, 40, '#9a9a9e') + rr(10, 10, 308, 168, 36, '#f2f1ee')
    b += rr(78, 30, 172, 128, 10, '#2a2a2e') + rr(96, 46, 136, 96, 2, '#8a8f80')
    for cx, cy in [(42, 70), (42, 120)]:
        for dx, dy in [(0, -14), (0, 14), (-14, 0), (14, 0)]:
            b += circle(cx + dx, cy + dy, 7, '#bdbcb8')
    b += circle(282, 82, 13, '#bdbcb8') + circle(300, 112, 13, '#bdbcb8')
    return svg(328, 188, b)


def lynx():
    # The Lynx II: a wide dark body, the screen in the middle, the D-pad left, A and B
    # right.
    b = rr(4, 4, 340, 168, 54, '#4a4e52') + rr(10, 10, 328, 156, 48, '#5d6266')
    b += rr(100, 28, 148, 120, 10, '#1e2124') + rr(116, 44, 116, 86, 2, '#7d8a90')
    b += dpad(52, 88, 46, '#2e3134')
    b += circle(286, 72, 15, '#2e3134') + circle(302, 108, 15, '#2e3134')
    return svg(348, 176, b)


def gg():
    # The Game Gear: wide and black, a blue-rimmed screen, the D-pad left, 1 and 2 right.
    b = rr(4, 4, 340, 176, 58, '#1d1d20') + rr(10, 10, 328, 164, 52, '#2a2a2e')
    b += rr(104, 30, 140, 124, 12, '#111') + rr(120, 46, 108, 92, 2, '#6c7c8c')
    b += dpad(54, 92, 46, '#555')
    b += circle(288, 100, 15, '#6a6a70') + circle(308, 72, 15, '#6a6a70')
    return svg(348, 184, b)


def ngp():
    # The NeoGeo Pocket Color: rounded, the screen above the stick and two buttons.
    b = rr(4, 4, 200, 320, 52, '#9aa0a6') + rr(10, 10, 188, 308, 46, '#cfd3d8')
    b += rr(28, 30, 152, 140, 12, '#2a2a2e') + rr(44, 46, 120, 106, 2, '#8a9a8a')
    b += circle(64, 232, 26, '#555a60') + circle(64, 232, 12, '#3a3e42')
    b += circle(138, 252, 14, '#3a3e42') + circle(166, 222, 14, '#3a3e42')
    return svg(208, 328, b)


def tg16():
    # The TurboExpress: wide and black, the screen in the middle, the pad left, II and I
    # right.
    b = rr(4, 4, 340, 160, 26, '#202022') + rr(10, 10, 328, 148, 22, '#2e2e31')
    b += rr(98, 24, 152, 120, 8, '#151517') + rr(114, 38, 120, 92, 2, '#7a8288')
    b += dpad(52, 84, 44, '#505055')
    b += circle(282, 96, 15, '#d8d8d8') + circle(310, 72, 15, '#d8d8d8')
    return svg(348, 168, b)


def pokemini():
    # The Pokémon mini: a small, nearly round clear body, the screen at the top, the pad
    # bottom left, A and B bottom right, and C at the side.
    b = rr(4, 4, 240, 210, 70, '#9ab8d8', ' fill-opacity="0.9"')
    b += rr(12, 12, 224, 194, 64, '#d6e4f0')
    b += rr(64, 26, 120, 86, 10, '#3a4048') + rr(78, 38, 92, 62, 2, '#a8b090')
    b += dpad(70, 152, 42, '#3a72c0')
    b += circle(168, 166, 12, '#3a72c0') + circle(194, 142, 12, '#3a72c0')
    b += circle(214, 98, 7, '#3a72c0')
    return svg(248, 218, b)


ICONS = {'WS': ws, 'Lynx': lynx, 'GG': gg, 'NGP': ngp, 'TG16': tg16, 'PokeMini': pokemini}


def main():
    tags = sys.argv[1:] or list(ICONS)
    OUT.mkdir(parents=True, exist_ok=True)
    for tag in tags:
        (OUT / f'{tag}.svg').write_text(ICONS[tag]())
        print(f'web/public/platforms/{tag}.svg')


if __name__ == '__main__':
    main()
