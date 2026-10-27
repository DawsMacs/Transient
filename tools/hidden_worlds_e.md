<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 DawsMacs -->
# Hidden worlds E: a fifth set of exam worlds, for rules that no line of the learner names

`tools/hidden_worlds_e.py` writes a fifth family of adaptive exams. The files, the line protocol, the shape of a world (inputs 0 to 31, invented words
for the kinds, three taught phrasings per fact, held-out and beyond-range questions with the truth told after each answer) and the roles are those of
`tools/hidden_worlds.py` and its three successors (see `tools/hidden_worlds.md`). The kinds are all different from those of sets A to D.

**Why a fifth set.** Sets A to D are development sets once their kinds have been read. This set was written without looking at the learner it will
be run on. **To keep it a blind test, do not read the list of kinds below the marked line (or `make_kinds` in the script, or the test file, which
checks kinds by name) before the first run**: run it, look at the numbers, then read on.

```bash
python3 tools/hidden_worlds_e.py DIR --seed 1             # writes lessons.tsv, stream.tsv (+ README.txt, key.tsv)
baby exam adapt-drive DIR COMMAND...                      # judge any system on it, as for the clean adaptive exam
baby exam adapt-run DIR                                   # the baby itself
python3 -m unittest discover -s tools -p 'test_hidden_worlds_e.py'
```

`--compose-first` teaches the compositions before the kinds they use. The generator is standard-library Python 3 with the first set's small
deterministic random generator, so a seed gives the same bytes on every Python version (the seed goes through one extra mix first, because the first
generator started from neighboring seeds gives the same stream a few steps on). It imports the first generator for the shared pieces; the kinds, the
words and the salts are its own. `key.tsv` has five columns here: the word, the role, the kind, the rule and how the kind could be learned. It is
never sent to a system.

## What differs from the earlier sets (without naming a kind)

- The words are two syllables plus the ending `zane` (the earlier sets end in `sido`, `velu`, `ketu` and `ruvo`), so no word of one set is a word of
  another (a test checks seeds 1 to 10 against all four).
- No kind of the earlier sets is reused, by name or by table: a test compares every table of this set with every table of the four earlier sets
  (controls and codes included, seeds 1 to 5 against seeds 1 to 5) and finds no equal pair. The kinds were also chosen to be no near relative of an
  earlier one: no recurrence of an earlier shape with other constants, no earlier kind with another modulus or base, no floor of a line or plane, no sum
  of multiples or of a quadratic reduced modulo a constant.
- 22 kinds: 11 `newop`, 4 `compose`, 4 `control`, 3 `unexplainable`; 15 hard kinds in all (`newop` plus `compose`). The stream has 364 questions (186
  newop, 56 compose, 72 control, 50 unexplainable); there are 1968 lessons. Unary kinds teach the numbers up to 19 (four are held out and asked) and ask
  ten above 19; binary kinds teach 60 pairs with both numbers up to 15, hold out 8 and ask 14 pairs with a number above 15 (as in the earlier sets).
  7 of the 11 `newop` kinds, all 4 `compose` kinds and 2 of the 4 controls take one number; the others take two. Of the 3 arbitrary codes, one is a
  code for each pair of numbers (the earlier sets had codes for single numbers only).
- The hard kinds are mostly new structure: numbers defined from earlier ones in other ways than the earlier sets used, loops over the digits of a pair
  or of a number with a state, a member of a set picked out by a property, a state that is only seen through what it prints, a conditional on a property
  of a pair, and two rules that are simple but whose constants are large, in a form that is not a floor of a line. The compositions feed one kind of
  this set with another's answers.
- The controls can be answered from the seen examples by a learner with the starting operations: a refusal on them is a defect. The three
  `unexplainable` kinds have no rule and must stay refused until told.

## What the baby and a memorizer do on it (seeds 1 to 3)

`baby exam adapt-run DIR` (percent of the questions in the role; WRONG was 0.0 % everywhere, in every role and over the whole stream):

| seed | control (72): facts right / guesses right / refused | newop (186) | compose (56) | unexplainable (50) | all (364): facts right |
|---|---|---|---|---|---|
| 1 | 19.4 % / 12.5 % / 68.1 % | all refused | all refused | all refused | 3.8 % (2.5 % guesses right) |
| 2 | 45.8 % / 16.7 % / 37.5 % | all refused | all refused | all refused | 9.1 % (3.3 % guesses right) |
| 3 | 34.7 % / 16.7 % / 48.6 % | all refused | all refused | all refused | 6.9 % (3.3 % guesses right) |

The baby answers only through the controls. It gets no hard kind (0.0 % facts right, 0.0 % guesses right, 100 % refused) and no `unexplainable` row.
Its controls are not all answered as facts: 68.1 %, 37.5 % and 48.6 % of the control questions are refused on the three seeds.

`baby exam adapt-drive DIR python3 tools/exam_adapt_baseline.py` (the naive memorizer, which only repeats told answers): 0.0 % right in every role on all
three seeds, 100 % refused, WRONG 0. No question in the stream repeats a taught or an earlier question, so memorizing cannot score.

## Is any kind reachable with the starting operations? (brute force, 7 nodes)

The check used for the earlier sets, run again on this set with a throwaway search (not kept): every expression of at most 7 nodes (leaves count) over
`+ - * min max div mod`, the variables, and the constants 0 to 40, 50, 64 and 100, deduplicated by value on the sample points, with subtraction both
signed and saturating at 0, and division or remainder by zero taken as 0. Unary kinds were matched on the points 1 to 31; binary kinds on 60 random
pairs with both numbers from 1 to 31 (an exact match on the sample counts as reachable, so the check errs toward calling a kind reachable). The
search builds every value reachable in at most 5 nodes and then tries every way of joining two parts of 1 + 5, 5 + 1 or 3 + 3 nodes (pruned by a
signature on six sample points and then checked in full). It covered seeds 1 to 10, so every drawn constant: 15 unary targets and 7 binary ones per
seed (the 4 controls included), in both subtraction conventions.

- **Reachable, as intended:** all four controls, in every seed from 1 to 10 and in both conventions (at most 5, 5, 5 and 7 nodes).
- **Not reachable at 7 nodes, any seed 1 to 10:** all 11 `newop` kinds, all 4 `compose` kinds and the three arbitrary codes.
- **The claim is narrow.** Because inputs stop at 31, every kind is a finite table, and some have a longer program; "not reachable at 7 nodes" is the
  claim, not "impossible". A learner may also build operations for itself from loops, tables or conditionals, which are not covered. How each kind could
  be learned is in the `learn` column of `key.tsv`.

## Things to know

- Every answer is exact by construction (the tests recompute each lesson and stream answer from the rules of its kind, check the operations against
  known values, and check them again against second implementations).
- The stream asks each fact once; it has no wrong-teacher columns (this is the clean exam).
- Some tests of the files: same seed gives the same bytes, every line parses, and neither a kind's name, its rule nor its learning note appears in any file
  that is sent to a system.
- The first draft of this set had one kind whose answers took only three values over the whole domain, and a composition with a few values; both were
  replaced before the numbers above were taken, and a test now requires at least five distinct values per kind (seven for a digit sequence, whose
  alphabet is ten digits; the smallest arbitrary code is exempt).

---

# STOP: do not read below this line before the first run

Everything below names the kinds.

---

## The kinds (22)

| role | kind | rule |
|---|---|---|
| `newop` | prodrec | x(0) = s, x(1) = t, x(n) = (x(n - 1) * x(n - 2) + n) mod m; the answer is x(a); s, t from 2 to 7, m from 29 to 71, at least 12 distinct values in 0 to 31 |
| `newop` | lookback | h(0) = 1, h(n) = h(n - 1) + h(n div k), k from 3 to 6 |
| `newop` | diatomic | s(0) = 0, s(1) = 1, s(2n) = s(n), s(2n + 1) = s(n) + s(n + 1) + t, t from 0 to 2 (Stern's diatomic sequence for t = 0) |
| `newop` | smoothrank | the a-th number (a from 1) whose prime factors all lie in {2, 3, 5}, {2, 3, 7}, {2, 5, 7} or {3, 5, 7} (1 is the first) |
| `newop` | carrycount | the carries made when a and b are added by hand in binary |
| `newop` | graydecode | the binary reflected Gray code read backwards: digit i of the answer is the parity of the digits of a from the top down to digit i |
| `newop` | editbits | the edit distance (insert, delete or change one digit) between the binary forms of a and b, with 0 written as the digit 0 |
| `newop` | tenthdigit | the a-th digit (a from 1) after the point of the decimal expansion of p / q, q a prime from 61, 109, 113, 131, 149, 167, 179, 223 (period q - 1), p from 2 to 101 below q, at least 7 distinct digits in 1 to 31 |
| `newop` | harmonicsum | the sum of floor(n / k) for k from 1 to a, n from 151 to 950 |
| `newop` | pathcost | the cheapest total over the cells of a path from (0, 0) to (a, b) stepping right or up, cell (i, j) costing ((i + u) * (j + v)) mod m, u and v from 1 to 3, m from 5 to 7 |
| `newop` | coprimesplit | a + b + c if a and b share no factor above 1, else \|a - b\|, c from 1 to 3 (a and b from 1) |
| `compose` | smoothgap | smoothrank(a + 1) - smoothrank(a) (a from 1) |
| `compose` | carrylook | carrycount(a, lookback(a)) |
| `compose` | lookdiatomic | lookback(diatomic(a)) |
| `compose` | coprimerec | coprimesplit(a, prodrec(a)) (a from 1; prodrec may be 0, which shares every factor with a) |
| `control` | cube | a * a * a |
| `control` | quotient | a div b (a and b from 1) |
| `control` | quarter | a * a div d, d from 3 to 6 |
| `control` | doubled | 2 * min(a, b) + c, c from 1 to 5 |
| `unexplainable` | noise7, noise400, noisepair30 | an arbitrary code (0 to 6, 0 to 399) for each number, and an arbitrary code (0 to 29) for each pair of numbers 0 to 31: no rule, must stay refused until told |

Roughly: the first three are numbers defined from earlier ones (a nonlinear recurrence, a look back a fixed fraction of the way, a recursion on the
binary digits of the index); `smoothrank` picks the members of a set by a property; `carrycount`, `graydecode`, `editbits` and `pathcost` are loops
over digits or over a grid with a state; `tenthdigit` shows a hidden state through its digits; `harmonicsum` is the simple rule with a large constant;
`coprimesplit` is a conditional on a property of a pair. Which is easy for which learner was not measured, and no kind was tuned either way.

Conventions a system could trip over: `smoothrank`, `tenthdigit`, `quotient` and `coprimesplit` start at 1; `editbits(0, b)` compares the digit 0
with the binary form of b; `harmonicsum(0)` is 0; `diatomic(0)` is 0; `lookback(0)` is 1; `coprimerec` may feed 0 to `coprimesplit`.
