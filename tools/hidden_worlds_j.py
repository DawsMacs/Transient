#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""A tenth set of exam worlds: closed forms of new shapes, rules that run a loop over digits, over a grid, over a ring of digits or over the subsets of a set, numbers
defined by their own earlier values, a branch on a property of the number, numbers spelled in six numeral grammars whose structure is not that of the earlier sets, numbers
whose forms are mixed inside a question, new ways of asking built from pieces of each other, and traps that look like the rewordings.

The same files, line protocol and shape of world as `tools/hidden_worlds.py` and its eight successors (so `baby exam adapt-drive DIR COMMAND...` judges any
system on it). To keep the set a blind test the kinds are not described here.

    python3 tools/hidden_worlds_j.py DIR [--seed N] [--compose-first]

What is new in the way the exam is put:
  numerals   some kinds take and give their numbers as words, in one of six grammars (chosen by the seed; each world uses three of them, one per family of
             words, and a fourth family that no kind reads). In the first family a pair of kinds ties the words to digits, so their meaning can be worked out; in
             the second no kind reads or writes the words and the meaning can only come through arithmetic done on them; in the third nothing in the lessons can
             fix the meaning. A learner is never told that the words are numbers.
  mixed      kinds that take one number as digits and the other as words, kinds whose lessons draw the form of each number, kinds whose answer is in words of
             one family for a question in digits, kinds whose answer is in digits for a question in words, a kind that answers in the words of another family than
             the question is in, and kinds taught in one form of numbers and asked in the other.
  wordings   every kind is taught in two or three of several ways of asking (three kinds in all of them). The facts that are held out are then asked in a way
             that was not taught for the kind, in a mix of the start of one taught way and the end of another, in a taught way with a filler word dropped, or in
             a taught way; next to them stand traps that look just like these rewordings and are not answerable.
  arguments  kinds with one, two and three numbers; questions that name the same number more than once, numerals with a word that occurs more than once, and
             ways of asking in which a word of the question occurs more than once.
  refusals   next to the kinds without a rule, rows with a taught word in a place it was not taught for, two kinds at once, a new word, a word that is almost
             a kind word or a number word, runs of words that are no numeral of the grammar, numerals followed by words of a family that no kind reads, words
             in an order no lesson used, and numbers in a form a kind was never taught with.

Roles, as the judge groups them (the stream's role column):
  newop          needs more than the starting operations in seven nodes (10 kinds)
  compose        a small composition of kinds of this set, taught after them (2 kinds)
  control        expressible with the starting operations, or a copy, a count or a way of spelling, so a regression shows (15 kinds)
  unexplainable  an arbitrary code for each number or pair, or a code that looks like a rule: no rule, must stay refused until told (5 kinds)
  trap           a question that looks like an ordinary one and has no taught answer: must be refused (no lessons)

The words and a few constants are drawn from the seed. Standard library only.
"""
import argparse
import itertools
import math
import os
import sys
from functools import lru_cache

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hidden_worlds import LIMIT, Kind, Rng  # noqa: E402

README = """THE HIDDEN WORLDS J (adaptive exam; the same files and protocol as `baby exam adapt-write`, written by tools/hidden_worlds_j.py)

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

ENDING = "bimo"  # shared by every kind word of this set (the earlier sets' words end in `sido`, `velu`, `ketu`, `ruvo`, `zane`, `yarn`, `tole`, `dopa` and `cuwe`)


def seeded(seed, salt):
    """A generator for a seed and a purpose (the seed goes through one extra mix first, so that neighboring seeds are not the same stream a few steps on)."""
    return Rng(Rng(seed * 4000037 + salt * 170189 + 59).next())


# ---- the operations ----


def rem_sum(a, b):
    """The sum of k mod b for k from 1 to a (b from 1)."""
    return sum(k % b for k in range(1, a + 1))


def prog_sum(a, b, c):
    """The first a terms of the progression that starts at b and grows by c at each step."""
    return a * b + c * (a * (a - 1) // 2)


def digit_run(a):
    """The largest total of a stretch of decimal digits that rise strictly from left to right (a stretch of one digit counts)."""
    best = cur = prev = -1
    for ch in str(a):
        d = int(ch)
        cur = cur + d if d > prev and prev >= 0 else d
        prev = d
        best = max(best, cur)
    return best


def least_turn(a):
    """The smallest number that the decimal digits of a make when they are turned round as a ring (a rotation may begin with 0)."""
    s = str(a)
    return min(int(s[i:] + s[:i]) for i in range(len(s)))


@lru_cache(maxsize=None)
def coin_split(n):
    """The most a coin of n can be turned into when it may be changed into three coins of n div 2, n div 3 and n div 4 (changed again as often as that pays)."""
    return n if n < 12 else max(n, coin_split(n // 2) + coin_split(n // 3) + coin_split(n // 4))


def least_prime_factor(a):
    return next(p for p in range(2, a + 1) if a % p == 0)


def is_prime(a):
    return a >= 2 and all(a % p for p in range(2, math.isqrt(a) + 1))


def grid_paths(a, b, m, r, p):
    """The paths of unit steps right or up from (0, 0) to (a, b) that avoid every cell (i, j) with i * j mod m = r, counted modulo p."""
    row = [0] * (b + 1)
    for i in range(a + 1):
        for j in range(b + 1):
            if (i * j) % m == r:
                row[j] = 0
            elif i == 0 and j == 0:
                row[j] = 1
            else:
                row[j] = ((row[j] if i else 0) + (row[j - 1] if j else 0)) % p
    return row[b]


def subset_count(a, b):
    """How many subsets of {1, ..., a} (the empty one included) have a sum divisible by b."""
    cnt = [1] + [0] * (b - 1)
    for x in range(1, a + 1):
        cnt = [cnt[r] + cnt[(r - x) % b] for r in range(b)]
    return cnt[0]


# ---- the words ----

CONSONANTS = "bdghklmnprstvz"
VOWELS = "aeiou"
LASTS = ["", "", "", "l", "n", "r", "s"]
NUMBER_FIRST = "f"  # every number word of this set begins with it, then a vowel, two different consonants and a vowel (the earlier sets' number words are other shapes)


class Frame:
    """A way of asking: tokens with {w} for the kind's word and {a}, {b}, {c} for the numbers. A word in [brackets] is a filler that can be left out."""

    def __init__(self, text):
        self.toks, self.drop = [], []
        for t in text.split(" "):
            if t.startswith("["):
                self.drop.append(len(self.toks))
                t = t[1:-1]
            self.toks.append(t)
        self.text = " ".join(self.toks)
        self.w = self.toks.index("{w}")
        self.slots = [t for t in self.toks if t in SLOTS]

    def markers(self):
        """{slot: index of the word just before the slot} for the numbers that follow a word of the frame."""
        return {t: i - 1 for i, t in enumerate(self.toks) if t in SLOTS and i > 0 and self.toks[i - 1] not in SLOTS and self.toks[i - 1] != "{w}"}


SLOTS = ("{a}", "{b}", "{c}")
FRAMES = {
    1: [Frame(t) for t in ["gauge {w} at {a} [promptly]", "{a} is sent into {w} [meanwhile]", "report what {w} gives for {a}", "for the {a} give the {w}", "track {w} across {a}", "once {a} enters {w} what leaves", "inside {w} the {a} is mapped to [what]", "declare {w} of {a} [please]"]],
    2: [Frame(t) for t in ["link {w} across {a} and {b}", "{a} followed by {b} enters {w} [promptly]", "bind {w} to the first {a} and the second {b}", "when {a} and {b} are given gauge {w} [meanwhile]", "{w} scanning {a} beside {b}", "with {a} to the west and {b} to the east report {w}", "track {w} across {a} and across {b}", "{a} coupled with {b} is sent into {w}"]],
    3: [Frame(t) for t in ["gauge {w} at {a} then {b} then {c}", "report {w} with first {a} second {b} third {c}", "when {a} and {b} and {c} are given gauge {w} [meanwhile]", "send {a} and {b} and {c} into {w} [promptly]", "track {w} across {a} and across {b} and across {c}", "bind {w} to the north {a} the center {b} the south {c}"]],
}
# a start or an end that no way of asking has, for any kind
NEW_STARTS = [["picture"], ["let", "me", "check"], ["pretend"], ["grant"]]
NEW_ENDS = [["anyhow"], ["by", "then"], ["if", "at", "all"], ["for", "us"]]
JOIN = ["plus", "afterwards"]
BARE = ["{a}", "just {a}", "how about {a}"]
FRAME_WORDS = {t for fs in FRAMES.values() for f in fs for t in f.toks if t not in SLOTS and t != "{w}"}
FRAME_WORDS |= {t for ws in NEW_STARTS + NEW_ENDS + [JOIN] for t in ws} | {t for b in BARE for t in b.split(" ") if t not in SLOTS}
FILLERS = ["at", "across", "to", "into"]  # words of the frames that stand in for the kind's word in a trap


class Words:
    """Invented words that are all different from each other and from the fixed words of the questions."""

    def __init__(self, rng):
        self.rng, self.used = rng, set(FRAME_WORDS)

    def fresh(self, make):
        while True:
            w = make(self.rng)
            if w not in self.used:
                self.used.add(w)
                return w

    def kind_word(self):
        return self.fresh(lambda r: "".join([CONSONANTS[r.below(14)], VOWELS[r.below(5)], CONSONANTS[r.below(14)], VOWELS[r.below(5)], LASTS[r.below(len(LASTS))], ENDING]))

    def number_word(self):
        while True:
            w = self.fresh(lambda r: NUMBER_FIRST + VOWELS[r.below(5)] + CONSONANTS[r.below(14)] + CONSONANTS[r.below(14)] + VOWELS[r.below(5)])
            if w[2] != w[3]:
                return w

    def almost(self, word, keep=0, number=False):
        """A word one letter away from `word` (a vowel for a vowel, a consonant for a consonant, none of the last `keep` letters, not the first letter of a number word)
        that is no word of the exam."""
        while True:
            i = (1 if number else 0) + self.rng.below(len(word) - keep - (1 if number else 0))
            pool = VOWELS if word[i] in VOWELS else CONSONANTS
            w = word[:i] + pool[self.rng.below(len(pool))] + word[i + 1 :]
            if w not in self.used and not (number and w[3] == w[2]):
                self.used.add(w)
                return w


# ---- the numerals ----


class Tongue:
    """How numbers are written in the questions and answers of a kind: digits, one of the numeral grammars, a mix of digits and the first family, or raw words."""

    cap = 999  # the largest number the grammar writes
    words = ()
    foreign = ()  # words of the other families, for runs that mix them
    _index = None

    def __init__(self, label):
        self.label = label

    def say(self, n):
        return str(n)

    def parse(self, text):
        if self._index is None:
            self._index = {self.say(n): n for n in range(self.cap + 1)}
        if text not in self._index:
            raise ValueError(text)
        return self._index[text]

    def valid(self, text):
        try:
            self.parse(text)
            return True
        except ValueError:
            return False

    def some(self, rng):
        """A number whose numeral has a few words."""
        return 12 + rng.below(self.cap - 12)

    # ways to write something that is no numeral; each returns a string for which `parse` fails
    def bad_junk(self, rng):
        """The words of a numeral with two of them exchanged."""
        for _ in range(500):
            toks = self.say(self.some(rng)).split(" ")
            i, j = rng.below(len(toks)), rng.below(len(toks))
            if toks[i] != toks[j]:
                toks[i], toks[j] = toks[j], toks[i]
                if not self.valid(" ".join(toks)):
                    return " ".join(toks)
        raise RuntimeError(self.label)

    def bad_repeat(self, rng):
        """The words of a numeral with one of them once or twice more next to itself."""
        for _ in range(500):
            toks = self.say(self.some(rng)).split(" ")
            i = rng.below(len(toks))
            toks[i:i] = [toks[i]] * (1 + rng.below(2))
            if not self.valid(" ".join(toks)):
                return " ".join(toks)
        raise RuntimeError(self.label)

    def spliced(self, rng):
        """A numeral of this grammar (of two words or more) with one of its words replaced by a word of another family."""
        toks = self.say(self.some(rng)).split(" ")
        while len(toks) < 2:
            toks = self.say(self.some(rng)).split(" ")
        toks[rng.below(len(toks))] = self.foreign[rng.below(len(self.foreign))]
        return " ".join(toks)


DIGITS = Tongue("digits")


class Raw(Tongue):
    def say(self, x):
        return x

    def parse(self, text):
        return text


RAW = Raw("raw")


class Residue(Tongue):
    """Three words that are remainders: the first of the number modulo one of 7, 11, 13, the second modulo another, the third modulo the last (the order of the three
    moduli differs between variants). One set of thirteen digit words 0 to 12 is shared by the three places, so the place of a word says which modulus it is a
    remainder of, and a word for 12 cannot stand where the modulus is 7 or 11. Numbers up to 999; every number has a numeral of three words."""

    cap = 999
    ORDERS = [(7, 11, 13), (13, 7, 11), (11, 13, 7)]

    def __init__(self, label, words, param):
        super().__init__(label)
        self.mods = self.ORDERS[param % 3]
        self.digit = [words.number_word() for _ in range(13)]
        self.words = list(self.digit)

    def say(self, n):
        return " ".join(self.digit[n % m] for m in self.mods)

    def bad_junk(self, rng):
        """A numeral with a digit word that is no remainder in its place (a remainder too large for the modulus there)."""
        toks = self.say(self.some(rng)).split(" ")
        small = [p for p in range(3) if self.mods[p] < 13]
        place = small[rng.below(len(small))]
        toks[place] = self.digit[self.mods[place] + rng.below(13 - self.mods[place])]
        return " ".join(toks)

    def bad_repeat(self, rng):
        toks = self.say(self.some(rng)).split(" ")
        i = rng.below(3)
        toks[i:i] = [toks[i]] * (1 + rng.below(2))
        return " ".join(toks)


class Runs(Tongue):
    """The binary form of the number as run lengths: the lengths of the runs of equal digits from the top (the first run is a run of 1s, so they alternate 1s, 0s, 1s ...),
    each a word of its own for the lengths 1 to 9, and a word for zero that stands alone (the lengths from the bottom in one variant). Numbers up to 999."""

    cap = 999

    def __init__(self, label, words, param):
        super().__init__(label)
        self.low_first = param % 2 == 1
        self.zero = words.number_word()
        self.run = {i: words.number_word() for i in range(1, 10)}
        self.words = [self.zero] + list(self.run.values())

    def lengths(self, n):
        return [len(list(g)) for _, g in itertools.groupby(bin(n)[2:])]

    def say(self, n):
        if n == 0:
            return self.zero
        ls = self.lengths(n)
        return " ".join(self.run[x] for x in (ls[::-1] if self.low_first else ls))

    def bad_junk(self, rng):
        """The zero word inside a numeral (it stands alone), or twice."""
        toks = self.say(self.some(rng)).split(" ")
        if rng.below(4) == 0:
            return f"{self.zero} {self.zero}"
        toks.insert(rng.below(len(toks) + 1), self.zero)
        return " ".join(toks)

    def bad_repeat(self, rng):
        """A word for a run of 5 or more three or four times in a row (a number of far more than ten binary digits)."""
        w = self.run[5 + rng.below(5)]
        return " ".join([w] * (3 + rng.below(2)))


class Comb(Tongue):
    """The combinatorial number system with three words: a number n is C(x, 3) + C(y, 2) + z with x > y > z >= 0, written as the three words of x, y and z in falling
    order (rising order in one variant); one set of twenty digit words 0 to 19 for all three places. Numbers up to 999; no repeated word within a numeral."""

    cap = 999

    def __init__(self, label, words, param):
        super().__init__(label)
        self.rising = param % 2 == 1
        self.digit = [words.number_word() for _ in range(20)]
        self.words = list(self.digit)

    def say(self, n):
        x = 2
        while math.comb(x + 1, 3) <= n:
            x += 1
        n -= math.comb(x, 3)
        y = 1
        while math.comb(y + 1, 2) <= n:
            y += 1
        toks = [self.digit[x], self.digit[y], self.digit[n - math.comb(y, 2)]]
        return " ".join(toks[::-1] if self.rising else toks)


class Countdown(Tongue):
    """A number counted down from the nearest triangular landmark above it: two words, the landmark k (the number k * (k + 1) / 2) and how far short of it the number is
    (from 0 to k - 1); one set of forty-five digit words 0 to 44 for the two places; the short-of word first in one variant; zero is the landmark 0 and no shortfall.
    Numbers up to 990."""

    cap = 990

    def __init__(self, label, words, param):
        super().__init__(label)
        self.short_first = param % 2 == 1
        self.digit = [words.number_word() for _ in range(45)]
        self.words = list(self.digit)

    def say(self, n):
        k = 0
        while k * (k + 1) // 2 < n:
            k += 1
        toks = [self.digit[k], self.digit[k * (k + 1) // 2 - n]]
        return " ".join(toks[::-1] if self.short_first else toks)


FIBS = [1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987]


class Zeckendorf(Tongue):
    """One word for each of the Fibonacci numbers 1, 2, 3, 5, 8, ... 987; a number is written as the words of the Fibonacci numbers that add up to it with no two
    neighbors in the sequence used (largest first, smallest first in one variant); a word for zero that stands alone. Numbers up to 999."""

    cap = 999

    def __init__(self, label, words, param):
        super().__init__(label)
        self.rising = param % 2 == 1
        self.zero = words.number_word()
        self.fib = [words.number_word() for _ in FIBS]
        self.words = [self.zero] + list(self.fib)

    def say(self, n):
        if n == 0:
            return self.zero
        toks = []
        for i in range(len(FIBS) - 1, -1, -1):
            if FIBS[i] <= n:
                toks.append(self.fib[i])
                n -= FIBS[i]
        return " ".join(toks[::-1] if self.rising else toks)

    def bad_junk(self, rng):
        """Two neighbors of the sequence side by side (in the order of the variant), or two words exchanged."""
        if rng.below(2):
            i = rng.below(len(FIBS) - 1)
            toks = [self.fib[i + 1], self.fib[i]]
            return " ".join(toks[::-1] if self.rising else toks)
        return super().bad_junk(rng)


class Negabase(Tongue):
    """Place value in base minus three: a word for each of the digits 0, 1, 2 and nothing else; the digits from the top (from the bottom in one variant), no zero in front
    (the number zero is the zero digit alone). Numbers up to 999; numerals of up to seven words, most with a word that occurs several times."""

    cap = 999

    def __init__(self, label, words, param):
        super().__init__(label)
        self.low_first = param % 2 == 1
        self.digit = [words.number_word() for _ in range(3)]
        self.words = list(self.digit)

    def say(self, n):
        ds = []
        while n:
            r = n % 3
            ds.append(r)
            n = (n - r) // -3
        ds = ds or [0]
        return " ".join(self.digit[d] for d in (ds if self.low_first else ds[::-1]))

    def bad_junk(self, rng):
        """A numeral with a zero in front, or a numeral of two words, which is always below zero."""
        if rng.below(2):
            toks = self.say(self.some(rng)).split(" ")
            toks.insert(len(toks) if self.low_first else 0, self.digit[0])
            return " ".join(toks)
        toks = [self.digit[1 + rng.below(2)], self.digit[rng.below(3)]]
        return " ".join(toks[::-1] if self.low_first else toks)

    def bad_repeat(self, rng):
        return " ".join([self.digit[0]] * (2 + rng.below(3)))


class Mix(Tongue):
    """Each number of a question written either in digits or in words of the first family (the form is drawn for each number of each fact)."""

    def __init__(self, label, first):
        super().__init__(label)
        self.first, self.cap, self.words = first, first.cap, first.words

    def say(self, n, pick=None):
        return str(n) if pick is None or pick.below(2) else self.first.say(n)

    def parse(self, text):
        try:
            return int(text)
        except ValueError:
            return self.first.parse(text)


GRAMMARS = ["residue", "comb", "runs", "zeck", "negabase", "countdown"]
KINDS_OF_GRAMMAR = {"residue": Residue, "comb": Comb, "runs": Runs, "zeck": Zeckendorf, "negabase": Negabase, "countdown": Countdown}
# the grammars of the first, second and third family for each seed are three in a row of this ring (the fourth is the next one; the seeds after the sixth begin again)
RING = ["residue", "comb", "runs", "zeck", "negabase", "countdown"]


def describe(g, tongue):
    if g == "residue":
        return "three remainders, modulo " + ", ".join(map(str, tongue.mods)) + " in that order, one set of thirteen digit words for the three places"
    if g == "comb":
        return "combinatorial number system C(x, 3) + C(y, 2) + z, three words " + ("rising" if tongue.rising else "falling")
    if g == "runs":
        return "run lengths of the binary form, " + ("from the bottom" if tongue.low_first else "from the top")
    if g == "zeck":
        return "Fibonacci words, no two neighbors used, " + ("smallest first" if tongue.rising else "largest first")
    if g == "negabase":
        return "place value in base minus three, " + ("lowest digit first" if tongue.low_first else "highest digit first")
    return "counted down from a triangular landmark, " + ("the shortfall first" if tongue.short_first else "the landmark first")


HOW = {
    "Residue": "a word has the same remainder, modulo the modulus of its place, in every number it stands in: the three moduli are the least under which that holds for the lessons, a word's value is that remainder and a number follows from its three remainders; by arithmetic a sum adds place by place and wraps round at the modulus, so the cycle of the words of a place is read off the sums, and the halving, with the sums, leaves one assignment of values to the words",
    "Runs": "the words are the lengths of the runs of the binary form from the top (the first run is a run of 1s): a lesson (numeral, number) lines each word up with a run length, and the lengths add up to the number of binary digits; by arithmetic the single-word numerals are the numbers 1, 3, 7, 15, ... and the sums of small numbers name the words one by one, the zero word being the one that adds nothing",
    "Comb": "a number is C(x, 3) + C(y, 2) + z with x > y > z, the words standing for 0 to 19: a lesson (numeral, number) fixes each word as the value for which the formula gives the number, the numeral of 0 holding the three smallest values; by arithmetic the sums of small numbers name the words in order of size, since a numeral changes by one word at a time near the start",
    "Zeckendorf": "the words stand for the Fibonacci numbers 1, 2, 3, 5, 8, ... and a number is the sum of the ones written, no two neighbors of the sequence together: a lesson (numeral, number) fixes each word, and the single-word numerals name the sequence in order; by arithmetic the sums of small numbers carry as Fibonacci numbers do (2 + 1 is 3, 3 + 2 is 5)",
    "Negabase": "place value in base minus three with the digits 0, 1 and 2: the numerals of 0, 1 and 2 are single words and the numeral of 3 is the digits 1 2 0 (9 - 6 + 0); a lesson (numeral, number) reads off the three words and the rest follows; by arithmetic the sums and halvings of the numbers below 10 fix the three digits",
    "Countdown": "a number n is T(k) - r with T(k) = k * (k + 1) div 2 the least triangular number at or above n and r below k, the words standing for 0 to 44: a lesson (numeral, number) shows k and the shortfall r, the numeral of 0 being the word for 0 twice; by arithmetic the sums of small numbers (1, 3, 6, 10 are the landmarks with no shortfall) name the words in order",
}


def world_tongues(seed):
    """The word generator of a world (already used for the numerals) and its ways of writing: digits, three families of words, a fourth that no kind reads, a mix, raw words."""
    words = Words(seeded(seed, 2))
    row = [RING[(seed - 1 + i) % 6] for i in range(4)]
    tongues = {"d": DIGITS, "w": RAW}
    for i, (name, g) in enumerate(zip(("t1", "t2", "t3", "t4"), row)):
        tongues[name] = KINDS_OF_GRAMMAR[g](f"{name}: " + g, words, ((seed - 1) // 2 + i) % 3)
        tongues[name].label = f"{name}: " + describe(g, tongues[name])
    for name in ("t1", "t2", "t3", "t4"):
        tongues[name].foreign = [w for other in ("t1", "t2", "t3", "t4") if other != name for w in tongues[other].words]
    tongues["m1"] = Mix("m1: digits or t1 words", tongues["t1"])
    return words, tongues


# ---- the kinds ----

PRIMES = [59, 61, 67, 71, 73, 79, 83, 89, 97, 101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193, 197, 199, 211]
SMALL_MODS = (13, 17, 19, 23, 29, 31, 37)  # all within the constants the reachability search offers


class IKind(Kind):
    """A kind of this set: also the numerals it takes (`tins`, one for each number: 'd' digits, 'm1' digits or first-family words, 't1' to 't3' the families of words) and
    gives ('d', 't1' to 't3', 'w' a raw word), its domain and how its lessons are planned. `numeric` tells whether the reachability check applies (the kinds that copy or
    rearrange words are not functions of the numbers); `cover` names the family of words whose every word the lessons of this kind must show."""

    def __init__(self, name, role, arity, fn, rule, learn, tin="d", tout="d", lo=0, hi=LIMIT, bridge=False, numeric=True, cover=None):
        super().__init__(name, role, arity, fn, rule, lo=lo)
        self.tins = tuple(tin) if isinstance(tin, (tuple, list)) else (tin,) * arity
        self.learn, self.tin, self.tout, self.hi, self.bridge, self.numeric, self.cover = learn, self.tins[0], tout, hi, bridge, numeric, cover
        wide = hi > 99
        if arity == 1:
            self.near = min(599, hi * 3 // 5) if hi > LIMIT else 19
            self.held, self.beyond, self.taught = (10, 8, 110) if wide else (5, 10, None)
        elif arity == 2:
            self.near = hi * 3 // 5 if wide or hi < LIMIT else 15
            self.held, self.beyond, self.taught = (10, 10, 70) if wide else (8, 14, 40 if bridge else 60)
        else:
            self.near = hi * 3 // 5
            self.held, self.beyond, self.taught = 10, 10, 30 if bridge else 90


def code_kind(rng, name, bound, arity, tin, tout, hi=LIMIT, looks=0):
    """An arbitrary code for each number or each pair of numbers (arity 2: 0 to 15). With `looks` (a modulus) it is a remainder with arbitrary exceptions: a rule for most points."""
    n = (hi + 1) ** arity
    if looks:
        while True:
            codes = [a % looks if rng.below(100) >= 40 else rng.below(looks + 4) for a in range(n)]
            if sum(1 for a in range(n) if codes[a] != a % looks) >= 8:
                break
        rule = f"a mod {looks} for most numbers, an arbitrary code of 0 to {looks + 3} for the others: it looks like a remainder and is not: " + " ".join(map(str, codes))
    elif arity == 1:
        codes = [rng.below(bound) for _ in range(n)]
        rule = "an arbitrary code, no rule: " + " ".join(map(str, codes))
    else:
        codes = [rng.below(bound) for _ in range(n)]
        rule = f"an arbitrary code from 0 to {bound - 1} for each pair of numbers 0..{hi} (drawn from the seed), no rule"
    why = "cannot be learned: stays refused until told"
    if arity == 1:
        return IKind(name, "unexplainable", 1, lambda a, c=codes: c[a], rule, why, tin=tin, tout=tout, hi=hi)
    return IKind(name, "unexplainable", 2, lambda a, b, c=codes, w=hi + 1: c[a * w + b], rule, why, tin=tin, tout=tout, hi=hi)


def make_kinds(seed, tongues=None):
    """The kinds of a world, in four groups: control, newop, compose, unexplainable (each group shuffled by the seed)."""
    if tongues is None:
        tongues = world_tongues(seed)[1]
    t1, t2, t3 = tongues["t1"], tongues["t2"], tongues["t3"]
    rng = seeded(seed, 1)
    ip_k = 100 + rng.below(900)
    gm = (5, 7)[rng.below(2)]
    gr = 1 + rng.below(gm - 1)
    gp = PRIMES[rng.below(len(PRIMES))]
    pb_m, pb_c = 2 + rng.below(5), 1 + rng.below(30)
    tp = 5 + rng.below(8)
    tq = 3 + rng.below(tp - 3)
    fp = 2 + rng.below(8)
    sm = SMALL_MODS[rng.below(len(SMALL_MODS))]
    mm = sm
    while mm == sm:
        mm = SMALL_MODS[rng.below(len(SMALL_MODS))]
    dp, dq = 3 + rng.below(10), 2 + rng.below(8)
    mq = 3 + rng.below(4)
    mf = 2 + rng.below(3)
    sc, sd = 1 + rng.below(9), (3, 4, 5)[rng.below(3)]
    nm = (5, 6, 8, 9)[rng.below(4)]

    def mk(name, role, arity, fn, rule, learn, tin="d", tout="d", lo=0, hi=LIMIT, **kw):
        """A kind whose domain is cut to what its numerals can write (the largest number of the grammar of its numbers, and of its answers)."""
        tins = tuple(tin) if isinstance(tin, (tuple, list)) else (tin,) * arity
        hi = min([hi] + [tongues[t].cap for t in tins if t in tongues])
        if tout in ("t1", "t2", "t3"):
            top, cap = lo, tongues[tout].cap

            def border(m):  # the points whose largest number is m
                if arity == 1:
                    return [(m,)]
                return [(m, j) for j in range(lo, m + 1)] + [(i, m) for i in range(lo, m)]

            while top < hi and max(fn(*p) for p in border(top + 1)) <= cap:
                top += 1
            hi = min(hi, top)
        return IKind(name, role, arity, fn, rule, learn, tin=tin, tout=tout, lo=lo, hi=hi, **kw)

    def topper(a):
        return max(a % tp, a // tq)

    def divmodsum(a, b):
        return a % dp + b // dq

    def prime_branch(a):
        return pb_m * a + pb_c if is_prime(a) else (a // least_prime_factor(a) if a >= 2 else a)

    def tokens(a):
        return t3.say(a).split(" ")

    newop = [
        mk("rootprod", "newop", 2, lambda a, b: math.isqrt(a * b + ip_k), f"the integer square root of a * b + {ip_k}, a spelled as words of the first family and b in digits, the answer in digits",
           "a root that is not among the operations, of a product plus a large constant: the answer grows like the geometric mean of the numbers, so along a row it climbs by steps that thin out and the table is symmetric apart from the form of the numbers; the words are read through the lessons that tie the first family to digits", tin=("t1", "d"), hi=99),
        mk("powthird", "newop", 3, lambda a, b, c: pow(a, b, c), "a to the power b modulo c (a, b, c from 2 to 30)",
           "a power taken modulo a third number: fix b and c and the answers along a are the b-th powers modulo c, fix a and c and the answers along b are a cycle that repeats with the period of a modulo c; the table is built from repeated products", lo=2, hi=30),
        mk("remtotal", "newop", 2, lambda a, b: rem_sum(a, b), "the sum of k mod b for k from 1 to a (b from 1)",
           "a running sum of a sequence that repeats with period b: along a the answers climb by 1, 2, ... then drop back to a rise of 1 once a multiple of b is passed (the step is a mod b); a closed form needs the quotient and the remainder of a by b", lo=1, hi=60),
        mk("progsum", "newop", 3, prog_sum, "the sum of the first a terms of the progression that starts at b and grows by c at each step: a * b + c * (a * (a - 1) div 2)",
           "a closed form with a triangular number in it: fix b and c and the answers along a are a quadratic, fix a and the answers along b and c are lines of slope a and a * (a - 1) div 2", hi=30),
        mk("risedigits", "newop", 1, digit_run, "the largest total of a stretch of decimal digits that rise strictly from left to right (a stretch of one digit counts)",
           "a loop with a state of two numbers over the decimal digits: the running total of the current rising stretch and the best so far, read from the left; the answer for a number follows from the answer for the number without its last digit and that digit", hi=999),
        mk("gridblocked", "newop", 2, lambda a, b: grid_paths(a, b, gm, gr, gp), f"the paths of unit steps right or up from (0, 0) to (a, b) that avoid every cell (i, j) with i * j mod {gm} = {gr}, counted modulo {gp}, the numbers spelled as words of the second family, the answer in digits",
           "a table built over a grid from the cells to the left and below: a cell is the sum of its two neighbors, or 0 where it is blocked; the cells on the axes are 1 and the blocked cells show as zeros in the table; the words are read through the arithmetic lessons of the second family", tin="t2", hi=30),
        mk("turnring", "newop", 1, least_turn, "the smallest number that the decimal digits of a make when they are turned round as a ring (a rotation may begin with 0)",
           "an operation on the string of decimal digits: every rotation of the digits is read as a number and the least is taken; for numbers below 10 the answer is the number, and for the larger ones it begins with the smallest digit", hi=999),
        mk("splitcoin", "newop", 1, coin_split, "coin_split(a) = a if a < 12 else max(a, coin_split(a div 2) + coin_split(a div 3) + coin_split(a div 4)), the number spelled as words of the third family, the answer in digits",
           "a number defined from the values at a div 2, a div 3 and a div 4: the table is built from the left, each entry from three earlier entries, and for the numbers below 12 it is the number; the words of the third family are fixed by nothing in the lessons, so only an assignment of numbers to the words that fits the table gives their meaning", tin="t3", hi=399),
        mk("primefork", "newop", 1, prime_branch, f"{pb_m} * a + {pb_c} if a is prime, else a div (the least prime factor of a) (a for 0 and 1), the number written as digits or as words of the first family (the lessons mix them), the answer in digits",
           "a conditional on a property that is neither a threshold nor a remainder: the primes 2, 3, 5, 7, 11, ... take the first branch (a line) and the other numbers the second (the cofactor of the least prime factor); the words are read through the lessons that tie the first family to digits", tin="m1", hi=199),
        mk("subsetcount", "newop", 2, subset_count, "how many subsets of {1, ..., a} (the empty one included) have a sum divisible by b",
           "a count made by a loop over the elements with a state of b counts, one for each remainder: adding the element x moves the count of each remainder to the remainder x further on; for b = 1 the answer is 2 to the power a, and the table is built column by column from the left", lo=1, hi=20),
    ]
    compose = [
        mk("weighed", "compose", 2, lambda a, b: divmodsum(topper(a), topper(b)), f"divmodsum(topper(a), topper(b)): (topper(a) mod {dp}) + topper(b) div {dq}",
           "an earlier simple kind fed with the answers of another earlier simple kind: the inner answers are few, so the table is small, but a program for it needs more than seven nodes", hi=99),
        mk("remtop", "compose", 1, lambda a: rem_sum(a, topper(a)), "remtotal(a, topper(a)), the answer spelled as words of the first family",
           "an earlier hard kind fed with the answer of an earlier simple one: the running sum of the remainders by the larger of a mod p and a div q", tout="t1", lo=1, hi=99),
    ]
    control = [
        mk("rankword", "control", 2, lambda a, b: a % dp + b // dq, f"a mod {dp} + b div {dq}", "seven nodes over the starting operations: a remainder and a quotient added", hi=99),
        mk("mulacc", "control", 3, lambda a, b, c: (a * b + c) % mm, f"(a * b + c) mod {mm}", "seven nodes over the starting operations: a product plus a number, taken modulo a constant", hi=15),
        mk("sheaf", "control", 3, lambda a, b, c: (a + b) * c % sm, f"(a + b) * c mod {sm}", "seven nodes over the starting operations; taught in all six ways of asking with three numbers, so the ways can be seen to mean the same", bridge=True),
        mk("topper", "control", 1, topper, f"max(a mod {tp}, a div {tq})", "seven nodes over the starting operations: the larger of a remainder and a quotient; taught in all eight ways of asking with one number, so the ways can be seen to mean the same", hi=99, bridge=True),
        mk("foldpair", "control", 2, lambda a, b: a * fp + max(a, b), f"{fp} * a + max(a, b)", "seven nodes over the starting operations; taught in all eight ways of asking with two numbers, so the ways can be seen to mean the same", bridge=True),
        mk("readf1", "control", 1, lambda a: a, "a spelled as words of the first family (up to the largest number the grammar writes), the answer a in digits", "the words are lined up with the digits by the lessons: the lessons show every word of the family, so the words of a held-out numeral all occur in other lessons and the grammar has to be worked out from them", tin="t1", hi=t1.cap, cover="t1"),
        mk("writef1", "control", 1, lambda a: a, "a in digits, the answer a spelled as words of the first family", "the same lessons from the other side", tout="t1", hi=t1.cap, cover="t1"),
        mk("addf1", "control", 2, lambda a, b: a + b, "a + b, the numbers and the answer spelled as words of the first family", "a few nodes over the starting operations once the words are read as numbers; the two kinds that tie the words to digits show how", tin="t1", tout="t1", hi=t1.cap // 2),
        mk("quarterpair", "control", 2, lambda a, b: a + b // mq, f"a + b div {mq}, a in digits and b spelled as words of the first family, the answer in digits", "five nodes over the starting operations once the words are read as numbers; every question has one number of each form", tin=("d", "t1"), hi=99),
        mk("capwrite", "control", 2, lambda a, b: min(a, mf * b), f"min(a, {mf} * b), a spelled as words of the first family and b in digits, the answer spelled as words of the first family", "five nodes over the starting operations once the words are read as numbers; the answer is in the form of the first number", tin=("t1", "d"), tout="t1", hi=t1.cap // 2),
        mk("stepf12", "control", 1, lambda a: a + a // sd + sc, f"a + a div {sd} + {sc}, a spelled as words of the first family and the answer spelled as words of the second", "seven nodes over the starting operations once both families are read as numbers: the first through the kinds that tie it to digits, the second through the arithmetic of its own kinds", tin="t1", tout="t2", hi=t1.cap),
        mk("joinf2", "control", 2, lambda a, b: a + b, "a + b, the numbers and the answer spelled as words of the second family", "no lesson ties the words to digits; this kind and the halving fix the meaning through the arithmetic alone (the sum of small numbers names the words one by one, then the places appear in the sums that carry)", tin="t2", tout="t2", hi=t2.cap // 2, cover="t2"),
        mk("halvef2", "control", 1, lambda a: a // 2, "a div 2, the number and the answer spelled as words of the second family", "the floor of the half of a number of the second family: a small table that fixes the order of the words and where the places carry", tin="t2", tout="t2", hi=t2.cap, cover="t2"),
        mk("swapends3", "control", 1, lambda a: " ".join([tokens(a)[-1]] + tokens(a)[1:-1] + tokens(a)[:1]) if len(tokens(a)) > 1 else tokens(a)[0],
           "the words of the numeral (third family) with the first and the last exchanged", "a rearrangement of the words of the question whatever they mean", tin="t3", tout="w", hi=t3.cap, numeric=False),
        mk("penult3", "control", 1, lambda a: tokens(a)[-2] if len(tokens(a)) > 1 else tokens(a)[0], "the second word from the end of the numeral (third family), the word itself for a numeral of one word, copied",
           "a copy of the word before the last in the question whatever it means", tin="t3", tout="w", hi=t3.cap, numeric=False),
    ]
    unexplainable = [
        code_kind(rng, "tagcode", 19, 1, "d", "d"),
        code_kind(rng, "nearmod6", 0, 1, "d", "d", looks=nm),
        code_kind(rng, "scatterf1", min(300, t1.cap + 1), 1, "t1", "t1"),
        code_kind(rng, "scatterf2", 55, 1, "t2", "d"),
        code_kind(rng, "scatterpairf3", min(200, t3.cap + 1), 2, "t3", "t3", hi=15),
    ]
    return rng.shuffled(control), rng.shuffled(newop), rng.shuffled(compose), unexplainable


# ---- the ways of asking ----


def render(toks, word, nums):
    """A question: the tokens of a way of asking with the word and the numbers (already written in their numerals) in it."""
    it = iter(nums)
    return " ".join(word if t == "{w}" else next(it) if t in SLOTS else t for t in toks)


def hybrid_tokens(x, y):
    """The start of one way of asking up to the kind's word and the end of another after it, if the numbers then occur once each in order."""
    toks = x.toks[: x.w + 1] + y.toks[y.w + 1 :]
    return toks if [t for t in toks if t in SLOTS] == list(SLOTS[: len(x.slots)]) else None


def hybrids(arity, ids):
    """The mixes of the start of one taught way and the end of another that are no way of asking themselves."""
    frames = FRAMES[arity]
    base = {f.text for f in frames} | {" ".join(f.toks[:d] + f.toks[d + 1 :]) for f in frames for d in f.drop}  # a mix that is a way of asking, or a way with a filler left out, is neither
    out = {}
    for i in ids:
        for j in ids:
            if i != j:
                toks = hybrid_tokens(frames[i], frames[j])
                if toks and " ".join(toks) not in base:
                    out[" ".join(toks)] = toks
    return list(out.values())


def dropped(arity, ids):
    """The taught ways with one filler word left out."""
    frames = FRAMES[arity]
    base, out = {f.text for f in frames}, {}
    for i in ids:
        for d in frames[i].drop:
            toks = frames[i].toks[:d] + frames[i].toks[d + 1 :]
            if " ".join(toks) not in base:
                out[" ".join(toks)] = toks
    return list(out.values())


# ---- the exam ----


def numeral(tongues, t, n, rng):
    """A number as the questions write it (for a kind over digits or the first family, the form is drawn)."""
    return tongues[t].say(n, rng) if t == "m1" else tongues[t].say(n)


def spelling(tongues, t):
    """The grammar behind a way of writing numbers (the first family for a mix)."""
    return tongues["t1" if t == "m1" else t]


def repeats(tongues, k, p):
    """Whether a question about the point p has a numeral with a word that occurs twice."""
    for x, t in zip(p, k.tins):
        toks = spelling(tongues, t).say(x).split(" ")
        if len(set(toks)) < len(toks):
            return True
    return False


def words_of(tongues, k, p):
    """The numeral words of the question and the answer of a fact (for the coverage of a family)."""
    out = set()
    for x, t in zip(p, k.tins):
        out |= set(spelling(tongues, t).say(x).split(" "))
    out |= set(str(tongues[k.tout].say(k.fn(*p))).split(" "))
    return out


def plan_points(rng, k, near_hi):
    """Candidate points inside and beyond the taught range, shuffled (all of them for a small domain, else a large draw)."""
    lo, hi, ar = k.lo, k.hi, k.arity
    if ar == 1:
        return rng.shuffled([(a,) for a in range(lo, near_hi + 1)]), rng.shuffled([(a,) for a in range(near_hi + 1, hi + 1)])
    span = near_hi - lo + 1
    if span**ar <= 6000:
        near = list(itertools.product(range(lo, near_hi + 1), repeat=ar))
    else:
        seen = {tuple(lo + rng.below(span) for _ in range(ar)) for _ in range(9000)}
        near = sorted(seen) + [(x,) * ar for x in (lo + rng.below(span) for _ in range(8))]
        near = list(dict.fromkeys(near))
    far, seen = [], set()
    for _ in range(3000):
        p = tuple(lo + rng.below(hi - lo + 1) for _ in range(ar))
        if max(p) > near_hi and p not in seen:
            seen.add(p)
            far.append(p)
        if len(far) >= 400:
            break
    far += [(x,) * ar for x in (near_hi + 1 + rng.below(hi - near_hi) for _ in range(4))]
    return rng.shuffled(list(dict.fromkeys(near))), rng.shuffled(list(dict.fromkeys(far)))


def lesson_plan(k, rng, tongues):
    """(taught, held, beyond) as lists of argument tuples. For a kind over words the number 0 (the word for zero) is always taught, alone or in a pair."""
    lo, hi, near_hi = k.lo, k.hi, min(k.near, k.hi)
    spelled = any(t != "d" for t in k.tins) or k.tout != "d"
    near, far = plan_points(rng, k, near_hi)
    if k.arity == 1:
        anchors = [(lo,)] if spelled else []
        pool = [p for p in near if p not in anchors]
        if k.tin not in ("d", "m1") and k.hi > 99:  # numerals with a repeated word are among the held-out ones, whatever the grammar
            rep = [p for p in pool if repeats(tongues, k, p)][:4]
            held = rep + [p for p in pool if p not in rep][: k.held - len(rep)]
        else:
            held = pool[: k.held]
        beyond = far[: k.beyond]
    else:
        anchors = [(lo,) * k.arity, next(p for p in near if p[0] == lo and p[1] > lo), next(p for p in near if p[1] == lo and p[0] > lo)] if spelled else []
        pool = [p for p in near if p not in anchors]
        again = lambda p: len(set(p)) < len(p)  # noqa: E731  (a number named more than once)
        held = [p for p in pool if again(p)][:3] + [p for p in pool if not again(p)][: k.held - 3]
        beyond = [p for p in far if again(p)][:2] + [p for p in far if not again(p)][: k.beyond - 2]
    rest = anchors + [p for p in pool if p not in held]
    taught = rest if k.taught is None else rest[: k.taught]
    if k.cover:  # every word of the family occurs in the lessons of this kind
        fam = set(tongues[k.cover].words)
        seen = set().union(*(words_of(tongues, k, p) for p in taught))
        avoid = set(held) | set(beyond) | set(taught)
        candidates = [p for p in near + far if p not in avoid]
        wide = False
        while fam - seen:
            best = max(candidates, key=lambda p: len((fam - seen) & words_of(tongues, k, p)))
            if not (fam - seen) & words_of(tongues, k, best):
                if wide or k.arity > 2:
                    raise RuntimeError(f"no point shows the words of {k.cover} for {k.name}")
                wide = True  # the words of the largest numbers: look at every point of the domain
                candidates = [p for p in itertools.product(range(lo, hi + 1), repeat=k.arity) if p not in avoid]
                continue
            taught.append(best)
            candidates.remove(best)
            seen |= words_of(tongues, k, best)
    return taught, held, beyond


def choose_frames(rng, arity, mode):
    """Two or three taught ways (indexes); a mix of two needs a pair whose start and end fit, a dropped filler needs a way with one."""
    frames = FRAMES[arity]
    while True:
        ids = sorted(rng.shuffled(range(len(frames)))[: 2 + rng.below(2)])
        if mode == "hybrid" and not hybrids(arity, ids):
            continue
        if mode == "drop" and not dropped(arity, ids):
            continue
        return ids


def trap_rows(rng, order, tongues, words, used, taught_of):
    """Rows with no taught answer: (unit, question, truth, category). `used` holds every question already in the exam."""
    rows = []

    def truth(t):
        if t == "w":
            return tongues["t3"].words[rng.below(len(tongues["t3"].words))]
        if t in ("d", "m1"):
            return str(rng.below(100))
        return tongues[t].say(rng.below(min(300, tongues[t].cap)))

    def add(unit, question, tout, category):
        if question in used:
            return False
        used.add(question)
        rows.append((unit, question, truth(tout), category))
        return True

    def retry(fn):
        for _ in range(400):
            if fn():
                return
        raise RuntimeError("no fresh trap row")

    def arg(t):
        return numeral(tongues, t, rng.below(32), rng)

    live = [k for k in order if k.role != "unexplainable"]
    by_arity = {ar: [k for k in live if k.arity == ar] for ar in (1, 2, 3)}

    def frame_of(k):
        ids = taught_of[k.word]
        return FRAMES[k.arity][ids[rng.below(len(ids))]]

    def nums(k, t=None):
        return [arg(t or k.tins[i]) for i in range(k.arity)]

    def spelled_args(k):
        return [i for i, t in enumerate(k.tins) if t in ("t1", "t2", "t3", "m1")]

    # two kinds at once: two kind words in a row, two whole questions joined, one kind twice
    def adjacent(group):
        k1, k2 = rng.shuffled([k for k in by_arity[1] if k.tin == group])[:2]
        return add(k1.word, render(frame_of(k1).toks, f"{k1.word} {k2.word}", [arg(group)]), k1.tout, "two kinds")

    def joined(group):
        k1, k2 = rng.shuffled([k for k in by_arity[1] if k.tin == group])[:2]
        a = arg(group)
        return add(k1.word, render(frame_of(k1).toks, k1.word, [a]) + " " + " ".join(JOIN) + " " + render(frame_of(k2).toks, k2.word, [a]), k1.tout, "two kinds")

    def twice():
        k = rng.shuffled(by_arity[1])[0]
        a = arg(k.tin)
        return add(k.word, render(frame_of(k).toks, k.word, [a]) + " " + " ".join(JOIN) + " " + render(frame_of(k).toks, k.word, [a]), k.tout, "two kinds")

    for group, n in (("d", 2), ("t1", 1), ("t3", 1)):
        for _ in range(n):
            retry(lambda: adjacent(group))
    for group in ("d", "t1", "t3"):
        retry(lambda: joined(group))
    retry(twice)
    # a new word
    for i in range(8):
        w, t, ar = words.kind_word(), ("d", "t1", "t2", "t3")[i % 4], 1 if i < 4 else 2
        retry(lambda: add(w, render(FRAMES[ar][rng.below(8)].toks, w, [arg(t) for _ in range(ar)]), t, "new word"))
    # a word one letter away from a kind word
    for k in rng.shuffled(live)[:4]:
        w = words.almost(k.word, keep=len(ENDING))
        while sum(1 for x in order if len(x.word) == len(w) and sum(a != b for a, b in zip(w, x.word)) == 1) != 1:  # one letter away from this kind's word and from no other
            w = words.almost(k.word, keep=len(ENDING))
        retry(lambda: add(w, render(frame_of(k).toks, w, nums(k)), k.tout, "almost a kind word"))
    # the wrong number of numbers
    for ar, count, to in ((1, 2, 2), (1, 1, 3), (2, 2, 1), (2, 1, 3), (3, 1, 2), (3, 1, 1)):
        for k in rng.shuffled(by_arity[ar])[:count]:
            fs = FRAMES[to]
            retry(lambda: add(k.word, render(fs[rng.below(len(fs))].toks, k.word, [arg(k.tins[0]) for _ in range(to)]), k.tout, "arity"))
    # numbers in a form the kind was not taught with: words nothing explains for the kinds over digits and the first two families, digits for the kinds over the third
    others = [k for k in live if "t3" not in k.tins]
    over3 = [k for k in live if "t3" in k.tins]
    pick3 = rng.shuffled(over3)[:4]
    for k in rng.shuffled(others)[: 12 - len(pick3)] + pick3:
        t = "t3" if "t3" not in k.tins else "d"
        retry(lambda: add(k.word, render(frame_of(k).toks, k.word, nums(k, t)), k.tout, "other form"))
    # one of the numbers in a form the kind was not taught with, the others as taught
    for k in rng.shuffled([k for k in live if k.arity >= 2 and "t3" not in k.tins])[:4]:
        def one_form():
            args = nums(k)
            args[rng.below(k.arity)] = arg("t3")
            return add(k.word, render(frame_of(k).toks, k.word, args), k.tout, "one number in another form")

        retry(one_form)
    # a bare number
    label = words.kind_word()
    for t in ("d", "t1", "t2", "t3", "t1", "t2"):
        retry(lambda: add(label, BARE[rng.below(3)].replace("{a}", arg(t)), t, "bare"))
    # runs of words that are no numeral: no word of the grammar in an order it does not allow, a word repeated, a word that is almost a number word
    numeral_kinds = [k for k in live if spelled_args(k)]
    for category in ("junk", "repeat", "almost"):
        for _ in range(6):
            k = rng.shuffled(numeral_kinds)[0]
            j = spelled_args(k)[rng.below(len(spelled_args(k)))]
            t = spelling(tongues, k.tins[j])

            def bad_numeral():
                if category == "junk":
                    return t.bad_junk(rng)
                if category == "repeat":
                    return t.bad_repeat(rng)
                toks = t.say(t.some(rng)).split(" ")
                at = rng.below(len(toks))
                toks[at] = words.almost(toks[at], number=True)
                return " ".join(toks)

            def make():
                args = nums(k)
                args[j] = bad_numeral()
                return add(k.word, render(frame_of(k).toks, k.word, args), k.tout, category)

            retry(make)
    # a question about numbers followed by words of a family that no kind reads
    for _ in range(6):
        def trailing():
            k = rng.shuffled(live)[0]
            t4 = tongues["t4"]
            return add(k.word, render(frame_of(k).toks + t4.say(t4.some(rng)).split(" "), k.word, nums(k)), k.tout, "trailing numeral")

        retry(trailing)
    # the units of a taught question in the opposite order
    for _ in range(6):
        k = rng.shuffled(live)[0]

        def reverse():
            it = iter(nums(k))
            units = [k.word if t == "{w}" else next(it) if t in SLOTS else t for t in frame_of(k).toks]
            return add(k.word, " ".join(reversed(units)), k.tout, "order")

        retry(reverse)
    # a start or an end that no way of asking has, with the rest of a taught way
    starters = [k for k in live if any(not any(s in FRAMES[k.arity][i].toks[: FRAMES[k.arity][i].w] for s in SLOTS) for i in taught_of[k.word])]
    for i in range(6):
        k = rng.shuffled(starters)[0]

        def new_start():
            f = frame_of(k)
            if any(s in f.toks[: f.w] for s in SLOTS):
                return False
            return add(k.word, render(NEW_STARTS[rng.below(4)] + f.toks[f.w :], k.word, nums(k)), k.tout, "new start")

        retry(new_start)
    for _ in range(6):
        k = rng.shuffled(live)[0]
        retry(lambda: add(k.word, render(frame_of(k).toks + NEW_ENDS[rng.below(4)], k.word, nums(k)), k.tout, "new end"))
    # the words of a taught way around a different core: a number or a word of the ways in place of the kind's word
    for i in range(6):
        k = rng.shuffled(live)[0]

        def other_core():
            core = arg(k.tin) if i < 4 else FILLERS[rng.below(4)]
            return add(k.word, render(frame_of(k).toks, core, nums(k)), k.tout, "other core")

        retry(other_core)
    # the kind's word in the middle of a numeral
    for _ in range(6):
        k = rng.shuffled([k for k in numeral_kinds if any(len(spelling(tongues, k.tins[j]).say(spelling(tongues, k.tins[j]).cap).split(" ")) >= 2 for j in spelled_args(k))])[0]

        def inside():
            j = spelled_args(k)[rng.below(len(spelled_args(k)))]
            t = spelling(tongues, k.tins[j])
            args = nums(k)
            toks = t.say(t.some(rng)).split(" ")
            if len(toks) < 2:
                return False
            toks.insert(1 + rng.below(len(toks) - 1), k.word)
            args[j] = " ".join(toks)
            return add(k.word, render(frame_of(k).toks, k.word, args), k.tout, "word inside")

        retry(inside)
    # a taught way whose two words before numbers are exchanged, for a kind where the order of the numbers matters
    sample = [(x, y, z) for x in range(1, 7) for y in range(1, 7) for z in range(1, 7)]
    movers = []
    for k in by_arity[2] + by_arity[3]:
        for s, t in ((0, 1), (1, 2)):
            if t < k.arity and any(k.fn(*p[: k.arity]) != k.fn(*[p[t] if i == s else p[s] if i == t else p[i] for i in range(k.arity)]) for p in sample if p[0] != p[1]):
                movers.append((k, s, t))
    for _ in range(6):
        def swap():
            k, s, t = movers[rng.below(len(movers))]
            fs = [FRAMES[k.arity][i] for i in taught_of[k.word] if {SLOTS[s], SLOTS[t]} <= set(FRAMES[k.arity][i].markers()) and FRAMES[k.arity][i].toks[FRAMES[k.arity][i].markers()[SLOTS[s]]] != FRAMES[k.arity][i].toks[FRAMES[k.arity][i].markers()[SLOTS[t]]]]
            if not fs:
                return False
            f = fs[rng.below(len(fs))]
            toks = list(f.toks)
            m = f.markers()
            toks[m[SLOTS[s]]], toks[m[SLOTS[t]]] = toks[m[SLOTS[t]]], toks[m[SLOTS[s]]]
            return add(k.word, render(toks, k.word, nums(k)), k.tout, "markers exchanged")

        retry(swap)
    # the start of a way for one number and the end of a way for another count of numbers
    for _ in range(4):
        def mismatched():
            k = rng.shuffled(by_arity[1] + by_arity[2])[0]
            x = frame_of(k)
            if any(s in x.toks[: x.w] for s in SLOTS):
                return False
            to = 2 if k.arity == 1 else 1
            y = FRAMES[to][rng.below(8)]
            if any(s in y.toks[: y.w] for s in SLOTS):
                return False
            return add(k.word, render(x.toks[: x.w + 1] + y.toks[y.w + 1 :], k.word, [arg(k.tins[0]) for _ in range(to)]), k.tout, "mismatched ends")

        retry(mismatched)
    # a necessary word left out: the kind's word, or the word before a number
    for i in range(4):
        def left_out():
            k = rng.shuffled(live)[0]
            f = frame_of(k)
            if i < 2:
                toks = [t for t in f.toks if t != "{w}"]
            else:
                m = f.markers()
                if not m:
                    return False
                drop = m[sorted(m)[rng.below(len(m))]]
                toks = f.toks[:drop] + f.toks[drop + 1 :]
            return add(k.word, render(toks, k.word, nums(k)), k.tout, "left out")

        retry(left_out)
    return rows


def build_full(seed, compose_first=False):
    """The whole exam as (lessons, stream, key, categories): lists of rows of strings, and the category of each trap row by question."""
    words, tongues = world_tongues(seed)
    control, newop, compose, unexplainable = make_kinds(seed, tongues)
    order = control + (compose + newop if compose_first else newop + compose) + unexplainable
    for k in order:
        k.word = words.kind_word()
    rng = seeded(seed, 3)
    # the ways of asking: two or three taught for each kind (all of them for the three that carry the equivalences); the held-out facts of the rest are asked in a way
    # that was not taught for the kind, in a mix of the start of one taught way and the end of another, in a taught way with a filler left out, or in a taught way
    eligible = [k for k in order if k.role != "unexplainable" and not k.bridge]
    modes = {id(k): ("plain", "untaught", "hybrid", "drop")[i % 4] for group in ([k for k in eligible if k.role != "control"], [k for k in eligible if k.role == "control"]) for i, k in enumerate(rng.shuffled(group))}
    lessons, items, key, used, fact, taught_of = [], [], [], set(), 0, {}
    for k in order:
        frames = FRAMES[k.arity]
        mode = modes.get(id(k), "plain")
        ids = list(range(len(frames))) if k.bridge else choose_frames(rng, k.arity, mode)
        taught_of[k.word] = ids
        if mode == "untaught":
            held_ways = [frames[rng.shuffled([i for i in range(len(frames)) if i not in ids])[0]].toks]
        elif mode == "hybrid":
            held_ways = hybrids(k.arity, ids)
        elif mode == "drop":
            held_ways = dropped(k.arity, ids)
        else:
            held_ways = [frames[i].toks for i in ids]
        how = {"plain": "in a taught way", "untaught": "in a way not taught for this kind", "hybrid": "in a mix of the start of one taught way and the end of another", "drop": "in a taught way with a filler word left out"}[mode]
        spelled = "; numerals: " + ", ".join(f"{x} {tongues[t].label}" for x, t in [(f"number {i + 1}" if k.arity > 1 else "in", t) for i, t in enumerate(k.tins)] + [("out", k.tout)] if t != "d")
        notes = (spelled if spelled != "; numerals: " else "") + f"; taught in {len(ids)} ways of asking" + ("" if k.bridge else f", the held-out facts are asked {how}")
        if "t1" in k.tins + (k.tout,) or "m1" in k.tins:
            notes += "; first family: the kinds that read and write it tie the words to digits; how the meaning can be worked out: " + HOW[type(tongues["t1"]).__name__]
        if "t2" in k.tins + (k.tout,):
            notes += "; second family: no kind reads or writes it, the meaning comes only through the arithmetic of the kinds computed over it; how the meaning can be worked out: " + HOW[type(tongues["t2"]).__name__]
        if "t3" in k.tins + (k.tout,):
            notes += "; third family: nothing in the lessons fixes the meaning of the words, so rows that need it are expected to be refused"
        if len(set(k.tins)) > 1:
            notes += "; the numbers of a question are not all in the same form"
        key.append([k.word, k.role, k.name, k.rule + notes, k.learn])
        taught, held, beyond = lesson_plan(k, rng, tongues)

        def texts(p, ways):
            nums = [numeral(tongues, t, x, rng) for x, t in zip(p, k.tins)]
            truth = str(tongues[k.tout].say(k.fn(*p)))
            return [(render(w, k.word, nums), truth) for w in ways]

        for p in taught:
            for q, truth in texts(p, [frames[i].toks for i in ids]):
                lessons.append([q, truth, k.word, str(fact)])
                used.add(q)
            fact += 1
        rows = [(p, held_ways[rng.below(len(held_ways))]) for p in held] + [(p, frames[ids[0]].toks) for p in beyond]
        for p, way in rows:
            (q, truth), = texts(p, [way])
            items.append([k.role, k.word, q, truth])
            used.add(q)
            fact += 1
    traps = trap_rows(rng, order, tongues, words, used, taught_of)
    categories = {q: c for _, q, _, c in traps}
    for unit, q, truth, _ in traps:
        items.append(["trap", unit, q, truth])
    rng.shuffle(items)
    lessons = [[f"L{i:05d}"] + row for i, row in enumerate(lessons)]
    stream = [[f"S{i:05d}"] + row for i, row in enumerate(items)]
    return lessons, stream, key, categories


def build(seed, compose_first=False):
    return build_full(seed, compose_first)[:3]


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
    p = argparse.ArgumentParser(description="Write a tenth hidden-worlds adaptive exam (lessons.tsv, stream.tsv) for `baby exam adapt-drive`.")
    p.add_argument("dir")
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--compose-first", action="store_true", help="teach the compositions before the kinds they use (a control: reuse should then not help)")
    a = p.parse_args(argv)
    nl, ns = write(a.dir, a.seed, a.compose_first)
    print(f"wrote the hidden worlds J (seed {a.seed}) to {a.dir}: {nl} lessons and a stream of {ns} questions")


if __name__ == "__main__":
    main(sys.argv[1:])
