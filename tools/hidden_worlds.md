<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 DawsMacs -->
# Hidden worlds: adaptive exams whose kinds need an operation the learner does not start with

`tools/hidden_worlds.py` writes an adaptive exam (the same files and line protocol as `baby exam adapt-write`) in which every interesting kind needs an
operation outside `+ - * min max div mod` (div is integer division). A system that only has those seven operations cannot express the rule of a `newop`
kind in a short program; it has to acquire the operation, or fail. The `control` kinds are expressible with the starting operations, so a regression shows.

```bash
python3 tools/hidden_worlds.py DIR --seed 1               # writes lessons.tsv, stream.tsv (+ README.txt, key.tsv)
baby exam adapt-drive DIR COMMAND...                      # judge any system on it, as for the clean adaptive exam
baby exam adapt-run DIR                                   # the baby itself
python3 -m unittest discover -s tools -p 'test_hidden_worlds.py'
```

`--compose-first` teaches the compositions before the operations they use (a control: reuse should then not help). The generator is pure standard-library
Python 3 with its own small random generator, so a seed gives the same bytes on every Python version.

## The format

`lessons.tsv` is `id TAB question TAB answer TAB unit TAB fact`, `stream.tsv` is `id TAB role TAB unit TAB question TAB truth`: the columns, the ids
(`L00000`, `S00000`), the three taught phrasings per fact and the fourth only-asked rewording are those of the baseline exam. A question is numbers plus
an invented word that names the kind: `what is 13 zorbosido 6`, `what is xarkelsido 21`. Differences from the baseline, on purpose:

- The unit name is the invented word alone (the baseline's unit names say what the kind is, e.g. `sum-...`); `key.tsv` (`unit TAB role TAB kind TAB rule`)
  says what each word means, for the people reading the results. It is never sent to a system.
- The words, the lookup table, the arbitrary codes, the ceiling's divisor and the offset of `popplus` are drawn from the seed, and the order of the kinds
  inside each group is shuffled by it, so a system cannot be tuned to one world.
- Inputs are 0 to 31. Unary kinds teach the numbers up to 19 (four of those are held out and asked in the stream, any of the four phrasings) and ask ten
  numbers above 19. Binary kinds teach 60 pairs with both numbers up to 15, hold out 8 more pairs, and ask 14 pairs with a number above 15. Kinds whose
  rule is undefined or conventional at 0 (the divisor count, the gcd...) simply do not use 0.
- Lessons are in groups, each kind contiguous, in this order: controls, newop, compose, unexplainable. The stream (about 430 questions) is shuffled.

## The kinds (25)

| role | kind | rule |
|---|---|---|
| `newop` | popcount | number of 1 bits of a |
| `newop` | isqrt | integer square root of a |
| `newop` | divisors | number of divisors of a (1 to 31) |
| `newop` | lookup | a small table (4 to 6 entries, drawn from the seed) indexed by a mod its length |
| `newop` | pow2up | smallest power of 2 at least a (1 to 31) |
| `newop` | trailzeros | number of trailing 0 bits of a (1 to 31) |
| `newop` | mirror | the 5 binary digits of a, read backwards |
| `newop` | xor | bitwise exclusive-or of a and b |
| `newop` | and | bitwise and of a and b |
| `newop` | or | bitwise or of a and b |
| `newop` | gcd | greatest common divisor (1 to 31) |
| `newop` | lastpow | last decimal digit of a to the power b (1 to 31) |
| `compose` | popplus | popcount(a) + k, k from 1 to 3 drawn from the seed |
| `compose` | bitparity | popcount(a) mod 2 |
| `compose` | hamming | popcount(a xor b) |
| `compose` | gcdhalf | gcd(a, b) div 2 |
| `compose` | lcm | a * b div gcd(a, b) |
| `compose` | sqrtpop | isqrt(a) + popcount(a) |
| `control` | sum | a + b |
| `control` | difference | a - b, asked only with a at least b |
| `control` | square | a * a |
| `control` | ceiling | ceiling of a / d, d from 3 to 5 drawn from the seed |
| `unexplainable` | code0, code1, code2 | an arbitrary code (0 to 99) for each number: no rule, must stay refused until told |

## Is any kind reachable with the starting operations? (brute force, 7 nodes)

A throwaway search enumerated every expression of at most 7 nodes (leaves count) over `+ - * min max div mod`, the variables, and the constants 0 to 40, 50,
64 and 100, deduplicated by value on sample points (all of 0 to 31 for unary kinds, 28 random pairs for binary kinds), with subtraction both signed and
saturating at 0, and division or remainder by zero taken as 0. An exact match on the sample would count as reachable. The script is not kept in the
repository (it needs numpy and a few minutes); the result:

- **Reachable, as intended:** the controls. `square` at 3 nodes, `sum` at 3, `difference` at 3, `ceiling` at 5 (`(a + d - 1) div d`).
- **Not reachable at 7 nodes, any seed tried** (seeds 1 to 7 for the unary kinds, including the lookup tables of 4 to 6 entries; seed 1 for the binary
  kinds): all 12 `newop` kinds, all 6 `compose` kinds, and the arbitrary codes. The closest anything came to an exact fit on its sample points: isqrt
  0.91, pow2up 0.90, bitparity 0.88, gcdhalf 0.78, lookup 0.53 to 0.84; the others are well below. (This is the best fraction of the sample points a
  7-node expression gets right, not a claim about longer programs.)
- **Found reachable and replaced: floor of log base 2.** On the numbers 1 to 31, `16 div (4 + 15 div x)` equals floor(log2 x) exactly: a hyperbola
  with magic constants. Bit length (floor(log2 x) + 1, with 0 for 0) misses only at 0 with a division by zero convention, so it was replaced as well
  (a different convention for dividing by zero could make it reachable). The kind that took their place, `mirror`, has a best fit of 0.34.
- **Reachable by reasoning, so left out:** triangular numbers (`a * (a + 1) div 2`, 7 nodes); decimal digit reversal of a two-digit number
  (`a mod 10 * 10 + a div 10`, 9 nodes); parity of the decimal digit sum (`(a div 10 + a mod 10) mod 2`, 9 nodes). Parity of the *binary* digit sum is not:
  it is `bitparity`, a composition on popcount.
- Not covered: programs longer than 7 nodes, or with operations a learner might build for itself out of loops or conditionals. "Not reachable at 7 nodes"
  is the claim, not "impossible".

## What the baby and a memorizer do on it (seeds 1 to 3)

`baby exam adapt-run DIR` (percent of questions in the role; WRONG was 0.0 % everywhere; control rows are facts right + guesses right + refused):

| seed | control (72) | newop (208) | compose (108) | unexplainable (42) |
|---|---|---|---|---|
| 1 | 68.1 % right, 12.5 % guessed right, 19.4 % refused | all refused | all refused | all refused |
| 2 | 66.7 % right, 13.9 % guessed right, 19.4 % refused | all refused | all refused | all refused |
| 3 | 77.8 % right, 22.2 % guessed right, 0 % refused | all refused | all refused | all refused |

Over the whole stream the baby's facts right are 11.4 %, 11.2 % and 13.0 %. As expected it handles the controls and gets none of the 12 + 6 new-operation
kinds, which it refuses honestly. Nothing in the new-operation kinds is reached by its mechanisms, so none of these kinds had to be replaced for being
too easy for the baby (the one replacement above was for being reachable by the starting operations).

`baby exam adapt-drive DIR python3 tools/exam_adapt_baseline.py` (the naive memorizer): 0.0 % right in every role on all three seeds, 100 % refused, WRONG 0.
No question in the stream repeats a taught or an earlier question, so memorizing cannot score.

## Things to know

- Every answer is exact by construction (the tests recompute each lesson and stream answer from the rules of its kind).
- The stream asks each fact once; it has no wrong-teacher columns (this is the clean exam).
- The words end in `sido` as in the baseline exam, but their start is a different pattern (consonant, vowel, consonant pair, vowel, optional consonant);
  none of the words of seeds 1 to 3 is a word of the baseline exam's seed 1.
