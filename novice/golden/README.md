# Novice goldens

What Novice produces on the reference platform (Linux x86_64, glibc), kept so that every change to its logic that moves an answer, a program it found, or the counted work shows up as a difference. `check.sh` generates the same outputs on the machine running it and compares them with these files.

| File | What it is |
|---|---|
| `static60-seed1.txt` | The static entry exam (`baby exam write DIR --units 60 --seed 1`): the judge's table, then Novice's predictions (`novice static`). |
| `adapt-seed1.txt` | The adaptive entry exam (`baby exam adapt-write DIR --seed 1`) driven with `novice serve`: the judge's table, Novice's account of its work (kinds explained, the programs it found, counted evaluations), then every reply. |
| `hidden-a-seed1.txt` to `hidden-j-seed1.txt` | The same for the ten sets of hidden worlds (`tools/hidden_worlds*.py`, seed 1). All ten are development sets. |
| `hidden-i-seed3.txt` | The ninth set, seed 3: the one seed whose number words count in fives, so it pins the reading of a count and its unit word. |
| `PROVENANCE.txt` | Which compiler and machine produced them. Informational; not compared. |

Only deterministic outputs are compared. Timings, memory and joules are never part of a golden (the exams are driven with `EXAM_NO_JOULES=1`).

```bash
novice/golden/check.sh            # generate twice, require the runs to agree, compare with the goldens
novice/golden/check.sh update     # regenerate on purpose; the commit says why
```

Exit status: 0 every golden matches; 1 at least one differs; 2 an output could not be generated; 3 the runs on this machine disagree with each other. A difference is never adopted silently: it is either a platform difference or a logic change that was not followed by an update. A change meant to change answers runs `update` in the same commit and says why. Run `update` on the reference platform only.

## 2026-10-30 publication correction

The Novice 0.0.1 research snapshot removes answer-dependent exclusions from recurrence validation. A reachable held-out error now rejects the recurrence; acquisition can decrease and work or replies can change. Goldens regenerated for this correction pin the stricter behavior. They do not reclassify any already-read exam as blind, and they do not provide an independent outer evaluation of recurrence-shape selection.
