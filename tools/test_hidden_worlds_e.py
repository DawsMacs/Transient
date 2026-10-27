# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""Tests for tools/hidden_worlds_e.py: it is deterministic, writes the format `baby exam adapt-write` writes, the key never reaches the exam files,
and every answer follows from its rule.

    python3 -m unittest discover -s tools -p 'test_hidden_worlds_e.py'
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
import hidden_worlds_e as he

EARLIER = (first, second, third, fourth)


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


class HiddenWorldsE(unittest.TestCase):
    def write(self, module, seed, **kw):
        d = tempfile.mkdtemp()
        self.addCleanup(lambda: __import__("shutil").rmtree(d, ignore_errors=True))
        module.write(d, seed, **kw)
        return d

    def test_same_seed_same_files_other_seed_other_files(self):
        a, b, c = self.write(he, 1), self.write(he, 1), self.write(he, 2)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv", "README.txt"):
            self.assertEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(b, name)), name)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv"):
            self.assertNotEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(c, name)), name)

    def test_the_files_parse_in_the_existing_format(self):
        for seed in (1, 2, 3):
            d = self.write(he, seed)
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
                self.assertTrue(re.fullmatch(r"(what is )?(\d+ )?[a-z]+zane( \d+)?( equals)?", r[3]), r[3])

    def test_the_key_never_reaches_the_exam_files(self):
        for seed in (1, 2, 3):
            d = self.write(he, seed)
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
            for token in re.findall(r"[a-z]+zane", letters):
                self.assertIn(token, words)
            files = [f for f in os.listdir(d)]
            self.assertEqual(sorted(files), ["README.txt", "key.tsv", "lessons.tsv", "stream.tsv"])

    def test_the_kinds_by_role(self):
        d = self.write(he, 1)
        key = read_rows(os.path.join(d, "key.tsv"), 5)
        counts = {}
        for r in key:
            counts[r[1]] = counts.get(r[1], 0) + 1
        self.assertEqual(counts, {"control": 4, "newop": 11, "compose": 4, "unexplainable": 3})
        stream = read_rows(os.path.join(d, "stream.tsv"), 5)
        per_role = {}
        for r in stream:
            per_role[r[1]] = per_role.get(r[1], 0) + 1
        self.assertEqual(per_role, {"control": 72, "newop": 186, "compose": 56, "unexplainable": 50})

    def test_the_words_are_this_sets_own(self):
        for seed in range(1, 11):
            mine = {r[0] for r in read_rows(os.path.join(self.write(he, seed), "key.tsv"), 5)}
            self.assertEqual(len(mine), 22)
            for module, columns in ((first, 4), (second, 4), (third, 4), (fourth, 5)):
                for other in range(1, 11):
                    theirs = {r[0] for r in read_rows(os.path.join(self.write(module, other), "key.tsv"), columns)}
                    self.assertFalse(mine & theirs, (seed, other))
            self.assertTrue(all(w.endswith("zane") for w in mine))

    def test_no_kind_is_one_of_the_earlier_sets(self):
        # by name, and by table: none of this set's kinds equals any earlier kind (the controls and the arbitrary codes included) on its whole domain
        earlier = [k for module in EARLIER for seed in range(1, 6) for group in module.make_kinds(seed) for k in group]
        names = {k.name for k in earlier}
        tables = {(k.arity, k.lo, k.ordered, table(k)) for k in earlier}
        for seed in range(1, 6):
            for group in he.make_kinds(seed):
                for k in group:
                    self.assertNotIn(k.name, names)
                    self.assertNotIn((k.arity, k.lo, k.ordered, table(k)), tables, (seed, k.name))

    def test_answers_agree_with_the_rules(self):
        for seed in (1, 2, 3):
            d = self.write(he, seed)
            key = {r[0]: r[2] for r in read_rows(os.path.join(d, "key.tsv"), 5)}
            kinds = {k.name: k for group in he.make_kinds(seed) for k in group}
            for name in ("lessons.tsv", "stream.tsv"):
                for r in read_rows(os.path.join(d, name), 5):
                    unit, q, truth = (r[3], r[1], r[2]) if name == "lessons.tsv" else (r[2], r[3], r[4])
                    nums = [int(x) for x in re.findall(r"\d+", q)]
                    k = kinds[key[unit]]
                    self.assertEqual(len(nums), k.arity, q)
                    self.assertTrue(all(k.lo <= n <= 31 for n in nums), q)
                    args = nums + [0] * (2 - len(nums))
                    self.assertEqual(str(k.fn(*args)), truth, q)

    def test_the_operations_against_known_values(self):
        self.assertEqual([he.prod_rec(n, 3, 4, 37) for n in range(8)], [3, 4, 14, 22, 16, 24, 20, 6])
        self.assertEqual([he.look_back(n, 3) for n in range(12)], [1, 2, 3, 5, 7, 9, 12, 15, 18, 23, 28, 33])
        self.assertEqual([he.stern(n, 0) for n in range(20)], [0, 1, 1, 2, 1, 3, 2, 3, 1, 4, 3, 5, 2, 5, 3, 4, 1, 5, 4, 7])
        self.assertEqual([he.nth_smooth(n, (2, 3, 5)) for n in range(1, 16)], [1, 2, 3, 4, 5, 6, 8, 9, 10, 12, 15, 16, 18, 20, 24])
        self.assertEqual([he.carries(a, b, base) for a, b, base in ((7, 1, 2), (5, 5, 10), (99, 1, 10), (4, 4, 5), (2, 2, 3))], [3, 1, 2, 1, 1])
        self.assertEqual([he.inverse_gray(n) for n in range(8)], [0, 1, 3, 2, 7, 6, 4, 5])
        self.assertEqual([he.edit_distance(a, b) for a, b in ((5, 2), (0, 31), (6, 6), (1, 2), (8, 7))], [1, 5, 0, 1, 3])
        self.assertEqual([he.frac_digit(n, 1, 7) for n in range(1, 9)], [1, 4, 2, 8, 5, 7, 1, 4])
        self.assertEqual([he.frac_digit(n, 22, 7) for n in range(1, 7)], [1, 4, 2, 8, 5, 7])
        self.assertEqual([he.floor_sum(n, 100) for n in range(6)], [0, 100, 150, 183, 208, 228])
        self.assertEqual([he.coprime_branch(a, b, 2) for a, b in ((3, 4), (4, 6), (1, 1), (9, 3), (7, 14))], [9, 2, 4, 6, 7])
        self.assertEqual([he.grid_path(a, b, 1, 1, 5) for a, b in ((0, 0), (1, 0), (1, 1), (2, 2))], [1, 3, 7, 11])

    def test_the_rules_against_second_implementations(self):
        # the recurrences, written as recursions
        def rec_x(n, x0, x1, m):
            return (x0, x1)[n] if n < 2 else (rec_x(n - 1, x0, x1, m) * rec_x(n - 2, x0, x1, m) + n) % m

        def rec_h(n, k):
            return 1 if n == 0 else rec_h(n - 1, k) + rec_h(n // k, k)

        for x0, x1, m in ((2, 7, 29), (6, 5, 71)):
            for n in range(14):
                self.assertEqual(he.prod_rec(n, x0, x1, m), rec_x(n, x0, x1, m))
        for k in (3, 4, 5, 6):
            for n in range(40):
                self.assertEqual(he.look_back(n, k), rec_h(n, k))
        # the diatomic sequence: neighbors are coprime, and the row from 2^k to 2^(k+1) - 1 adds to 3^k
        for n in range(1, 200):
            self.assertEqual(math.gcd(he.stern(n, 0), he.stern(n + 1, 0)), 1)
        for k in range(8):
            self.assertEqual(sum(he.stern(n, 0) for n in range(2**k, 2 ** (k + 1))), 3**k)
        # smooth numbers by trial division
        for primes in he.SMOOTH_SETS:
            listed = [n for n in range(1, 3000) if factor_set(n) <= set(primes)]
            self.assertEqual([he.nth_smooth(i + 1, primes) for i in range(40)], listed[:40])
        # carries by Kummer's rule: the digit sums of a and b exceed that of a + b by (base - 1) for every carry
        def digit_sum(n, base):
            return sum(int(c) for c in digits(n, base))

        for base in (2, 3, 4):
            for a in range(0, 40):
                for b in range(0, 40):
                    self.assertEqual((base - 1) * he.carries(a, b, base), digit_sum(a, base) + digit_sum(b, base) - digit_sum(a + b, base))
        # the Gray code undoes it
        for a in range(0, 64):
            g = he.inverse_gray(a)
            self.assertEqual(g ^ (g >> 1), a)
        # the edit distance as a memoized recursion; symmetric, zero only on equal numbers, at least the difference of lengths
        def lev(x, y):
            memo = {}

            def go(i, j):
                if (i, j) not in memo:
                    if i == 0 or j == 0:
                        memo[(i, j)] = i + j
                    else:
                        memo[(i, j)] = min(go(i - 1, j) + 1, go(i, j - 1) + 1, go(i - 1, j - 1) + (x[i - 1] != y[j - 1]))
                return memo[(i, j)]

            return go(len(x), len(y))

        for a in range(0, 32):
            for b in range(0, 32):
                d = he.edit_distance(a, b)
                self.assertEqual(d, lev(format(a, "b"), format(b, "b")))
                self.assertEqual(d, he.edit_distance(b, a))
                self.assertEqual(d == 0, a == b)
                self.assertGreaterEqual(d, abs(len(format(a, "b")) - len(format(b, "b"))))
        # the digits after the point by long division
        for p, q in ((355, 113), (3, 61), (50, 131)):
            r, out = p % q, []
            for _ in range(30):
                r *= 10
                out.append(r // q)
                r %= q
            self.assertEqual([he.frac_digit(n, p, q) for n in range(1, 31)], out)
        # the sum of floors counts the pairs (k, m) with k * m <= n and k <= a
        for n in (7, 100, 333):
            for a in range(0, 20):
                self.assertEqual(he.floor_sum(a, n), sum(1 for k in range(1, a + 1) for m in range(1, n + 1) if k * m <= n))
        # the cheapest path by trying every path
        for u, v, m in ((1, 1, 5), (2, 3, 7)):
            for a in range(0, 5):
                for b in range(0, 5):
                    best = None
                    for order in set(itertools.permutations([0] * a + [1] * b)):
                        i = j = 0
                        total = ((i + u) * (j + v)) % m
                        for step in order:
                            i, j = (i + 1, j) if step == 0 else (i, j + 1)
                            total += ((i + u) * (j + v)) % m
                        best = total if best is None else min(best, total)
                    self.assertEqual(he.grid_path(a, b, u, v, m), best)

    def test_the_decimal_primes_have_the_longest_period(self):
        for q in he.FULL_REPTEND:
            self.assertTrue(all(q % d for d in range(2, q)))
            self.assertEqual(min(k for k in range(1, q) if pow(10, k, q) == 1), q - 1)

    def test_no_kind_is_constant_or_nearly_so(self):
        for seed in range(1, 21):
            for group in he.make_kinds(seed):
                for k in group:
                    if k.name == "tenthdigit":
                        self.assertGreaterEqual(len(set(table(k))), 7, (seed, k.name))
                    elif k.name != "noise7":
                        self.assertGreaterEqual(len(set(table(k))), 5, (seed, k.name))

    def test_the_nonlinear_recurrence_does_not_fall_into_a_short_cycle(self):
        for seed in range(1, 21):
            kinds = {k.name: k for group in he.make_kinds(seed) for k in group}
            self.assertGreaterEqual(len(set(table(kinds["prodrec"]))), 12, seed)

    def test_the_large_constants_are_large(self):
        for seed in range(1, 21):
            kinds = {k.name: k for group in he.make_kinds(seed) for k in group}
            n = int(re.findall(r"\d+", kinds["harmonicsum"].rule)[0])
            self.assertGreaterEqual(n, 151)
            self.assertNotIn(n, he.SMALL)

    def test_the_arbitrary_codes_are_not_a_pattern(self):
        for seed in (1, 2, 3):
            kinds = {k.name: k for group in he.make_kinds(seed) for k in group}
            self.assertEqual(kinds["noisepair30"].arity, 2)
            cells = [kinds["noisepair30"].fn(a, b) for a in range(32) for b in range(32)]
            self.assertGreaterEqual(len(set(cells)), 25)
            self.assertTrue(max(cells) < 30 and max(kinds["noise7"].fn(a, 0) for a in range(32)) < 7)
            # no row of the pair code repeats another or counts up by a constant
            rows = {tuple(kinds["noisepair30"].fn(a, b) for b in range(32)) for a in range(32)}
            self.assertEqual(len(rows), 32)

    def test_compose_first_changes_only_the_teaching_order(self):
        a, b = self.write(he, 3), self.write(he, 3, compose_first=True)
        la, lb = read_rows(os.path.join(a, "lessons.tsv"), 5), read_rows(os.path.join(b, "lessons.tsv"), 5)
        self.assertNotEqual(la, lb)
        self.assertEqual(len(la), len(lb))
        ka, kb = read_rows(os.path.join(a, "key.tsv"), 5), read_rows(os.path.join(b, "key.tsv"), 5)
        roles = [r[1] for r in kb]
        self.assertLess(roles.index("compose"), roles.index("newop"))
        self.assertLess([r[1] for r in ka].index("newop"), [r[1] for r in ka].index("compose"))


def digits(n, base):
    out = ""
    while n:
        out = str(n % base) + out
        n //= base
    return out or "0"


def factor_set(n):
    """The set of primes that divide n, by trial division."""
    out, d = set(), 2
    while n > 1:
        if n % d == 0:
            out.add(d)
            while n % d == 0:
                n //= d
        d += 1
    return out


if __name__ == "__main__":
    unittest.main()
