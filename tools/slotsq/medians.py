#!/usr/bin/env python3
"""In-game battery medians from runs saved by `dev.sh logs <name>`.

    python3 tools/slotsq/medians.py gambatte-E alsa panfrost

For each name it reads logs/slot-<name>.log and logs/power-<name>.log, takes the power
samples from 15 s after the last game launch until 25 s before the log ends (the in-game
menu and quitting), and prints the median draw, the median CPU clock, the GPU clocks seen
and the vblanks missed in the whole run. The battery percentage at the start is printed
too, since a fuller battery reads slightly differently.
"""

import re
import statistics
import sys
from collections import Counter
from pathlib import Path

LOGS = Path(__file__).resolve().parents[2] / "logs"
SETTLE_S = 15
QUIT_S = 25


def seconds(stamp):
    h, m, s = (int(x) for x in stamp.split(":"))
    return h * 3600 + m * 60 + s


def summary(name):
    slot = (LOGS / f"slot-{name}.log").read_text()
    power = (LOGS / f"power-{name}.log").read_text().splitlines()
    launches = re.findall(r"launch: first frame shown at ([0-9.]+) ms", slot)
    if not launches or len(power) < 3:
        return f"{name}: no game launch or no power samples"
    start = float(launches[-1]) / 1000
    t0 = seconds(power[0].split()[0])
    rows = []
    for line in power[1:]:
        p = line.split()
        # 12:09:42 battery 2486 mW 77% Discharging cpu 1608 MHz gpu 200 MHz
        rows.append((seconds(p[0]) - t0, int(p[2]), int(p[7]), int(p[10]), p[4]))
    end = rows[-1][0] - QUIT_S
    game = [r for r in rows if start + SETTLE_S <= r[0] <= end]
    if not game:
        return f"{name}: the game ran too briefly to measure"
    mw = statistics.median(r[1] for r in game)
    cpu = statistics.median(r[2] for r in game)
    gpu = dict(sorted(Counter(r[3] for r in game).items()))
    missed = sum(int(m) for m in re.findall(r"pace: 600 frames, (\d+) missed", slot))
    return (
        f"{name}: {len(game)} samples, median {mw / 1000:.2f} W, cpu {cpu:.0f} MHz, "
        f"gpu MHz {gpu}, {missed} vblanks missed in the run, battery {rows[0][4]} at start"
    )


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    for arg in sys.argv[1:]:
        print(summary(arg))
