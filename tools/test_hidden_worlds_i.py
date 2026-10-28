# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""Tests for tools/hidden_worlds_i.py: it is deterministic, writes the format `baby exam adapt-write` writes, the key never reaches the exam files, every answer
follows from its rule, every numeral reads back to its number (checked against a second reading of each grammar), the ways of asking are what the key says, every trap
row is what its category says, no table, name, way of asking or invented word is one of the earlier sets', and (with numpy) no hard kind is reachable by a program of
at most 7 nodes while the controls are.

    python3 -m unittest discover -s tools -p 'test_hidden_worlds_i.py'

The reachability test (seeds 1 to 6 by default, `HW_I_SEARCH_SEEDS=N` for 1 to N; the time is mostly the search itself) is skipped without numpy or with `HW_I_SKIP_SEARCH=1`.
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
import hidden_worlds_i as ii

try:
    import numpy as np
except ImportError:  # the reachability search is the only part that needs it
    np = None

EARLIER = (first, second, third, fourth, fifth, sixth, seventh, eighth)
KEY_COLUMNS = ((first, 4), (second, 4), (third, 4), (fourth, 5), (fifth, 5), (sixth, 5), (seventh, 5), (eighth, 5))
ROLES = {"newop", "compose", "control", "unexplainable", "trap"}
FAMILIES = ("d", "t1", "t2", "t3", "t4", "m1")
MAX_WORDS = 22  # the longest numeral of any grammar has 17 words
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
        elif t in ii.SLOTS:
            ok = oks[ii.SLOTS.index(t)]
            for n in range(1, MAX_WORDS + 1):
                text = " ".join(q[j : j + n])
                if j + n <= len(q) and ok(text):
                    yield from walk(i + 1, j + n, args + [text])
        elif j < len(q) and q[j] == t:
            yield from walk(i + 1, j + 1, args)

    return list(walk(0, 0, []))


def every_template(arity):
    """{text: (kind, tokens, frame ids)}: the ways of asking, then every one with a filler left out, then every mix of the start of one and the end of another."""
    frames = ii.FRAMES[arity]
    out = {f.text: ("frame", f.toks, (i,)) for i, f in enumerate(frames)}
    for i, f in enumerate(frames):
        for d in f.drop:
            toks = f.toks[:d] + f.toks[d + 1 :]
            if " ".join(toks) not in out:
                out[" ".join(toks)] = ("drop", toks, (i,))
    for i, x in enumerate(frames):
        for j, y in enumerate(frames):
            toks = ii.hybrid_tokens(x, y) if i != j else None
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


class HiddenWorldsI(unittest.TestCase):
    def write(self, module, seed, **kw):
        d = tempfile.mkdtemp()
        self.addCleanup(lambda: __import__("shutil").rmtree(d, ignore_errors=True))
        module.write(d, seed, **kw)
        return d

    def kinds(self, seed):
        _, tongues = ii.world_tongues(seed)
        return {k.name: k for group in ii.make_kinds(seed, tongues) for k in group}

    def exam(self, seed):
        d = self.write(ii, seed)
        _, tongues = ii.world_tongues(seed)
        key = read_rows(os.path.join(d, "key.tsv"), 5)
        return d, tongues, {r[0]: r[2] for r in key}, self.kinds(seed)

    def test_same_seed_same_files_other_seed_other_files(self):
        a, b, c = self.write(ii, 1), self.write(ii, 1), self.write(ii, 2)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv", "README.txt"):
            self.assertEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(b, name)), name)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv"):
            self.assertNotEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(c, name)), name)

    def test_the_files_parse_in_the_existing_format(self):
        for seed in (1, 2, 3):
            d = self.write(ii, seed)
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
            d = self.write(ii, seed)
            key = read_rows(os.path.join(d, "key.tsv"), 5)
            exam = "".join(read_bytes(os.path.join(d, name)).decode("utf-8") for name in ("lessons.tsv", "stream.tsv", "README.txt"))
            for word, role, name, rule, learn in key:
                self.assertTrue(rule and learn)
                self.assertNotIn(name, exam, name)
                self.assertNotIn(rule, exam, name)
                self.assertNotIn(learn, exam, name)
            # what the exam files hold is the fixed words of the questions, the kinds' words, the numerals, the numbers, and (in trap rows only) new words, words one letter
            # away from a kind word or a number word, and the words of a fourth family that no lesson uses
            _, tongues = ii.world_tongues(seed)
            numeral_words = {w for t in ("t1", "t2", "t3") for w in tongues[t].words}
            fourth_words = set(tongues["t4"].words)
            kind_words = {r[0] for r in key}
            allowed = set(ii.FRAME_WORDS) | kind_words | numeral_words
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
            self.assertTrue(all(w.endswith(ii.ENDING) for w in new | near_kind))
            self.assertGreaterEqual(len(near_kind), 4)
            self.assertEqual(len(near_numeral), 6)
            self.assertTrue(strip(stream) & fourth_words)
            self.assertFalse(strip(lessons) & fourth_words)
            self.assertEqual(sorted(os.listdir(d)), ["README.txt", "key.tsv", "lessons.tsv", "stream.tsv"])

    def test_the_kinds_by_role(self):
        for seed in (1, 2, 3):
            d = self.write(ii, seed)
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
            self.assertEqual({fam: sum(1 for k in kinds if fam in k.tins + (k.tout,)) for fam in ("t1", "t2", "t3", "t4", "m1")}, {"t1": 10, "t2": 6, "t3": 4, "t4": 0, "m1": 1})
            self.assertEqual(sum(1 for k in kinds if any(t != "d" for t in k.tins) or k.tout != "d"), 20)
            self.assertEqual(sum(1 for k in kinds if len(set(k.tins)) > 1), 3)  # one number as digits and another as words, or the form drawn (cubepair, mixpair, mixwrite)
            self.assertEqual(sorted(k.name for k in kinds if len(set(k.tins)) > 1 and k.arity > 1), ["cubepair", "mixpair", "mixwrite"])
            self.assertEqual(sorted(k.name for k in kinds if k.arity == 3), ["absring", "capsum", "cubeprod", "trimin"])
            self.assertEqual(sorted(k.name for k in kinds if k.bridge), ["capsum", "farness", "slanted"])
            # a question in one form and the answer in another (digits to words, words to digits, a family to another family)
            self.assertEqual(sorted(k.name for k in kinds if k.tout in ("t1", "t2", "t3") and any(t != k.tout for t in k.tins)), ["dfafar", "encode1", "mixwrite", "step12"])

    def test_the_words_are_this_sets_own(self):
        mine, invented = set(), set()
        for seed in range(1, 7):
            d = self.write(ii, seed)
            words = {r[0] for r in read_rows(os.path.join(d, "key.tsv"), 5)}
            self.assertEqual(len(words), 32)
            self.assertTrue(all(w.endswith(ii.ENDING) for w in words))
            mine |= words
            _, tongues = ii.world_tongues(seed)
            invented |= words | {w for t in ("t1", "t2", "t3", "t4") for w in tongues[t].words}
            stream = read_bytes(os.path.join(d, "stream.tsv")).decode("utf-8")
            invented |= set(re.findall(r"[a-z]+", re.sub(r"\bS\d{5}\b", "", stream))) - ii.FRAME_WORDS - ROLES
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
        earlier |= {f.text for fs in eighth.FRAMES.values() for f in fs}
        for ar, frames in ii.FRAMES.items():
            self.assertEqual(len(frames), 8 if ar < 3 else 6)
            self.assertEqual(len({f.text for f in frames}), len(frames))
            self.assertFalse({f.text for f in frames} & earlier)
            self.assertFalse({" ".join(f.toks[:f.w + 1]) for f in frames if f.w} & {" ".join(t.split(" ")[:t.split(" ").index("{w}") + 1]) for t in earlier if "{w}" in t.split(" ")})
        # a word of a way occurs more than once in some of them, and some ways have a number next to a number
        self.assertTrue(any(len(f.toks) > len(set(f.toks)) for fs in ii.FRAMES.values() for f in fs))

    def test_no_kind_is_one_of_the_earlier_sets(self):
        # by name, and by table: none of this set's kinds equals any earlier kind (the controls and the arbitrary codes included) on its whole domain or on the numbers below 32;
        # the kinds that read and write words, add words, halve words and the like are the identity, a sum or a floor of a half on a domain of their own
        earlier = [k for module in EARLIER for seed in range(1, 6) for group in module.make_kinds(seed) for k in group]
        names = {k.name for k in earlier}
        tables = {(k.arity, k.lo, 31, earlier_table(k)) for k in earlier}
        for seed in range(1, 6):
            for group in ii.make_kinds(seed):
                for k in group:
                    self.assertNotIn(k.name, names)
                    self.assertFalse((k.arity, k.lo, k.hi, table(k)) in tables, (seed, k.name))
                    if k.hi >= 31 and k.name not in ("decode1", "encode1", "sumw1", "join2", "halve2"):  # on the first 32 numbers too
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
                self.assertIn(len(ids), ((len(ii.FRAMES[k.arity]),) if k.bridge else (2, 3)), k.name)
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
                for kind in ("quadfloor", "cubepair", "mixpair", "mixwrite"):
                    unit = [u for u, n in key.items() if n == kind][0]
                    k = kinds[kind]
                    rows = [(r[1] if lesson else r[3]) for r in read_rows(os.path.join(d, name), 5) if (r[3] if lesson else r[2]) == unit and r[1] != "trap"]
                    forms = [set() for _ in range(k.arity)]
                    for q in rows:
                        (_, _, args), = fits(q, unit, k.arity, of_kind(tongues, k))
                        for i, x in enumerate(args):
                            forms[i].add(forms_of(x))
                    if kind == "quadfloor":  # the form is drawn for each number
                        self.assertEqual(forms[0], {"digits", "words"}, (name, kind))
                    else:  # one number always in digits and the other always in words
                        self.assertEqual([sorted(f) for f in forms], [["digits"], ["words"]] if kind != "mixwrite" else [["words"], ["digits"]], (name, kind))
            # the kinds that read and write words keep their forms apart: words in, digits out; digits in, words out; a question in one family is answered in another
            for r in read_rows(os.path.join(d, "lessons.tsv"), 5):
                if key[r[3]] == "decode1":
                    self.assertTrue(r[2].isdigit() and not any(x.isdigit() for x in r[1].split(" ")), r)
                if key[r[3]] == "encode1":
                    self.assertFalse(r[2].isdigit(), r)
                if key[r[3]] == "step12":
                    self.assertFalse(r[2].isdigit() or set(r[2].split(" ")) & set(tongues["t1"].words), r)
                    self.assertTrue(set(r[2].split(" ")) <= set(tongues["t2"].words), r)

    def test_the_numerals(self):
        grammars = {}
        for seed in range(1, 13):
            words, tongues = ii.world_tongues(seed)
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
                self.assertFalse(set(tongue.words) & set(ii.FRAME_WORDS))
                self.assertTrue(all(len(w) == 4 and w[0] in ii.NEW_LETTERS for w in tongue.words))
                self.assertGreaterEqual(tongue.cap, 199)
                said = [tongue.say(n) for n in range(tongue.cap + 1)]
                self.assertEqual(len(set(said)), tongue.cap + 1, (seed, t))
                for n, text in enumerate(said):
                    self.assertEqual(tongue.parse(text), n, (seed, t, n))
                    self.assertLessEqual(len(text.split(" ")), MAX_WORDS - 3)
                    self.assertTrue(set(text.split(" ")) <= set(tongue.words))
            # the grammars are written with words of their own
            self.assertTrue(all(set(tongues[t].foreign) == seen_words - set(tongues[t].words) for t in ("t1", "t2", "t3", "t4")))
            self.assertTrue(tongues["m1"].valid("17") and tongues["m1"].valid(tongues["t1"].say(17)) and not tongues["m1"].valid(tongues["t2"].say(17)))
        self.assertEqual(sorted(grammars), ["Biquinary", "Chunks", "Doubling", "Fives", "Sandhi", "Vigesimal"])
        self.assertTrue(all(len(v) >= 2 for v in grammars.values()), grammars)
        # seeds 1 to 3: a twenties count in two surface forms, and the grammars none of the earlier sets used
        self.assertEqual([[type(ii.world_tongues(s)[1][t]).__name__ for t in ("t1", "t2", "t3")] for s in (1, 2, 3)],
                         [["Vigesimal", "Doubling", "Biquinary"], ["Sandhi", "Fives", "Chunks"], ["Fives", "Biquinary", "Doubling"]])

    def test_each_grammar_does_what_it_says(self):
        """A second reading of each grammar, from its words and the rule in the docstring, against `say` for every number."""
        for seed in range(1, 13):
            _, tongues = ii.world_tongues(seed)
            for t in ("t1", "t2", "t3", "t4"):
                tongue = tongues[t]
                for n in range(tongue.cap + 1):
                    toks = tongue.say(n).split(" ")
                    if toks == [tongue.zero if hasattr(tongue, "zero") else None]:
                        got = 0
                    elif isinstance(tongue, ii.Vigesimal):
                        value = {w: i for i, w in tongue.digit.items()}
                        value[tongue.ten], value[tongue.fifteen] = 10, 15
                        end = toks.index(tongue.score) if tongue.score in toks else None
                        if end is None:
                            top, low = [], toks
                        elif tongue.low_first:
                            top, low = toks[end + 1 :], toks[:end]
                        else:
                            top, low = toks[:end], toks[end + 1 :]
                        got = 20 * sum(value[w] for w in top) + sum(value[w] for w in low)
                        self.assertTrue(n == 0 or toks.count(tongue.score) <= 1)
                    elif isinstance(tongue, ii.Biquinary):
                        value = {w: i for i, w in tongue.digit.items()}
                        value[tongue.five_u] = value[tongue.five_t] = 5
                        got, pending = 0, 0
                        for w in toks:
                            if w == tongue.hund:
                                got, pending = got + 100 * pending, 0
                            elif w == tongue.tens:
                                got, pending = got + 10 * pending, 0
                            else:
                                pending += value[w]
                        got += pending
                        # the five of the tens is in front of the tens word only, the five of the units only after the last place word
                        if tongue.five_t in toks:
                            self.assertIn(tongue.tens, toks[toks.index(tongue.five_t) :])
                    elif isinstance(tongue, ii.Doubling):
                        self.assertEqual(toks[0], tongue.one)
                        got = 1
                        for w in toks[1:]:
                            got = got * 2 if w == tongue.twice else got + 1
                        self.assertNotIn(tongue.zero, toks)
                    elif isinstance(tongue, ii.Chunks):
                        value = tongue.value
                        self.assertLessEqual(len(toks), 2)
                        if len(toks) == 1:
                            got = value[toks[0]]
                        else:
                            hi, lo = (toks[1], toks[0]) if tongue.low_first else (toks)
                            got = value[hi] * 100 + value[lo]
                            self.assertTrue(1 <= value[hi] <= 9)
                    elif isinstance(tongue, ii.Sandhi):
                        base = {w: i for i, w in tongue.digit.items()}
                        got = 0
                        for i, w in enumerate(toks):
                            nxt = toks[i + 1] if i + 1 < len(toks) else None
                            if w in tongue.before_ten.values():
                                self.assertEqual(nxt, tongue.ten)
                                got += 10 * [k for k, v in tongue.before_ten.items() if v == w][0]
                            elif w in tongue.before_hund.values():
                                self.assertEqual(nxt, tongue.hund)
                                got += 100 * [k for k, v in tongue.before_hund.items() if v == w][0]
                            elif w in base:
                                self.assertIsNone(nxt)
                                got += base[w]
                    else:
                        value = {w: i for i, w in tongue.digit.items()}
                        got = 0
                        segments = [[]]
                        for w in toks:
                            if w == tongue.link:
                                segments.append([])
                            else:
                                segments[-1].append(w)
                        self.assertLessEqual(len(segments), 2)
                        for seg in segments:
                            pending = 0
                            for w in seg:
                                if w == tongue.score:
                                    got, pending = got + 20 * pending, 0
                                elif w == tongue.five:
                                    got, pending = got + 5 * pending, 0
                                else:
                                    pending = value[w]
                            got += pending
                        if len(segments) == 2:  # the twenties are on the side the variant says
                            twenties = 1 if tongue.first else 0
                            self.assertIn(tongue.score, segments[twenties])
                            self.assertNotIn(tongue.score, segments[1 - twenties])
                    self.assertEqual(got, n, (seed, t, type(tongue).__name__, n, toks))
                # a few spot checks of the shapes named in the documentation
                if isinstance(tongue, ii.Vigesimal):
                    d = tongue.digit
                    self.assertEqual(tongue.say(60), " ".join([tongue.score, d[3]] if tongue.low_first else [d[3], tongue.score]))
                    self.assertEqual(tongue.say(20), " ".join([tongue.score, d[1]] if tongue.low_first else [d[1], tongue.score]))
                    self.assertEqual(tongue.say(35), " ".join([tongue.fifteen, tongue.score, d[1]] if tongue.low_first else [d[1], tongue.score, tongue.fifteen]))
                    self.assertEqual(tongue.say(11), f"{tongue.ten} {d[1]}")
                    self.assertEqual(tongue.say(15), tongue.fifteen)
                    self.assertEqual(tongue.say(19), f"{tongue.fifteen} {d[4]}")
                    self.assertEqual(tongue.say(10 * 20), " ".join([tongue.score, tongue.ten] if tongue.low_first else [tongue.ten, tongue.score]))
                elif isinstance(tongue, ii.Biquinary):
                    self.assertEqual(tongue.say(5), tongue.five_u)
                    self.assertEqual(tongue.say(50), f"{tongue.five_t} {tongue.tens}")
                    self.assertEqual(tongue.say(100), f"{tongue.digit[1]} {tongue.hund}")
                    self.assertEqual(tongue.say(444), " ".join([tongue.digit[4], tongue.hund] + tongue.part(4, tongue.five_t) + [tongue.tens] + tongue.part(4, tongue.five_u)))
                    self.assertEqual(tongue.say(4), tongue.digit[4])
                elif isinstance(tongue, ii.Doubling):
                    self.assertEqual(tongue.say(1), tongue.one)
                    self.assertEqual(tongue.say(4), f"{tongue.one} {tongue.twice} {tongue.twice}")
                    self.assertEqual(tongue.say(5), f"{tongue.one} {tongue.twice} {tongue.twice} {tongue.plus}")
                    self.assertEqual(tongue.say(0), tongue.zero)
                elif isinstance(tongue, ii.Chunks):
                    self.assertEqual(tongue.say(99), tongue.chunk[99])
                    self.assertEqual(tongue.say(7), tongue.chunk[7])
                    self.assertEqual(tongue.say(300), " ".join([tongue.chunk[0], tongue.chunk[3]] if tongue.low_first else [tongue.chunk[3], tongue.chunk[0]]))
                elif isinstance(tongue, ii.Sandhi):
                    self.assertEqual(tongue.say(10), f"{tongue.before_ten[1]} {tongue.ten}")
                    self.assertEqual(tongue.say(300), f"{tongue.before_hund[3]} {tongue.hund}")
                    self.assertEqual(tongue.say(305), f"{tongue.before_hund[3]} {tongue.hund} {tongue.digit[5]}")
                    self.assertTrue(all(a[:3] == b[:3] == c[:3] and len({a, b, c}) == 3 for a, b, c in zip(tongue.digit.values(), tongue.before_ten.values(), tongue.before_hund.values())))
                else:
                    d = tongue.digit
                    self.assertEqual(tongue.say(20), f"{d[1]} {tongue.score}")
                    self.assertEqual(tongue.say(5), f"{d[1]} {tongue.five}")
                    self.assertEqual(tongue.say(4), d[4])
                    rest = f"{d[3]} {tongue.five} {d[4]}"
                    self.assertEqual(tongue.say(199), f"{rest} {tongue.link} {d[9]} {tongue.score}" if tongue.first else f"{d[9]} {tongue.score} {tongue.link} {rest}")

    def test_what_is_no_numeral_is_none(self):
        for seed in range(1, 13):
            words, tongues = ii.world_tongues(seed)
            rng = ii.seeded(seed, 11)
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
                self.assertTrue(near[0] in ii.NEW_LETTERS)
                self.assertEqual(sum(a != b for a, b in zip(word, near)), 1)
            self.assertTrue(tongues["d"].valid("17") and not tongues["d"].valid("seventeen"))

    def test_the_words_of_held_out_numerals_occur_in_lessons(self):
        # the kinds that read and write the first family, and those that add and halve in the second, show every word of it (the zero word included), and every
        # word of every held-out and beyond row of the first two families is among them
        for seed in range(1, 9):
            lessons, stream, key = ii.build(seed)
            _, tongues = ii.world_tongues(seed)
            kind = {r[0]: r[2] for r in key}
            for fam, names in (("t1", {"decode1", "encode1"}), ("t2", {"join2", "halve2"})):
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
                    self.assertGreaterEqual(sum(1 for a in args if len(set(tongue.say(a[0]).split(" "))) < len(tongue.say(a[0]).split(" "))), {ii.Sandhi: 0, ii.Chunks: 1}.get(type(tongue), 4), name)  # a chunk word repeats only where the hundreds equal the last two digits

    def test_the_operations_against_known_values(self):
        self.assertEqual([ii.cantor_count(n) for n in range(14)], [1, 2, 2, 3, 4, 4, 4, 4, 4, 5, 6, 6, 7, 8])
        self.assertEqual([ii.tail_pairs(a, b) for a, b in ((0, 0), (1, 0), (1, 1), (5, 3), (2, 0))], [1, 3, 3, 6, 3])
        self.assertEqual([ii.mod_inverse(a, 7) for a in range(1, 7)], [1, 4, 5, 2, 3, 6])
        self.assertEqual(ii.mod_inverse(2, 101), 51)
        self.assertEqual([ii.board_squares(a, b) for a, b in ((1, 1), (2, 2), (3, 3), (2, 3), (1, 5), (4, 4))], [1, 5, 14, 8, 5, 30])
        self.assertEqual([a for a in range(20) if ii.adjacent_ones(a)], [3, 6, 7, 11, 12, 13, 14, 15, 19])
        parity = {0: (0, 1), 1: (1, 0)}
        self.assertEqual([ii.run_dfa(a, parity, [0, 1]) for a in range(8)], [0, 1, 1, 0, 1, 0, 0, 1])

    def test_the_rules_against_second_implementations(self):
        # the count of numbers without a 2 in base 3, from the digits
        for a in range(0, 300):
            digits = lambda x: [] if x == 0 else digits(x // 3) + [x % 3]  # noqa: E731
            self.assertEqual(ii.cantor_count(a), len([x for x in range(a + 1) if 2 not in digits(x)]))
        # the walk of pairs, with the next pair worked out by hand and the first repeat found by comparing with all the earlier ones in a list
        for a in range(0, 25):
            for b in range(0, 25):
                path = [(a, b)]
                while True:
                    x, y = path[-1]
                    nxt = (y, x - y if x >= y else y - x)
                    if nxt in path:
                        break
                    path.append(nxt)
                self.assertEqual(ii.tail_pairs(a, b), len(path))
        # the inverse against Python's own, and the product comes back to 1
        for p in (59, 101, 211):
            for a in range(1, p):
                self.assertEqual(ii.mod_inverse(a, p), pow(a, -1, p))
        # the squares of a board by counting the squares one corner and one size at a time
        for a in range(1, 9):
            for b in range(1, 9):
                count = sum(1 for x in range(a) for y in range(b) for s in range(1, 9) if x + s <= a and y + s <= b)
                self.assertEqual(ii.board_squares(a, b), count)
                self.assertEqual(ii.board_squares(a, b), ii.board_squares(b, a))
        # two 1 digits side by side: the string has "11" in it
        for a in range(0, 600):
            self.assertEqual(ii.adjacent_ones(a), "11" in bin(a))
        # the machine run on the string of digits, with the table read the other way (a row for each digit)
        for seed in range(1, 5):
            kinds = self.kinds(seed)
            rule = kinds["dfa2"].rule
            outs = eval(re.search(r"output (\[[^\]]*\])", rule).group(1))
            table_ = eval(re.search(r"transitions (\{[^}]*\})", rule).group(1))
            cols = {digit: {s: table_[s][digit] for s in table_} for digit in (0, 1)}
            for a in range(0, 256):
                state = 0
                for c in (bin(a)[2:] if a else ""):
                    state = cols[int(c)][state]
                self.assertEqual(kinds["dfa2"].fn(a), outs[state])

    def test_every_kind_has_enough_values_and_the_constants_are_in_range(self):
        for seed in range(1, 31):
            kinds = self.kinds(seed)
            _, tongues = ii.world_tongues(seed)
            for k in kinds.values():
                self.assertGreaterEqual(len(set(table(k))), 3 if k.name in ("rot3", "middle3") else 5, (seed, k.name))
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
            m = int(re.search(r"mod (\d+)", kinds["cubepair"].rule).group(1))
            self.assertTrue(m in ii.PRIMES and m >= 59)
            m = int(re.search(r"modulo (\d+)", kinds["modinv"].rule).group(1))
            self.assertTrue(m in ii.PRIMES and m >= 101)
            self.assertTrue(all(kinds["modinv"].fn(a) * a % m == 1 for a in range(1, 101)))
            self.assertIn(int(re.search(r"mod (\d+)", kinds["cubeprod"].rule).group(1)), ii.SMALL_MODS)
            q = re.search(r"\(a \* a \+ (\d+) \* a \+ (\d+)\) div (\d+)", kinds["quadfloor"].rule)
            self.assertTrue(11 <= int(q.group(1)) < 41 and 100 <= int(q.group(2)) < 900 and 7 <= int(q.group(3)) < 32)
            self.assertEqual(kinds["step12"].hi, min(tongues["t1"].cap, kinds["step12"].hi))
            self.assertEqual(kinds["capsum"].hi, 31)
        for p in ii.PRIMES:
            self.assertTrue(all(p % q for q in range(2, int(math.sqrt(p)) + 1)))
        # the three-number sum of differences is twice the spread, the third of the minimum forms each win, the branch takes both sides
        kinds = self.kinds(1)
        self.assertTrue(all(kinds["absring"].fn(a, b, c) == 2 * (max(a, b, c) - min(a, b, c)) for a in range(0, 41, 4) for b in range(0, 41, 5) for c in range(0, 41, 3)))
        for seed in range(1, 31):
            k = self.kinds(seed)["trimin"]
            pts = list(itertools.product(range(0, 41, 2), repeat=3))
            tm = [int(x) for x in re.findall(r"(\d+) \*", k.rule)]
            forms = [lambda a, b, c: tm[0] * a + tm[1] * b, lambda a, b, c: tm[2] * b + tm[3] * c, lambda a, b, c: tm[4] * c + tm[5] * a]
            for f in forms:
                self.assertGreater(sum(1 for p in pts if f(*p) == k.fn(*p)), len(pts) // 20)
            self.assertEqual({ii.adjacent_ones(a) for a in range(100)}, {True, False})
            by_bits, by_rest = {}, {m: {} for m in range(2, 9)}
            dfa = self.kinds(seed)["dfa2"]
            vals = [dfa.fn(a) for a in range(128)]
            self.assertFalse(all(by_bits.setdefault(bin(a).count("1"), vals[a]) == vals[a] for a in range(128)))
            for m in range(2, 9):
                self.assertFalse(all(by_rest[m].setdefault(a % m, vals[a]) == vals[a] for a in range(128)), (seed, m))

    def test_the_arbitrary_codes_are_not_a_pattern(self):
        for seed in (1, 2, 3):
            kinds = self.kinds(seed)
            _, tongues = ii.world_tongues(seed)
            cells = [kinds["scatterpair3"].fn(a, b) for a in range(16) for b in range(16)]
            self.assertGreaterEqual(len(set(cells)), 25)
            self.assertLess(max(cells), min(200, tongues["t3"].cap + 1))
            rows = {tuple(kinds["scatterpair3"].fn(a, b) for b in range(16)) for a in range(16)}
            self.assertEqual(len(rows), 16)
            self.assertLess(max(kinds["dotcode"].fn(a) for a in range(32)), 17)
            self.assertLess(max(kinds["scatterw1"].fn(a) for a in range(32)), min(300, tongues["t1"].cap + 1))
            self.assertLess(max(kinds["scatterw2"].fn(a) for a in range(32)), 50)
            # the remainder that is not one: a mod m for most numbers, something else for at least eight of them
            m = int(re.match(r"a mod (\d+) for most", kinds["nearmod5"].rule).group(1))
            self.assertIn(m, (5, 6, 8, 9))
            codes = [kinds["nearmod5"].fn(a) for a in range(32)]
            self.assertGreaterEqual(sum(1 for a in range(32) if codes[a] != a % m), 8)
            self.assertGreaterEqual(sum(1 for a in range(32) if codes[a] == a % m), 14)
            self.assertTrue(all(kinds[n].role == "unexplainable" for n in ("dotcode", "nearmod5", "scatterw1", "scatterw2", "scatterpair3")))

    def test_the_trap_rows(self):
        expected = {"two kinds": 8, "new word": 8, "almost a kind word": 4, "arity": 8, "other form": 12, "one number in another form": 4, "bare": 6, "junk": 6, "repeat": 6,
                    "almost": 6, "trailing numeral": 6, "order": 6, "new start": 6, "new end": 6, "other core": 6, "word inside": 6, "markers exchanged": 6,
                    "mismatched ends": 4, "left out": 4}
        self.assertEqual(sum(expected.values()), 118)
        for seed in (1, 2, 3):
            lessons, stream, key, categories = ii.build_full(seed)
            _, tongues = ii.world_tongues(seed)
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
                    self.assertTrue(unit in toks and unit.endswith(ii.ENDING), q)
                elif cat == "almost a kind word":
                    self.assertNotIn(unit, words)
                    near = [w for w in words if len(w) == len(unit) and sum(a != b for a, b in zip(unit, w)) == 1]
                    self.assertEqual(len(near), 1, q)
                    self.assertTrue(unit in toks and unit.endswith(ii.ENDING), q)
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
                    self.assertTrue(valid_anywhere(re.sub(r"^(so|what about) ", "", q)), q)
                elif cat in ("junk", "repeat", "almost"):
                    k = kinds[words[unit]]
                    self.assertFalse(fits(q, unit, k.arity, of_kind(tongues, k)), q)
                    anything = lambda x: bool(x) and not set(x.split(" ")) & ii.FRAME_WORDS  # noqa: E731  (a run of words that holds no word of the ways of asking)
                    self.assertTrue(any(template_matches(f.toks, unit, q, [anything] * k.arity) for f in ii.FRAMES[k.arity]), q)
                elif cat == "trailing numeral":
                    k = kinds[words[unit]]
                    t4 = tongues["t4"]
                    cuts = [i for i in range(1, len(toks)) if t4.valid(" ".join(toks[i:])) and fits(" ".join(toks[:i]), unit, k.arity, of_kind(tongues, k))]
                    self.assertTrue(cuts, q)  # an ordinary question and then a numeral of the fourth family
                elif cat == "order":
                    k = kinds[words[unit]]
                    self.assertIn(unit, toks)
                    self.assertFalse(any(toks[0] == f.toks[0] for f in ii.FRAMES[k.arity] if f.toks[0] not in ii.SLOTS), q)
                elif cat == "new start":
                    start = [s for s in ii.NEW_STARTS if q.startswith(" ".join(s) + " ")]
                    self.assertEqual(len(start), 1, q)
                    self.assertFalse({s[0] for s in ii.NEW_STARTS} & {f.toks[0] for fs in ii.FRAMES.values() for f in fs})  # no kind has a way that starts so
                elif cat == "new end":
                    k = kinds[words[unit]]
                    ends = [e for e in ii.NEW_ENDS if q.endswith(" " + " ".join(e))]
                    self.assertEqual(len(ends), 1, q)
                    body = q[: -len(" " + " ".join(ends[0]))]
                    self.assertTrue(fits(body, unit, k.arity, of_kind(tongues, k)), q)  # the rest is an ordinary question
                    self.assertFalse({e[-1] for e in ii.NEW_ENDS} & {f.toks[-1] for fs in ii.FRAMES.values() for f in fs})  # no kind has a way that ends so
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
                    self.assertTrue(any(template_matches(swap, unit, q, [t.valid for t in of_kind(tongues, k)]) for f in ii.FRAMES[k.arity] for swap in self.exchanged(f)), q)
                    self.assertTrue(order_matters(k), q)  # the order of the numbers matters
                elif cat == "mismatched ends":
                    k = kinds[words[unit]]
                    ok = [t.valid for t in of_kind(tongues, k)]
                    self.assertTrue(any(template_matches(x.toks[: x.w + 1] + y.toks[y.w + 1 :], unit, q, (ok * 3)[: 3]) for x in ii.FRAMES[k.arity] for y in ii.FRAMES[3 - k.arity]), q)
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
        a, b = self.write(ii, 3), self.write(ii, 3, compose_first=True)
        la, lb = read_rows(os.path.join(a, "lessons.tsv"), 5), read_rows(os.path.join(b, "lessons.tsv"), 5)
        self.assertNotEqual(la, lb)
        ka, kb = read_rows(os.path.join(a, "key.tsv"), 5), read_rows(os.path.join(b, "key.tsv"), 5)
        roles = [r[1] for r in kb]
        self.assertLess(roles.index("compose"), roles.index("newop"))
        self.assertLess([r[1] for r in ka].index("newop"), [r[1] for r in ka].index("compose"))

    @unittest.skipIf(np is None, "numpy is needed for the reachability search")
    @unittest.skipIf(os.environ.get("HW_I_SKIP_SEARCH"), "HW_I_SKIP_SEARCH is set")
    def test_no_hard_kind_is_reachable_and_the_controls_are(self):
        seeds = range(1, int(os.environ.get("HW_I_SEARCH_SEEDS", "6")) + 1)
        groups, roles = {}, {}
        for seed in seeds:
            for group in ii.make_kinds(seed):
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


# ---- the reachability search: every expression of at most 7 nodes (leaves count) over + - * min max div mod, the variables and the constants 0 to 40, 50, 64 and 100 ----

SMALL = list(range(41)) + [50, 64, 100]
OPS = ("add", "sub", "mul", "min", "max", "div", "mod")
BUCKETS = {1: (31, 63, 99, 199, 399), 2: (15, 31, 60, 99, 199, 399), 3: (15, 24, 31)}


def check_points(k):
    """The sample a kind is matched on, and its name (kinds with the same sample are searched together): the numbers a kind stands for, not their words."""
    rng = first.Rng(777)
    top = max([b for b in BUCKETS[k.arity] if b <= k.hi] or [k.hi])
    if k.arity == 1:
        if top <= 99:
            return f"unary 1..{top}", [(a,) for a in range(1, top + 1)]
        return f"unary wide {top}", [(a,) for a in list(range(1, 25)) + list(range(30, 200, 17)) + [x for x in range(200, 395, 59) if x <= top]]
    if k.arity == 2:
        small = min(24, top)
        return f"binary {top}", [(1 + rng.below(small), 1 + rng.below(small)) for _ in range(30)] + [(1 + rng.below(top), 1 + rng.below(top)) for _ in range(30)]
    return f"ternary {top}", [(1 + rng.below(top), 1 + rng.below(top), 1 + rng.below(top)) for _ in range(40)]


def apply(op, x, y, saturating):
    if op == "add":
        return x + y
    if op == "sub":
        r = x - y
        return np.maximum(r, 0) if saturating else r
    if op == "mul":
        return x * y
    if op == "min":
        return np.minimum(x, y)
    if op == "max":
        return np.maximum(x, y)
    safe = np.where(y == 0, 1, y)  # a division or a remainder by zero is 0
    return np.where(y == 0, 0, np.floor_divide(x, safe) if op == "div" else np.mod(x, safe))


class Hasher:
    def __init__(self, width):
        rs = np.random.RandomState(12345)
        self.w = rs.randint(1, 2**62, size=width, dtype=np.int64).astype(np.uint64) | np.uint64(1)

    def __call__(self, rows):
        return (rows.astype(np.int64).view(np.uint64) * self.w[: rows.shape[-1]]).sum(axis=-1, dtype=np.uint64)


def search(points, targets, saturating, prefix=10):
    """{name: nodes} for the targets (name -> values on the points) that an expression of at most 7 nodes reaches. An exact match on the sample counts as reached, so the check
    errs toward calling a kind reachable. Every function of 1, 3 and 5 nodes is built (deduplicated by value on the sample), then every way of joining two of 1 + 5, 5 + 1
    and 3 + 3 nodes is tried on the first points of the sample and checked on all of them where those agree with a target."""
    width, nvars = len(points), len(points[0])
    k = min(prefix, width)
    full, head = Hasher(width), Hasher(k)
    leaves = np.stack([np.array([p[i] for p in points], dtype=np.int64) for i in range(nvars)] + [np.full(width, c, dtype=np.int64) for c in SMALL])
    by_hash = {}
    for name, values in targets.items():
        by_hash.setdefault(int(full(np.array(values, dtype=np.int64))), []).append(name)
    heads = np.array(sorted({int(head(np.array(v, dtype=np.int64)[:k])) for v in targets.values()}), dtype=np.uint64)
    found, seen = {}, set()

    def note(rows, size):
        for h in np.unique(full(rows)):
            for name in by_hash.get(int(h), ()):
                found.setdefault(name, size)

    def fresh(rows):
        h = full(rows)
        _, index = np.unique(h, return_index=True)
        keep = [i for i in index if int(h[i]) not in seen]
        seen.update(int(h[i]) for i in keep)
        return rows[keep]

    s1 = fresh(leaves)
    note(s1, 1)
    s3 = np.concatenate([fresh(apply(op, x[None, :], s1, saturating)) for op in OPS for x in s1])
    note(s3, 3)
    parts = [fresh(apply(op, x[None, :], s3, saturating)) for op in OPS for x in s1]
    parts += [fresh(apply(op, s3, y[None, :], saturating)) for op in OPS for y in s1]
    s5 = np.concatenate(parts)
    note(s5, 5)

    def join(xs, ys):
        for op in OPS:
            step = max(1, 2_000_000 // (len(ys) * k))
            for i in range(0, len(xs), step):
                chunk = xs[i : i + step]
                hit = np.isin(head(apply(op, chunk[:, None, :k], ys[None, :, :k], saturating)), heads)
                for a, b in zip(*np.nonzero(hit)):
                    for name in by_hash.get(int(full(apply(op, chunk[a][None, :], ys[b][None, :], saturating))[0]), ()):
                        found.setdefault(name, 7)

    join(s1, s5)
    join(s5, s1)
    join(s3, s3)
    return found


if __name__ == "__main__":
    unittest.main()
