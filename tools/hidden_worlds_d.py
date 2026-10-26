#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""A fourth set of exam worlds: rules that no line of the learner names, to be acquired from told answers.

The same files, line protocol and shape of world as `tools/hidden_worlds.py`, `tools/hidden_worlds_b.py` and `tools/hidden_worlds_c.py` (so
`baby exam adapt-drive DIR COMMAND...` judges any system on it), with kinds that none of the earlier sets used: a few whose rule is simple but whose
constants are large (a scaled floor, a quadratic wrap, a residue puzzle), numbers defined from earlier ones in ways the earlier sets did not use
(a term that indexes into the sequence itself, a step that adds a gcd, an iterated square), other ways to write a number, a count that takes a
loop, and conditionals on a property of the input that is not a threshold.

    python3 tools/hidden_worlds_d.py DIR [--seed N] [--compose-first]

Roles, as the judge groups them (the stream's role column):
  newop          needs more than the starting operations in a few nodes (11 kinds)
  compose        a small composition of two kinds of this set, taught after them (4 kinds)
  control        expressible with the starting operations, so a regression shows (4 kinds)
  unexplainable  an arbitrary code for each number: no rule, must stay refused until told (3 kinds)

The words and a few constants are drawn from the seed. Standard library only.
"""
import argparse
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hidden_worlds import BEYOND_BINARY, BEYOND_UNARY, HELD_BINARY, HELD_UNARY, LIMIT, TAUGHT_BINARY, Kind, Rng, phrasings, points  # noqa: E402

README = """THE HIDDEN WORLDS D (adaptive exam; the same files and protocol as `baby exam adapt-write`, written by tools/hidden_worlds_d.py)

lessons.tsv   id TAB question TAB answer TAB unit TAB fact      what is taught first
stream.tsv    id TAB role TAB unit TAB question TAB truth       the questions, in the order they are put (the truth is NOT shown to the system until after its answer)
              roles: newop, compose, control, unexplainable
key.tsv       unit TAB role TAB kind TAB rule TAB learn         what each invented word means and how it could be learned (for the people reading the results; never sent to a system)

PROTOCOL (what `baby exam adapt-drive DIR COMMAND...` speaks to a system over its standard input/output; one tab-separated line per message, flush after each reply):
  -> TEACH id question answer unit fact      (once per lesson)
  -> READY                                    (all lessons sent)        <- READY
  -> ASK id question                          <- ANSWER id status answer confidence      (status fact | guess | refuse)
  -> TOLD id truth unit                       (after every answer: the teacher tells the true answer; no reply)
  -> QUIT
"""

ENDING = "ruvo"  # shared by every word of this set (the earlier sets' words end in `sido`, `velu` and `ketu`)

# ---- the operations ----


def popcount(a):
    return bin(a).count("1")


def golomb(a):
    """Golomb's sequence: G(1) = 1, G(n) = 1 + G(n - G(G(n - 1)))."""
    g = [0, 1]
    for n in range(2, a + 1):
        g.append(1 + g[n - g[g[n - 1]]])
    return g[a]


def rowland(a, start):
    """Rowland's sequence: R(1) = start, R(n) = R(n - 1) + gcd(n, R(n - 1))."""
    r = start
    for n in range(2, a + 1):
        r += math.gcd(n, r)
    return r


def quad_iter(a, x0, c, m):
    """x(0) = x0, x(n + 1) = (x(n)^2 + c) mod m."""
    x = x0
    for _ in range(a):
        x = (x * x + c) % m
    return x


def zeckendorf_bits(a):
    """The Zeckendorf form of a (a sum of non-consecutive Fibonacci numbers 1, 2, 3, 5, 8, ...) written as binary digits, 1 for each Fibonacci number used."""
    fibs = [1, 2]
    while fibs[-1] <= a:
        fibs.append(fibs[-1] + fibs[-2])
    out = 0
    for i in range(len(fibs) - 1, -1, -1):
        if fibs[i] <= a:
            a -= fibs[i]
            out |= 1 << i
    return out


def euclid_steps(a, b):
    """Divisions that Euclid's algorithm makes on (a, b) before the remainder is 0 (b from 1)."""
    n = 0
    while b:
        a, b = b, a % b
        n += 1
    return n


def triangle_part(a):
    """If a is a triangular number k * (k + 1) / 2 the index k, else the distance from a to the largest triangular number below it."""
    k = 0
    while (k + 1) * (k + 2) // 2 <= a:
        k += 1
    t = k * (k + 1) // 2
    return k if t == a else a - t


def evil_branch(a, p):
    """3 * a + p if the number of 1 bits of a is even, else a div 2."""
    return 3 * a + p if popcount(a) % 2 == 0 else a // 2


def crt(a, b, p, q):
    """The number x from 0 to p * q - 1 with x mod p = a mod p and x mod q = b mod q (p and q coprime)."""
    for x in range(p * q):
        if x % p == a % p and x % q == b % q:
            return x
    raise ValueError("p and q are not coprime")


# ---- the kinds ----

SMALL = list(range(41)) + [50, 64, 100]  # the constants a short program over the starting operations may use as leaves (the same as in the earlier sets' checks)


def near_small_scaled_floor(table):
    """Whether (x * a + y) div z, (x * a) div z + y or ((a + y) * x) div z with x, y, z from SMALL gives the table for a = 0..31 (a 7-node program, so the kind would be no gap)."""
    for x in SMALL:
        for z in SMALL[1:]:
            for y in SMALL:
                if all((x * a + y) // z == table[a] for a in range(LIMIT + 1)):
                    return True
                if all((x * a) // z + y == table[a] for a in range(LIMIT + 1)):
                    return True
                if all(((a + y) * x) // z == table[a] for a in range(LIMIT + 1)):
                    return True
    return False


def kind(name, role, arity, fn, rule, learn, **kw):
    k = Kind(name, role, arity, fn, rule, **kw)
    k.learn = learn
    return k


def code_kind(rng, name, bound):
    """An arbitrary code for each number 0..31 (no rule)."""
    codes = [rng.below(bound) for _ in range(LIMIT + 1)]
    return kind(name, "unexplainable", 1, lambda a, _b, c=codes: c[a], "an arbitrary code, no rule: " + " ".join(map(str, codes)), "cannot be learned: stays refused until told")


def make_kinds(seed):
    """The kinds of a world, in four groups: control, newop, compose, unexplainable (each group shuffled by the seed)."""
    rng = Rng(seed)
    # a scaled floor with constants outside the usual small ones: slope m / d between 1.6 and 4.6
    while True:
        fm, fd, fc = 41 + rng.below(23), 11 + rng.below(27), 41 + rng.below(49)
        if fm not in (50, 64) and fc not in (50, 64) and math.gcd(fm, fd) == 1 and 1.6 <= fm / fd <= 4.6 and not near_small_scaled_floor([(fm * a + fc) // fd for a in range(LIMIT + 1)]):
            break
    mx_m, mx_n = 11 + rng.below(19), 11 + rng.below(19)
    while mx_n == mx_m:
        mx_n = 11 + rng.below(19)
    mx_c, mx_d = 41 + rng.below(39), 41 + rng.below(23)
    crt_p, crt_q = [(5, 8), (7, 9), (8, 9), (7, 10), (9, 10), (5, 12), (7, 12), (8, 11), (9, 11)][rng.below(9)]
    quad_k, quad_c, quad_p = 23 + rng.below(39), 9 + rng.below(80), [73, 79, 83, 89, 97, 101, 103, 107, 109, 113][rng.below(10)]
    row_start = [7, 8, 9, 11, 13][rng.below(5)]
    while True:  # an orbit that does not fall into a short cycle in the first 32 steps
        it_x0, it_c, it_m = 2 + rng.below(4), 1 + rng.below(9), 29 + rng.below(33)
        if len({quad_iter(a, it_x0, it_c, it_m) for a in range(LIMIT + 1)}) >= 14:
            break
    evil_p = 1 + rng.below(4)
    grow_d, small_p = 3 + rng.below(3), 5 + rng.below(5)

    def evil(a):
        return evil_branch(a, evil_p)

    newop = [
        kind("bigfloor", "newop", 1, lambda a, _b: (fm * a + fc) // fd, f"({fm} * a + {fc}) div {fd}", f"a floor of a line: the steps between neighbors are {fm // fd} or {fm // fd + 1}; fit the slope {fm}/{fd} and the offset from where the steps fall, or a three-constant program"),
        kind("mixfloor", "newop", 2, lambda a, b: (mx_m * a + mx_n * b + mx_c) // mx_d, f"({mx_m} * a + {mx_n} * b + {mx_c}) div {mx_d}", "a floor of a plane: fit the two slopes from the rows and columns and the offset, or an eleven-node program with large constants"),
        kind("residue", "newop", 2, lambda a, b: crt(a, b, crt_p, crt_q), f"the number x from 0 to {crt_p * crt_q - 1} with x mod {crt_p} = a mod {crt_p} and x mod {crt_q} = b mod {crt_q}", f"(u * a + v * b) mod {crt_p * crt_q} with u = 1 mod {crt_p} and 0 mod {crt_q}, v the other way: a nine-node program with large constants, or two periods read off a row and a column"),
        kind("quadwrap", "newop", 1, lambda a, _b: (quad_k * a * a + quad_c) % quad_p, f"({quad_k} * a * a + {quad_c}) mod {quad_p}", "second differences of a quadratic mod a prime: the answers are a quadratic in a taken mod p; a nine-node program with large constants"),
        kind("selfindex", "newop", 1, lambda a, _b: golomb(a), "Golomb's sequence: G(1) = 1, G(n) = 1 + G(n - G(G(n - 1))) (1, 2, 2, 3, 3, 4, 4, 4, ...)", "a recurrence whose term looks itself up at an index computed from earlier terms; it also equals the number of times n appears, so the table can be rebuilt from its own runs", lo=1),
        kind("gcdstep", "newop", 1, lambda a, _b: rowland(a, row_start), f"R(1) = {row_start}, R(n) = R(n - 1) + gcd(n, R(n - 1))", "a recurrence on the previous term and the index through a gcd; mostly it adds 1, so the table is a ramp with occasional jumps that each divide the index", lo=1),
        kind("squareorbit", "newop", 1, lambda a, _b: quad_iter(a, it_x0, it_c, it_m), f"x(0) = {it_x0}, x(n + 1) = (x(n)^2 + {it_c}) mod {it_m}; the answer is x(a)", "a first-order recurrence in the answers themselves: the answer for a is a fixed function of the answer for a - 1, which a table of consecutive answers shows as a function of the previous one"),
        kind("fibcode", "newop", 1, lambda a, _b: zeckendorf_bits(a), "the Zeckendorf form of a (non-consecutive Fibonacci numbers 1, 2, 3, 5, 8, ...) written as binary digits, one digit per Fibonacci number", "a greedy representation in a numeral system with Fibonacci weights; the answers jump where a new Fibonacci number enters"),
        kind("remainders", "newop", 2, lambda a, b: euclid_steps(a, b), "the number of divisions Euclid's algorithm makes on (a, b) before the remainder is 0 (a and b from 1)", "a loop on a pair: replace (a, b) by (b, a mod b) and count; a swap costs one step when a is below b", lo=1),
        kind("trianglepos", "newop", 1, lambda a, _b: triangle_part(a), "if a is a triangular number k * (k + 1) / 2 then k, else the distance from a to the largest triangular number below it", "a conditional on a property of the input that is not a threshold: the answer follows the position in the rows of a triangle, with the row index on a triangular number"),
        kind("evenbits", "newop", 1, lambda a, _b: evil(a), f"3 * a + {evil_p} if the number of 1 bits of a is even, else a div 2", "a conditional on the parity of the number of 1 bits; the two branches are each a few nodes, and the parity splits the numbers in a pattern that is not a threshold or a remainder"),
    ]
    compose = [
        kind("golombsum", "compose", 1, lambda a, _b: sum(golomb(i) for i in range(1, a + 1)), "selfindex(1) + ... + selfindex(a)", "a running total of an earlier kind: the difference of neighbors is the earlier kind", lo=1),
        kind("fibcodesteps", "compose", 1, lambda a, _b: euclid_steps(a, zeckendorf_bits(a)), "remainders(a, fibcode(a))", "an earlier two-number kind applied to a number and an earlier one-number kind of it", lo=1),
        kind("gcdexcess", "compose", 1, lambda a, _b: rowland(a, row_start) - a, "gcdstep(a) - a", "an earlier kind minus the input: the lead of the sequence over its index, which only grows at the jumps", lo=1),
        kind("evenbitstri", "compose", 1, lambda a, _b: evil(triangle_part(a)), "evenbits(trianglepos(a))", "one conditional kind fed with the answer of the other"),
    ]
    control = [
        kind("grow", "control", 1, lambda a, _b: a + a // grow_d, f"a + a div {grow_d}", "a few nodes over the starting operations"),
        kind("residual", "control", 1, lambda a, _b: a % small_p, f"a mod {small_p}", "a remainder: the answers repeat with the period"),
        kind("larger", "control", 2, lambda a, b: max(a, b), "max(a, b)", "one node over the starting operations"),
        kind("product", "control", 2, lambda a, b: a * b, "a * b", "one node over the starting operations"),
    ]
    unexplainable = [code_kind(rng, "noise6", 6), code_kind(rng, "noise45", 45), code_kind(rng, "noise900", 900)]
    return rng.shuffled(control), rng.shuffled(newop), rng.shuffled(compose), unexplainable


# ---- the words ----

CONSONANTS = "cdfhjkmnpstwyz"
VOWELS = "aeiou"
LASTS = ["", "", "", "k", "m", "t", "x"]


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
    for k, w in zip(order, invent_words(Rng(seed ^ 0xD0D0D), len(order))):
        k.word = w
    rng = Rng(seed ^ 0xD15C0)
    lessons, items, key = [], [], []
    fact = 0
    for k in order:
        key.append([k.word, k.role, k.name, k.rule, k.learn])
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
    p = argparse.ArgumentParser(description="Write a fourth hidden-worlds adaptive exam (lessons.tsv, stream.tsv) for `baby exam adapt-drive`.")
    p.add_argument("dir")
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--compose-first", action="store_true", help="teach the compositions before the kinds they use (a control: reuse should then not help)")
    a = p.parse_args(argv)
    nl, ns = write(a.dir, a.seed, a.compose_first)
    print(f"wrote the hidden worlds D (seed {a.seed}) to {a.dir}: {nl} lessons and a stream of {ns} questions")


if __name__ == "__main__":
    main(sys.argv[1:])
