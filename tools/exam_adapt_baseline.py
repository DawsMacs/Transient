#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""A deliberately naive system for the adaptive exam, speaking its line protocol on standard input/output, in a different language from the baby.

It memorises every lesson and every answer it is told, word for word, states a fact when a question is an exact repeat of one it has seen, and refuses
everything else. It never learns a rule, so it does not adapt: it shows how a foreign system plugs in, and it is the floor the climb across the stream
is measured against.

    baby exam adapt-drive DIR python3 tools/exam_adapt_baseline.py
"""
import sys

memory = {}
questions = {}
for raw in sys.stdin:
    parts = raw.rstrip("\n").split("\t")
    kind = parts[0]
    if kind == "TEACH" and len(parts) >= 4:
        memory[parts[2].strip().lower()] = parts[3]
    elif kind == "ASK" and len(parts) >= 3:
        qid, question = parts[1], parts[2].strip().lower()
        questions[qid] = question
        if question in memory:
            print(f"ANSWER\t{qid}\tfact\t{memory[question]}\t", flush=True)
        else:
            print(f"ANSWER\t{qid}\trefuse\t\t", flush=True)
    elif kind == "TOLD" and len(parts) >= 3:
        if parts[1] in questions:
            memory[questions[parts[1]]] = parts[2]
    elif kind == "QUIT":
        break
