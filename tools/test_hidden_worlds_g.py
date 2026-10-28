# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""Tests for tools/hidden_worlds_g.py: it is deterministic, writes the format `baby exam adapt-write` writes, the key never reaches the exam files, every answer
follows from its rule, every numeral reads back to its number, and (with numpy) no hard kind is reachable by a program of at most 7 nodes while the controls are.

    python3 -m unittest discover -s tools -p 'test_hidden_worlds_g.py'

The reachability test (seeds 1 to 10 by default, `HW_G_SEARCH_SEEDS=N` for 1 to N; the time is mostly the search itself, about two and a half minutes) is skipped without numpy or with `HW_G_SKIP_SEARCH=1`.
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
import hidden_worlds_g as hg

try:
    import numpy as np
except ImportError:  # the reachability search is the only part that needs it
    np = None

EARLIER = (first, second, third, fourth, fifth, sixth)
KEY_COLUMNS = ((first, 4), (second, 4), (third, 4), (fourth, 5), (fifth, 5), (sixth, 5))
ROLES = {"newop", "compose", "control", "unexplainable", "trap"}
FAMILIES = ("d", "t1", "t2", "t3")


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
    if k.arity == 1:
        return tuple(k.fn(a, 0) for a in range(k.lo, k.hi + 1))
    return tuple(k.fn(a, b) for a in range(k.lo, k.hi + 1) for b in range(k.lo, k.hi + 1))


def earlier_table(k):
    if k.arity == 1:
        return tuple(k.fn(a, 0) for a in range(k.lo, 32))
    return tuple(k.fn(a, b) for a in range(k.lo, 32) for b in range(k.lo, 32) if not k.ordered or a >= b)


def frame_matches(frame, word, question, ok):
    """Every way the question can be the frame with this word and numerals for which `ok` holds (a numeral is one to eight words), each as the list of numerals."""
    f, q = frame.split(" "), question.split(" ")

    def walk(i, j, args):
        if i == len(f):
            if j == len(q):
                yield list(args)
            return
        if f[i] == "{w}":
            if j < len(q) and q[j] == word:
                yield from walk(i + 1, j + 1, args)
        elif f[i] in ("{a}", "{b}"):
            for n in range(1, 9):
                text = " ".join(q[j : j + n])
                if j + n <= len(q) and ok(text):
                    yield from walk(i + 1, j + n, args + [text])
        elif j < len(q) and q[j] == f[i]:
            yield from walk(i + 1, j + 1, args)

    return list(walk(0, 0, []))


def two_kinds(question, words):
    """Whether the question asks two taught kinds about one numeral."""
    for frame in hg.TWO_KINDS:
        pattern = re.escape(frame).replace(re.escape("{w1}"), r"(\S+)").replace(re.escape("{w2}"), r"(\S+)").replace(re.escape("{a}"), "(.+)")
        m = re.fullmatch(pattern, question)
        if m and m.group(1) in words and m.group(3) in words and m.group(2) == m.group(4):
            return True
    return False


def parse_question(question, word, arity, tongue):
    """The ways of asking (indexes) that fit the question with numerals that the tongue reads, each with the numerals."""
    out = []
    for i, frame in enumerate(hg.frames_of(arity)):
        for args in frame_matches(frame, word, question, tongue.valid):
            out.append((i, args))
    return out


class HiddenWorldsG(unittest.TestCase):
    def write(self, module, seed, **kw):
        d = tempfile.mkdtemp()
        self.addCleanup(lambda: __import__("shutil").rmtree(d, ignore_errors=True))
        module.write(d, seed, **kw)
        return d

    def kinds(self, seed):
        _, tongues = hg.world_tongues(seed)
        return {k.name: k for group in hg.make_kinds(seed, tongues) for k in group}

    def test_same_seed_same_files_other_seed_other_files(self):
        a, b, c = self.write(hg, 1), self.write(hg, 1), self.write(hg, 2)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv", "README.txt"):
            self.assertEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(b, name)), name)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv"):
            self.assertNotEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(c, name)), name)

    def test_the_files_parse_in_the_existing_format(self):
        for seed in (1, 2, 3):
            d = self.write(hg, seed)
            lessons = read_rows(os.path.join(d, "lessons.tsv"), 5)
            stream = read_rows(os.path.join(d, "stream.tsv"), 5)
            self.assertEqual([r[0] for r in lessons], [f"L{i:05d}" for i in range(len(lessons))])
            self.assertEqual([r[0] for r in stream], [f"S{i:05d}" for i in range(len(stream))])
            for r in lessons + stream:
                for field in r:
                    self.assertTrue(field and field == field.strip() and "\t" not in field, r)
            self.assertEqual(len({r[3] for r in lessons}), 33)
            self.assertEqual({r[1] for r in stream}, ROLES)
            # every fact is taught in two, three or (for the two kinds that carry the equivalences) eight ways of asking, all with the same answer
            facts = {}
            for r in lessons:
                facts.setdefault(r[4], []).append(r)
            self.assertEqual({len(rows) for rows in facts.values()}, {2, 3, 8})
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
                if r[1] != "trap":
                    self.assertIn(r[2], r[3].split(" "))

    def test_the_key_never_reaches_the_exam_files(self):
        for seed in (1, 2, 3):
            d = self.write(hg, seed)
            key = read_rows(os.path.join(d, "key.tsv"), 5)
            exam = "".join(read_bytes(os.path.join(d, name)).decode("utf-8") for name in ("lessons.tsv", "stream.tsv", "README.txt"))
            for word, role, name, rule, learn in key:
                self.assertTrue(rule and learn)
                self.assertNotIn(name, exam, name)
                self.assertNotIn(rule, exam, name)
                self.assertNotIn(learn, exam, name)
            # what the exam files hold is the fixed words of the questions, the kinds' words, the numerals, the numbers, and (in trap rows only) new words and
            # words one letter away from a number word
            _, tongues = hg.world_tongues(seed)
            numeral_words = {w for t in ("t1", "t2", "t3") for w in tongues[t].words}
            allowed = set(hg.FRAME_WORDS) | {r[0] for r in key} | numeral_words
            lessons = "".join(read_bytes(os.path.join(d, "lessons.tsv")).decode("utf-8"))
            stream = "".join(read_bytes(os.path.join(d, "stream.tsv")).decode("utf-8"))
            strip = lambda body: set(re.findall(r"[a-z]+", re.sub(r"\bL\d{5}\b|\bS\d{5}\b", "", body))) - ROLES  # noqa: E731
            self.assertFalse(strip(lessons) - allowed)
            fresh = strip(stream) - allowed
            near = {w for w in fresh if any(len(w) == len(x) and sum(a != b for a, b in zip(w, x)) == 1 for x in numeral_words)}
            self.assertEqual(len(fresh - near), 9, fresh)  # the eight new words and the label of the bare numbers
            self.assertTrue(all(w.endswith(hg.ENDING) for w in fresh - near))
            self.assertEqual(len(near), 6)
            self.assertEqual(sorted(os.listdir(d)), ["README.txt", "key.tsv", "lessons.tsv", "stream.tsv"])

    def test_the_kinds_by_role(self):
        d = self.write(hg, 1)
        counts = {}
        for r in read_rows(os.path.join(d, "key.tsv"), 5):
            counts[r[1]] = counts.get(r[1], 0) + 1
        self.assertEqual(counts, {"control": 13, "newop": 12, "compose": 3, "unexplainable": 5})
        per_role = {}
        for r in read_rows(os.path.join(d, "stream.tsv"), 5):
            per_role[r[1]] = per_role.get(r[1], 0) + 1
        self.assertEqual(per_role, {"control": 248, "newop": 204, "compose": 64, "unexplainable": 78, "trap": 63})
        # the families of numerals: how many kinds take or give words of each family
        kinds = self.kinds(1).values()
        self.assertEqual({fam: sum(1 for k in kinds if fam in (k.tin, k.tout)) for fam in ("t1", "t2", "t3")}, {"t1": 8, "t2": 6, "t3": 6})
        self.assertEqual(sum(1 for k in kinds if "d" != k.tin or "d" != k.tout), 20)
        self.assertEqual(sorted(k.name for k in kinds if k.role == "control" and k.tin == "d" and k.tout == "d"), ["halves", "lastprod", "mix3", "triple"])

    def test_the_words_are_this_sets_own(self):
        mine = set()
        for seed in range(1, 7):
            words = {r[0] for r in read_rows(os.path.join(self.write(hg, seed), "key.tsv"), 5)}
            self.assertEqual(len(words), 33)
            self.assertTrue(all(w.endswith("tole") for w in words))
            mine |= words
        for module, columns in KEY_COLUMNS:
            for other in range(1, 7):
                theirs = {r[0] for r in read_rows(os.path.join(self.write(module, other), "key.tsv"), columns)}
                self.assertFalse(mine & theirs, (module.__name__, other))

    def test_no_kind_is_one_of_the_earlier_sets(self):
        # by name, and by table: none of this set's kinds equals any earlier kind (the controls and the arbitrary codes included) on its whole domain; the two
        # kinds that read and write words are the identity on a domain of their own, as the spelling controls of the sixth set are on a smaller one
        earlier = [k for module in EARLIER for seed in range(1, 6) for group in module.make_kinds(seed) for k in group]
        names = {k.name for k in earlier}
        tables = {(k.arity, k.lo, 31, earlier_table(k)) for k in earlier}
        for seed in range(1, 6):
            for group in hg.make_kinds(seed):
                for k in group:
                    self.assertNotIn(k.name, names)
                    self.assertFalse((k.arity, k.lo, k.hi, table(k)) in tables, (seed, k.name))
                    if k.name not in ("read1", "write1", "sum1", "sum2"):  # on the first 32 numbers too (the identity and the sum are those of the controls of other sets, on a larger domain)
                        head = tuple(k.fn(a, 0) for a in range(k.lo, 32)) if k.arity == 1 else tuple(k.fn(a, b) for a in range(k.lo, 32) for b in range(k.lo, 32))
                        self.assertFalse((k.arity, k.lo, 31, head) in tables, (seed, k.name))

    def test_answers_agree_with_the_rules(self):
        for seed in (1, 2, 3, 4, 5, 6):
            d = self.write(hg, seed)
            _, tongues = hg.world_tongues(seed)
            key = {r[0]: r[2] for r in read_rows(os.path.join(d, "key.tsv"), 5)}
            kinds = self.kinds(seed)
            for name in ("lessons.tsv", "stream.tsv"):
                for r in read_rows(os.path.join(d, name), 5):
                    if name == "lessons.tsv":
                        unit, q, truth = r[3], r[1], r[2]
                    elif r[1] == "trap":
                        continue
                    else:
                        unit, q, truth = r[2], r[3], r[4]
                    k = kinds[key[unit]]
                    fits = parse_question(q, unit, k.arity, tongues[k.tin])
                    self.assertEqual(len(fits), 1, (q, fits))
                    args = [tongues[k.tin].parse(x) for x in fits[0][1]]
                    self.assertTrue(all(k.lo <= n <= k.hi for n in args), q)
                    self.assertEqual(tongues[k.tout].say(k.fn(*(args + [0] * (2 - len(args))))), truth, q)

    def test_the_ways_of_asking(self):
        # each kind is taught in two or three ways (all eight for the two bridges); 16 kinds have held-out facts asked in a way that was not taught for them
        for seed in (1, 2, 3):
            d = self.write(hg, seed)
            _, tongues = hg.world_tongues(seed)
            key = {r[0]: r[2] for r in read_rows(os.path.join(d, "key.tsv"), 5)}
            kinds = self.kinds(seed)

            def frames_of(unit, q):
                k = kinds[key[unit]]
                return {i for i, _ in parse_question(q, unit, k.arity, tongues[k.tin])}

            taught = {}
            for r in read_rows(os.path.join(d, "lessons.tsv"), 5):
                taught.setdefault(r[3], set()).update(frames_of(r[3], r[1]))
            self.assertEqual(len(taught), 33)
            for unit, frames in taught.items():
                k = kinds[key[unit]]
                self.assertIn(len(frames), (8,) if k.bridge else (2, 3), k.name)
            self.assertEqual({kinds[key[u]].name for u, fr in taught.items() if len(fr) == 8}, {"halves", "mix3"})
            reworded = set()
            for r in read_rows(os.path.join(d, "stream.tsv"), 5):
                if r[1] != "trap":
                    if not frames_of(r[2], r[3]) & taught[r[2]]:
                        reworded.add(r[2])
            self.assertEqual(len(reworded), 16, seed)
            self.assertTrue({kinds[key[u]].role for u in reworded} >= {"newop", "control", "compose"})
            self.assertFalse({kinds[key[u]].name for u in reworded} & {"halves", "mix3"})

    def test_the_numerals(self):
        grammars = {}
        for seed in range(1, 13):
            words, tongues = hg.world_tongues(seed)
            names = [type(tongues[t]).__name__ for t in ("t1", "t2", "t3")]
            self.assertEqual(len(set(names)), 3, seed)
            self.assertNotIn("Mixed", names[2:])  # a numeral of fixed width is never the family without meaning
            grammars.setdefault(names[0], []).append(seed)
            seen_words = set()
            for t in ("t1", "t2", "t3"):
                tongue = tongues[t]
                self.assertEqual(len(set(tongue.words)), len(tongue.words))
                self.assertFalse(seen_words & set(tongue.words))
                seen_words |= set(tongue.words)
                self.assertFalse(set(tongue.words) & set(hg.FRAME_WORDS))
                self.assertGreaterEqual(tongue.cap, 995)
                said = [tongue.say(n) for n in range(tongue.cap + 1)]
                self.assertEqual(len(set(said)), tongue.cap + 1, (seed, t))
                for n, text in enumerate(said):
                    self.assertEqual(tongue.parse(text), n, (seed, t, n))
                    self.assertLessEqual(len(text.split(" ")), 6)
                # every number of the exam's domain has a numeral with a word that occurs twice for many of them: the repeats are not rare
                self.assertGreaterEqual(sum(1 for x in said if len(set(x.split(" "))) < len(x.split(" "))), 60, (seed, t))
            # the three grammars are written with words of their own
            self.assertTrue(all(set(tongues[t].foreign) == seen_words - set(tongues[t].words) for t in ("t1", "t2", "t3")))
        self.assertEqual(sorted(grammars), ["Lsd", "Mixed", "Mult", "Sub"])

    def test_each_grammar_does_what_it_says(self):
        for seed in range(1, 13):
            _, tongues = hg.world_tongues(seed)
            for t in ("t1", "t2", "t3"):
                tongue = tongues[t]
                if isinstance(tongue, hg.Sub):
                    words = {w: i for i, w in tongue.digit.items()}
                    # a last digit of r or more is written as the next ten minus the rest, `twenty minus two` for 18 when r is 8 or less
                    for n in range(1, tongue.cap + 1):
                        toks = tongue.say(n).split(" ")
                        self.assertEqual(tongue.minus in toks, n % 10 >= tongue.r, n)
                        if tongue.minus in toks:
                            self.assertEqual(toks[-2], tongue.minus)
                            self.assertEqual(tongue.parse(" ".join(toks[:-2])) - words[toks[-1]], n)
                    self.assertIn(tongue.r, (6, 7, 8))
                elif isinstance(tongue, hg.Mult):
                    b = tongue.base
                    self.assertEqual(tongue.say(b), tongue.tens)  # the digit 1 before a multiplier is left out
                    self.assertEqual(tongue.say(b * b), tongue.hund)
                    self.assertEqual(tongue.say(b + 1), f"{tongue.tens} {tongue.digit[1]}")
                    self.assertEqual(tongue.say(3 * b * b + 2 * b + 4), " ".join([tongue.digit[3], tongue.hund, tongue.digit[2], tongue.tens, tongue.digit[4]]))
                    self.assertEqual(tongue.say(2 * b * b + 4), " ".join([tongue.digit[2], tongue.hund, tongue.digit[4]]))
                elif isinstance(tongue, hg.Lsd):
                    b = tongue.base
                    self.assertEqual(tongue.say(b + 3), f"{tongue.digit[3]} {tongue.digit[1]}")
                    self.assertEqual(tongue.say(b * b), f"{tongue.digit[0]} {tongue.digit[0]} {tongue.digit[1]}")
                    # a number with a second word for a digit reads the same with either
                    rng = hg.seeded(seed, 9)
                    differing = 0
                    for n in range(1, 400):
                        text = tongue.say(n, rng)
                        differing += text != tongue.say(n)
                        self.assertEqual(tongue.parse(text), n)
                    self.assertGreater(differing, 40)
                    self.assertEqual(len(tongue.alt), 2)
                else:
                    r0, r1, r2 = tongue.radix
                    self.assertEqual(sorted(tongue.radix), [7, 12, 12])
                    self.assertEqual(tongue.say(r0), f"{tongue.digit[0]} {tongue.digit[1]} {tongue.digit[0]}")
                    self.assertEqual(tongue.say(r0 * r1), f"{tongue.digit[0]} {tongue.digit[0]} {tongue.digit[1]}")
                    self.assertTrue(all(len(tongue.say(n).split(" ")) == 3 for n in range(1000)))

    def test_what_is_no_numeral_is_none(self):
        for seed in range(1, 13):
            words, tongues = hg.world_tongues(seed)
            rng = hg.seeded(seed, 11)
            everywhere = [tongues[t] for t in FAMILIES]
            for t in ("t1", "t2", "t3"):
                tongue = tongues[t]
                for make in (tongue.bad_junk, tongue.bad_repeat):
                    for _ in range(40):
                        bad = make(rng)
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
            lessons, stream, key = hg.build(seed)
            _, tongues = hg.world_tongues(seed)
            kind = {r[0]: r[2] for r in key}
            for fam, names in (("t1", {"read1", "write1"}), ("t2", {"sum2", "double2"})):
                seen = set()
                for r in lessons:
                    if kind[r[3]] in names:
                        seen |= set(r[1].split(" ")) | set(r[2].split(" "))
                self.assertEqual(set(tongues[fam].words) - seen, set(), (seed, fam))

    def test_repeats_are_in_the_stream(self):
        for seed in (1, 2, 3, 4):
            d = self.write(hg, seed)
            _, tongues = hg.world_tongues(seed)
            key = {r[0]: r[2] for r in read_rows(os.path.join(d, "key.tsv"), 5)}
            kinds = self.kinds(seed)
            stream = [r for r in read_rows(os.path.join(d, "stream.tsv"), 5) if r[1] != "trap"]
            for unit, name in key.items():
                k = kinds[name]
                rows = [r for r in stream if r[2] == unit]
                args = [[tongues[k.tin].parse(x) for x in parse_question(r[3], unit, k.arity, tongues[k.tin])[0][1]] for r in rows]
                if k.arity == 2:  # a number named twice in a question
                    self.assertGreaterEqual(sum(1 for a in args if a[0] == a[1]), 3, name)
                if k.arity == 1 and k.tin != "d" and k.hi > 99:  # a numeral with a word that occurs twice
                    self.assertGreaterEqual(sum(1 for a in args if len(set(tongues[k.tin].say(a[0]).split(" "))) < len(tongues[k.tin].say(a[0]).split(" "))), 4, name)

    def test_the_operations_against_known_values(self):
        self.assertEqual(hg.SIDON[:12], [1, 2, 4, 8, 13, 21, 31, 45, 66, 81, 97, 123])
        self.assertEqual(hg.SIDON[-1], 1395)
        self.assertEqual([hg.look_say_length(n) for n in range(1, 17)], [1, 2, 2, 4, 6, 6, 8, 10, 14, 20, 26, 34, 46, 62, 78, 102])
        self.assertEqual(hg.look_say_length(31), 5808)
        self.assertEqual([hg.sandpile_cells(n) for n in range(0, 10)], [0, 1, 1, 1, 4, 5, 5, 5, 4, 5])
        self.assertEqual([hg.right_triangles(n) for n in (1, 4, 5, 9, 10, 12, 13, 15, 17, 20, 25, 26, 29, 30, 31)], [0, 0, 1, 1, 2, 2, 3, 4, 5, 6, 8, 9, 10, 11, 11])
        self.assertEqual([hg.sorted_cube(n) for n in (0, 1, 3, 4, 10, 11, 31)], [0, 1, 27, 46, 1, 1133, 12799])
        self.assertEqual(hg.luhn_digit(7992739871), 3)
        self.assertEqual([hg.luhn_digit(n) for n in (0, 1, 5, 12, 1234)], [0, 8, 9, 5, 4])
        self.assertEqual([hg.niven_branch(a, 3) for a in (1, 10, 11, 12, 13, 19, 21)], [1, 10, 5, 4, 7, 37, 7])
        self.assertEqual([hg.digit_sum(n) for n in (0, 9, 10, 99, 100, 989)], [0, 9, 1, 18, 1, 26])

    def test_the_rules_against_second_implementations(self):
        # the sequence with all sums different: the property itself, and that every term is the smallest that keeps it
        terms = hg.SIDON
        for i in range(len(terms)):
            sums = [terms[x] + terms[y] for x in range(i + 1) for y in range(x, i + 1)]
            self.assertEqual(len(sums), len(set(sums)))
            if i:
                for m in range(terms[i - 1] + 1, terms[i]):
                    with_m = [terms[x] + terms[y] for x in range(i) for y in range(x, i)] + [m + terms[x] for x in range(i)] + [2 * m]
                    self.assertNotEqual(len(with_m), len(set(with_m)), (i, m))
        # the strings that read themselves aloud, by groupby
        s = "1"
        for n in range(1, 25):
            self.assertEqual(hg.look_say_length(n), len(s))
            s = "".join(str(len(list(g))) + c for c, g in itertools.groupby(s))
        # the sandpile on a board with walls far away, by sweeping until nothing changes
        for a in range(0, 32):
            size = 41
            grid = [[0] * size for _ in range(size)]
            grid[20][20] = a
            changed = True
            while changed:
                changed = False
                for x in range(size):
                    for y in range(size):
                        if grid[x][y] >= 4:
                            grid[x][y] -= 4
                            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                                grid[x + dx][y + dy] += 1
                            changed = True
            self.assertEqual(hg.sandpile_cells(a), sum(1 for row in grid for v in row if v > 0), a)
        # the right triangles: from the formula for all of them, (k * (m*m - n*n), k * 2*m*n, k * (m*m + n*n))
        for limit in range(1, 80):
            triples = set()
            for m in range(2, 12):
                for n in range(1, m):
                    for k in range(1, 40):
                        x, y, z = sorted((k * (m * m - n * n), k * 2 * m * n))[0], sorted((k * (m * m - n * n), k * 2 * m * n))[1], k * (m * m + n * n)
                        if z <= limit:
                            triples.add((x, y, z))
            self.assertEqual(hg.right_triangles(limit), len(triples), limit)
        # the check digit: the number with it appended passes the usual test (double every second digit from the right, starting from the second)
        for n in range(0, 3000):
            digits = [int(c) for c in str(n * 10 + hg.luhn_digit(n))][::-1]
            total = sum(d if i % 2 == 0 else (d * 2 if d < 5 else d * 2 - 9) for i, d in enumerate(digits))
            self.assertEqual(total % 10, 0, n)
        # the sorted cube with the digits as a list
        for a in range(0, 32):
            self.assertEqual(hg.sorted_cube(a), int("".join(sorted(list(str(a**3))))))
        # the digit-sum branch: a is divisible by its digit sum exactly for the Niven numbers
        niven = [a for a in range(1, 100) if a % hg.digit_sum(a) == 0]
        self.assertEqual(niven[:16], [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 18, 20, 21, 24, 27])

    def test_every_kind_has_enough_values_and_the_constants_are_in_range(self):
        for seed in range(1, 31):
            kinds = self.kinds(seed)
            for k in kinds.values():
                self.assertGreaterEqual(len(set(table(k))), 3 if k.name in ("size3", "lastword3") else 5, (seed, k.name))
                self.assertLessEqual(k.hi, 999)
                _, tongues = hg.world_tongues(seed)
                if k.arity == 1:
                    outputs = {k.fn(a, 0) for a in range(k.lo, k.hi + 1)}
                else:
                    outputs = {k.fn(a, b) for a in range(k.lo, min(k.hi, 40) + 1) for b in range(k.lo, min(k.hi, 40) + 1)}
                if k.tout in ("t1", "t2", "t3"):
                    self.assertLessEqual(max(outputs), tongues[k.tout].cap, (seed, k.name))
                    self.assertGreaterEqual(min(outputs), 0)
                if k.tin in ("t1", "t2", "t3"):
                    self.assertLessEqual(k.hi, tongues[k.tin].cap, (seed, k.name))
            self.assertGreaterEqual(len({kinds["ratiofloor"].fn(a, b) for a in range(32) for b in range(32)}), 6)
            self.assertGreaterEqual(len({kinds["twomult"].fn(a, b) for a in range(32) for b in range(32)}), 40)
            m = int(re.search(r"mod (\d+)", kinds["twomult"].rule).group(1))
            self.assertTrue(m in hg.PRIMES and m >= 101)
            self.assertGreaterEqual(int(re.search(r"\((\d+) div", kinds["bigquot"].rule).group(1)), 30000)
            self.assertEqual(kinds["scale1"].hi, hg.world_tongues(seed)[1]["t1"].cap // int(re.match(r"(\d+) \*", kinds["scale1"].rule).group(1)))
        for p in hg.PRIMES:
            self.assertTrue(all(p % q for q in range(2, int(math.sqrt(p)) + 1)))
        # the quadratic form wraps (it is not a polynomial that stays below its modulus) and the two linear forms of the minimum both win somewhere
        kinds = self.kinds(1)
        self.assertGreater(max(kinds["quadpair"].fn(a, b) for a in range(32) for b in range(32)), 40)
        for seed in range(1, 31):
            k = self.kinds(seed)["minform"]
            p, q, c = (int(x) for x in re.match(r"min\((\d+) \* a \+ b, a \+ (\d+) \* b\) \+ (\d+)", k.rule).groups())
            self.assertTrue(any(p * a + b < a + q * b for a in range(32) for b in range(32)) and any(p * a + b > a + q * b for a in range(32) for b in range(32)))

    def test_the_arbitrary_codes_are_not_a_pattern(self):
        for seed in (1, 2, 3):
            kinds = self.kinds(seed)
            cells = [kinds["pairnoise3"].fn(a, b) for a in range(32) for b in range(32)]
            self.assertGreaterEqual(len(set(cells)), 25)
            self.assertLess(max(cells), 100)
            rows = {tuple(kinds["pairnoise3"].fn(a, b) for b in range(32)) for a in range(32)}
            self.assertEqual(len(rows), 32)
            self.assertLess(max(kinds["noisedig"].fn(a, 0) for a in range(32)), 23)
            self.assertLess(max(kinds["noise2"].fn(a, 0) for a in range(32)), 300)
            self.assertLess(max(kinds["noise3"].fn(a, 0) for a in range(32)), 200)
            # the line that is not one: 4a plus 0, 1 or 2, all three occurring
            extra = {kinds["noisyline"].fn(a, 0) - 4 * a for a in range(32)}
            self.assertEqual(extra, {0, 1, 2})
            self.assertTrue(all(kinds[n].role == "unexplainable" for n in ("noisedig", "noisyline", "noise2", "noise3", "pairnoise3")))

    def test_the_trap_rows(self):
        for seed in (1, 2, 3):
            d = self.write(hg, seed)
            _, tongues = hg.world_tongues(seed)
            key = read_rows(os.path.join(d, "key.tsv"), 5)
            words = {r[0]: r[2] for r in key}
            kinds = self.kinds(seed)
            lessons = read_rows(os.path.join(d, "lessons.tsv"), 5)
            traps = [r for r in read_rows(os.path.join(d, "stream.tsv"), 5) if r[1] == "trap"]
            self.assertEqual(len(traps), 63)
            self.assertEqual(len({r[3] for r in traps}), 63)
            self.assertFalse({r[3] for r in traps} & {r[1] for r in lessons})
            # no lesson ever mentions the words that only trap rows carry
            only_traps = {r[2] for r in traps} - set(words)
            self.assertEqual(len(only_traps), 9)  # the eight new words and the label of the bare numbers
            self.assertGreaterEqual(len({r[4] for r in traps}), 45)  # the truths are drawn one by one
            everywhere = [tongues[t] for t in FAMILIES]
            valid_anywhere = lambda x: any(t.valid(x) for t in everywhere)  # noqa: E731
            counts = {"two kinds": 0, "new word": 0, "arity": 0, "other family": 0, "bare": 0, "no numeral": 0, "order": 0}
            anything = lambda x: not set(x.split(" ")) & hg.FRAME_WORDS  # noqa: E731  (a run of words that holds no word of the question's frame)
            for r in traps:
                unit, q = r[2], r[3]
                if two_kinds(q, words):
                    counts["two kinds"] += 1
                elif unit not in words:
                    counts["bare" if valid_anywhere(q.replace("what is ", "")) else "new word"] += 1
                else:
                    k = kinds[words[unit]]
                    own, other = hg.frames_of(k.arity), hg.frames_of(3 - k.arity)
                    scrambled = hg.SCRAMBLED_UNARY + hg.SCRAMBLED_BINARY
                    self.assertFalse(parse_question(q, unit, k.arity, tongues[k.tin]), q)  # a trap row is never an ordinary question
                    if any(frame_matches(f, unit, q, valid_anywhere) for f in scrambled):
                        counts["order"] += 1
                    elif any(frame_matches(f, unit, q, valid_anywhere) for f in own):
                        counts["other family"] += 1
                    elif any(frame_matches(f, unit, q, anything) for f in own):
                        counts["no numeral"] += 1
                    else:
                        self.assertTrue(any(frame_matches(f, unit, q, valid_anywhere) for f in other), q)
                        counts["arity"] += 1
            self.assertEqual(counts, {"two kinds": 8, "new word": 8, "arity": 8, "other family": 8, "bare": 7, "no numeral": 18, "order": 6})

    def test_compose_first_changes_only_the_teaching_order(self):
        a, b = self.write(hg, 3), self.write(hg, 3, compose_first=True)
        la, lb = read_rows(os.path.join(a, "lessons.tsv"), 5), read_rows(os.path.join(b, "lessons.tsv"), 5)
        self.assertNotEqual(la, lb)
        ka, kb = read_rows(os.path.join(a, "key.tsv"), 5), read_rows(os.path.join(b, "key.tsv"), 5)
        roles = [r[1] for r in kb]
        self.assertLess(roles.index("compose"), roles.index("newop"))
        self.assertLess([r[1] for r in ka].index("newop"), [r[1] for r in ka].index("compose"))

    @unittest.skipIf(np is None, "numpy is needed for the reachability search")
    @unittest.skipIf(os.environ.get("HW_G_SKIP_SEARCH"), "HW_G_SKIP_SEARCH is set")
    def test_no_hard_kind_is_reachable_and_the_controls_are(self):
        seeds = range(1, int(os.environ.get("HW_G_SEARCH_SEEDS", "10")) + 1)
        groups, roles = {}, {}
        for seed in seeds:
            for group in hg.make_kinds(seed):
                for k in group:
                    if not k.numeric:
                        continue
                    name, points = check_points(k)
                    args = lambda p: p + (0,) * (2 - len(p))  # noqa: E731
                    groups.setdefault(name, (points, {}))[1][f"{seed}:{k.name}"] = tuple(k.fn(*args(p)) for p in points)
                    roles[f"{seed}:{k.name}"] = k.role
        reached = {}
        for saturating in (False, True):  # subtraction signed, and stopping at 0
            for name, (points, targets) in groups.items():
                for target, nodes in search(points, targets, saturating).items():
                    reached.setdefault(target, []).append(nodes)
        self.assertEqual(len(roles), len(seeds) * 30)  # every kind but the three that copy or count words
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


# ---- the reachability search: every expression of at most 7 nodes (leaves count) over + - * min max div mod, the variables and the constants 0 to 40, 50, 64 and 100 ----

SMALL = list(range(41)) + [50, 64, 100]
OPS = ("add", "sub", "mul", "min", "max", "div", "mod")


def check_points(k):
    """The sample a kind is matched on, and its name (kinds with the same sample are searched together): the numbers a kind stands for, not their words."""
    rng = first.Rng(777)
    if k.arity == 1:
        if k.hi <= 31:
            return "unary 1..31", [(a,) for a in range(1, 32)]
        if k.hi <= 99 or k.name == "scale1":
            return "unary 1..80", [(a,) for a in range(1, 81)]
        return "unary wide", [(a,) for a in list(range(1, 25)) + list(range(30, 200, 17)) + list(range(200, 495, 59))]
    if k.hi <= 31:
        return "binary 1..31", [(1 + rng.below(31), 1 + rng.below(31)) for _ in range(60)]
    return "binary wide", [(1 + rng.below(24), 1 + rng.below(24)) for _ in range(30)] + [(1 + rng.below(494), 1 + rng.below(494)) for _ in range(30)]


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
