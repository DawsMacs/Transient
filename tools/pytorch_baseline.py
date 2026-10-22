#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 DawsMacs
"""A conventional PyTorch transformer baseline for `baby compare`.

Why this exists: the transformer in baby/src/transformer.rs is plain Rust loops with one fixed recipe and no tuning. A fair comparison needs
the industry's own tools, tuned. This script trains a standard PyTorch pre-LayerNorm encoder (nn.TransformerEncoder) on EXACTLY the lessons the
baby was taught and grades it on exactly the same four sets of questions (export them with `baby compare --export DIR [--units N]`).

Rules that keep it honest:
  * Settings are chosen by a random search scored ONLY on facts held out of the training lessons (all phrasings of 20 % of the facts), never
    on the reworded / never-taught / beyond sets. The baby also picks its way of learning from facts it holds back.
  * The cost of the search (time, and joules if the CPU counter is readable) is reported next to the cost of the final training. The baby has no
    search to pay for.
  * Vocabulary and answers come only from the training lessons, as in the Rust comparison.
Run it with the virtual environment that has torch (CPU build is enough for these sizes):  python tools/pytorch_baseline.py DIR --trials 24
"""
import argparse
import json
import random
import time

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

RAPL = "/sys/class/powercap/intel-rapl:0"


def tokenize(text):
    return "".join(c if c.isalnum() else " " for c in text.lower()).split()


def cpu_seconds():
    """(busy CPU seconds of the whole machine, this process's CPU seconds)."""
    f = open("/proc/stat").readline().split()[1:]
    n = [float(x) for x in f]
    busy = (sum(n) - n[3] - n[4]) / 100.0
    return busy, time.process_time()


class Meter:
    """CPU package energy above idle, if the kernel lets this user read the counter. It is the whole package, so whatever else the machine does
    is counted too. The idle baseline is re-measured right before every measurement (the lowest of three one-second readings; a baseline taken
    once goes stale when the background changes, and clamped later readings to 0.00 J), and every reading records how much of the busy CPU time
    went to OTHER processes, so a reading on a busy machine is flagged. The energy is not scaled by CPU share (that under-counted)."""

    def __init__(self):
        self.ok = False
        self.worst_other = 0.0
        try:
            self.max_uj = int(open(f"{RAPL}/max_energy_range_uj").read())
            self.read()
            self.ok = True
        except Exception:
            pass

    def read(self):
        return int(open(f"{RAPL}/energy_uj").read())

    def idle_w(self):
        lows = []
        for _ in range(3):
            e0 = self.read()
            time.sleep(1)
            lows.append((self.read() - e0) / 1e6)
        return min(lows)

    def start(self):
        idle = self.idle_w() if self.ok else 0.0
        return (time.time(), self.read() if self.ok else 0, cpu_seconds(), idle)

    def stop(self, mark):
        secs = time.time() - mark[0]
        b1, m1 = cpu_seconds()
        busy = b1 - mark[2][0]
        if busy > 0.05 and secs > 0.1:
            self.worst_other = max(self.worst_other, min(max((busy - (m1 - mark[2][1])) / busy, 0.0), 1.0))
        if not self.ok:
            return secs, None
        d = self.read() - mark[1]
        if d < 0:
            d += self.max_uj
        return secs, max(d / 1e6 - mark[3] * secs, 0.0)

    def warning(self):
        if self.ok and self.worst_other > 0.25:
            return f"WARNING: other processes used up to {100 * self.worst_other:.0f}% of the busy CPU time during some measurements, so the energies include their share (an upper bound); for clean figures rerun with the machine quiet."
        return None


def load(dirname, name):
    rows = []
    for line in open(f"{dirname}/{name}.tsv", encoding="utf-8"):
        parts = line.rstrip("\n").split("\t")
        if len(parts) >= 4:
            rows.append((parts[0], parts[1], int(parts[2]), int(parts[3])))
    return rows


class Lexicon:
    def __init__(self, train):
        self.words, self.classes, self.longest = {}, [], 1
        for q, a, _, _ in train:
            toks = tokenize(q)
            self.longest = max(self.longest, len(toks))
            for w in toks:
                self.words.setdefault(w, len(self.words) + 1)
            if a not in self.classes:
                self.classes.append(a)
        self.class_id = {c: i for i, c in enumerate(self.classes)}

    def encode(self, q):
        return [self.words.get(w, 0) for w in tokenize(q)][: self.longest] or [0]

    def tensors(self, rows):
        seqs = [self.encode(r[0]) for r in rows]
        t = max(len(s) for s in seqs)
        ids = torch.zeros(len(rows), t, dtype=torch.long)
        mask = torch.ones(len(rows), t, dtype=torch.bool)  # True = padding
        for i, s in enumerate(seqs):
            ids[i, : len(s)] = torch.tensor(s)
            mask[i, : len(s)] = False
        y = torch.tensor([self.class_id.get(r[1], -1) for r in rows], dtype=torch.long)
        return ids, mask, y


class Net(nn.Module):
    def __init__(self, vocab, classes, d, heads, layers, ff, maxlen, dropout):
        super().__init__()
        self.emb = nn.Embedding(vocab, d)
        self.pos = nn.Embedding(maxlen, d)
        layer = nn.TransformerEncoderLayer(d, heads, ff, dropout, activation="relu", batch_first=True, norm_first=True)
        self.enc = nn.TransformerEncoder(layer, layers, enable_nested_tensor=False)
        self.ln = nn.LayerNorm(d)
        self.out = nn.Linear(d, classes)

    def forward(self, ids, mask):
        x = self.emb(ids) + self.pos(torch.arange(ids.shape[1]))
        x = self.ln(self.enc(x, src_key_padding_mask=mask))
        keep = (~mask).unsqueeze(-1).float()
        return self.out((x * keep).sum(1) / keep.sum(1))


@torch.no_grad()
def predict(model, data):
    model.eval()
    ids, mask, _ = data
    p = F.softmax(model(ids, mask), dim=-1)
    conf, best = p.max(dim=-1)
    return best, conf


def accuracy(model, data):
    best, _ = predict(model, data)
    return (best == data[2]).float().mean().item()


def train(model, data, lr, wd, batch, epochs, seed):
    """Train until every training lesson is answered right (checked every 5 epochs) or the epochs run out. Returns the epochs used."""
    g = torch.Generator().manual_seed(seed)
    opt = torch.optim.AdamW(model.parameters(), lr=lr, weight_decay=wd)
    ids, mask, y = data
    n = ids.shape[0]
    for epoch in range(1, epochs + 1):
        model.train()
        order = torch.randperm(n, generator=g)
        for i in range(0, n, batch):
            idx = order[i : i + batch]
            opt.zero_grad()
            loss = F.cross_entropy(model(ids[idx], mask[idx]), y[idx])
            loss.backward()
            opt.step()
        if epoch % 5 == 0 and accuracy(model, data) == 1.0:
            return epoch
    return epochs


def make(cfg, lex, seed):
    torch.manual_seed(seed)
    return Net(len(lex.words) + 1, len(lex.classes), cfg["d"], cfg["heads"], cfg["layers"], cfg["ff"], lex.longest, cfg["dropout"])


def params(model):
    return sum(p.numel() for p in model.parameters())


def grade(model, lex, rows, threshold):
    data = lex.tensors(rows)
    best, conf = predict(model, data)
    right = refused = wrong = 0
    for b, c, y in zip(best.tolist(), conf.tolist(), data[2].tolist()):
        if c < threshold:
            refused += 1
        elif b == y:
            right += 1
        else:
            wrong += 1
    n = max(len(rows), 1)
    return 100 * right / n, 100 * refused / n, 100 * wrong / n


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dir")
    ap.add_argument("--trials", type=int, default=24)
    ap.add_argument("--epochs", type=int, default=300)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--threads", type=int, default=2)
    ap.add_argument("--out", default="")
    a = ap.parse_args()
    torch.set_num_threads(a.threads)
    rng = random.Random(a.seed)
    meter = Meter()

    train_rows = load(a.dir, "train")
    sets = {name: load(a.dir, f) for name, f in [("taught", "train"), ("reworded", "reworded"), ("never taught", "never"), ("beyond", "beyond")]}
    lex = Lexicon(train_rows)
    print(f"{len(train_rows)} lessons, {len(lex.words)} words, {len(lex.classes)} answers; energy is {'MEASURED' if meter.ok else 'not measured'}")

    # a validation split of FACTS (all phrasings of 20 % of them), carved from the training lessons only
    facts = sorted({r[3] for r in train_rows})
    rng.shuffle(facts)
    held = set(facts[: max(1, len(facts) // 5)])
    fit_rows = [r for r in train_rows if r[3] not in held]
    val_rows = [r for r in train_rows if r[3] in held]
    fit_data, val_data = lex.tensors(fit_rows), lex.tensors(val_rows)

    space = {"d": [32, 64, 128], "layers": [1, 2], "lr": [1e-3, 3e-3, 1e-2], "wd": [0.0, 0.01, 0.1], "dropout": [0.0, 0.1, 0.2], "ffmul": [2, 4], "batch": [16, 32]}
    trials, seen = [], set()
    while len(trials) < a.trials:
        c = {k: rng.choice(v) for k, v in space.items()}
        key = tuple(sorted(c.items()))
        if key not in seen:
            seen.add(key)
            c["heads"] = 4
            c["ff"] = c["ffmul"] * c["d"]
            trials.append(c)

    print(f"\nsearch: {len(trials)} random settings, scored on {len(held)} held-out facts ({len(val_rows)} questions) never used for fitting or for the four test sets")
    mark = meter.start()
    results = []
    for i, c in enumerate(trials):
        t0 = time.time()
        m = make(c, lex, a.seed)
        ep = train(m, fit_data, c["lr"], c["wd"], c["batch"], a.epochs, a.seed)
        va = accuracy(m, val_data)
        results.append((va, params(m), i))
        print(f"  {i + 1:>2}/{len(trials)} d={c['d']:<3} layers={c['layers']} lr={c['lr']:<6} wd={c['wd']:<4} drop={c['dropout']} ff={c['ff']:<3} batch={c['batch']:<2}  epochs={ep:<3}  held-out facts right {100 * va:5.1f}%  ({time.time() - t0:.1f}s)")
    sweep_secs, sweep_j = meter.stop(mark)
    results.sort(key=lambda r: (-r[0], r[1]))
    best = trials[results[0][2]]
    print(f"\nbest setting: {json.dumps({k: v for k, v in best.items() if k != 'ffmul'})}  held-out facts right {100 * results[0][0]:.1f}%")
    print(f"cost of the search: {sweep_secs:.1f} s, {'%.2f J' % sweep_j if sweep_j is not None else 'energy not measured'}")

    # the final model: the best setting, trained on ALL the training lessons
    full = lex.tensors(train_rows)
    mark = meter.start()
    model = make(best, lex, a.seed)
    ep = train(model, full, best["lr"], best["wd"], best["batch"], a.epochs, a.seed)
    secs, j = meter.stop(mark)
    print(f"final training: {params(model)} parameters, {ep} epochs, {secs:.2f} s, {'%.2f J' % j if j is not None else 'energy not measured'}")

    print("\nright / refused / WRONG, percent of each set")
    out = {"params": params(model), "epochs": ep, "train_secs": secs, "train_j": j, "search_secs": sweep_secs, "search_j": sweep_j, "best": {k: v for k, v in best.items() if k != "ffmul"}, "sets": {}}
    print(f"{'':<32}" + "".join(f"{n:>26}" for n in sets))
    for label, thr in [("always answers", 0.0), ("answers if p >= 0.9", 0.9)]:
        row = {n: grade(model, lex, rows, thr) for n, rows in sets.items()}
        out["sets"][label] = row
        print(f"{'pytorch ' + label:<32}" + "".join(f"{'%.1f / %.1f / %.1f' % row[n]:>26}" for n in sets))
    if meter.warning():
        print("\n" + meter.warning())
    out["other_load_share"] = meter.worst_other
    if a.out:
        json.dump(out, open(a.out, "w"), indent=1)


if __name__ == "__main__":
    main()
