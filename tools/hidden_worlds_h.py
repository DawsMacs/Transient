#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""An eighth set of exam worlds: closed forms with three arguments and large constants, numbers spelled in six new kinds of numeral (bijective, balanced,
tally, twenties with a leftover, place-bound digit words, order-signed), several ways of asking built from pieces of each other, and traps that look like the
rewordings.

The same files, line protocol and shape of world as `tools/hidden_worlds.py` and its six successors (so `baby exam adapt-drive DIR COMMAND...` judges any
system on it). To keep the set a blind test the kinds are not described here.

    python3 tools/hidden_worlds_h.py DIR [--seed N] [--compose-first]

What is new in the way the exam is put:
  numerals   some kinds take and give their numbers as words, in one of six grammars (chosen by the seed; each world uses three of them, one per family of
             words). In the first family a pair of kinds ties the words to digits, so their meaning can be worked out; in the second no kind reads or writes
             the words and the meaning can only come through arithmetic done on them; in the third nothing in the lessons can fix the meaning. A learner is
             never told that the words are numbers. A few kinds are taught with digits and words mixed inside their lessons.
  wordings   every kind is taught in two or three of several ways of asking (three kinds in all of them). The facts that are held out are then asked in a way
             that was not taught for the kind, in a mix of the start of one taught way and the end of another, in a taught way with a filler word dropped, or in
             a taught way; next to them stand traps that look just like these rewordings and are not answerable.
  arguments  kinds with one, two and three numbers; questions that name the same number more than once, and numerals with a word that occurs more than once.
  refusals   next to the kinds without a rule, rows with a taught word in a place it was not taught for, two kinds at once, a new word, a word that is almost
             a kind word or a number word, runs of words that are no numeral of the grammar, words in an order no lesson used, and numbers in a form a kind
             was never taught with.

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

README = """THE HIDDEN WORLDS H (adaptive exam; the same files and protocol as `baby exam adapt-write`, written by tools/hidden_worlds_h.py)

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

ENDING = "dopa"  # shared by every kind word of this set (the earlier sets' words end in `sido`, `velu`, `ketu`, `ruvo`, `zane`, `yarn` and `tole`)


def seeded(seed, salt):
    """A generator for a seed and a purpose (the seed goes through one extra mix first, so that neighboring seeds are not the same stream a few steps on)."""
    return Rng(Rng(seed * 2000003 + salt * 104729 + 29).next())


# ---- the operations ----


def bit_double(a):
    """The binary digits of a, each written twice, read as a number."""
    return int("".join(c * 2 for c in bin(a)[2:]), 2)


def van_eck(a):
    """The a-th term (a from 1) of the sequence that starts with 0 and goes on with 0 if the last term is new and with how long ago it last occurred if not."""
    last, cur = {}, 0
    for n in range(1, a):
        nxt = n - last[cur] if cur in last else 0
        last[cur] = n
        cur = nxt
    return cur


def ant_black(a):
    """Cells that are black after a steps of an ant on an endless white grid: on a white cell it turns right, flips the cell and steps forward; on a black cell it turns left."""
    black, x, y, dx, dy = set(), 0, 0, 0, -1
    for _ in range(a):
        if (x, y) in black:
            black.remove((x, y))
            dx, dy = dy, -dx
        else:
            black.add((x, y))
            dx, dy = -dy, dx
        x, y = x + dx, y + dy
    return len(black)


def kaprekar_steps(n):
    """How many times the step 'digits in descending order minus digits in ascending order' (three digits, zeros in front) is applied before the value stays."""
    steps = 0
    while True:
        d = sorted("%03d" % n)
        nxt = int("".join(reversed(d))) - int("".join(d))
        if nxt == n:
            return steps
        n, steps = nxt, steps + 1


def digit_sum(a):
    return sum(int(c) for c in str(a))


def rising(a):
    s = str(a)
    return all(x < y for x, y in zip(s, s[1:]))


# ---- the words ----

CONSONANTS = "bdghklmnprstvz"
VOWELS = "aeiou"
LASTS = ["", "", "", "l", "n", "r", "s"]


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
    1: [Frame(t) for t in ["work out {w} on {a}", "tell [me] {w} of {a} [please]", "{a} run through {w} [now]", "put {a} through {w}", "by {w} the {a} becomes", "how much is {w} for {a}", "{a} meets {w} and yields", "consult {w} with {a}"]],
    2: [Frame(t) for t in ["{w} from {a} to {b}", "work out {w} for first {a} second {b}", "by {w} take {a} against {b}", "{a} meets {b} in {w}", "tell [me] {w} of left {a} right {b} [please]", "given {a} consult {w} with {b}", "{a} then {b} through {w} [now]", "consult {w} on {a} and {b}"]],
    3: [Frame(t) for t in ["{w} from {a} to {b} via {c}", "work out {w} for first {a} second {b} third {c}", "by {w} take {a} against {b} with {c}", "{a} meets {b} meets {c} in {w}", "tell [me] {w} of left {a} middle {b} right {c} [please]", "consult {w} on {a} and {b} and {c}"]],
}
# a start or an end that no way of asking has, for any kind
NEW_STARTS = [["reckon"], ["figure", "up"], ["weigh", "up"], ["ponder"]]
NEW_ENDS = [["regardless"], ["at", "last"], ["if", "so"], ["thereby"]]
JOIN = ["and", "then"]
BARE = ["{a}", "what is {a}", "tell me {a}"]
FRAME_WORDS = {t for fs in FRAMES.values() for f in fs for t in f.toks if t not in SLOTS and t != "{w}"}
FRAME_WORDS |= {t for ws in NEW_STARTS + NEW_ENDS + [JOIN] for t in ws} | {t for b in BARE for t in b.split(" ") if t not in SLOTS}
FILLERS = ["on", "with", "of", "for"]  # words of the frames that stand in for the kind's word in a trap


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
        return self.fresh(lambda r: "".join([CONSONANTS[r.below(14)], VOWELS[r.below(5)], CONSONANTS[r.below(14)], VOWELS[r.below(5)]]))

    def almost(self, word, keep=0):
        """A word one letter away from `word` (a vowel for a vowel, a consonant for a consonant, none of the last `keep` letters) that is no word of the exam."""
        while True:
            i = self.rng.below(len(word) - keep)
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


class Bijective(Tongue):
    """Positional numeration with no digit for zero: digits 1 to b, most significant first, a word of its own for the number zero that stands alone."""

    def __init__(self, label, words, param):
        super().__init__(label)
        self.base = (5, 6, 8)[param]
        self.zero = words.number_word()
        self.digit = {i: words.number_word() for i in range(1, self.base + 1)}
        self.words = [self.zero] + list(self.digit.values())

    def say(self, n):
        if n == 0:
            return self.zero
        ds = []
        while n:
            r = n % self.base or self.base
            ds.append(r)
            n = (n - r) // self.base
        return " ".join(self.digit[d] for d in reversed(ds))

    def bad_junk(self, rng):
        toks = self.say(self.some(rng)).split(" ")
        toks.insert(rng.below(len(toks) + 1), self.zero)
        return " ".join(toks)

    def bad_repeat(self, rng):
        return " ".join([self.zero, self.zero] + ([] if rng.below(2) else [self.digit[1 + rng.below(self.base)]]))


class Balanced(Tongue):
    """Positional numeration with digits from minus h to h (b = 2h + 1), a word for each (the word of minus one has nothing to do with the word of one), most significant first."""

    def __init__(self, label, words, param):
        super().__init__(label)
        self.base = (3, 5, 7)[param]
        self.h = (self.base - 1) // 2
        self.digit = {d: words.number_word() for d in range(-self.h, self.h + 1)}
        self.words = [self.digit[d] for d in range(-self.h, self.h + 1)]

    def say(self, n):
        if n == 0:
            return self.digit[0]
        ds = []
        while n:
            r = n % self.base
            if r > self.h:
                r -= self.base
            ds.append(r)
            n = (n - r) // self.base
        return " ".join(self.digit[d] for d in reversed(ds))

    def bad_junk(self, rng):
        toks = self.say(self.some(rng)).split(" ")
        return " ".join([self.digit[0]] + toks) if rng.below(2) else " ".join([self.digit[-1 - rng.below(self.h)]] + toks[1:])

    def bad_repeat(self, rng):
        return " ".join([self.digit[0], self.digit[0]] + ([] if rng.below(2) else [self.digit[1 + rng.below(self.h)]]))


class Tally(Tongue):
    """A word for each place value (1, b, b * b, ...) repeated as often as the digit says, the largest place first; a word for zero that stands alone."""

    def __init__(self, label, words, param):
        super().__init__(label)
        self.base = (4, 5, 6)[param]
        places = [1]
        while places[-1] * self.base <= 999:
            places.append(places[-1] * self.base)
        self.places = places[::-1]
        self.zero = words.number_word()
        self.place = {p: words.number_word() for p in self.places}
        self.words = [self.zero] + [self.place[p] for p in self.places]

    def say(self, n):
        if n == 0:
            return self.zero
        out = []
        for p in self.places:
            c, n = divmod(n, p)
            out += [self.place[p]] * c
        return " ".join(out)


class Roman(Tongue):
    """Added words of value 1, 5, 10, 50, 100 and 500, largest first; a smaller word in front of a larger one is taken off instead (always, or only for the units in the fully
    additive variant); a word for zero that stands alone."""

    cap = 899

    def __init__(self, label, words, param):
        super().__init__(label)
        self.strict = param % 2 == 1
        self.zero = words.number_word()
        self.sym = {v: words.number_word() for v in (1, 5, 10, 50, 100, 500)}
        s = self.sym
        steps = [(500, [500]), (400, [100, 500]), (100, [100]), (90, [10, 100]), (50, [50]), (40, [10, 50]), (10, [10]), (9, [1, 10]), (5, [5]), (4, [1, 5]), (1, [1])]
        self.steps = steps if self.strict else [x for x in steps if x[0] not in (400, 90, 40)]
        self.words = [self.zero] + [s[v] for v in (1, 5, 10, 50, 100, 500)]

    def say(self, n):
        if n == 0:
            return self.zero
        out = []
        for v, syms in self.steps:
            while n >= v:
                out += [self.sym[x] for x in syms]
                n -= v
        return " ".join(out)


class Place(Tongue):
    """Positional numeration with three places and a different set of digit words for each place (units, tens, hundreds), no zero in front; the places in order of size, largest
    first or smallest first."""

    def __init__(self, label, words, param):
        super().__init__(label)
        self.ascending = param % 2 == 1
        self.digit = [[words.number_word() for _ in range(10)] for _ in range(3)]
        self.words = self.digit[0] + self.digit[1] + self.digit[2][1:]

    def say(self, n):
        ds = [int(c) for c in reversed(str(n))]
        toks = [self.digit[p][d] for p, d in enumerate(ds)]
        return " ".join(toks if self.ascending else toks[::-1])


class Score(Tongue):
    """Counting in twenties with a leftover: digit words 1 to 9, a word for ten, a word for twenty and a link word; 'n twenties' is the count (a lone twenty word for one),
    then the link and the leftover below twenty, both parts written as ten and a digit (the leftover first in one variant); a word for zero that stands alone."""

    cap = 399

    def __init__(self, label, words, param):
        super().__init__(label)
        self.first = param % 2 == 1
        self.digit = {i: words.number_word() for i in range(1, 10)}
        self.ten, self.twenty, self.link, self.zero = (words.number_word() for _ in range(4))
        self.words = [self.zero] + list(self.digit.values()) + [self.ten, self.twenty, self.link]

    def small(self, m):
        return [self.digit[m]] if m < 10 else [self.ten] if m == 10 else [self.ten, self.digit[m - 10]]

    def say(self, n):
        if n == 0:
            return self.zero
        q, r = divmod(n, 20)
        top = ([] if q == 1 else self.small(q)) + [self.twenty] if q else []
        rest = self.small(r) if r else []
        if q and r:
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


GRAMMARS = ["bijective", "balanced", "tally", "score", "place", "roman"]
KINDS_OF_GRAMMAR = {"bijective": Bijective, "balanced": Balanced, "tally": Tally, "score": Score, "place": Place, "roman": Roman}


def describe(g, tongue):
    if g == "bijective":
        return f"bijective base {tongue.base}, no digit for zero"
    if g == "balanced":
        return f"balanced base {tongue.base}, digits from minus {tongue.h} to {tongue.h}"
    if g == "tally":
        return f"tally words, a word per place of base {tongue.base} repeated"
    if g == "score":
        return "twenties with a leftover, " + ("leftover first" if tongue.first else "twenties first")
    if g == "place":
        return "three places with a set of digit words for each place, " + ("smallest place first" if tongue.ascending else "largest place first")
    return "order-signed words, " + ("subtractive pairs for 4, 9, 40, 90, 400" if tongue.strict else "a subtractive pair for 4 and 9 only")


def world_tongues(seed):
    """The word generator of a world (already used for the numerals) and its ways of writing: digits, three families of words, a mix, raw words."""
    words = Words(seeded(seed, 2))
    start = (3 * (seed - 1) + (seed - 1) // 2) % 6
    tongues = {"d": DIGITS, "w": RAW}
    for i, name in enumerate(("t1", "t2", "t3")):
        g = GRAMMARS[(start + i) % 6]
        tongues[name] = KINDS_OF_GRAMMAR[g](f"{name}: " + g, words, ((seed - 1) // 2 + i) % 3)
        tongues[name].label = f"{name}: " + describe(g, tongues[name])
    for name in ("t1", "t2", "t3"):
        tongues[name].foreign = [w for other in ("t1", "t2", "t3") if other != name for w in tongues[other].words]
    tongues["m1"] = Mix("m1: digits or t1 words", tongues["t1"])
    return words, tongues


# ---- the kinds ----

PRIMES = [59, 61, 67, 71, 73, 79, 83, 89, 97, 101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193, 197, 199, 211]
SMALL_MODS = (13, 17, 19, 23, 29, 31, 37)  # all within the constants the reachability search offers


class HKind(Kind):
    """A kind of this set: also the numerals it takes and gives ('d' digits, 'm1' digits or first-family words, 't1' to 't3' the families of words, 'w' a raw word), its domain and
    how its lessons are planned. `numeric` tells whether the reachability check applies (the kinds that copy or count words are not functions of the numbers); `cover` names the
    family of words whose every word the lessons of this kind must show."""

    def __init__(self, name, role, arity, fn, rule, learn, tin="d", tout="d", lo=0, hi=LIMIT, bridge=False, numeric=True, cover=None):
        super().__init__(name, role, arity, fn, rule, lo=lo)
        self.learn, self.tin, self.tout, self.hi, self.bridge, self.numeric, self.cover = learn, tin, tout, hi, bridge, numeric, cover
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


def code_kind(rng, name, bound, arity, tin, tout, hi=LIMIT, looks=False):
    """An arbitrary code for each number or each pair of numbers (arity 2: 0 to 15). With `looks` it is a remainder with arbitrary exceptions: a rule for most points."""
    n = (hi + 1) ** arity
    if looks:
        while True:
            codes = [a % 7 if rng.below(100) >= 40 else rng.below(10) for a in range(n)]
            if sum(1 for a in range(n) if codes[a] != a % 7) >= 8:
                break
        rule = "a mod 7 for most numbers, an arbitrary code of 0 to 9 for the others: it looks like a remainder and is not: " + " ".join(map(str, codes))
    elif arity == 1:
        codes = [rng.below(bound) for _ in range(n)]
        rule = "an arbitrary code, no rule: " + " ".join(map(str, codes))
    else:
        codes = [rng.below(bound) for _ in range(n)]
        rule = f"an arbitrary code from 0 to {bound - 1} for each pair of numbers 0..{hi} (drawn from the seed), no rule"
    why = "cannot be learned: stays refused until told"
    if arity == 1:
        return HKind(name, "unexplainable", 1, lambda a, c=codes: c[a], rule, why, tin=tin, tout=tout, hi=hi)
    return HKind(name, "unexplainable", 2, lambda a, b, c=codes, w=hi + 1: c[a * w + b], rule, why, tin=tin, tout=tout, hi=hi)


def make_kinds(seed, tongues=None):
    """The kinds of a world, in four groups: control, newop, compose, unexplainable (each group shuffled by the seed)."""
    if tongues is None:
        tongues = world_tongues(seed)[1]
    t1, t2, t3 = tongues["t1"], tongues["t2"], tongues["t3"]
    rng = seeded(seed, 1)
    fm_m, tm_m = PRIMES[rng.below(len(PRIMES))], PRIMES[rng.below(len(PRIMES))]
    pq_k, pq_d, pq_m = 100 + rng.below(300), 7 + rng.below(25), 17 + rng.below(37)
    while True:  # a remainder of a quotient and a cube of it that take enough different values
        cm_m = SMALL_MODS[rng.below(len(SMALL_MODS))]
        mq_p, mq_q = 3 + rng.below(7), 5 + rng.below(5)
        if len({pow((a // mq_p) % mq_q, 3, cm_m) for a in range(100)}) >= 6:
            break
    lp_p = 2 + rng.below(8)
    lp_q = lp_p
    while lp_q == lp_p:
        lp_q = 2 + rng.below(8)
    sc_k = 6 + rng.below(7)
    asc_p, asc_c, asc_q, asc_r = 2 + rng.below(4), 1 + rng.below(9), 6 + rng.below(8), rng.below(5)

    def modquot(a):
        return (a // mq_p) % mq_q

    def cubemod(a):
        return a * a * a % cm_m

    def ascbranch(a):
        return asc_p * digit_sum(a) + asc_c if rising(a) else a % asc_q + asc_r

    def tokens(a):
        return t3.say(a).split(" ")

    newop = [
        HKind("fourthmod", "newop", 1, lambda a: a**4 % fm_m, f"a * a * a * a mod {fm_m}, the number written as digits or as words of the first family (the lessons mix them), the answer in digits",
              "a power taken modulo a large prime: the fourth power is a nine-node program (a product of two squares, then the remainder); the answers rise like a quartic and wrap, so the differences of the fourth order are constant until a wrap; the words are read through the lessons that tie the first family to digits", tin="m1", hi=63),
        HKind("trimod", "newop", 3, lambda a, b, c: (a * b + b * c + c * a) % tm_m, f"(a * b + b * c + c * a) mod {tm_m}",
              "the sum of the three products of pairs taken modulo a prime: thirteen nodes; fix two of the numbers and the answers along the third are a line that wraps; the three-number form is read off rows through the table", hi=15),
        HKind("prodquotmod", "newop", 2, lambda a, b: ((a * b + pq_k) // pq_d) % pq_m, f"((a * b + {pq_k}) div {pq_d}) mod {pq_m}, the numbers and the answer spelled as words of the second family",
              "a floor of a product, then a remainder: nine nodes with large constants; along a row the answers climb in steps of a * b / d and wrap at m; the words are read through the arithmetic lessons of the second family", tin="t2", tout="t2", hi=60),
        HKind("gcd3", "newop", 3, lambda a, b, c: math.gcd(math.gcd(a, b), c), "the greatest common divisor of a, b and c (all from 1)",
              "a loop that takes remainders (or a table over factors): the answer is 1 for most triples and rises where all three share a factor; the shared factor of the three is found by trial", lo=1, hi=24),
        HKind("median", "newop", 3, lambda a, b, c: sorted((a, b, c))[1], "the middle one of a, b and c in size",
              "a closed form of nine nodes with min and max (the larger of the smaller of the first two and the smaller of the larger of them and the third); the answer is one of the three and is read off by sorting", hi=31),
        HKind("bitdouble", "newop", 1, lambda a: bit_double(a), "the binary digits of a, each written twice, read as a number",
              "a loop over the binary digits that doubles each one: the answers are 3 times the next lower power sum; the table shows a spread of the bits of a (0, 3, 12, 15, 48, 51, ...)"),
        HKind("vaneck", "newop", 1, lambda a: van_eck(a), "the a-th term (a from 1) of the sequence that starts with 0 and goes on with 0 if the last term is new and with how long ago it last occurred if not: 0, 0, 1, 0, 2, 0, 2, 2, 1, 6, ..., the number and the answer spelled as words of the first family",
              "a number defined from the earlier ones and from where each earlier value last occurred: a table filled in from the left, one entry from the entries before it; the numbers of the first family are read through the lessons that tie them to digits", tin="t1", tout="t1", lo=1, hi=99),
        HKind("ant", "newop", 1, lambda a: ant_black(a), "the number of black cells after a steps of an ant on an endless white grid (on a white cell it turns right, flips the cell and steps forward; on a black cell it turns left, flips it and steps forward), the number and the answer spelled as words of the second family",
              "a simulation on a grid with a position and a heading: the count changes by one at every step and the table is read from running the ant; the words are read through the arithmetic lessons of the second family", tin="t2", tout="t2", hi=99),
        HKind("kaprekar", "newop", 1, lambda a: kaprekar_steps(a), "how many times the step 'digits in descending order minus digits in ascending order' (a with zeros in front to three digits) is applied before the value stays, the number spelled as words of the third family, the answer in digits",
              "a loop over the digits that sorts them twice and subtracts: the answers are 0 to 6, 0 for 0 and 495 and for no other number; the words of the third family are fixed by nothing in the lessons, so only an assignment of numbers to the words that fits the table gives their meaning", tin="t3", hi=t3.cap),
        HKind("ascbranch", "newop", 1, ascbranch, f"{asc_p} * (the sum of the decimal digits of a) + {asc_c} if the decimal digits of a strictly rise from left to right (a single digit counts), else (a mod {asc_q}) + {asc_r} (a to 99)",
              "a conditional on a property of the digits that is neither a threshold nor a remainder: the numbers 12, 13, ..., 19, 23, ..., 89 and the single digits take the first branch (a digit sum, a loop over the digits) and the rest take the other (a remainder); the branches are a few nodes each, the test is a loop over pairs of digits", hi=99),
    ]
    compose = [
        HKind("cubequot", "compose", 1, lambda a: cubemod(modquot(a)), f"cubemod(modquot(a)): (((a div {mq_p}) mod {mq_q}) * ((a div {mq_p}) mod {mq_q}) * ((a div {mq_p}) mod {mq_q})) mod {cm_m}",
              "an earlier simple kind fed with the answer of another earlier simple kind: the inner answers are few, so the table is small, but a program for it needs more than seven nodes", hi=99),
        HKind("vanbits", "compose", 1, lambda a: bit_double(van_eck(a)), "bitdouble(vaneck(a)), the number spelled as words of the first family, the answer in digits",
              "an earlier hard kind fed with the answer of another: the bits of the terms of the sequence, doubled", tin="t1", lo=1, hi=99),
    ]
    control = [
        HKind("muldiv3", "control", 3, lambda a, b, c: a * b // c, "a * b div c (c from 1)", "five nodes over the starting operations: a floor of a product over a third number", lo=1, hi=15),
        HKind("avg3", "control", 3, lambda a, b, c: (a + b + c) // 3, "(a + b + c) div 3", "seven nodes over the starting operations; taught in all six ways of asking with three numbers, so the ways can be seen to mean the same", hi=15, bridge=True),
        HKind("modquot", "control", 1, modquot, f"(a div {mq_p}) mod {mq_q}", "five nodes over the starting operations: a remainder of a quotient; taught in all eight ways of asking with one number, so the ways can be seen to mean the same", hi=99, bridge=True),
        HKind("cubemod", "control", 1, cubemod, f"a * a * a mod {cm_m}", "seven nodes over the starting operations: a power with a small exponent taken modulo a constant"),
        HKind("lopsided", "control", 2, lambda a, b: min(lp_p * a, lp_q * b), f"min({lp_p} * a, {lp_q} * b)", "seven nodes over the starting operations; taught in all eight ways of asking with two numbers, so the ways can be seen to mean the same", bridge=True),
        HKind("unspell1", "control", 1, lambda a: a, "a spelled as words of the first family (up to the largest number the grammar writes), the answer a in digits", "the words are lined up with the digits by the lessons: the lessons show every word of the family, so the words of a held-out numeral all occur in other lessons and the grammar has to be worked out from them", tin="t1", hi=t1.cap, cover="t1"),
        HKind("spell1", "control", 1, lambda a: a, "a in digits, the answer a spelled as words of the first family", "the same lessons from the other side", tout="t1", hi=t1.cap, cover="t1"),
        HKind("plus1", "control", 2, lambda a, b: a + b, "a + b, the numbers and the answer spelled as words of the first family", "a few nodes over the starting operations once the words are read as numbers; the two kinds that tie the words to digits show how", tin="t1", tout="t1", hi=t1.cap // 2),
        HKind("times1", "control", 1, lambda a: sc_k * a, f"{sc_k} * a, the number and the answer spelled as words of the first family", "one multiplication once the words are read as numbers", tin="t1", tout="t1", hi=t1.cap // sc_k),
        HKind("mixsum", "control", 2, lambda a, b: a + b, "a + b, each number written either in digits or as words of the first family (the lessons mix the two), the answer in digits", "a few nodes over the starting operations; the lessons that write the same sum with digits and with words tie the words to numbers", tin="m1", hi=99),
        HKind("plus2", "control", 2, lambda a, b: a + b, "a + b, the numbers and the answer spelled as words of the second family", "no lesson ties the words to digits; this kind and the double fix the meaning through the arithmetic alone (the sum of small numbers names the words one by one, then the places appear in the sums that carry)", tin="t2", tout="t2", hi=t2.cap // 2, cover="t2"),
        HKind("twice2", "control", 1, lambda a: 2 * a, "2 * a, the number and the answer spelled as words of the second family", "the double of a number of the second family: a small table that fixes the order of the words and where the places carry", tin="t2", tout="t2", hi=t2.cap // 2, cover="t2"),
        HKind("flip3", "control", 1, lambda a: " ".join(reversed(tokens(a))), "the words of the numeral (third family) in the opposite order", "a rearrangement of the words of the question whatever they mean", tin="t3", tout="w", hi=t3.cap, numeric=False),
        HKind("distinct3", "control", 1, lambda a: len(set(tokens(a))), "the number of different words of the numeral (third family), in digits", "a count of the different words of the question after the kind's word: the meaning of the words is not needed", tin="t3", hi=t3.cap, numeric=False),
        HKind("firstword3", "control", 1, lambda a: tokens(a)[0], "the first word of the numeral (third family), copied", "a copy of the first word whatever it means", tin="t3", tout="w", hi=t3.cap, numeric=False),
    ]
    unexplainable = [
        code_kind(rng, "scatterdig", 17, 1, "d", "d"),
        code_kind(rng, "nearmod", 0, 1, "d", "d", looks=True),
        code_kind(rng, "scatter1", 300, 1, "t1", "t1"),
        code_kind(rng, "scatter3", 200, 1, "t3", "t3"),
        code_kind(rng, "scatterpair2", 300, 2, "t2", "t2", hi=15),
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


def repeats(tongues, k, p):
    """Whether a question about the point p has a numeral with a word that occurs twice."""
    t = "t1" if k.tin == "m1" else k.tin
    for x in p:
        toks = tongues[t].say(x).split(" ")
        if len(set(toks)) < len(toks):
            return True
    return False


def words_of(tongues, k, p):
    """The numeral words of the question and the answer of a fact (for the coverage of a family)."""
    out = set()
    for x in p:
        out |= set(tongues["t1" if k.tin == "m1" else k.tin].say(x).split(" "))
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
    spelled = k.tin != "d" or k.tout != "d"
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
        return tongues[t].say(rng.below(300))

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
        return [arg(t or k.tin) for _ in range(k.arity)]

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
            retry(lambda: add(k.word, render(fs[rng.below(len(fs))].toks, k.word, [arg(k.tin) for _ in range(to)]), k.tout, "arity"))
    # numbers in a form the kind was not taught with: words nothing explains for the kinds over digits and the first two families, digits for the kinds over the third
    others = [k for k in live if k.tin != "t3"]
    over3 = [k for k in live if k.tin == "t3"]
    for k in rng.shuffled(others)[:8] + rng.shuffled(over3)[:4]:
        t = "t3" if k.tin != "t3" else "d"
        retry(lambda: add(k.word, render(frame_of(k).toks, k.word, nums(k, t)), k.tout, "other form"))
    # a bare number
    label = words.kind_word()
    for t in ("d", "t1", "t2", "t3", "t1", "t2"):
        retry(lambda: add(label, BARE[rng.below(3)].replace("{a}", arg(t)), t, "bare"))
    # runs of words that are no numeral: no word of the grammar in an order it does not allow, a word repeated, a word that is almost a number word
    numeral_kinds = [k for k in live if k.tin in ("t1", "t2", "t3", "m1")]
    for category in ("junk", "repeat", "almost"):
        for _ in range(6):
            k = rng.shuffled(numeral_kinds)[0]
            t = tongues["t1" if k.tin == "m1" else k.tin]

            def bad_numeral():
                if category == "junk":
                    return t.bad_junk(rng)
                if category == "repeat":
                    return t.bad_repeat(rng)
                toks = t.say(t.some(rng)).split(" ")
                j = rng.below(len(toks))
                toks[j] = words.almost(toks[j])
                return " ".join(toks)

            def make():
                args = nums(k)
                args[rng.below(k.arity)] = bad_numeral()
                return add(k.word, render(frame_of(k).toks, k.word, args), k.tout, category)

            retry(make)
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
        k = rng.shuffled(numeral_kinds)[0]
        t = tongues["t1" if k.tin == "m1" else k.tin]

        def inside():
            args = nums(k, "t1" if k.tin == "m1" else k.tin)
            j = rng.below(k.arity)
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
            return add(k.word, render(x.toks[: x.w + 1] + y.toks[y.w + 1 :], k.word, [arg(k.tin) for _ in range(to)]), k.tout, "mismatched ends")

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
    # the ways of asking: two or three taught for each kind (all of them for the three that carry the equivalences); the held-out facts of the rest are asked in a way that
    # was not taught for the kind, in a mix of the start of one taught way and the end of another, in a taught way with a filler left out, or in a taught way
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
        spelled = "; numerals: " + ", ".join(f"{x} {tongues[t].label}" for x, t in (("in", k.tin), ("out", k.tout)) if t != "d")
        notes = (spelled if spelled != "; numerals: " else "") + f"; taught in {len(ids)} ways of asking" + ("" if k.bridge else f", the held-out facts are asked {how}")
        if "t1" in (k.tin, k.tout) or k.tin == "m1":
            notes += "; first family: the kinds that read and write it tie the words to digits"
        if "t2" in (k.tin, k.tout):
            notes += "; second family: no kind reads or writes it, the meaning comes only through the arithmetic of the kinds computed over it"
        if "t3" in (k.tin, k.tout):
            notes += "; third family: nothing in the lessons fixes the meaning of the words, so rows that need it are expected to be refused"
        key.append([k.word, k.role, k.name, k.rule + notes, k.learn])
        taught, held, beyond = lesson_plan(k, rng, tongues)

        def texts(p, ways):
            nums = [numeral(tongues, k.tin, x, rng) for x in p]
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
    p = argparse.ArgumentParser(description="Write an eighth hidden-worlds adaptive exam (lessons.tsv, stream.tsv) for `baby exam adapt-drive`.")
    p.add_argument("dir")
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--compose-first", action="store_true", help="teach the compositions before the kinds they use (a control: reuse should then not help)")
    a = p.parse_args(argv)
    nl, ns = write(a.dir, a.seed, a.compose_first)
    print(f"wrote the hidden worlds H (seed {a.seed}) to {a.dir}: {nl} lessons and a stream of {ns} questions")


if __name__ == "__main__":
    main(sys.argv[1:])
