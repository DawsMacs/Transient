<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 DawsMacs -->
# Hidden worlds B: a second set of exam worlds whose kinds need an operation the learner does not start with

`tools/hidden_worlds_b.py` writes a second family of adaptive exams. The files, the line protocol, the shape of a world (inputs 0 to 31, invented words
for the kinds, three taught phrasings per fact, held-out and beyond-range questions with the truth told after each answer) and the roles are those of
`tools/hidden_worlds.py` (see `tools/hidden_worlds.md`); the kinds are all different, and drawn from other families. Every interesting kind needs an
operation outside `+ - * min max div mod` (div is integer division).

**Why a second set.** A set of kinds that a mechanism was developed against cannot also be the test of that mechanism. This set exists to be run once
the mechanism is fixed. **To keep it a blind test, do not read the list of kinds below (or `make_kinds` in the script) before the first run**: run it,
look at the numbers, then read on.

```bash
python3 tools/hidden_worlds_b.py DIR --seed 1             # writes lessons.tsv, stream.tsv (+ README.txt, key.tsv)
baby exam adapt-drive DIR COMMAND...                      # judge any system on it, as for the clean adaptive exam
baby exam adapt-run DIR                                   # the baby itself
python3 -m unittest discover -s tools -p 'test_hidden_worlds_b.py'
```

`--compose-first` teaches the compositions before the operations they use. The generator is pure standard-library Python 3 with the first set's small
deterministic random generator, so a seed gives the same bytes on every Python version. It imports the first generator for the shared pieces (the random
generator, the phrasings, the choice of points); the kinds, the words and the seeds' salts are its own.

## What differs from the first set

- The words are two syllables plus the ending `velu` (the first set's words end in `sido`), drawn with another salt, so no word of one set is a word of
  the other (a test checks it).
- Every kind of the first set is out: popcount, xor, and, or, gcd, lcm, isqrt, divisors, lookup table, next power of 2, trailing zeros, bit mirror,
  last digit of a power, bit-count parity, ceiling division and the others built on them. A test checks that no kind name or kind of the first set's
  worlds appears here.
- The families: number theory, sequences by recurrence, loops with a state, digits in base 3, combinatorial counts, order statistics over derived values.
  Nothing here is a bit trick.
- 23 kinds: 11 `newop`, 5 `compose`, 4 `control`, 3 `unexplainable`. The stream has 370 questions (170 newop, 86 compose, 72 control, 42 unexplainable);
  there are 1881 lessons. Unary kinds teach the numbers up to 19 (15 or 16 of them; four are held out and asked) and ask ten above 19; binary kinds teach 60 pairs with both
  numbers up to 15, hold out 8 and ask 14 pairs with a number above 15 (the same constants as the first set).

## The kinds (23)

| role | kind | rule |
|---|---|---|
| `newop` | totient | Euler's totient: how many of 1 to a share no factor with a (1 to 31) |
| `newop` | omega | number of distinct prime factors of a (1 to 31) |
| `newop` | nextprime | the smallest prime greater than a |
| `newop` | fibmod | Fibonacci number a (0, 1, 1, 2, ...) mod m, m from 8, 9, 10, 12, 13 drawn from the seed |
| `newop` | collatz | steps to reach 1 from a: halve if even, else 3a + 1 (1 to 31; up to 111 steps) |
| `newop` | partitions | number of ways to write a as a sum of positive integers, order ignored (1 to 6842) |
| `newop` | catalan | Catalan number a (1, 1, 2, 5, 14, ...) mod m, m from 9 to 13 drawn from the seed |
| `newop` | digsum3 | sum of the digits of a written in base 3 |
| `newop` | rev3 | the digits of a in base 3 read backwards (no leading zeros kept), as a number |
| `newop` | binom | a choose b mod m, m from 7, 10, 11, 12 drawn from the seed (asked only with a at least b) |
| `newop` | josephus | a people in a circle, every b-th one leaves: the place (1 to a) of the last one standing (a, b from 1) |
| `compose` | totsum | totient(1) + ... + totient(a) |
| `compose` | primegap | nextprime(a) - a |
| `compose` | totrank | the b-th smallest of totient(1), ..., totient(a) (b from 1, at most a) |
| `compose` | fibcollatz | fibmod(collatz(a)) with the same m as fibmod |
| `compose` | digdiff3 | the absolute difference of digsum3(a) and digsum3(b) |
| `control` | mean | (a + b) div 2 |
| `control` | spread | max(a, b) - min(a, b) |
| `control` | triangle | a * (a + 1) div 2 |
| `control` | wrap | (k * a + c) mod m, k coprime to m, drawn from the seed |
| `unexplainable` | code10, code100, code1000 | an arbitrary code (0 to 9, 0 to 99, 0 to 999) for each number: no rule, must stay refused until told |

The kinds were chosen for variety, not to suit a particular learner. Roughly: some a person can learn from a table by noticing a recurrence
(fibmod, partitions by its differences), some need a loop with a state (collatz, josephus, nextprime, totrank), some are periodic in disguise (fibmod,
catalan mod a small m), and some are hard for any small learner (partitions, catalan, binom, josephus, totrank). Which is easy for which learner was not
measured, and none of the kinds was tuned either way.

## Is any kind reachable with the starting operations? (brute force, 7 nodes)

A throwaway search (not kept) enumerated every expression of at most 7 nodes (leaves count) over `+ - * min max div mod`, the variable or variables, and
the constants 0 to 40, 50, 64 and 100, deduplicated by value on the sample points, with subtraction both signed and saturating at 0 and division or
remainder by zero taken as 0. Unary kinds were matched on all of 0 to 31 (their whole domain); binary kinds on 60 random pairs from 0 to 31 (fewer where
the kind's domain is smaller). An exact match on the sample would count as reachable, so the check errs toward calling a kind reachable. It covered
seeds 1 to 10, so every drawn modulus and constant: 56 distinct unary targets and 9 binary ones.

- **Reachable, as intended:** all four controls. `mean` at 5 nodes, `spread` at 7, `triangle` at 7, `wrap` at 5 or 7 depending on the seed.
- **Not reachable at 7 nodes, any seed 1 to 10:** all 11 `newop` kinds (every modulus tried), all 5 `compose` kinds, and the arbitrary codes. No kind
  had to be replaced. The best fraction of the sample points that a 5-node expression gets right (7 nodes was only checked for exact matches): omega 0.74,
  primegap 0.59, fibmod mod 8 0.59 (others 0.31 to 0.44), binom mod 7 0.63, digsum3 0.53, totrank 0.50, catalan 0.34 to 0.53, nextprime 0.38, rev3 0.38,
  totient 0.39, collatz 0.29, josephus 0.25, partitions 0.19, totsum 0.16.
- **The claim is narrow.** Because inputs stop at 31, every kind here is a finite table, and some have a longer expression: the base-3 digit sum is
  `a mod 3 + a div 3 mod 3 + a div 9 mod 3 + a div 27` (19 nodes, checked to match on 0 to 31), and unrolling the digits similarly reaches `rev3` and
  `digdiff3`. Those are far beyond 7 nodes and give no way to generalize past 31, but "not reachable at 7 nodes" is the claim, not "impossible" or "not
  expressible by a long program". Operations a learner might build for itself out of loops or conditionals are not covered either.
- **Left out on purpose:** polynomial sequences such as the tetrahedral numbers (`a * (a + 1) * (a + 2) div 6`, 11 nodes: too easy to write by hand once
  the triangular numbers are known); `triangle` is kept as a control for that reason.

## What the baby and a memorizer do on it (seeds 1 to 3)

`baby exam adapt-run DIR` (percent of the questions in the role; WRONG was 0.0 % everywhere; a guess right is a guess the baby marked as a guess):

| seed | control (72) | newop (170) | compose (86) | unexplainable (42) |
|---|---|---|---|---|
| 1 | 55.6 % facts right, 25.0 % guessed right, 19.4 % refused | all refused | all refused | all refused |
| 2 | 54.2 % facts right, 26.4 % guessed right, 19.4 % refused | all refused | all refused | all refused |
| 3 | 58.3 % facts right, 22.2 % guessed right, 19.4 % refused | all refused | all refused | all refused |

Over the whole stream the baby's facts right are 10.8 %, 10.5 % and 11.4 %. As expected it handles the controls and gets none of the 11 + 5 new-operation
kinds, which it refuses honestly.

`baby exam adapt-drive DIR python3 tools/exam_adapt_baseline.py` (the naive memorizer): 0.0 % right in every role on all three seeds, 100 % refused,
WRONG 0. No question in the stream repeats a taught or an earlier question, so memorizing cannot score.

## Things to know

- Every answer is exact by construction (the tests recompute each lesson and stream answer from the rules of its kind, and check the operations against
  known values: totients, prime counts, Collatz and Josephus tables, partition and Catalan numbers, base-3 digits).
- The stream asks each fact once; it has no wrong-teacher columns (this is the clean exam).
- Conventions a system could trip over: `nextprime(0)` and `nextprime(1)` are 2; `fibmod` starts 0, 1; `catalan` starts 1, 1; `josephus` counts places from 1;
  `rev3` drops leading zeros of the reversal (rev3 of 3, which is `10` in base 3, is 1); `totrank` counts from 1.
- The `wrap` multiplier is chosen coprime to its modulus so that the kind is never a short cycle; the first seeds' constants are in `key.tsv`.
