# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""Tests for tools/hidden_worlds_k.py: it is deterministic, writes the format `baby exam adapt-write` writes, the key never reaches the exam files, every answer
follows from its rule, every numeral reads back to its number (checked against a second reading of each grammar), a small decoder recovers the meaning of the words of the
first family from the lessons that tie them to digits and of the second from the arithmetic lessons alone, the ways of asking are what the key says (and the lessons
show which words of a way can be left out), every trap row is what its category says, no table, name, way of asking or invented word is one of the earlier sets', and
(with numpy) no hard kind is reachable by a program of at most 7 nodes while the controls are.

    python3 -m unittest discover -s tools -p 'test_hidden_worlds_k.py'

The reachability test (seeds 1 to 6 by default, `HW_K_SEARCH_SEEDS=N` for 1 to N; the time is mostly the search itself, which is the one of the set I test) is skipped without
numpy or with `HW_K_SKIP_SEARCH=1`.
"""
import itertools
import math
import os
import re
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import hidden_worlds as first
import hidden_worlds_b as second
import hidden_worlds_c as third
import hidden_worlds_d as fourth
import hidden_worlds_e as fifth
import hidden_worlds_f as sixth
import hidden_worlds_g as seventh
import hidden_worlds_h as eighth
import hidden_worlds_i as ninth
import hidden_worlds_j as tenth
import hidden_worlds_k as kk
from test_hidden_worlds_i import check_points, search

try:
    import numpy as np
except ImportError:  # the reachability search is the only part that needs it
    np = None

EARLIER = (first, second, third, fourth, fifth, sixth, seventh, eighth, ninth, tenth)
KEY_COLUMNS = ((first, 4), (second, 4), (third, 4), (fourth, 5), (fifth, 5), (sixth, 5), (seventh, 5), (eighth, 5), (ninth, 5), (tenth, 5))
ROLES = {"newop", "compose", "control", "unexplainable", "trap"}
FAMILIES = ("d", "t1", "t2", "t3", "t4", "m1")
MAX_WORDS = 22  # the longest numeral of any grammar has 9 words, a run made for a trap up to 12
NUMERIC = 30  # the kinds that are functions of the numbers they stand for (all but the two that copy or rearrange words)
TRAP_COUNTS = {"two kinds": 8, "new word": 8, "almost a kind word": 4, "arity": 8, "other form": 12, "one number in another form": 4, "bare": 6, "junk": 6, "repeat": 6,
               "almost": 6, "trailing numeral": 6, "unread numeral": 4, "order": 6, "new start": 6, "new end": 6, "foreign kind word": 4, "other core": 6, "word inside": 6,
               "markers exchanged": 6, "mismatched ends": 4, "left out": 4}


def read_bytes(path):
    with open(path, "rb") as f:
        return f.read()


def read_rows(path, columns):
    with open(path, encoding="utf-8") as f:
        text = f.read()
    assert text.endswith("\n")
    rows = [line.split("\t") for line in text.split("\n")[:-1]]
    assert all(len(r) == columns for r in rows), (path, columns)
    return rows


def table(k):
    """A kind's whole table over its own domain, as a tuple of values."""
    return tuple(k.fn(*p) for p in itertools.product(range(k.lo, k.hi + 1), repeat=k.arity))


def earlier_table(k, top=32):
    """An earlier kind's table over the numbers below `top` (the first sets' functions take two arguments whatever their arity)."""
    fn = getattr(k.fn, "__wrapped__", k.fn)  # (a function that remembers its values)
    padded = fn.__code__.co_argcount - len(fn.__defaults__ or ()) == 2 and k.arity == 1
    call = (lambda p: k.fn(*p, 0)) if padded else (lambda p: k.fn(*p))
    return tuple(call(p) for p in itertools.product(range(k.lo, min(top, getattr(k, "hi", 31) + 1)), repeat=k.arity) if not (k.arity == 2 and k.ordered and p[0] < p[1]))


def template_matches(toks, word, question, oks):
    """Every way the question can be the template (a list of tokens) with this word and, for the n-th number, a numeral for which `oks[n]` holds (a number that the template
    names twice has the same numeral both times), each as the list of numerals."""
    q = question.split(" ")

    def walk(i, j, args):
        if i == len(toks):
            if j == len(q):
                yield [args[s] for s in sorted(args)]
            return
        t = toks[i]
        if t == "{w}":
            if j < len(q) and q[j] == word:
                yield from walk(i + 1, j + 1, args)
        elif t in kk.SLOTS:
            s = kk.SLOTS.index(t)
            if s in args:
                n = len(args[s].split(" "))
                if q[j : j + n] == args[s].split(" "):
                    yield from walk(i + 1, j + n, args)
            else:
                for n in range(1, MAX_WORDS + 1):
                    text = " ".join(q[j : j + n])
                    if j + n <= len(q) and oks[s](text):
                        yield from walk(i + 1, j + n, {**args, s: text})
        elif j < len(q) and q[j] == t:
            yield from walk(i + 1, j + 1, args)

    return list(walk(0, 0, {}))


def every_template(arity):
    """{text: (kind, tokens, frame ids)}: the ways of asking, then every one with a filler left out, then every mix of the start of one and the end of another."""
    frames = kk.FRAMES[arity]
    out = {f.text: ("frame", f.toks, (i,)) for i, f in enumerate(frames)}
    for i, f in enumerate(frames):
        for d in f.drop:
            toks = f.toks[:d] + f.toks[d + 1 :]
            if " ".join(toks) not in out:
                out[" ".join(toks)] = ("drop", toks, (i,))
    for i, x in enumerate(frames):
        for j, y in enumerate(frames):
            toks = kk.hybrid_tokens(x, y) if i != j else None
            if toks and " ".join(toks) not in out:
                out[" ".join(toks)] = ("hybrid", toks, (i, j))
    return out


TEMPLATES = {ar: every_template(ar) for ar in (1, 2, 3)}


def fits(question, word, arity, tongues):
    """[(kind of way, ids, numerals)] for each way of asking (and numerals the tongues read, one for each number, or one for all) that gives the question."""
    tongues = list(tongues) if isinstance(tongues, (list, tuple)) else [tongues] * arity
    out = []
    for text, (kind, toks, ids) in TEMPLATES[arity].items():
        for args in template_matches(toks, word, question, [t.valid for t in tongues]):
            out.append((kind, ids, args))
    return out


def order_matters(k):
    """Whether exchanging two neighboring numbers changes the answer somewhere."""
    sample = [p[: k.arity] for p in itertools.product(range(1, 7), repeat=3)]
    swaps = [(0, 1)] + ([(1, 2)] if k.arity == 3 else [])
    return any(k.fn(*p) != k.fn(*[p[t] if i == s else p[s] if i == t else p[i] for i in range(k.arity)]) for p in sample for s, t in swaps)


def of_kind(tongues, k):
    """The tongue of each number of a kind."""
    return [tongues[t] for t in k.tins]


# The readings of the six grammars, written from their descriptions and not through `say`. A reading takes the words of a numeral and the value of each word and gives the number
# (or None where the words are no numeral); a decoder knows the kind of grammar a family might be and which of its two variants, but not the values of the words.


def reading_quot(flip):
    def f(toks, a):
        if len(toks) != 3:
            return None
        q, q2, r = [a[t] for t in (toks[::-1] if flip else toks)]
        if q == 0:
            return 0 if q2 == 0 and r == 0 else None
        n = q * q2 + r
        return n if r < q and math.isqrt(n) == q and n // q == q2 and n <= 999 else None

    return f


def reading_morton(flip):
    def f(toks, a):
        if len(toks) != 2:
            return None
        x, y = [a[t] for t in (toks[::-1] if flip else toks)]
        n = sum((x >> i & 1) << (2 * i) | (y >> i & 1) << (2 * i + 1) for i in range(5))
        return n if n <= 999 else None

    return f


def reading_hilbert(flip):
    def f(toks, a):
        if len(toks) != 2:
            return None
        x, y = [a[t] for t in (toks[::-1] if flip else toks)]
        n = kk.hilbert_step(x, y)
        return n if n <= 999 else None

    return f


def gray_value(g):
    k = len(g)
    if k == 0:
        return 0
    top, rest = g[0], gray_value(g[1:])
    return top * 4 ** (k - 1) + (rest if top % 2 == 0 else 4 ** (k - 1) - 1 - rest)


def reading_gray(low_first):
    def f(toks, a):
        g = [a[t] for t in (toks[::-1] if low_first else toks)]
        if len(g) > 5 or (len(g) > 1 and g[0] == 0):
            return None
        n = gray_value(g)
        return n if n <= 999 else None

    return f


def reading_skew(low_first):
    def f(toks, a):
        d = [a[t] for t in (toks[::-1] if low_first else toks)]
        size = len(d)
        if size > 9 or (size > 1 and d[0] == 0):
            return None
        nonzero = [i for i, x in enumerate(d) if x]
        if 2 in d and (d.count(2) > 1 or nonzero[-1] != d.index(2)):
            return None
        n = sum(x * (2 ** (size - i) - 1) for i, x in enumerate(d))
        return n if n <= 999 else None

    return f


def reading_diffs(low_first):
    def f(toks, a):
        v = [a[t] for t in toks]
        if len(v) > 4:
            return None
        ds = []
        for i, x in enumerate(v):
            ds.append(x if i == 0 else (ds[-1] + x) % 7)
        top = ds[::-1] if low_first else ds
        if len(top) > 1 and top[0] == 0:
            return None
        n = 0
        for x in top:
            n = n * 7 + x
        return n if n <= 999 else None

    return f


# (the variants to try, the values a word may have)
DECODERS = {
    kk.Quot: (lambda: [(v, reading_quot(v)) for v in (False, True)], range(0, 33)),
    kk.Morton: (lambda: [(v, reading_morton(v)) for v in (False, True)], range(0, 32)),
    kk.Hilbert: (lambda: [(v, reading_hilbert(v)) for v in (False, True)], range(0, 32)),
    kk.Gray: (lambda: [(v, reading_gray(v)) for v in (False, True)], range(0, 4)),
    kk.Skew: (lambda: [(v, reading_skew(v)) for v in (False, True)], range(0, 3)),
    kk.Diffs: (lambda: [(v, reading_diffs(v)) for v in (False, True)], range(0, 7)),
}
READINGS = {kk.Quot: reading_quot, kk.Morton: reading_morton, kk.Hilbert: reading_hilbert, kk.Gray: reading_gray, kk.Skew: reading_skew, kk.Diffs: reading_diffs}


def variant_of(tongue):
    return tongue.flip if hasattr(tongue, "flip") else tongue.low_first


def true_values(tongue):
    return {w: i for i, w in enumerate(tongue.digit)}


def solve(reading, domain, constraints, limit=2):
    """The assignments (different words, different values) under which every constraint (numerals, check on their values) holds, at most `limit` of them: the search takes the
    constraint with the fewest words that have no value yet and tries the values for those."""
    cons = [(numerals, check, frozenset(w for toks in numerals for w in toks)) for numerals, check in constraints]
    words = sorted(set().union(*(c[2] for c in cons)))
    out = []

    def holds(c, a):
        vals = [reading(toks, a) for toks in c[0]]
        return None not in vals and c[1](*vals)

    def rec(a, used):
        if len(out) >= limit:
            return
        pending = [c for c in cons if not c[2] <= a.keys()]
        if not pending:
            free = [w for w in words if w not in a]
            for vs in itertools.permutations([v for v in domain if v not in used], len(free)):
                out.append({**a, **dict(zip(free, vs))})
                if len(out) >= limit:
                    return
            return
        c = min(pending, key=lambda c: len(c[2] - a.keys()))
        free = sorted(c[2] - a.keys())
        for vs in itertools.permutations([v for v in domain if v not in used], len(free)):
            b = {**a, **dict(zip(free, vs))}
            if all(holds(d, b) for d in cons if d[2] <= b.keys() and not d[2] <= a.keys()):
                rec(b, used | set(vs))

    rec({}, set())
    return out


def runs_of_words(question, unit):
    """The runs of words of a question that are no word of the ways of asking, no kind word and no digits: one numeral each (the ways of asking never put two numbers side by side)."""
    runs, cur = [], []
    for t in question.split(" "):
        if t in kk.FRAME_WORDS or t == unit or t.isdigit():
            if cur:
                runs.append(cur)
            cur = []
        else:
            cur.append(t)
    return runs + ([cur] if cur else [])


class HiddenWorldsK(unittest.TestCase):
    def write(self, module, seed, **kw):
        d = tempfile.mkdtemp()
        self.addCleanup(lambda: __import__("shutil").rmtree(d, ignore_errors=True))
        module.write(d, seed, **kw)
        return d

    def kinds(self, seed):
        _, tongues = kk.world_tongues(seed)
        return {k.name: k for group in kk.make_kinds(seed, tongues) for k in group}

    def exam(self, seed):
        d = self.write(kk, seed)
        _, tongues = kk.world_tongues(seed)
        key = read_rows(os.path.join(d, "key.tsv"), 5)
        return d, tongues, {r[0]: r[2] for r in key}, self.kinds(seed)

    def test_same_seed_same_files_other_seed_other_files(self):
        a, b, c = self.write(kk, 1), self.write(kk, 1), self.write(kk, 2)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv", "README.txt"):
            self.assertEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(b, name)), name)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv"):
            self.assertNotEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(c, name)), name)

    def test_the_files_parse_in_the_existing_format(self):
        for seed in (1, 2, 3):
            d = self.write(kk, seed)
            lessons = read_rows(os.path.join(d, "lessons.tsv"), 5)
            stream = read_rows(os.path.join(d, "stream.tsv"), 5)
            self.assertEqual([r[0] for r in lessons], [f"L{i:05d}" for i in range(len(lessons))])
            self.assertEqual([r[0] for r in stream], [f"S{i:05d}" for i in range(len(stream))])
            for r in lessons + stream:
                for field in r:
                    self.assertTrue(field and field == field.strip() and "\t" not in field and "  " not in field, r)
            self.assertEqual(len({r[3] for r in lessons}), 32)
            self.assertEqual({r[1] for r in stream}, ROLES)
            # every fact is taught in two or three ways of asking, or (for the three kinds that carry the equivalences) in every way and every way with its filler left out
            facts = {}
            for r in lessons:
                facts.setdefault(r[4], []).append(r)
            self.assertEqual({len(rows) for rows in facts.values()}, {2, 3, 10, 13})
            for rows in facts.values():
                self.assertEqual(len({(x[2], x[3]) for x in rows}), 1)
                self.assertEqual(len(rows), len({x[1] for x in rows}))
            # the stream asks nothing that was taught and nothing twice
            taught = {r[1] for r in lessons}
            asked = [r[3] for r in stream]
            self.assertEqual(len(set(asked)), len(asked))
            self.assertFalse(taught & set(asked))
            self.assertEqual(len(taught), len(lessons))
            for r in stream:
                if r[1] != "trap" and r[2] not in r[3].split(" "):
                    self.fail(r)

    def test_the_key_never_reaches_the_exam_files(self):
        for seed in (1, 2, 3):
            d = self.write(kk, seed)
            key = read_rows(os.path.join(d, "key.tsv"), 5)
            exam = "".join(read_bytes(os.path.join(d, name)).decode("utf-8") for name in ("lessons.tsv", "stream.tsv", "README.txt"))
            for word, role, name, rule, learn in key:
                self.assertTrue(rule and learn)
                self.assertNotIn(name, exam, name)
                self.assertNotIn(rule, exam, name)
                self.assertNotIn(learn, exam, name)
            # what the exam files hold is the fixed words of the questions, the kinds' words, the numerals, the numbers, and (in trap rows only) new words, words one letter
            # away from a kind word or a number word, and the words of a fourth family that no lesson uses
            _, tongues = kk.world_tongues(seed)
            numeral_words = {w for t in ("t1", "t2", "t3") for w in tongues[t].words}
            fourth_words = set(tongues["t4"].words)
            kind_words = {r[0] for r in key}
            allowed = set(kk.FRAME_WORDS) | kind_words | numeral_words
            lessons = read_bytes(os.path.join(d, "lessons.tsv")).decode("utf-8")
            stream = read_bytes(os.path.join(d, "stream.tsv")).decode("utf-8")
            strip = lambda body: set(re.findall(r"[a-z]+", re.sub(r"\bL\d{5}\b|\bS\d{5}\b", "", body))) - ROLES  # noqa: E731
            self.assertFalse(strip(lessons) - allowed)
            fresh = strip(stream) - allowed - fourth_words
            near = lambda w, pool: any(len(w) == len(x) and sum(a != b for a, b in zip(w, x)) == 1 for x in pool)  # noqa: E731
            near_numeral = {w for w in fresh if near(w, numeral_words)}
            near_kind = {w for w in fresh - near_numeral if near(w, kind_words)}
            new = fresh - near_numeral - near_kind
            # the eight new words and the label of the bare numbers, and the four words one letter away from a kind word (a new word may by chance be one letter away from one)
            self.assertEqual(len(new | near_kind), 13, new | near_kind)
            self.assertTrue(all(w.endswith(kk.ENDING) for w in new | near_kind))
            self.assertGreaterEqual(len(near_kind), 4)
            self.assertEqual(len(near_numeral), 6)
            self.assertTrue(strip(stream) & fourth_words)
            self.assertFalse(strip(lessons) & fourth_words)
            self.assertEqual(sorted(os.listdir(d)), ["README.txt", "key.tsv", "lessons.tsv", "stream.tsv"])

    def test_the_kinds_by_role(self):
        for seed in (1, 2, 3):
            d = self.write(kk, seed)
            counts = {}
            for r in read_rows(os.path.join(d, "key.tsv"), 5):
                counts[r[1]] = counts.get(r[1], 0) + 1
            self.assertEqual(counts, {"control": 15, "newop": 10, "compose": 2, "unexplainable": 5})
            per_role = {}
            for r in read_rows(os.path.join(d, "stream.tsv"), 5):
                per_role[r[1]] = per_role.get(r[1], 0) + 1
            self.assertEqual(set(per_role), ROLES)
            self.assertEqual(per_role["trap"], sum(TRAP_COUNTS.values()))
            # the families of numerals: how many kinds take or give words of each family, how many take a form other than digits or give one, how many mix forms
            kinds = self.kinds(seed).values()
            self.assertEqual({fam: sum(1 for k in kinds if fam in k.tins + (k.tout,)) for fam in ("t1", "t2", "t3", "t4", "m1")}, {"t1": 9, "t2": 6, "t3": 4, "t4": 0, "m1": 1})
            self.assertEqual(sum(1 for k in kinds if any(t != "d" for t in k.tins) or k.tout != "d"), 19)
            self.assertEqual(sorted(k.name for k in kinds if len(set(k.tins)) > 1), sorted(["mixback", "blendpair", "varquot"]))  # one number as digits and another as words
            self.assertEqual(sorted(k.name for k in kinds if k.arity == 3), ["capmod", "modmod", "peakmod", "sumquot"])
            self.assertEqual(sorted(k.name for k in kinds if k.arity == 1), sorted(["clipmix", "divways", "rootfork", "gridwalk", "walkshift", "offmod", "numin1", "numout1", "shiftf12", "thirdf2", "echo3", "oddplaces3", "nearquot", "noisef1", "noisef2", "noisef3"]))
            self.assertEqual(sorted(k.name for k in kinds if k.bridge), ["offmod", "peakmod", "subwrap"])
            # a question in one form and the answer in another (digits to words, words to digits, a family to another family)
            self.assertEqual(sorted(k.name for k in kinds if k.tout in ("t1", "t2", "t3") and any(t != k.tout for t in k.tins)), sorted(["gridwalk", "mixback", "shiftf12", "walkshift", "numout1"]))
            self.assertEqual(sorted(k.name for k in kinds if k.tout == "d" and any(t != "d" for t in k.tins)), sorted(["divways", "blendpair", "numin1", "rootfork", "noisef2", "shiftprod", "varquot"]))
            # closed forms with one, two and three numbers, and one, two and three constants
            self.assertTrue(all(len(re.findall(r"\d+", self.kinds(seed)[n].rule.split(",")[0])) >= 1 for n in ("clipmix", "shiftprod", "varquot", "capmod", "modmod")))

    def test_the_words_are_this_sets_own(self):
        mine, invented = set(), set()
        for seed in range(1, 7):
            d = self.write(kk, seed)
            words = {r[0] for r in read_rows(os.path.join(d, "key.tsv"), 5)}
            self.assertEqual(len(words), 32)
            self.assertTrue(all(w.endswith(kk.ENDING) for w in words))
            mine |= words
            _, tongues = kk.world_tongues(seed)
            invented |= words | {w for t in ("t1", "t2", "t3", "t4") for w in tongues[t].words}
            stream = read_bytes(os.path.join(d, "stream.tsv")).decode("utf-8")
            invented |= set(re.findall(r"[a-z]+", re.sub(r"\bS\d{5}\b", "", stream))) - kk.FRAME_WORDS - ROLES
        for module, columns in KEY_COLUMNS:
            for other in range(1, 7):
                d = self.write(module, other)
                theirs = {r[0] for r in read_rows(os.path.join(d, "key.tsv"), columns)}
                self.assertFalse(mine & theirs, (module.__name__, other))
                # no invented word of this set (kind words, number words, the words that only trap rows carry) is any word at all of an earlier exam
                text = "".join(read_bytes(os.path.join(d, name)).decode("utf-8") for name in ("lessons.tsv", "stream.tsv"))
                self.assertFalse(invented & set(re.findall(r"[a-z]+", text)), (module.__name__, other))

    def test_no_way_of_asking_is_one_of_the_earlier_sets(self):
        earlier = {t for ar in (1, 2) for t in first.phrasings("{w}", ar, "{a}", "{b}")}
        earlier |= set(sixth.UNARY) | set(sixth.BINARY) | set(seventh.UNARY) | set(seventh.BINARY)
        earlier |= {f.text for fs in eighth.FRAMES.values() for f in fs} | {f.text for fs in ninth.FRAMES.values() for f in fs} | {f.text for fs in tenth.FRAMES.values() for f in fs}
        for ar, frames in kk.FRAMES.items():
            self.assertEqual(len(frames), 8 if ar < 3 else 6)
            self.assertEqual(len({f.text for f in frames}), len(frames))
            self.assertFalse({f.text for f in frames} & earlier)
            self.assertFalse({" ".join(f.toks[: f.w + 1]) for f in frames if f.w} & {" ".join(t.split(" ")[: t.split(" ").index("{w}") + 1]) for t in earlier if "{w}" in t.split(" ")})
            # at most one filler in a way, and every dropped way is no way of asking
            self.assertTrue(all(len(f.drop) <= 1 for f in frames))
            self.assertTrue(any(f.drop for f in frames))
            self.assertFalse({" ".join(t) for t in kk.dropped(ar, range(len(frames)))} & {f.text for f in frames})
        # a word of a way occurs more than once in some of them, a number occurs twice in some, and some ways have a filler that is a necessary word of another way
        self.assertTrue(any(len(f.toks) > len(set(f.toks)) for fs in kk.FRAMES.values() for f in fs))
        self.assertTrue(any(len(f.slots) < sum(1 for t in f.toks if t in kk.SLOTS) for fs in kk.FRAMES.values() for f in fs))
        fillers = {f.toks[d] for fs in kk.FRAMES.values() for f in fs for d in f.drop}
        necessary = {t for fs in kk.FRAMES.values() for f in fs for i, t in enumerate(f.toks) if i not in f.drop}
        self.assertTrue(fillers & necessary)
        # no word of a way is a number word or a kind word
        self.assertFalse({w for w in kk.FRAME_WORDS if w.endswith(kk.ENDING)})

    def test_no_kind_is_one_of_the_earlier_sets(self):
        # by name, and by table: none of this set's kinds equals any earlier kind (the controls and the arbitrary codes included) on its whole domain or on the numbers below 32;
        # the kinds that read and write words are the identity on a domain of their own
        earlier = [k for module in EARLIER for seed in range(1, 6) for group in module.make_kinds(seed) for k in group]
        names = {k.name for k in earlier}
        tables = {(k.arity, k.lo, 31, earlier_table(k)) for k in earlier}
        mine = [k for seed in range(1, 6) for group in kk.make_kinds(seed) for k in group]
        shapes = {(k.arity, k.lo, k.hi) for k in mine}
        whole = {(k.arity, k.lo, k.hi, earlier_table(k, k.hi + 1)) for k in earlier if hasattr(k, "hi") and (k.arity, k.lo, k.hi) in shapes}  # only the earlier kinds of the same domain can be equal
        for seed in range(1, 6):
            for group in kk.make_kinds(seed):
                for k in group:
                    self.assertNotIn(k.name, names)
                    self.assertFalse((k.arity, k.lo, k.hi, table(k)) in whole, (seed, k.name))
                    if k.hi >= 31 and k.name not in ("numin1", "numout1"):  # on the first 32 numbers too
                        head = tuple(k.fn(*p) for p in itertools.product(range(k.lo, 32), repeat=k.arity))
                        self.assertFalse((k.arity, k.lo, 31, head) in tables, (seed, k.name))

    def test_answers_agree_with_the_rules(self):
        for seed in (1, 2, 3, 4, 5, 6):
            d, tongues, key, kinds = self.exam(seed)
            for name in ("lessons.tsv", "stream.tsv"):
                for r in read_rows(os.path.join(d, name), 5):
                    if name == "lessons.tsv":
                        unit, q, truth = r[3], r[1], r[2]
                    elif r[1] == "trap":
                        continue
                    else:
                        unit, q, truth = r[2], r[3], r[4]
                    k = kinds[key[unit]]
                    found = fits(q, unit, k.arity, of_kind(tongues, k))
                    self.assertEqual(len(found), 1, (q, found))
                    args = [t.parse(x) for t, x in zip(of_kind(tongues, k), found[0][2])]
                    self.assertTrue(all(k.lo <= n <= k.hi for n in args), q)
                    self.assertEqual(str(tongues[k.tout].say(k.fn(*args))), truth, q)

    def test_the_ways_of_asking(self):
        # each kind is taught in two or three ways (every way and every way with its filler left out for the three bridges); the held-out facts of 24 kinds are asked six each in a
        # taught way, in a way that was not taught for the kind, in a mix of the start of one taught way and the end of another, or in a taught way with a filler left out; beyond rows in the first taught way
        for seed in (1, 2, 3):
            d, tongues, key, kinds = self.exam(seed)
            taught, dropped_ids = {}, {}
            for r in read_rows(os.path.join(d, "lessons.tsv"), 5):
                k = kinds[key[r[3]]]
                (kind, ids, _), = fits(r[1], r[3], k.arity, of_kind(tongues, k))
                self.assertIn(kind, ("frame", "drop") if k.bridge else ("frame",))
                taught.setdefault(r[3], set()).add(ids[0])
                if kind == "drop":
                    dropped_ids.setdefault(r[3], set()).add(ids[0])
            self.assertEqual(len(taught), 32)
            for unit, ids in taught.items():
                k = kinds[key[unit]]
                self.assertIn(len(ids), ((len(kk.FRAMES[k.arity]),) if k.bridge else (2, 3)), k.name)
                # the bridge shows every filler left out, and only a filler
                if k.bridge:
                    self.assertEqual(dropped_ids[unit], {i for i, f in enumerate(kk.FRAMES[k.arity]) if f.drop}, k.name)
                else:
                    self.assertNotIn(unit, dropped_ids)
            used_modes = {}
            for r in read_rows(os.path.join(d, "stream.tsv"), 5):
                if r[1] == "trap":
                    continue
                k = kinds[key[r[2]]]
                (kind, ids, _), = fits(r[3], r[2], k.arity, of_kind(tongues, k))
                if kind == "frame" and ids[0] not in taught[r[2]]:
                    kind = "untaught"
                elif kind == "hybrid":
                    self.assertTrue(set(ids) <= taught[r[2]], r)
                elif kind == "drop":
                    self.assertTrue(ids[0] in taught[r[2]], r)
                    # the evidence: a kind that is taught with and without this filler, for the arity of the kind
                    bridges = [u for u, n in key.items() if kinds[n].bridge and kinds[n].arity == k.arity]
                    self.assertEqual(len(bridges), 1)
                    self.assertIn(ids[0], dropped_ids[bridges[0]], r)
                used_modes.setdefault(r[2], set()).add(kind)
            by_mode = {}
            for unit, modes in used_modes.items():
                k = kinds[key[unit]]
                if k.role == "unexplainable" or k.bridge:
                    self.assertTrue(modes <= ({"frame", "drop"} if k.bridge else {"frame"}), k.name)  # a bridge is taught in the ways with a filler left out as well
                    continue
                self.assertLessEqual(len(modes), 2, (k.name, modes))  # the beyond rows are in a taught way, the held-out rows in the kind's mode
                mode = (modes - {"frame"}) or {"frame"}
                by_mode.setdefault(next(iter(mode)), []).append(k.name)
            self.assertEqual({m: len(v) for m, v in by_mode.items()}, {"frame": 6, "untaught": 6, "hybrid": 6, "drop": 6})
            # a way not taught for a kind is one taught for the bridges; the three roles of kinds with lessons are among the reworded
            roles = {kinds[n].role for names in by_mode.values() for n in names}
            self.assertEqual(roles, {"newop", "compose", "control"})
            for m in ("frame", "untaught", "hybrid", "drop"):
                self.assertEqual(sum(1 for n in by_mode[m] if kinds[n].role == "control"), 3, m)  # three controls and three hard kinds in each

    def test_a_filler_is_shown_dropped_with_the_same_facts_and_nothing_else_is(self):
        # in the lessons of a bridge the same fact is asked in a way with a filler and in the same way without it; no lesson leaves out a word that is not a filler
        for seed in (1, 2):
            d, tongues, key, kinds = self.exam(seed)
            facts = {}
            for r in read_rows(os.path.join(d, "lessons.tsv"), 5):
                k = kinds[key[r[3]]]
                if k.bridge:
                    (kind, ids, _), = fits(r[1], r[3], k.arity, of_kind(tongues, k))
                    facts.setdefault(r[4], []).append((kind, ids[0], k.arity))
            for pairs in facts.values():
                arity = pairs[0][2]
                with_filler = {i for kind, i, _ in pairs if kind == "frame" and kk.FRAMES[arity][i].drop}
                without = {i for kind, i, _ in pairs if kind == "drop"}
                self.assertEqual(with_filler, without)
                self.assertEqual(len(pairs), len(kk.FRAMES[arity]) + len(without))
            # the stream: the held-out rows that leave a filler out are those of six kinds, the trap rows that leave a necessary word out are four
            held = 0
            for r in read_rows(os.path.join(d, "stream.tsv"), 5):
                if r[1] == "trap":
                    continue
                k = kinds[key[r[2]]]
                (kind, _, _), = fits(r[3], r[2], k.arity, of_kind(tongues, k))
                held += kind == "drop" and not k.bridge  # (the bridges are asked in the ways they were taught in, some of them with a filler left out)
            self.assertGreaterEqual(held, 6 * 5)
            self.assertLessEqual(held, 6 * 10)

    def test_forms_mix_inside_a_kind(self):
        for seed in (1, 2, 3):
            d, tongues, key, kinds = self.exam(seed)
            forms_of = lambda x: "digits" if x.isdigit() else "words"  # noqa: E731
            for name in ("lessons.tsv", "stream.tsv"):
                lesson = name == "lessons.tsv"
                for kind in ("rootfork", "blendpair", "mixback", "varquot"):
                    unit = [u for u, n in key.items() if n == kind][0]
                    k = kinds[kind]
                    rows = [(r[1] if lesson else r[3]) for r in read_rows(os.path.join(d, name), 5) if (r[3] if lesson else r[2]) == unit and r[1] != "trap"]
                    forms = [set() for _ in range(k.arity)]
                    for q in rows:
                        (_, _, args), = fits(q, unit, k.arity, of_kind(tongues, k))
                        for i, x in enumerate(args):
                            forms[i].add(forms_of(x))
                    if kind == "rootfork":  # the form is drawn for each number
                        self.assertEqual(forms[0], {"digits", "words"}, (name, kind))
                    else:  # one number always in digits and the other always in words
                        self.assertEqual([sorted(f) for f in forms], [["digits"], ["words"]] if kind == "blendpair" else [["words"], ["digits"]], (name, kind))
            # the kinds that read and write words keep their forms apart: words in, digits out; digits in, words out; a question in one family is answered in another; digits in, words out
            for r in read_rows(os.path.join(d, "lessons.tsv"), 5):
                if key[r[3]] == "numin1":
                    self.assertTrue(r[2].isdigit() and not any(x.isdigit() for x in r[1].split(" ")), r)
                if key[r[3]] == "numout1":
                    self.assertFalse(r[2].isdigit(), r)
                if key[r[3]] == "shiftf12":
                    self.assertFalse(r[2].isdigit() or set(r[2].split(" ")) & set(tongues["t1"].words), r)
                    self.assertTrue(set(r[2].split(" ")) <= set(tongues["t2"].words), r)
                if key[r[3]] in ("gridwalk", "walkshift"):
                    self.assertTrue(set(r[2].split(" ")) <= set(tongues["t1"].words) and any(x.isdigit() for x in r[1].split(" ")), r)
                if key[r[3]] == "mixback":
                    self.assertTrue(set(r[2].split(" ")) <= set(tongues["t1"].words), r)

    def test_the_numerals(self):
        grammars = {}
        for seed in range(1, 13):
            words, tongues = kk.world_tongues(seed)
            names = [type(tongues[t]).__name__ for t in ("t1", "t2", "t3", "t4")]
            self.assertEqual(len(set(names)), 4, seed)
            for n in names[:3]:
                grammars.setdefault(n, []).append(seed)
            seen_words = set()
            for t in ("t1", "t2", "t3", "t4"):
                tongue = tongues[t]
                self.assertEqual(len(set(tongue.words)), len(tongue.words))
                self.assertFalse(seen_words & set(tongue.words))
                seen_words |= set(tongue.words)
                self.assertFalse(set(tongue.words) & set(kk.FRAME_WORDS))
                self.assertTrue(all(re.fullmatch(r"w[aeiou][bdghklmnprstvz][aeiou][bdghklmnprstvz]", w) for w in tongue.words))
                self.assertGreaterEqual(tongue.cap, 990)
                said = [tongue.say(n) for n in range(tongue.cap + 1)]
                self.assertEqual(len(set(said)), tongue.cap + 1, (seed, t))
                for n, text in enumerate(said):
                    self.assertEqual(tongue.parse(text), n, (seed, t, n))
                    self.assertLessEqual(len(text.split(" ")), MAX_WORDS - 13)
                    self.assertTrue(set(text.split(" ")) <= set(tongue.words))
            # the grammars are written with words of their own
            self.assertTrue(all(set(tongues[t].foreign) == seen_words - set(tongues[t].words) for t in ("t1", "t2", "t3", "t4")))
            self.assertTrue(tongues["m1"].valid("17") and tongues["m1"].valid(tongues["t1"].say(17)) and not tongues["m1"].valid(tongues["t2"].say(17)))
        self.assertEqual(sorted(grammars), ["Diffs", "Gray", "Hilbert", "Morton", "Quot", "Skew"])
        self.assertTrue(all(len(v) >= 2 for v in grammars.values()), grammars)
        # seeds 1 to 3: the grammars of the three families
        self.assertEqual([[type(kk.world_tongues(s)[1][t]).__name__ for t in ("t1", "t2", "t3")] for s in (1, 2, 3)],
                         [["Morton", "Quot", "Gray"], ["Quot", "Gray", "Hilbert"], ["Gray", "Hilbert", "Skew"]])
        # over the seeds 1 to 6 each grammar is the first family once and the second family once, and a family of repeating words is among the three of each world
        for role in ("t1", "t2"):
            self.assertEqual(sorted(type(kk.world_tongues(s)[1][role]).__name__ for s in range(1, 7)), sorted(["Diffs", "Gray", "Hilbert", "Morton", "Quot", "Skew"]))
        for s in range(1, 7):
            self.assertTrue({type(kk.world_tongues(s)[1][t]).__name__ for t in ("t1", "t2", "t3")} & {"Gray", "Skew", "Diffs"}, s)
        # in every grammar some numerals have a word that occurs twice
        for seed in range(1, 7):
            tongues = kk.world_tongues(seed)[1]
            for t in ("t1", "t2", "t3"):
                self.assertGreaterEqual(sum(1 for n in range(tongues[t].cap + 1) if len(set(tongues[t].say(n).split(" "))) < len(tongues[t].say(n).split(" "))), 25, (seed, t))

    def test_each_grammar_does_what_it_says(self):
        """A second reading of each grammar, from its words and the rule in the docstring, against `say` for every number."""
        for seed in range(1, 13):
            _, tongues = kk.world_tongues(seed)
            for t in ("t1", "t2", "t3", "t4"):
                tongue = tongues[t]
                reading = READINGS[type(tongue)](variant_of(tongue))
                values = true_values(tongue)
                for n in range(tongue.cap + 1):
                    self.assertEqual(reading(tongue.say(n).split(" "), values), n, (seed, t, type(tongue).__name__, n))
                d = tongue.digit
                ordered = lambda toks: toks[::-1] if variant_of(tongue) else toks  # noqa: E731  (the variants that write the words the other way round)
                if isinstance(tongue, kk.Quot):
                    self.assertEqual(tongue.say(0), " ".join([d[0]] * 3))
                    for n, t3 in ((1, [1, 1, 0]), (2, [1, 2, 0]), (3, [1, 3, 0]), (4, [2, 2, 0]), (8, [2, 4, 0]), (999, [31, 32, 7]), (960, [30, 32, 0]), (35, [5, 7, 0]), (34, [5, 6, 4])):
                        self.assertEqual(tongue.say(n), " ".join(ordered([d[x] for x in t3])), n)
                elif isinstance(tongue, kk.Morton):
                    self.assertEqual(tongue.say(0), " ".join([d[0]] * 2))
                    for n, xy in ((1, [1, 0]), (2, [0, 1]), (3, [1, 1]), (4, [2, 0]), (999, [27, 29]), (341, [31, 0]), (682, [0, 31])):
                        self.assertEqual(tongue.say(n), " ".join(ordered([d[x] for x in xy])), n)
                elif isinstance(tongue, kk.Hilbert):
                    cells = [kk.hilbert_cell(n) for n in range(1024)]
                    self.assertEqual(cells[:4], [(0, 0), (0, 1), (1, 1), (1, 0)])
                    self.assertEqual(len(set(cells)), 1024)
                    self.assertTrue(all(abs(a[0] - b[0]) + abs(a[1] - b[1]) == 1 for a, b in zip(cells, cells[1:])))
                    self.assertTrue(all(kk.hilbert_step(*c) == n for n, c in enumerate(cells)))
                    self.assertEqual(tongue.say(0), " ".join([d[0]] * 2))
                    self.assertEqual(tongue.say(1), " ".join(ordered([d[0], d[1]])))
                elif isinstance(tongue, kk.Gray):
                    words = [tongue.say(n).split(" ") for n in range(1024)]
                    ds = [[d.index(x) for x in (w[::-1] if tongue.low_first else w)] for w in words]
                    ds = [[0] * (5 - len(x)) + x for x in ds]
                    self.assertTrue(all(sum(a != b for a, b in zip(p, q)) == 1 for p, q in zip(ds, ds[1:])))  # neighbors differ in one word
                    self.assertTrue(all(abs(a - b) == 1 for p, q in zip(ds, ds[1:]) for a, b in zip(p, q) if a != b))
                    self.assertEqual(tongue.say(0), d[0])
                    for n, g in ((3, [3]), (4, [1, 3]), (5, [1, 2]), (7, [1, 0]), (8, [2, 0]), (15, [3, 0])):
                        self.assertEqual(tongue.say(n), " ".join(ordered([d[x] for x in g])), n)
                elif isinstance(tongue, kk.Skew):
                    self.assertEqual(tongue.say(0), d[0])
                    for n, g in ((1, [1]), (2, [2]), (3, [1, 0]), (4, [1, 1]), (5, [1, 2]), (6, [2, 0]), (7, [1, 0, 0]), (13, [1, 2, 0]), (999, [1, 1, 1, 1, 1, 0, 1, 1, 2])):
                        self.assertEqual(tongue.say(n), " ".join(ordered([d[x] for x in g])), n)
                else:
                    self.assertEqual(tongue.say(0), d[0])
                    if not tongue.low_first:  # 999 is 2625 in base seven: the first digit, then the step up to each next one (6 - 2 = 4, 2 - 6 = -4 = 3, 5 - 2 = 3)
                        cases = ((1, [1]), (6, [6]), (7, [1, 6]), (8, [1, 0]), (48, [6, 0]), (49, [1, 6, 0]), (999, [2, 4, 3, 3]))
                    else:  # the lowest digit as it is, then the step from each digit up to the next one (999: 5, then 2 - 5 = 4, 6 - 2 = 4, 2 - 6 = 3)
                        cases = ((1, [1]), (6, [6]), (7, [0, 1]), (8, [1, 0]), (48, [6, 0]), (49, [0, 0, 1]), (999, [5, 4, 4, 3]))
                    for n, g in cases:
                        self.assertEqual(tongue.say(n), " ".join(d[x] for x in g), n)

    def test_what_is_no_numeral_is_none(self):
        for seed in range(1, 13):
            words, tongues = kk.world_tongues(seed)
            rng = kk.seeded(seed, 11)
            everywhere = [tongues[t] for t in FAMILIES]
            for t in ("t1", "t2", "t3", "t4"):
                tongue = tongues[t]
                for make in (tongue.bad_junk, tongue.bad_repeat):
                    for _ in range(40):
                        bad = make(rng)
                        self.assertFalse(any(x.valid(bad) for x in everywhere), (seed, t, bad))
                for _ in range(10):
                    bad = tongue.spliced(rng)
                    self.assertFalse(any(x.valid(bad) for x in everywhere), (seed, t, bad))
                # a word one letter away from a number word: no word of any family
                word = tongue.words[rng.below(len(tongue.words))]
                near = words.almost(word, number=True)
                self.assertFalse(near in tongue.words or any(x.valid(near) for x in everywhere))
                self.assertTrue(near[0] == kk.NUMBER_FIRST)
                self.assertEqual(sum(a != b for a, b in zip(word, near)), 1)
            self.assertTrue(tongues["d"].valid("17") and not tongues["d"].valid("seventeen"))

    def lesson_constraints(self, d, key, names):
        """The constraints on the values of the words that the lessons of the named kinds make (one for each fact; the facts are shown in several ways of asking, once each here)."""
        cons, seen = [], set()
        for r in read_rows(os.path.join(d, "lessons.tsv"), 5):
            name = key[r[3]]
            if name not in names or r[4] in seen:
                continue
            seen.add(r[4])
            if name == "numin1":  # the numeral in the question, the number in digits in the answer
                cons.append(([runs_of_words(r[1], r[3])[0]], lambda v, n=int(r[2]): v == n))
            elif name == "numout1":  # the number in digits in the question, the numeral in the answer
                cons.append(([r[2].split(" ")], lambda v, n=int(re.findall(r"\d+", r[1])[0]): v == n))
            elif name == "skewsumf2":
                a, b = runs_of_words(r[1], r[3])
                cons.append(([a, b, r[2].split(" ")], lambda x, y, z: x + 3 * y == z))
            elif name == "thirdf2":
                cons.append(([runs_of_words(r[1], r[3])[0], r[2].split(" ")], lambda x, y: x // 3 == y))
            else:
                raise AssertionError(name)
        return cons

    def decoded(self, seed, fam, names):
        d, tongues, key, kinds = self.exam(seed)
        tongue = tongues[fam]
        variants, domain = DECODERS[type(tongue)]
        cons = self.lesson_constraints(d, {u: n for u, n in key.items()}, names)
        found = []
        for variant, reading in variants():
            for a in solve(reading, domain, cons):
                found.append((variant, reading, a))
        return tongue, cons, found

    def check_decoded(self, seed, fam, names):
        tongue, cons, found = self.decoded(seed, fam, names)
        self.assertGreaterEqual(len(cons), 100, (seed, fam))
        self.assertEqual(len(found), 1, (seed, fam, "the lessons leave", len(found), "meanings"))  # one variant and one assignment
        variant, reading, assign = found[0]
        self.assertEqual(set(assign), set(tongue.words))
        self.assertEqual(variant, variant_of(tongue))
        # the one meaning is the true one and gives every number its numeral back (all numbers up to the largest the grammar writes, so also every numeral of a held-out or beyond row)
        self.assertEqual(assign, true_values(tongue))
        for n in range(tongue.cap + 1):
            self.assertEqual(reading(tongue.say(n).split(" "), assign), n, (seed, fam, n))
        return variant

    def test_the_first_family_is_decoded_from_the_lessons_that_tie_it_to_digits(self):
        # the kind that reads the words and the kind that writes them, taught on about a hundred numbers each: a decoder that knows the kind of grammar and nothing else
        for seed in range(1, 13):
            self.check_decoded(seed, "t1", {"numin1", "numout1"})

    def test_the_second_family_is_decoded_from_the_arithmetic_lessons_alone(self):
        # the sum with a tripled second number and the third of a number, no lesson in digits: the one meaning that fits is the true one
        for seed in range(1, 13):
            self.check_decoded(seed, "t2", {"skewsumf2", "thirdf2"})

    def test_the_decoders_need_the_lessons(self):
        # for two of the grammars (seeds 3 and 6) the lessons of the kind that takes a third alone leave more than one meaning for the second family, so that the sum is needed
        # as well, and a decoder with the wrong grammar finds none
        for seed in (3, 6):
            tongue, cons, found = self.decoded(seed, "t2", {"thirdf2"})
            self.assertGreater(len(found), 1, seed)
        for seed in (1, 3, 5):
            d, tongues, key, kinds = self.exam(seed)
            wrong = next(c for c in DECODERS if c is not type(tongues["t1"]) and c is not type(tongues["t2"]))
            variants, domain = DECODERS[wrong]
            cons = self.lesson_constraints(d, key, {"numin1", "numout1"})
            self.assertEqual([a for v, rd in variants() for a in solve(rd, domain, cons)], [], seed)

    def test_the_third_family_has_no_lesson_that_ties_its_words_to_numbers(self):
        # the kinds over the third family copy or rearrange words (their answers follow from the words of the question alone), are arbitrary codes, or are a hard kind whose answer
        # is a number that a learner can only relate to the words by knowing its rule
        for seed in (1, 2, 3, 4, 5, 6):
            d, tongues, key, kinds = self.exam(seed)
            over3 = sorted(k.name for k in kinds.values() if "t3" in k.tins + (k.tout,))
            self.assertEqual(over3, sorted(["divways", "echo3", "oddplaces3", "noisef3"]))
            for r in read_rows(os.path.join(d, "lessons.tsv"), 5):
                name = key[r[3]]
                toks = runs_of_words(r[1], r[3])
                if name == "echo3":
                    self.assertEqual(r[2].split(" "), toks[0] + toks[0])
                elif name == "oddplaces3":
                    self.assertEqual(r[2].split(" "), toks[0][::2])
                elif name == "noisef3":
                    self.assertEqual(len({tuple(t) for t in toks}), 1)  # (a way of asking may name the number twice)
                    self.assertFalse(set(r[2].split(" ")) - set(tongues["t3"].words))
                elif name == "divways":
                    self.assertTrue(r[2].isdigit())
            # the codes answer with numerals of the family that are no function of the words of the question (no word of the answer is a fixed place of the question)
            codes = [(runs_of_words(r[1], r[3])[0], r[2].split(" ")) for r in read_rows(os.path.join(d, "lessons.tsv"), 5) if key[r[3]] == "noisef3"]
            self.assertFalse(all(a[-1] == b[-1] or a[0] == b[0] for a, b in codes))

    def test_the_words_of_held_out_numerals_occur_in_lessons(self):
        # the kinds that read and write the first family, and those that add and take thirds in the second, show every word of it, and every
        # word of every held-out and beyond row of the first two families is among them
        for seed in range(1, 9):
            lessons, stream, key = kk.build(seed)
            _, tongues = kk.world_tongues(seed)
            kind = {r[0]: r[2] for r in key}
            for fam, names in (("t1", {"numin1", "numout1"}), ("t2", {"skewsumf2", "thirdf2"})):
                for name in names:
                    seen = set()
                    for r in lessons:
                        if kind[r[3]] == name:
                            seen |= set(r[1].split(" ")) | set(r[2].split(" "))
                    self.assertEqual(set(tongues[fam].words) - seen, set(), (seed, fam, name))
                fam_words = set(tongues[fam].words)
                everything = set().union(*(set(r[1].split(" ")) | set(r[2].split(" ")) for r in lessons if kind[r[3]] in names))
                for r in stream:
                    if r[1] != "trap" and kind[r[2]] not in ("noisef1", "noisef2"):
                        self.assertFalse((set(r[3].split(" ")) | set(r[4].split(" "))) & fam_words - everything, r)

    def test_repeats_are_in_the_stream(self):
        for seed in (1, 2, 3, 4):
            d, tongues, key, kinds = self.exam(seed)
            stream = [r for r in read_rows(os.path.join(d, "stream.tsv"), 5) if r[1] != "trap"]
            for unit, name in key.items():
                k = kinds[name]
                rows = [r for r in stream if r[2] == unit]
                args = [[t.parse(x) for t, x in zip(of_kind(tongues, k), fits(r[3], unit, k.arity, of_kind(tongues, k))[0][2])] for r in rows]
                if k.arity >= 2:  # a number named twice in a question
                    self.assertGreaterEqual(sum(1 for a in args if len(set(a)) < len(a)), 3 if k.arity == 2 else 2, name)
                tongue = tongues[k.tin]
                if k.arity == 1 and k.tin not in ("d", "m1") and k.hi > 99:  # a numeral with a word that occurs twice
                    self.assertGreaterEqual(sum(1 for a in args if len(set(tongue.say(a[0]).split(" "))) < len(tongue.say(a[0]).split(" "))), 4, name)
            # a number that the way of asking names twice, in the lessons and in the stream
            twice = 0
            for name in ("lessons.tsv", "stream.tsv"):
                for r in read_rows(os.path.join(d, name), 5):
                    unit, q = (r[3], r[1]) if name == "lessons.tsv" else (r[2], r[3])
                    if r[1] != "trap" and unit in key and kinds[key[unit]].arity == 1:
                        (_, ids, _), = fits(q, unit, 1, of_kind(tongues, kinds[key[unit]]))
                        twice += len(kk.FRAMES[1][ids[0]].slots) < sum(1 for t in kk.FRAMES[1][ids[0]].toks if t in kk.SLOTS)
            self.assertGreater(twice, 100)

    def test_the_operations_against_known_values(self):
        self.assertEqual([kk.ordered_factorizations(n) for n in range(1, 13)], [1, 1, 1, 2, 1, 3, 1, 4, 2, 3, 1, 8])
        self.assertEqual([kk.is_square(n) for n in range(0, 17)], [n in (0, 1, 4, 9, 16) for n in range(0, 17)])
        self.assertEqual([kk.digit_walk(a) for a in (0, 5, 12, 123, 1000, 99)], [0, 5, 1, 4, 1, 18])
        self.assertEqual([kk.text_rank(a, 20) for a in (1, 2, 10, 19, 20)], [1, 12, 2, 11, 13])
        self.assertEqual(kk.text_rank(5, 3), 3)  # a above b: every number to b comes before it
        self.assertEqual([kk.under_line(a, b) for a, b in ((1, 5), (2, 2), (3, 3), (4, 4), (3, 6), (5, 3))], [0, 0, 1, 3, 4, 4])
        self.assertEqual([kk.interleave(x, y) for x, y in ((0, 0), (1, 0), (0, 1), (3, 3), (31, 0), (0, 31), (31, 31))], [0, 1, 2, 15, 341, 682, 1023])
        self.assertEqual([kk.gray_digits(n, 2) for n in (0, 3, 4, 7, 8, 15)], [[0, 0], [0, 3], [1, 3], [1, 0], [2, 0], [3, 0]])

    def test_a_dozen_small_cases_of_each_hard_kind(self):
        for seed in (1, 2):
            kinds = self.kinds(seed)
            c1, c2, c3 = (int(x) for x in re.fullmatch(r"\(a mod (\d+)\) \* (\d+) \+ a div (\d+)", kinds["clipmix"].rule).groups())
            self.assertEqual([kinds["clipmix"].fn(a) for a in (0, 1, c1, c1 + 1, 999)], [0, c2, c1 // c3, c2 + (c1 + 1) // c3, (999 % c1) * c2 + 999 // c3])
            s1, s2, s3 = (int(x) for x in re.match(r"\(a \+ (\d+)\) \* \(b \+ (\d+)\) mod (\d+)", kinds["shiftprod"].rule).groups())
            self.assertEqual([kinds["shiftprod"].fn(a, b) for a, b in ((0, 0), (5, 7), (99, 99))], [s1 * s2 % s3, (5 + s1) * (7 + s2) % s3, (99 + s1) * (99 + s2) % s3])
            v1, v2 = (int(x) for x in re.match(r"\(a \* (\d+) \+ b\) div \(b mod (\d+) \+ 1\)", kinds["varquot"].rule).groups())
            self.assertEqual([kinds["varquot"].fn(a, b) for a, b in ((0, 0), (5, v2 - 1), (9, v2))], [0, (5 * v1 + v2 - 1) // v2, (9 * v1 + v2) // 1])
            p1, p2, p3 = (int(x) for x in re.match(r"min\(a \* (\d+) \+ b, c \* (\d+) mod (\d+)\)", kinds["capmod"].rule).groups())
            self.assertEqual([kinds["capmod"].fn(*p) for p in ((0, 0, 0), (1, 2, 3), (30, 30, 30))], [0, min(p1 + 2, 3 * p2 % p3), min(30 * p1 + 30, 30 * p2 % p3)])
            m1 = int(re.search(r"mod \(a \+ (\d+)\)", kinds["modmod"].rule).group(1))
            self.assertEqual([kinds["modmod"].fn(*p) for p in ((0, 5, 7), (3, 4, 5), (30, 30, 30))], [7 % m1, 17 % (3 + m1), 930 % (30 + m1)])
            q1, q2, q3, q4 = (int(x) for x in re.match(r"(\d+) \* \(the integer square root of a\) \+ (\d+) if a is a perfect square, else \(a mod (\d+)\) \+ a div (\d+)", kinds["rootfork"].rule).groups())
            self.assertEqual([kinds["rootfork"].fn(a) for a in (0, 1, 2, 16, 17, 900)], [q2, q1 + q2, 2 % q3 + 2 // q4, 4 * q1 + q2, 17 % q3 + 17 // q4, 30 * q1 + q2])
            self.assertEqual([kinds["divways"].fn(a) for a in (1, 6, 12)], [1, 3, 8])
            self.assertEqual([kinds["textrank"].fn(a, b) for a, b in ((1, 20), (2, 20), (99, 99))], [1, 12, 99])
            self.assertEqual([kinds["belowline"].fn(a, b) for a, b in ((3, 3), (4, 4))], [1, 3])
            self.assertEqual([kinds["gridwalk"].fn(a) for a in (5, 12, 123)], [5, 1, 4])

    def test_the_rules_against_second_implementations(self):
        # ordered factorizations: list every ordered product
        def products(n):
            return 1 if n == 1 else sum(products(n // d) for d in range(2, n + 1) if n % d == 0)

        for n in range(1, 400):
            self.assertEqual(kk.ordered_factorizations(n), products(n))
        # the walk: complex numbers, a right turn is a multiplication by -i
        for a in range(0, 3000):
            pos, heading = 0j, 1j
            for ch in str(a):
                heading *= (-1j) ** int(ch)
                pos += int(ch) * heading
            self.assertEqual(kk.digit_walk(a), round(abs(pos.real) + abs(pos.imag)), a)
        # the rank among the numbers in the order of the alphabet: sort the strings
        for b in (1, 9, 10, 11, 20, 99, 100):
            order = sorted(str(x) for x in range(1, b + 1))
            for a in range(1, b + 1):
                self.assertEqual(kk.text_rank(a, b), order.index(str(a)) + 1)
        # the points under a line: Pick's formula for the interior points of a right triangle with the legs a and b
        for a in range(1, 41):
            for b in range(1, 41):
                self.assertEqual(kk.under_line(a, b), ((a - 1) * (b - 1) - (math.gcd(a, b) - 1)) // 2)
        # the squares against a sieve
        squares = {n * n for n in range(40)}
        self.assertEqual([kk.is_square(n) for n in range(1000)], [n in squares for n in range(1000)])
        # the interleaving of bits against a string
        for x in range(32):
            for y in range(32):
                s = "".join(a + b for a, b in zip(format(y, "05b"), format(x, "05b")))
                self.assertEqual(kk.interleave(x, y), int(s, 2))

    def test_every_kind_has_enough_values_and_the_constants_are_in_range(self):
        for seed in range(1, 31):
            kinds = self.kinds(seed)
            _, tongues = kk.world_tongues(seed)
            for k in kinds.values():
                self.assertGreaterEqual(len(set(table(k))), 3 if k.name in ("echo3", "oddplaces3") else 5, (seed, k.name))
                self.assertLessEqual(k.hi, 999)
                outputs = set(table(k))
                if k.tout in ("t1", "t2", "t3"):
                    self.assertLessEqual(max(outputs), tongues[k.tout].cap, (seed, k.name))
                    self.assertGreaterEqual(min(outputs), 0)
                for t in k.tins:
                    if t in ("t1", "t2", "t3", "m1"):
                        self.assertLessEqual(k.hi, tongues[t].cap, (seed, k.name))
                if k.tout == "d":
                    self.assertTrue(all(isinstance(x, int) and x >= 0 for x in outputs), (seed, k.name))
            c1, c2, c3 = (int(x) for x in re.fullmatch(r"\(a mod (\d+)\) \* (\d+) \+ a div (\d+)", kinds["clipmix"].rule).groups())
            self.assertTrue(7 <= c1 <= 31 and 3 <= c2 <= 12 and c3 in kk.MIDDLE)
            s1, s2, s3 = (int(x) for x in re.match(r"\(a \+ (\d+)\) \* \(b \+ (\d+)\) mod (\d+)", kinds["shiftprod"].rule).groups())
            self.assertTrue(3 <= s1 <= 60 and 3 <= s2 <= 60 and s3 in kk.LARGE)
            v1, v2 = (int(x) for x in re.match(r"\(a \* (\d+) \+ b\) div \(b mod (\d+) \+ 1\)", kinds["varquot"].rule).groups())
            self.assertTrue(3 <= v1 <= 30 and 4 <= v2 <= 12)
            p1, p2, p3 = (int(x) for x in re.match(r"min\(a \* (\d+) \+ b, c \* (\d+) mod (\d+)\)", kinds["capmod"].rule).groups())
            self.assertTrue(2 <= p1 <= 9 and 2 <= p2 <= 9 and p3 in kk.MIDDLE)
            self.assertTrue(1 <= int(re.search(r"mod \(a \+ (\d+)\)", kinds["modmod"].rule).group(1)) <= 30)
            # the constants of the controls are among those the reachability search offers
            offered = set(range(0, 41)) | {50, 64, 100}
            for name in ("offmod", "subwrap", "peakmod", "ratshift", "sumquot", "blendpair", "mixback", "shiftf12", "bumpf1", "skewsumf2", "thirdf2"):
                self.assertTrue({int(x) for x in re.findall(r"\d+", kinds[name].rule.split(",")[0])} <= offered, (seed, name))
            for name in ("offmod", "subwrap", "peakmod"):
                m = int(re.findall(r"mod (\d+)", kinds[name].rule)[-1])
                self.assertIn(m, kk.SMALL_MODS)
        for p in kk.MIDDLE + kk.LARGE:
            self.assertTrue(all(p % q for q in range(2, int(math.sqrt(p)) + 1)))
        # the first branch of the fork on squares is taken by the squares and only by them
        kinds = self.kinds(1)
        for a in range(0, 200):
            self.assertEqual(kk.is_square(a), a in {0, 1, 4, 9, 16, 25, 36, 49, 64, 81, 100, 121, 144, 169, 196})
        self.assertEqual({kk.is_square(a) for a in range(100)}, {True, False})
        self.assertEqual(kinds["rootfork"].hi, 999)

    def test_the_arbitrary_codes_are_not_a_pattern(self):
        for seed in (1, 2, 3):
            kinds = self.kinds(seed)
            _, tongues = kk.world_tongues(seed)
            cells = [kinds["tagpair"].fn(a, b) for a in range(16) for b in range(16)]
            self.assertGreaterEqual(len(set(cells)), 10)
            self.assertLess(max(cells), 20)
            rows = {tuple(kinds["tagpair"].fn(a, b) for b in range(16)) for a in range(16)}
            self.assertEqual(len(rows), 16)
            self.assertLess(max(kinds["noisef1"].fn(a) for a in range(32)), min(300, tongues["t1"].cap + 1))
            self.assertLess(max(kinds["noisef2"].fn(a) for a in range(32)), 55)
            self.assertLess(max(kinds["noisef3"].fn(a) for a in range(32)), min(200, tongues["t3"].cap + 1))
            # the quotient that is not one: a div q for most numbers, something else for at least eight of them
            q = int(re.match(r"a div (\d+) for most", kinds["nearquot"].rule).group(1))
            self.assertIn(q, (3, 4, 5, 6))
            codes = [kinds["nearquot"].fn(a) for a in range(32)]
            self.assertGreaterEqual(sum(1 for a in range(32) if codes[a] != a // q), 8)
            self.assertGreaterEqual(sum(1 for a in range(32) if codes[a] == a // q), 14)
            self.assertTrue(all(kinds[n].role == "unexplainable" for n in ("tagpair", "nearquot", "noisef1", "noisef2", "noisef3")))

    def test_the_trap_rows(self):
        self.assertEqual(sum(TRAP_COUNTS.values()), 126)
        for seed in (1, 2, 3):
            lessons, stream, key, categories = kk.build_full(seed)
            _, tongues = kk.world_tongues(seed)
            words = {r[0]: r[2] for r in key}
            kinds = self.kinds(seed)
            taught_questions = {r[1] for r in lessons}
            traps = [r for r in stream if r[1] == "trap"]
            self.assertEqual(len(traps), 126)
            self.assertEqual(len({r[3] for r in traps}), 126)
            self.assertFalse({r[3] for r in traps} & taught_questions)
            self.assertGreaterEqual(len({r[4] for r in traps}), 45)  # the truths are drawn one by one
            counts = {}
            for c in categories.values():
                counts[c] = counts.get(c, 0) + 1
            self.assertEqual(counts, TRAP_COUNTS)
            everywhere = [tongues[t] for t in FAMILIES]
            valid_anywhere = lambda x: any(t.valid(x) for t in everywhere)  # noqa: E731
            left_out = 0
            word_of = lambda q: [w for w in q.split(" ") if w in words]  # noqa: E731
            for r in traps:
                unit, q, cat = r[2], r[3], categories[r[3]]
                toks = q.split(" ")
                if unit in words:
                    k = kinds[words[unit]]
                    # a trap row is never an ordinary question, a mix or a dropped filler of the kind's taught ways
                    self.assertFalse(fits(q, unit, k.arity, of_kind(tongues, k)), q)
                if cat == "two kinds":
                    self.assertGreaterEqual(len(word_of(q)), 2, q)
                elif cat == "new word":
                    self.assertNotIn(unit, words)
                    self.assertTrue(unit in toks and unit.endswith(kk.ENDING), q)
                elif cat == "almost a kind word":
                    self.assertNotIn(unit, words)
                    near = [w for w in words if len(w) == len(unit) and sum(a != b for a, b in zip(unit, w)) == 1]
                    self.assertEqual(len(near), 1, q)
                    self.assertTrue(unit in toks and unit.endswith(kk.ENDING), q)
                elif cat == "arity":
                    k = kinds[words[unit]]
                    other = [ar for ar in (1, 2, 3) if ar != k.arity and fits(q, unit, ar, tongues[k.tin])]
                    self.assertEqual(len(other), 1, q)
                elif cat == "other form":
                    k = kinds[words[unit]]
                    other = tongues["t3"] if "t3" not in k.tins else tongues["d"]
                    self.assertTrue(fits(q, unit, k.arity, other), q)
                    self.assertFalse(fits(q, unit, k.arity, of_kind(tongues, k)), q)
                elif cat == "one number in another form":
                    k = kinds[words[unit]]
                    others = [[tongues["t3"] if i == j else tongues[t] for i, t in enumerate(k.tins)] for j in range(k.arity)]
                    self.assertEqual(sum(1 for o in others if fits(q, unit, k.arity, o)), 1, q)
                elif cat == "bare":
                    self.assertNotIn(unit, words)
                    self.assertIn(unit, [r[2] for r in traps if categories[r[3]] == "bare"])
                    self.assertTrue(valid_anywhere(re.sub(r"^(only|regarding) ", "", q)), q)
                elif cat in ("junk", "repeat", "almost"):
                    k = kinds[words[unit]]
                    self.assertFalse(fits(q, unit, k.arity, of_kind(tongues, k)), q)
                    anything = lambda x: bool(x) and not set(x.split(" ")) & kk.FRAME_WORDS  # noqa: E731  (a run of words that holds no word of the ways of asking)
                    self.assertTrue(any(template_matches(f.toks, unit, q, [anything] * k.arity) for f in kk.FRAMES[k.arity]), q)
                elif cat == "trailing numeral":
                    k = kinds[words[unit]]
                    t4 = tongues["t4"]
                    cuts = [i for i in range(1, len(toks)) if t4.valid(" ".join(toks[i:])) and fits(" ".join(toks[:i]), unit, k.arity, of_kind(tongues, k))]
                    self.assertTrue(cuts, q)  # an ordinary question and then a numeral of the fourth family
                elif cat == "unread numeral":
                    k = kinds[words[unit]]
                    t4 = tongues["t4"]
                    others = [[t4 if i == j else tongues[t] for i, t in enumerate(k.tins)] for j in range(k.arity)]
                    self.assertTrue(any(fits(q, unit, k.arity, o) for o in others), q)  # an ordinary question with one numeral of the family that no kind reads
                elif cat == "order":
                    k = kinds[words[unit]]
                    self.assertIn(unit, toks)
                    self.assertFalse(any(toks[0] == f.toks[0] for f in kk.FRAMES[k.arity] if f.toks[0] not in kk.SLOTS), q)
                elif cat == "new start":
                    start = [s for s in kk.NEW_STARTS if q.startswith(" ".join(s) + " ")]
                    self.assertEqual(len(start), 1, q)
                    self.assertFalse({s[0] for s in kk.NEW_STARTS} & {f.toks[0] for fs in kk.FRAMES.values() for f in fs})  # no kind has a way that starts so
                elif cat == "new end":
                    k = kinds[words[unit]]
                    ends = [e for e in kk.NEW_ENDS if q.endswith(" " + " ".join(e))]
                    self.assertEqual(len(ends), 1, q)
                    body = q[: -len(" " + " ".join(ends[0]))]
                    self.assertTrue(fits(body, unit, k.arity, of_kind(tongues, k)), q)  # the rest is an ordinary question
                    self.assertFalse({e[-1] for e in kk.NEW_ENDS} & {f.toks[-1] for fs in kk.FRAMES.values() for f in fs})  # no kind has a way that ends so
                elif cat == "foreign kind word":
                    self.assertGreaterEqual(len(set(word_of(q))), 2, q)  # a taught word of another kind where a filler is, or after the way
                    k = kinds[words[unit]]
                    other = [w for w in set(word_of(q)) if w != unit]
                    self.assertEqual(len(other), 1, q)
                    plain = " ".join(t for t in toks if t != other[0])
                    self.assertTrue(fits(plain, unit, k.arity, of_kind(tongues, k)), q)  # without that word it is an ordinary question (or one with a filler left out)
                elif cat == "other core":
                    self.assertNotIn(unit, toks)
                    self.assertFalse(word_of(q))
                elif cat == "word inside":
                    where = [i for i, t in enumerate(toks) if t == unit]
                    numeral_words = {w for t in ("t1", "t2", "t3") for w in tongues[t].words}
                    self.assertGreaterEqual(len(where), 2, q)  # the kind's word in its place and once more inside a numeral (twice, where the way of asking names the number twice)
                    self.assertTrue(any(toks[i - 1] in numeral_words and toks[i + 1] in numeral_words for i in where if 0 < i < len(toks) - 1), q)
                elif cat == "markers exchanged":
                    k = kinds[words[unit]]
                    self.assertTrue(any(template_matches(swap, unit, q, [t.valid for t in of_kind(tongues, k)]) for f in kk.FRAMES[k.arity] for swap in self.exchanged(f)), q)
                    self.assertTrue(order_matters(k), q)  # the order of the numbers matters
                elif cat == "mismatched ends":
                    k = kinds[words[unit]]
                    ok = [t.valid for t in of_kind(tongues, k)]
                    self.assertTrue(any(template_matches(x.toks[: x.w + 1] + y.toks[y.w + 1 :], unit, q, (ok * 3)[:3]) for x in kk.FRAMES[k.arity] for y in kk.FRAMES[3 - k.arity]), q)
                elif cat == "left out":
                    left_out += unit not in toks
            self.assertEqual(left_out, 2)  # the kind's word left out in two rows, a word before a number in the other two

    @staticmethod
    def exchanged(frame):
        """The tokens of the frame with the words before two of its numbers exchanged, for each pair of such words."""
        m = frame.markers()
        out = []
        for s, t in [(s, t) for s in m for t in m if s < t and frame.toks[m[s]] != frame.toks[m[t]]]:
            toks = list(frame.toks)
            toks[m[s]], toks[m[t]] = toks[m[t]], toks[m[s]]
            out.append(toks)
        return out

    def test_compose_first_changes_only_the_teaching_order(self):
        a, b = self.write(kk, 3), self.write(kk, 3, compose_first=True)
        la, lb = read_rows(os.path.join(a, "lessons.tsv"), 5), read_rows(os.path.join(b, "lessons.tsv"), 5)
        self.assertNotEqual(la, lb)
        ka, kb = read_rows(os.path.join(a, "key.tsv"), 5), read_rows(os.path.join(b, "key.tsv"), 5)
        roles = [r[1] for r in kb]
        self.assertLess(roles.index("compose"), roles.index("newop"))
        self.assertLess([r[1] for r in ka].index("newop"), [r[1] for r in ka].index("compose"))

    @unittest.skipIf(np is None, "numpy is needed for the reachability search")
    @unittest.skipIf(os.environ.get("HW_K_SKIP_SEARCH"), "HW_K_SKIP_SEARCH is set")
    def test_no_hard_kind_is_reachable_and_the_controls_are(self):
        seeds = range(1, int(os.environ.get("HW_K_SEARCH_SEEDS", "6")) + 1)
        groups, roles = {}, {}
        for seed in seeds:
            for group in kk.make_kinds(seed):
                for k in group:
                    if not k.numeric:
                        continue
                    name, points = check_points(k)
                    groups.setdefault(name, (points, {}))[1][f"{seed}:{k.name}"] = tuple(k.fn(*p) for p in points)
                    roles[f"{seed}:{k.name}"] = k.role
        reached = {}
        for saturating in (False, True):  # subtraction signed, and stopping at 0
            for name, (points, targets) in groups.items():
                for target, nodes in search(points, targets, saturating).items():
                    reached.setdefault(target, []).append(nodes)
        self.assertEqual(len(roles), len(seeds) * NUMERIC)  # every kind but the two that copy or rearrange words
        for target, role in roles.items():
            if role == "control":
                # a control is reachable in both conventions (the one that subtracts too: with the difference that stops at zero, a + m - b is never below zero)
                self.assertEqual(len(reached.get(target, [])), 2, f"{target} should be reachable in both conventions")
                self.assertLessEqual(max(reached[target]), 7)
            else:
                self.assertEqual(reached.get(target, []), [], f"{target} ({role}) is reachable with at most 7 nodes")
        # the search itself finds what it should: a seven-node kind, and nothing for a table that only a longer program makes
        points = [(a,) for a in range(1, 32)]
        found = search(points, {"seven": tuple(a // 2 + a // 3 for (a,) in points), "nine": tuple(a // 2 + a // 3 + a // 5 for (a,) in points)}, False)
        self.assertEqual(found, {"seven": 7})
        triples = [(a, b, c) for a in range(1, 6) for b in range(1, 6) for c in range(1, 6)]
        found = search(triples, {"five": tuple(a * b // c for a, b, c in triples), "median": tuple(sorted(t)[1] for t in triples)}, False)
        self.assertEqual(found, {"five": 5})


if __name__ == "__main__":
    unittest.main()
