#!/usr/bin/env python3
"""Slot²: writes cores/options.json, every option each pinned core declares and the values
it takes, from the core's source at its pin (design/core-options.md). A test checks every
option Slot² sets against it, so a misspelt key or value fails before it reaches a core
that ignores it.

    python3 -I tools/slotsq/core_options.py

Run it again after a pin or a patch changes. Sources are fetched from GitHub (the pin's
archive) unless vendor/<core>-src.tar.gz holds them already.
"""

import io
import json
import re
import sys
import tarfile
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import cores  # noqa: E402  (pins(), the core map's own reading of taskfile.yml)

REPO = HERE.parents[1]
OUT = REPO / "cores" / "options.json"


def tokens(src):
    """C tokens enough for an options table: strings (adjacent ones joined), braces and
    words; comments and preprocessor lines skipped."""
    out, i, n, last_end = [], 0, len(src), -1
    while i < n:
        c = src[i]
        if src.startswith("/*", i):
            j = src.find("*/", i + 2)
            i = n if j < 0 else j + 2
            continue
        if src.startswith("//", i) or c == "#":
            j = src.find("\n", i)
            i = n if j < 0 else j
            continue
        if c == "'":
            # A char literal ('"' among them): skipped, so it isn't taken for a string.
            j = i + 1
            while j < n and src[j] != "'":
                j += 2 if src[j] == "\\" else 1
            i = j + 1
            continue
        if c == '"':
            j, buf = i + 1, ""
            while j < n and src[j] != '"':
                if src[j] == "\\":
                    buf += src[j : j + 2]
                    j += 2
                    continue
                buf += src[j]
                j += 1
            if out and out[-1][0] == "s" and src[last_end:i].strip() == "":
                out[-1] = ("s", out[-1][1] + buf)
            else:
                out.append(("s", buf))
            i = j + 1
            last_end = i
        elif c in "{}":
            out.append((c, None))
            i += 1
        elif c.isalpha() or c == "_":
            j = i
            while j < n and (src[j].isalnum() or src[j] == "_"):
                j += 1
            out.append(("w", src[i:j]))
            i = j
        else:
            i += 1
    return out


def libretro_options(src, whole=True):
    """A libretro_core_options.h's English table: {key: [values]}. `whole` is a whole
    header, which must have the table (a patch's added lines needn't)."""
    toks = tokens(src)
    start = next(
        (k for k, (t, v) in enumerate(toks) if t == "w" and v in ("option_defs_us", "option_v2_definitions_us")),
        None,
    )
    if start is None:
        if whole:
            raise SystemExit("no option_defs_us table: read the header and teach this script its name")
        start = 0
    opts, k = {}, start
    while k < len(toks):
        t, v = toks[k]
        if t == "w" and v in ("options_us", "option_defs_intl", "options_intl") and opts:
            break
        if t == "{" and k + 1 < len(toks) and toks[k + 1][0] == "s" and re.fullmatch(r"[a-z0-9]+_[a-z0-9_]+", toks[k + 1][1]):
            key, depth, m, values = toks[k + 1][1], 0, k, []
            while True:
                tt, vv = toks[m]
                if tt == "{":
                    depth += 1
                elif tt == "}":
                    depth -= 1
                    if depth == 0:
                        break
                elif depth == 3 and tt == "s" and toks[m - 1][0] == "{":
                    values.append(vv)  # the first string of a {value, label} pair
                m += 1
            opts[key] = values
            k = m + 1
            continue
        k += 1
    return opts


def variables(src):
    """retro_variable tables, as a plain libretro wrapper declares them: {"key", "Name; a|b"}."""
    found = {}
    for key, text in re.findall(r'\{\s*"([a-z0-9]+_[a-z0-9_]+)"\s*,\s*"([^"]*;[^"]*)"\s*\}', src):
        found[key] = [v.strip() for v in text.split(";", 1)[1].split("|")]
    return found


def source(core, repo, commit):
    cached = REPO / "vendor" / f"{core}-src.tar.gz"
    if cached.exists():
        data = cached.read_bytes()
    else:
        url = f"https://github.com/{repo}/archive/{commit}.tar.gz"
        req = urllib.request.Request(url, headers={"User-Agent": "slotsq-core-options"})
        with urllib.request.urlopen(req, timeout=120) as r:
            data = r.read()
    return tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")


def main():
    table = {}
    for core, var, repo, commit in cores.pins():
        with source(core, repo, commit) as tar:
            for member in tar.getmembers():
                name = member.name
                if not member.isfile():
                    continue
                text = None
                if name.endswith("libretro_core_options.h") and "intl" not in name:
                    text = tar.extractfile(member).read().decode("utf-8", "replace")
                    table.setdefault(core, {}).update(libretro_options(text))
        # Options Slot²'s own patches add (cores/<core>/*.patch).
        for patch in sorted((REPO / "cores" / core).glob("*.patch")):
            added = "\n".join(l[1:] for l in patch.read_text().splitlines() if l.startswith("+"))
            table.setdefault(core, {}).update(libretro_options(added, whole=False))
            table.setdefault(core, {}).update(variables(added))
    OUT.write_text(json.dumps(table, indent=1, sort_keys=True, ensure_ascii=False) + "\n")
    for core in sorted(table):
        print(f"{core:<11} {len(table[core])} options")
    return 0


if __name__ == "__main__":
    sys.exit(main())
