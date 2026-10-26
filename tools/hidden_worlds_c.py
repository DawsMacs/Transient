#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""A third set of exam worlds whose kinds need an operation outside {+, -, *, min, max, div, mod}.

The same files, line protocol and shape of world as `tools/hidden_worlds.py` and `tools/hidden_worlds_b.py` (so `baby exam adapt-drive DIR COMMAND...`
judges any system on it), with kinds that neither earlier set used: loops with a state over the digits of a number, a conditional sequence step,
sums of divisors and orders modulo a number, counts such as derangements and Stirling numbers, a recurrence of order three, and a two-heap game.

    python3 tools/hidden_worlds_c.py DIR [--seed N] [--compose-first]

Roles, as the judge groups them (the stream's role column):
  newop          needs an operation outside the starting set (11 kinds)
  compose        a small composition on top of those operations, taught after them (5 kinds)
  control        expressible with the starting operations, so a regression shows (4 kinds)
  unexplainable  an arbitrary code for each number: no rule, must stay refused until told (3 kinds)

The words and a few constants (the moduli, the step rule, the constants of the controls, the codes) are drawn from the seed. Standard library only.
"""
import argparse
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hidden_worlds import BEYOND_BINARY, BEYOND_UNARY, HELD_BINARY, HELD_UNARY, LIMIT, TAUGHT_BINARY, Kind, Rng, phrasings, points  # noqa: E402

README = """THE HIDDEN WORLDS C (adaptive exam; the same files and protocol as `baby exam adapt-write`, written by tools/hidden_worlds_c.py)

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

ENDING = "ketu"  # shared by every word of this set (the first set's words end in `sido`, the second set's in `velu`)

# ---- the operations the starting set lacks ----


def digits(a, base):
    """The digits of a in the given base, least significant first (0 is the single digit 0)."""
    out = []
    while True:
        out.append(a % base)
        a //= base
        if a == 0:
            return out


def longest_run(a):
    """The length of the longest run of equal bits in the binary form of a (a from 1)."""
    best = run = 0
    prev = -1
    for bit in digits(a, 2):
        run = run + 1 if bit == prev else 1
        prev = bit
        best = max(best, run)
    return best


def palindrome_base(a):
    """The smallest base (from 2) in which the digits of a read the same backwards."""
    b = 2
    while True:
        d = digits(a, b)
        if d == d[::-1]:
            return b
        b += 1


def common_digit5(a):
    """The digit that occurs most often among the base-5 digits of a (the smaller one on a tie)."""
    d = digits(a, 5)
    return max(range(5), key=lambda x: (d.count(x), -x))


def multiples_below(a):
    """The sum of the numbers below a that are divisible by 3 or by 5."""
    return sum(i for i in range(a) if i % 3 == 0 or i % 5 == 0)


def hail_steps(a, p, q):
    """Steps to reach 1 from a: halve if even, else divide by 3 if divisible by 3, else p * a + q."""
    n = 0
    while a != 1:
        a = a // 2 if a % 2 == 0 else (a // 3 if a % 3 == 0 else p * a + q)
        n += 1
    return n


def divisor_sum(a):
    return sum(d for d in range(1, a + 1) if a % d == 0)


def order_of_two(a):
    """The multiplicative order of 2 modulo 2a + 1 (a from 1): the least k >= 1 with 2^k = 1 there."""
    m = 2 * a + 1
    k, x = 1, 2 % m
    while x != 1:
        x, k = x * 2 % m, k + 1
    return k


def derangements_mod(a, m):
    """Permutations of a things with no fixed point, mod m: D(0) = 1, D(1) = 0, D(n) = (n - 1)(D(n - 1) + D(n - 2))."""
    x, y = 1, 0
    if a == 0:
        return 1 % m
    for n in range(2, a + 1):
        x, y = y, (n - 1) * (x + y)
    return y % m


def stirling2(a, b):
    """Ways to split a things into b non-empty groups, order of groups and of things ignored (exact)."""
    row = [1] + [0] * b  # row for n = 0
    for n in range(1, a + 1):
        row = [0] + [(k * row[k] + row[k - 1]) for k in range(1, b + 1)]
    return row[b]


def tribonacci_mod(a, m):
    """T(0) = 0, T(1) = 0, T(2) = 1, T(n) = T(n - 1) + T(n - 2) + T(n - 3), mod m."""
    x, y, z = 0, 0, 1
    for _ in range(a):
        x, y, z = y, z, (x + y + z) % m
    return x % m


_WYTHOFF = {}


def wythoff(a, b):
    """The Grundy value of two heaps, a move taking any positive number from one heap or the same number from both."""
    if not _WYTHOFF:
        n = LIMIT + 1
        for i in range(n):
            for j in range(n):
                seen = {_WYTHOFF[i - k, j] for k in range(1, i + 1)}
                seen |= {_WYTHOFF[i, j - k] for k in range(1, j + 1)}
                seen |= {_WYTHOFF[i - k, j - k] for k in range(1, min(i, j) + 1)}
                g = 0
                while g in seen:
                    g += 1
                _WYTHOFF[i, j] = g
    return _WYTHOFF[a, b]


# ---- the kinds ----


def code_kind(rng, name, bound):
    """An arbitrary code for each number 0..31 (no rule)."""
    codes = [rng.below(bound) for _ in range(LIMIT + 1)]
    return Kind(name, "unexplainable", 1, lambda a, _b, c=codes: c[a], "an arbitrary code, no rule: " + " ".join(map(str, codes)))


def make_kinds(seed):
    """The kinds of a world, in four groups: control, newop, compose, unexplainable (each group shuffled by the seed)."""
    rng = Rng(seed)
    hail_p, hail_q = [(1, 2), (1, 3), (1, 4), (2, 2), (2, 4)][rng.below(5)]
    der_m = [7, 9, 10, 11, 12, 13][rng.below(6)]
    stir_m = [7, 8, 9, 10, 11, 12][rng.below(6)]
    tri_m = [9, 10, 11, 12, 13, 14][rng.below(6)]
    clamp_lo, clamp_hi = 4 + rng.below(6), 18 + rng.below(8)
    fold, floor = 11 + rng.below(4), 3 + rng.below(4)
    mult, add = 3 + rng.below(5), 1 + rng.below(4)
    div = 5 + rng.below(5)
    while math.gcd(mult, div) != 1:  # coprime, so the scaling never cancels
        div += 1
    knee, slope = 8 + rng.below(6), 2 + rng.below(3)
    newop = [
        Kind("longrun", "newop", 1, lambda a, _b: longest_run(a), "the longest run of equal bits in the binary form of a", lo=1),
        Kind("palbase", "newop", 1, lambda a, _b: palindrome_base(a), "the smallest base from 2 in which the digits of a read the same backwards"),
        Kind("commondigit", "newop", 1, lambda a, _b: common_digit5(a), "the digit that occurs most often among the base-5 digits of a (the smaller on a tie)"),
        Kind("multsum", "newop", 1, lambda a, _b: multiples_below(a), "the sum of the numbers below a that are divisible by 3 or by 5"),
        Kind("hail", "newop", 1, lambda a, _b: hail_steps(a, hail_p, hail_q), f"steps to reach 1 from a: halve if even, else divide by 3 if divisible by 3, else {hail_p} * a + {hail_q}", lo=1),
        Kind("sigma", "newop", 1, lambda a, _b: divisor_sum(a), "the sum of the divisors of a", lo=1),
        Kind("order2", "newop", 1, lambda a, _b: order_of_two(a), "the multiplicative order of 2 modulo 2a + 1: the least k from 1 with 2^k = 1 there", lo=1),
        Kind("derange", "newop", 1, lambda a, _b: derangements_mod(a, der_m), f"derangements of a things (1, 0, 1, 2, 9, 44, ...) mod {der_m}"),
        Kind("stirling", "newop", 2, lambda a, b: stirling2(a, b) % stir_m, f"Stirling number of the second kind: ways to split a things into b non-empty groups, mod {stir_m} (asked only with a at least b, b from 1)", lo=1, ordered=True),
        Kind("tribo", "newop", 1, lambda a, _b: tribonacci_mod(a, tri_m), f"tribonacci number a (0, 0, 1, 1, 2, 4, 7, 13, ...) mod {tri_m}"),
        Kind("wythoff", "newop", 2, lambda a, b: wythoff(a, b), "Grundy value of two heaps of a and b: a move takes any number from one heap, or the same number from both"),
    ]
    compose = [
        Kind("aliquot", "compose", 1, lambda a, _b: divisor_sum(a) - a, "sigma(a) - a: the sum of the divisors of a below a", lo=1),
        Kind("bellpart", "compose", 2, lambda a, b: sum(stirling2(a, k) for k in range(1, b + 1)) % stir_m, f"stirling(a, 1) + ... + stirling(a, b), with the same modulus {stir_m} (b from 1, at most a)", lo=1, ordered=True),
        Kind("wythdiag", "compose", 1, lambda a, _b: wythoff(a, a), "wythoff(a, a)"),
        Kind("multpal", "compose", 1, lambda a, _b: multiples_below(palindrome_base(a)), "multsum(palbase(a))"),
        Kind("triborder", "compose", 1, lambda a, _b: tribonacci_mod(order_of_two(a), tri_m), f"tribo(order2(a)) with the same modulus {tri_m}", lo=1),
    ]
    control = [
        Kind("clampshift", "control", 1, lambda a, _b: min(max(a, clamp_lo), clamp_hi) - clamp_lo, f"a clamped to {clamp_lo} to {clamp_hi}, minus {clamp_lo}"),
        Kind("scale", "control", 1, lambda a, _b: (mult * a + add) // div, f"({mult} * a + {add}) div {div}"),
        Kind("fold", "control", 1, lambda a, _b: max(a % fold, floor) - floor, f"max(a mod {fold}, {floor}) - {floor}"),
        Kind("bracket", "control", 1, lambda a, _b: min(a, knee) + slope * max(a - knee, 0), f"min(a, {knee}) + {slope} * max(a - {knee}, 0)"),
    ]
    unexplainable = [code_kind(rng, "noise8", 8), code_kind(rng, "noise60", 60), code_kind(rng, "noise700", 700)]
    return rng.shuffled(control), rng.shuffled(newop), rng.shuffled(compose), unexplainable


# ---- the words ----

CONSONANTS = "bcdghjlmnprstwy"
VOWELS = "aeiou"
LASTS = ["", "", "", "l", "r", "s", "n"]


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
    for k, w in zip(order, invent_words(Rng(seed ^ 0xC0DEC), len(order))):
        k.word = w
    rng = Rng(seed ^ 0xC0FFEE)
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
    p = argparse.ArgumentParser(description="Write a third hidden-worlds adaptive exam (lessons.tsv, stream.tsv) for `baby exam adapt-drive`.")
    p.add_argument("dir")
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--compose-first", action="store_true", help="teach the compositions before the operations they build on (a control: reuse should then not help)")
    a = p.parse_args(argv)
    nl, ns = write(a.dir, a.seed, a.compose_first)
    print(f"wrote the hidden worlds C (seed {a.seed}) to {a.dir}: {nl} lessons and a stream of {ns} questions")


if __name__ == "__main__":
    main(sys.argv[1:])
