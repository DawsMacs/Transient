# Novice goldens

What Novice produces on the reference platform (Linux x86_64, glibc), kept so that every change to its logic that moves an answer, a program it found, or the counted work shows up as a difference. `check.sh` generates the same outputs on the machine running it and compares them with these files.

| File | What it is |
|---|---|
| `static60-seed1.txt` | The static entry exam (`baby exam write DIR --units 60 --seed 1`): the judge's table, then Novice's predictions (`novice static`). |
| `adapt-seed1.txt` | The adaptive entry exam (`baby exam adapt-write DIR --seed 1`) driven with `novice serve`: the judge's table, Novice's account of its work (kinds explained, the programs it found, counted evaluations), then every reply. |
| `hidden-a-seed1.txt` to `hidden-f-seed1.txt` | The same for the six sets of hidden worlds (`tools/hidden_worlds*.py`, seed 1). All six are development sets. |
| `PROVENANCE.txt` | Which compiler and machine produced them. Informational; not compared. |

Only deterministic outputs are compared. Timings, memory and joules are never part of a golden (the exams are driven with `EXAM_NO_JOULES=1`).

```bash
novice/golden/check.sh            # generate twice, require the runs to agree, compare with the goldens
novice/golden/check.sh update     # regenerate on purpose; the commit says why
```

Exit status: 0 every golden matches; 1 at least one differs; 2 an output could not be generated; 3 the runs on this machine disagree with each other. A difference is never adopted silently: it is either a platform difference or a logic change that was not followed by an update. A change meant to change answers runs `update` in the same commit and says why. Run `update` on the reference platform only.
