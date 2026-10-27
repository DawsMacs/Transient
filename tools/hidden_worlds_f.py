#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""A sixth set of exam worlds: rules that no line of the learner names, numbers that are spelled out as words, questions that are worded in several ways.

The same files, line protocol and shape of world as `tools/hidden_worlds.py` and its four successors (so `baby exam adapt-drive DIR COMMAND...`
judges any system on it), with kinds that none of the earlier sets used. To keep the set a blind test the kinds are not described here.

    python3 tools/hidden_worlds_f.py DIR [--seed N] [--compose-first]

What is new in the way the exam is put:
  numerals   some kinds take and give their numbers as words. One family of words is a way of counting that its lessons show (a way of spelling a
             number as words, with lessons that tie the words to digits and to sums); the other family is a vocabulary of single words, and no lesson
             ties it to anything. The learner is never told that these words are numbers.
  wordings   every kind is taught in two or three ways of asking (two kinds in all eight); some kinds are then asked in a way that was not taught
             for them but was taught for the two kinds that are taught in every way.
  refusals   next to the kinds without a rule, rows with a taught word in a place it was not taught for, two kinds at once, a new word, the wrong
             number of numbers, and a bare number.

Roles, as the judge groups them (the stream's role column):
  newop          needs more than the starting operations in a few nodes (12 kinds)
  compose        a small composition of kinds of this set, taught after them (3 kinds)
  control        expressible with the starting operations, or a copy or a way of spelling, so a regression shows (10 kinds)
  unexplainable  an arbitrary code for each number, word or pair: no rule, must stay refused until told (4 kinds)
  trap           a question that looks like an ordinary one and has no taught answer: must be refused (38 rows, no lessons)

The words and a few constants are drawn from the seed. Standard library only.
"""
import argparse
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hidden_worlds import BEYOND_BINARY, BEYOND_UNARY, HELD_BINARY, HELD_UNARY, LIMIT, TAUGHT_BINARY, Kind, Rng, points  # noqa: E402

README = """THE HIDDEN WORLDS F (adaptive exam; the same files and protocol as `baby exam adapt-write`, written by tools/hidden_worlds_f.py)

lessons.tsv   id TAB question TAB answer TAB unit TAB fact      what is taught first
stream.tsv    id TAB role TAB unit TAB question TAB truth       the questions, in the order they are put (the truth is NOT shown to the system until after its answer)
              roles: newop, compose, control, unexplainable, trap
key.tsv       unit TAB role TAB kind TAB rule TAB learn         what each invented word means and how it could be learned (for the people reading the results; never sent to a system)

PROTOCOL (what `baby exam adapt-drive DIR COMMAND...` speaks to a system over its standard input/output; one tab-separated line per message, flush after each reply):
  -> TEACH id question answer unit fact      (once per lesson)
  -> READY                                    (all lessons sent)        <- READY
  -> ASK id question                          <- ANSWER id status answer confidence      (status fact | guess | refuse)
  -> TOLD id truth unit                       (after every answer: the teacher tells the true answer; no reply)
  -> QUIT
"""

ENDING = "yarn"  # shared by every kind word of this set (the earlier sets' words end in `sido`, `velu`, `ketu`, `ruvo` and `zane`)

# ---- the operations ----


def seeded(seed, salt):
    """A generator for a seed and a purpose. The first set's generator started from neighboring seeds is the same stream a few steps on, so the seed goes through one extra mix first."""
    return Rng(Rng(seed * 1000003 + salt).next())


def recaman(a, s0):
    """x(0) = s0, x(n) = x(n - 1) - n if that is above 0 and not yet in the sequence, else x(n - 1) + n."""
    x, seen = s0, {s0}
    for n in range(1, a + 1):
        y = x - n
        x = y if y > 0 and y not in seen else x + n
        seen.add(x)
    return x


def min_coins(a, coins):
    """The fewest coins that make a (the coins can be used any number of times; 1 is among them)."""
    best = [0] + [None] * a
    for v in range(1, a + 1):
        best[v] = 1 + min(best[v - c] for c in coins if c <= v)
    return best[a]


_LEAPERS = {}


def leaper_table(p, q, margin=10):
    """Distances from (0, 0) on a board without edges for a piece that moves p squares one way and q the other, as a dict over the window -margin..31+margin."""
    if (p, q, margin) not in _LEAPERS:
        moves = {(s * x, t * y) for x, y in ((p, q), (q, p)) for s in (1, -1) for t in (1, -1)}
        dist, frontier = {(0, 0): 0}, [(0, 0)]
        while frontier:
            nxt = []
            for x, y in frontier:
                for dx, dy in moves:
                    c = (x + dx, y + dy)
                    if c not in dist and -margin <= c[0] <= LIMIT + margin and -margin <= c[1] <= LIMIT + margin:
                        dist[c] = dist[(x, y)] + 1
                        nxt.append(c)
            frontier = nxt
        _LEAPERS[(p, q, margin)] = dist
    return _LEAPERS[(p, q, margin)]


def leaper_moves(a, b, p, q):
    return leaper_table(p, q)[(a, b)]


def automaton_cells(a, rule):
    """Live cells after a steps of the one-dimensional automaton with the given rule number (Wolfram's numbering), from one live cell on a row without ends. The rule must be even (an empty row stays empty)."""
    live = {0}
    for _ in range(a):
        if not live:
            break
        lo, hi = min(live) - 1, max(live) + 1
        live = {i for i in range(lo, hi + 1) if rule >> (((i - 1 in live) << 2) | ((i in live) << 1) | (i + 1 in live)) & 1}
    return len(live)


def binary_palindrome(a):
    s = format(a, "b")
    return s == s[::-1]


def pal_branch(a, c, k):
    """3a + c if the binary form of a reads the same backwards (0 is the digit 0), else a div k + 1."""
    return 3 * a + c if binary_palindrome(a) else a // k + 1


def weave(a, b, first):
    """The decimal digits of a and b taken in turn (those of `first` going first), the rest of the longer one added at the end; read as a number."""
    x, y = (str(a), str(b)) if first == "a" else (str(b), str(a))
    out = []
    for i in range(max(len(x), len(y))):
        out.extend(s[i] for s in (x, y) if i < len(s))
    return int("".join(out))


def digit_sum(n):
    return sum(int(c) for c in str(n))


def digit_step(a, s):
    """x(0) = s, x(n + 1) = x(n) + (the sum of the decimal digits of x(n))."""
    x = s
    for _ in range(a):
        x += digit_sum(x)
    return x


def exp_mod(a, g, p):
    return pow(g, a, p)


def spiral_distance(a):
    """The cell numbered a of the square spiral that starts with 1 at the middle, 2 to its right and goes up first; the distance to the middle counted in steps along the grid."""
    x = y = 0
    dx, dy = 1, 0
    n, seg = 1, 1
    while n < a:
        for _ in range(2):
            for _ in range(seg):
                if n >= a:
                    return abs(x) + abs(y)
                x, y, n = x + dx, y + dy, n + 1
            dx, dy = -dy, dx
        seg += 1
    return abs(x) + abs(y)


def to_base(n, base):
    out = ""
    while n:
        out = str(n % base) + out
        n //= base
    return out or "0"


def distinct_substrings(a, base):
    """The number of different strings that occur inside the digits of a written in the given base."""
    s = to_base(a, base)
    return len({s[i:j] for i in range(len(s)) for j in range(i + 1, len(s) + 1)})


def last_nonzero(a, k):
    """The last digit that is not 0 in the decimal form of (a + k)!."""
    return int(str(math.factorial(a + k)).rstrip("0")[-1])


def next_palindrome(a, t):
    """The smallest number that is at least a + t and reads the same backwards in decimal."""
    n = a + t
    while str(n) != str(n)[::-1]:
        n += 1
    return n


# ---- the numerals ----

ONES = "zero one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen".split()
TENS = "zero ten twenty thirty forty fifty sixty seventy eighty ninety".split()


def english(n):
    if n < 20:
        return ONES[n]
    if n < 100:
        return TENS[n // 10] + ("" if n % 10 == 0 else " " + ONES[n % 10])
    raise ValueError(n)


CONSONANTS = "bdghklmnrstvxz"
VOWELS = "aeiou"
LASTS = ["", "", "", "l", "n", "r", "s"]
FRAME_WORDS = {"what", "is", "tell", "me", "please", "of", "and", "equals"}


class Words:
    """Invented words that are all different from each other and from the fixed words of the questions and of the English numerals."""

    def __init__(self, rng):
        self.rng, self.used = rng, set(FRAME_WORDS) | set(ONES) | set(TENS)

    def fresh(self, make):
        while True:
            w = make(self.rng)
            if w not in self.used:
                self.used.add(w)
                return w

    def kind_word(self):
        return self.fresh(lambda r: "".join([CONSONANTS[r.below(14)], VOWELS[r.below(5)], CONSONANTS[r.below(14)], VOWELS[r.below(5)], LASTS[r.below(len(LASTS))], ENDING]))

    def digit_word(self):
        return self.fresh(lambda r: "".join([CONSONANTS[r.below(14)], VOWELS[r.below(5)], CONSONANTS[r.below(14)]]))

    def vocab_word(self):
        return self.fresh(lambda r: "".join([CONSONANTS[r.below(14)], VOWELS[r.below(5)], CONSONANTS[r.below(14)], VOWELS[r.below(5)], CONSONANTS[r.below(14)], VOWELS[r.below(5)]]))


class Tongue:
    """How numbers are written in the questions and answers of a kind: digits, English words, an invented way of counting (digit words, most significant first), or a vocabulary of single words."""

    def __init__(self, label, say, parse):
        self.label, self.say, self.parse = label, say, parse


DIGITS = Tongue("digits", str, int)
SPELLED_UP_TO = 99  # the counting family spells numbers up to here
VOCAB = 64  # the vocabulary family has a word for each number 0 to 63


def make_counting(seed, words):
    """The counting family: English words for odd seeds, an invented positional way of counting (base 5, 6 or 7, digit words most significant first) for even seeds."""
    if seed % 2 == 1:
        return Tongue("english", english, parse_english)
    base = 5 + (seed // 2) % 3
    digit_words = [words.digit_word() for _ in range(base)]

    def say(n):
        return " ".join(digit_words[int(c)] for c in to_base(n, base))

    def parse(t):
        n = 0
        for w in t.split(" "):
            n = n * base + digit_words.index(w)
        return n

    tongue = Tongue(f"base{base}", say, parse)
    tongue.digit_words = digit_words
    return tongue


def parse_english(t):
    parts = t.split(" ")
    if len(parts) == 1:
        return ONES.index(parts[0]) if parts[0] in ONES else TENS.index(parts[0]) * 10
    return TENS.index(parts[0]) * 10 + ONES.index(parts[1])


def make_vocabulary(words):
    """The vocabulary family: a word of its own for each number 0 to 63, with nothing to say which comes before which."""
    vocab = [words.vocab_word() for _ in range(VOCAB)]
    tongue = Tongue("vocabulary", lambda n: vocab[n], lambda t: vocab.index(t))
    tongue.vocab = vocab
    return tongue


# ---- the kinds ----

SMALL = list(range(41)) + [50, 64, 100]  # the constants a short program over the starting operations may use as leaves (the same as in the earlier sets' checks)
PRIMES = [101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193, 197, 199, 211, 223, 227, 229, 233, 239, 241, 251]
COIN_SETS = [(1, 4, 9), (1, 5, 12), (1, 6, 10), (1, 7, 11), (1, 8, 13), (1, 5, 8), (1, 9, 14)]
LEAPERS = [(1, 2), (1, 2), (2, 3), (1, 2)]
RULES = [30, 54, 62, 110, 158, 182]
TEACH_ALL = 8  # the kinds taught in every way of asking carry the equivalences between the ways


class FKind(Kind):
    """A kind of this set: also which numerals it takes and gives ('d' digits, 'c' the counting family, 'v' the vocabulary family) and how its lessons are planned."""

    def __init__(self, name, role, arity, fn, rule, learn, tin="d", tout="d", lo=0, ordered=False, wide=False, bridge=False):
        super().__init__(name, role, arity, fn, rule, lo=lo, ordered=ordered)
        self.learn, self.tin, self.tout, self.wide, self.bridge = learn, tin, tout, wide, bridge


def code_kind(rng, name, bound, arity, tin, tout, stair=False):
    """An arbitrary code for each number (arity 1) or each pair of numbers 0..31 (arity 2): no rule. A `stair` is a rising run of codes (it looks like a rule and is not)."""
    n = (LIMIT + 1) ** arity
    if stair:
        codes, v = [], rng.below(5)
        for _ in range(n):
            codes.append(v)
            v += 1 + rng.below(9)
    else:
        codes = [rng.below(bound) for _ in range(n)]
    why = "cannot be learned: stays refused until told"
    if arity == 1:
        return FKind(name, "unexplainable", 1, lambda a, _b, c=codes: c[a], "an arbitrary code, no rule: " + " ".join(map(str, codes)), why, tin=tin, tout=tout)
    return FKind(name, "unexplainable", 2, lambda a, b, c=codes: c[a * (LIMIT + 1) + b], f"an arbitrary code from 0 to {bound - 1} for each pair of numbers 0..31 (drawn from the seed), no rule", why, tin=tin, tout=tout)


def make_kinds(seed):
    """The kinds of a world, in four groups: control, newop, compose, unexplainable (each group shuffled by the seed)."""
    rng = seeded(seed, 1)
    rc_s0 = rng.below(4) * 2  # 0, 2, 4 or 6
    while True:  # the coin set and the offset of the composition: the composition must take enough different values
        coins, bump_c = COIN_SETS[rng.below(len(COIN_SETS))], 1 + rng.below(3)
        if len({min_coins(2 * a + bump_c, coins) for a in range(LIMIT + 1)}) >= 7:
            break
    leap = LEAPERS[rng.below(len(LEAPERS))]
    while True:  # an automaton whose live cells take enough different values in 32 steps
        ca_rule = RULES[rng.below(len(RULES))]
        if len({automaton_cells(a, ca_rule) for a in range(LIMIT + 1)}) >= 12:
            break
    pb_c, pb_k = 1 + rng.below(5), 2 + rng.below(2)
    wv_first = "ab"[rng.below(2)]
    dg_s = 1000 + rng.below(9000)
    while True:  # powers of a base modulo a prime: the base must have a long period, so that 32 steps do not repeat
        em_p, em_g = PRIMES[rng.below(len(PRIMES))], 2 + rng.below(40)
        if len({pow(em_g, a, em_p) for a in range(LIMIT + 1)}) >= 28:
            break
    ds_base = 2 + rng.below(2)
    ln_k = rng.below(2)
    np_t = 1 + rng.below(3)
    tr_t = 3 + rng.below(3)
    lin_c = 1 + rng.below(4)

    def rec(a):
        return recaman(a, rc_s0)

    def bump(a):
        return 2 * a + bump_c

    def subs(a):
        return distinct_substrings(a, ds_base)

    def lin(a):
        return 3 * a + lin_c

    newop = [
        FKind("recaman", "newop", 1, lambda a, _b: rec(a), f"x({0}) = {rc_s0}, x(n) = x(n - 1) - n if that is above 0 and not yet in the sequence, else x(n - 1) + n; the answer is x(a)", "a number defined from the earlier ones and from the set of all the earlier ones: the answers step back by n when they can, and the table of answers shows which steps were taken back"),
        FKind("mincoins", "newop", 1, lambda a, _b: min_coins(a, coins), f"the fewest coins that make a with coins of {coins[0]}, {coins[1]} and {coins[2]}", "a table filled in from the left: the answer for a is 1 + the least of the answers at a - c for the coins c; the answers mostly rise by steps of 0 or 1 and fall back at the multiples of the large coin, and the greedy way is not always the best"),
        FKind("leaper", "newop", 2, lambda a, b: leaper_moves(a, b, *leap), f"the fewest moves of a piece that moves {leap[0]} squares one way and {leap[1]} the other (any of the eight ways) from (0, 0) to (a, b) on a board without edges", "a breadth-first count over a grid: it is symmetric in a and b, 0 only at (0, 0), and grows by about a third of the larger number; the corner squares near the start are the odd ones", ),
        FKind("automaton", "newop", 1, lambda a, _b: automaton_cells(a, ca_rule), f"the number of live cells after a steps of the one-dimensional automaton with rule {ca_rule} (Wolfram's numbering), from one live cell", f"a row rewritten a times by a table of 8 entries over three neighbors; a live cell appears only next to a live cell, and the row grows by one cell on each side per step at most"),
        FKind("palbranch", "newop", 1, lambda a, _b: pal_branch(a, pb_c, pb_k), f"3 * a + {pb_c} if the binary form of a reads the same backwards (0 is the digit 0), else a div {pb_k} + 1", "a branch on a property that is neither a threshold nor a remainder: the numbers 0, 1, 3, 5, 7, 9, 15, 17, 21, 27, 31 give the large answers, and the property can be read off the binary forms of those", ),
        FKind("weave", "newop", 2, lambda a, b: weave(a, b, wv_first), f"the decimal digits of {'a and b' if wv_first == 'a' else 'b and a'} taken in turn ({'a' if wv_first == 'a' else 'b'} first), the rest of the longer one added at the end, read as a number", "a string made of two strings: a one-digit pair gives its two digits side by side (so 10 * the first + the second), and a longer number has its digits moved apart by the other's"),
        FKind("digitstep", "newop", 1, lambda a, _b: digit_step(a, dg_s), f"x(0) = {dg_s}, x(n + 1) = x(n) + the sum of the decimal digits of x(n); the answer is x(a)", "a recurrence on the digits of the last answer, with a large start: each answer is the one before it plus a digit sum, so consecutive answers differ by 1 to 36"),
        FKind("powmod", "newop", 1, lambda a, _b: exp_mod(a, em_g, em_p), f"{em_g}^a mod {em_p}", f"a simple rule with large constants: each answer is {em_g} times the one before it, mod {em_p}, starting at 1; the quotient of consecutive answers is the same constant"),
        FKind("spiral", "newop", 1, lambda a, _b: spiral_distance(a), "the cell numbered a of the square spiral that starts with 1 at the middle, 2 to its right and goes up first: its distance to the middle counted in steps along the grid (a from 1)", "a walk on a grid: the answers rise and fall as the walk goes round the square, and the ring they belong to grows at 1, 9, 25 (the corners of the rings are the largest answers)", tin="c", tout="c", lo=1),
        FKind("substrings", "newop", 1, lambda a, _b: subs(a), f"the number of different strings that occur inside the base-{ds_base} digits of a (a from 1)", "a count over the substrings of a string: it is read off the digits of a in the other base, and the number of digits sets how many substrings there are at most", tin="c", tout="c", lo=1),
        FKind("lastnonzero", "newop", 1, lambda a, _b: last_nonzero(a, ln_k), f"the last digit that is not 0 in the decimal form of (a + {ln_k})!", "a running product with the zeros taken off: each answer is a function of the one before it (and of the digit it multiplies by), so the table repeats in a pattern of 4 and 5", tin="v", tout="v"),
        FKind("nextpal", "newop", 1, lambda a, _b: next_palindrome(a, np_t), f"the smallest number that is at least a + {np_t} and reads the same backwards in decimal", "a search above the input for a property of its digits: the answers equal a + t when that already reads the same backwards and otherwise jump to the next such number", tin="v", tout="v"),
    ]
    compose = [
        FKind("digitsrec", "compose", 1, lambda a, _b: digit_sum(rec(a)), "the sum of the decimal digits of recaman(a)", "an earlier simple kind (the digit sum) applied to an earlier hard kind"),
        FKind("coinsbump", "compose", 1, lambda a, _b: min_coins(bump(a), coins), f"mincoins(2 * a + {bump_c})", "an earlier hard kind fed with the answer of a simple one"),
        FKind("spelledsub", "compose", 1, lambda a, _b: lin(subs(a)), f"3 * substrings(a) + {lin_c}, with the numbers spelled as words", "a simple rule of the counting family fed with the answer of a hard kind of the same family", tin="c", tout="c", lo=1),
    ]
    control = [
        FKind("digitsum", "control", 1, lambda a, _b: digit_sum(a), "the sum of the decimal digits of a", "a few nodes over the starting operations on 0 to 31 (a mod 10 + a div 10)"),
        FKind("ratio", "control", 2, lambda a, b: a * b // (a + b), "a * b div (a + b) (a and b from 1)", "a few nodes over the starting operations", lo=1),
        FKind("tricky", "control", 1, lambda a, _b: a // tr_t * (a % tr_t), f"(a div {tr_t}) * (a mod {tr_t})", "a few nodes over the starting operations"),
        FKind("bumpby", "control", 1, lambda a, _b: bump(a), f"2 * a + {bump_c}", "a few nodes over the starting operations; taught in all eight ways of asking, so the ways can be seen to mean the same", bridge=True),
        FKind("diffsquares", "control", 2, lambda a, b: a * a - b * b, "a * a - b * b (asked only with a at least b)", "a few nodes over the starting operations; taught in all eight ways of asking, so the ways can be seen to mean the same", ordered=True, bridge=True),
        FKind("spellnum", "control", 1, lambda a, _b: a, "a written in digits, the answer is a spelled as words (0 to 99)", "the words are lined up with the digits by the lessons: every number from 0 to 99 is taught but ten, the pieces of the held-out ones all occur in other lessons", tout="c", wide=True),
        FKind("readnum", "control", 1, lambda a, _b: a, "a spelled as words, the answer is a in digits (0 to 99)", "the same lessons from the other side: the pieces of a held-out numeral all occur in other lessons", tin="c", wide=True),
        FKind("spelledmix", "control", 2, lambda a, b: 2 * a + b, "2 * a + b with both numbers and the answer spelled as words", "a few nodes over the starting operations once the words are read as numbers; the lessons that tie the words to digits show how", tin="c", tout="c"),
        FKind("spelledlin", "control", 1, lambda a, _b: lin(a), f"3 * a + {lin_c} with the number and the answer spelled as words", "a few nodes over the starting operations once the words are read as numbers", tin="c", tout="c"),
        FKind("firstof", "control", 2, lambda a, _b: a, "the first of the two words (words of the vocabulary family)", "a copy: the answer is the first word of the question, whatever the words mean", tin="v", tout="v"),
    ]
    unexplainable = [
        code_kind(rng, "noise9", 9, 1, "d", "d"),
        code_kind(rng, "stair", 0, 1, "d", "d", stair=True),
        code_kind(rng, "noisewords", 40, 1, "v", "v"),
        code_kind(rng, "noisepairwords", 40, 2, "v", "v"),
    ]
    return rng.shuffled(control), rng.shuffled(newop), rng.shuffled(compose), unexplainable


# ---- the ways of asking ----

UNARY = ["what is {w} {a}", "{w} {a}", "{a} {w}", "what is {a} {w}", "tell me what is {w} {a}", "please {w} {a}", "{w} of {a}", "what is {w} of {a}"]
BINARY = ["what is {a} {w} {b}", "{a} {w} {b}", "{a} {w} {b} equals", "what is {a} {w} {b} equals", "tell me what is {a} {w} {b}", "please {a} {w} {b}", "{w} of {a} and {b}", "what is {w} of {a} and {b}"]


def frames_of(arity):
    return UNARY if arity == 1 else BINARY


def put(frame, word, *args):
    """A question: a way of asking with the word and the numbers (already written in their numerals) in it."""
    out = frame.replace("{w}", word)
    for slot, x in zip("ab", args):
        out = out.replace("{" + slot + "}", x)
    return out


# ---- the exam ----


def world_tongues(seed):
    """The word generator of a world (already used for the numerals) and its three ways of writing numbers."""
    words = Words(seeded(seed, 2))
    return words, {"d": DIGITS, "c": make_counting(seed, words), "v": make_vocabulary(words)}


def lesson_plan(k, rng):
    """(taught, held, beyond) as lists of argument tuples."""
    if k.wide:
        held = rng.shuffled([n for n in range(21, SPELLED_UP_TO + 1) if n % 10])[:10]
        return [(n,) for n in range(SPELLED_UP_TO + 1) if n not in held], [(n,) for n in held], []
    near, far = (rng.shuffled(xs) for xs in points(k))
    if k.arity == 1:
        return near[HELD_UNARY:], near[:HELD_UNARY], far[:BEYOND_UNARY]
    taught = 40 if k.bridge else TAUGHT_BINARY
    return near[HELD_BINARY : HELD_BINARY + taught], near[:HELD_BINARY], far[:BEYOND_BINARY]


def trap_rows(rng, order, tongues, words, used):
    """Rows with no taught answer: (unit, question, truth). `used` holds every question already in the exam."""
    say = lambda t, n: tongues[t].say(n)  # noqa: E731
    top = {"d": 100, "c": SPELLED_UP_TO + 1, "v": VOCAB}
    rows = []

    def add(unit, question, tout):
        if question not in used:
            used.add(question)
            rows.append((unit, question, say(tout, rng.below(top[tout]))))
            return True
        return False

    def arg(t):
        return say(t, rng.below(32))

    live = [k for k in order if k.role != "unexplainable"]
    unary = [k for k in live if k.arity == 1]
    binary = [k for k in live if k.arity == 2]
    # two kinds at once
    for group, n in (("d", 6), ("c", 2)):
        pool = [k for k in unary if k.tin == group and k.tout == group]
        done = 0
        while done < n:
            k1, k2 = rng.shuffled(pool)[:2]
            a = arg(group)
            frame = ["what is {w1} {a} and {w2} {a}", "{w1} {a} and {w2} {a}"][rng.below(2)]
            done += add(k1.word, frame.replace("{w1}", k1.word).replace("{w2}", k2.word).replace("{a}", a), k1.tout)
    # a new word
    fresh = [words.kind_word() for _ in range(8)]
    for i, w in enumerate(fresh):
        t = "ddcv"[i % 4]
        if i < 4:
            while not add(w, put(UNARY[i], w, arg(t)), t):
                pass
        else:
            while not add(w, put(BINARY[i % 2], w, arg(t), arg(t)), t):
                pass
    # the wrong number of numbers
    for k in rng.shuffled(unary)[:4]:
        while not add(k.word, put(BINARY[rng.below(2)], k.word, arg(k.tin), arg(k.tin)), k.tout):
            pass
    for k in rng.shuffled(binary)[:4]:
        while not add(k.word, put(UNARY[rng.below(4)], k.word, arg(k.tin)), k.tout):
            pass
    # a taught word with numbers of a family it was not taught with
    other = {"d": "v", "c": "v", "v": "c"}
    for k in rng.shuffled(unary)[:4]:
        while not add(k.word, put(UNARY[rng.below(4)], k.word, arg(other[k.tin])), k.tout):
            pass
    for k in rng.shuffled(binary)[:4]:
        t = other[k.tin]
        while not add(k.word, put(BINARY[rng.below(2)], k.word, arg(t), arg(t)), k.tout):
            pass
    # a bare number
    label = words.kind_word()
    for t in ("d", "d", "c", "c", "v", "v"):
        while not add(label, ["what is {a}", "{a}"][rng.below(2)].replace("{a}", arg(t)), t):
            pass
    return rows


def build(seed, compose_first=False):
    """The whole exam as (lessons, stream, key): lists of rows of strings."""
    control, newop, compose, unexplainable = make_kinds(seed)
    order = control + (compose + newop if compose_first else newop + compose) + unexplainable
    words, tongues = world_tongues(seed)
    for k in order:
        k.word = words.kind_word()
    rng = seeded(seed, 3)
    # the ways of asking: two or three taught for each kind (all eight for the two that carry the equivalences); 14 kinds are asked in one that was not taught for them
    eligible = [k for k in order if k.role != "unexplainable" and not k.bridge]
    reworded = {id(k) for k in rng.shuffled(eligible)[:14]}
    lessons, items, key, used, fact = [], [], [], set(), 0
    for k in order:
        frames = frames_of(k.arity)
        taught_frames = list(range(len(frames))) if k.bridge else sorted(rng.shuffled(range(len(frames)))[: 2 + rng.below(2)])
        untaught = rng.shuffled([i for i in range(len(frames)) if i not in taught_frames])[0] if id(k) in reworded else None
        notes = f"; numerals: {', '.join(f'{x} {tongues[t].label}' for x, t in (('in', k.tin), ('out', k.tout)))}; taught in {len(taught_frames)} ways of asking" + ("" if untaught is None else ", the held-out facts are asked in another")
        if "c" in (k.tin, k.tout):
            notes += "; the counting family: " + ("the lessons of the two spelling kinds line the words up with digits and the sums show the sums" if tongues["c"].label == "english" else "digit words, most significant first, whose meaning the lessons of the two spelling kinds fix")
        if "v" in (k.tin, k.tout):
            notes += "; the vocabulary family: no lesson ties the words to digits or to an order, so only an assignment of numbers to the words that fits the lessons of all the kinds of the family at once would give their meaning, and a refusal is the expected answer for most rows"
        key.append([k.word, k.role, k.name, k.rule + notes, k.learn])
        taught, held, beyond = lesson_plan(k, rng)

        def text(p, frame):
            args = p + (0,) * (2 - len(p))
            return put(frames[frame], k.word, *(tongues[k.tin].say(x) for x in p)), tongues[k.tout].say(k.fn(*args))

        for p in taught:
            for f in taught_frames:
                q, truth = text(p, f)
                lessons.append([q, truth, k.word, str(fact)])
                used.add(q)
            fact += 1
        rows = [(p, untaught if untaught is not None else taught_frames[rng.below(len(taught_frames))]) for p in held] + [(p, taught_frames[0]) for p in beyond]
        for p, f in rows:
            q, truth = text(p, f)
            items.append([k.role, k.word, q, truth])
            used.add(q)
            fact += 1
    for unit, q, truth in trap_rows(rng, order, tongues, words, used):
        items.append(["trap", unit, q, truth])
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
    p = argparse.ArgumentParser(description="Write a sixth hidden-worlds adaptive exam (lessons.tsv, stream.tsv) for `baby exam adapt-drive`.")
    p.add_argument("dir")
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--compose-first", action="store_true", help="teach the compositions before the kinds they use (a control: reuse should then not help)")
    a = p.parse_args(argv)
    nl, ns = write(a.dir, a.seed, a.compose_first)
    print(f"wrote the hidden worlds F (seed {a.seed}) to {a.dir}: {nl} lessons and a stream of {ns} questions")


if __name__ == "__main__":
    main(sys.argv[1:])
