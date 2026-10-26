<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 DawsMacs -->
# Hidden worlds D: a fourth set of exam worlds, for rules that no line of the learner names

`tools/hidden_worlds_d.py` writes a fourth family of adaptive exams. The files, the line protocol, the shape of a world (inputs 0 to 31, invented words
for the kinds, three taught phrasings per fact, held-out and beyond-range questions with the truth told after each answer) and the roles are those of
`tools/hidden_worlds.py`, `tools/hidden_worlds_b.py` and `tools/hidden_worlds_c.py` (see `tools/hidden_worlds.md`). The kinds are all different from
those sets.

**Why a fourth set.** Sets A, B and C are development sets once their kinds have been read. This set was written without looking at the learner it will
be run on. **To keep it a blind test, do not read the list of kinds below the marked line (or `make_kinds` in the script) before the first run**: run
it, look at the numbers, then read on.

```bash
python3 tools/hidden_worlds_d.py DIR --seed 1             # writes lessons.tsv, stream.tsv (+ README.txt, key.tsv)
baby exam adapt-drive DIR COMMAND...                      # judge any system on it, as for the clean adaptive exam
baby exam adapt-run DIR                                   # the baby itself
python3 -m unittest discover -s tools -p 'test_hidden_worlds_d.py'
```

`--compose-first` teaches the compositions before the kinds they use. The generator is standard-library Python 3 with the first set's small
deterministic random generator, so a seed gives the same bytes on every Python version. It imports the first generator for the shared pieces; the
kinds, the words and the salts are its own. `key.tsv` has five columns here: the word, the role, the kind, the rule and how the kind could be learned.
It is never sent to a system.

## What differs from the earlier sets (without naming a kind)

- The words are two syllables plus the ending `ruvo` (the earlier sets end in `sido`, `velu` and `ketu`), so no word of one set is a word of another
  (a test checks seeds 1 to 10 against all three).
- No kind of the earlier sets is reused, by name or by table: a test compares every table of this set with every table of the three earlier sets
  (controls and codes included) and finds no equal pair. No kind is a re-parametrization of an earlier one with another base or modulus.
- 22 kinds: 11 `newop`, 4 `compose`, 4 `control`, 3 `unexplainable`; 15 hard kinds in all (`newop` plus `compose`). The stream has 348 questions (178
  newop, 56 compose, 72 control, 42 unexplainable); there are 1701 lessons. Unary kinds teach the numbers up to 19 (four are held out and asked) and ask
  ten above 19; binary kinds teach 60 pairs with both numbers up to 15, hold out 8 and ask 14 pairs with a number above 15 (as in the earlier sets).
  8 of the 11 `newop` kinds, all 4 `compose` kinds and 2 of the 4 controls take one number; the others take two.
- A few hard kinds have a simple rule but large constants, or are a scaled floor; the others are new structure (numbers defined from earlier ones
  in ways the earlier sets did not use, other numeral systems, loops over a pair, conditionals on a property of the input that is not a threshold).
- The controls can be answered from the seen examples by a learner with the starting operations: a refusal on them is a defect. The three
  `unexplainable` kinds have no rule and must stay refused until told.

## What the baby and a memorizer do on it (seeds 1 to 3)

`baby exam adapt-run DIR` (percent of the questions in the role; WRONG was 0.0 % everywhere, in every role and over the whole stream):

| seed | control (72): facts right / guesses right / refused | newop (178) | compose (56) | unexplainable (42) | all (348): facts right |
|---|---|---|---|---|---|
| 1 | 59.7 % / 19.4 % / 20.8 % | all refused | all refused | all refused | 12.4 % (4.0 % guesses right) |
| 2 | 59.7 % / 16.7 % / 23.6 % | all refused | all refused | all refused | 12.4 % (3.4 % guesses right) |
| 3 | 38.9 % / 20.8 % / 40.3 % | all refused | all refused | all refused | 8.0 % (4.3 % guesses right) |

The baby answers only through the controls. It gets no hard kind (0.0 % facts right, 0.0 % guesses right, 100 % refused) and no `unexplainable` row.
Its controls are not all answered as facts: 20.8 %, 23.6 % and 40.3 % of the control questions are refused on the three seeds.

`baby exam adapt-drive DIR python3 tools/exam_adapt_baseline.py` (the naive memorizer, which only repeats told answers): 0.0 % right in every role on all
three seeds, 100 % refused, WRONG 0. No question in the stream repeats a taught or an earlier question, so memorizing cannot score.

## Is any kind reachable with the starting operations? (brute force, 7 nodes)

The check used for the earlier sets, run again on this set with a throwaway search (not kept): every expression of at most 7 nodes (leaves count) over
`+ - * min max div mod`, the variables, and the constants 0 to 40, 50, 64 and 100, deduplicated by value on the sample points, with subtraction both
signed and saturating at 0, and division or remainder by zero taken as 0. Unary kinds were matched on their whole domain (0 or 1 to 31); binary kinds on
60 random pairs with both numbers from 1 to 31 (an exact match on the sample counts as reachable, so the check errs toward calling a kind reachable).
It covered seeds 1 to 10, so every drawn constant: 170 unary targets and 50 binary ones, in both subtraction conventions.

- **Reachable, as intended:** all four controls, in every seed from 1 to 10 and in both conventions (at 3, 3, 3 and 5 nodes).
- **Not reachable at 7 nodes, any seed 1 to 10:** all 11 `newop` kinds, all 4 `compose` kinds and the arbitrary codes. One draft was replaced while
  the set was written: with some drawn constants the scaled floor equals a much simpler fraction on 0 to 31 and was found at 7 nodes, so the generator now
  rejects such draws (and a test checks seeds 1 to 20).
- **The claim is narrow.** Because inputs stop at 31, every kind is a finite table, and some have a longer program; "not reachable at 7 nodes" is the
  claim, not "impossible". A learner may also build operations for itself from loops, tables or conditionals, which are not covered. How each kind could
  be learned is in the `learn` column of `key.tsv`.

## Things to know

- Every answer is exact by construction (the tests recompute each lesson and stream answer from the rules of its kind, check the operations against
  known values, and check them again against second implementations).
- The stream asks each fact once; it has no wrong-teacher columns (this is the clean exam).
- Some tests of the files: same seed gives the same bytes, every line parses, and neither a kind's name, its rule nor its learning note appears in any file
  that is sent to a system.

---

# STOP: do not read below this line before the first run

Everything below names the kinds.

---

## The kinds (22)

| role | kind | rule |
|---|---|---|
| `newop` | bigfloor | (m * a + c) div d, m from 41 to 63 (not 50), d from 11 to 37, c from 41 to 89 (not 50), m / d from 1.6 to 4.6, coprime; draws that a short program over the small constants matches are rejected |
| `newop` | mixfloor | (m * a + n * b + c) div d, m and n different from 11 to 29, c from 41 to 79, d from 41 to 63 |
| `newop` | residue | the number x below p * q with x mod p = a mod p and x mod q = b mod q; (p, q) from (5, 8), (7, 9), (8, 9), (7, 10), (9, 10), (5, 12), (7, 12), (8, 11), (9, 11) |
| `newop` | quadwrap | (k * a * a + c) mod P, k from 23 to 61, c from 9 to 88, P a prime from 73 to 113 |
| `newop` | selfindex | Golomb's sequence: G(1) = 1, G(n) = 1 + G(n - G(G(n - 1))) (a from 1; 1 to 10) |
| `newop` | gcdstep | R(1) = s, R(n) = R(n - 1) + gcd(n, R(n - 1)), s from 7, 8, 9, 11, 13 (a from 1) |
| `newop` | squareorbit | x(0) = x0, x(n + 1) = (x(n)^2 + c) mod m, answer x(a); x0 from 2 to 5, c from 1 to 9, m from 29 to 61, at least 14 distinct values in 0 to 31 |
| `newop` | fibcode | the Zeckendorf form of a (non-consecutive Fibonacci numbers 1, 2, 3, 5, 8, ...) written as binary digits, one digit per Fibonacci number (0 to 82) |
| `newop` | remainders | the number of divisions Euclid's algorithm makes on (a, b), both from 1 (1 to 7) |
| `newop` | trianglepos | if a is a triangular number k * (k + 1) / 2 then k, else the distance from a to the largest triangular number below it |
| `newop` | evenbits | 3 * a + p if the number of 1 bits of a is even (p from 1 to 4), else a div 2 |
| `compose` | golombsum | selfindex(1) + ... + selfindex(a) (a from 1) |
| `compose` | fibcodesteps | remainders(a, fibcode(a)) (a from 1) |
| `compose` | gcdexcess | gcdstep(a) - a (a from 1) |
| `compose` | evenbitstri | evenbits(trianglepos(a)) |
| `control` | grow | a + a div d, d from 3 to 5 |
| `control` | residual | a mod p, p from 5 to 9 |
| `control` | larger | max(a, b) |
| `control` | product | a * b |
| `unexplainable` | noise6, noise45, noise900 | an arbitrary code (0 to 5, 0 to 44, 0 to 899) for each number: no rule, must stay refused until told |

Roughly: the first four are the large-constant and scaled-floor kinds (the open gaps); the next three define a number from earlier ones (a term that
looks itself up, a step that adds a gcd, an iterated square); `fibcode` and `remainders` are another numeral system and a loop over a pair; the last two
`newop` kinds are conditionals on a property of the input (triangular or not, the parity of the number of 1 bits). Which is easy for which learner was
not measured, and no kind was tuned either way.

Conventions a system could trip over: `selfindex`, `gcdstep` and `remainders` start at 1; `fibcode(0)` is 0; `trianglepos(0)` is 0 (0 is triangular);
`evenbits(0)` takes the even branch (zero 1 bits); `gcdexcess` is always at least 6 (the start is at least 7).
