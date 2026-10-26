<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 DawsMacs -->
# Hidden worlds C: a third set of exam worlds whose kinds need an operation the learner does not start with

`tools/hidden_worlds_c.py` writes a third family of adaptive exams. The files, the line protocol, the shape of a world (inputs 0 to 31, invented words
for the kinds, three taught phrasings per fact, held-out and beyond-range questions with the truth told after each answer) and the roles are those of
`tools/hidden_worlds.py` and `tools/hidden_worlds_b.py` (see `tools/hidden_worlds.md`); the kinds are all different from both sets and drawn from
families that neither used heavily. Every interesting kind needs an operation outside `+ - * min max div mod` (div is integer division).

**Why a third set.** Once a mechanism has been developed against the kinds of a set, that set is no longer a fair test of it. This set was written
without looking at the learner it will be run on, and without tuning any kind for or against a recurrence-based or a program-search learner. **To
keep it a blind test, do not read the list of kinds below the marked line (or `make_kinds` in the script) before the first run**: run it, look at the
numbers, then read on.

```bash
python3 tools/hidden_worlds_c.py DIR --seed 1             # writes lessons.tsv, stream.tsv (+ README.txt, key.tsv)
baby exam adapt-drive DIR COMMAND...                      # judge any system on it, as for the clean adaptive exam
baby exam adapt-run DIR                                   # the baby itself
python3 -m unittest discover -s tools -p 'test_hidden_worlds_c.py'
```

`--compose-first` teaches the compositions before the operations they use. The generator is pure standard-library Python 3 with the first set's small
deterministic random generator, so a seed gives the same bytes on every Python version. It imports the first generator for the shared pieces (the
random generator, the phrasings, the choice of points); the kinds, the words and the seeds' salts are its own.

## What differs from the earlier sets (without naming a kind)

- The words are two syllables plus the ending `ketu` (the first set's end in `sido`, the second set's in `velu`), drawn with another salt and another
  alphabet of consonants, so no word of one set is a word of another (a test checks seeds 1 to 10 against both).
- No kind of the earlier sets is reused, by name or by table: a test compares every table of this set with every table of both earlier sets
  (including their controls and their compositions) and finds no equal pair. Nothing is a re-parametrization (another base, another modulus) of an
  earlier kind.
- 23 kinds: 11 `newop`, 5 `compose`, 4 `control`, 3 `unexplainable`. The stream has 346 questions (170 newop, 78 compose, 56 control, 42
  unexplainable); there are 1482 lessons. Unary kinds teach the numbers up to 19 (15 or 16 of them; four are held out and asked) and ask ten above
  19; binary kinds teach 60 pairs with both numbers up to 15, hold out 8 and ask 14 pairs with a number above 15 (the same constants as the earlier
  sets). 9 of the 11 `newop` kinds and 4 of the 5 `compose` kinds take one number; the others take two.

## What the baby and a memorizer do on it (seeds 1 to 3)

`baby exam adapt-run DIR` (percent of the questions in the role; WRONG was 0.0 % everywhere):

| seed | control (56) | newop (170) | compose (78) | unexplainable (42) |
|---|---|---|---|---|
| 1 | all refused | all refused | all refused | all refused |
| 2 | all refused | all refused | all refused | all refused |
| 3 | all refused | all refused | all refused | all refused |

Over the whole stream the baby's facts right are 0.0 % on all three seeds, with 100 % refused and no guess at all. **The baby does not solve the
controls of this set**, unlike those of the second set (55.6 % facts right on seed 1). This is not a fault of the files: with the controls swapped
for four simple ones in a throwaway run (a mean, a triangular number, a double plus three, a wrap) the same code answered 60.9 % of the control
questions as facts. The baby's base language has the constants 1 and 2 only and learns larger constants from rules it has fitted; the controls here
need constants from 3 up to 14 inside expressions of five to seven nodes, and the baby does not find them. The controls are still
expressible with the starting operations (next section).

`baby exam adapt-drive DIR python3 tools/exam_adapt_baseline.py` (the naive memorizer): 0.0 % right in every role on all three seeds, 100 % refused,
WRONG 0. No question in the stream repeats a taught or an earlier question, so memorizing cannot score.

---

# STOP: do not read below this line before the first run

Everything below names the kinds.

---

## The kinds (23)

The families: loops with a state over the digits of a number in several bases, a conditional sequence step with its own constants, sums and orders
from number theory other than the earlier ones, counts other than binomial, Catalan and partition numbers, a recurrence of order three, and a game.
The controls are clamps, a threshold and a scaled floor, with constants drawn from the seed.

| role | kind | rule |
|---|---|---|
| `newop` | longrun | the longest run of equal bits in the binary form of a (a from 1; 1 to 5) |
| `newop` | palbase | the smallest base from 2 in which the digits of a read the same backwards (2 to 18) |
| `newop` | commondigit | the digit that occurs most often among the base-5 digits of a, the smaller on a tie (0 to 4) |
| `newop` | multsum | the sum of the numbers below a that are divisible by 3 or by 5 (0 to 225) |
| `newop` | hail | steps to reach 1 from a: halve if even, else divide by 3 if divisible by 3, else p * a + q, with (p, q) drawn from (1, 2), (1, 3), (1, 4), (2, 2), (2, 4) (a from 1; every pair stops for every a up to 31) |
| `newop` | sigma | the sum of the divisors of a (a from 1; 1 to 72) |
| `newop` | order2 | the multiplicative order of 2 modulo 2a + 1: the least k from 1 with 2^k = 1 there (a from 1; 2 to 60) |
| `newop` | derange | derangements of a things (1, 0, 1, 2, 9, 44, ...) mod m, m from 7, 9, 10, 11, 12, 13 drawn from the seed |
| `newop` | stirling | Stirling number of the second kind: ways to split a things into b non-empty groups, mod m, m from 7 to 12 drawn from the seed (a at least b, b from 1) |
| `newop` | tribo | tribonacci number a (0, 0, 1, 1, 2, 4, 7, 13, ...) mod m, m from 9 to 14 drawn from the seed |
| `newop` | wythoff | Grundy value of two heaps of a and b, a move taking any number from one heap or the same number from both (0 to 49) |
| `compose` | aliquot | sigma(a) - a (a from 1) |
| `compose` | bellpart | stirling(a, 1) + ... + stirling(a, b), with the modulus of `stirling` (a at least b, b from 1) |
| `compose` | wythdiag | wythoff(a, a) |
| `compose` | multpal | multsum(palbase(a)) |
| `compose` | triborder | tribo(order2(a)) with the modulus of `tribo` (a from 1) |
| `control` | clampshift | a clamped to the range lo to hi, minus lo (lo from 4 to 9, hi from 18 to 25) |
| `control` | bracket | min(a, k) + s * max(a - k, 0), k from 8 to 13, s from 2 to 4 |
| `control` | fold | max(a mod p, c) - c, p from 11 to 14, c from 3 to 6 |
| `control` | scale | (m * a + c) div d, m from 3 to 7, d from 5 to 13 and coprime to m, c from 1 to 4 |
| `unexplainable` | noise8, noise60, noise700 | an arbitrary code (0 to 7, 0 to 59, 0 to 699) for each number: no rule, must stay refused until told |

The kinds were chosen for variety, not to suit a particular learner. Roughly: some a person can learn from a table by noticing a recurrence
(derange, tribo, sigma by its multiplicative structure), some need a loop with a state (longrun, palbase, commondigit, hail, order2, multsum), some
are periodic in disguise (tribo and derange mod a small m, order2), and some are hard for any small learner (wythoff, stirling, bellpart, multpal).
Which is easy for which learner was not measured, and none of the kinds was tuned either way.

## Is any kind reachable with the starting operations? (brute force, 7 nodes)

A throwaway search (not kept) enumerated every expression of at most 7 nodes (leaves count) over `+ - * min max div mod`, the variable or variables, and
the constants 0 to 40, 50, 64 and 100, deduplicated by value on the sample points, with subtraction both signed and saturating at 0 and division or
remainder by zero taken as 0. Unary kinds were matched on their whole domain (0 or 1 to 31); binary kinds on 60 random pairs (a at least b from 1 for
`stirling` and `bellpart`, any pair from 0 to 31 for `wythoff`). An exact match on the sample counts as reachable, so the check errs toward calling a
kind reachable. It covered seeds 1 to 10, so every drawn modulus and constant: 99 distinct unary targets (15 over 1 to 31, 84 over 0 to 31; the kinds of every
role and the codes of every seed) and 13 binary ones.

- **Reachable, as intended:** all four controls, in every seed from 1 to 10 and in both subtraction conventions: `clampshift` at 5 or 7 nodes,
  `bracket` at 5 or 7, `fold` at 5 or 7, `scale` at 7 (three constants).
  The `bracket` knee is limited to 13 so that (s - 1) * k is at most 39, a constant the search has; with a knee of 14 or 15 and s = 4 the control
  needs a larger constant and goes beyond 7 nodes in the search.
- **Not reachable at 7 nodes, any seed 1 to 10:** all 11 `newop` kinds (every modulus and every step rule tried), all 5 `compose` kinds, and the
  arbitrary codes. No kind had to be replaced for being reachable. (Three drafts were replaced while the set was written: two controls that
  needed 9 or more nodes and one composition that took only two values.) The best
  fraction of the sample points that a 5-node expression gets right (7 nodes was only checked for exact matches): commondigit 0.72, multpal 0.66,
  derange 0.59, stirling 0.58, palbase 0.56, longrun 0.55, aliquot 0.45, tribo 0.44, triborder 0.42, hail 0.39, sigma
  0.39, bellpart 0.32, order2 0.32, multsum 0.31, wythdiag 0.25, wythoff 0.20, the codes 0.12 to 0.38.
- **The claim is narrow.** Because inputs stop at 31, every kind here is a finite table, and some have a longer expression: the base-5 digits of a
  number up to 31 are `a mod 5`, `a div 5 mod 5` and `a div 25`, so `commondigit` unrolls into a long expression, and `longrun` into one over the five bits. Those are far beyond 7 nodes and give no way to generalize past 31, but "not reachable at 7
  nodes" is the claim, not "impossible" or "not expressible by a long program". Operations a learner might build for itself out of loops or conditionals are not covered either.

## Things to know

- Every answer is exact by construction (the tests recompute each lesson and stream answer from the rules of its kind, check the operations against
  known values, and check the counts again by brute force: set partitions by restricted-growth strings, derangements by permutations, the game by a
  memoized mex written a second way, the order of 2 and the base of the first palindrome by their definitions).
- The stream asks each fact once; it has no wrong-teacher columns (this is the clean exam).
- Conventions a system could trip over: `palbase(0)` and `palbase(1)` are 2 (a single digit is a palindrome in any base, and the smallest base is 2);
  `commondigit` takes the smaller digit on a tie and counts only the digits written (no leading zeros; 0 is the single digit 0); `multsum(a)` sums
  numbers strictly below a (and counts 0, which adds nothing); `derange(0)` is 1 and `derange(1)` is 0; `tribo` starts 0, 0, 1; `wythoff(0, b)` is b;
  `stirling` and `bellpart` count groups from 1 and are asked with a at least b; `order2` is the order modulo 2a + 1, so a = 1 is the order modulo 3.
- Both heaps of `wythoff` are asked in either order, so a system sees (a, b) and (b, a) as separate questions with the same truth.
