# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""Tests for tools/hidden_worlds_c.py: it is deterministic, writes the format `baby exam adapt-write` writes, and every answer follows from its rule.

    python3 -m unittest discover -s tools -p 'test_hidden_worlds_c.py'
"""
import itertools
import os
import re
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import hidden_worlds as first
import hidden_worlds_b as second
import hidden_worlds_c as hc


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


class HiddenWorldsC(unittest.TestCase):
    def write(self, module, seed, **kw):
        d = tempfile.mkdtemp()
        self.addCleanup(lambda: __import__("shutil").rmtree(d, ignore_errors=True))
        module.write(d, seed, **kw)
        return d

    def test_same_seed_same_files_other_seed_other_files(self):
        a, b, c = self.write(hc, 1), self.write(hc, 1), self.write(hc, 2)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv", "README.txt"):
            self.assertEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(b, name)), name)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv"):
            self.assertNotEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(c, name)), name)

    def test_the_files_parse_in_the_existing_format(self):
        for seed in (1, 2, 3):
            d = self.write(hc, seed)
            lessons = read_rows(os.path.join(d, "lessons.tsv"), 5)
            stream = read_rows(os.path.join(d, "stream.tsv"), 5)
            self.assertEqual([r[0] for r in lessons], [f"L{i:05d}" for i in range(len(lessons))])
            self.assertEqual([r[0] for r in stream], [f"S{i:05d}" for i in range(len(stream))])
            for answer in [r[2] for r in lessons] + [r[4] for r in stream]:
                self.assertTrue(re.fullmatch(r"\d+", answer), answer)
            units = {r[3] for r in lessons}
            self.assertEqual(len(units), 23)
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
                self.assertTrue(re.fullmatch(r"(what is )?(\d+ )?[a-z]+ketu( \d+)?( equals)?", r[3]), r[3])

    def test_the_kinds_by_role(self):
        d = self.write(hc, 1)
        key = read_rows(os.path.join(d, "key.tsv"), 4)
        counts = {}
        for r in key:
            counts[r[1]] = counts.get(r[1], 0) + 1
        self.assertEqual(counts, {"control": 4, "newop": 11, "compose": 5, "unexplainable": 3})

    def test_the_words_are_this_sets_own(self):
        for seed in range(1, 11):
            mine = {r[0] for r in read_rows(os.path.join(self.write(hc, seed), "key.tsv"), 4)}
            self.assertEqual(len(mine), 23)
            for module in (first, second):
                for other in range(1, 11):
                    theirs = {r[0] for r in read_rows(os.path.join(self.write(module, other), "key.tsv"), 4)}
                    self.assertFalse(mine & theirs, (seed, other))
            self.assertTrue(all(w.endswith("ketu") and not w.endswith(("sido", "velu")) for w in mine))

    def test_no_kind_is_one_of_the_earlier_sets(self):
        # by name, and by table: none of this set's non-control kinds equals any earlier kind (or the earlier controls) on its whole domain
        # (the arbitrary codes are drawn from the seed and are the same shape in every set on purpose)
        for seed in (1, 2, 3):
            earlier = [k for module in (first, second) for group in module.make_kinds(seed) for k in group]
            names = {k.name for k in earlier}
            tables = {(k.arity, k.lo, k.ordered, table(k)) for k in earlier}
            for group in hc.make_kinds(seed):
                for k in group:
                    self.assertNotIn(k.name, names)
                    self.assertNotIn((k.arity, k.lo, k.ordered, table(k)), tables, (seed, k.name))

    def test_answers_agree_with_the_rules(self):
        for seed in (1, 2, 3):
            d = self.write(hc, seed)
            key = {r[0]: r[2] for r in read_rows(os.path.join(d, "key.tsv"), 4)}
            kinds = {k.name: k for group in hc.make_kinds(seed) for k in group}
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
        self.assertEqual([hc.longest_run(n) for n in (1, 3, 5, 7, 12, 22, 31)], [1, 2, 1, 3, 2, 2, 5])
        self.assertEqual([hc.palindrome_base(n) for n in range(7)], [2, 2, 3, 2, 3, 2, 5])
        self.assertEqual([hc.common_digit5(n) for n in (0, 4, 5, 6, 7, 31)], [0, 4, 0, 1, 1, 1])
        self.assertEqual([hc.multiples_below(n) for n in (0, 4, 6, 10, 16)], [0, 3, 8, 23, 60])
        self.assertEqual([hc.hail_steps(n, 1, 2) for n in (1, 2, 3, 5, 6)], [0, 1, 1, 4, 2])
        self.assertEqual([hc.divisor_sum(n) for n in (1, 2, 6, 12, 28)], [1, 3, 12, 28, 56])
        self.assertEqual([hc.order_of_two(n) for n in range(1, 7)], [2, 4, 3, 6, 10, 12])
        self.assertEqual([hc.derangements_mod(n, 10**9) for n in range(8)], [1, 0, 1, 2, 9, 44, 265, 1854])
        self.assertEqual([hc.tribonacci_mod(n, 10**9) for n in range(10)], [0, 0, 1, 1, 2, 4, 7, 13, 24, 44])
        self.assertEqual([hc.stirling2(4, 2), hc.stirling2(5, 3), hc.stirling2(6, 3), hc.stirling2(7, 1), hc.stirling2(5, 5)], [7, 25, 90, 1, 1])
        self.assertEqual([hc.wythoff(0, 5), hc.wythoff(1, 1), hc.wythoff(2, 2)], [5, 2, 1])
        self.assertEqual([hc.wythoff(a, b) for a, b in ((1, 2), (3, 5), (4, 7), (6, 10), (8, 13))], [0] * 5)

    def test_the_counts_against_brute_force(self):
        # Stirling numbers: count the set partitions by restricted-growth strings
        for n in range(1, 8):
            counts = {}
            for s in itertools.product(range(n), repeat=n):
                if s[0] == 0 and all(s[i] <= max(s[:i]) + 1 for i in range(1, n)):
                    counts[max(s) + 1] = counts.get(max(s) + 1, 0) + 1
            for k in range(1, n + 1):
                self.assertEqual(hc.stirling2(n, k), counts[k], (n, k))
        # derangements: count the permutations with no fixed point
        for n in range(1, 8):
            fixed_free = sum(1 for p in itertools.permutations(range(n)) if all(p[i] != i for i in range(n)))
            self.assertEqual(hc.derangements_mod(n, 10**9), fixed_free)
        # the two-heap game: mex over the moves, written again with a memo and no table
        memo = {}

        def g(a, b):
            if (a, b) not in memo:
                options = {g(a - k, b) for k in range(1, a + 1)} | {g(a, b - k) for k in range(1, b + 1)} | {g(a - k, b - k) for k in range(1, min(a, b) + 1)}
                memo[a, b] = next(v for v in itertools.count() if v not in options)
            return memo[a, b]

        for a, b in itertools.product(range(0, 12), repeat=2):
            self.assertEqual(hc.wythoff(a, b), g(a, b))
        # the order of 2 is the least k with 2^k = 1 modulo 2a + 1; the run length by strings
        for a in range(1, 32):
            k = hc.order_of_two(a)
            self.assertEqual(pow(2, k, 2 * a + 1), 1)
            self.assertTrue(all(pow(2, j, 2 * a + 1) != 1 for j in range(1, k)))
        for a in range(1, 32):
            self.assertEqual(hc.longest_run(a), max(len(m.group()) for m in re.finditer(r"0+|1+", bin(a)[2:])))
        # the base of the first palindrome: no smaller base works, and the base found does
        for a in range(32):
            b = hc.palindrome_base(a)
            self.assertEqual(hc.digits(a, b), hc.digits(a, b)[::-1])
            for c in range(2, b):
                self.assertNotEqual(hc.digits(a, c), hc.digits(a, c)[::-1])

    def test_the_step_rules_stop_for_every_number_in_range(self):
        for p, q in [(1, 2), (1, 3), (1, 4), (2, 2), (2, 4)]:
            for a in range(1, 32):
                self.assertLess(hc.hail_steps(a, p, q), 100, (p, q, a))

    def test_no_kind_is_constant_or_nearly_so(self):
        for seed in range(1, 21):
            for group in hc.make_kinds(seed):
                for k in group:
                    self.assertGreaterEqual(len(set(table(k))), 5, (seed, k.name))

    def test_compose_first_changes_only_the_teaching_order(self):
        a, b = self.write(hc, 3), self.write(hc, 3, compose_first=True)
        la, lb = read_rows(os.path.join(a, "lessons.tsv"), 5), read_rows(os.path.join(b, "lessons.tsv"), 5)
        self.assertNotEqual(la, lb)
        self.assertEqual(len(la), len(lb))
        ka, kb = read_rows(os.path.join(a, "key.tsv"), 4), read_rows(os.path.join(b, "key.tsv"), 4)
        roles = [r[1] for r in kb]
        self.assertLess(roles.index("compose"), roles.index("newop"))
        self.assertLess([r[1] for r in ka].index("newop"), [r[1] for r in ka].index("compose"))


if __name__ == "__main__":
    unittest.main()
