#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""Summarize the lines `novice serve` prints on standard error when NOVICE_SHADOW is set (what a feasibility check for the recurrence search would
say, next to what the search then did).

    NOVICE_SHADOW=1 baby exam adapt-drive DIR "novice serve" 2>&1 >/dev/null | grep '^shadow' | python3 tools/shadow_report.py

For each candidate rule (the sound cost bound, the "no reduction" test, a soft feature of the table), how many of the calls that found a recurrence
it would have rejected (must be none to be safe) and how much of the recurrence work it would have saved on the calls that found nothing.
"""
import re
import sys

PATTERN = re.compile(r"shadow\s+(\S+)\s+rows\s+(\d+) found (\d) evaluations\s+(\d+) table_bits\s+([\d.]+) distinct\s+(\d+) reductions\s+(\d+) no_reduction (\d) cost_bound_rejects (\d) support ([\d.]+)")
rows = []
for line in sys.stdin:
    m = PATTERN.search(line)
    if m:
        key, n, found, ev, tb, dist, red, nr, cb, sup = m.groups()
        rows.append(dict(key=key, rows=int(n), found=int(found), ev=int(ev), dist=int(dist), nr=int(nr), cb=int(cb), sup=float(sup)))
pos = [r for r in rows if r["found"]]
neg = [r for r in rows if not r["found"]]
total = sum(r["ev"] for r in rows) or 1
print("%d recurrence searches, %d found a rule; %.1f M evaluations in all" % (len(rows), len(pos), total / 1e6))


def rule(name, rejects):
    false = sum(1 for r in pos if rejects(r))
    saved = sum(r["ev"] for r in neg if rejects(r))
    print("%-46s rejects %2d of %d rules found; would save %5.1f %% of the work (%d failing searches)" % (name, false, len(pos), 100 * saved / total, sum(1 for r in neg if rejects(r))))


rule("sound: cheapest reduction costs more than table", lambda r: r["cb"])
rule("sound: no reduction at all", lambda r: r["nr"])
for th in (0.5, 0.6, 0.7):
    rule("soft: support below %.1f" % th, lambda r, th=th: r["sup"] < th)
for th in (0.5, 0.6, 0.8):
    rule("soft: more than %.0f %% of answers distinct" % (100 * th), lambda r, th=th: r["dist"] / r["rows"] > th)
