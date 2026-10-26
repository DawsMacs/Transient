#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""Exam worlds whose kinds need an operation outside {+, -, *, min, max, div, mod}.

A seed in, the same files out: `lessons.tsv` and `stream.tsv` in exactly the format `baby exam adapt-write` writes for the clean adaptive exam (so
`baby exam adapt-drive DIR COMMAND...` can judge any system on it), plus `README.txt` (the line protocol, as in the baseline) and `key.tsv` (which
word is which kind; for the people reading the results, never sent to a system).

    python3 tools/hidden_worlds.py DIR [--seed N] [--compose-first]

Every question is numbers plus an invented word that names the kind (`what is 13 zorbosido 6`, `what is xarkelsido 21`). Each fact is taught in three
phrasings, a fourth rewording is only asked. Inputs are 0 to 31: the lessons teach a subset of the small numbers (some small facts are held out), and
the stream asks the held-out facts and numbers above anything taught, telling the truth after each answer.

Roles, as the judge groups them (the stream's role column):
  newop          needs an operation outside the starting set (12 kinds)
  compose        a small composition on top of those operations, taught after them (6 kinds)
  control        expressible with the starting operations, so a regression shows (4 kinds)
  unexplainable  an arbitrary code for each number: no rule, must stay refused until told (3 kinds)

The words, the tables and a few constants (the lookup table, the ceiling's divisor, the offset of one composition) are drawn from the seed, so a
system cannot be tuned to one world. Standard library only.
"""
import argparse
import math
import os
import sys

MASK = (1 << 64) - 1

README = """THE HIDDEN WORLDS (adaptive exam; the same files and protocol as `baby exam adapt-write`, written by tools/hidden_worlds.py)

lessons.tsv   id TAB question TAB answer TAB unit TAB fact      what is taught first
stream.tsv    id TAB role TAB unit TAB question TAB truth       the questions, in the order they are put (the truth is NOT shown to the system until after its answer)
              roles: newop, compose, control, unexplainable
key.tsv       unit TAB role TAB kind TAB rule                   what each invented word means (for the people reading the results; never sent to a system)

PROTOCOL (what `baby exam adapt-drive DIR COMMAND...` speaks to a system over its standard input/output; one tab-separated line per message, flush after each reply):
  -> TEACH id question answer unit fact      (once per lesson)
  -> READY                                    (all lessons sent)        <- READY
  -> ASK id question                          <- ANSWER id status answer confidence      (status fact | guess | refuse)
  -> TOLD id truth unit                       (after every answer: the teacher tells the true answer; no reply)
  -> QUIT
"""

# ---- a small deterministic generator (the standard library's may change between Python versions) ----


class Rng:
    def __init__(self, seed):
        self.s = (seed * 0x9E3779B97F4A7C15 + 0x1234567) & MASK

    def next(self):
        self.s = (self.s + 0x9E3779B97F4A7C15) & MASK
        z = self.s
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK
        return z ^ (z >> 31)

    def below(self, n):
        return self.next() % n

    def shuffle(self, xs):
        for i in range(len(xs) - 1, 0, -1):
            j = self.below(i + 1)
            xs[i], xs[j] = xs[j], xs[i]

    def shuffled(self, xs):
        xs = list(xs)
        self.shuffle(xs)
        return xs


# ---- the operations the starting set lacks ----


def popcount(a):
    return bin(a).count("1")




def divisors(a):
    return sum(1 for d in range(1, a + 1) if a % d == 0)


def next_pow2(a):
    return 1 if a <= 1 else 1 << (a - 1).bit_length()


def trailing_zeros(a):
    return (a & -a).bit_length() - 1


def lcm(a, b):
    return a * b // math.gcd(a, b)


# ---- the kinds ----

LIMIT = 31  # inputs are 0..31
NEAR_UNARY = 19  # unary facts up to here may be taught; the rest are beyond
NEAR_BINARY = 15  # binary facts with both operands up to here may be taught
HELD_UNARY, BEYOND_UNARY = 4, 10  # per kind: held-out small facts asked, beyond facts asked
HELD_BINARY, BEYOND_BINARY, TAUGHT_BINARY = 8, 14, 60


class Kind:
    def __init__(self, name, role, arity, fn, rule, lo=0, ordered=False):
        self.name, self.role, self.arity, self.fn, self.rule, self.lo, self.ordered = name, role, arity, fn, rule, lo, ordered
        self.word = ""


def code_kind(rng, name):
    """An arbitrary code for each number 0..31 (no rule)."""
    codes = [rng.below(100) for _ in range(LIMIT + 1)]
    return Kind(name, "unexplainable", 1, lambda a, _b, c=codes: c[a], "an arbitrary code, no rule: " + " ".join(map(str, codes)))


def lookup_kind(rng):
    """A small table indexed by the remainder: the same few values come round again and again."""
    period = 4 + rng.below(3)
    while True:
        t = [rng.below(10) for _ in range(period)]
        steps = {t[i + 1] - t[i] for i in range(period - 1)}
        if len(set(t)) == period and len(steps) > 1 and max(t) - min(t) >= 5 and t != sorted(t) and t != sorted(t, reverse=True):
            break
    return Kind("lookup", "newop", 1, lambda a, _b, t=t, p=period: t[a % p], f"table {t} indexed by a mod {period}")


def make_kinds(seed):
    """The kinds of a world, in four groups: control, newop, compose, unexplainable (each group shuffled by the seed)."""
    rng = Rng(seed)
    ceil_d = 3 + rng.below(3)
    offset = 1 + rng.below(3)
    newop = [
        Kind("popcount", "newop", 1, lambda a, _b: popcount(a), "number of 1 bits of a"),
        Kind("isqrt", "newop", 1, lambda a, _b: math.isqrt(a), "integer square root of a"),
        Kind("mirror", "newop", 1, lambda a, _b: int(format(a, "05b")[::-1], 2), "the 5 binary digits of a, read backwards"),
        Kind("divisors", "newop", 1, lambda a, _b: divisors(a), "number of divisors of a", lo=1),
        lookup_kind(rng),
        Kind("pow2up", "newop", 1, lambda a, _b: next_pow2(a), "smallest power of 2 that is at least a", lo=1),
        Kind("trailzeros", "newop", 1, lambda a, _b: trailing_zeros(a), "number of trailing 0 bits of a", lo=1),
        Kind("xor", "newop", 2, lambda a, b: a ^ b, "bitwise exclusive-or of a and b"),
        Kind("and", "newop", 2, lambda a, b: a & b, "bitwise and of a and b"),
        Kind("or", "newop", 2, lambda a, b: a | b, "bitwise or of a and b"),
        Kind("gcd", "newop", 2, lambda a, b: math.gcd(a, b), "greatest common divisor of a and b", lo=1),
        Kind("lastpow", "newop", 2, lambda a, b: pow(a, b, 10), "last decimal digit of a to the power b", lo=1),
    ]
    compose = [
        Kind("popplus", "compose", 1, lambda a, _b: popcount(a) + offset, f"popcount(a) + {offset}"),
        Kind("bitparity", "compose", 1, lambda a, _b: popcount(a) % 2, "popcount(a) mod 2"),
        Kind("hamming", "compose", 2, lambda a, b: popcount(a ^ b), "popcount(a xor b)"),
        Kind("gcdhalf", "compose", 2, lambda a, b: math.gcd(a, b) // 2, "gcd(a, b) div 2", lo=1),
        Kind("lcm", "compose", 2, lambda a, b: lcm(a, b), "lcm(a, b) = a * b div gcd(a, b)", lo=1),
        Kind("sqrtpop", "compose", 1, lambda a, _b: math.isqrt(a) + popcount(a), "isqrt(a) + popcount(a)"),
    ]
    control = [
        Kind("sum", "control", 2, lambda a, b: a + b, "a + b"),
        Kind("difference", "control", 2, lambda a, b: a - b, "a - b (asked only with a at least b)", ordered=True),
        Kind("square", "control", 1, lambda a, _b: a * a, "a * a"),
        Kind("ceiling", "control", 1, lambda a, _b, d=ceil_d: (a + d - 1) // d, f"ceiling of a / {ceil_d}"),
    ]
    unexplainable = [code_kind(rng, f"code{i}") for i in range(3)]
    return rng.shuffled(control), rng.shuffled(newop), rng.shuffled(compose), unexplainable


# ---- the words ----

CONSONANTS = "bdfgklmnprstvxz"
VOWELS = "aeiou"
CLUSTERS = ["rk", "lb", "rz", "nd", "sk", "rt", "mp", "lt", "xt", "gn", "br", "rd", "kn", "st", "nk", "rm", "lg", "pt"]
ENDINGS = ["", "", "l", "n", "r", "s", "x"]


def invent_words(rng, n):
    """Five- or six-letter nonsense (consonant, vowel, cluster, vowel, maybe a last consonant) plus the shared ending `sido`."""
    words = []
    while len(words) < n:
        w = (
            CONSONANTS[rng.below(len(CONSONANTS))]
            + VOWELS[rng.below(5)]
            + CLUSTERS[rng.below(len(CLUSTERS))]
            + VOWELS[rng.below(5)]
            + ENDINGS[rng.below(len(ENDINGS))]
            + "sido"
        )
        if w not in words:
            words.append(w)
    return words


# ---- the phrasings (three taught, a fourth only asked) ----


def phrasings(word, arity, a, b):
    if arity == 1:
        return [f"what is {word} {a}", f"{word} {a}", f"{a} {word}", f"what is {a} {word}"]
    return [f"what is {a} {word} {b}", f"{a} {word} {b}", f"{a} {word} {b} equals", f"what is {a} {word} {b} equals"]


# ---- the exam ----


def points(kind):
    """(points inside the taught range, points beyond it) of a kind, as argument tuples."""
    lo = kind.lo
    if kind.arity == 1:
        near = [(a,) for a in range(lo, NEAR_UNARY + 1)]
        far = [(a,) for a in range(NEAR_UNARY + 1, LIMIT + 1)]
    else:
        pairs = [(a, b) for a in range(lo, LIMIT + 1) for b in range(lo, LIMIT + 1) if not kind.ordered or a >= b]
        near = [p for p in pairs if p[0] <= NEAR_BINARY and p[1] <= NEAR_BINARY]
        far = [p for p in pairs if p[0] > NEAR_BINARY or p[1] > NEAR_BINARY]
    return near, far


def build(seed, compose_first=False):
    """The whole exam as (lessons, stream, key): lists of rows of strings."""
    control, newop, compose, unexplainable = make_kinds(seed)
    order = control + (compose + newop if compose_first else newop + compose) + unexplainable
    for k, w in zip(order, invent_words(Rng(seed ^ 0x57A1E), len(order))):
        k.word = w
    rng = Rng(seed ^ 0xADA97)
    lessons, items, key = [], [], []
    fact = 0
    for k in order:
        key.append([k.word, k.role, k.name, k.rule])
        near, far = (rng.shuffled(xs) for xs in points(k))
        if k.arity == 1:
            held, beyond, taught = near[:HELD_UNARY], far[:BEYOND_UNARY], near[HELD_UNARY:]
        else:
            held, beyond, taught = near[:HELD_BINARY], far[:BEYOND_BINARY], near[HELD_BINARY : HELD_BINARY + TAUGHT_BINARY]
        for p in taught:
            args = p + (0,) * (2 - len(p))
            for q in phrasings(k.word, k.arity, *args)[:3]:
                lessons.append([q, str(k.fn(*args)), k.word, str(fact)])
            fact += 1
        # a held-out fact is asked in any of the four phrasings, a beyond fact in the first
        for p, phrasing in [(p, rng.below(4)) for p in held] + [(p, 0) for p in beyond]:
            args = p + (0,) * (2 - len(p))
            items.append([k.role, k.word, phrasings(k.word, k.arity, *args)[phrasing], str(k.fn(*args))])
            fact += 1
    rng.shuffle(items)
    lessons = [[f"L{i:05d}"] + row for i, row in enumerate(lessons)]
    stream = [[f"S{i:05d}"] + row for i, row in enumerate(items)]
    return lessons, stream, key


def write(directory, seed, compose_first=False):
    lessons, stream, key = build(seed, compose_first)
    os.makedirs(directory, exist_ok=True)
    for name, rows in (("lessons.tsv", lessons), ("stream.tsv", stream), ("key.tsv", key)):
        with open(os.path.join(directory, name), "w", encoding="utf-8", newline="") as f:
            f.write("".join("\t".join(r) + "\n" for r in rows))
    with open(os.path.join(directory, "README.txt"), "w", encoding="utf-8", newline="") as f:
        f.write(README)
    return len(lessons), len(stream)


def main(argv):
    p = argparse.ArgumentParser(description="Write a hidden-worlds adaptive exam (lessons.tsv, stream.tsv) for `baby exam adapt-drive`.")
    p.add_argument("dir")
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--compose-first", action="store_true", help="teach the compositions before the operations they build on (a control: reuse should then not help)")
    a = p.parse_args(argv)
    nl, ns = write(a.dir, a.seed, a.compose_first)
    print(f"wrote the hidden worlds (seed {a.seed}) to {a.dir}: {nl} lessons and a stream of {ns} questions")


if __name__ == "__main__":
    main(sys.argv[1:])
