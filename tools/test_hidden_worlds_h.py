# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""Tests for tools/hidden_worlds_h.py: it is deterministic, writes the format `baby exam adapt-write` writes, the key never reaches the exam files, every answer
follows from its rule, every numeral reads back to its number (checked against a second reading of each grammar), the ways of asking are what the key says, every trap
row is what its category says, and (with numpy) no hard kind is reachable by a program of at most 7 nodes while the controls are.

    python3 -m unittest discover -s tools -p 'test_hidden_worlds_h.py'

The reachability test (seeds 1 to 6 by default, `HW_H_SEARCH_SEEDS=N` for 1 to N; the time is mostly the search itself) is skipped without numpy or with `HW_H_SKIP_SEARCH=1`.
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
import hidden_worlds_h as hh

try:
    import numpy as np
except ImportError:  # the reachability search is the only part that needs it
    np = None

EARLIER = (first, second, third, fourth, fifth, sixth, seventh)
KEY_COLUMNS = ((first, 4), (second, 4), (third, 4), (fourth, 5), (fifth, 5), (sixth, 5), (seventh, 5))
ROLES = {"newop", "compose", "control", "unexplainable", "trap"}
FAMILIES = ("d", "t1", "t2", "t3", "m1")
MAX_WORDS = 22  # the longest numeral of any grammar has 19 words


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


def earlier_table(k):
    if k.arity == 1:
        return tuple(k.fn(a, 0) for a in range(k.lo, 32))
    return tuple(k.fn(a, b) for a in range(k.lo, 32) for b in range(k.lo, 32) if not k.ordered or a >= b)


def template_matches(toks, word, question, ok):
    """Every way the question can be the template (a list of tokens) with this word and numerals for which `ok` holds, each as the list of numerals."""
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
        elif t in hh.SLOTS:
            for n in range(1, MAX_WORDS + 1):
                text = " ".join(q[j : j + n])
                if j + n <= len(q) and ok(text):
                    yield from walk(i + 1, j + n, args + [text])
        elif j < len(q) and q[j] == t:
            yield from walk(i + 1, j + 1, args)

    return list(walk(0, 0, []))


def every_template(arity):
    """{text: (kind, tokens, frame ids)}: the ways of asking, then every one with a filler left out, then every mix of the start of one and the end of another."""
    frames = hh.FRAMES[arity]
    out = {f.text: ("frame", f.toks, (i,)) for i, f in enumerate(frames)}
    for i, f in enumerate(frames):
        for d in f.drop:
            toks = f.toks[:d] + f.toks[d + 1 :]
            if " ".join(toks) not in out:
                out[" ".join(toks)] = ("drop", toks, (i,))
    for i, x in enumerate(frames):
        for j, y in enumerate(frames):
            toks = hh.hybrid_tokens(x, y) if i != j else None
            if toks and " ".join(toks) not in out:
                out[" ".join(toks)] = ("hybrid", toks, (i, j))
    return out


TEMPLATES = {ar: every_template(ar) for ar in (1, 2, 3)}


def fits(question, word, arity, tongue):
    """[(kind of way, ids, numerals)] for each way of asking (and numerals the tongue reads) that gives the question."""
    out = []
    for text, (kind, toks, ids) in TEMPLATES[arity].items():
        for args in template_matches(toks, word, question, tongue.valid):
            out.append((kind, ids, args))
    return out


def two_kinds_form(question, words):
    """Whether the question puts two taught kind words in one question (side by side, or two questions joined)."""
    toks = question.split(" ")
    hits = [t for t in toks if t in words]
    return len(hits) >= 2


class HiddenWorldsH(unittest.TestCase):
    def write(self, module, seed, **kw):
        d = tempfile.mkdtemp()
        self.addCleanup(lambda: __import__("shutil").rmtree(d, ignore_errors=True))
        module.write(d, seed, **kw)
        return d

    def kinds(self, seed):
        _, tongues = hh.world_tongues(seed)
        return {k.name: k for group in hh.make_kinds(seed, tongues) for k in group}

    def exam(self, seed):
        d = self.write(hh, seed)
        _, tongues = hh.world_tongues(seed)
        key = read_rows(os.path.join(d, "key.tsv"), 5)
        return d, tongues, {r[0]: r[2] for r in key}, self.kinds(seed)

    def test_same_seed_same_files_other_seed_other_files(self):
        a, b, c = self.write(hh, 1), self.write(hh, 1), self.write(hh, 2)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv", "README.txt"):
            self.assertEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(b, name)), name)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv"):
            self.assertNotEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(c, name)), name)

    def test_the_files_parse_in_the_existing_format(self):
        for seed in (1, 2, 3):
            d = self.write(hh, seed)
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
            d = self.write(hh, seed)
            key = read_rows(os.path.join(d, "key.tsv"), 5)
            exam = "".join(read_bytes(os.path.join(d, name)).decode("utf-8") for name in ("lessons.tsv", "stream.tsv", "README.txt"))
            for word, role, name, rule, learn in key:
                self.assertTrue(rule and learn)
                self.assertNotIn(name, exam, name)
                self.assertNotIn(rule, exam, name)
                self.assertNotIn(learn, exam, name)
            # what the exam files hold is the fixed words of the questions, the kinds' words, the numerals, the numbers, and (in trap rows only) new words, words one letter
            # away from a kind word or a number word
            _, tongues = hh.world_tongues(seed)
            numeral_words = {w for t in ("t1", "t2", "t3") for w in tongues[t].words}
            kind_words = {r[0] for r in key}
            allowed = set(hh.FRAME_WORDS) | kind_words | numeral_words
            lessons = read_bytes(os.path.join(d, "lessons.tsv")).decode("utf-8")
            stream = read_bytes(os.path.join(d, "stream.tsv")).decode("utf-8")
            strip = lambda body: set(re.findall(r"[a-z]+", re.sub(r"\bL\d{5}\b|\bS\d{5}\b", "", body))) - ROLES  # noqa: E731
            self.assertFalse(strip(lessons) - allowed)
            fresh = strip(stream) - allowed
            near = lambda w, pool: any(len(w) == len(x) and sum(a != b for a, b in zip(w, x)) == 1 for x in pool)  # noqa: E731
            near_numeral = {w for w in fresh if near(w, numeral_words)}
            near_kind = {w for w in fresh - near_numeral if near(w, kind_words)}
            new = fresh - near_numeral - near_kind
            self.assertEqual(len(new), 9, new)  # the eight new words and the label of the bare numbers
            self.assertTrue(all(w.endswith(hh.ENDING) for w in new | near_kind))
            self.assertEqual(len(near_kind), 4)
            self.assertEqual(len(near_numeral), 6)
            self.assertEqual(sorted(os.listdir(d)), ["README.txt", "key.tsv", "lessons.tsv", "stream.tsv"])

    def test_the_kinds_by_role(self):
        for seed in (1, 2, 3):
            d = self.write(hh, seed)
            counts = {}
            for r in read_rows(os.path.join(d, "key.tsv"), 5):
                counts[r[1]] = counts.get(r[1], 0) + 1
            self.assertEqual(counts, {"control": 15, "newop": 10, "compose": 2, "unexplainable": 5})
            per_role = {}
            for r in read_rows(os.path.join(d, "stream.tsv"), 5):
                per_role[r[1]] = per_role.get(r[1], 0) + 1
            self.assertEqual(set(per_role), ROLES)
            self.assertEqual(per_role["trap"], 108)
            # the families of numerals: how many kinds take or give words of each family, and how many take numbers in any other form than digits
            kinds = self.kinds(seed).values()
            self.assertEqual({fam: sum(1 for k in kinds if fam in (k.tin, k.tout)) for fam in ("t1", "t2", "t3", "m1")}, {"t1": 7, "t2": 5, "t3": 5, "m1": 2})
            self.assertEqual(sum(1 for k in kinds if k.tin != "d" or k.tout != "d"), 19)
            self.assertEqual(sorted(k.name for k in kinds if k.arity == 3), ["avg3", "gcd3", "median", "muldiv3", "trimod"])
            self.assertEqual(sorted(k.name for k in kinds if k.bridge), ["avg3", "lopsided", "modquot"])

    def test_the_words_are_this_sets_own(self):
        mine, invented = set(), set()
        for seed in range(1, 7):
            d = self.write(hh, seed)
            words = {r[0] for r in read_rows(os.path.join(d, "key.tsv"), 5)}
            self.assertEqual(len(words), 32)
            self.assertTrue(all(w.endswith("dopa") for w in words))
            mine |= words
            _, tongues = hh.world_tongues(seed)
            invented |= words | {w for t in ("t1", "t2", "t3") for w in tongues[t].words}
            stream = read_bytes(os.path.join(d, "stream.tsv")).decode("utf-8")
            invented |= set(re.findall(r"[a-z]+", re.sub(r"\bS\d{5}\b", "", stream))) - hh.FRAME_WORDS - ROLES
        for module, columns in KEY_COLUMNS:
            for other in range(1, 7):
                d = self.write(module, other)
                theirs = {r[0] for r in read_rows(os.path.join(d, "key.tsv"), columns)}
                self.assertFalse(mine & theirs, (module.__name__, other))
                # no invented word of this set (kind words, number words, the words that only trap rows carry) is any word at all of an earlier exam
                text = "".join(read_bytes(os.path.join(d, name)).decode("utf-8") for name in ("lessons.tsv", "stream.tsv"))
                self.assertFalse(invented & set(re.findall(r"[a-z]+", text)), (module.__name__, other))

    def test_no_kind_is_one_of_the_earlier_sets(self):
        # by name, and by table: none of this set's kinds equals any earlier kind (the controls and the arbitrary codes included) on its whole domain; the kinds that read and
        # write words, add words and double words are the identity, the sum and the double on a domain of their own, as the spelling controls of earlier sets are on a smaller one
        earlier = [k for module in EARLIER for seed in range(1, 6) for group in module.make_kinds(seed) for k in group]
        names = {k.name for k in earlier}
        tables = {(k.arity, k.lo, 31, earlier_table(k)) for k in earlier}
        for seed in range(1, 6):
            for group in hh.make_kinds(seed):
                for k in group:
                    self.assertNotIn(k.name, names)
                    self.assertFalse((k.arity, k.lo, k.hi, table(k)) in tables, (seed, k.name))
                    if k.arity < 3 and k.hi >= 31 and k.name not in ("unspell1", "spell1", "plus1", "plus2", "mixsum", "twice2", "times1"):  # on the first 32 numbers too
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
                    found = fits(q, unit, k.arity, tongues[k.tin])
                    self.assertEqual(len(found), 1, (q, found))
                    args = [tongues[k.tin].parse(x) for x in found[0][2]]
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
                (kind, ids, _), = fits(r[1], r[3], k.arity, tongues[k.tin])
                self.assertEqual(kind, "frame")
                taught.setdefault(r[3], set()).add(ids[0])
            self.assertEqual(len(taught), 32)
            for unit, ids in taught.items():
                k = kinds[key[unit]]
                self.assertIn(len(ids), ((len(hh.FRAMES[k.arity]),) if k.bridge else (2, 3)), k.name)
            used_modes = {}
            for r in read_rows(os.path.join(d, "stream.tsv"), 5):
                if r[1] == "trap":
                    continue
                k = kinds[key[r[2]]]
                (kind, ids, _), = fits(r[3], r[2], k.arity, tongues[k.tin])
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

    def test_digits_and_words_mix_inside_a_kind(self):
        for seed in (1, 2, 3):
            d, tongues, key, kinds = self.exam(seed)
            for name in ("lessons.tsv", "stream.tsv"):
                for kind in ("fourthmod", "mixsum"):
                    unit = [u for u, n in key.items() if n == kind][0]
                    rows = [(r[1] if name == "lessons.tsv" else r[3]) for r in read_rows(os.path.join(d, name), 5) if (r[3] if name == "lessons.tsv" else r[2]) == unit and r[1] != "trap"]
                    forms = set()
                    for q in rows:
                        (_, _, args), = fits(q, unit, kinds[kind].arity, tongues["m1"])
                        forms |= {"digits" if x.isdigit() else "words" for x in args}
                    self.assertEqual(forms, {"digits", "words"}, (name, kind))
            # the kinds that read and write words keep their forms apart: words in, digits out; digits in, words out
            for r in read_rows(os.path.join(d, "lessons.tsv"), 5):
                if key[r[3]] == "unspell1":
                    self.assertTrue(r[2].isdigit() and not any(x.isdigit() for x in r[1].split(" ")), r)
                if key[r[3]] == "spell1":
                    self.assertFalse(r[2].isdigit(), r)

    def test_the_numerals(self):
        grammars = {}
        for seed in range(1, 13):
            words, tongues = hh.world_tongues(seed)
            names = [type(tongues[t]).__name__ for t in ("t1", "t2", "t3")]
            self.assertEqual(len(set(names)), 3, seed)
            for n in names:
                grammars.setdefault(n, []).append(seed)
            seen_words = set()
            for t in ("t1", "t2", "t3"):
                tongue = tongues[t]
                self.assertEqual(len(set(tongue.words)), len(tongue.words))
                self.assertFalse(seen_words & set(tongue.words))
                seen_words |= set(tongue.words)
                self.assertFalse(set(tongue.words) & set(hh.FRAME_WORDS))
                self.assertTrue(all(len(w) == 4 for w in tongue.words))
                self.assertGreaterEqual(tongue.cap, 399)
                said = [tongue.say(n) for n in range(tongue.cap + 1)]
                self.assertEqual(len(set(said)), tongue.cap + 1, (seed, t))
                for n, text in enumerate(said):
                    self.assertEqual(tongue.parse(text), n, (seed, t, n))
                    self.assertLessEqual(len(text.split(" ")), MAX_WORDS - 3)
                    self.assertTrue(set(text.split(" ")) <= set(tongue.words))
            # the three grammars are written with words of their own
            self.assertTrue(all(set(tongues[t].foreign) == seen_words - set(tongues[t].words) for t in ("t1", "t2", "t3")))
            self.assertTrue(tongues["m1"].valid("17") and tongues["m1"].valid(tongues["t1"].say(17)) and not tongues["m1"].valid(tongues["t2"].say(17)))
        self.assertEqual(sorted(grammars), ["Balanced", "Bijective", "Place", "Roman", "Score", "Tally"])
        self.assertTrue(all(len(v) >= 2 for v in grammars.values()), grammars)

    def test_each_grammar_does_what_it_says(self):
        """A second reading of each grammar, from its words and the rule in the docstring, against `say` for every number."""
        for seed in range(1, 13):
            _, tongues = hh.world_tongues(seed)
            for t in ("t1", "t2", "t3"):
                tongue = tongues[t]
                for n in range(tongue.cap + 1):
                    toks = tongue.say(n).split(" ")
                    if isinstance(tongue, hh.Bijective):
                        value = {w: i for i, w in tongue.digit.items()}
                        got = 0 if toks == [tongue.zero] else sum(value[w] * tongue.base**i for i, w in enumerate(reversed(toks)))
                        self.assertNotIn(tongue.zero, toks if n else [])
                        self.assertTrue(n == 0 or all(1 <= value[w] <= tongue.base for w in toks))
                    elif isinstance(tongue, hh.Balanced):
                        value = {w: d for d, w in tongue.digit.items()}
                        got = sum(value[w] * tongue.base**i for i, w in enumerate(reversed(toks)))
                        self.assertTrue(n == 0 or value[toks[0]] > 0)  # no zero in front, and the first digit is never the minus one
                    elif isinstance(tongue, hh.Tally):
                        value = {w: p for p, w in tongue.place.items()}
                        got = 0 if toks == [tongue.zero] else sum(value[w] for w in toks)
                        self.assertEqual(toks, sorted(toks, key=lambda w: -value.get(w, 0)))  # largest place first
                        self.assertTrue(n == 0 or all(toks.count(w) < tongue.base for w in set(toks) if value[w] != tongue.places[0]))
                    elif isinstance(tongue, hh.Roman):
                        value = {w: v for v, w in tongue.sym.items()}
                        got = 0 if toks == [tongue.zero] else sum(-value[w] if i + 1 < len(toks) and value[toks[i + 1]] > value[w] else value[w] for i, w in enumerate(toks))
                    elif isinstance(tongue, hh.Place):
                        where = {w: (p, d) for p in range(3) for d, w in enumerate(tongue.digit[p])}
                        got = sum(d * 10**where[w][0] for w in toks for d in (where[w][1],))
                        self.assertEqual([where[w][0] for w in toks], sorted((where[w][0] for w in toks), reverse=not tongue.ascending))
                        self.assertEqual(len({where[w][0] for w in toks}), len(toks))  # every word is a digit of a place of its own
                    else:
                        value = {w: i for i, w in tongue.digit.items()}
                        got, top, rest = 0, [], []
                        if toks == [tongue.zero]:
                            pass
                        else:
                            self.assertEqual(toks.count(tongue.link) <= 1, True)
                            if tongue.link in toks:
                                i = toks.index(tongue.link)
                                a, b = toks[:i], toks[i + 1 :]
                                lead, tail = (b, a) if tongue.first else (a, b)  # lead holds the twenties, tail the leftover
                            elif tongue.twenty in toks:
                                lead, tail = toks, []
                            else:
                                lead, tail = [], toks
                            small = lambda ws: sum(10 if w == tongue.ten else value[w] for w in ws)  # noqa: E731
                            if lead:
                                self.assertEqual(lead[-1], tongue.twenty)
                                got += 20 * (small(lead[:-1]) or 1)
                            got += small(tail)
                    self.assertEqual(got, n, (seed, t, type(tongue).__name__, n, toks))
                # a few spot checks of the shapes named in the documentation
                if isinstance(tongue, hh.Bijective):
                    b = tongue.base
                    self.assertEqual(tongue.say(b), tongue.digit[b])  # the largest digit stands for b, so b + 1 needs two digits
                    self.assertEqual(tongue.say(b + 1), f"{tongue.digit[1]} {tongue.digit[1]}")
                    self.assertEqual(tongue.say(b * b + b), f"{tongue.digit[b]} {tongue.digit[b]}")
                elif isinstance(tongue, hh.Balanced):
                    self.assertEqual(tongue.say(tongue.base - 1), f"{tongue.digit[1]} {tongue.digit[-1]}")
                elif isinstance(tongue, hh.Tally):
                    self.assertEqual(tongue.say(tongue.base), tongue.place[tongue.base])
                    self.assertEqual(tongue.say(tongue.base - 1), " ".join([tongue.place[1]] * (tongue.base - 1)))
                elif isinstance(tongue, hh.Roman):
                    self.assertEqual(tongue.say(4), f"{tongue.sym[1]} {tongue.sym[5]}")
                    self.assertEqual(tongue.say(9), f"{tongue.sym[1]} {tongue.sym[10]}")
                    self.assertEqual(tongue.say(40), f"{tongue.sym[10]} {tongue.sym[50]}" if tongue.strict else " ".join([tongue.sym[10]] * 4))
                elif isinstance(tongue, hh.Place):
                    self.assertEqual(len(tongue.words), 29)
                    self.assertEqual(tongue.say(305), " ".join([tongue.digit[2][3], tongue.digit[1][0], tongue.digit[0][5]][:: 1 if not tongue.ascending else -1]))
                else:
                    self.assertEqual(tongue.say(20), tongue.twenty)
                    self.assertEqual(tongue.say(80), f"{tongue.digit[4]} {tongue.twenty}")
                    self.assertEqual(tongue.say(90), (f"{tongue.ten} {tongue.link} {tongue.digit[4]} {tongue.twenty}" if tongue.first else f"{tongue.digit[4]} {tongue.twenty} {tongue.link} {tongue.ten}"))

    def test_what_is_no_numeral_is_none(self):
        for seed in range(1, 13):
            words, tongues = hh.world_tongues(seed)
            rng = hh.seeded(seed, 11)
            everywhere = [tongues[t] for t in FAMILIES]
            for t in ("t1", "t2", "t3"):
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
                near = words.almost(word)
                self.assertFalse(near in tongue.words or any(x.valid(near) for x in everywhere))
                self.assertEqual(sum(a != b for a, b in zip(word, near)), 1)
            self.assertTrue(tongues["d"].valid("17") and not tongues["d"].valid("seventeen"))

    def test_the_words_of_held_out_numerals_occur_in_lessons(self):
        # the kinds that read and write the first family, and those that add and double in the second, show every word of it (the zero word included), and every
        # word of every held-out and beyond row of the first two families is among them
        for seed in range(1, 9):
            lessons, stream, key = hh.build(seed)
            _, tongues = hh.world_tongues(seed)
            kind = {r[0]: r[2] for r in key}
            for fam, names in (("t1", {"unspell1", "spell1"}), ("t2", {"plus2", "twice2"})):
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
                tongue = tongues[k.tin]
                rows = [r for r in stream if r[2] == unit]
                args = [[tongue.parse(x) for x in fits(r[3], unit, k.arity, tongue)[0][2]] for r in rows]
                if k.arity >= 2:  # a number named twice in a question
                    self.assertGreaterEqual(sum(1 for a in args if len(set(a)) < len(a)), 3 if k.arity == 2 else 2, name)
                if k.arity == 1 and k.tin not in ("d", "m1") and k.hi > 99:  # a numeral with a word that occurs twice
                    self.assertGreaterEqual(sum(1 for a in args if len(set(tongue.say(a[0]).split(" "))) < len(tongue.say(a[0]).split(" "))), 4 if not isinstance(tongue, hh.Place) else 0, name)

    def test_the_operations_against_known_values(self):
        self.assertEqual([hh.van_eck(n) for n in range(1, 31)], [0, 0, 1, 0, 2, 0, 2, 2, 1, 6, 0, 5, 0, 2, 6, 5, 4, 0, 5, 3, 0, 3, 2, 9, 0, 4, 9, 3, 6, 14])
        self.assertEqual([hh.bit_double(n) for n in (0, 1, 2, 3, 5, 6, 31)], [0, 3, 12, 15, 51, 60, 1023])
        self.assertEqual([hh.kaprekar_steps(n) for n in (0, 495, 111, 999, 100, 1, 6, 352, 549)], [0, 0, 1, 1, 6, 6, 2, 4, 1])
        self.assertEqual(max(hh.kaprekar_steps(n) for n in range(1000)), 6)
        self.assertEqual([hh.ant_black(n) for n in range(0, 12)], [0, 1, 2, 3, 4, 3, 4, 5, 6, 7, 6, 7])
        self.assertEqual([n for n in range(0, 30) if hh.rising(n)], list(range(10)) + [12, 13, 14, 15, 16, 17, 18, 19, 23, 24, 25, 26, 27, 28, 29])

    def test_the_rules_against_second_implementations(self):
        # van Eck's sequence: the gap to the last time the term occurred, by searching backwards in the list
        seq = [0]
        for _ in range(400):
            last = seq[-1]
            gap = 0
            for back in range(len(seq) - 2, -1, -1):
                if seq[back] == last:
                    gap = len(seq) - 1 - back
                    break
            seq.append(gap)
        self.assertEqual([hh.van_eck(n) for n in range(1, 401)], seq[:400])
        # the ant: headings as numbers 0 to 3, the grid as a dictionary
        for steps in range(0, 120):
            grid, x, y, heading = {}, 0, 0, 0
            moves = ((0, -1), (1, 0), (0, 1), (-1, 0))
            for _ in range(steps):
                black = grid.get((x, y), 0)
                heading = (heading + (3 if black else 1)) % 4
                grid[(x, y)] = 1 - black
                x, y = x + moves[heading][0], y + moves[heading][1]
            self.assertEqual(hh.ant_black(steps), sum(grid.values()), steps)
        # the three-digit step, on lists of digits; every number ends in 495 or in 0
        for n in range(1000):
            steps, cur = 0, n
            while True:
                ds = sorted(int(c) for c in "%03d" % cur)
                nxt = (ds[2] * 100 + ds[1] * 10 + ds[0]) - (ds[0] * 100 + ds[1] * 10 + ds[2])
                if nxt == cur:
                    break
                cur, steps = nxt, steps + 1
            self.assertIn(cur, (0, 495))
            self.assertEqual(hh.kaprekar_steps(n), steps)
            self.assertEqual(cur == 0, len(set("%03d" % n)) == 1)
        # doubling the bits, by building the number a bit at a time
        for a in range(0, 200):
            out = 0
            for c in bin(a)[2:]:
                out = out * 4 + int(c) * 3
            self.assertEqual(hh.bit_double(a), out)
        # a rising run of digits is a number whose digit string is sorted and has no repeat
        for a in range(0, 1000):
            self.assertEqual(hh.rising(a), len(set(str(a))) == len(str(a)) and "".join(sorted(str(a))) == str(a))

    def test_every_kind_has_enough_values_and_the_constants_are_in_range(self):
        for seed in range(1, 31):
            kinds = self.kinds(seed)
            _, tongues = hh.world_tongues(seed)
            for k in kinds.values():
                self.assertGreaterEqual(len(set(table(k)) if k.arity < 3 else {k.fn(*p) for p in itertools.product(range(k.lo, k.hi + 1), repeat=3)}), 3 if k.name in ("distinct3", "scatterpair2", "firstword3", "flip3") else 5, (seed, k.name))
                self.assertLessEqual(k.hi, 999)
                outputs = set(table(k))
                if k.tout in ("t1", "t2", "t3"):
                    self.assertLessEqual(max(outputs), tongues[k.tout].cap, (seed, k.name))
                    self.assertGreaterEqual(min(outputs), 0)
                if k.tin in ("t1", "t2", "t3", "m1"):
                    self.assertLessEqual(k.hi, tongues[k.tin].cap, (seed, k.name))
                if k.tout == "d":
                    self.assertTrue(all(isinstance(x, int) and x >= 0 for x in outputs), (seed, k.name))
            m = int(re.search(r"mod (\d+)", kinds["fourthmod"].rule).group(1))
            self.assertTrue(m in hh.PRIMES and m >= 59)
            m = int(re.search(r"mod (\d+)", kinds["trimod"].rule).group(1))
            self.assertTrue(m in hh.PRIMES and m >= 59)
            self.assertIn(int(re.search(r"mod (\d+)", kinds["cubemod"].rule).group(1)), hh.SMALL_MODS)
            k = re.search(r"\(\(a \* b \+ (\d+)\) div (\d+)\) mod (\d+)", kinds["prodquotmod"].rule)
            self.assertTrue(100 <= int(k.group(1)) < 400 and 7 <= int(k.group(2)) < 32 and 17 <= int(k.group(3)) < 54)
            self.assertEqual(kinds["times1"].hi, tongues["t1"].cap // int(re.match(r"(\d+) \*", kinds["times1"].rule).group(1)))
            # the ternary kinds keep their lessons inside their domains
            self.assertEqual(kinds["avg3"].hi, 15)
        for p in hh.PRIMES:
            self.assertTrue(all(p % q for q in range(2, int(math.sqrt(p)) + 1)))
        # the fourth power wraps, the median is one of its three numbers, the gcd of three is below each, the branch takes both sides
        kinds = self.kinds(1)
        self.assertGreater(max(kinds["fourthmod"].fn(a) for a in range(64)), 40)
        self.assertTrue(all(kinds["median"].fn(a, b, c) in (a, b, c) and kinds["median"].fn(a, b, c) == sorted((a, b, c))[1] for a in range(0, 32, 3) for b in range(0, 32, 5) for c in range(0, 32, 7)))
        self.assertTrue(all(kinds["gcd3"].fn(a, b, c) <= min(a, b, c) and all(x % kinds["gcd3"].fn(a, b, c) == 0 for x in (a, b, c)) for a in range(1, 25) for b in range(1, 25) for c in range(1, 25)))
        self.assertEqual({hh.rising(a) for a in range(100)}, {True, False})
        self.assertEqual([hh.digit_sum(a) for a in (0, 9, 10, 99, 100, 989)], [0, 9, 1, 18, 1, 26])
        for seed in range(1, 31):  # both branches of the conditional occur, and the second is a remainder (it takes at least 6 values on the numbers that do not rise)
            k = self.kinds(seed)["ascbranch"]
            self.assertEqual({hh.rising(a) for a in range(100)}, {True, False})
            self.assertGreaterEqual(len({k.fn(a) for a in range(100) if not hh.rising(a)}), 6)

    def test_the_arbitrary_codes_are_not_a_pattern(self):
        for seed in (1, 2, 3):
            kinds = self.kinds(seed)
            cells = [kinds["scatterpair2"].fn(a, b) for a in range(16) for b in range(16)]
            self.assertGreaterEqual(len(set(cells)), 25)
            self.assertLess(max(cells), 300)
            rows = {tuple(kinds["scatterpair2"].fn(a, b) for b in range(16)) for a in range(16)}
            self.assertEqual(len(rows), 16)
            self.assertLess(max(kinds["scatterdig"].fn(a) for a in range(32)), 17)
            self.assertLess(max(kinds["scatter1"].fn(a) for a in range(32)), 300)
            self.assertLess(max(kinds["scatter3"].fn(a) for a in range(32)), 200)
            # the remainder that is not one: a mod 7 for most numbers, something else for at least eight of them
            codes = [kinds["nearmod"].fn(a) for a in range(32)]
            self.assertGreaterEqual(sum(1 for a in range(32) if codes[a] != a % 7), 8)
            self.assertGreaterEqual(sum(1 for a in range(32) if codes[a] == a % 7), 14)
            self.assertTrue(all(kinds[n].role == "unexplainable" for n in ("scatterdig", "nearmod", "scatter1", "scatter3", "scatterpair2")))

    def test_the_trap_rows(self):
        expected = {"two kinds": 8, "new word": 8, "almost a kind word": 4, "arity": 8, "other form": 12, "bare": 6, "junk": 6, "repeat": 6, "almost": 6, "order": 6,
                    "new start": 6, "new end": 6, "other core": 6, "word inside": 6, "markers exchanged": 6, "mismatched ends": 4, "left out": 4}
        self.assertEqual(sum(expected.values()), 108)
        for seed in (1, 2, 3):
            lessons, stream, key, categories = hh.build_full(seed)
            _, tongues = hh.world_tongues(seed)
            words = {r[0]: r[2] for r in key}
            kinds = self.kinds(seed)
            taught_questions = {r[1] for r in lessons}
            traps = [r for r in stream if r[1] == "trap"]
            self.assertEqual(len(traps), 108)
            self.assertEqual(len({r[3] for r in traps}), 108)
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
                    self.assertFalse(any(fits(q, unit, ar, tongues[k.tin]) for ar in (k.arity,)), q)
                if cat == "two kinds":
                    self.assertGreaterEqual(len(word_of(q)), 2, q)
                elif cat == "new word":
                    self.assertNotIn(unit, words)
                    self.assertTrue(unit in toks and unit.endswith(hh.ENDING), q)
                elif cat == "almost a kind word":
                    self.assertNotIn(unit, words)
                    near = [w for w in words if len(w) == len(unit) and sum(a != b for a, b in zip(unit, w)) == 1]
                    self.assertEqual(len(near), 1, q)
                    self.assertTrue(unit in toks and unit.endswith(hh.ENDING), q)
                elif cat == "arity":
                    k = kinds[words[unit]]
                    other = [ar for ar in (1, 2, 3) if ar != k.arity and fits(q, unit, ar, tongues[k.tin])]
                    self.assertEqual(len(other), 1, q)
                elif cat == "other form":
                    k = kinds[words[unit]]
                    other = tongues["t3"] if k.tin != "t3" else tongues["d"]
                    self.assertTrue(fits(q, unit, k.arity, other), q)
                    self.assertFalse(fits(q, unit, k.arity, tongues[k.tin]), q)
                elif cat == "bare":
                    self.assertNotIn(unit, words)
                    self.assertIn(unit, [r[2] for r in traps if categories[r[3]] == "bare"])
                    self.assertTrue(valid_anywhere(q.replace("what is ", "").replace("tell me ", "")), q)
                elif cat in ("junk", "repeat", "almost"):
                    k = kinds[words[unit]]
                    self.assertFalse(fits(q, unit, k.arity, tongues["t1" if k.tin == "m1" else k.tin]), q)
                    anything = lambda x: bool(x) and not set(x.split(" ")) & hh.FRAME_WORDS  # noqa: E731  (a run of words that holds no word of the ways of asking)
                    self.assertTrue(any(template_matches(f.toks, unit, q, anything) for f in hh.FRAMES[k.arity]), q)
                elif cat == "order":
                    k = kinds[words[unit]]
                    self.assertIn(unit, toks)
                    self.assertFalse(any(toks[0] == f.toks[0] for f in hh.FRAMES[k.arity] if f.toks[0] not in hh.SLOTS), q)
                elif cat == "new start":
                    start = [s for s in hh.NEW_STARTS if q.startswith(" ".join(s) + " ")]
                    self.assertEqual(len(start), 1, q)
                    self.assertFalse({s[0] for s in hh.NEW_STARTS} & {f.toks[0] for fs in hh.FRAMES.values() for f in fs})  # no kind has a way that starts so
                elif cat == "new end":
                    k = kinds[words[unit]]
                    ends = [e for e in hh.NEW_ENDS if q.endswith(" " + " ".join(e))]
                    self.assertEqual(len(ends), 1, q)
                    body = q[: -len(" " + " ".join(ends[0]))]
                    self.assertTrue(fits(body, unit, k.arity, tongues[k.tin]), q)  # the rest is an ordinary question
                    self.assertFalse({e[-1] for e in hh.NEW_ENDS} & {f.toks[-1] for fs in hh.FRAMES.values() for f in fs})  # no kind has a way that ends so
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
                    self.assertTrue(any(template_matches(swap, unit, q, tongues[k.tin].valid) for f in hh.FRAMES[k.arity] for swap in self.exchanged(f)), q)
                    self.assertIn(words[unit], ("lopsided", "muldiv3"))  # only kinds where the order of the numbers matters
                elif cat == "mismatched ends":
                    k = kinds[words[unit]]
                    ok = tongues[k.tin].valid
                    self.assertTrue(any(template_matches(x.toks[: x.w + 1] + y.toks[y.w + 1 :], unit, q, ok) for x in hh.FRAMES[k.arity] for y in hh.FRAMES[3 - k.arity]), q)
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
        a, b = self.write(hh, 3), self.write(hh, 3, compose_first=True)
        la, lb = read_rows(os.path.join(a, "lessons.tsv"), 5), read_rows(os.path.join(b, "lessons.tsv"), 5)
        self.assertNotEqual(la, lb)
        ka, kb = read_rows(os.path.join(a, "key.tsv"), 5), read_rows(os.path.join(b, "key.tsv"), 5)
        roles = [r[1] for r in kb]
        self.assertLess(roles.index("compose"), roles.index("newop"))
        self.assertLess([r[1] for r in ka].index("newop"), [r[1] for r in ka].index("compose"))

    @unittest.skipIf(np is None, "numpy is needed for the reachability search")
    @unittest.skipIf(os.environ.get("HW_H_SKIP_SEARCH"), "HW_H_SKIP_SEARCH is set")
    def test_no_hard_kind_is_reachable_and_the_controls_are(self):
        seeds = range(1, int(os.environ.get("HW_H_SEARCH_SEEDS", "6")) + 1)
        groups, roles = {}, {}
        for seed in seeds:
            for group in hh.make_kinds(seed):
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
        self.assertEqual(len(roles), len(seeds) * 29)  # every kind but the three that copy, reverse or count words
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
