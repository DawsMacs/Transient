#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""A seventh set of exam worlds: closed forms with large constants, numbers spelled in grammars of their own (multipliers, least significant digit first,
subtractive words, mixed radix), questions that repeat words, and more ways of asking.

The same files, line protocol and shape of world as `tools/hidden_worlds.py` and its five successors (so `baby exam adapt-drive DIR COMMAND...` judges any
system on it). To keep the set a blind test the kinds are not described here.

    python3 tools/hidden_worlds_g.py DIR [--seed N] [--compose-first]

What is new in the way the exam is put:
  numerals   some kinds take and give their numbers as words, in one of four grammars (chosen by the seed; each world uses three of them, one per family of
             words). In the first family a pair of kinds ties the words to digits, so their meaning can be worked out; in the second no kind reads or writes
             the words and the meaning can only come through arithmetic done on them; in the third nothing in the lessons can fix the meaning. A learner is
             never told that the words are numbers.
  repeats    numerals whose words occur more than once, and questions that name the same number twice, so that a question read as a set of words is caught.
  wordings   every kind is taught in two or three of eight ways of asking (two kinds in all eight); some kinds are then asked in a way that was not taught
             for them but was taught for the two kinds that carry all eight.
  refusals   next to the kinds without a rule, rows with a taught word in a place it was not taught for, two kinds at once, a new word, a word that is
             almost a number word, runs of words that are no numeral of the grammar, and words in an order no lesson used.

Roles, as the judge groups them (the stream's role column):
  newop          needs more than the starting operations in a few nodes (12 kinds)
  compose        a small composition of kinds of this set, taught after them (3 kinds)
  control        expressible with the starting operations, or a copy, a count or a way of spelling, so a regression shows (13 kinds)
  unexplainable  an arbitrary code for each number or pair, or a code that looks like a rule: no rule, must stay refused until told (5 kinds)
  trap           a question that looks like an ordinary one and has no taught answer: must be refused (63 rows, no lessons)

The words and a few constants are drawn from the seed. Standard library only.
"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hidden_worlds import LIMIT, Kind, Rng  # noqa: E402

README = """THE HIDDEN WORLDS G (adaptive exam; the same files and protocol as `baby exam adapt-write`, written by tools/hidden_worlds_g.py)

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

ENDING = "tole"  # shared by every kind word of this set (the earlier sets' words end in `sido`, `velu`, `ketu`, `ruvo`, `zane` and `yarn`)
FRAME_WORDS = {"apply", "to", "under", "the", "of", "for", "given", "and", "with", "what", "is", "gives", "so", "applied"}

# ---- the operations ----


def seeded(seed, salt):
    """A generator for a seed and a purpose (the seed goes through one extra mix first, so that neighboring seeds are not the same stream a few steps on)."""
    return Rng(Rng(seed * 1000033 + salt * 7919 + 17).next())


def digit_sum(n):
    return sum(int(c) for c in str(n))


def sidon_terms(count):
    """The greedy sequence in which all sums of two terms (a term may be used twice) are different: 1, 2, 4, 8, 13, 21, 31, ..."""
    terms, sums = [1], {2}
    while len(terms) < count:
        m = terms[-1] + 1
        while any(m + t in sums for t in terms) or 2 * m in sums:
            m += 1
        sums.update(m + t for t in terms)
        sums.add(2 * m)
        terms.append(m)
    return terms


SIDON = sidon_terms(LIMIT)


def sidon(a):
    return SIDON[a - 1]


def look_say_length(a):
    """The number of digits of the a-th term of 1, 11, 21, 1211, 111221, ... (each term reads the one before it aloud as runs; a from 1)."""
    s = "1"
    for _ in range(a - 1):
        out, i = [], 0
        while i < len(s):
            j = i
            while j < len(s) and s[j] == s[i]:
                j += 1
            out.append(str(j - i) + s[i])
            i = j
        s = "".join(out)
    return len(s)


def sandpile_cells(a):
    """Grains placed on one cell of an endless grid; a cell with 4 or more gives one grain to each neighbor until none has 4 or more. The number of cells left with a grain."""
    grid = {(0, 0): a}
    todo = [(0, 0)] if a >= 4 else []
    while todo:
        x, y = todo.pop()
        n = grid.get((x, y), 0)
        if n < 4:
            continue
        grid[(x, y)] = n % 4
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            c = (x + dx, y + dy)
            grid[c] = grid.get(c, 0) + n // 4
            if grid[c] >= 4:
                todo.append(c)
    return sum(1 for v in grid.values() if v > 0)


def right_triangles(a):
    """The number of right triangles with whole sides and a hypotenuse of at most a (a from 1)."""
    n = 0
    for z in range(1, a + 1):
        for x in range(1, z):
            y2 = z * z - x * x
            y = int(y2**0.5 + 0.5)
            if y * y == y2 and y > x:
                n += 1
    return n


def sorted_cube(a):
    """The decimal digits of a * a * a put in order from the smallest to the largest, read as a number (0 first is dropped)."""
    return int("".join(sorted(str(a * a * a))))


def luhn_digit(n):
    """The check digit that the Luhn rule appends to the decimal digits of n: double every second digit from the right (starting with the last digit of n), take 9 off
    a double above 9, add all up, and give what makes the sum a multiple of 10."""
    s = 0
    for i, c in enumerate(reversed(str(n))):
        d = int(c)
        if i % 2 == 0:
            d = d * 2 - 9 if d > 4 else d * 2
        s += d
    return (10 - s % 10) % 10


def niven_branch(a, c):
    """a div s if a is divisible by s, the sum of its decimal digits, else c * (a mod s) + s (a from 1)."""
    s = digit_sum(a)
    return a // s if a % s == 0 else c * (a % s) + s


# ---- the numerals ----

CONSONANTS = "bdghklmnrstvxz"
VOWELS = "aeiou"
LASTS = ["", "", "", "l", "n", "r", "s"]


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

    def digit_word(self):
        return self.fresh(lambda r: "".join([CONSONANTS[r.below(14)], VOWELS[r.below(5)], CONSONANTS[r.below(14)]]))

    def almost(self, word):
        """A word one letter away from `word` (a vowel for a vowel, a consonant for a consonant) that is no word of the exam."""
        while True:
            i = self.rng.below(len(word))
            pool = VOWELS if word[i] in VOWELS else CONSONANTS
            w = word[:i] + pool[self.rng.below(len(pool))] + word[i + 1 :]
            if w not in self.used:
                self.used.add(w)
                return w


class Tongue:
    """How numbers are written in the questions and answers of a kind: digits, one of the numeral grammars, or raw words (a word of a numeral, copied)."""

    cap = 999  # the largest number the grammar writes in this set
    words = ()
    foreign = ()  # words of the other families, for runs that mix them

    def __init__(self, label):
        self.label = label

    def say(self, n, alt=None):
        return str(n)

    def parse(self, text):
        return int(text)

    def valid(self, text):
        try:
            self.parse(text)
            return True
        except ValueError:
            return False

    # ways to write something that is no numeral; each returns a string for which `parse` fails
    def bad_repeat(self, rng):
        raise NotImplementedError

    def bad_junk(self, rng):
        raise NotImplementedError

    def spliced(self, rng, n=None):
        """A numeral of this grammar with one of its words replaced by a word of another family."""
        while True:
            toks = self.say(n if n is not None else 12 + rng.below(980)).split(" ")
            toks[rng.below(len(toks))] = self.foreign[rng.below(len(self.foreign))]
            text = " ".join(toks)
            if not self.valid(text):
                return text


DIGITS = Tongue("digits")


class Raw(Tongue):
    def parse(self, text):
        return text


RAW = Raw("raw")


class Mult(Tongue):
    """Multiplier words: one word per digit 1 to base - 1, a word for the base and a word for its square that multiply the digit word before them (a missing digit
    word means 1), a word for zero that stands alone; parts of the number in the order of size, zeros and ones in front of a multiplier left out."""

    def __init__(self, label, words, param):
        super().__init__(label)
        self.base = (10, 12, 20)[param]
        self.zero = words.digit_word()
        self.digit = {i: words.digit_word() for i in range(1, self.base)}
        self.tens, self.hund = words.digit_word(), words.digit_word()
        self.value = {w: i for i, w in self.digit.items()}
        self.words = [self.zero] + list(self.digit.values()) + [self.tens, self.hund]

    def additive(self, n):
        b = self.base
        h, r = divmod(n, b * b)
        t, u = divmod(r, b)
        if h >= b:
            raise ValueError(n)
        out = []
        for coef, unit in ((h, self.hund), (t, self.tens)):
            if coef:
                if coef > 1:
                    out.append(self.digit[coef])
                out.append(unit)
        if u:
            out.append(self.digit[u])
        return out

    def read_additive(self, toks):
        total, pending = 0, 0
        for w in toks:
            if w in self.value:
                if pending:
                    raise ValueError(w)
                pending = self.value[w]
            elif w in (self.tens, self.hund):
                total += (pending or 1) * (self.base if w == self.tens else self.base**2)
                pending = 0
            else:
                raise ValueError(w)
        return total + pending

    def say(self, n, alt=None):
        return self.zero if n == 0 else " ".join(self.additive(n))

    def parse(self, text):
        toks = text.split(" ")
        if toks == [self.zero]:
            return 0
        n = self.read_additive(toks)
        if n > self.cap or self.say(n) != text:
            raise ValueError(text)
        return n

    def bad_repeat(self, rng):
        while True:
            toks = self.say(10 + rng.below(980)).split(" ")
            text = " ".join(toks + [toks[-1]])
            if not self.valid(text):
                return text

    def bad_junk(self, rng):
        while True:
            toks = self.say(100 + rng.below(880)).split(" ")
            if len(toks) >= 3:
                text = " ".join(toks[::-1])
                if not self.valid(text):
                    return text


class Sub(Mult):
    """Multipliers (base 10) and a subtractive word: a last digit of r or more is written as the next ten (or hundred) minus what is missing, as `twenty minus two`."""

    def __init__(self, label, words, param):
        super().__init__(label, words, 0)
        self.r = 8 - param
        self.minus = words.digit_word()
        self.words.append(self.minus)
        self.cap = 989 + self.r  # above this the rounded number would need a word for a thousand

    def say(self, n, alt=None):
        if n == 0:
            return self.zero
        u = n % 10
        if u >= self.r:
            return " ".join(self.additive(n + 10 - u) + [self.minus, self.digit[10 - u]])
        return " ".join(self.additive(n))

    def parse(self, text):
        toks = text.split(" ")
        if toks == [self.zero]:
            return 0
        if self.minus in toks:
            i = toks.index(self.minus)
            if i == 0 or len(toks) != i + 2 or toks[i + 1] not in self.value:
                raise ValueError(text)
            n = self.read_additive(toks[:i]) - self.value[toks[i + 1]]
        else:
            n = self.read_additive(toks)
        if n < 0 or n > self.cap or self.say(n) != text:
            raise ValueError(text)
        return n


class Lsd(Tongue):
    """A positional numeral with the least significant digit first, one word per digit 0 to base - 1 and a second word for two of the digits (either is read; the
    answers always use the first), no word for a last digit of 0 except for zero itself."""

    def __init__(self, label, words, param, rng):
        super().__init__(label)
        self.base = (7, 12, 20)[param]
        self.digit = [words.digit_word() for _ in range(self.base)]
        first = 1 + rng.below(self.base - 1)
        second = first
        while second == first:
            second = 1 + rng.below(self.base - 1)
        self.alt = {first: words.digit_word(), second: words.digit_word()}
        self.words = self.digit + list(self.alt.values())
        self.value = {w: i for i, w in enumerate(self.digit)}
        self.value.update({w: d for d, w in self.alt.items()})
        self.synonyms = True

    def say(self, n, alt=None):
        ds, m = [0] if n == 0 else [], n
        while m:
            ds.append(m % self.base)
            m //= self.base
        return " ".join(self.alt[d] if alt is not None and d in self.alt and alt.below(2) else self.digit[d] for d in ds)

    def parse(self, text):
        toks = text.split(" ")
        if any(w not in self.value for w in toks) or (len(toks) > 1 and self.value[toks[-1]] == 0):
            raise ValueError(text)
        n = sum(self.value[w] * self.base**i for i, w in enumerate(toks))
        if n > self.cap:
            raise ValueError(text)
        return n

    def bad_repeat(self, rng):
        w = self.foreign[rng.below(len(self.foreign))]
        return " ".join(self.say(1 + rng.below(980)).split(" ") + [w, w])

    def bad_junk(self, rng):
        return self.spliced(rng)


class Mixed(Tongue):
    """Three digit words, least significant first, with a different count of digits at each place (7, 12 and 12 in some order): every number is written with three words."""

    def __init__(self, label, words, param):
        super().__init__(label)
        self.radix = [(7, 12, 12), (12, 7, 12), (12, 12, 7)][param]
        self.digit = [words.digit_word() for _ in range(12)]
        self.words = list(self.digit)
        self.value = {w: i for i, w in enumerate(self.digit)}

    def say(self, n, alt=None):
        out = []
        for r in self.radix[:2]:
            out.append(self.digit[n % r])
            n //= r
        out.append(self.digit[n])
        return " ".join(out)

    def parse(self, text):
        toks = text.split(" ")
        if len(toks) != 3 or any(w not in self.value for w in toks):
            raise ValueError(text)
        d = [self.value[w] for w in toks]
        if any(x >= r for x, r in zip(d, self.radix)):
            raise ValueError(text)
        n = d[0] + self.radix[0] * (d[1] + self.radix[1] * d[2])
        if n > self.cap:
            raise ValueError(text)
        return n

    def bad_repeat(self, rng):
        toks = self.say(rng.below(1000)).split(" ")
        return " ".join(toks + [toks[rng.below(3)]])

    def bad_junk(self, rng):
        i = self.radix.index(7)
        toks = self.say(rng.below(1000)).split(" ")
        toks[i] = self.digit[7 + rng.below(5)]
        return " ".join(toks)


GRAMMARS = ["mult", "lsd", "sub", "mixed"]


def make_tongue(kind, label, words, param, rng):
    if kind == "mult":
        return Mult(f"{label}: multiplier words, base {(10, 12, 20)[param]}", words, param)
    if kind == "lsd":
        return Lsd(f"{label}: least significant digit first, base {(7, 12, 20)[param]}, two digits with a second word", words, param, rng)
    if kind == "sub":
        return Sub(f"{label}: multiplier words with a subtractive word from a last digit of {8 - param}", words, param)
    return Mixed(f"{label}: three words, least significant first, places of {'x'.join(map(str, [(7, 12, 12), (12, 7, 12), (12, 12, 7)][param]))}", words, param)


def world_tongues(seed):
    """The word generator of a world (already used for the numerals) and its five ways of writing: digits, three families of words, raw words."""
    words, rng = Words(seeded(seed, 2)), seeded(seed, 4)
    first = (seed - 1) % 4
    left = [g for g in range(4) if g not in (first, (first + 1) % 4)]
    third = left[0] if GRAMMARS[left[0]] != "mixed" else left[1]  # a numeral of fixed width would leave the count of words and the last word with nothing to tell
    tongues = {"d": DIGITS, "w": RAW}
    for i, (name, g) in enumerate((("t1", first), ("t2", (first + 1) % 4), ("t3", third))):
        param = ((seed - 1) // 4 + i) % 3
        tongues[name] = make_tongue(GRAMMARS[g], name, words, param, rng)
    for name in ("t1", "t2", "t3"):
        tongues[name].foreign = [w for other in ("t1", "t2", "t3") if other != name for w in tongues[other].words]
    return words, tongues


# ---- the kinds ----

HALVES = [(2, 3), (2, 5), (3, 4), (3, 5), (4, 5), (2, 7)]
MIXES = [(3, 2), (2, 5), (4, 3), (5, 2), (3, 4)]
PRIMES = [59, 61, 67, 71, 73, 79, 83, 89, 97, 101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193, 197, 199, 211]
WIDE_FAR = 8
TEACH_ALL = 8  # the kinds taught in every way of asking carry the equivalences between the ways


class GKind(Kind):
    """A kind of this set: also the numerals it takes and gives ('d' digits, 't1' to 't3' the families of words, 'w' a raw word), its domain and how its lessons are
    planned. `numeric` tells whether the reachability check applies (the kinds that copy or count words are not functions of the numbers)."""

    def __init__(self, name, role, arity, fn, rule, learn, tin="d", tout="d", lo=0, hi=LIMIT, near=None, taught=None, held=None, beyond=None, bridge=False, numeric=True):
        super().__init__(name, role, arity, fn, rule, lo=lo)
        self.learn, self.tin, self.tout, self.hi, self.bridge, self.numeric = learn, tin, tout, hi, bridge, numeric
        wide = hi > 99
        if arity == 1:
            self.near = near if near is not None else (min(599, hi * 3 // 5) if hi > LIMIT else 19)
            self.held, self.beyond = held if held is not None else (10 if wide else 4), beyond if beyond is not None else (WIDE_FAR if wide else 10)
            self.taught = taught if taught is not None else (110 if wide else None)
        else:
            self.near = near if near is not None else (hi * 3 // 5 if wide else 15)
            self.held, self.beyond = held if held is not None else (10 if wide else 8), beyond if beyond is not None else (10 if wide else 14)
            self.taught = taught if taught is not None else (70 if wide else 40 if bridge else 60)


def code_kind(rng, name, bound, arity, tin, tout, hi=LIMIT, looks=False):
    """An arbitrary code for each number (arity 1) or each pair of numbers 0..31 (arity 2): no rule. With `looks` it is 4a plus a code of 0 to 2: a line, but not one."""
    n = (hi + 1) ** arity
    if looks:
        codes = [4 * a + rng.below(3) for a in range(n)]
        rule = "4 * a + a code of 0 to 2 for each a: it looks like a line and is not: " + " ".join(map(str, codes))
    elif arity == 1:
        codes = [rng.below(bound) for _ in range(n)]
        rule = "an arbitrary code, no rule: " + " ".join(map(str, codes))
    else:
        codes = [rng.below(bound) for _ in range(n)]
        rule = f"an arbitrary code from 0 to {bound - 1} for each pair of numbers 0..{hi} (drawn from the seed), no rule"
    why = "cannot be learned: stays refused until told"
    if arity == 1:
        return GKind(name, "unexplainable", 1, lambda a, _b, c=codes: c[a], rule, why, tin=tin, tout=tout, hi=hi)
    return GKind(name, "unexplainable", 2, lambda a, b, c=codes, w=hi + 1: c[a * w + b], rule, why, tin=tin, tout=tout, hi=hi)


def make_kinds(seed, tongues=None):
    """The kinds of a world, in four groups: control, newop, compose, unexplainable (each group shuffled by the seed)."""
    if tongues is None:
        tongues = world_tongues(seed)[1]
    t1, t2, t3 = tongues["t1"], tongues["t2"], tongues["t3"]
    rng = seeded(seed, 1)
    hv_p, hv_q = HALVES[rng.below(len(HALVES))]
    mx_p, mx_q = MIXES[rng.below(len(MIXES))]
    lp_m = (7, 9, 10, 11, 12)[rng.below(5)]
    sc_k = 6 + rng.below(7)
    while True:  # a floor of a ratio of two linear forms that takes enough different values
        rf = [7 + rng.below(8), 2 + rng.below(7), 1 + rng.below(9), 1 + rng.below(3), 1 + rng.below(3), 2 + rng.below(4)]
        if len({(rf[0] * a + rf[1] * b + rf[2]) // (rf[3] * a + rf[4] * b + rf[5]) for a in range(LIMIT + 1) for b in range(LIMIT + 1)}) >= 6:
            break
    tm_p, tm_q, tm_m = 11 + rng.below(27), 11 + rng.below(27), PRIMES[9 + rng.below(len(PRIMES) - 9)]
    qp_c, qp_d, qp_m = 2 + rng.below(8), 2 + rng.below(4), PRIMES[rng.below(9)]
    mf_p, mf_q, mf_c = 2 + rng.below(5), 2 + rng.below(5), 1 + rng.below(9)
    bq_k, bq_c, bq_m = 30000 + rng.below(70000), 1 + rng.below(6), (83, 89, 97)[rng.below(3)]
    nv_c = 2 + rng.below(3)

    def halves(a):
        return a // hv_p + a // hv_q

    def quadpair(a, b):
        return (a * a + qp_c * a * b + qp_d * b * b) % qp_m

    def luhnsum(a, b):
        return luhn_digit(a + b)

    cap1, cap2, cap3 = t1.cap, t2.cap, t3.cap
    k1, k2 = cap1 // sc_k, cap1 // 2
    newop = [
        GKind("ratiofloor", "newop", 2, lambda a, b: (rf[0] * a + rf[1] * b + rf[2]) // (rf[3] * a + rf[4] * b + rf[5]), f"({rf[0]} * a + {rf[1]} * b + {rf[2]}) div ({rf[3]} * a + {rf[4]} * b + {rf[5]})", "a floor of a ratio of two linear forms: the answers climb along a with a slope that flattens (the larger a, the more the ratio sits near its limit) and fall along b; fit the two forms from where the steps fall, or a nineteen-node program"),
        GKind("twomult", "newop", 2, lambda a, b: (tm_p * a + tm_q * b) % tm_m, f"({tm_p} * a + {tm_q} * b) mod {tm_m}", f"a sum of two multiples taken modulo a prime: along a row the answers rise by {tm_p % tm_m} and wrap at {tm_m}, along a column by {tm_q % tm_m}; a nine-node program with large constants"),
        GKind("quadpair", "newop", 2, quadpair, f"(a * a + {qp_c} * a * b + {qp_d} * b * b) mod {qp_m}, the numbers and the answer spelled as words of the second family", "a quadratic form in two numbers taken modulo a prime: rows and columns are quadratics wrapped at the prime, so the second differences along a row are constant modulo the prime; a seventeen-node program; the words are read through the arithmetic lessons of the second family", tin="t2", tout="t2"),
        GKind("minform", "newop", 2, lambda a, b: min(mf_p * a + b, a + mf_q * b) + mf_c, f"min({mf_p} * a + b, a + {mf_q} * b) + {mf_c}, the numbers and the answer spelled as words of the second family", "the smaller of two linear forms plus a constant: the plane is two flat pieces that meet along a line; each piece is read off a row and a column; a thirteen-node program; the words are read through the arithmetic lessons of the second family", tin="t2", tout="t2"),
        GKind("bigquot", "newop", 1, lambda a, _b: (bq_k // (a + bq_c)) % bq_m, f"({bq_k} div (a + {bq_c})) mod {bq_m}, the number and the answer spelled as words of the first family", "a large constant divided by a shifted a, then taken modulo a prime: the quotient falls like a hyperbola and the remainder scrambles it; the constant has five digits, which the small constants cannot name, so a program needs ten or more nodes", tin="t1", tout="t1"),
        GKind("sidon", "newop", 1, lambda a, _b: sidon(a), "the a-th term (a from 1) of the greedy sequence in which every sum of two terms (a term may be used twice) is different: 1, 2, 4, 8, 13, 21, 31, 45, ...", "a number defined from all the earlier ones: each term is the smallest number above the last one whose sums with all the earlier terms (and with itself) are new; the table shows gaps that grow", lo=1),
        GKind("lookandsay", "newop", 1, lambda a, _b: look_say_length(a), "the number of digits of the a-th term (a from 1) of 1, 11, 21, 1211, 111221, ...: each term reads the one before it aloud as runs of equal digits", "a string rewritten by a rule that counts runs; the lengths grow by about three tenths each step and are read off by writing the strings out", lo=1),
        GKind("rightcount", "newop", 1, lambda a, _b: right_triangles(a), "the number of right triangles with whole sides and a hypotenuse of at most a (a from 1), the number and the answer spelled as words of the first family", "a running total of a property of pairs: the answer rises where a is the hypotenuse of a triple (5, 10, 13, 15, 17, 20, 25, 26, 29, 30) and stays where it is not; the property is a search over pairs", tin="t1", tout="t1", lo=1),
        GKind("sortcube", "newop", 1, lambda a, _b: sorted_cube(a), "the decimal digits of a * a * a put in order from the smallest to the largest, read as a number", "a string made from the digits of a cube, sorted: cube first, then the digits in order (the zeros fall away at the front)"),
        GKind("sandpile", "newop", 1, lambda a, _b: sandpile_cells(a), "a grains on one cell of an endless grid; a cell with 4 or more gives one grain to each of its four neighbors until none has 4 or more; the answer is the number of cells left with a grain, the number spelled as words of the third family", "a grid in which a cell reacts to its count: the answers are a small rule over a pile that spreads, and the table is read from a simulation; the words of the third family are fixed by nothing but this table, which rises in steps, so only an assignment of numbers to the words that fits it gives their meaning", tin="t3"),
        GKind("luhn", "newop", 1, lambda a, _b: luhn_digit(a), "the check digit that the Luhn rule appends to the decimal digits of a (double every second digit from the right starting with the last, take 9 off a double above 9, add up, give what makes a multiple of 10), the number spelled as words of the first family", "a loop over the digits with a state (the place, odd or even) and a carry-like fix on the doubles; three digits make it a short fixed loop, and the numbers of the first family are read through the lessons that tie them to digits", tin="t1", hi=cap1),
        GKind("niven", "newop", 1, lambda a, _b: niven_branch(a, nv_c), f"a div s if a is divisible by s, the sum of the decimal digits of a, else {nv_c} * (a mod s) + s (a from 1 to 99)", "a conditional on a property that is neither a threshold nor a remainder: whether a number is divisible by its own digit sum; the two branches are a few nodes each and the digit sum for two digits is seven", lo=1, hi=99),
    ]
    compose = [
        GKind("halvestriple", "compose", 2, lambda a, b: halves(a + b + min(a, b)), "the floors-of-a control applied to the answer of the control that adds both numbers and the smaller one", "an earlier simple kind fed with the answer of another earlier simple kind"),
        GKind("luhnsum", "compose", 2, luhnsum, "luhn(a + b), the numbers spelled as words of the first family", "an earlier hard kind fed with the answer of an earlier sum of the first family", tin="t1", hi=k2),
        GKind("quadtwice", "compose", 2, lambda a, b: 2 * quadpair(a, b), "2 * quadpair(a, b), the numbers and the answer spelled as words of the second family", "the double of the second family applied to an earlier hard kind", tin="t2", tout="t2"),
    ]
    control = [
        GKind("halves", "control", 1, lambda a, _b: halves(a), f"a div {hv_p} + a div {hv_q}", "seven nodes over the starting operations; taught in all eight ways of asking, so the ways can be seen to mean the same", bridge=True),
        GKind("mix3", "control", 2, lambda a, b: mx_p * a + mx_q * b, f"{mx_p} * a + {mx_q} * b", "seven nodes over the starting operations: a plane through the origin, read off a row and a column", bridge=True),
        GKind("lastprod", "control", 2, lambda a, b: a * b % lp_m, f"(a * b) mod {lp_m}", "five nodes over the starting operations"),
        GKind("triple", "control", 2, lambda a, b: a + b + min(a, b), "a + b + min(a, b)", "seven nodes over the starting operations (taught for diagonal pairs too)"),
        GKind("read1", "control", 1, lambda a, _b: a, "a spelled as words of the first family (up to the largest number the grammar writes, 995 to 999), the answer a in digits", "the words are lined up with the digits by the lessons: every word of a held-out numeral occurs in other lessons of the first family; the grammar has to be worked out from them", tin="t1", hi=cap1),
        GKind("write1", "control", 1, lambda a, _b: a, "a in digits, the answer a spelled as words of the first family", "the same lessons from the other side", tout="t1", hi=cap1),
        GKind("sum1", "control", 2, lambda a, b: a + b, "a + b, the numbers and the answer spelled as words of the first family", "a few nodes over the starting operations once the words are read as numbers; the two kinds that tie the words to digits show how", tin="t1", tout="t1", hi=k2),
        GKind("scale1", "control", 1, lambda a, _b: sc_k * a, f"{sc_k} * a, the number and the answer spelled as words of the first family", "one multiplication once the words are read as numbers", tin="t1", tout="t1", hi=k1),
        GKind("sum2", "control", 2, lambda a, b: a + b, "a + b, the numbers and the answer spelled as words of the second family", "no lesson ties the words to digits; this kind and the double fix the meaning through the arithmetic alone (the sum of small numbers names the words one by one, then the place values appear in the sums that carry)", tin="t2", tout="t2", hi=cap2 // 2),
        GKind("double2", "control", 1, lambda a, _b: 2 * a, "2 * a, the number and the answer spelled as words of the second family", "the double of a number of the second family: a small table that fixes the order of the words and where the places carry", tin="t2", tout="t2", hi=cap2 // 2),
        GKind("second3", "control", 2, lambda _a, b: b, "the second of the two numerals (words of the third family), copied", "a copy: the answer is the second numeral of the question, whatever the words mean", tin="t3", tout="t3", hi=299, numeric=False),
        GKind("size3", "control", 1, lambda a, _b: len(t3.say(a).split(" ")), "the number of words of the numeral (words of the third family), in digits", "a count of the words of the question after the kind's word, repeats counted: the meaning of the words is not needed", tin="t3", hi=cap3, numeric=False),
        GKind("lastword3", "control", 1, lambda a, _b: t3.say(a).split(" ")[-1], "the last word of the numeral (words of the third family), copied", "a copy of the last word whatever it means", tin="t3", tout="w", hi=cap3, numeric=False),
    ]
    unexplainable = [
        code_kind(rng, "noisedig", 23, 1, "d", "d"),
        code_kind(rng, "noisyline", 0, 1, "d", "d", looks=True),
        code_kind(rng, "noise2", 300, 1, "t2", "t2"),
        code_kind(rng, "noise3", 200, 1, "t3", "t3"),
        code_kind(rng, "pairnoise3", 100, 2, "t3", "t3"),
    ]
    return rng.shuffled(control), rng.shuffled(newop), rng.shuffled(compose), unexplainable


# ---- the ways of asking ----

UNARY = ["apply {w} to {a}", "{w} {a}", "{a} under {w}", "the {w} of {a}", "{w} for {a}", "given {a} what is {w}", "{w} {a} gives", "{w} applied to {a}"]
BINARY = ["apply {w} to {a} and {b}", "{a} {w} {b}", "the {w} of {a} with {b}", "{w} for {a} and {b}", "given {a} and {b} what is {w}", "{a} {w} {b} gives", "so {a} {w} {b}", "{w} applied to {a} with {b}"]
# orders of the same words that no lesson used and that have no taught reading
SCRAMBLED_UNARY = ["{a} {w} of", "of {a} {w}", "{a} to {w} apply", "gives {w} {a} so"]
SCRAMBLED_BINARY = ["{a} {b} {w}", "{w} {b} {a} with", "so {w} {a} {b}", "{a} of {w} and {b}"]
TWO_KINDS = ["{w1} {a} and {w2} {a}", "apply {w1} to {a} and apply {w2} to {a}"]


def frames_of(arity):
    return UNARY if arity == 1 else BINARY


def put(frame, word, *args):
    """A question: a way of asking with the word and the numbers (already written in their numerals) in it."""
    out = frame.replace("{w}", word)
    for slot, x in zip("ab", args):
        out = out.replace("{" + slot + "}", x)
    return out


# ---- the exam ----


def numeral(tongues, t, n, rng):
    """A number as the questions write it: a family with second words picks them (when a fact is built), the rest write one way."""
    tongue = tongues[t]
    return tongue.say(n, rng if getattr(tongue, "synonyms", False) and t != "t3" else None)


def repeats(tongues, k, p):
    """Whether a question about the point p has a numeral with a word that occurs twice."""
    for x in p:
        toks = tongues[k.tin].say(x).split(" ")
        if len(set(toks)) < len(toks):
            return True
    return False


def lesson_plan(k, rng, tongues):
    """(taught, held, beyond) as lists of argument tuples. For a kind over words the number 0 (the word for zero) is always taught, alone or in a pair."""
    lo, hi, near_hi = k.lo, k.hi, min(k.near, k.hi)
    spelled = "d" != k.tin or "d" != k.tout
    if k.arity == 1:
        near, far = [(a,) for a in range(lo, near_hi + 1)], [(a,) for a in range(near_hi + 1, hi + 1)]
        near, far = rng.shuffled(near), rng.shuffled(far)
        anchors = [(lo,)] if spelled else []
        pool = [p for p in near if p not in anchors]
        if k.tin != "d" and k.hi > 99:  # numerals with a repeated word are among the held-out ones, whatever the grammar
            rep = [p for p in pool if repeats(tongues, k, p)][:4]
            held = rep + [p for p in pool if p not in rep][: k.held - 4]
        else:
            held = pool[: k.held]
        rest = anchors + [p for p in pool if p not in held]
        return (rest if k.taught is None else rest[: k.taught]), held, far[: k.beyond]
    near, far = [], []
    for a in range(lo, hi + 1):
        for b in range(lo, hi + 1):
            (near if a <= near_hi and b <= near_hi else far).append((a, b))
    near, far = rng.shuffled(near), rng.shuffled(far)
    anchors = [(lo, lo), next(p for p in near if p[0] == lo and p[1] > lo), next(p for p in near if p[1] == lo and p[0] > lo)] if spelled else []
    pool = [p for p in near if p not in anchors]
    diag = [p for p in pool if p[0] == p[1]]
    held = diag[:3] + [p for p in pool if p[0] != p[1]][: k.held - 3]
    rest = anchors + [p for p in pool if p not in held]
    fdiag = [p for p in far if p[0] == p[1]]
    beyond = fdiag[:2] + [p for p in far if p[0] != p[1]][: k.beyond - 2]
    return rest[: k.taught], held, beyond


def trap_rows(rng, order, tongues, words, used):
    """Rows with no taught answer: (unit, question, truth). `used` holds every question already in the exam."""
    top = {"d": 100, "t1": 1000, "t2": 1000, "t3": 1000, "w": 1}
    rows = []

    def truth(t):
        return tongues[t].say(rng.below(top[t]))

    def add(unit, question, tout):
        if question not in used:
            used.add(question)
            rows.append((unit, question, truth(tout)))
            return True
        return False

    def arg(t):
        return tongues[t].say(rng.below(32))

    live = [k for k in order if k.role != "unexplainable" and k.tout != "w"]
    unary = [k for k in live if k.arity == 1]
    binary = [k for k in live if k.arity == 2]
    # two kinds at once, over numbers of one family
    for group, n in (("d", 3), ("t1", 3), ("t3", 2)):
        pool = [k for k in unary if k.tin == group]
        done = 0
        while done < n:
            k1, k2 = rng.shuffled(pool)[:2]
            frame = TWO_KINDS[rng.below(2)]
            done += add(k1.word, frame.replace("{w1}", k1.word).replace("{w2}", k2.word).replace("{a}", arg(group)), k1.tout)
    # a new word
    fresh = [words.kind_word() for _ in range(8)]
    for i, w in enumerate(fresh):
        t = ("d", "t1", "t2", "t3")[i % 4]
        frames = UNARY if i < 4 else BINARY
        while not add(w, put(frames[rng.below(4)], w, *[arg(t) for _ in range(1 if i < 4 else 2)]), t):
            pass
    # the wrong number of numbers
    for k in rng.shuffled(unary)[:4]:
        while not add(k.word, put(BINARY[rng.below(4)], k.word, arg(k.tin), arg(k.tin)), k.tout):
            pass
    for k in rng.shuffled(binary)[:4]:
        while not add(k.word, put(UNARY[rng.below(4)], k.word, arg(k.tin)), k.tout):
            pass
    # a taught word with numbers of a family it was not taught with
    for k in rng.shuffled(unary)[:4] + rng.shuffled(binary)[:4]:
        t = [x for x in ("d", "t1", "t2", "t3") if x != k.tin][rng.below(3)]
        while not add(k.word, put(frames_of(k.arity)[rng.below(4)], k.word, *[arg(t) for _ in range(k.arity)]), k.tout):
            pass
    # a bare number
    label = words.kind_word()
    for t in ("d", "t1", "t2", "t3", "t1", "t2", "t3"):
        while not add(label, ["what is {a}", "{a}"][rng.below(2)].replace("{a}", arg(t)), t):
            pass
    # runs of words that are no numeral: no word of the grammar in an order it does not allow, a word repeated, a word that is almost a number word
    numeral_kinds = [k for k in live if k.tin in ("t1", "t2", "t3")]
    for category in ("junk", "repeat", "almost"):
        for i in range(6):
            k = rng.shuffled(numeral_kinds)[0]
            t = tongues[k.tin]
            while True:
                if category == "junk":
                    bad = t.bad_junk(rng)
                elif category == "repeat":
                    bad = t.bad_repeat(rng)
                else:
                    toks = t.say(12 + rng.below(980)).split(" ")
                    j = rng.below(len(toks))
                    toks[j] = words.almost(toks[j])
                    bad = " ".join(toks)
                args = [bad] + [arg(k.tin) for _ in range(k.arity - 1)]
                if k.arity == 2 and rng.below(2):
                    args.reverse()
                if add(k.word, put(frames_of(k.arity)[rng.below(4)], k.word, *args), k.tout):
                    break
    # the words of a taught question in an order that no lesson used
    for i in range(6):
        k = rng.shuffled(live)[0]
        frames = SCRAMBLED_UNARY if k.arity == 1 else SCRAMBLED_BINARY
        while not add(k.word, put(frames[rng.below(4)], k.word, *[arg(k.tin) for _ in range(k.arity)]), k.tout):
            pass
    return rows


def build(seed, compose_first=False):
    """The whole exam as (lessons, stream, key): lists of rows of strings."""
    words, tongues = world_tongues(seed)
    control, newop, compose, unexplainable = make_kinds(seed, tongues)
    order = control + (compose + newop if compose_first else newop + compose) + unexplainable
    for k in order:
        k.word = words.kind_word()
    rng = seeded(seed, 3)
    # the ways of asking: two or three taught for each kind (all eight for the two that carry the equivalences); 16 kinds are asked in one that was not taught for them
    eligible = [k for k in order if k.role != "unexplainable" and not k.bridge]
    reworded = {id(k) for k in rng.shuffled(eligible)[:16]}
    lessons, items, key, used, fact = [], [], [], set(), 0
    for k in order:
        frames = frames_of(k.arity)
        taught_frames = list(range(len(frames))) if k.bridge else sorted(rng.shuffled(range(len(frames)))[: 2 + rng.below(2)])
        untaught = rng.shuffled([i for i in range(len(frames)) if i not in taught_frames])[0] if id(k) in reworded else None
        notes = f"; numerals: {', '.join(f'{x} {tongues[t].label}' for x, t in (('in', k.tin), ('out', k.tout)))}; taught in {len(taught_frames)} ways of asking" + ("" if untaught is None else ", the held-out facts are asked in another")
        if "t1" in (k.tin, k.tout):
            notes += "; first family: the kinds that read and write it tie the words to digits"
        if "t2" in (k.tin, k.tout):
            notes += "; second family: no kind reads or writes it, the meaning comes only through the arithmetic of the kinds computed over it"
        if "t3" in (k.tin, k.tout):
            notes += "; third family: nothing in the lessons fixes the meaning of the words, so rows that need it are expected to be refused"
        key.append([k.word, k.role, k.name, k.rule + notes, k.learn])
        taught, held, beyond = lesson_plan(k, rng, tongues)

        def texts(p, frame_ids):
            args = p + (0,) * (2 - len(p))
            nums = [numeral(tongues, k.tin, x, rng) for x in p]
            truth = tongues[k.tout].say(k.fn(*args))
            return [(put(frames[f], k.word, *nums), truth) for f in frame_ids]

        for p in taught:
            for q, truth in texts(p, taught_frames):
                lessons.append([q, truth, k.word, str(fact)])
                used.add(q)
            fact += 1
        rows = [(p, untaught if untaught is not None else taught_frames[rng.below(len(taught_frames))]) for p in held] + [(p, taught_frames[0]) for p in beyond]
        for p, f in rows:
            (q, truth), = texts(p, [f])
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
    p = argparse.ArgumentParser(description="Write a seventh hidden-worlds adaptive exam (lessons.tsv, stream.tsv) for `baby exam adapt-drive`.")
    p.add_argument("dir")
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--compose-first", action="store_true", help="teach the compositions before the kinds they use (a control: reuse should then not help)")
    a = p.parse_args(argv)
    nl, ns = write(a.dir, a.seed, a.compose_first)
    print(f"wrote the hidden worlds G (seed {a.seed}) to {a.dir}: {nl} lessons and a stream of {ns} questions")


if __name__ == "__main__":
    main(sys.argv[1:])
