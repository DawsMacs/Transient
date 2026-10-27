#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""A fifth set of exam worlds: rules that no line of the learner names, to be acquired from told answers.

The same files, line protocol and shape of world as `tools/hidden_worlds.py` and its three successors (so `baby exam adapt-drive DIR COMMAND...`
judges any system on it), with kinds that none of the earlier sets used: numbers defined from earlier ones in other ways (a nonlinear recurrence on
the two terms before, a term that looks back a fixed fraction of the way, a recursion on the binary digits of the index), loops over the digits of a
pair or of a number with a state (carries, a running parity, an edit distance, a cheapest path over a grid), an element of a set picked out by a
property, a hidden state read out digit by digit, a few rules that are simple but whose constants are large (a sum of floors of a constant over the
index), and a conditional on a property of a pair.

    python3 tools/hidden_worlds_e.py DIR [--seed N] [--compose-first]

Roles, as the judge groups them (the stream's role column):
  newop          needs more than the starting operations in a few nodes (11 kinds)
  compose        a small composition of kinds of this set, taught after them (4 kinds)
  control        expressible with the starting operations, so a regression shows (4 kinds)
  unexplainable  an arbitrary code for each number or pair: no rule, must stay refused until told (3 kinds)

The words and a few constants are drawn from the seed. Standard library only.
"""
import argparse
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hidden_worlds import BEYOND_BINARY, BEYOND_UNARY, HELD_BINARY, HELD_UNARY, LIMIT, TAUGHT_BINARY, Kind, Rng, phrasings, points  # noqa: E402

README = """THE HIDDEN WORLDS E (adaptive exam; the same files and protocol as `baby exam adapt-write`, written by tools/hidden_worlds_e.py)

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

ENDING = "zane"  # shared by every word of this set (the earlier sets' words end in `sido`, `velu`, `ketu` and `ruvo`)

# ---- the operations ----


def seeded(seed, salt):
    """A generator for a seed and a purpose. The first set's generator started from neighboring seeds is the same stream a few steps on, so the seed goes through one extra mix first."""
    return Rng(Rng(seed * 1000003 + salt).next())


def prod_rec(a, x0, x1, m):
    """x(0) = x0, x(1) = x1, x(n) = (x(n - 1) * x(n - 2) + n) mod m."""
    x = [x0, x1]
    for n in range(2, a + 1):
        x.append((x[-1] * x[-2] + n) % m)
    return x[a]


def look_back(a, k):
    """h(0) = 1, h(n) = h(n - 1) + h(n div k)."""
    h = [1]
    for n in range(1, a + 1):
        h.append(h[n - 1] + h[n // k])
    return h[a]


def stern(a, t):
    """s(0) = 0, s(1) = 1, s(2n) = s(n), s(2n + 1) = s(n) + s(n + 1) + t (Stern's diatomic sequence for t = 0)."""
    s = [0, 1]
    for n in range(2, a + 1):
        s.append(s[n // 2] if n % 2 == 0 else s[n // 2] + s[n // 2 + 1] + t)
    return s[a]


def smooth_member(n, primes):
    for p in primes:
        while n % p == 0:
            n //= p
    return n == 1


def nth_smooth(a, primes):
    """The a-th number (a from 1) whose prime factors all lie in `primes` (1 is the first)."""
    n = 0
    while a:
        n += 1
        if smooth_member(n, primes):
            a -= 1
    return n


def carries(a, b, base):
    """Carries made when a and b are added by hand in the given base."""
    n = carry = 0
    while a or b or carry:
        carry = 1 if a % base + b % base + carry >= base else 0
        n += carry
        a //= base
        b //= base
    return n


def inverse_gray(a):
    """The binary reflected Gray code read backwards: each digit of the answer is the parity of the digits of a from the top down to it."""
    g, s = a, a >> 1
    while s:
        g ^= s
        s >>= 1
    return g


def edit_distance(a, b):
    """Levenshtein distance between the binary forms of a and b (0 is the one digit 0)."""
    x, y = format(a, "b"), format(b, "b")
    row = list(range(len(y) + 1))
    for i, c in enumerate(x, 1):
        new = [i]
        for j, d in enumerate(y, 1):
            new.append(min(row[j] + 1, new[j - 1] + 1, row[j - 1] + (c != d)))
        row = new
    return row[-1]


def frac_digit(a, p, q):
    """The a-th digit (a from 1) after the point of the decimal expansion of p / q."""
    return 10 * (p * pow(10, a - 1, q) % q) // q


def floor_sum(a, n):
    """floor(n / 1) + floor(n / 2) + ... + floor(n / a)."""
    return sum(n // k for k in range(1, a + 1))


def coprime_branch(a, b, c):
    """a + b + c if a and b share no factor, else |a - b|."""
    return a + b + c if math.gcd(a, b) == 1 else abs(a - b)


def grid_path(a, b, u, v, m):
    """The cheapest total over the cells of a path from (0, 0) to (a, b) that steps right or up, cell (i, j) costing ((i + u) * (j + v)) mod m."""
    best = {}
    for i in range(a + 1):
        for j in range(b + 1):
            c = ((i + u) * (j + v)) % m
            prev = [best[p] for p in ((i - 1, j), (i, j - 1)) if p in best]
            best[(i, j)] = c + (min(prev) if prev else 0)
    return best[(a, b)]


# ---- the kinds ----

SMALL = list(range(41)) + [50, 64, 100]  # the constants a short program over the starting operations may use as leaves (the same as in the earlier sets' checks)
SMOOTH_SETS = [(2, 3, 5), (2, 3, 7), (2, 5, 7), (3, 5, 7)]
FULL_REPTEND = [61, 109, 113, 131, 149, 167, 179, 223]  # primes for which 10 has the longest possible period q - 1 (at least 60 digits before the expansion repeats)


def kind(name, role, arity, fn, rule, learn, **kw):
    k = Kind(name, role, arity, fn, rule, **kw)
    k.learn = learn
    return k


def code_kind(rng, name, bound, arity=1):
    """An arbitrary code for each number 0..31 (arity 1) or each pair of them (arity 2): no rule."""
    n = (LIMIT + 1) ** arity
    codes = [rng.below(bound) for _ in range(n)]
    if arity == 1:
        return kind(name, "unexplainable", 1, lambda a, _b, c=codes: c[a], "an arbitrary code, no rule: " + " ".join(map(str, codes)), "cannot be learned: stays refused until told")
    return kind(name, "unexplainable", 2, lambda a, b, c=codes: c[a * (LIMIT + 1) + b], f"an arbitrary code from 0 to {bound - 1} for each pair of numbers 0..31 (drawn from the seed), no rule", "cannot be learned: stays refused until told", lo=0)


def make_kinds(seed):
    """The kinds of a world, in four groups: control, newop, compose, unexplainable (each group shuffled by the seed)."""
    rng = seeded(seed, 1)
    while True:  # a nonlinear recurrence that does not fall into a short cycle in the first 32 steps
        pr_m, pr_0, pr_1 = 29 + rng.below(43), 2 + rng.below(6), 2 + rng.below(6)
        if len({prod_rec(a, pr_0, pr_1, pr_m) for a in range(LIMIT + 1)}) >= 12:
            break
    lb_k = 3 + rng.below(4)
    st_t = rng.below(3)
    sm = SMOOTH_SETS[rng.below(len(SMOOTH_SETS))]
    while True:
        fd_q, fd_p = FULL_REPTEND[rng.below(len(FULL_REPTEND))], 2 + rng.below(100)
        if fd_p < fd_q and len({frac_digit(a, fd_p, fd_q) for a in range(1, LIMIT + 1)}) >= 7:
            break
    fs_n = 151 + rng.below(800)
    cp_c = 1 + rng.below(3)
    gp_u, gp_v, gp_m = 1 + rng.below(3), 1 + rng.below(3), 5 + rng.below(3)
    sq_d = 3 + rng.below(4)
    low_c = 1 + rng.below(5)

    def smooth(a):
        return nth_smooth(a, sm)

    def stern_t(a):
        return stern(a, st_t)

    def prod(a):
        return prod_rec(a, pr_0, pr_1, pr_m)

    newop = [
        kind("prodrec", "newop", 1, lambda a, _b: prod(a), f"x(0) = {pr_0}, x(1) = {pr_1}, x(n) = (x(n - 1) * x(n - 2) + n) mod {pr_m}; the answer is x(a)", "a second-order recurrence on the answers and the index: each answer is a fixed function of the two answers before it and of a, which consecutive answers show"),
        kind("lookback", "newop", 1, lambda a, _b: look_back(a, lb_k), f"h(0) = 1, h(n) = h(n - 1) + h(n div {lb_k})", f"the answer for a is the one before it plus the answer at a div {lb_k}: the table rebuilds from itself; it climbs by 1 for the first {lb_k} numbers and then in steps that repeat the early answers"),
        kind("diatomic", "newop", 1, lambda a, _b: stern_t(a), f"s(0) = 0, s(1) = 1, s(2n) = s(n), s(2n + 1) = s(n) + s(n + 1) + {st_t}", "a recursion on the binary digits of the index: even places copy the answer at half, odd places add the two answers beside half"),
        kind("smoothrank", "newop", 1, lambda a, _b: smooth(a), f"the a-th number (a from 1) whose prime factors all lie in {set(sm)} (1 is the first)", "a member of a set picked out by a property: the answers increase, and the gaps between them mark which numbers belong; the property can be read off the members", lo=1),
        kind("carrycount", "newop", 2, lambda a, b: carries(a, b, 2), "the carries made when a and b are added by hand in binary", "a loop over the binary digits of a pair with a state (the carry); the count can also be read as digit sums: the numbers of 1 bits of a and b minus the number of 1 bits of a + b make the carries"),
        kind("graydecode", "newop", 1, lambda a, _b: inverse_gray(a), "the binary reflected Gray code read backwards: digit i of the answer is the parity of the digits of a from the top down to digit i", "a loop down the binary digits with a state (a running parity); each answer digit depends on all the digits above it"),
        kind("editbits", "newop", 2, lambda a, b: edit_distance(a, b), "the edit distance (insert, delete or change one digit) between the binary forms of a and b, with 0 written as the digit 0", "a table filled in over the digits of two strings: each cell is the least of three neighbors plus a cost; it is 0 on the diagonal and symmetric"),
        kind("tenthdigit", "newop", 1, lambda a, _b: frac_digit(a, fd_p, fd_q), f"the a-th digit (a from 1) after the point of the decimal expansion of {fd_p} / {fd_q}", f"a hidden state read out digit by digit: the remainder r goes to 10 * r mod {fd_q} and the digit is 10 * r div {fd_q}; the digits do not show the state, the fraction can be guessed from them", lo=1),
        kind("harmonicsum", "newop", 1, lambda a, _b: floor_sum(a, fs_n), f"the sum of floor({fs_n} / k) for k from 1 to a", f"a running total whose steps are floor({fs_n} / a): the differences of neighbors fall like a hyperbola, with a large constant"),
        kind("pathcost", "newop", 2, lambda a, b: grid_path(a, b, gp_u, gp_v, gp_m), f"the cheapest total over the cells of a path from (0, 0) to (a, b) stepping right or up, cell (i, j) costing ((i + {gp_u}) * (j + {gp_v})) mod {gp_m}", "a table over a grid filled from the corner: each cell is its own cost plus the cheaper of the cell to the left and the cell below"),
        kind("coprimesplit", "newop", 2, lambda a, b: coprime_branch(a, b, cp_c), f"a + b + {cp_c} if a and b share no factor above 1, else |a - b|", "a conditional on a property of the pair that is neither a threshold nor a remainder: the two branches are a few nodes each, and the pairs with a common factor are the ones with the smaller answers", lo=1),
    ]
    compose = [
        kind("smoothgap", "compose", 1, lambda a, _b: smooth(a + 1) - smooth(a), "smoothrank(a + 1) - smoothrank(a)", "the difference of neighbors of an earlier kind: the gap to the next member of the set", lo=1),
        kind("carrylook", "compose", 1, lambda a, _b: carries(a, look_back(a, lb_k), 2), "carrycount(a, lookback(a))", "an earlier two-number kind applied to a number and an earlier one-number kind of it"),
        kind("lookdiatomic", "compose", 1, lambda a, _b: look_back(stern_t(a), lb_k), "lookback(diatomic(a))", "one recursion fed with the answer of the other"),
        kind("coprimerec", "compose", 1, lambda a, _b: coprime_branch(a, prod(a), cp_c), "coprimesplit(a, prodrec(a))", "an earlier pair kind applied to a number and an earlier kind of it (prodrec may answer 0, which shares every factor with a)", lo=1),
    ]
    control = [
        kind("cube", "control", 1, lambda a, _b: a * a * a, "a * a * a", "a few nodes over the starting operations"),
        kind("quotient", "control", 2, lambda a, b: a // b, "a div b (b from 1)", "one node over the starting operations", lo=1),
        kind("quarter", "control", 1, lambda a, _b: a * a // sq_d, f"a * a div {sq_d}", "a few nodes over the starting operations"),
        kind("doubled", "control", 2, lambda a, b: 2 * min(a, b) + low_c, f"2 * min(a, b) + {low_c}", "a few nodes over the starting operations"),
    ]
    unexplainable = [code_kind(rng, "noise7", 7), code_kind(rng, "noise400", 400), code_kind(rng, "noisepair30", 30, arity=2)]
    return rng.shuffled(control), rng.shuffled(newop), rng.shuffled(compose), unexplainable


# ---- the words ----

CONSONANTS = "bdghklmnrstvxz"
VOWELS = "aeiou"
LASTS = ["", "", "", "l", "n", "r", "s"]


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
    for k, w in zip(order, invent_words(seeded(seed, 2), len(order))):
        k.word = w
    rng = seeded(seed, 3)
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
    p = argparse.ArgumentParser(description="Write a fifth hidden-worlds adaptive exam (lessons.tsv, stream.tsv) for `baby exam adapt-drive`.")
    p.add_argument("dir")
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--compose-first", action="store_true", help="teach the compositions before the kinds they use (a control: reuse should then not help)")
    a = p.parse_args(argv)
    nl, ns = write(a.dir, a.seed, a.compose_first)
    print(f"wrote the hidden worlds E (seed {a.seed}) to {a.dir}: {nl} lessons and a stream of {ns} questions")


if __name__ == "__main__":
    main(sys.argv[1:])
