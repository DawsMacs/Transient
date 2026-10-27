# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""Tests for tools/hidden_worlds_f.py: it is deterministic, writes the format `baby exam adapt-write` writes, the key never reaches the exam files,
and every answer follows from its rule.

    python3 -m unittest discover -s tools -p 'test_hidden_worlds_f.py'
"""
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
import hidden_worlds_f as hf

EARLIER = (first, second, third, fourth, fifth)
FRAME_WORDS = {"what", "is", "tell", "me", "please", "of", "and", "equals"}


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
    """A kind's whole table over its domain, as a tuple of values."""
    if k.arity == 1:
        return tuple(k.fn(a, 0) for a in range(k.lo, 32))
    return tuple(k.fn(a, b) for a in range(k.lo, 32) for b in range(k.lo, 32) if not k.ordered or a >= b)


def split_args(question, word):
    """The numerals (as strings) of a question and the index of its way of asking is found by the caller; this takes the question apart."""
    tokens = question.split(" ")
    if tokens[-1] == "equals":
        tokens = tokens[:-1]
    for prefix in (["tell", "me", "what", "is"], ["what", "is"], ["please"]):
        if tokens[: len(prefix)] == prefix:
            tokens = tokens[len(prefix) :]
            break
    i = tokens.index(word)
    if tokens[:2] == [word, "of"]:
        rest = tokens[2:]
        return [" ".join(x) for x in split_on(rest, "and")]
    left, right = tokens[:i], tokens[i + 1 :]
    return [" ".join(x) for x in (left, right) if x]


def split_on(tokens, sep):
    out, cur = [], []
    for t in tokens:
        if t == sep:
            out.append(cur)
            cur = []
        else:
            cur.append(t)
    return out + [cur]


class HiddenWorldsF(unittest.TestCase):
    def write(self, module, seed, **kw):
        d = tempfile.mkdtemp()
        self.addCleanup(lambda: __import__("shutil").rmtree(d, ignore_errors=True))
        module.write(d, seed, **kw)
        return d

    def kinds(self, seed):
        return {k.name: k for group in hf.make_kinds(seed) for k in group}

    def test_same_seed_same_files_other_seed_other_files(self):
        a, b, c = self.write(hf, 1), self.write(hf, 1), self.write(hf, 2)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv", "README.txt"):
            self.assertEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(b, name)), name)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv"):
            self.assertNotEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(c, name)), name)

    def test_the_files_parse_in_the_existing_format(self):
        for seed in (1, 2, 3):
            d = self.write(hf, seed)
            lessons = read_rows(os.path.join(d, "lessons.tsv"), 5)
            stream = read_rows(os.path.join(d, "stream.tsv"), 5)
            self.assertEqual([r[0] for r in lessons], [f"L{i:05d}" for i in range(len(lessons))])
            self.assertEqual([r[0] for r in stream], [f"S{i:05d}" for i in range(len(stream))])
            for r in lessons + stream:
                for field in r:
                    self.assertTrue(field and field == field.strip() and "\t" not in field, r)
            units = {r[3] for r in lessons}
            self.assertEqual(len(units), 29)
            self.assertEqual({r[1] for r in stream}, {"newop", "compose", "control", "unexplainable", "trap"})
            # every fact is taught in two, three or (for the two kinds that carry the equivalences) eight ways of asking, all with the same answer
            facts = {}
            for r in lessons:
                facts.setdefault(r[4], []).append(r)
            self.assertEqual({len(rows) for rows in facts.values()}, {2, 3, 8})
            for rows in facts.values():
                self.assertEqual(len({(x[2], x[3]) for x in rows}), 1)
                self.assertEqual(len({x[1] for x in rows}), len(rows))
            # the stream asks nothing that was taught and nothing twice; every lesson question is asked once too
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
            d = self.write(hf, seed)
            key = read_rows(os.path.join(d, "key.tsv"), 5)
            exam = "".join(read_bytes(os.path.join(d, name)).decode("utf-8") for name in ("lessons.tsv", "stream.tsv", "README.txt"))
            for word, role, name, rule, learn in key:
                self.assertTrue(rule and learn)
                self.assertNotIn(name, exam, name)
                self.assertNotIn(rule, exam, name)
                self.assertNotIn(learn, exam, name)
            # what the exam files hold is the fixed words of the questions, the kinds' words, the numerals and numbers
            _, tongues = hf.world_tongues(seed)
            allowed = set(FRAME_WORDS) | {r[0] for r in key} | set(hf.ONES) | set(hf.TENS) | set(tongues["v"].vocab)
            allowed |= set(getattr(tongues["c"], "digit_words", []))
            body = "".join(read_bytes(os.path.join(d, name)).decode("utf-8") for name in ("lessons.tsv", "stream.tsv"))
            body = re.sub(r"\bL\d{5}\b|\bS\d{5}\b", "", body)
            tokens = set(re.findall(r"[a-z]+", body)) - {"newop", "compose", "control", "unexplainable", "trap"}
            fresh = tokens - allowed
            # the only words of the exam that are in no key row are the new words and the label of the bare numbers in the trap rows
            self.assertTrue(all(t.endswith(hf.ENDING) for t in fresh), fresh)
            self.assertLessEqual(len(fresh), 9)
            self.assertEqual(sorted(os.listdir(d)), ["README.txt", "key.tsv", "lessons.tsv", "stream.tsv"])

    def test_the_kinds_by_role(self):
        d = self.write(hf, 1)
        key = read_rows(os.path.join(d, "key.tsv"), 5)
        counts = {}
        for r in key:
            counts[r[1]] = counts.get(r[1], 0) + 1
        self.assertEqual(counts, {"control": 10, "newop": 12, "compose": 3, "unexplainable": 4})
        stream = read_rows(os.path.join(d, "stream.tsv"), 5)
        per_role = {}
        for r in stream:
            per_role[r[1]] = per_role.get(r[1], 0) + 1
        self.assertEqual(per_role, {"control": 164, "newop": 184, "compose": 42, "unexplainable": 64, "trap": 38})

    def test_the_words_are_this_sets_own(self):
        for seed in range(1, 11):
            mine = {r[0] for r in read_rows(os.path.join(self.write(hf, seed), "key.tsv"), 5)}
            self.assertEqual(len(mine), 29)
            self.assertTrue(all(w.endswith("yarn") for w in mine))
            for module, columns in ((first, 4), (second, 4), (third, 4), (fourth, 5), (fifth, 5)):
                for other in range(1, 11):
                    theirs = {r[0] for r in read_rows(os.path.join(self.write(module, other), "key.tsv"), columns)}
                    self.assertFalse(mine & theirs, (seed, other))

    def test_no_kind_is_one_of_the_earlier_sets(self):
        # by name, and by table: none of this set's kinds equals any earlier kind (the controls and the arbitrary codes included) on its whole domain
        earlier = [k for module in EARLIER for seed in range(1, 6) for group in module.make_kinds(seed) for k in group]
        names = {k.name for k in earlier}
        tables = {(k.arity, k.lo, k.ordered, table(k)) for k in earlier}
        for seed in range(1, 6):
            for group in hf.make_kinds(seed):
                for k in group:
                    self.assertNotIn(k.name, names)
                    self.assertNotIn((k.arity, k.lo, k.ordered, table(k)), tables, (seed, k.name))

    def test_answers_agree_with_the_rules(self):
        for seed in (1, 2, 3, 4):
            d = self.write(hf, seed)
            _, tongues = hf.world_tongues(seed)
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
                    args = [tongues[k.tin].parse(x) for x in split_args(q, unit)]
                    self.assertEqual(len(args), k.arity, q)
                    self.assertTrue(all(k.lo <= n <= 31 or (k.wide and n <= 99) for n in args), q)
                    self.assertEqual(tongues[k.tout].say(k.fn(*(args + [0] * (2 - len(args))))), truth, q)

    def test_the_ways_of_asking(self):
        # each kind is taught in two or three ways (all eight for the two bridges); 14 kinds have held-out facts asked in a way that was not taught for them
        for seed in (1, 2, 3):
            d = self.write(hf, seed)
            key = {r[0]: r[2] for r in read_rows(os.path.join(d, "key.tsv"), 5)}
            kinds = self.kinds(seed)

            def frame_of(unit, q):
                frames = hf.frames_of(kinds[key[unit]].arity)
                args = split_args(q, unit)
                return [i for i, fr in enumerate(frames) if hf.put(fr, unit, *args) == q]

            taught = {}
            for r in read_rows(os.path.join(d, "lessons.tsv"), 5):
                taught.setdefault(r[3], set()).update(frame_of(r[3], r[1]))
            self.assertEqual(len(taught), 29)
            for unit, frames in taught.items():
                k = kinds[key[unit]]
                self.assertIn(len(frames), (8,) if k.bridge else (2, 3), k.name)
            reworded = set()
            for r in read_rows(os.path.join(d, "stream.tsv"), 5):
                if r[1] != "trap":
                    hits = set(frame_of(r[2], r[3]))
                    if not hits & taught[r[2]]:
                        reworded.add(r[2])
            self.assertEqual(len(reworded), 14, seed)
            self.assertTrue({kinds[key[u]].role for u in reworded} >= {"newop", "control"})
            self.assertFalse({kinds[key[u]].name for u in reworded} & {"bumpby", "diffsquares"})

    def test_the_numerals(self):
        for n in range(100):
            self.assertEqual(hf.parse_english(hf.english(n)), n)
        self.assertEqual([hf.english(n) for n in (0, 7, 13, 20, 21, 40, 99)], ["zero", "seven", "thirteen", "twenty", "twenty one", "forty", "ninety nine"])
        for seed in range(1, 11):
            words, tongues = hf.world_tongues(seed)
            self.assertEqual(tongues["c"].label == "english", seed % 2 == 1)
            for n in range(100):
                self.assertEqual(tongues["c"].parse(tongues["c"].say(n)), n)
            self.assertEqual(len({tongues["c"].say(n) for n in range(100)}), 100)
            vocab = tongues["v"].vocab
            self.assertEqual(len(set(vocab)), 64)
            for n in range(64):
                self.assertEqual(tongues["v"].parse(tongues["v"].say(n)), n)
            if tongues["c"].label != "english":
                digit_words = tongues["c"].digit_words
                self.assertEqual(len(set(digit_words)), len(digit_words))
                self.assertFalse(set(digit_words) & set(vocab))
            # no word is two things
            self.assertFalse(set(vocab) & (set(hf.ONES) | set(hf.TENS) | FRAME_WORDS))
            # the two families are never mixed up inside a kind, and every number the vocabulary family needs has a word
            for k in self.kinds(seed).values():
                self.assertIn(k.tin, "dcv")
                self.assertIn(k.tout, "dcv")
                if "v" in (k.tin, k.tout):
                    self.assertEqual((k.tin, k.tout), ("v", "v"))
                    self.assertLess(max(table(k)), 64)
                if "c" in (k.tin, k.tout) and k.name not in ("spellnum", "readnum"):
                    self.assertEqual((k.tin, k.tout), ("c", "c"))
                    self.assertLessEqual(max(table(k)), 99)

    def test_the_spelling_kinds_cover_the_pieces(self):
        # the words of a held-out numeral all occur in other lessons of the same kind, so that the meaning can be worked out
        for seed in (1, 2, 3, 4):
            d = self.write(hf, seed)
            key = {r[0]: r[2] for r in read_rows(os.path.join(d, "key.tsv"), 5)}
            lessons = read_rows(os.path.join(d, "lessons.tsv"), 5)
            stream = read_rows(os.path.join(d, "stream.tsv"), 5)
            for name, column in (("spellnum", 2), ("readnum", 1)):
                unit = next(u for u, n in key.items() if n == name)
                seen = {t for r in lessons if r[3] == unit for t in (r[2] if column == 2 else split_args(r[1], unit)[0]).split(" ")}
                rows = [r for r in stream if r[2] == unit and r[1] != "trap"]
                self.assertEqual(len(rows), 10)
                for r in rows:
                    numeral = r[4] if column == 2 else split_args(r[3], unit)[0]
                    self.assertTrue(set(numeral.split(" ")) <= seen, (name, numeral))
                self.assertEqual(len([r for r in lessons if r[3] == unit]) // 90 in (2, 3), True)

    def test_the_operations_against_known_values(self):
        self.assertEqual([hf.recaman(n, 0) for n in range(12)], [0, 1, 3, 6, 2, 7, 13, 20, 12, 21, 11, 22])
        self.assertEqual([hf.min_coins(n, (1, 6, 10)) for n in (0, 1, 6, 10, 12, 18, 19, 30)], [0, 1, 1, 1, 2, 3, 4, 3])
        self.assertEqual([hf.leaper_moves(a, b, 1, 2) for a, b in ((0, 0), (1, 0), (1, 1), (2, 2), (1, 2), (3, 3), (2, 0), (4, 5))], [0, 3, 2, 4, 1, 2, 2, 3])
        self.assertEqual([hf.automaton_cells(n, 30) for n in range(10)], [1, 3, 3, 6, 4, 9, 5, 12, 7, 12])
        self.assertEqual([hf.automaton_cells(n, 90) for n in range(8)], [2 ** bin(n).count("1") for n in range(8)])
        self.assertEqual([n for n in range(32) if hf.binary_palindrome(n)], [0, 1, 3, 5, 7, 9, 15, 17, 21, 27, 31])
        self.assertEqual([hf.pal_branch(n, 2, 3) for n in (0, 1, 2, 3, 4, 5, 6)], [2, 5, 1, 11, 2, 17, 3])
        self.assertEqual([hf.weave(12, 34, "a"), hf.weave(12, 34, "b"), hf.weave(5, 678, "a"), hf.weave(678, 5, "a"), hf.weave(7, 3, "a"), hf.weave(0, 0, "a")], [1324, 3142, 5678, 6578, 73, 0])
        self.assertEqual([hf.digit_step(n, 19) for n in range(8)], [19, 29, 40, 44, 52, 59, 73, 83])
        self.assertEqual([hf.exp_mod(n, 3, 101) for n in range(6)], [1, 3, 9, 27, 81, 41])
        self.assertEqual([hf.spiral_distance(n) for n in range(1, 14)], [0, 1, 2, 1, 2, 1, 2, 1, 2, 3, 2, 3, 4])
        self.assertEqual([hf.distinct_substrings(n, 2) for n in (1, 2, 3, 5, 7, 10)], [1, 3, 2, 5, 3, 7])
        self.assertEqual([hf.last_nonzero(n, 0) for n in range(13)], [1, 1, 2, 6, 4, 2, 2, 4, 2, 8, 8, 8, 6])
        self.assertEqual([hf.next_palindrome(a, 1) for a in (0, 8, 9, 10, 11, 98, 99, 120)], [1, 9, 11, 11, 22, 99, 101, 121])
        self.assertEqual([hf.next_palindrome(a, 3) for a in (0, 7, 8, 30)], [3, 11, 11, 33])

    def test_the_rules_against_second_implementations(self):
        # the sequence that steps back: the order of the steps, and a list instead of a set
        for s0 in (0, 2, 4, 6):
            seq = [s0]
            for n in range(1, 40):
                y = seq[-1] - n
                seq.append(y if y > 0 and y not in seq else seq[-1] + n)
            self.assertEqual([hf.recaman(n, s0) for n in range(40)], seq)
        # the cheapest payment: by trying every number of coins in turn
        for coins in hf.COIN_SETS:
            for a in range(0, 32):
                best = next(n for n in range(0, a + 1) if reachable(a, coins, n))
                self.assertEqual(hf.min_coins(a, coins), best, (coins, a))
        # the leaper: symmetric, 0 only at the start, one more than the best neighbor, and the same on a wider board
        for p, q in ((1, 2), (2, 3)):
            t = hf.leaper_table(p, q)
            moves = {(s * x, u * y) for x, y in ((p, q), (q, p)) for s in (1, -1) for u in (1, -1)}
            for a in range(32):
                for b in range(32):
                    d = hf.leaper_moves(a, b, p, q)
                    self.assertEqual(d, hf.leaper_moves(b, a, p, q))
                    self.assertEqual(d == 0, (a, b) == (0, 0))
                    if (a, b) != (0, 0):
                        self.assertEqual(d, 1 + min(t[(a + dx, b + dy)] for dx, dy in moves if (a + dx, b + dy) in t))
                    self.assertEqual(d, hf.leaper_table(p, q, 18)[(a, b)])
        # the automaton: a row of bits written out, with the rule as a table of 8 entries
        for rule in hf.RULES:
            row = [0] * 80 + [1] + [0] * 80
            for n in range(0, 33):
                self.assertEqual(hf.automaton_cells(n, rule), sum(row), (rule, n))
                row = [(rule >> (row[i - 1] * 4 + row[i] * 2 + row[(i + 1) % len(row)])) & 1 if 0 < i < len(row) - 1 else 0 for i in range(len(row))]
        # the spiral: neighbors on the walk are one step apart, the ring of cell n is set by the squares of odd numbers
        for n in range(1, 200):
            self.assertEqual(abs(hf.spiral_distance(n + 1) - hf.spiral_distance(n)), 1)
        for ring in range(1, 6):
            self.assertEqual(max(hf.spiral_distance(n) for n in range((2 * ring - 1) ** 2 + 1, (2 * ring + 1) ** 2 + 1)), 2 * ring)
        # the substrings, counted by the lengths in turn
        for base in (2, 3):
            for a in range(1, 64):
                s = hf.to_base(a, base)
                total = sum(len({s[i : i + L] for i in range(len(s) - L + 1)}) for L in range(1, len(s) + 1))
                self.assertEqual(hf.distinct_substrings(a, base), total)
                self.assertEqual(int(s, base), a)
        # the last nonzero digit of a factorial by taking the zeros out of the running product at each step
        x = 1
        for n in range(0, 40):
            if n:
                x *= n
                while x % 10 == 0:
                    x //= 10
            self.assertEqual(hf.last_nonzero(n, 0), x % 10)
            self.assertEqual(hf.last_nonzero(n, 1), hf.last_nonzero(n + 1, 0))
        # the next palindrome, found by building them in order
        pals = [n for n in range(0, 2000) if str(n) == str(n)[::-1]]
        for a in range(0, 1500):
            for t in (1, 2, 3):
                self.assertEqual(hf.next_palindrome(a, t), next(p for p in pals if p >= a + t))
        # the digit step is the number plus its digit sum, written as a loop on strings
        x = 4321
        for n in range(0, 20):
            self.assertEqual(hf.digit_step(n, 4321), x)
            x += sum(map(int, str(x)))
        # the power by repeated multiplication
        for g, p in ((2, 101), (37, 211), (5, 251)):
            x = 1
            for n in range(0, 40):
                self.assertEqual(hf.exp_mod(n, g, p), x)
                x = x * g % p
        # the weave: the digits of the pair, taken one by one
        for a, b in ((1, 2), (10, 3), (3, 10), (99, 100), (123, 4567)):
            for first in "ab":
                x, y = (str(a), str(b)) if first == "a" else (str(b), str(a))
                out = ""
                while x or y:
                    out += x[:1] + y[:1]
                    x, y = x[1:], y[1:]
                self.assertEqual(hf.weave(a, b, first), int(out))

    def test_every_kind_has_enough_values_and_the_constants_are_in_range(self):
        for seed in range(1, 31):
            for k in self.kinds(seed).values():
                self.assertGreaterEqual(len(set(table(k))), 5 if k.name != "noise9" else 5, (seed, k.name))
            kinds = self.kinds(seed)
            self.assertGreaterEqual(len(set(table(kinds["automaton"]))), 12, seed)
            self.assertGreaterEqual(len(set(table(kinds["powmod"]))), 28, seed)
            self.assertTrue(re.search(r"x\(0\) = (\d{4})", kinds["digitstep"].rule), seed)
            self.assertGreaterEqual(int(re.search(r"x\(0\) = (\d+)", kinds["digitstep"].rule).group(1)), 1000)
            p = int(re.search(r"mod (\d+)", kinds["powmod"].rule).group(1))
            self.assertIn(p, hf.PRIMES)
            self.assertGreater(p, 100)
        # the coin sets always contain a coin of 1 (so that every amount can be paid) and are not all won by taking the largest coin first
        for coins in hf.COIN_SETS:
            self.assertEqual(coins[0], 1)
            self.assertTrue(any(hf.min_coins(a, coins) < greedy(a, coins) for a in range(32)), coins)
        for rule in hf.RULES:
            self.assertEqual(rule % 2, 0)

    def test_the_arbitrary_codes_are_not_a_pattern(self):
        for seed in (1, 2, 3):
            kinds = self.kinds(seed)
            self.assertEqual(kinds["noisepairwords"].arity, 2)
            cells = [kinds["noisepairwords"].fn(a, b) for a in range(32) for b in range(32)]
            self.assertGreaterEqual(len(set(cells)), 25)
            self.assertLess(max(cells), 40)
            rows = {tuple(kinds["noisepairwords"].fn(a, b) for b in range(32)) for a in range(32)}
            self.assertEqual(len(rows), 32)
            self.assertLess(max(kinds["noise9"].fn(a, 0) for a in range(32)), 9)
            stair = [kinds["stair"].fn(a, 0) for a in range(32)]
            self.assertEqual(stair, sorted(stair))
            self.assertEqual(len(set(stair)), 32)
            self.assertGreater(len({y - x for x, y in zip(stair, stair[1:])}), 4)
            self.assertLess(max(kinds["noisewords"].fn(a, 0) for a in range(32)), 40)

    def test_the_trap_rows(self):
        for seed in (1, 2, 3):
            d = self.write(hf, seed)
            key = read_rows(os.path.join(d, "key.tsv"), 5)
            words = {r[0] for r in key}
            lessons = read_rows(os.path.join(d, "lessons.tsv"), 5)
            traps = [r for r in read_rows(os.path.join(d, "stream.tsv"), 5) if r[1] == "trap"]
            self.assertEqual(len(traps), 38)
            self.assertEqual(len({r[3] for r in traps}), 38)
            self.assertFalse({r[3] for r in traps} & {r[1] for r in lessons})
            # no lesson ever mentions the units that only trap rows carry
            only_traps = {r[2] for r in traps} - words
            self.assertEqual(len(only_traps), 9)
            self.assertFalse(only_traps & {r[3] for r in lessons})
            # the truths do not repeat as a pattern: they are drawn one by one
            self.assertGreaterEqual(len({r[4] for r in traps}), 30)
            self.assertEqual(sum(1 for r in traps if " and " in r[3]), 8)
            self.assertEqual(sum(1 for r in traps if r[2] in only_traps and r[2] in r[3].split(" ")), 8)
            self.assertEqual(sum(1 for r in traps if r[2] in only_traps and r[2] not in r[3].split(" ")), 6)

    def test_compose_first_changes_only_the_teaching_order(self):
        a, b = self.write(hf, 3), self.write(hf, 3, compose_first=True)
        la, lb = read_rows(os.path.join(a, "lessons.tsv"), 5), read_rows(os.path.join(b, "lessons.tsv"), 5)
        self.assertNotEqual(la, lb)
        ka, kb = read_rows(os.path.join(a, "key.tsv"), 5), read_rows(os.path.join(b, "key.tsv"), 5)
        roles = [r[1] for r in kb]
        self.assertLess(roles.index("compose"), roles.index("newop"))
        self.assertLess([r[1] for r in ka].index("newop"), [r[1] for r in ka].index("compose"))


def reachable(a, coins, n):
    """Whether a can be paid with exactly n coins of the given values (some may be used any number of times)."""
    if n == 0:
        return a == 0
    return any(a >= c and reachable(a - c, coins, n - 1) for c in coins)


def greedy(a, coins):
    n = 0
    for c in sorted(coins, reverse=True):
        n += a // c
        a %= c
    return n


if __name__ == "__main__":
    unittest.main()
