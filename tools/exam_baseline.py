#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""A deliberately naive system for the entry exam, in a different language from the baby: it memorises every lesson word for word, states a fact when the
question is an exact repeat of a lesson, and refuses everything else. It exists to show that `baby exam score` judges a foreign system, and to give the
floor that coverage is compared against: a system worth building passes the same honesty rules and covers more.

    python3 tools/exam_baseline.py DIR predictions.tsv
    baby exam score DIR predictions.tsv
"""
import sys

def main(directory, out):
    lessons = {}
    with open(f"{directory}/lessons.tsv", encoding="utf-8") as f:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            if len(parts) >= 3:
                lessons[parts[1].strip().lower()] = parts[2]
    with open(f"{directory}/questions.tsv", encoding="utf-8") as f, open(out, "w", encoding="utf-8") as o:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 3:
                continue
            qid, question = parts[0], parts[2].strip().lower()
            if question in lessons:
                o.write(f"{qid}\tfact\t{lessons[question]}\t\n")
            else:
                o.write(f"{qid}\trefuse\t\t\n")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    main(sys.argv[1], sys.argv[2])
