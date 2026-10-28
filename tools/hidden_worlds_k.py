#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""An eleventh set of exam worlds: closed forms of new shapes with one, two and three constants, rules that run a loop over digits, over a grid, over strings or over the
divisors of a number, a branch on a property of the number, numbers spelled in six numeral grammars whose structures are not those of the earlier sets, numbers whose forms
are mixed inside a question, new ways of asking built from pieces of each other (with evidence in the lessons for which words of a way can be left out), and traps that
look like the rewordings.

The same files, line protocol and shape of world as `tools/hidden_worlds.py` and its nine successors (so `baby exam adapt-drive DIR COMMAND...` judges any
system on it). To keep the set a blind test the kinds are not described here.

    python3 tools/hidden_worlds_k.py DIR [--seed N] [--compose-first]

What is new in the way the exam is put:
  numerals   some kinds take and give their numbers as words, in one of six grammars (chosen by the seed; each world uses three of them, one per family of
             words, and a fourth family that no kind reads). In the first family a pair of kinds ties the words to digits, so their meaning can be worked out; in
             the second no kind reads or writes the words and the meaning can only come through arithmetic done on them; in the third nothing in the lessons can
             fix the meaning. A learner is never told that the words are numbers.
  mixed      kinds that take one number as digits and the other as words, kinds whose lessons draw the form of each number, kinds whose answer is in words of
             one family for a question in digits, kinds whose answer is in digits for a question in words, a kind that answers in the words of another family than
             the question is in, and kinds taught in one form of numbers and asked in the other.
  wordings   every kind is taught in two or three of several ways of asking (three kinds in all of them, each also with every droppable filler left out, which is
             the evidence for which words of a way can go). The facts that are held out are then asked in a way that was not taught for the kind, in a mix of the
             start of one taught way and the end of another, in a taught way with a filler word dropped, or in a taught way; next to them stand traps that look
             just like these rewordings and are not answerable.
  arguments  kinds with one, two and three numbers; questions that name the same number more than once, numerals with a word that occurs more than once, and
             ways of asking in which a word, or the number itself, occurs more than once.
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

README = """THE HIDDEN WORLDS K (adaptive exam; the same files and protocol as `baby exam adapt-write`, written by tools/hidden_worlds_k.py)

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

ENDING = "pesu"  # shared by every kind word of this set (the earlier sets' words end in `sido`, `velu`, `ketu`, `ruvo`, `zane`, `yarn`, `tole`, `dopa`, `cuwe` and `bimo`)


def seeded(seed, salt):
    """A generator for a seed and a purpose (the seed goes through one extra mix first, so that neighboring seeds are not the same stream a few steps on)."""
    return Rng(Rng(seed * 6000011 + salt * 250037 + 91).next())


# ---- the operations ----


@lru_cache(maxsize=None)
def ordered_factorizations(n):
    """In how many ways n is a product of numbers above 1, order counted (n from 1; 1 is the empty product): the sum of the values at the divisors below n."""
    return 1 if n == 1 else sum(ordered_factorizations(d) for d in range(1, n) if n % d == 0)


def is_square(a):
    return math.isqrt(a) ** 2 == a


def digit_walk(a):
    """A walk on a grid told by the decimal digits: start at (0, 0) facing north; for each digit d from the left turn right d quarter turns, then step d cells forward.
    The answer is how far the end is from the start along the grid lines (|x| + |y|)."""
    x = y = h = 0
    for ch in str(a):
        d = int(ch)
        h = (h + d) % 4
        dx, dy = ((0, 1), (1, 0), (0, -1), (-1, 0))[h]
        x, y = x + d * dx, y + d * dy
    return abs(x) + abs(y)


def text_rank(a, b):
    """Where a stands among the numbers 1 to b when they are put in the order of the alphabet (as strings of decimal digits), counting the numbers that come at or before it (a need not be in 1 to b)."""
    return sum(1 for x in range(1, b + 1) if str(x) <= str(a))


def under_line(a, b):
    """The points (x, y) of the grid with x and y at least 1 that lie strictly under the straight line from (a, 0) to (0, b)."""
    return sum(1 for x in range(1, a) for y in range(1, b) if x * b + y * a < a * b)


# ---- the words ----

CONSONANTS = "bdghklmnprstvz"
VOWELS = "aeiou"
SONORANTS = "lmnrs"
NUMBER_FIRST = "w"  # every number word of this set begins with it, then a vowel, a consonant, a vowel and a consonant


class Frame:
    """A way of asking: tokens with {w} for the kind's word and {a}, {b}, {c} for the numbers. A word in [brackets] is a filler that can be left out. A number may occur twice."""

    def __init__(self, text):
        self.toks, self.drop = [], []
        for t in text.split(" "):
            if t.startswith("["):
                self.drop.append(len(self.toks))
                t = t[1:-1]
            self.toks.append(t)
        self.text = " ".join(self.toks)
        self.w = self.toks.index("{w}")
        self.slots = list(dict.fromkeys(t for t in self.toks if t in SLOTS))

    def markers(self):
        """{slot: index of the word just before the first place of the slot} for the numbers that follow a word of the frame."""
        out = {}
        for i, t in enumerate(self.toks):
            if t in SLOTS and t not in out and i > 0 and self.toks[i - 1] not in SLOTS and self.toks[i - 1] != "{w}":
                out[t] = i - 1
        return out


SLOTS = ("{a}", "{b}", "{c}")
FRAMES = {
    1: [Frame(t) for t in ["weigh {w} on {a} [swiftly]", "{a} is relayed through {w} [again]", "so how exactly does {w} treat {a}", "take {a} and let {w} work on {a}", "beneath {w} the {a} is altered [exactly]", "then {w} accepts {a} [slowly] and hands it back changed", "{a} goes in then {w} works then the result comes out", "[truly] affirm {w} for the value {a}"]],
    2: [Frame(t) for t in ["pair {w} with {a} and {b} [swiftly]", "{a} then {b} are relayed through {w} [again]", "so how exactly does {w} treat {a} against {b}", "take {a} and take {b} and let {w} work on both", "beneath {w} the {a} meets the {b} and both are altered [exactly]", "then {w} accepts {a} beside {b} [slowly]", "{a} goes in then {b} goes in then {w} works", "[truly] affirm {w} for the left {a} and the right {b}"]],
    3: [Frame(t) for t in ["pair {w} with {a} and {b} and {c} [swiftly]", "{a} then {b} then {c} are relayed through {w} [again]", "so how exactly does {w} treat {a} against {b} against {c}", "take {a} and take {b} and take {c} and let {w} work on all", "then {w} accepts {a} beside {b} beside {c} [slowly]", "[truly] affirm {w} for the left {a} the middle {b} and the right {c}"]],
}
# a start or an end that no way of asking has, for any kind
NEW_STARTS = [["suppose"], ["imagine", "now"], ["hereby"], ["quickly"]]
NEW_ENDS = [["regardless"], ["at", "dawn"], ["so", "it", "seems"], ["or", "not"]]
JOIN = ["moreover", "besides"]
BARE = ["{a}", "only {a}", "regarding {a}"]
FRAME_WORDS = {t for fs in FRAMES.values() for f in fs for t in f.toks if t not in SLOTS and t != "{w}"}
FRAME_WORDS |= {t for ws in NEW_STARTS + NEW_ENDS + [JOIN] for t in ws} | {t for b in BARE for t in b.split(" ") if t not in SLOTS}
FILLERS = ["on", "through", "with", "beside"]  # words of the frames that stand in for the kind's word in a trap


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
        return self.fresh(lambda r: "".join([CONSONANTS[r.below(14)], VOWELS[r.below(5)], SONORANTS[r.below(5)], CONSONANTS[r.below(14)], VOWELS[r.below(5)], ENDING]))

    def number_word(self):
        return self.fresh(lambda r: NUMBER_FIRST + VOWELS[r.below(5)] + CONSONANTS[r.below(14)] + VOWELS[r.below(5)] + CONSONANTS[r.below(14)])

    def almost(self, word, keep=0, number=False):
        """A word one letter away from `word` (a vowel for a vowel, a consonant for a consonant, none of the last `keep` letters, not the first letter of a number word)
        that is no word of the exam."""
        while True:
            i = (1 if number else 0) + self.rng.below(len(word) - keep - (1 if number else 0))
            pool = VOWELS if word[i] in VOWELS else CONSONANTS
            w = word[:i] + pool[self.rng.below(len(pool))] + word[i + 1 :]
            if w not in self.used:
                self.used.add(w)
                return w


# ---- the numerals ----


class Tongue:
    """How numbers are written in the questions and answers of a kind: digits, one of the numeral grammars, a mix of digits and the first family, or raw words."""

    cap = 999  # the largest number the grammar writes
    words = ()
    foreign = ()  # words of the other families, for runs that mix them
    junk_lens = (2, 3, 4)  # lengths of the runs of words that bad_junk draws
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
        """Words of the grammar in a run of a length the grammar uses or one beside it, that is no numeral."""
        for _ in range(4000):
            toks = [self.words[rng.below(len(self.words))] for _ in range(self.junk_lens[rng.below(len(self.junk_lens))])]
            if not self.valid(" ".join(toks)):
                return " ".join(toks)
        raise RuntimeError(self.label)

    def bad_repeat(self, rng):
        """The words of a numeral with one of them once or twice more next to itself (a run too long to be a numeral for the grammars with a fixed length)."""
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


class Quot(Tongue):
    """The number named by two factors and a remainder: n = q * q2 + r with q the integer square root of n, q2 = n div q (so q2 is q, q + 1 or q + 2) and r = n mod q;
    three words from one set of thirty-three digit words 0 to 32 for the three places (the order is reversed in one variant); zero is the word for 0 three times.
    Numbers up to 999."""

    cap = 999
    junk_lens = (2, 3, 3, 3, 4)

    def __init__(self, label, words, param):
        super().__init__(label)
        self.flip = param % 2 == 1
        self.digit = [words.number_word() for _ in range(33)]
        self.words = list(self.digit)

    def say(self, n):
        if n == 0:
            t = [0, 0, 0]
        else:
            q = math.isqrt(n)
            t = [q, n // q, n - q * (n // q)]
        toks = [self.digit[x] for x in t]
        return " ".join(toks[::-1] if self.flip else toks)


def interleave(x, y):
    """The number whose bits at the even places are those of x and at the odd places those of y."""
    n = 0
    for i in range(5):
        n |= (x >> i & 1) << (2 * i) | (y >> i & 1) << (2 * i + 1)
    return n


class Morton(Tongue):
    """The bits of the number shared out between two words: the bits at the even places (counted from the least) make the first number, the bits at the odd places the second
    (the other way round in one variant); one set of thirty-two digit words 0 to 31 for both. Numbers up to 999; the numeral of 0 is the word for 0 twice."""

    cap = 999
    junk_lens = (1, 3, 3, 4)

    def __init__(self, label, words, param):
        super().__init__(label)
        self.flip = param % 2 == 1
        self.digit = [words.number_word() for _ in range(32)]
        self.words = list(self.digit)

    def say(self, n):
        x = sum((n >> (2 * i) & 1) << i for i in range(5))
        y = sum((n >> (2 * i + 1) & 1) << i for i in range(5))
        toks = [self.digit[x], self.digit[y]]
        return " ".join(toks[::-1] if self.flip else toks)


def hilbert_cell(d, side=32):
    """The cell (x, y) of a Hilbert curve over a square of the given side (a power of 2) that the d-th step stands on."""
    x = y = 0
    t = d
    s = 1
    while s < side:
        rx = 1 & (t // 2)
        ry = 1 & (t ^ rx)
        if ry == 0:
            if rx == 1:
                x, y = s - 1 - x, s - 1 - y
            x, y = y, x
        x, y = x + s * rx, y + s * ry
        t //= 4
        s *= 2
    return x, y


def hilbert_step(x, y, side=32):
    """The step of the curve a cell stands on (the inverse of hilbert_cell)."""
    d = 0
    s = side // 2
    while s > 0:
        rx = 1 if x & s else 0
        ry = 1 if y & s else 0
        d += s * s * ((3 * rx) ^ ry)
        if ry == 0:
            if rx == 1:
                x, y = side - 1 - x, side - 1 - y
            x, y = y, x
        s //= 2
    return d


class Hilbert(Tongue):
    """The place of the number along a Hilbert curve that fills a square of 32 by 32 cells, written as the two coordinates of its cell (x first, y first in one variant);
    one set of thirty-two digit words 0 to 31 for both. Numbers up to 999, so neighboring numbers are neighboring cells."""

    cap = 999
    junk_lens = (1, 3, 3, 4)

    def __init__(self, label, words, param):
        super().__init__(label)
        self.flip = param % 2 == 1
        self.digit = [words.number_word() for _ in range(32)]
        self.words = list(self.digit)

    def say(self, n):
        x, y = hilbert_cell(n)
        toks = [self.digit[x], self.digit[y]]
        return " ".join(toks[::-1] if self.flip else toks)


def gray_digits(n, k, base=4):
    """The n-th word (n below base ** k) of the reflected Gray code of k digits: the first digit as it is, the rest reflected when it is odd."""
    if k == 0:
        return []
    top, rest = divmod(n, base ** (k - 1))
    return [top] + gray_digits(rest if top % 2 == 0 else base ** (k - 1) - 1 - rest, k - 1, base)


class Gray(Tongue):
    """The reflected Gray code in base four: four digit words, the first digit of the code is the first digit of the number and each next digit is reflected when the digit
    before it in the number is odd, so that neighboring numbers differ in one word; no zero in front (zero is the word for 0 alone); the highest digit first
    (the lowest first in one variant). Numbers up to 999, numerals of up to five words, most with a word that occurs several times."""

    cap = 999
    junk_lens = (1, 2, 3, 4, 5, 6, 6)

    def __init__(self, label, words, param):
        super().__init__(label)
        self.low_first = param % 2 == 1
        self.digit = [words.number_word() for _ in range(4)]
        self.words = list(self.digit)

    def say(self, n):
        k = 1
        while 4**k <= n:
            k += 1
        ds = gray_digits(n, k)
        return " ".join(self.digit[d] for d in (ds[::-1] if self.low_first else ds))


SKEW_WEIGHTS = [2 ** (i + 1) - 1 for i in range(10)]  # 1, 3, 7, 15, ..., 1023


class Skew(Tongue):
    """Place value with the weights 1, 3, 7, 15, 31, ... (2 to the power k, minus 1): a word for each of the digits 0, 1, 2; the digits are chosen greedily, the heaviest weight
    that fits first, so that only the lowest digit that is not 0 may be a 2; the heaviest digit first (the lightest first in one variant); no zero in front (zero is
    the word for 0 alone). Numbers up to 999, numerals of up to nine words."""

    cap = 999
    junk_lens = (2, 3, 4, 5, 6, 7, 10, 11)

    def __init__(self, label, words, param):
        super().__init__(label)
        self.low_first = param % 2 == 1
        self.digit = [words.number_word() for _ in range(3)]
        self.words = list(self.digit)

    def say(self, n):
        top = max(i for i, w in enumerate(SKEW_WEIGHTS) if w <= max(n, 1))
        ds = []
        for i in range(top, -1, -1):
            ds.append(n // SKEW_WEIGHTS[i])
            n %= SKEW_WEIGHTS[i]
        return " ".join(self.digit[d] for d in (ds[::-1] if self.low_first else ds))

    def bad_repeat(self, rng):
        """One word repeated ten times or more (more weights than any numeral up to the cap needs)."""
        return " ".join([self.digit[rng.below(3)]] * (10 + rng.below(3)))


class Diffs(Tongue):
    """The digits of the number in base seven, each digit written as how far it is above the digit before it (taken modulo 7), the first digit as it is; seven digit words
    0 to 6; the digits from the top (from the bottom in one variant: the lowest digit as it is, each next digit as how far it is above the one below). No zero in front
    (zero is the word for 0 alone). Numbers up to 999, numerals of up to four words, most with a word that occurs several times."""

    cap = 999
    junk_lens = (2, 3, 4, 5)

    def __init__(self, label, words, param):
        super().__init__(label)
        self.low_first = param % 2 == 1
        self.digit = [words.number_word() for _ in range(7)]
        self.words = list(self.digit)

    def say(self, n):
        ds = []
        while n:
            ds.append(n % 7)
            n //= 7
        ds = (ds or [0])[::-1]  # from the top
        order = ds[::-1] if self.low_first else ds
        return " ".join(self.digit[(d - (order[i - 1] if i else 0)) % 7] for i, d in enumerate(order))

    def bad_repeat(self, rng):
        """One word repeated five times or more (more digits than any numeral up to the cap has)."""
        return " ".join([self.digit[rng.below(7)]] * (5 + rng.below(3)))


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


KINDS_OF_GRAMMAR = {"morton": Morton, "quot": Quot, "gray": Gray, "hilbert": Hilbert, "skew": Skew, "diffs": Diffs}
# the grammars of the first, second and third family for each seed are three in a row of this ring (the fourth is the next one; the seeds after the sixth begin again)
RING = ["morton", "quot", "gray", "hilbert", "skew", "diffs"]


def describe(g, tongue):
    if g == "quot":
        return "two factors and a remainder, n = q * q2 + r with q the integer square root, one set of thirty-three digit words 0 to 32 for the three places" + (", the remainder first" if tongue.flip else ", the root first")
    if g == "morton":
        return "the bits of the number shared out between two words (even places, odd places), one set of thirty-two digit words" + (", odd places first" if tongue.flip else ", even places first")
    if g == "hilbert":
        return "the cell of a Hilbert curve over 32 by 32 cells that the number stands on, two coordinate words" + (", y first" if tongue.flip else ", x first")
    if g == "gray":
        return "reflected Gray code in base four, four digit words, " + ("lowest digit first" if tongue.low_first else "highest digit first")
    if g == "skew":
        return "place value with the weights 1, 3, 7, 15, ..., greedy, " + ("lightest digit first" if tongue.low_first else "heaviest digit first")
    return "base-seven digits written as the step up from the digit before, " + ("from the bottom" if tongue.low_first else "from the top")


# (the grammar, what a lesson that gives a numeral and its number shows, what the arithmetic of the kinds computed over the numerals shows)
HOW = {
    "Quot": ("a number n is q * q2 + r with q the integer square root of n, q2 = n div q and r = n mod q (0 is three zero words), the three places sharing the words 0 to 32",
             "a lesson (numeral, number) shows q, q2 and r, so that every word is lined up with the value that makes the formula give the number",
             "the numerals of 1, 2, 3, 4 (the values 1 1 0, 1 2 0, 1 3 0, 2 2 0) name the words in order of size, since a sum of small numbers is a numeral that changes by one word at a time, and sums of larger numbers name the rest"),
    "Morton": ("the bits of n at the even places make the first number and at the odd places the second, five bits each, the words standing for 0 to 31",
               "a lesson (numeral, number) fixes the words one at a time, since the numerals of 1, 2, 4, 8, ... are the single bits and the numerals of 3, 5, ... are their sums",
               "the sums of small numbers (1 + 2 * 1 = 3 is both words stepping from 0 to 1) name the words in order of size"),
    "Hilbert": ("n is the place along a Hilbert curve over a square of 32 by 32 cells and the numeral is the two coordinates of the cell",
                "a lesson (numeral, number) fixes the words, since the numeral of 0 is the corner, the next numerals each step one cell and the numerals of 0 to 3 go round a square of four cells",
                "the sums of small numbers name the words in the order that the curve visits cells (each step changes one word by one)"),
    "Gray": ("the digits of n in base four are changed so that neighboring numbers differ in one word (a digit after an odd digit is reflected), the four words standing for 0 to 3",
             "a lesson (numeral, number) fixes each word, since the numerals of 0 to 3 are the single words and those of 4 to 7 begin with the word for 1",
             "the sums of small numbers name the words in order, and the numeral grows by one word at a time"),
    "Skew": ("place value with the weights 1, 3, 7, 15, ... (2 to the power k minus 1) and the digits 0, 1, 2, a 2 only at the lowest digit that is not 0: the numerals of 0, 1 and 2 are single words and the numeral of 3 is the words for 1 0",
             "a lesson (numeral, number) reads off the three words and the rest follows",
             "the sums and thirds of the numbers below 10 fix the three digits"),
    "Diffs": ("n in base seven, each word the step of a digit above the digit before it modulo 7, the first as it is, the words standing for 0 to 6: the numerals of 1 to 6 are single words and the numeral of 7, the digits 1 0, is the word for 1 and then the word for 6 (0 is 6 steps above 1 modulo 7)",
              "a lesson (numeral, number) fixes the words",
              "the sums of small numbers name the words in order of the value of the first digit"),
}


def world_tongues(seed):
    """The word generator of a world (already used for the numerals) and its ways of writing: digits, three families of words, a fourth that no kind reads, a mix, raw words."""
    words = Words(seeded(seed, 2))
    row = [RING[(seed - 1 + i) % 6] for i in range(4)]
    tongues = {"d": DIGITS, "w": RAW}
    for i, (name, g) in enumerate(zip(("t1", "t2", "t3", "t4"), row)):
        tongues[name] = KINDS_OF_GRAMMAR[g](f"{name}: " + g, words, ((seed - 1) // 2 + i) % 2)
        tongues[name].label = f"{name}: " + describe(g, tongues[name])
    for name in ("t1", "t2", "t3", "t4"):
        tongues[name].foreign = [w for other in ("t1", "t2", "t3", "t4") if other != name for w in tongues[other].words]
    tongues["m1"] = Mix("m1: digits or t1 words", tongues["t1"])
    return words, tongues


# ---- the kinds ----

MIDDLE = [47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97, 101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193, 197, 199, 211]
LARGE = [211, 223, 227, 229, 233, 239, 241, 251, 257, 263, 269, 271, 277, 281, 283, 293, 307, 311, 313, 317, 331, 337, 347, 349, 353, 359, 367, 373, 379, 383, 389, 397, 401, 409, 419, 421, 431, 433, 439, 443, 449, 457, 461, 463, 467, 479, 487, 491, 499]
SMALL_MODS = (13, 17, 19, 23, 29, 31, 37)  # all within the constants the reachability search offers


class KKind(Kind):
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
    """An arbitrary code for each number or each pair of numbers (arity 2: 0 to 15). With `looks` (a divisor) it is a quotient with arbitrary exceptions: a rule for most points."""
    n = (hi + 1) ** arity
    if looks:
        while True:
            codes = [a // looks if rng.below(100) >= 40 else rng.below(looks + 9) for a in range(n)]
            if sum(1 for a in range(n) if codes[a] != a // looks) >= 8:
                break
        rule = f"a div {looks} for most numbers, an arbitrary code of 0 to {looks + 8} for the others: it looks like a quotient and is not: " + " ".join(map(str, codes))
    elif arity == 1:
        codes = [rng.below(bound) for _ in range(n)]
        rule = "an arbitrary code, no rule: " + " ".join(map(str, codes))
    else:
        codes = [rng.below(bound) for _ in range(n)]
        rule = f"an arbitrary code from 0 to {bound - 1} for each pair of numbers 0..{hi} (drawn from the seed), no rule"
    why = "cannot be learned: stays refused until told"
    if arity == 1:
        return KKind(name, "unexplainable", 1, lambda a, c=codes: c[a], rule, why, tin=tin, tout=tout, hi=hi)
    return KKind(name, "unexplainable", 2, lambda a, b, c=codes, w=hi + 1: c[a * w + b], rule, why, tin=tin, tout=tout, hi=hi)


def make_kinds(seed, tongues=None):
    """The kinds of a world, in four groups: control, newop, compose, unexplainable (each group shuffled by the seed)."""
    if tongues is None:
        tongues = world_tongues(seed)[1]
    t1, t2, t3 = tongues["t1"], tongues["t2"], tongues["t3"]
    rng = seeded(seed, 1)
    cm1, cm2, cm3 = 7 + rng.below(25), 3 + rng.below(10), MIDDLE[rng.below(len(MIDDLE))]  # clipmix
    sp1, sp2, sp3 = 3 + rng.below(58), 3 + rng.below(58), LARGE[rng.below(len(LARGE))]  # shiftprod
    vq1, vq2 = 3 + rng.below(28), 4 + rng.below(9)  # varquot
    cp1, cp2, cp3 = 2 + rng.below(8), 2 + rng.below(8), MIDDLE[10 + rng.below(20)]  # capmod
    mm1 = 1 + rng.below(30)  # modmod
    sq1, sq2, sq3, sq4 = 3 + rng.below(7), 1 + rng.below(40), 7 + rng.below(13), 9 + rng.below(52)  # rootfork
    om1, om2 = 3 + rng.below(38), SMALL_MODS[rng.below(len(SMALL_MODS))]  # offmod
    sw1 = SMALL_MODS[rng.below(len(SMALL_MODS))]  # subwrap
    rs1, rs2 = 2 + rng.below(8), 1 + rng.below(9)  # ratshift
    pm1 = SMALL_MODS[rng.below(len(SMALL_MODS))]  # peakmod
    sq_ = 1 + rng.below(9)  # sumquot
    mp1 = 7 + rng.below(25)  # blendpair
    mb1 = 3 + rng.below(38)  # mixback
    sf1, sf2 = ((5, 4), (7, 6), (3, 4), (5, 6), (2, 3), (7, 8), (8, 9))[rng.below(7)]  # shiftf12
    tq = 3 + rng.below(4)  # nearquot

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
        return KKind(name, role, arity, fn, rule, learn, tin=tin, tout=tout, lo=lo, hi=hi, **kw)

    def offmod(a):
        return (a + om1) % om2

    def ratshift(a, b):
        return a * rs1 // (b + rs2)

    def tokens(a):
        return t3.say(a).split(" ")

    newop = [
        mk("clipmix", "newop", 1, lambda a: (a % cm1) * cm2 + a // cm3, f"(a mod {cm1}) * {cm2} + a div {cm3}",
           "nine nodes with three constants, a remainder scaled and a quotient added: along a the answers climb by the step of the scale and fall back at every multiple of the first constant, and the quotient adds one more at every multiple of the last; the three constants show as the period, the step and the width of the slow rise", hi=999),
        mk("shiftprod", "newop", 2, lambda a, b: (a + sp1) * (b + sp2) % sp3, f"(a + {sp1}) * (b + {sp2}) mod {sp3}, both numbers in words of the second family, the answer in digits",
           "nine nodes with three constants, a product of two shifted numbers taken modulo a large constant: along a the answers climb by the shifted b and wrap at the modulus, and along b by the shifted a; each shift shows as the value of the row or column at 0; the words are read through the arithmetic lessons of the second family", tin="t2", hi=99),
        mk("varquot", "newop", 2, lambda a, b: (a * vq1 + b) // (b % vq2 + 1), f"(a * {vq1} + b) div (b mod {vq2} + 1), a in words of the second family and b in digits, the answer in digits",
           "eleven nodes with two constants: a divisor that depends on the second number through a remainder, so along a the answers climb by a step of the first constant div the divisor, and the divisor cycles with period of the second constant along b; the words are read through the arithmetic lessons of the second family", tin=("t2", "d"), hi=99),
        mk("capmod", "newop", 3, lambda a, b, c: min(a * cp1 + b, c * cp2 % cp3), f"min(a * {cp1} + b, c * {cp2} mod {cp3})",
           "eleven nodes with three constants, the smaller of a line in a and b and a wrapped multiple of c: where c is small the second part is a line in c, and it wraps at the modulus; the first part wins where it is the smaller, so the table has two regimes", hi=30),
        mk("modmod", "newop", 3, lambda a, b, c: (a * b + c) % (a + mm1), f"(a * b + c) mod (a + {mm1})",
           "nine nodes with one constant, a modulus that grows with a: fix a and the answers along b and c are a line wrapped at a + the constant; the modulus is the first value at which the wrap comes", hi=30),
        mk("divways", "newop", 1, ordered_factorizations, "the number of ways to write a as a product of numbers above 1, order counted (a from 1; 1 is the empty product), the number spelled as words of the third family, the answer in digits",
           "a number defined from the values at the divisors of a, the table built from the left, each entry the sum of the entries at the divisors below it: 1, 1, 1, 2, 1, 3, 1, 4, 2, 3, ...; the words of the third family are fixed by nothing in the lessons, so only an assignment of numbers to the words that fits the table gives their meaning", tin="t3", lo=1, hi=299),
        mk("rootfork", "newop", 1, lambda a: math.isqrt(a) * sq1 + sq2 if is_square(a) else a % sq3 + a // sq4,
           f"{sq1} * (the integer square root of a) + {sq2} if a is a perfect square, else (a mod {sq3}) + a div {sq4}, the number written as digits or as words of the first family (the lessons mix them), the answer in digits",
           "a conditional on a property that is neither a threshold nor a remainder: the squares 0, 1, 4, 9, 16, ... take the first branch (a line in their root) and the other numbers the second (a wrapped line); the words are read through the lessons that tie the first family to digits", tin="m1", hi=999),
        mk("textrank", "newop", 2, text_rank, "where a stands among the numbers 1 to b when they are put in the order of the alphabet (as strings of decimal digits): how many of them come at or before it (a from 1)",
           "an order on strings of digits that is not the order of numbers: 1, 10, 100, 11, 12, ..., 19, 2, 20, ...: for b a power of 10 the answer for a number a is its place in that list; the table is built by counting, for each a, the numbers up to b whose digit string is not after that of a", lo=1, hi=99),
        mk("gridwalk", "newop", 1, digit_walk, "a walk on a grid told by the decimal digits: start at (0, 0) facing north; for each digit d from the left turn right d quarter turns, then step d cells forward; the answer is how far the end is from the start along the grid lines (|x| + |y|), spelled as words of the first family",
           "a loop with a state of a position and a heading over the decimal digits: read from the left, each digit turns the heading and moves the position; the answer is small and follows from the answer for the number without its last digit only together with the position and heading it left", tout="t1", hi=999),
        mk("belowline", "newop", 2, under_line, "the points (x, y) of the grid with x and y at least 1 that lie strictly under the straight line from (a, 0) to (0, b)",
           "a count of points of a grid under a line: for each column x from 1 to a - 1 the number of y below the line is (b * (a - x) - 1) div a, so a loop over columns adds the floors; for a = 2 it is b - 1 div 2 and the table is symmetric in a and b", lo=1, hi=60),
    ]
    compose = [
        mk("chainratio", "compose", 2, lambda a, b: ratshift(offmod(a), offmod(b)), f"ratshift(offmod(a), offmod(b)): (a + {om1}) mod {om2} * {rs1} div ((b + {om1}) mod {om2} + {rs2})",
           "an earlier simple kind fed with the answers of another earlier simple kind: the inner answers are few and cycle, so the table repeats with the period of the first constant in both numbers, but a program for it needs more than seven nodes", hi=399),
        mk("walkshift", "compose", 1, lambda a: digit_walk(offmod(a)), f"gridwalk(offmod(a)): the walk told by the digits of (a + {om1}) mod {om2}, the answer spelled as words of the first family",
           "an earlier hard kind fed with the answer of an earlier simple one: the walk of a number below the modulus, so the answers repeat with the period of the modulus", tout="t1", hi=999),
    ]
    control = [
        mk("offmod", "control", 1, offmod, f"(a + {om1}) mod {om2}", "five nodes over the starting operations: a shift and a remainder; taught in all eight ways of asking with one number, each also with its filler left out, so the ways can be seen to mean the same", hi=399, bridge=True),
        mk("subwrap", "control", 2, lambda a, b: (a - b) % sw1, f"(a - b) mod {sw1} (a remainder of a difference that may be below zero is taken as a number from 0 up)", "five nodes over the starting operations: a difference and a remainder; taught in all eight ways of asking with two numbers, each also with its filler left out", bridge=True),
        mk("peakmod", "control", 3, lambda a, b, c: max(a, b) * c % pm1, f"max(a, b) * c mod {pm1}", "seven nodes over the starting operations: the larger of two numbers times a third, taken modulo a constant; taught in all six ways of asking with three numbers, each also with its filler left out", bridge=True),
        mk("ratshift", "control", 2, ratshift, f"a * {rs1} div (b + {rs2})", "seven nodes over the starting operations: a product over a shifted number", hi=99),
        mk("sumquot", "control", 3, lambda a, b, c: (a + b) // (c + sq_), f"(a + b) div (c + {sq_})", "seven nodes over the starting operations: a sum over a shifted number", hi=24),
        mk("numin1", "control", 1, lambda a: a, "a spelled as words of the first family (up to the largest number the grammar writes), the answer a in digits", "the words are lined up with the digits by the lessons: the lessons show every word of the family, so the words of a held-out numeral all occur in other lessons and the grammar has to be worked out from them", tin="t1", lo=1, hi=t1.cap, cover="t1"),
        mk("numout1", "control", 1, lambda a: a, "a in digits, the answer a spelled as words of the first family", "the same lessons from the other side", tout="t1", lo=1, hi=t1.cap, cover="t1"),
        mk("bumpf1", "control", 2, lambda a, b: a + b // 2, "a + b div 2, the numbers and the answer spelled as words of the first family", "four nodes over the starting operations once the words are read as numbers; the two kinds that tie the words to digits show how", tin="t1", tout="t1", hi=t1.cap),
        mk("blendpair", "control", 2, lambda a, b: a + b % mp1, f"a + b mod {mp1}, a in digits and b spelled as words of the first family, the answer in digits", "five nodes over the starting operations once the words are read as numbers; every question has one number of each form", tin=("d", "t1"), hi=299),
        mk("mixback", "control", 2, lambda a, b: min(a + mb1, b * 3), f"min(a + {mb1}, 3 * b), a spelled as words of the first family and b in digits, the answer spelled as words of the first family", "seven nodes over the starting operations once the words are read as numbers; the answer is in the form of the first number", tin=("t1", "d"), tout="t1", hi=t1.cap),
        mk("shiftf12", "control", 1, lambda a: a * sf1 // sf2, f"a * {sf1} div {sf2}, a spelled as words of the first family and the answer spelled as words of the second", "five nodes over the starting operations once both families are read as numbers: the first through the kinds that tie it to digits, the second through the arithmetic of its own kinds", tin="t1", tout="t2", hi=t1.cap),
        mk("skewsumf2", "control", 2, lambda a, b: a + 3 * b, "a + 3 * b, the numbers and the answer spelled as words of the second family", "no lesson ties the words to digits; this kind and the third-taking kind fix the meaning through the arithmetic alone (the small values name the words one by one, then the places appear in the sums that carry)", tin="t2", tout="t2", hi=t2.cap, cover="t2"),
        mk("thirdf2", "control", 1, lambda a: a // 3, "a div 3, the number and the answer spelled as words of the second family", "the floor of a third of a number of the second family: a small table that fixes the order of the words and where the places carry", tin="t2", tout="t2", hi=t2.cap, cover="t2"),
        mk("echo3", "control", 1, lambda a: " ".join(tokens(a) + tokens(a)), "the words of the numeral (third family) written twice in a row", "a copy of the words of the question, twice, whatever they mean", tin="t3", tout="w", hi=t3.cap, numeric=False),
        mk("oddplaces3", "control", 1, lambda a: " ".join(tokens(a)[::2]), "the words of the numeral (third family) that stand in the odd places (the first, the third, ...)", "a selection of the words of the question by their places whatever they mean", tin="t3", tout="w", hi=t3.cap, numeric=False),
    ]
    unexplainable = [
        code_kind(rng, "tagpair", 20, 2, "d", "d", hi=15),
        code_kind(rng, "nearquot", 0, 1, "d", "d", looks=tq),
        code_kind(rng, "noisef1", min(300, t1.cap + 1), 1, "t1", "t1"),
        code_kind(rng, "noisef2", 55, 1, "t2", "d"),
        code_kind(rng, "noisef3", min(200, t3.cap + 1), 1, "t3", "t3"),
    ]
    found = " How it could be found: a search over the expressions of at most seven nodes over + - * min max div mod, the variables and small constants, with the constants read off the table (a modulus is where a column wraps, a divisor is the width of a step, a shift is the value at 0)."
    for k in control:
        if k.name in ("offmod", "subwrap", "peakmod", "ratshift", "sumquot", "bumpf1", "blendpair", "mixback", "shiftf12", "skewsumf2", "thirdf2"):
            k.learn += "." + found if not k.learn.endswith(".") else found
    return rng.shuffled(control), rng.shuffled(newop), rng.shuffled(compose), unexplainable


# ---- the ways of asking ----


def render(toks, word, nums):
    """A question: the tokens of a way of asking with the word and the numbers (already written in their numerals) in it."""
    return " ".join(word if t == "{w}" else nums[SLOTS.index(t)] if t in SLOTS else t for t in toks)


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


def all_ways(arity):
    """The token lists of the ways a kind that carries the equivalences is taught in: every way of asking, and every one of them that has a filler once with it left out."""
    return [f.toks for f in FRAMES[arity]] + dropped(arity, range(len(FRAMES[arity])))


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
    # runs of words that are no numeral: a run of words of the grammar that it does not write, a word repeated, a word that is almost a number word
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
    # a numeral of the family that no kind reads in the place of a number
    for _ in range(4):
        def unread():
            k = rng.shuffled(live)[0]
            t4 = tongues["t4"]
            args = nums(k)
            args[rng.below(k.arity)] = t4.say(t4.some(rng))
            return add(k.word, render(frame_of(k).toks, k.word, args), k.tout, "unread numeral")

        retry(unread)
    # the units of a taught question in the opposite order
    for _ in range(6):
        k = rng.shuffled(live)[0]

        def reverse():
            n = nums(k)
            units = [k.word if t == "{w}" else n[SLOTS.index(t)] if t in SLOTS else t for t in frame_of(k).toks]
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
    # the word of another kind in the place of a filler of a taught way, or after it
    for _ in range(4):
        def foreign_word():
            k, k2 = rng.shuffled(live)[:2]
            f = frame_of(k)
            toks = list(f.toks)
            if f.drop and rng.below(2):
                toks[f.drop[0]] = k2.word
            else:
                toks = toks + [k2.word]
            return add(k.word, render(toks, k.word, nums(k)), k.tout, "foreign kind word")

        retry(foreign_word)
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
    # the ways of asking: two or three taught for each kind (all of them, and each with its filler left out, for the three that carry the equivalences); the held-out facts of
    # the rest are asked in a way that was not taught for the kind, in a mix of the start of one taught way and the end of another, in a taught way with a filler left out, or in a taught way
    eligible = [k for k in order if k.role != "unexplainable" and not k.bridge]
    modes = {id(k): ("plain", "untaught", "hybrid", "drop")[i % 4] for group in ([k for k in eligible if k.role != "control"], [k for k in eligible if k.role == "control"]) for i, k in enumerate(rng.shuffled(group))}
    lessons, items, key, used, fact, taught_of = [], [], [], set(), 0, {}
    for k in order:
        frames = FRAMES[k.arity]
        mode = modes.get(id(k), "plain")
        ids = list(range(len(frames))) if k.bridge else choose_frames(rng, k.arity, mode)
        taught_of[k.word] = ids
        taught_ways = all_ways(k.arity) if k.bridge else [frames[i].toks for i in ids]
        if mode == "untaught":
            held_ways = [frames[rng.shuffled([i for i in range(len(frames)) if i not in ids])[0]].toks]
        elif mode == "hybrid":
            held_ways = hybrids(k.arity, ids)
        elif mode == "drop":
            held_ways = dropped(k.arity, ids)
        else:
            held_ways = taught_ways
        how = {"plain": "in a taught way", "untaught": "in a way not taught for this kind", "hybrid": "in a mix of the start of one taught way and the end of another", "drop": "in a taught way with a filler word left out"}[mode]
        spelled = "; numerals: " + ", ".join(f"{x} {tongues[t].label}" for x, t in [(f"number {i + 1}" if k.arity > 1 else "in", t) for i, t in enumerate(k.tins)] + [("out", k.tout)] if t != "d")
        notes = (spelled if spelled != "; numerals: " else "") + f"; taught in {len(taught_ways)} ways of asking" + ("" if k.bridge else f", the held-out facts are asked {how}")
        if k.bridge:
            notes += "; every way with a filler is also taught with the filler left out, which shows what can be dropped"
        if "t1" in k.tins + (k.tout,) or "m1" in k.tins:
            how = HOW[type(tongues["t1"]).__name__]
            notes += f"; first family: the kinds that read and write it tie the words to digits; how the meaning can be worked out: {how[0]}: {how[1]}; and by arithmetic: {how[2]}"
        if "t2" in k.tins + (k.tout,):
            how = HOW[type(tongues["t2"]).__name__]
            notes += f"; second family: no kind reads or writes it, so no lesson gives a number in digits and the meaning comes only through the arithmetic of the kinds computed over it; how the meaning can be worked out: {how[0]}: {how[2]}"
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
            for q, truth in texts(p, taught_ways):
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
    p = argparse.ArgumentParser(description="Write an eleventh hidden-worlds adaptive exam (lessons.tsv, stream.tsv) for `baby exam adapt-drive`.")
    p.add_argument("dir")
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--compose-first", action="store_true", help="teach the compositions before the kinds they use (a control: reuse should then not help)")
    a = p.parse_args(argv)
    nl, ns = write(a.dir, a.seed, a.compose_first)
    print(f"wrote the hidden worlds K (seed {a.seed}) to {a.dir}: {nl} lessons and a stream of {ns} questions")


if __name__ == "__main__":
    main(sys.argv[1:])
