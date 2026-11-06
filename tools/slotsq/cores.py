#!/usr/bin/env python3
"""Slot²: keeps the core map (cores/README.md) honest.

    python3 -I tools/slotsq/cores.py          check now, and show every core
    python3 -I tools/slotsq/cores.py daily    at most once a day, and only what changed

It checks that the map names every core pinned in taskfile.yml (`<NAME>_COMMIT`, with its
source in cores/<name>/build.sh), and asks GitHub how far each source has moved past its
pin: commits since, and the newest release. The last answer is kept in
logs/cores-check.json, so `daily` reports only what's new. GITHUB_TOKEN, if set, is used
for GitHub's API (60 requests an hour without one; a check takes about 30).
"""

import datetime
import json
import os
import re
import sys
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
STATE = REPO / "logs" / "cores-check.json"
MAP = REPO / "cores" / "README.md"


def pins():
    """Every pinned source: (core, variable, owner/repo, commit)."""
    taskfile = (REPO / "taskfile.yml").read_text()
    commits = dict(re.findall(r"^\s+([A-Z_]+_COMMIT):\s*([0-9a-f]{40})", taskfile, re.M))
    found = []
    for build in sorted(REPO.glob("cores/*/build.sh")):
        core = build.parent.name
        text = build.read_text()
        # A recipe names its source as `source=` (and DSperate its core as
        # `dsperate_source=`), from a github.com archive or tree URL.
        sources = re.findall(r'echo "(\w*source)=https://github\.com/([^/]+/[^/]+)/', text)
        wrapped = any(key != "source" for key, _ in sources)
        for key, url in sources:
            # A wrapped core's `source` is its wrapper (DSPERATE_LR_COMMIT).
            var = f"{core.upper()}{'_LR' if wrapped and key == 'source' else ''}_COMMIT"
            commit = commits.get(var)
            if commit:
                found.append((core, var, url, commit))
    return found


def github(path):
    req = urllib.request.Request(
        "https://api.github.com/" + path, headers={"User-Agent": "slotsq-cores-check"}
    )
    if os.environ.get("GITHUB_TOKEN"):
        req.add_header("Authorization", "Bearer " + os.environ["GITHUB_TOKEN"])
    with urllib.request.urlopen(req, timeout=20) as r:
        return json.load(r)


def upstream(repo, commit, branch):
    """How far `repo` is past `commit`, and its newest release. `branch` is its main
    branch if a check before learned it."""
    branch = branch or github(f"repos/{repo}")["default_branch"]
    ahead = github(f"repos/{repo}/compare/{commit}...{branch}")["ahead_by"]
    releases = github(f"repos/{repo}/releases?per_page=1")
    release = releases[0]["tag_name"] if releases else None
    return {"ahead": ahead, "release": release, "branch": branch}


def main():
    daily = sys.argv[1:] == ["daily"]
    today = datetime.date.today().isoformat()
    before = json.loads(STATE.read_text()) if STATE.exists() else {}
    if daily and before.get("checked") == today:
        return 0
    lines = []
    found = pins()
    text = MAP.read_text() if MAP.exists() else ""
    missing = [(c, v, s[:7]) for c, v, _, s in found if s[:7] not in text]
    for core, var, short in missing:
        lines.append(f"map: cores/README.md doesn't name {core}'s pin {var} = {short}")
    # A check that couldn't reach every source isn't today's: the next one tries again.
    now = {"checked": today, "sources": {}}
    failed = []
    for core, var, repo, commit in found:
        key = f"{repo}@{commit[:7]}"
        old = before.get("sources", {}).get(key)
        try:
            got = upstream(repo, commit, (old or {}).get("branch"))
        except Exception as e:  # offline, rate-limited: said once below; keep the old answer
            failed.append((repo, str(e)))
            now["checked"] = before.get("checked")
            if old:
                now["sources"][key] = old
            continue
        now["sources"][key] = got
        if daily and old == got:
            continue
        news = f"{got['ahead']} commit{'s' if got['ahead'] != 1 else ''} past the pin"
        if got["release"]:
            news += f", newest release {got['release']}"
        if old and old != got:
            news += f" (was {old['ahead']}, {old['release'] or 'no release'})"
        lines.append(f"{core:<11} {commit[:7]}  {repo:<36} {news}")
    if failed:
        reasons = sorted({why for _, why in failed})
        lines.append(
            f"{len(failed)} of {len(found)} sources could not be checked ({'; '.join(reasons)}); "
            "the next session tries again"
        )
    STATE.parent.mkdir(exist_ok=True)
    STATE.write_text(json.dumps(now, indent=1))
    if lines:
        print("cores check: " + ("what changed upstream" if daily else "every pinned core"))
        print("\n".join("  " + l for l in lines))
        print("  (cores/README.md is the map; update it with any pin or core change)")
    elif not daily:
        print("cores check: the map names every pin; no source checked")
    return 0


if __name__ == "__main__":
    sys.exit(main())
