# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""Tests for tools/hidden_worlds_d.py: it is deterministic, writes the format `baby exam adapt-write` writes, the key never reaches the exam files,
and every answer follows from its rule.

    python3 -m unittest discover -s tools -p 'test_hidden_worlds_d.py'
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
import hidden_worlds_d as hd


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


class HiddenWorldsD(unittest.TestCase):
    def write(self, module, seed, **kw):
        d = tempfile.mkdtemp()
        self.addCleanup(lambda: __import__("shutil").rmtree(d, ignore_errors=True))
        module.write(d, seed, **kw)
        return d

    def test_same_seed_same_files_other_seed_other_files(self):
        a, b, c = self.write(hd, 1), self.write(hd, 1), self.write(hd, 2)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv", "README.txt"):
            self.assertEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(b, name)), name)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv"):
            self.assertNotEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(c, name)), name)

    def test_the_files_parse_in_the_existing_format(self):
        for seed in (1, 2, 3):
            d = self.write(hd, seed)
            lessons = read_rows(os.path.join(d, "lessons.tsv"), 5)
            stream = read_rows(os.path.join(d, "stream.tsv"), 5)
            self.assertEqual([r[0] for r in lessons], [f"L{i:05d}" for i in range(len(lessons))])
            self.assertEqual([r[0] for r in stream], [f"S{i:05d}" for i in range(len(stream))])
            for answer in [r[2] for r in lessons] + [r[4] for r in stream]:
                self.assertTrue(re.fullmatch(r"\d+", answer), answer)
            units = {r[3] for r in lessons}
            self.assertEqual(len(units), 22)
            self.assertEqual({r[1] for r in stream}, {"newop", "compose", "control", "unexplainable"})
            self.assertTrue({r[2] for r in stream} <= units)
            # every fact is taught in three phrasings, all with the same answer, and the fact column is the same for the three
            facts = {}
            for r in lessons:
                facts.setdefault(r[4], []).append(r)
            for rows in facts.values():
                self.assertEqual(len(rows), 3)
                self.assertEqual(len({(x[2], x[3]) for x in rows}), 1)
            # the stream asks nothing that was taught, nothing twice, and the question names the kind's word
            taught = {r[1] for r in lessons}
            asked = [r[3] for r in stream]
            self.assertEqual(len(set(asked)), len(asked))
            self.assertFalse(taught & set(asked))
            for r in stream:
                self.assertIn(r[2], r[3].split(" "))
                self.assertTrue(re.fullmatch(r"(what is )?(\d+ )?[a-z]+ruvo( \d+)?( equals)?", r[3]), r[3])

    def test_the_key_never_reaches_the_exam_files(self):
        for seed in (1, 2, 3):
            d = self.write(hd, seed)
            key = read_rows(os.path.join(d, "key.tsv"), 5)
            exam = "".join(read_bytes(os.path.join(d, name)).decode("utf-8") for name in ("lessons.tsv", "stream.tsv", "README.txt"))
            for word, role, name, rule, learn in key:
                self.assertTrue(rule and learn)
                self.assertNotIn(name, exam, name)
                self.assertNotIn(rule, exam, name)
                self.assertNotIn(learn, exam, name)
            # what the exam files hold of a kind is its word (the key column that names it) and numbers; no other letters but the fixed phrases
            words = {r[0] for r in key}
            letters = re.sub(r"\b(what|is|equals)\b", "", exam)
            letters = re.sub(r"\bL\d{5}\b|\bS\d{5}\b|\bF\d+\b", "", letters)
            for token in re.findall(r"[a-z]+ruvo", letters):
                self.assertIn(token, words)
            files = [f for f in os.listdir(d)]
            self.assertEqual(sorted(files), ["README.txt", "key.tsv", "lessons.tsv", "stream.tsv"])

    def test_the_kinds_by_role(self):
        d = self.write(hd, 1)
        key = read_rows(os.path.join(d, "key.tsv"), 5)
        counts = {}
        for r in key:
            counts[r[1]] = counts.get(r[1], 0) + 1
        self.assertEqual(counts, {"control": 4, "newop": 11, "compose": 4, "unexplainable": 3})

    def test_the_words_are_this_sets_own(self):
        for seed in range(1, 11):
            mine = {r[0] for r in read_rows(os.path.join(self.write(hd, seed), "key.tsv"), 5)}
            self.assertEqual(len(mine), 22)
            for module, columns in ((first, 4), (second, 4), (third, 4)):
                for other in range(1, 11):
                    theirs = {r[0] for r in read_rows(os.path.join(self.write(module, other), "key.tsv"), columns)}
                    self.assertFalse(mine & theirs, (seed, other))
            self.assertTrue(all(w.endswith("ruvo") for w in mine))

    def test_no_kind_is_one_of_the_earlier_sets(self):
        # by name, and by table: none of this set's kinds equals any earlier kind (the controls and the arbitrary codes included) on its whole domain
        for seed in (1, 2, 3):
            earlier = [k for module in (first, second, third) for group in module.make_kinds(seed) for k in group]
            names = {k.name for k in earlier}
            tables = {(k.arity, k.lo, k.ordered, table(k)) for k in earlier}
            for group in hd.make_kinds(seed):
                for k in group:
                    self.assertNotIn(k.name, names)
                    self.assertNotIn((k.arity, k.lo, k.ordered, table(k)), tables, (seed, k.name))

    def test_answers_agree_with_the_rules(self):
        for seed in (1, 2, 3):
            d = self.write(hd, seed)
            key = {r[0]: r[2] for r in read_rows(os.path.join(d, "key.tsv"), 5)}
            kinds = {k.name: k for group in hd.make_kinds(seed) for k in group}
            for name in ("lessons.tsv", "stream.tsv"):
                for r in read_rows(os.path.join(d, name), 5):
                    unit, q, truth = (r[3], r[1], r[2]) if name == "lessons.tsv" else (r[2], r[3], r[4])
                    nums = [int(x) for x in re.findall(r"\d+", q)]
                    k = kinds[key[unit]]
                    self.assertEqual(len(nums), k.arity, q)
                    self.assertTrue(all(k.lo <= n <= 31 for n in nums), q)
                    if k.ordered:
                        self.assertGreaterEqual(nums[0], nums[1], q)
                    args = nums + [0] * (2 - len(nums))
                    self.assertEqual(str(k.fn(*args)), truth, q)

    def test_the_operations_against_known_values(self):
        self.assertEqual([hd.golomb(n) for n in range(1, 16)], [1, 2, 2, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6, 6])
        self.assertEqual([hd.rowland(n, 7) for n in range(1, 25)], [7, 8, 9, 10, 15, 18, 19, 20, 21, 22, 33, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 69, 72])
        self.assertEqual([hd.zeckendorf_bits(n) for n in (0, 1, 2, 3, 4, 5, 6, 12, 31)], [0, 1, 2, 4, 5, 8, 9, 21, 82])
        self.assertEqual([hd.euclid_steps(a, b) for a, b in ((21, 13), (12, 18), (5, 5), (7, 1), (1, 7))], [6, 3, 1, 1, 2])
        self.assertEqual([hd.triangle_part(n) for n in (0, 1, 2, 3, 4, 5, 6, 7, 9, 10)], [0, 1, 1, 2, 1, 2, 3, 1, 3, 4])
        self.assertEqual([hd.evil_branch(n, 4) for n in (0, 1, 2, 3, 4, 7)], [4, 0, 1, 13, 2, 3])
        self.assertEqual([hd.quad_iter(n, 2, 1, 7) for n in range(6)], [2, 5, 5, 5, 5, 5])
        self.assertEqual([hd.crt(a, b, 5, 8) for a, b in ((0, 0), (1, 0), (0, 1), (3, 6), (31, 30))], [0, 16, 25, 38, 6])

    def test_the_rules_against_second_implementations(self):
        # Golomb: n occurs G(n) times, and the sequence never decreases
        seq = [hd.golomb(n) for n in range(1, 400)]
        self.assertEqual(seq, sorted(seq))
        for v in range(1, seq[-1]):
            self.assertEqual(seq.count(v), hd.golomb(v))
        # Rowland, written as a recursion; the iterated square, written as a list
        def r(n, s):
            return s if n == 1 else r(n - 1, s) + math.gcd(n, r(n - 1, s))

        for s in (7, 8, 9, 11, 13):
            for n in range(1, 14):
                self.assertEqual(hd.rowland(n, s), r(n, s))
        xs = [3]
        for _ in range(40):
            xs.append((xs[-1] ** 2 + 5) % 31)
        self.assertEqual([hd.quad_iter(n, 3, 5, 31) for n in range(41)], xs)
        # Zeckendorf: the digits are non-adjacent, and the Fibonacci numbers they pick add up to a
        fibs = [1, 2, 3, 5, 8, 13, 21, 34, 55, 89]
        for a in range(0, 90):
            bits = hd.zeckendorf_bits(a)
            self.assertEqual(bits & (bits >> 1), 0)
            self.assertEqual(sum(f for i, f in enumerate(fibs) if bits >> i & 1), a)
        # Euclid: the last non-zero remainder is the gcd, and the step count is the length of the chain of remainders
        for a in range(1, 40):
            for b in range(1, 40):
                chain, x, y = [], a, b
                while y:
                    chain.append(y)
                    x, y = y, x % y
                self.assertEqual(hd.euclid_steps(a, b), len(chain))
                self.assertEqual(chain[-1], math.gcd(a, b))
        # triangular numbers by listing them
        tri = [k * (k + 1) // 2 for k in range(12)]
        for a in range(0, 60):
            below = max(t for t in tri if t <= a)
            expect = tri.index(a) if a in tri else a - below
            self.assertEqual(hd.triangle_part(a), expect)
        # the residue: the congruences hold, the answer is below p * q, and every pair of residues is hit once
        for p, q in ((5, 8), (7, 9), (9, 11)):
            seen = set()
            for a in range(p):
                for b in range(q):
                    x = hd.crt(a, b, p, q)
                    self.assertEqual((x % p, x % q), (a, b))
                    seen.add(x)
            self.assertEqual(len(seen), p * q)

    def test_the_large_constants_are_large(self):
        # the scaled floor has no short form over the small constants, in any seed (the full check is the 7-node search in the notes)
        for seed in range(1, 21):
            kinds = {k.name: k for group in hd.make_kinds(seed) for k in group}
            self.assertFalse(hd.near_small_scaled_floor([kinds["bigfloor"].fn(a, 0) for a in range(32)]), seed)
            self.assertIn(re.findall(r"\d+", kinds["bigfloor"].rule)[0], map(str, range(41, 64)))

    def test_no_kind_is_constant_or_nearly_so(self):
        for seed in range(1, 21):
            for group in hd.make_kinds(seed):
                for k in group:
                    self.assertGreaterEqual(len(set(table(k))), 5, (seed, k.name))

    def test_the_orbit_of_the_iterated_square_does_not_fall_into_a_short_cycle(self):
        for seed in range(1, 21):
            kinds = {k.name: k for group in hd.make_kinds(seed) for k in group}
            self.assertGreaterEqual(len(set(table(kinds["squareorbit"]))), 14, seed)

    def test_compose_first_changes_only_the_teaching_order(self):
        a, b = self.write(hd, 3), self.write(hd, 3, compose_first=True)
        la, lb = read_rows(os.path.join(a, "lessons.tsv"), 5), read_rows(os.path.join(b, "lessons.tsv"), 5)
        self.assertNotEqual(la, lb)
        self.assertEqual(len(la), len(lb))
        ka, kb = read_rows(os.path.join(a, "key.tsv"), 5), read_rows(os.path.join(b, "key.tsv"), 5)
        roles = [r[1] for r in kb]
        self.assertLess(roles.index("compose"), roles.index("newop"))
        self.assertLess([r[1] for r in ka].index("newop"), [r[1] for r in ka].index("compose"))


if __name__ == "__main__":
    unittest.main()
