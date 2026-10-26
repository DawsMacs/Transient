#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""A second set of exam worlds whose kinds need an operation outside {+, -, *, min, max, div, mod}.

The same files, line protocol and shape of world as `tools/hidden_worlds.py` (so `baby exam adapt-drive DIR COMMAND...` judges any system on it), with
different kinds, drawn from different families: number theory, sequences by recurrence or by a loop with a state, digits in base 3, combinatorial counts
and order statistics over derived values.

    python3 tools/hidden_worlds_b.py DIR [--seed N] [--compose-first]

Roles, as the judge groups them (the stream's role column):
  newop          needs an operation outside the starting set (11 kinds)
  compose        a small composition on top of those operations, taught after them (5 kinds)
  control        expressible with the starting operations, so a regression shows (4 kinds)
  unexplainable  an arbitrary code for each number: no rule, must stay refused until told (3 kinds)

The words and a few constants (the moduli, the affine control, the codes) are drawn from the seed. Standard library only.
"""
import argparse
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hidden_worlds import BEYOND_BINARY, BEYOND_UNARY, HELD_BINARY, HELD_UNARY, LIMIT, TAUGHT_BINARY, Kind, Rng, phrasings, points  # noqa: E402

README = """THE HIDDEN WORLDS B (adaptive exam; the same files and protocol as `baby exam adapt-write`, written by tools/hidden_worlds_b.py)

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

ENDING = "velu"  # shared by every word of this set (the first set's words end in another syllable)

# ---- the operations the starting set lacks ----


def is_prime(a):
    return a >= 2 and all(a % d for d in range(2, math.isqrt(a) + 1))


def prime_factors(a):
    """The distinct prime factors of a, ascending."""
    out, d = [], 2
    while d * d <= a:
        if a % d == 0:
            out.append(d)
            while a % d == 0:
                a //= d
        d += 1
    if a > 1:
        out.append(a)
    return out


def totient(a):
    return sum(1 for i in range(1, a + 1) if math.gcd(i, a) == 1)


def next_prime(a):
    a += 1
    while not is_prime(a):
        a += 1
    return a


def fib_mod(a, m):
    x, y = 0, 1
    for _ in range(a):
        x, y = y, (x + y) % m
    return x % m


def collatz_steps(a):
    n = 0
    while a != 1:
        a = a // 2 if a % 2 == 0 else 3 * a + 1
        n += 1
    return n


def partitions(a):
    """Number of ways to write a as a sum of positive integers, order ignored."""
    ways = [1] + [0] * a
    for part in range(1, a + 1):
        for s in range(part, a + 1):
            ways[s] += ways[s - part]
    return ways[a]


def catalan_mod(a, m):
    return math.comb(2 * a, a) // (a + 1) % m


def digit_sum3(a):
    s = 0
    while a:
        s, a = s + a % 3, a // 3
    return s


def reverse3(a):
    r = 0
    while a:
        r, a = r * 3 + a % 3, a // 3
    return r


def josephus(n, k):
    """n people in a circle, every k-th one leaves: the place (1 to n) of the last one standing."""
    j = 0
    for i in range(2, n + 1):
        j = (j + k) % i
    return j + 1


def totient_rank(a, b):
    """The b-th smallest (b from 1) of the totients of 1 to a."""
    return sorted(totient(i) for i in range(1, a + 1))[b - 1]


# ---- the kinds ----


def code_kind(rng, name, bound):
    """An arbitrary code for each number 0..31 (no rule)."""
    codes = [rng.below(bound) for _ in range(LIMIT + 1)]
    return Kind(name, "unexplainable", 1, lambda a, _b, c=codes: c[a], "an arbitrary code, no rule: " + " ".join(map(str, codes)))


def make_kinds(seed):
    """The kinds of a world, in four groups: control, newop, compose, unexplainable (each group shuffled by the seed)."""
    rng = Rng(seed)
    fib_m = [8, 9, 10, 12, 13][rng.below(5)]
    cat_m = [9, 10, 11, 12, 13][rng.below(5)]
    bin_m = [7, 10, 11, 12][rng.below(4)]
    wrap_m = 7 + rng.below(7)
    mult = next(m for m in (3 + rng.below(7) + i for i in range(wrap_m)) if math.gcd(m, wrap_m) == 1)  # a multiplier that visits every residue
    shift = 1 + rng.below(8)
    newop = [
        Kind("totient", "newop", 1, lambda a, _b: totient(a), "Euler's totient: how many of 1 to a share no factor with a", lo=1),
        Kind("omega", "newop", 1, lambda a, _b: len(prime_factors(a)), "number of distinct prime factors of a", lo=1),
        Kind("nextprime", "newop", 1, lambda a, _b: next_prime(a), "the smallest prime greater than a"),
        Kind("fibmod", "newop", 1, lambda a, _b: fib_mod(a, fib_m), f"Fibonacci number a (0, 1, 1, 2, 3, ...) mod {fib_m}"),
        Kind("collatz", "newop", 1, lambda a, _b: collatz_steps(a), "steps to reach 1 from a: halve if even, else 3a + 1", lo=1),
        Kind("partitions", "newop", 1, lambda a, _b: partitions(a), "number of ways to write a as a sum of positive integers, order ignored"),
        Kind("catalan", "newop", 1, lambda a, _b: catalan_mod(a, cat_m), f"Catalan number a (1, 1, 2, 5, 14, ...) mod {cat_m}"),
        Kind("digsum3", "newop", 1, lambda a, _b: digit_sum3(a), "sum of the digits of a written in base 3"),
        Kind("rev3", "newop", 1, lambda a, _b: reverse3(a), "the digits of a in base 3, read backwards (no leading zeros kept), as a number"),
        Kind("binom", "newop", 2, lambda a, b: math.comb(a, b) % bin_m, f"a choose b mod {bin_m} (asked only with a at least b)", ordered=True),
        Kind("josephus", "newop", 2, lambda a, b: josephus(a, b), "a people in a circle, every b-th leaves: place of the last one (1 to a)", lo=1),
    ]
    compose = [
        Kind("totsum", "compose", 1, lambda a, _b: sum(totient(i) for i in range(1, a + 1)), "totient(1) + ... + totient(a)", lo=1),
        Kind("primegap", "compose", 1, lambda a, _b: next_prime(a) - a, "nextprime(a) - a"),
        Kind("totrank", "compose", 2, lambda a, b: totient_rank(a, b), "the b-th smallest of totient(1), ..., totient(a) (b from 1, at most a)", lo=1, ordered=True),
        Kind("fibcollatz", "compose", 1, lambda a, _b: fib_mod(collatz_steps(a), fib_m), f"fibmod(collatz(a)): Fibonacci number (steps from a) mod {fib_m}", lo=1),
        Kind("digdiff3", "compose", 2, lambda a, b: abs(digit_sum3(a) - digit_sum3(b)), "|digsum3(a) - digsum3(b)|"),
    ]
    control = [
        Kind("mean", "control", 2, lambda a, b: (a + b) // 2, "(a + b) div 2"),
        Kind("spread", "control", 2, lambda a, b: max(a, b) - min(a, b), "max(a, b) - min(a, b)"),
        Kind("triangle", "control", 1, lambda a, _b: a * (a + 1) // 2, "a * (a + 1) div 2"),
        Kind("wrap", "control", 1, lambda a, _b: (mult * a + shift) % wrap_m, f"({mult} * a + {shift}) mod {wrap_m}"),
    ]
    unexplainable = [code_kind(rng, "code10", 10), code_kind(rng, "code100", 100), code_kind(rng, "code1000", 1000)]
    return rng.shuffled(control), rng.shuffled(newop), rng.shuffled(compose), unexplainable


# ---- the words ----

CONSONANTS = "bdfgklmnprstvzh"
VOWELS = "aeiou"
LASTS = ["", "", "m", "n", "k", "t", "z"]


def invent_words(rng, n):
    """Nonsense of two syllables (consonant, vowel, consonant, vowel, maybe a last consonant) plus the shared ending."""
    words = []
    while len(words) < n:
        w = "".join(
            [
                CONSONANTS[rng.below(len(CONSONANTS))],
                VOWELS[rng.below(5)],
                CONSONANTS[rng.below(len(CONSONANTS))],
                VOWELS[rng.below(5)],
                LASTS[rng.below(len(LASTS))],
                ENDING,
            ]
        )
        if w not in words:
            words.append(w)
    return words


# ---- the exam ----


def build(seed, compose_first=False):
    """The whole exam as (lessons, stream, key): lists of rows of strings."""
    control, newop, compose, unexplainable = make_kinds(seed)
    order = control + (compose + newop if compose_first else newop + compose) + unexplainable
    for k, w in zip(order, invent_words(Rng(seed ^ 0xB0B5), len(order))):
        k.word = w
    rng = Rng(seed ^ 0xADA98)
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
    p = argparse.ArgumentParser(description="Write a second hidden-worlds adaptive exam (lessons.tsv, stream.tsv) for `baby exam adapt-drive`.")
    p.add_argument("dir")
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--compose-first", action="store_true", help="teach the compositions before the operations they build on (a control: reuse should then not help)")
    a = p.parse_args(argv)
    nl, ns = write(a.dir, a.seed, a.compose_first)
    print(f"wrote the hidden worlds B (seed {a.seed}) to {a.dir}: {nl} lessons and a stream of {ns} questions")


if __name__ == "__main__":
    main(sys.argv[1:])
