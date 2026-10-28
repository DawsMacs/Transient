# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""Tests for tools/hidden_worlds_j.py: it is deterministic, writes the format `baby exam adapt-write` writes, the key never reaches the exam files, every answer
follows from its rule, every numeral reads back to its number (checked against a second reading of each grammar), a small decoder recovers the meaning of the words of the
first family from the lessons that tie them to digits and of the second from the arithmetic lessons alone, the ways of asking are what the key says, every trap row is what
its category says, no table, name, way of asking or invented word is one of the earlier sets', and (with numpy) no hard kind is reachable by a program of at most 7 nodes
while the controls are.

    python3 -m unittest discover -s tools -p 'test_hidden_worlds_j.py'

The reachability test (seeds 1 to 6 by default, `HW_J_SEARCH_SEEDS=N` for 1 to N; the time is mostly the search itself, which is the one of the set I test) is skipped without
numpy or with `HW_J_SKIP_SEARCH=1`.
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
import hidden_worlds_j as jj
from test_hidden_worlds_i import check_points, search

try:
    import numpy as np
except ImportError:  # the reachability search is the only part that needs it
    np = None

EARLIER = (first, second, third, fourth, fifth, sixth, seventh, eighth, ninth)
KEY_COLUMNS = ((first, 4), (second, 4), (third, 4), (fourth, 5), (fifth, 5), (sixth, 5), (seventh, 5), (eighth, 5), (ninth, 5))
ROLES = {"newop", "compose", "control", "unexplainable", "trap"}
FAMILIES = ("d", "t1", "t2", "t3", "t4", "m1")
MAX_WORDS = 22  # the longest numeral of any grammar has 10 words
NUMERIC = 30  # the kinds that are functions of the numbers they stand for (all but the two that copy or rearrange words)


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
    padded = k.fn.__code__.co_argcount - len(k.fn.__defaults__ or ()) == 2 and k.arity == 1
    call = (lambda p: k.fn(*p, 0)) if padded else (lambda p: k.fn(*p))
    return tuple(call(p) for p in itertools.product(range(k.lo, min(top, getattr(k, "hi", 31) + 1)), repeat=k.arity) if not (k.arity == 2 and k.ordered and p[0] < p[1]))


def template_matches(toks, word, question, oks):
    """Every way the question can be the template (a list of tokens) with this word and, for the n-th number, a numeral for which `oks[n]` holds, each as the list of numerals."""
    q = question.split(" ")

    def walk(i, j, args):
        if i == len(toks):
            if j == len(q):
                yield list(args)
            return
        t = toks[i]
        if t == "{w}":
            if j < len(q) and q[j] == word:
                yield from walk(i + 1, j + 1, args)
        elif t in jj.SLOTS:
            ok = oks[jj.SLOTS.index(t)]
            for n in range(1, MAX_WORDS + 1):
                text = " ".join(q[j : j + n])
                if j + n <= len(q) and ok(text):
                    yield from walk(i + 1, j + n, args + [text])
        elif j < len(q) and q[j] == t:
            yield from walk(i + 1, j + 1, args)

    return list(walk(0, 0, []))


def every_template(arity):
    """{text: (kind, tokens, frame ids)}: the ways of asking, then every one with a filler left out, then every mix of the start of one and the end of another."""
    frames = jj.FRAMES[arity]
    out = {f.text: ("frame", f.toks, (i,)) for i, f in enumerate(frames)}
    for i, f in enumerate(frames):
        for d in f.drop:
            toks = f.toks[:d] + f.toks[d + 1 :]
            if " ".join(toks) not in out:
                out[" ".join(toks)] = ("drop", toks, (i,))
    for i, x in enumerate(frames):
        for j, y in enumerate(frames):
            toks = jj.hybrid_tokens(x, y) if i != j else None
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


FIBS = [1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987]


def read_numeral(tongue, toks):
    """The number a numeral stands for, read from its words by the description of its grammar (not through `say`)."""
    if isinstance(tongue, jj.Residue):
        ds = [tongue.digit.index(w) for w in toks]
        assert len(ds) == 3 and all(d < m for d, m in zip(ds, tongue.mods))
        return next(n for n in range(tongue.cap + 1) if all(n % m == d for m, d in zip(tongue.mods, ds)))
    if isinstance(tongue, jj.Runs):
        if toks == [tongue.zero]:
            return 0
        lengths = [{w: i for i, w in tongue.run.items()}[w] for w in toks]
        bits = "".join(("1" if i % 2 == 0 else "0") * x for i, x in enumerate(lengths[::-1] if tongue.low_first else lengths))
        return int(bits, 2)
    if isinstance(tongue, jj.Comb):
        x, y, z = [tongue.digit.index(w) for w in (toks[::-1] if tongue.rising else toks)]
        assert x > y > z
        return math.comb(x, 3) + math.comb(y, 2) + z
    if isinstance(tongue, jj.Countdown):
        k, r = [tongue.digit.index(w) for w in (toks[::-1] if tongue.short_first else toks)]
        assert r < k or (k, r) == (0, 0)
        return k * (k + 1) // 2 - r
    if isinstance(tongue, jj.Zeckendorf):
        if toks == [tongue.zero]:
            return 0
        idx = [tongue.fib.index(w) for w in (toks[::-1] if tongue.rising else toks)]
        assert all(a - b >= 2 for a, b in zip(idx, idx[1:]))
        return sum(FIBS[i] for i in idx)
    ds = [tongue.digit.index(w) for w in (toks if tongue.low_first else toks[::-1])]  # lowest digit first
    assert len(ds) == 1 or ds[-1] != 0
    return sum(d * (-3) ** i for i, d in enumerate(ds))


# The decoders. A decoder knows the kind of grammar a family might be (its readings below, written from the descriptions) but not the words' values, nor which of the
# few variants (the order of the moduli, the direction of the words) the world uses: it looks for the assignments of values to the words of the family under which every
# lesson of the kinds it may use comes out true, and a family is "decoded" when the only assignments there are give every number its numeral back.


def runs_from(lengths):
    bits = "".join(("1" if i % 2 == 0 else "0") * x for i, x in enumerate(lengths))
    return int(bits, 2)


def reading_residue(order):
    def f(toks, a):
        ds = [a[t] for t in toks]
        if len(ds) != 3 or any(d >= m for d, m in zip(ds, order)):
            return None
        return next(n for n in range(math.prod(order)) if all(n % m == d for m, d in zip(order, ds)))

    return f


def reading_runs(low_first):
    def f(toks, a):
        vs = [a[t] for t in toks]
        if vs == [0]:
            return 0
        if not vs or 0 in vs:
            return None
        n = runs_from(vs[::-1] if low_first else vs)
        return n if n <= 999 else None

    return f


def reading_comb(rising):
    def f(toks, a):
        if len(toks) != 3:
            return None
        x, y, z = [a[t] for t in (toks[::-1] if rising else toks)]
        n = math.comb(x, 3) + math.comb(y, 2) + z
        return n if x > y > z and n <= 999 else None

    return f


def reading_countdown(short_first):
    def f(toks, a):
        if len(toks) != 2:
            return None
        k, r = [a[t] for t in (toks[::-1] if short_first else toks)]
        if k == 0:
            return 0 if r == 0 else None
        n = k * (k + 1) // 2 - r
        return n if r < k and n <= 990 else None

    return f


def reading_zeck(rising):
    def f(toks, a):
        vs = [a[t] for t in toks]
        if vs == [0]:
            return 0
        if not vs or 0 in vs:
            return None
        vs = vs[::-1] if rising else vs
        if any(x - y < 2 for x, y in zip(vs, vs[1:])):
            return None
        n = sum(FIBS[v - 1] for v in vs)
        return n if n <= 999 else None

    return f


def reading_neg(low_first):
    def f(toks, a):
        ds = [a[t] for t in (toks if low_first else toks[::-1])]
        if len(ds) > 1 and ds[-1] == 0:
            return None
        n = sum(d * (-3) ** i for i, d in enumerate(ds))
        return n if 0 <= n <= 999 else None

    return f


# (the variants to try, the values a word may have)
DECODERS = {
    jj.Residue: (lambda: [(o, reading_residue(o)) for o in jj.Residue.ORDERS], range(0, 13)),
    jj.Runs: (lambda: [(v, reading_runs(v)) for v in (False, True)], range(0, 10)),
    jj.Comb: (lambda: [(v, reading_comb(v)) for v in (False, True)], range(0, 20)),
    jj.Countdown: (lambda: [(v, reading_countdown(v)) for v in (False, True)], range(0, 45)),
    jj.Zeckendorf: (lambda: [(v, reading_zeck(v)) for v in (False, True)], range(0, 16)),
    jj.Negabase: (lambda: [(v, reading_neg(v)) for v in (False, True)], range(0, 3)),
}


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
        if t in jj.FRAME_WORDS or t == unit or t.isdigit():
            if cur:
                runs.append(cur)
            cur = []
        else:
            cur.append(t)
    return runs + ([cur] if cur else [])


class HiddenWorldsJ(unittest.TestCase):
    def write(self, module, seed, **kw):
        d = tempfile.mkdtemp()
        self.addCleanup(lambda: __import__("shutil").rmtree(d, ignore_errors=True))
        module.write(d, seed, **kw)
        return d

    def kinds(self, seed):
        _, tongues = jj.world_tongues(seed)
        return {k.name: k for group in jj.make_kinds(seed, tongues) for k in group}

    def exam(self, seed):
        d = self.write(jj, seed)
        _, tongues = jj.world_tongues(seed)
        key = read_rows(os.path.join(d, "key.tsv"), 5)
        return d, tongues, {r[0]: r[2] for r in key}, self.kinds(seed)

    def test_same_seed_same_files_other_seed_other_files(self):
        a, b, c = self.write(jj, 1), self.write(jj, 1), self.write(jj, 2)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv", "README.txt"):
            self.assertEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(b, name)), name)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv"):
            self.assertNotEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(c, name)), name)

    def test_the_files_parse_in_the_existing_format(self):
        for seed in (1, 2, 3):
            d = self.write(jj, seed)
            lessons = read_rows(os.path.join(d, "lessons.tsv"), 5)
            stream = read_rows(os.path.join(d, "stream.tsv"), 5)
            self.assertEqual([r[0] for r in lessons], [f"L{i:05d}" for i in range(len(lessons))])
            self.assertEqual([r[0] for r in stream], [f"S{i:05d}" for i in range(len(stream))])
            for r in lessons + stream:
                for field in r:
                    self.assertTrue(field and field == field.strip() and "\t" not in field and "  " not in field, r)
            self.assertEqual(len({r[3] for r in lessons}), 32)
            self.assertEqual({r[1] for r in stream}, ROLES)
            # every fact is taught in two, three or (for the three kinds that carry the equivalences) all six or eight ways of asking, all with the same answer
            facts = {}
            for r in lessons:
                facts.setdefault(r[4], []).append(r)
            self.assertEqual({len(rows) for rows in facts.values()}, {2, 3, 6, 8})
            for rows in facts.values():
                self.assertEqual(len({(x[2], x[3]) for x in rows}), 1)
                self.assertEqual(len({x[1] for x in rows}), len(rows))
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
            d = self.write(jj, seed)
            key = read_rows(os.path.join(d, "key.tsv"), 5)
            exam = "".join(read_bytes(os.path.join(d, name)).decode("utf-8") for name in ("lessons.tsv", "stream.tsv", "README.txt"))
            for word, role, name, rule, learn in key:
                self.assertTrue(rule and learn)
                self.assertNotIn(name, exam, name)
                self.assertNotIn(rule, exam, name)
                self.assertNotIn(learn, exam, name)
            # what the exam files hold is the fixed words of the questions, the kinds' words, the numerals, the numbers, and (in trap rows only) new words, words one letter
            # away from a kind word or a number word, and the words of a fourth family that no lesson uses
            _, tongues = jj.world_tongues(seed)
            numeral_words = {w for t in ("t1", "t2", "t3") for w in tongues[t].words}
            fourth_words = set(tongues["t4"].words)
            kind_words = {r[0] for r in key}
            allowed = set(jj.FRAME_WORDS) | kind_words | numeral_words
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
            self.assertTrue(all(w.endswith(jj.ENDING) for w in new | near_kind))
            self.assertGreaterEqual(len(near_kind), 4)
            self.assertEqual(len(near_numeral), 6)
            self.assertTrue(strip(stream) & fourth_words)
            self.assertFalse(strip(lessons) & fourth_words)
            self.assertEqual(sorted(os.listdir(d)), ["README.txt", "key.tsv", "lessons.tsv", "stream.tsv"])

    def test_the_kinds_by_role(self):
        for seed in (1, 2, 3):
            d = self.write(jj, seed)
            counts = {}
            for r in read_rows(os.path.join(d, "key.tsv"), 5):
                counts[r[1]] = counts.get(r[1], 0) + 1
            self.assertEqual(counts, {"control": 15, "newop": 10, "compose": 2, "unexplainable": 5})
            per_role = {}
            for r in read_rows(os.path.join(d, "stream.tsv"), 5):
                per_role[r[1]] = per_role.get(r[1], 0) + 1
            self.assertEqual(set(per_role), ROLES)
            self.assertEqual(per_role["trap"], 118)
            # the families of numerals: how many kinds take or give words of each family, how many take a form other than digits or give one, how many mix forms
            kinds = self.kinds(seed).values()
            self.assertEqual({fam: sum(1 for k in kinds if fam in k.tins + (k.tout,)) for fam in ("t1", "t2", "t3", "t4", "m1")}, {"t1": 9, "t2": 5, "t3": 4, "t4": 0, "m1": 1})
            self.assertEqual(sum(1 for k in kinds if any(t != "d" for t in k.tins) or k.tout != "d"), 18)
            self.assertEqual(sum(1 for k in kinds if len(set(k.tins)) > 1), 3)  # one number as digits and another as words (rootprod, quarterpair, capwrite)
            self.assertEqual(sorted(k.name for k in kinds if len(set(k.tins)) > 1 and k.arity > 1), ["capwrite", "quarterpair", "rootprod"])
            self.assertEqual(sorted(k.name for k in kinds if k.arity == 3), ["mulacc", "powthird", "progsum", "sheaf"])
            self.assertEqual(sorted(k.name for k in kinds if k.bridge), ["foldpair", "sheaf", "topper"])
            # a question in one form and the answer in another (digits to words, words to digits, a family to another family)
            self.assertEqual(sorted(k.name for k in kinds if k.tout in ("t1", "t2", "t3") and any(t != k.tout for t in k.tins)), ["capwrite", "remtop", "stepf12", "writef1"])

    def test_the_words_are_this_sets_own(self):
        mine, invented = set(), set()
        for seed in range(1, 7):
            d = self.write(jj, seed)
            words = {r[0] for r in read_rows(os.path.join(d, "key.tsv"), 5)}
            self.assertEqual(len(words), 32)
            self.assertTrue(all(w.endswith(jj.ENDING) for w in words))
            mine |= words
            _, tongues = jj.world_tongues(seed)
            invented |= words | {w for t in ("t1", "t2", "t3", "t4") for w in tongues[t].words}
            stream = read_bytes(os.path.join(d, "stream.tsv")).decode("utf-8")
            invented |= set(re.findall(r"[a-z]+", re.sub(r"\bS\d{5}\b", "", stream))) - jj.FRAME_WORDS - ROLES
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
        earlier |= {f.text for fs in eighth.FRAMES.values() for f in fs} | {f.text for fs in ninth.FRAMES.values() for f in fs}
        for ar, frames in jj.FRAMES.items():
            self.assertEqual(len(frames), 8 if ar < 3 else 6)
            self.assertEqual(len({f.text for f in frames}), len(frames))
            self.assertFalse({f.text for f in frames} & earlier)
            self.assertFalse({" ".join(f.toks[:f.w + 1]) for f in frames if f.w} & {" ".join(t.split(" ")[:t.split(" ").index("{w}") + 1]) for t in earlier if "{w}" in t.split(" ")})
        # a word of a way occurs more than once in some of them, and some ways have a number next to a number
        self.assertTrue(any(len(f.toks) > len(set(f.toks)) for fs in jj.FRAMES.values() for f in fs))

    def test_no_kind_is_one_of_the_earlier_sets(self):
        # by name, and by table: none of this set's kinds equals any earlier kind (the controls and the arbitrary codes included) on its whole domain or on the numbers below 32;
        # the kinds that read and write words, add words, halve words and the like are the identity, a sum or a floor of a half on a domain of their own
        earlier = [k for module in EARLIER for seed in range(1, 6) for group in module.make_kinds(seed) for k in group]
        names = {k.name for k in earlier}
        tables = {(k.arity, k.lo, 31, earlier_table(k)) for k in earlier}
        for seed in range(1, 6):
            for group in jj.make_kinds(seed):
                for k in group:
                    self.assertNotIn(k.name, names)
                    self.assertFalse((k.arity, k.lo, k.hi, table(k)) in tables, (seed, k.name))
                    if k.hi >= 31 and k.name not in ("readf1", "writef1", "addf1", "joinf2", "halvef2"):  # on the first 32 numbers too
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
        # each kind is taught in two or three ways (all of them for the three bridges); the held-out facts of 24 kinds are asked six each in a taught way, in a way that was not
        # taught for the kind, in a mix of the start of one taught way and the end of another, or in a taught way with a filler left out; beyond rows in the first taught way
        for seed in (1, 2, 3):
            d, tongues, key, kinds = self.exam(seed)
            taught = {}
            for r in read_rows(os.path.join(d, "lessons.tsv"), 5):
                k = kinds[key[r[3]]]
                (kind, ids, _), = fits(r[1], r[3], k.arity, of_kind(tongues, k))
                self.assertEqual(kind, "frame")
                taught.setdefault(r[3], set()).add(ids[0])
            self.assertEqual(len(taught), 32)
            for unit, ids in taught.items():
                k = kinds[key[unit]]
                self.assertIn(len(ids), ((len(jj.FRAMES[k.arity]),) if k.bridge else (2, 3)), k.name)
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
                used_modes.setdefault(r[2], set()).add(kind)
            by_mode = {}
            for unit, modes in used_modes.items():
                k = kinds[key[unit]]
                if k.role == "unexplainable" or k.bridge:
                    self.assertEqual(modes, {"frame"}, k.name)
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

    def test_forms_mix_inside_a_kind(self):
        for seed in (1, 2, 3):
            d, tongues, key, kinds = self.exam(seed)
            forms_of = lambda x: "digits" if x.isdigit() else "words"  # noqa: E731
            for name in ("lessons.tsv", "stream.tsv"):
                lesson = name == "lessons.tsv"
                for kind in ("primefork", "rootprod", "quarterpair", "capwrite"):
                    unit = [u for u, n in key.items() if n == kind][0]
                    k = kinds[kind]
                    rows = [(r[1] if lesson else r[3]) for r in read_rows(os.path.join(d, name), 5) if (r[3] if lesson else r[2]) == unit and r[1] != "trap"]
                    forms = [set() for _ in range(k.arity)]
                    for q in rows:
                        (_, _, args), = fits(q, unit, k.arity, of_kind(tongues, k))
                        for i, x in enumerate(args):
                            forms[i].add(forms_of(x))
                    if kind == "primefork":  # the form is drawn for each number
                        self.assertEqual(forms[0], {"digits", "words"}, (name, kind))
                    else:  # one number always in digits and the other always in words
                        self.assertEqual([sorted(f) for f in forms], [["digits"], ["words"]] if kind == "quarterpair" else [["words"], ["digits"]], (name, kind))
            # the kinds that read and write words keep their forms apart: words in, digits out; digits in, words out; a question in one family is answered in another
            for r in read_rows(os.path.join(d, "lessons.tsv"), 5):
                if key[r[3]] == "readf1":
                    self.assertTrue(r[2].isdigit() and not any(x.isdigit() for x in r[1].split(" ")), r)
                if key[r[3]] == "writef1":
                    self.assertFalse(r[2].isdigit(), r)
                if key[r[3]] == "stepf12":
                    self.assertFalse(r[2].isdigit() or set(r[2].split(" ")) & set(tongues["t1"].words), r)
                    self.assertTrue(set(r[2].split(" ")) <= set(tongues["t2"].words), r)

    def test_the_numerals(self):
        grammars = {}
        for seed in range(1, 13):
            words, tongues = jj.world_tongues(seed)
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
                self.assertFalse(set(tongue.words) & set(jj.FRAME_WORDS))
                self.assertTrue(all(len(w) == 5 and w[0] == jj.NUMBER_FIRST and w[2] != w[3] for w in tongue.words))
                self.assertGreaterEqual(tongue.cap, 990)
                said = [tongue.say(n) for n in range(tongue.cap + 1)]
                self.assertEqual(len(set(said)), tongue.cap + 1, (seed, t))
                for n, text in enumerate(said):
                    self.assertEqual(tongue.parse(text), n, (seed, t, n))
                    self.assertLessEqual(len(text.split(" ")), MAX_WORDS - 3)
                    self.assertTrue(set(text.split(" ")) <= set(tongue.words))
            # the grammars are written with words of their own
            self.assertTrue(all(set(tongues[t].foreign) == seen_words - set(tongues[t].words) for t in ("t1", "t2", "t3", "t4")))
            self.assertTrue(tongues["m1"].valid("17") and tongues["m1"].valid(tongues["t1"].say(17)) and not tongues["m1"].valid(tongues["t2"].say(17)))
        self.assertEqual(sorted(grammars), ["Comb", "Countdown", "Negabase", "Residue", "Runs", "Zeckendorf"])
        self.assertTrue(all(len(v) >= 2 for v in grammars.values()), grammars)
        # seeds 1 to 3: the grammars of the three families
        self.assertEqual([[type(jj.world_tongues(s)[1][t]).__name__ for t in ("t1", "t2", "t3")] for s in (1, 2, 3)],
                         [["Residue", "Comb", "Runs"], ["Comb", "Runs", "Zeckendorf"], ["Runs", "Zeckendorf", "Negabase"]])
        # over seeds 1 to 6 each grammar is the first family once and the second family once, and a family of repeating words is among the three of each world
        for role in ("t1", "t2"):
            self.assertEqual(sorted(type(jj.world_tongues(s)[1][role]).__name__ for s in range(1, 7)), sorted(["Comb", "Countdown", "Negabase", "Residue", "Runs", "Zeckendorf"]))
        for s in range(1, 7):
            self.assertTrue({type(jj.world_tongues(s)[1][t]).__name__ for t in ("t1", "t2", "t3")} & {"Residue", "Runs", "Negabase"}, s)

    def test_each_grammar_does_what_it_says(self):
        """A second reading of each grammar, from its words and the rule in the docstring, against `say` for every number."""
        for seed in range(1, 13):
            _, tongues = jj.world_tongues(seed)
            for t in ("t1", "t2", "t3", "t4"):
                tongue = tongues[t]
                for n in range(tongue.cap + 1):
                    toks = tongue.say(n).split(" ")
                    got = read_numeral(tongue, toks)
                    self.assertEqual(got, n, (seed, t, type(tongue).__name__, n, toks))
                # a few spot checks of the shapes named in the documentation
                if isinstance(tongue, jj.Residue):
                    d = tongue.digit
                    self.assertEqual(tongue.say(0), " ".join([d[0]] * 3))
                    self.assertEqual(tongue.say(1), " ".join([d[1]] * 3))
                    self.assertEqual(tongue.say(63).split(" ")[tongue.mods.index(7)], d[0])
                    self.assertEqual(tongue.say(429), " ".join(d[{7: 2, 11: 0, 13: 0}[m]] for m in tongue.mods))
                elif isinstance(tongue, jj.Runs):
                    r = tongue.run
                    low = tongue.low_first
                    self.assertEqual(tongue.say(0), tongue.zero)
                    self.assertEqual(tongue.say(1), r[1])
                    self.assertEqual(tongue.say(511), r[9])
                    self.assertEqual(tongue.say(512), " ".join([r[1], r[9]] if not low else [r[9], r[1]]))
                    self.assertEqual(tongue.say(10), " ".join([r[1], r[1], r[1], r[1]]))
                    self.assertEqual(tongue.say(1000), " ".join([r[5], r[1], r[1], r[3]] if not low else [r[3], r[1], r[1], r[5]]))
                elif isinstance(tongue, jj.Comb):
                    d = tongue.digit
                    self.assertEqual(tongue.say(0), " ".join([d[0], d[1], d[2]] if tongue.rising else [d[2], d[1], d[0]]))
                    self.assertEqual(tongue.say(999), " ".join([d[2], d[8], d[19]] if tongue.rising else [d[19], d[8], d[2]]))
                    self.assertEqual(tongue.say(969), " ".join([d[0], d[1], d[19]] if tongue.rising else [d[19], d[1], d[0]]))
                elif isinstance(tongue, jj.Countdown):
                    d = tongue.digit
                    self.assertEqual(tongue.say(0), f"{d[0]} {d[0]}")
                    self.assertEqual(tongue.say(1), f"{d[1]} {d[0]}" if not tongue.short_first else f"{d[0]} {d[1]}")
                    self.assertEqual(tongue.say(6), f"{d[3]} {d[0]}" if not tongue.short_first else f"{d[0]} {d[3]}")
                    self.assertEqual(tongue.say(5), f"{d[3]} {d[1]}" if not tongue.short_first else f"{d[1]} {d[3]}")
                    self.assertEqual(tongue.say(990), f"{d[44]} {d[0]}" if not tongue.short_first else f"{d[0]} {d[44]}")
                    self.assertEqual(tongue.say(399), f"{d[28]} {d[7]}" if not tongue.short_first else f"{d[7]} {d[28]}")
                elif isinstance(tongue, jj.Zeckendorf):
                    f = tongue.fib
                    self.assertEqual(tongue.say(0), tongue.zero)
                    self.assertEqual(tongue.say(4), " ".join([f[2], f[0]] if not tongue.rising else [f[0], f[2]]))
                    self.assertEqual(tongue.say(987), f[14])
                    self.assertEqual(tongue.say(999), " ".join([f[14], f[4], f[2], f[0]] if not tongue.rising else [f[0], f[2], f[4], f[14]]))
                else:
                    d = tongue.digit
                    self.assertEqual(tongue.say(0), d[0])
                    self.assertEqual(tongue.say(2), d[2])
                    self.assertEqual(tongue.say(3), " ".join([d[1], d[2], d[0]] if not tongue.low_first else [d[0], d[2], d[1]]))
                    self.assertEqual(tongue.say(1), d[1])

    def test_what_is_no_numeral_is_none(self):
        for seed in range(1, 13):
            words, tongues = jj.world_tongues(seed)
            rng = jj.seeded(seed, 11)
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
                self.assertTrue(near[0] == jj.NUMBER_FIRST and near[2] != near[3])
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
            if name == "readf1":  # the numeral in the question, the number in digits in the answer
                cons.append(([runs_of_words(r[1], r[3])[0]], lambda v, n=int(r[2]): v == n))
            elif name == "writef1":  # the number in digits in the question, the numeral in the answer
                cons.append(([r[2].split(" ")], lambda v, n=int(re.findall(r"\d+", r[1])[0]): v == n))
            elif name == "joinf2":
                a, b = runs_of_words(r[1], r[3])
                cons.append(([a, b, r[2].split(" ")], lambda x, y, z: x + y == z))
            elif name == "halvef2":
                cons.append(([runs_of_words(r[1], r[3])[0], r[2].split(" ")], lambda x, y: x // 2 == y))
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
        # the one meaning gives every number its numeral back (all numbers up to the largest the grammar writes, so also every numeral of a held-out or beyond row)
        for n in range(tongue.cap + 1):
            self.assertEqual(reading(tongue.say(n).split(" "), assign), n, (seed, fam, n))
        return variant

    def test_the_first_family_is_decoded_from_the_lessons_that_tie_it_to_digits(self):
        # the kind that reads the words and the kind that writes them, taught on about a hundred numbers each: a decoder that knows the kind of grammar and nothing else
        for seed in range(1, 7):
            self.check_decoded(seed, "t1", {"readf1", "writef1"})

    def test_the_second_family_is_decoded_from_the_arithmetic_lessons_alone(self):
        # the sum and the halving of numerals, no lesson in digits: the one meaning that fits is the true one
        for seed in range(1, 7):
            self.check_decoded(seed, "t2", {"joinf2", "halvef2"})

    def test_the_third_family_has_no_lesson_that_ties_its_words_to_numbers(self):
        # the kinds over the third family copy or rearrange words (their answers follow from the words of the question alone), are arbitrary codes, or are a hard kind whose answer
        # is a number that a learner can only relate to the words by knowing its rule
        for seed in (1, 2, 3, 4, 5, 6):
            d, tongues, key, kinds = self.exam(seed)
            over3 = sorted(k.name for k in kinds.values() if "t3" in k.tins + (k.tout,))
            self.assertEqual(over3, ["penult3", "scatterpairf3", "splitcoin", "swapends3"])
            for r in read_rows(os.path.join(d, "lessons.tsv"), 5):
                name = key[r[3]]
                toks = runs_of_words(r[1], r[3])
                if name == "swapends3":
                    self.assertEqual(r[2].split(" "), toks[0] if len(toks[0]) == 1 else [toks[0][-1]] + toks[0][1:-1] + toks[0][:1])
                elif name == "penult3":
                    self.assertEqual(r[2], toks[0][-2] if len(toks[0]) > 1 else toks[0][0])
                elif name == "scatterpairf3":
                    self.assertEqual(len(toks), 2)
                    self.assertFalse(set(r[2].split(" ")) - set(tongues["t3"].words))
                elif name == "splitcoin":
                    self.assertTrue(r[2].isdigit())
            # the codes over pairs of words answer with words of the family that are no function of the words of the question (no word of the answer is a fixed place of the question)
            codes = [(runs_of_words(r[1], r[3]), r[2]) for r in read_rows(os.path.join(d, "lessons.tsv"), 5) if key[r[3]] == "scatterpairf3"]
            self.assertFalse(all(a[0][-1] == b or a[1][-1] == b for a, b in codes))

    def test_the_words_of_held_out_numerals_occur_in_lessons(self):
        # the kinds that read and write the first family, and those that add and halve in the second, show every word of it (the zero word included), and every
        # word of every held-out and beyond row of the first two families is among them
        for seed in range(1, 9):
            lessons, stream, key = jj.build(seed)
            _, tongues = jj.world_tongues(seed)
            kind = {r[0]: r[2] for r in key}
            for fam, names in (("t1", {"readf1", "writef1"}), ("t2", {"joinf2", "halvef2"})):
                for name in names:
                    seen = set()
                    for r in lessons:
                        if kind[r[3]] == name:
                            seen |= set(r[1].split(" ")) | set(r[2].split(" "))
                    self.assertEqual(set(tongues[fam].words) - seen, set(), (seed, fam, name))

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
                    self.assertGreaterEqual(sum(1 for a in args if len(set(tongue.say(a[0]).split(" "))) < len(tongue.say(a[0]).split(" "))), {jj.Comb: 0, jj.Zeckendorf: 0, jj.Countdown: 0}.get(type(tongue), 4), name)  # no word occurs twice in a numeral of these (but the zero of the countdown)

    def test_the_operations_against_known_values(self):
        self.assertEqual([jj.rem_sum(a, 3) for a in range(0, 8)], [0, 1, 3, 3, 4, 6, 6, 7])
        self.assertEqual([jj.rem_sum(a, 1) for a in (1, 10, 50)], [0, 0, 0])
        self.assertEqual([jj.prog_sum(a, 2, 3) for a in range(0, 5)], [0, 2, 7, 15, 26])
        self.assertEqual([jj.digit_run(a) for a in (0, 5, 12, 21, 123, 321, 1357, 9, 99, 4562, 7777)], [0, 5, 3, 2, 6, 3, 16, 9, 9, 15, 7])
        self.assertEqual([jj.least_turn(a) for a in (7, 10, 21, 305, 120, 4321, 100)], [7, 1, 12, 53, 12, 1432, 1])
        self.assertEqual([jj.coin_split(a) for a in (0, 11, 12, 24, 100)], [0, 11, 13, 27, 120])
        self.assertEqual([jj.is_prime(a) for a in range(0, 15)], [False, False, True, True, False, True, False, True, False, False, False, True, False, True, False])
        self.assertEqual([jj.least_prime_factor(a) for a in (2, 9, 35, 49, 97, 100)], [2, 3, 5, 7, 97, 2])
        self.assertEqual([jj.subset_count(a, 1) for a in (1, 2, 5)], [2, 4, 32])
        self.assertEqual([jj.subset_count(a, 2) for a in (1, 2, 3, 4)], [1, 2, 4, 8])
        self.assertEqual([jj.subset_count(a, 3) for a in (1, 2, 3, 4, 5)], [1, 2, 4, 6, 12])
        self.assertEqual([jj.grid_paths(a, b, 1000, 999, 1000) for a, b in ((0, 0), (1, 1), (2, 2), (3, 3), (2, 5))], [1, 2, 6, 20, 21])
        # a blocked cell: the cell (1, 1) has i * j mod 5 = 1, so the paths to (2, 2) go round it
        self.assertEqual([jj.grid_paths(a, b, 5, 1, 1000) for a, b in ((1, 1), (2, 1), (2, 2), (0, 4))], [0, 1, 2, 1])

    def test_a_dozen_small_cases_of_each_hard_kind(self):
        for seed in (1, 2):
            kinds = self.kinds(seed)
            ip_k = int(re.search(r"b \+ (\d+)", kinds["rootprod"].rule).group(1))
            self.assertEqual([kinds["rootprod"].fn(a, b) for a, b in ((0, 0), (10, 10), (99, 99), (5, 7))], [math.isqrt(ip_k), math.isqrt(100 + ip_k), math.isqrt(9801 + ip_k), math.isqrt(35 + ip_k)])
            self.assertEqual([kinds["powthird"].fn(*p) for p in ((2, 10, 25), (3, 4, 7), (7, 3, 5), (2, 5, 30))], [24, 4, 3, 2])
            self.assertEqual([kinds["remtotal"].fn(a, 4) for a in (1, 2, 3, 4, 5, 6)], [1, 3, 6, 6, 7, 9])
            self.assertEqual([kinds["progsum"].fn(*p) for p in ((3, 5, 2), (1, 9, 9), (4, 0, 1))], [21, 9, 6])
            self.assertEqual([kinds["splitcoin"].fn(a) for a in (11, 12, 100)], [11, 13, 120])

    def test_the_rules_against_second_implementations(self):
        # the sum of remainders from the sums of the full periods and the part period
        for a in range(0, 150):
            for b in range(1, 40):
                q, r = divmod(a, b)
                self.assertEqual(jj.rem_sum(a, b), q * (b * (b - 1) // 2) + r * (r + 1) // 2)
        # the progression: add up the terms
        for a in range(0, 25):
            for b in range(0, 10):
                for c in range(0, 10):
                    self.assertEqual(jj.prog_sum(a, b, c), sum(b + c * i for i in range(a)))
        # the stretch of rising digits: every stretch tried
        for a in range(0, 3000):
            ds = [int(c) for c in str(a)]
            best = 0
            for i in range(len(ds)):
                for j in range(i, len(ds)):
                    if all(ds[k] < ds[k + 1] for k in range(i, j)):
                        best = max(best, sum(ds[i : j + 1]))
            self.assertEqual(jj.digit_run(a), best, a)
        # the smallest turn: the digits turned by cutting the string and putting the front at the back
        for a in range(0, 3000):
            s = str(a)
            self.assertEqual(jj.least_turn(a), min(int(s[i:] + s[:i]) for i in range(len(s))))
            self.assertLessEqual(jj.least_turn(a), a)
        # the coin of n split: the best total found by trying every split of the coin the other way (as a recursion on a list of coins)
        def best(n, memo={}):
            if n not in memo:
                memo[n] = max(n, sum(best(m) for m in (n // 2, n // 3, n // 4))) if n else 0
            return memo[n]

        for n in range(0, 400):
            self.assertEqual(jj.coin_split(n), best(n))
            self.assertGreaterEqual(jj.coin_split(n), n)
        # the primes against a sieve, the least prime factor against trial division by every number
        sieve = [True] * 1000
        sieve[0] = sieve[1] = False
        for i in range(2, 1000):
            if sieve[i]:
                for j in range(i * i, 1000, i):
                    sieve[j] = False
        self.assertEqual([jj.is_prime(a) for a in range(1000)], sieve)
        for a in range(2, 1000):
            self.assertEqual(jj.least_prime_factor(a), [d for d in range(2, a + 1) if a % d == 0][0])
        # the subsets counted by looking at every subset (small sets), and the count for b = 1 is a power of 2
        for a in range(1, 11):
            for b in range(1, 13):
                self.assertEqual(jj.subset_count(a, b), sum(1 for mask in range(1 << a) if sum(i + 1 for i in range(a) if mask >> i & 1) % b == 0))
        # the paths on the grid counted by walking every path (small boards), and by the formula with no blocked cell
        for m, r in ((5, 1), (5, 3), (7, 2), (7, 6)):
            for a in range(0, 6):
                for b in range(0, 6):
                    count = 0
                    for ups in itertools.combinations(range(a + b), b):
                        x, y, ok = 0, 0, (0 * 0) % m != r
                        for step in range(a + b):
                            if step in ups:
                                y += 1
                            else:
                                x += 1
                            ok = ok and (x * y) % m != r
                        count += ok
                    self.assertEqual(jj.grid_paths(a, b, m, r, 10**9), count, (m, r, a, b))
        for a in range(0, 7):
            for b in range(0, 7):
                self.assertEqual(jj.grid_paths(a, b, 1000, 999, 10**9), math.comb(a + b, a))
        # the integer root of a product against the definition, the power against repeated products
        for n in range(0, 20000, 7):
            r = math.isqrt(n)
            self.assertTrue(r * r <= n < (r + 1) * (r + 1))
        for a in range(2, 12):
            for b in range(2, 12):
                for c in range(2, 12):
                    x = 1
                    for _ in range(b):
                        x = x * a % c
                    self.assertEqual(pow(a, b, c), x)

    def test_every_kind_has_enough_values_and_the_constants_are_in_range(self):
        for seed in range(1, 31):
            kinds = self.kinds(seed)
            _, tongues = jj.world_tongues(seed)
            for k in kinds.values():
                self.assertGreaterEqual(len(set(table(k))), 3 if k.name in ("swapends3", "penult3") else 5, (seed, k.name))
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
            self.assertEqual(kinds["stepf12"].hi, min(tongues["t1"].cap, kinds["stepf12"].hi))
            self.assertEqual(kinds["sheaf"].hi, 31)
            self.assertIn(int(re.search(r"mod (\d+)", kinds["sheaf"].rule).group(1)), jj.SMALL_MODS)
            self.assertIn(int(re.search(r"mod (\d+)", kinds["mulacc"].rule).group(1)), jj.SMALL_MODS)
            self.assertNotEqual(re.search(r"mod (\d+)", kinds["sheaf"].rule).group(1), re.search(r"mod (\d+)", kinds["mulacc"].rule).group(1))
            k0 = int(re.search(r"b \+ (\d+)", kinds["rootprod"].rule).group(1))
            self.assertTrue(100 <= k0 < 1000)
            m, r, p = (int(x) for x in re.search(r"i \* j mod (\d+) = (\d+), counted modulo (\d+)", kinds["gridblocked"].rule).groups())
            self.assertTrue(m in (5, 7) and 1 <= r < m and p in jj.PRIMES, (m, r, p))
            m, c = (int(x) for x in re.search(r"(\d+) \* a \+ (\d+) if a is prime", kinds["primefork"].rule).groups())
            self.assertTrue(2 <= m <= 6 and 1 <= c <= 30)
            tp, tq = (int(x) for x in re.search(r"max\(a mod (\d+), a div (\d+)\)", kinds["topper"].rule).groups())
            self.assertTrue(5 <= tp <= 12 and 3 <= tq < tp)
        for p in jj.PRIMES:
            self.assertTrue(all(p % q for q in range(2, int(math.sqrt(p)) + 1)))
        # the first branch of the prime fork is taken by the primes and the second by the others, the grid has blocked cells on the axes' neighbors, the rotation can start with 0
        kinds = self.kinds(1)
        for a in range(0, 200):
            if a in (2, 3, 5, 7, 11, 13):
                self.assertTrue(kinds["primefork"].fn(a) > a, a)
            if a in (4, 6, 8, 9, 10, 12):
                self.assertTrue(kinds["primefork"].fn(a) < a, a)
        self.assertEqual({jj.is_prime(a) for a in range(100)}, {True, False})
        self.assertEqual(kinds["turnring"].fn(305), 53)
        for seed in range(1, 31):
            k = self.kinds(seed)["gridblocked"]
            self.assertGreaterEqual(sum(1 for a in range(0, 31) for b in range(0, 31) if k.fn(a, b) == 0), 20)

    def test_the_arbitrary_codes_are_not_a_pattern(self):
        for seed in (1, 2, 3):
            kinds = self.kinds(seed)
            _, tongues = jj.world_tongues(seed)
            cells = [kinds["scatterpairf3"].fn(a, b) for a in range(16) for b in range(16)]
            self.assertGreaterEqual(len(set(cells)), 25)
            self.assertLess(max(cells), min(200, tongues["t3"].cap + 1))
            rows = {tuple(kinds["scatterpairf3"].fn(a, b) for b in range(16)) for a in range(16)}
            self.assertEqual(len(rows), 16)
            self.assertLess(max(kinds["tagcode"].fn(a) for a in range(32)), 19)
            self.assertLess(max(kinds["scatterf1"].fn(a) for a in range(32)), min(300, tongues["t1"].cap + 1))
            self.assertLess(max(kinds["scatterf2"].fn(a) for a in range(32)), 55)
            # the remainder that is not one: a mod m for most numbers, something else for at least eight of them
            m = int(re.match(r"a mod (\d+) for most", kinds["nearmod6"].rule).group(1))
            self.assertIn(m, (5, 6, 8, 9))
            codes = [kinds["nearmod6"].fn(a) for a in range(32)]
            self.assertGreaterEqual(sum(1 for a in range(32) if codes[a] != a % m), 8)
            self.assertGreaterEqual(sum(1 for a in range(32) if codes[a] == a % m), 14)
            self.assertTrue(all(kinds[n].role == "unexplainable" for n in ("tagcode", "nearmod6", "scatterf1", "scatterf2", "scatterpairf3")))

    def test_the_trap_rows(self):
        expected = {"two kinds": 8, "new word": 8, "almost a kind word": 4, "arity": 8, "other form": 12, "one number in another form": 4, "bare": 6, "junk": 6, "repeat": 6,
                    "almost": 6, "trailing numeral": 6, "order": 6, "new start": 6, "new end": 6, "other core": 6, "word inside": 6, "markers exchanged": 6,
                    "mismatched ends": 4, "left out": 4}
        self.assertEqual(sum(expected.values()), 118)
        for seed in (1, 2, 3):
            lessons, stream, key, categories = jj.build_full(seed)
            _, tongues = jj.world_tongues(seed)
            words = {r[0]: r[2] for r in key}
            kinds = self.kinds(seed)
            taught_questions = {r[1] for r in lessons}
            traps = [r for r in stream if r[1] == "trap"]
            self.assertEqual(len(traps), 118)
            self.assertEqual(len({r[3] for r in traps}), 118)
            self.assertFalse({r[3] for r in traps} & taught_questions)
            self.assertGreaterEqual(len({r[4] for r in traps}), 45)  # the truths are drawn one by one
            counts = {}
            for c in categories.values():
                counts[c] = counts.get(c, 0) + 1
            self.assertEqual(counts, expected)
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
                    self.assertTrue(unit in toks and unit.endswith(jj.ENDING), q)
                elif cat == "almost a kind word":
                    self.assertNotIn(unit, words)
                    near = [w for w in words if len(w) == len(unit) and sum(a != b for a, b in zip(unit, w)) == 1]
                    self.assertEqual(len(near), 1, q)
                    self.assertTrue(unit in toks and unit.endswith(jj.ENDING), q)
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
                    self.assertTrue(valid_anywhere(re.sub(r"^(just|how about) ", "", q)), q)
                elif cat in ("junk", "repeat", "almost"):
                    k = kinds[words[unit]]
                    self.assertFalse(fits(q, unit, k.arity, of_kind(tongues, k)), q)
                    anything = lambda x: bool(x) and not set(x.split(" ")) & jj.FRAME_WORDS  # noqa: E731  (a run of words that holds no word of the ways of asking)
                    self.assertTrue(any(template_matches(f.toks, unit, q, [anything] * k.arity) for f in jj.FRAMES[k.arity]), q)
                elif cat == "trailing numeral":
                    k = kinds[words[unit]]
                    t4 = tongues["t4"]
                    cuts = [i for i in range(1, len(toks)) if t4.valid(" ".join(toks[i:])) and fits(" ".join(toks[:i]), unit, k.arity, of_kind(tongues, k))]
                    self.assertTrue(cuts, q)  # an ordinary question and then a numeral of the fourth family
                elif cat == "order":
                    k = kinds[words[unit]]
                    self.assertIn(unit, toks)
                    self.assertFalse(any(toks[0] == f.toks[0] for f in jj.FRAMES[k.arity] if f.toks[0] not in jj.SLOTS), q)
                elif cat == "new start":
                    start = [s for s in jj.NEW_STARTS if q.startswith(" ".join(s) + " ")]
                    self.assertEqual(len(start), 1, q)
                    self.assertFalse({s[0] for s in jj.NEW_STARTS} & {f.toks[0] for fs in jj.FRAMES.values() for f in fs})  # no kind has a way that starts so
                elif cat == "new end":
                    k = kinds[words[unit]]
                    ends = [e for e in jj.NEW_ENDS if q.endswith(" " + " ".join(e))]
                    self.assertEqual(len(ends), 1, q)
                    body = q[: -len(" " + " ".join(ends[0]))]
                    self.assertTrue(fits(body, unit, k.arity, of_kind(tongues, k)), q)  # the rest is an ordinary question
                    self.assertFalse({e[-1] for e in jj.NEW_ENDS} & {f.toks[-1] for fs in jj.FRAMES.values() for f in fs})  # no kind has a way that ends so
                elif cat == "other core":
                    self.assertNotIn(unit, toks)
                    self.assertFalse(word_of(q))
                elif cat == "word inside":
                    where = [i for i, t in enumerate(toks) if t == unit]
                    numeral_words = {w for t in ("t1", "t2", "t3") for w in tongues[t].words}
                    self.assertEqual(len(where), 2, q)  # the kind's word in its place and once more inside a numeral
                    self.assertTrue(any(toks[i - 1] in numeral_words and toks[i + 1] in numeral_words for i in where if 0 < i < len(toks) - 1), q)
                elif cat == "markers exchanged":
                    k = kinds[words[unit]]
                    self.assertTrue(any(template_matches(swap, unit, q, [t.valid for t in of_kind(tongues, k)]) for f in jj.FRAMES[k.arity] for swap in self.exchanged(f)), q)
                    self.assertTrue(order_matters(k), q)  # the order of the numbers matters
                elif cat == "mismatched ends":
                    k = kinds[words[unit]]
                    ok = [t.valid for t in of_kind(tongues, k)]
                    self.assertTrue(any(template_matches(x.toks[: x.w + 1] + y.toks[y.w + 1 :], unit, q, (ok * 3)[: 3]) for x in jj.FRAMES[k.arity] for y in jj.FRAMES[3 - k.arity]), q)
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
        a, b = self.write(jj, 3), self.write(jj, 3, compose_first=True)
        la, lb = read_rows(os.path.join(a, "lessons.tsv"), 5), read_rows(os.path.join(b, "lessons.tsv"), 5)
        self.assertNotEqual(la, lb)
        ka, kb = read_rows(os.path.join(a, "key.tsv"), 5), read_rows(os.path.join(b, "key.tsv"), 5)
        roles = [r[1] for r in kb]
        self.assertLess(roles.index("compose"), roles.index("newop"))
        self.assertLess([r[1] for r in ka].index("newop"), [r[1] for r in ka].index("compose"))

    @unittest.skipIf(np is None, "numpy is needed for the reachability search")
    @unittest.skipIf(os.environ.get("HW_J_SKIP_SEARCH"), "HW_J_SKIP_SEARCH is set")
    def test_no_hard_kind_is_reachable_and_the_controls_are(self):
        seeds = range(1, int(os.environ.get("HW_J_SEARCH_SEEDS", "6")) + 1)
        groups, roles = {}, {}
        for seed in seeds:
            for group in jj.make_kinds(seed):
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
