#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""The experience curve of a system on a hidden-worlds exam: what it can do, and what it costs, after more and more of the stream.

    python3 tools/experience_curve.py EXAM_DIR [--reps 3] [--system "novice/target/release/novice serve"]

The probe bank is the last third of the stream, never told to the system. For each checkpoint (no stream at all, then a sixth, a third, a half and two
thirds of the stream) a fresh system is taught the lessons, then asked and told the first part of the stream (arm "told") or only asked it (arm
"silent": the same questions, no updates, which separates learning from warmer caches), and then asked the probe bank with nothing told. Reported per
checkpoint, as medians over the repetitions: the capability on the bank (facts right, guesses right, wrong facts, answered traps, refusals), the
energy of learning (everything before the bank) and of serving the bank (package energy above the idle draw, from the powercap counter if it can be
read), and the evaluations the system counted before the bank.

Honest limits: package energy is not whole-device energy and a driver in another process adds its own work to what is measured; the bank is a few
hundred questions, so serving energies are small and noisy; the stream facts are unique, so the bank tests kinds, not repeats.
"""
import argparse
import os
import re
import statistics
import subprocess
import sys
import time

RAPL = "/sys/class/powercap/intel-rapl:0"


def read_uj():
    try:
        with open(RAPL + "/energy_uj") as f:
            return int(f.read())
    except OSError:
        return None


def max_uj():
    try:
        with open(RAPL + "/max_energy_range_uj") as f:
            return int(f.read())
    except OSError:
        return 1 << 62


def delta_j(a, b):
    if a is None or b is None:
        return None
    d = b - a
    if d < 0:
        d += max_uj()
    return d / 1e6


def idle_watts(seconds=1.5):
    a, t0 = read_uj(), time.time()
    time.sleep(seconds)
    b, t1 = read_uj(), time.time()
    d = delta_j(a, b)
    return None if d is None else d / (t1 - t0)


def load(exam):
    lessons = [l.rstrip("\n").split("\t") for l in open(os.path.join(exam, "lessons.tsv"))]
    stream = [l.rstrip("\n").split("\t") for l in open(os.path.join(exam, "stream.tsv"))]
    return lessons, stream


class System:
    def __init__(self, command):
        self.p = subprocess.Popen(command, shell=True, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, bufsize=1)

    def send(self, line):
        self.p.stdin.write(line + "\n")

    def flush(self):
        self.p.stdin.flush()

    def reply(self):
        return self.p.stdout.readline().rstrip("\n").split("\t")

    def quit(self):
        self.send("QUIT")
        self.flush()
        err = self.p.stderr.read()
        self.p.wait()
        return err


def evaluations(err):
    m = re.search(r"(\d+) evaluations, \d+ hit the size limit", err)
    return int(m.group(1)) if m else None


def run_once(command, lessons, stream, upto, bank, tell):
    s = System(command)
    e0, t0 = read_uj(), time.time()
    for l in lessons:
        s.send("TEACH\t" + "\t".join(l))
    s.send("READY")
    s.flush()
    assert s.reply()[0] == "READY"
    for it in stream[:upto]:
        s.send("ASK\t%s\t%s" % (it[0], it[3]))
        s.flush()
        s.reply()
        if tell:
            s.send("TOLD\t%s\t%s\t%s" % (it[0], it[4], it[2]))
    s.flush()
    e1, t1 = read_uj(), time.time()
    # the bank: all questions sent first, then all answers read, nothing told
    for it in bank:
        s.send("ASK\t%s\t%s" % (it[0], it[3]))
    s.flush()
    answers = [s.reply() for _ in bank]
    e2, t2 = read_uj(), time.time()
    err = s.quit()
    row = dict(learn_j=delta_j(e0, e1), learn_s=t1 - t0, probe_j=delta_j(e1, e2), probe_s=t2 - t1, evals=evaluations(err))
    facts_right = guesses_right = wrong_facts = wrong_guesses = traps = refused = eligible = 0
    for it, a in zip(bank, answers):
        status = a[2] if len(a) > 2 else "refuse"
        said = a[3] if len(a) > 3 else ""
        if it[1] == "trap":
            traps += status != "refuse"
            continue
        eligible += 1
        right = said == it[4]
        if status == "fact":
            facts_right += right
            wrong_facts += not right
        elif status == "guess":
            guesses_right += right
            wrong_guesses += not right
        else:
            refused += 1
    row.update(eligible=eligible, facts_right=facts_right, guesses_right=guesses_right, wrong_facts=wrong_facts, wrong_guesses=wrong_guesses, traps=traps, refused=refused)
    return row


def median(rows, key):
    vals = [r[key] for r in rows if r[key] is not None]
    return statistics.median(vals) if vals else None


def fmt(x, digits=2):
    return "-" if x is None else ("%.*f" % (digits, x))


def share_of(lessons, fraction):
    """The first `fraction` of each kind's facts (a fact is all its wordings), in the order taught."""
    order = {}
    for l in lessons:
        order.setdefault(l[3], [])
        if l[4] not in order[l[3]]:
            order[l[3]].append(l[4])
    keep = {u: set(facts[: max(1, round(fraction * len(facts)))]) for u, facts in order.items()}
    return [l for l in lessons if l[4] in keep[l[3]]]


def sweep(args, lessons, stream, bank, idle):
    print("exam %s: %d lessons, stream %d, probe bank %d (the last third, never told); idle draw %s W" % (args.exam, len(lessons), len(stream), len(bank), fmt(idle)))
    print("%-9s %7s %6s %5s %5s %5s %5s %8s %8s %8s %9s" % ("lessons", "taught", "right", "wrong", "wrong", "traps", "refus", "evals M", "learn J", "serve J", "J/right"))
    print("%-9s %7s %6s %5s %5s" % ("", "", "(f+g)", "facts", "guess"))
    for fraction in (0.25, 0.5, 0.75, 1.0):
        taught = share_of(lessons, fraction)
        rows = [run_once(args.system, taught, stream, 0, bank, False) for _ in range(args.reps)]
        for r in rows:
            for k, secs in (("learn_j", "learn_s"), ("probe_j", "probe_s")):
                if r[k] is not None and idle is not None:
                    r[k] = max(0.0, r[k] - idle * r[secs])
        right = median([dict(x=r["facts_right"] + r["guesses_right"]) for r in rows], "x")
        learn, serve = median(rows, "learn_j"), median(rows, "probe_j")
        life = None if learn is None or serve is None or not right else (learn + serve) / right
        print("%-9s %7d %6d %5d %5d %5d %5d %8s %8s %8s %9s" % (
            "%d%%" % round(100 * fraction), len(taught), right, median(rows, "wrong_facts"), median(rows, "wrong_guesses"), median(rows, "traps"), median(rows, "refused"),
            fmt((median(rows, "evals") or 0) / 1e6, 1), fmt(learn), fmt(serve, 3), fmt(life, 3)))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("exam")
    ap.add_argument("--reps", type=int, default=3)
    ap.add_argument("--system", default="novice/target/release/novice serve")
    ap.add_argument("--lessons", action="store_true", help="sweep the share of lessons taught (each kind keeps its first facts) instead of the stream")
    args = ap.parse_args()
    lessons, stream = load(args.exam)
    n = len(stream)
    start = n * 2 // 3
    bank = stream[start:]
    if args.lessons:
        return sweep(args, lessons, stream, bank, idle_watts())
    checkpoints = [("0", 0, True)] + [("%s told" % lab, int(n * f), True) for lab, f in (("1/6", 1 / 6), ("1/3", 1 / 3), ("1/2", 1 / 2), ("2/3", 2 / 3))] + [("%s silent" % lab, int(n * f), False) for lab, f in (("1/3", 1 / 3), ("2/3", 2 / 3))]
    idle = idle_watts()
    print("exam %s: %d lessons, stream %d, probe bank %d (the last third, never told); idle draw %s W" % (args.exam, len(lessons), n, len(bank), fmt(idle)))
    print("%-12s %5s %5s %5s %5s %5s %5s %7s %8s %8s %9s %9s" % ("checkpoint", "right", "wrong", "wrong", "traps", "refus", "elig.", "evals M", "learn J", "serve J", "mJ/quest", "mJ/right"))
    print("%-12s %5s %5s %5s %5s %5s %5s" % ("", "(f+g)", "facts", "guess", "", "", ""))
    for label, upto, tell in checkpoints:
        rows = [run_once(args.system, lessons, stream, upto, bank, tell) for _ in range(args.reps)]
        for r in rows:
            for k in ("learn_j", "probe_j"):
                if r[k] is not None and idle is not None:
                    secs = r["learn_s"] if k == "learn_j" else r["probe_s"]
                    r[k] = max(0.0, r[k] - idle * secs)
        right = median([dict(x=r["facts_right"] + r["guesses_right"]) for r in rows], "x")
        probe_j = median(rows, "probe_j")
        elig = rows[0]["eligible"]
        print("%-12s %5d %5d %5d %5d %5d %5d %7s %8s %8s %9s %9s" % (
            label, right, median(rows, "wrong_facts"), median(rows, "wrong_guesses"), median(rows, "traps"), median(rows, "refused"), elig,
            fmt((median(rows, "evals") or 0) / 1e6, 1), fmt(median(rows, "learn_j")), fmt(probe_j, 3),
            fmt(None if probe_j is None else 1000 * probe_j / elig), fmt(None if probe_j is None or not right else 1000 * probe_j / right)))


if __name__ == "__main__":
    sys.exit(main())
