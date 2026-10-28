#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""A ninth set of exam worlds: new closed forms (a floor of a quadratic, a sum of absolute differences, a polynomial in two numbers modulo a prime, a minimum of
three linear forms), rules over the digits in base 2 and base 3, numbers spelled in six more kinds of numeral, numbers whose forms are mixed inside a question, new
ways of asking built from pieces of each other, and traps that look like the rewordings.

The same files, line protocol and shape of world as `tools/hidden_worlds.py` and its seven successors (so `baby exam adapt-drive DIR COMMAND...` judges any
system on it). To keep the set a blind test the kinds are not described here.

    python3 tools/hidden_worlds_i.py DIR [--seed N] [--compose-first]

What is new in the way the exam is put:
  numerals   some kinds take and give their numbers as words, in one of six grammars (chosen by the seed; each world uses three of them, one per family of
             words, and a fourth family that no kind reads). In the first family a pair of kinds ties the words to digits, so their meaning can be worked out;
             in the second no kind reads or writes the words and the meaning can only come through arithmetic done on them; in the third nothing in the lessons
             can fix the meaning. A learner is never told that the words are numbers.
  mixed      kinds that take one number as digits and the other as words, kinds whose lessons draw the form of each number, kinds whose answer is in words of
             one family for a question in words of another, and kinds taught in one form of numbers and asked in the other.
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

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hidden_worlds import LIMIT, Kind, Rng  # noqa: E402

README = """THE HIDDEN WORLDS I (adaptive exam; the same files and protocol as `baby exam adapt-write`, written by tools/hidden_worlds_i.py)

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

ENDING = "cuwe"  # shared by every kind word of this set (the earlier sets' words end in `sido`, `velu`, `ketu`, `ruvo`, `zane`, `yarn`, `tole` and `dopa`)


def seeded(seed, salt):
    """A generator for a seed and a purpose (the seed goes through one extra mix first, so that neighboring seeds are not the same stream a few steps on)."""
    return Rng(Rng(seed * 3000017 + salt * 130363 + 41).next())


# ---- the operations ----


def base3(a):
    out = ""
    while a:
        out = str(a % 3) + out
        a //= 3
    return out


def cantor_count(a):
    """How many of the numbers 0 to a have no digit 2 in base 3."""
    return sum(1 for x in range(a + 1) if "2" not in base3(x))


def tail_pairs(a, b):
    """How many different pairs the walk (x, y) -> (y, |x - y|) from (a, b) visits before a pair comes round again."""
    seen, x, y = set(), a, b
    while (x, y) not in seen:
        seen.add((x, y))
        x, y = y, abs(x - y)
    return len(seen)


def mod_inverse(a, p):
    """The smallest x from 1 with a * x = 1 modulo the prime p (a from 1 to p - 1)."""
    return next(x for x in range(1, p) if a * x % p == 1)


def board_squares(a, b):
    """The squares of every size, with sides along the lines, on a board of a by b cells."""
    return sum((a - k + 1) * (b - k + 1) for k in range(1, min(a, b) + 1))


def adjacent_ones(a):
    """Whether the binary form of a has two 1 digits side by side."""
    return (a & (a >> 1)) != 0


def run_dfa(a, table, out):
    """The state of a machine with the table {state: (next on 0, next on 1)} after the binary digits of a (none for 0, no zeros in front), read from the largest; out[state]."""
    state = 0
    for c in bin(a)[2:] if a else "":
        state = table[state][int(c)]
    return out[state]


# ---- the words ----

CONSONANTS = "bdghklmnprstvz"
VOWELS = "aeiou"
LASTS = ["", "", "", "l", "n", "r", "s"]
NEW_LETTERS = "cjqwy"  # no word of an earlier set has one of these: every number word of this set begins with one
ALL_CONSONANTS = CONSONANTS + NEW_LETTERS


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
    1: [Frame(t) for t in ["evaluate {w} at {a} [kindly]", "{a} is fed to {w} [today]", "state what {w} makes of {a}", "of the {w} on the {a}", "see {w} over {a}", "if {a} goes through {w} what comes out", "under {w} the {a} turns into [what]", "name {w} for {a} [pray]"]],
    2: [Frame(t) for t in ["relate {w} between {a} and {b}", "{a} then {b} into {w} [today]", "set {w} to work on the first {a} and the second {b}", "given {a} and {b} evaluate {w} [kindly]", "{w} reading {a} against {b}", "with {a} on the left and {b} on the right name {w}", "see {w} over {a} and over {b}", "{a} paired with {b} is handed to {w}"]],
    3: [Frame(t) for t in ["evaluate {w} at {a} then {b} then {c}", "state {w} with first {a} second {b} third {c}", "given {a} and {b} and {c} evaluate {w} [kindly]", "hand {a} and {b} and {c} to {w} [today]", "see {w} over {a} and over {b} and over {c}", "set {w} to work on the top {a} the middle {b} the bottom {c}"]],
}
# a start or an end that no way of asking has, for any kind
NEW_STARTS = [["suppose"], ["let", "us", "see"], ["imagine"], ["assume"]]
NEW_ENDS = [["somehow"], ["in", "the", "end"], ["or", "not"], ["to", "me"]]
JOIN = ["also", "after"]
BARE = ["{a}", "so {a}", "what about {a}"]
FRAME_WORDS = {t for fs in FRAMES.values() for f in fs for t in f.toks if t not in SLOTS and t != "{w}"}
FRAME_WORDS |= {t for ws in NEW_STARTS + NEW_ENDS + [JOIN] for t in ws} | {t for b in BARE for t in b.split(" ") if t not in SLOTS}
FILLERS = ["at", "to", "over", "of"]  # words of the frames that stand in for the kind's word in a trap


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
        return self.fresh(lambda r: "".join([NEW_LETTERS[r.below(5)], VOWELS[r.below(5)], ALL_CONSONANTS[r.below(19)], VOWELS[r.below(5)]]))

    def sandhi_word(self):
        """A digit word that ends in a, o or u, and the two forms it takes before the ten word and before the hundred word (the last vowel becomes i or e)."""
        while True:
            w = "".join([NEW_LETTERS[self.rng.below(5)], VOWELS[self.rng.below(5)], ALL_CONSONANTS[self.rng.below(19)], "aou"[self.rng.below(3)]])
            forms = (w, w[:3] + "i", w[:3] + "e")
            if not any(f in self.used for f in forms):
                self.used.update(forms)
                return forms

    def almost(self, word, keep=0, number=False):
        """A word one letter away from `word` (a vowel for a vowel, a consonant for a consonant, none of the last `keep` letters, the first letter of a number word
        from the new letters) that is no word of the exam."""
        while True:
            i = self.rng.below(len(word) - keep)
            pool = VOWELS if word[i] in VOWELS else NEW_LETTERS if number and i == 0 else ALL_CONSONANTS if number else CONSONANTS
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
        """A numeral of this grammar with one of its words replaced by a word of another family."""
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


class Vigesimal(Tongue):
    """Counting in twenties with digits of base 20: digit words 1 to 9, a word for ten and one for fifteen (11 to 14 are ten and a digit, 16 to 19 fifteen and a digit),
    a word for twenty that closes the count of twenties (always with its digit), the count of twenties first (in one variant the leftover first and the word for
    twenty in front of its count); a word for zero that stands alone."""

    cap = 399

    def __init__(self, label, words, param):
        super().__init__(label)
        self.low_first = param % 2 == 1
        self.digit = {i: words.number_word() for i in range(1, 10)}
        self.ten, self.fifteen, self.score, self.zero = (words.number_word() for _ in range(4))
        self.words = [self.zero] + list(self.digit.values()) + [self.ten, self.fifteen, self.score]

    def small(self, c):
        if c < 10:
            return [self.digit[c]]
        if c == 10:
            return [self.ten]
        if c < 15:
            return [self.ten, self.digit[c - 10]]
        if c == 15:
            return [self.fifteen]
        return [self.fifteen, self.digit[c - 15]]

    def say(self, n):
        if n == 0:
            return self.zero
        c1, c0 = divmod(n, 20)
        low = self.small(c0) if c0 else []
        if self.low_first:  # the count of twenties follows its word, so that the leftover and the count cannot run together
            return " ".join(low + ([self.score] + self.small(c1) if c1 else []))
        return " ".join((self.small(c1) + [self.score] if c1 else []) + low)


class Biquinary(Tongue):
    """Decimal places of one to four, with a word for each of 1 to 4, a word for five of its own in the units and another in the tens (a digit from 5 to 9 is that five and
    the word of the rest, the rest first in one variant), a word that closes the tens and one that closes the hundreds (a count of 1 to 4 before it); a word for zero that
    stands alone."""

    cap = 499

    def __init__(self, label, words, param):
        super().__init__(label)
        self.after = param % 2 == 1
        self.digit = {i: words.number_word() for i in range(1, 5)}
        self.five_u, self.five_t, self.tens, self.hund, self.zero = (words.number_word() for _ in range(5))
        self.words = [self.zero] + list(self.digit.values()) + [self.five_u, self.five_t, self.tens, self.hund]

    def part(self, x, five):
        toks = ([five] if x >= 5 else []) + ([self.digit[x % 5]] if x % 5 else [])
        return toks[::-1] if self.after else toks

    def say(self, n):
        if n == 0:
            return self.zero
        h, r = divmod(n, 100)
        t, u = divmod(r, 10)
        out = []
        if h:
            out += [self.digit[h], self.hund]
        if t:
            out += self.part(t, self.five_t) + [self.tens]
        if u:
            out += self.part(u, self.five_u)
        return " ".join(out)


class Doubling(Tongue):
    """A word for one and the words 'times two' and 'and one': the binary digits of the number from the second on, each a 'times two' and a 1 also an 'and one', so that
    a repeated word means doubling again; a word for zero that stands alone."""

    cap = 999

    def __init__(self, label, words, param):
        super().__init__(label)
        self.one, self.twice, self.plus, self.zero = (words.number_word() for _ in range(4))
        self.words = [self.zero, self.one, self.twice, self.plus]

    def say(self, n):
        if n == 0:
            return self.zero
        out = [self.one]
        for b in bin(n)[3:]:
            out.append(self.twice)
            if b == "1":
                out.append(self.plus)
        return " ".join(out)


class Chunks(Tongue):
    """A word of its own for each of the 100 two-digit chunks 0 to 99: a number below 100 is one word, a larger one the word of the hundreds and the word of the last two
    digits (the last two digits first in one variant, whatever the zero chunk is called)."""

    cap = 999

    def __init__(self, label, words, param):
        super().__init__(label)
        self.low_first = param % 2 == 1
        self.chunk = [words.number_word() for _ in range(100)]
        self.value = {w: i for i, w in enumerate(self.chunk)}
        self.words = list(self.chunk)

    def say(self, n):
        if n < 100:
            return self.chunk[n]
        hi, lo = divmod(n, 100)
        return " ".join([self.chunk[lo], self.chunk[hi]] if self.low_first else [self.chunk[hi], self.chunk[lo]])

    def some(self, rng):
        return 100 + rng.below(self.cap - 99)

    def bad_junk(self, rng):
        """Two words of which the one that stands for the hundreds is a chunk of 10 or more."""
        toks = [self.chunk[10 + rng.below(90)], self.chunk[rng.below(100)]]
        return " ".join(toks[::-1] if self.low_first else toks)

    def bad_repeat(self, rng):
        if rng.below(2):
            w = self.chunk[rng.below(100)]
            return f"{w} {w} {w}"
        w = self.chunk[10 + rng.below(90)]
        return f"{w} {w}"


class Sandhi(Tongue):
    """Decimal places with a word for 1 to 9, a word for ten and one for hundred, each place a digit and its place word (a digit is never left out before a place
    word); a digit word changes its last vowel to i before the ten word and to e before the hundred word; a word for zero that stands alone."""

    cap = 999

    def __init__(self, label, words, param):
        super().__init__(label)
        forms = [words.sandhi_word() for _ in range(9)]
        self.digit = {i + 1: f[0] for i, f in enumerate(forms)}
        self.before_ten = {i + 1: f[1] for i, f in enumerate(forms)}
        self.before_hund = {i + 1: f[2] for i, f in enumerate(forms)}
        self.ten, self.hund, self.zero = (words.number_word() for _ in range(3))
        self.words = [self.zero] + list(self.digit.values()) + list(self.before_ten.values()) + list(self.before_hund.values()) + [self.ten, self.hund]
        self.plain = {**{v: self.digit[k] for k, v in self.before_ten.items()}, **{v: self.digit[k] for k, v in self.before_hund.items()}}

    def say(self, n):
        if n == 0:
            return self.zero
        h, r = divmod(n, 100)
        t, u = divmod(r, 10)
        out = []
        if h:
            out += [self.before_hund[h], self.hund]
        if t:
            out += [self.before_ten[t], self.ten]
        if u:
            out += [self.digit[u]]
        return " ".join(out)

    def bad_junk(self, rng):
        """A numeral with the digit word before a place word in its plain form, or with two words exchanged."""
        if rng.below(2):
            toks = self.say(12 + rng.below(self.cap - 12)).split(" ")
            where = [i for i, t in enumerate(toks) if t in self.plain]
            i = where[rng.below(len(where))]
            toks[i] = self.plain[toks[i]]
            return " ".join(toks)
        return super().bad_junk(rng)


class Fives(Tongue):
    """Counting in twenties, then in fives, then in ones: digit words 1 to 9, a word for twenty and one for five (a count and the word, a lone digit word for the ones, no
    count of 0), a link word between the twenties and the rest (the rest first in one variant); a word for zero that stands alone. Numbers up to 199."""

    cap = 199

    def __init__(self, label, words, param):
        super().__init__(label)
        self.first = param % 2 == 1
        self.digit = {i: words.number_word() for i in range(1, 10)}
        self.score, self.five, self.link, self.zero = (words.number_word() for _ in range(4))
        self.words = [self.zero] + list(self.digit.values()) + [self.score, self.five, self.link]

    def say(self, n):
        if n == 0:
            return self.zero
        q, r = divmod(n, 20)
        f, u = divmod(r, 5)
        top = [self.digit[q], self.score] if q else []
        rest = ([self.digit[f], self.five] if f else []) + ([self.digit[u]] if u else [])
        if top and rest:
            return " ".join(rest + [self.link] + top if self.first else top + [self.link] + rest)
        return " ".join(top or rest)


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


GRAMMARS = ["vigesimal", "biquinary", "doubling", "chunks", "sandhi", "fives"]
KINDS_OF_GRAMMAR = {"vigesimal": Vigesimal, "biquinary": Biquinary, "doubling": Doubling, "chunks": Chunks, "sandhi": Sandhi, "fives": Fives}
# the grammars of the first, second and third family for each seed (the seeds after the sixth begin again)
SCHEDULE = [("vigesimal", "doubling", "biquinary"), ("sandhi", "fives", "chunks"), ("fives", "biquinary", "doubling"), ("chunks", "vigesimal", "sandhi"), ("doubling", "sandhi", "vigesimal"), ("biquinary", "doubling", "fives")]


def describe(g, tongue):
    if g == "vigesimal":
        return "twenties with ten and fifteen words, " + ("leftover first" if tongue.low_first else "twenties first")
    if g == "biquinary":
        return "bi-quinary places with a five word for the units and another for the tens, " + ("the rest before the five" if tongue.after else "the five first")
    if g == "doubling":
        return "a word for one, 'times two' and 'and one' (a repeated word doubles again)"
    if g == "chunks":
        return "a word for each two-digit chunk, " + ("last two digits first" if tongue.low_first else "hundreds first")
    if g == "sandhi":
        return "decimal places with a digit word that changes before the ten and hundred words"
    return "twenties then fives then ones, " + ("rest first" if tongue.first else "twenties first")


def world_tongues(seed):
    """The word generator of a world (already used for the numerals) and its ways of writing: digits, three families of words, a fourth that no kind reads, a mix, raw words."""
    words = Words(seeded(seed, 2))
    row = SCHEDULE[(seed - 1) % 6]
    unused = [g for g in GRAMMARS if g not in row]
    tongues = {"d": DIGITS, "w": RAW}
    for i, (name, g) in enumerate(zip(("t1", "t2", "t3", "t4"), row + (unused[(seed - 1) % 3],))):
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
    count words are not functions of the numbers); `cover` names the family of words whose every word the lessons of this kind must show."""

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
    qf_k, qf_c, qf_d = 11 + rng.below(30), 100 + rng.below(800), 7 + rng.below(25)
    cp_k, cp_p = 5 + rng.below(40), PRIMES[rng.below(len(PRIMES))]
    mi_p = PRIMES[9 + rng.below(len(PRIMES) - 9)]
    while True:  # three linear forms of which each is the smallest somewhere
        tm = [1 + rng.below(6) for _ in range(6)]
        tm_forms = [lambda a, b, c: tm[0] * a + tm[1] * b, lambda a, b, c: tm[2] * b + tm[3] * c, lambda a, b, c: tm[4] * c + tm[5] * a]
        pts = list(itertools.product(range(41), repeat=3))[::7]
        wins = [sum(1 for p in pts if f(*p) == min(g(*p) for g in tm_forms)) for f in tm_forms]
        if min(wins) > len(pts) // 10 and len({tuple(tm[0:2]), tuple(tm[2:4]), tuple(tm[4:6])}) == 3:
            break
    while True:  # a five-state machine over binary digits that is no function of the remainder or of the number of 1 digits
        table = {s: (rng.below(5), rng.below(5)) for s in range(5)}
        outs = rng.shuffled(range(10))[:5]
        values = [run_dfa(a, table, outs) for a in range(128)]
        by_ones, by_rest = {}, [dict() for _ in range(9)]
        simple = all(by_ones.setdefault(bin(a).count("1"), values[a]) == values[a] for a in range(128))
        for m in range(2, 9):
            simple = simple or all(by_rest[m].setdefault(a % m, values[a]) == values[a] for a in range(128))
        if not simple and len(set(values)) >= 5:
            break
    ab_p, ab_c, ab_d, ab_e = 2 + rng.below(4), 1 + rng.below(9), 2 + rng.below(5), rng.below(5)
    cm = SMALL_MODS[rng.below(len(SMALL_MODS))]
    fk = 5 + rng.below(30)
    tp = 3 + rng.below(10)
    tq = tp
    while tq == tp or len({(a % tp) * (a % tq) for a in range(100)}) < 8:
        tq = 3 + rng.below(10)
    sl_p, sl_q = 2 + rng.below(8), 2 + rng.below(8)
    st_c = 1 + rng.below(9)
    nm = (5, 6, 8, 9)[rng.below(4)]

    def mk(name, role, arity, fn, rule, learn, tin="d", tout="d", lo=0, hi=LIMIT, **kw):
        """A kind whose domain is cut to what its numerals can write (the largest number of the grammar of its numbers, and of its answers)."""
        tins = tuple(tin) if isinstance(tin, (tuple, list)) else (tin,) * arity
        hi = min([hi] + [tongues[t].cap for t in tins if t in tongues])
        if tout in ("t1", "t2", "t3"):
            top, cap = lo, tongues[tout].cap
            while top < hi and max(fn(*p) for p in itertools.product(range(lo, top + 2), repeat=arity) if max(p) == top + 1) <= cap:
                top += 1
            hi = min(hi, top)
        return IKind(name, role, arity, fn, rule, learn, tin=tin, tout=tout, lo=lo, hi=hi, **kw)

    def farness(a):
        return abs(a - fk)

    def twinmod(a):
        return (a % tp) * (a % tq)

    def tokens(a):
        return t3.say(a).split(" ")

    newop = [
        mk("quadfloor", "newop", 1, lambda a: (a * a + qf_k * a + qf_c) // qf_d, f"(a * a + {qf_k} * a + {qf_c}) div {qf_d}, the number written as digits or as words of the first family (the lessons mix them), the answer in digits",
           "a floor of a quadratic with large constants: eleven nodes; the first differences of the answers climb by about 2 / d per step and the second differences are 0 or 1; the words are read through the lessons that tie the first family to digits", tin="m1", hi=99),
        mk("absring", "newop", 3, lambda a, b, c: abs(a - b) + abs(b - c) + abs(c - a), "|a - b| + |b - c| + |c - a|",
           "a sum of three absolute differences: seventeen nodes; it is twice the largest minus the smallest, so fix two of the numbers and the answers along the third are flat, then a line of slope 2 (or 0 and 2 in turn) beyond", hi=40),
        mk("cubepair", "newop", 2, lambda a, b: (a * a * a + cp_k * a * b + b * b) % cp_p, f"(a * a * a + {cp_k} * a * b + b * b) mod {cp_p}, a in digits and b spelled as words of the first family, the answer in digits",
           "a polynomial in two numbers taken modulo a prime: along the column the answers are a quadratic in b that wraps, along the row a cubic in a that wraps; the words are read through the lessons that tie the first family to digits", tin=("d", "t1"), hi=60),
        mk("trimin", "newop", 3, lambda a, b, c: min(tm[0] * a + tm[1] * b, tm[2] * b + tm[3] * c, tm[4] * c + tm[5] * a), f"min({tm[0]} * a + {tm[1]} * b, {tm[2]} * b + {tm[3]} * c, {tm[4]} * c + {tm[5]} * a), the numbers spelled as words of the second family, the answer in digits",
           "the smallest of three linear forms: fifteen nodes; each form is the smallest in a region and the answers are read off lines through the table; the words are read through the arithmetic lessons of the second family", tin="t2", hi=40),
        mk("dfa2", "newop", 1, lambda a: run_dfa(a, table, outs), f"the output of the state of a five-state machine after the binary digits of a (none for 0), read from the largest, output {outs}: transitions {table}",
           "a loop with a state over the binary digits: the answer for 2a and 2a + 1 follows from the state after the digits of a, so the table is built from the left and each entry from the entry at a div 2; the machine is no function of the remainder or of the number of 1 digits", hi=127),
        mk("cantorcount", "newop", 1, lambda a: cantor_count(a), "how many of the numbers 0 to a have no digit 2 in base 3, the number spelled as words of the third family, the answer in digits",
           "a running count of a property of the base-3 digits: the answers rise by 0 or 1 at each step and a run of numbers with a 2 does not rise; the words of the third family are fixed by nothing in the lessons, so only an assignment of numbers to the words that fits the table gives their meaning", tin="t3", hi=242),
        mk("tailpair", "newop", 2, lambda a, b: tail_pairs(a, b), "how many different pairs the walk (x, y) -> (y, |x - y|) visits from (a, b) before one comes round again, the numbers and the answer spelled as words of the second family",
           "a loop with a state that is a pair, with a record of the pairs seen: the walk shrinks like a subtractive Euclid algorithm and ends in the cycle (1, 1), (1, 0), (0, 1); the words are read through the arithmetic lessons of the second family", tin="t2", tout="t2", hi=40),
        mk("adjbranch", "newop", 1, lambda a: ab_p * a + ab_c if adjacent_ones(a) else a // ab_d + ab_e, f"{ab_p} * a + {ab_c} if the binary form of a has two 1 digits side by side, else (a div {ab_d}) + {ab_e}",
           "a conditional on a property of the binary digits that is neither a threshold nor a remainder: the numbers 3, 6, 7, 11, 12, 13, 14, 15, ... take the first branch (a line) and 0, 1, 2, 4, 5, 8, 9, 10, ... the second (a floor); the branches are a few nodes each", hi=199),
        mk("modinv", "newop", 1, lambda a: mod_inverse(a, mi_p), f"the smallest x from 1 with a * x = 1 modulo {mi_p} (a from 1 to 100)",
           "a number defined by an equation it satisfies: the answer for a is found by trying x until the product leaves remainder 1, so a lesson (a, x) is checked by one multiplication; the table has no pattern along a but pairs up (if x is the answer for a then a is the answer for x)", lo=1, hi=100),
        mk("boardsquares", "newop", 2, lambda a, b: board_squares(a, b), "the squares of every size, with sides along the lines, on a board of a by b cells, the numbers spelled as words of the first family, the answer in digits",
           "a sum over the sizes of square of the placements of that size: (a - k + 1) * (b - k + 1) for k from 1 to the smaller of the two; along a column it is a quadratic that turns into a line, and the table is symmetric; the words are read through the lessons that tie the first family to digits", tin="t1", lo=1, hi=40),
    ]
    compose = [
        mk("farthin", "compose", 1, lambda a: farness(twinmod(a)), f"farness(twinmod(a)): |(a mod {tp}) * (a mod {tq}) - {fk}|",
           "an earlier simple kind fed with the answer of another earlier simple kind: the inner answers are few, so the table is small, but a program for it needs more than seven nodes", hi=99),
        mk("dfafar", "compose", 1, lambda a: run_dfa(abs(a - fk), table, outs), "dfa2(farness(a)), the answer spelled as words of the first family",
           "an earlier hard kind fed with the answer of an earlier simple one: the machine run on the distance to a constant", tout="t1", hi=99),
    ]
    control = [
        mk("cubeprod", "control", 3, lambda a, b, c: a * b * c % cm, f"a * b * c mod {cm}", "seven nodes over the starting operations: a product of three taken modulo a constant", hi=15),
        mk("capsum", "control", 3, lambda a, b, c: min(a + b, c), "min(a + b, c)", "five nodes over the starting operations; taught in all six ways of asking with three numbers, so the ways can be seen to mean the same", bridge=True),
        mk("farness", "control", 1, farness, f"|a - {fk}|", "seven nodes over the starting operations (the larger of a - k and k - a); taught in all eight ways of asking with one number, so the ways can be seen to mean the same", hi=99, bridge=True),
        mk("twinmod", "control", 1, twinmod, f"(a mod {tp}) * (a mod {tq})", "seven nodes over the starting operations: the product of two remainders", hi=99),
        mk("slanted", "control", 2, lambda a, b: sl_p * a + b // sl_q, f"{sl_p} * a + b div {sl_q}", "seven nodes over the starting operations; taught in all eight ways of asking with two numbers, so the ways can be seen to mean the same", bridge=True),
        mk("decode1", "control", 1, lambda a: a, "a spelled as words of the first family (up to the largest number the grammar writes), the answer a in digits", "the words are lined up with the digits by the lessons: the lessons show every word of the family, so the words of a held-out numeral all occur in other lessons and the grammar has to be worked out from them", tin="t1", hi=t1.cap, cover="t1"),
        mk("encode1", "control", 1, lambda a: a, "a in digits, the answer a spelled as words of the first family", "the same lessons from the other side", tout="t1", hi=t1.cap, cover="t1"),
        mk("sumw1", "control", 2, lambda a, b: a + b, "a + b, the numbers and the answer spelled as words of the first family", "a few nodes over the starting operations once the words are read as numbers; the two kinds that tie the words to digits show how", tin="t1", tout="t1", hi=t1.cap // 2),
        mk("mixpair", "control", 2, lambda a, b: a + 2 * b, "a + 2 * b, a in digits and b spelled as words of the first family, the answer in digits", "five nodes over the starting operations once the words are read as numbers; every question has one number of each form", tin=("d", "t1"), hi=99),
        mk("mixwrite", "control", 2, lambda a, b: max(a, 2 * b), "max(a, 2 * b), a spelled as words of the first family and b in digits, the answer spelled as words of the first family", "four nodes over the starting operations once the words are read as numbers; the answer is in the form of the first number", tin=("t1", "d"), tout="t1", hi=t1.cap // 3),
        mk("step12", "control", 1, lambda a: 3 * a // 2 + st_c, f"(3 * a) div 2 + {st_c}, a spelled as words of the first family and the answer spelled as words of the second", "seven nodes over the starting operations once both families are read as numbers: the first through the kinds that tie it to digits, the second through the arithmetic of its own kinds", tin="t1", tout="t2", hi=t1.cap),
        mk("join2", "control", 2, lambda a, b: a + b, "a + b, the numbers and the answer spelled as words of the second family", "no lesson ties the words to digits; this kind and the halving fix the meaning through the arithmetic alone (the sum of small numbers names the words one by one, then the places appear in the sums that carry)", tin="t2", tout="t2", hi=t2.cap // 2, cover="t2"),
        mk("halve2", "control", 1, lambda a: a // 2, "a div 2, the number and the answer spelled as words of the second family", "the floor of the half of a number of the second family: a small table that fixes the order of the words and where the places carry", tin="t2", tout="t2", hi=t2.cap, cover="t2"),
        mk("rot3", "control", 1, lambda a: " ".join(tokens(a)[1:] + tokens(a)[:1]), "the words of the numeral (third family) with the first moved to the end", "a rearrangement of the words of the question whatever they mean", tin="t3", tout="w", hi=t3.cap, numeric=False),
        mk("middle3", "control", 1, lambda a: tokens(a)[len(tokens(a)) // 2], "the middle word of the numeral (third family; the later of the two in the middle for an even count), copied", "a copy of the word in the middle of the question whatever it means", tin="t3", tout="w", hi=t3.cap, numeric=False),
    ]
    unexplainable = [
        code_kind(rng, "dotcode", 17, 1, "d", "d"),
        code_kind(rng, "nearmod5", 0, 1, "d", "d", looks=nm),
        code_kind(rng, "scatterw1", min(300, t1.cap + 1), 1, "t1", "t1"),
        code_kind(rng, "scatterw2", 50, 1, "t2", "d"),
        code_kind(rng, "scatterpair3", min(200, t3.cap + 1), 2, "t3", "t3", hi=15),
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
        while fam - seen:
            best = max(candidates, key=lambda p: len((fam - seen) & words_of(tongues, k, p)))
            if not (fam - seen) & words_of(tongues, k, best):
                raise RuntimeError(f"no point shows the words of {k.cover} for {k.name}")
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
            notes += "; first family: the kinds that read and write it tie the words to digits"
        if "t2" in k.tins + (k.tout,):
            notes += "; second family: no kind reads or writes it, the meaning comes only through the arithmetic of the kinds computed over it"
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
    p = argparse.ArgumentParser(description="Write a ninth hidden-worlds adaptive exam (lessons.tsv, stream.tsv) for `baby exam adapt-drive`.")
    p.add_argument("dir")
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--compose-first", action="store_true", help="teach the compositions before the kinds they use (a control: reuse should then not help)")
    a = p.parse_args(argv)
    nl, ns = write(a.dir, a.seed, a.compose_first)
    print(f"wrote the hidden worlds I (seed {a.seed}) to {a.dir}: {nl} lessons and a stream of {ns} questions")


if __name__ == "__main__":
    main(sys.argv[1:])
