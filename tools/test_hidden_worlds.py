# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""Tests for tools/hidden_worlds.py: it is deterministic and writes the format `baby exam adapt-write` writes.

    python3 -m unittest discover -s tools -p 'test_hidden_worlds.py'
"""
import os
import re
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import hidden_worlds as hw


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


class HiddenWorlds(unittest.TestCase):
    def write(self, seed, **kw):
        d = tempfile.mkdtemp()
        self.addCleanup(lambda: __import__("shutil").rmtree(d, ignore_errors=True))
        hw.write(d, seed, **kw)
        return d

    def test_same_seed_same_files_other_seed_other_files(self):
        a, b, c = self.write(1), self.write(1), self.write(2)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv", "README.txt"):
            self.assertEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(b, name)), name)
        for name in ("lessons.tsv", "stream.tsv", "key.tsv"):
            self.assertNotEqual(read_bytes(os.path.join(a, name)), read_bytes(os.path.join(c, name)), name)

    def test_the_files_parse_in_the_existing_format(self):
        for seed in (1, 2, 3):
            d = self.write(seed)
            lessons = read_rows(os.path.join(d, "lessons.tsv"), 5)
            stream = read_rows(os.path.join(d, "stream.tsv"), 5)
            self.assertEqual([r[0] for r in lessons], [f"L{i:05d}" for i in range(len(lessons))])
            self.assertEqual([r[0] for r in stream], [f"S{i:05d}" for i in range(len(stream))])
            for answer in [r[2] for r in lessons] + [r[4] for r in stream]:
                self.assertTrue(re.fullmatch(r"\d+", answer), answer)
            units = {r[3] for r in lessons}
            self.assertEqual(len(units), 25)
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
                self.assertTrue(re.fullmatch(r"(what is )?(\d+ )?[a-z]+sido( \d+)?( equals)?", r[3]), r[3])

    def test_the_kinds_by_role(self):
        d = self.write(1)
        key = read_rows(os.path.join(d, "key.tsv"), 4)
        counts = {}
        for r in key:
            counts[r[1]] = counts.get(r[1], 0) + 1
        self.assertEqual(counts, {"control": 4, "newop": 12, "compose": 6, "unexplainable": 3})

    def test_answers_agree_with_the_rules(self):
        for seed in (1, 2):
            d = self.write(seed)
            key = {r[0]: r[2] for r in read_rows(os.path.join(d, "key.tsv"), 4)}
            kinds = {k.name: k for group in hw.make_kinds(seed) for k in group}
            for name in ("lessons.tsv", "stream.tsv"):
                rows = read_rows(os.path.join(d, name), 5)
                for r in rows:
                    unit, q, truth = (r[3], r[1], r[2]) if name == "lessons.tsv" else (r[2], r[3], r[4])
                    nums = [int(x) for x in re.findall(r"\d+", q)]
                    self.assertTrue(all(0 <= n <= 31 for n in nums), q)
                    args = nums + [0] * (2 - len(nums))
                    self.assertEqual(str(kinds[key[unit]].fn(*args)), truth, q)

    def test_compose_first_changes_only_the_teaching_order(self):
        a, b = self.write(3), self.write(3, compose_first=True)
        la, lb = read_rows(os.path.join(a, "lessons.tsv"), 5), read_rows(os.path.join(b, "lessons.tsv"), 5)
        self.assertNotEqual(la, lb)
        self.assertEqual(len(la), len(lb))
        ka, kb = read_rows(os.path.join(a, "key.tsv"), 4), read_rows(os.path.join(b, "key.tsv"), 4)
        roles = [r[1] for r in kb]
        self.assertLess(roles.index("compose"), roles.index("newop"))
        self.assertLess([r[1] for r in ka].index("newop"), [r[1] for r in ka].index("compose"))


if __name__ == "__main__":
    unittest.main()
