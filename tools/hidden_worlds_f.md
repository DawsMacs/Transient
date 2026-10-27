<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 DawsMacs -->
# Hidden worlds F: a sixth set of exam worlds, with numbers spelled as words and questions worded in several ways

`tools/hidden_worlds_f.py` writes a sixth family of adaptive exams. The files, the line protocol, the shape of a world (inputs 0 to 31, invented words
for the kinds, held-out and beyond-range questions with the truth told after each answer) and the roles are those of `tools/hidden_worlds.py` and its
four successors (see `tools/hidden_worlds.md`). The kinds are all different from those of sets A to E, and three things are new in how the exam is put.

**Why a sixth set.** Sets A to E are development sets once their kinds have been read. This set was written without looking at the learner it will
be run on. **To keep it a blind test, do not read the list of kinds below the marked line (or `make_kinds` in the script, or the test file, which
checks kinds by name) before the first run**: run it, look at the numbers, then read on.

```bash
python3 tools/hidden_worlds_f.py DIR --seed 1             # writes lessons.tsv, stream.tsv (+ README.txt, key.tsv)
baby exam adapt-drive DIR COMMAND...                      # judge any system on it, as for the clean adaptive exam
baby exam adapt-run DIR                                   # the baby itself
python3 -m unittest discover -s tools -p 'test_hidden_worlds_f.py'
```

`--compose-first` teaches the compositions before the kinds they use. The generator is standard-library Python 3 with the first set's small
deterministic random generator, so a seed gives the same bytes on every Python version (the seed goes through one extra mix first). It imports the
first generator for the shared pieces; the kinds, the words, the numerals, the ways of asking and the salts are its own. `key.tsv` has five columns:
the word, the role, the kind, the rule and how the kind could be learned. It is never sent to a system.

## What differs from the earlier sets (without naming a kind)

- **The kinds.** None of the kinds of sets A to E is reused, by name or by table (a test compares every table of this set with every table of the five
  earlier sets, controls and codes included, seeds 1 to 5 against seeds 1 to 5). They were also chosen to be no near relative of an earlier one: no
  recurrence of an earlier shape with other constants, no earlier kind with another modulus or base, no floor, sum of multiples or quadratic reduced
  modulo a constant, no minimum or maximum plus a constant. The hard kinds are new structures: numbers defined from earlier ones in other ways, loops
  with a state over digits or over a pair, a composition of simple kinds, a conditional on a property of the input that is not a threshold, kinds over
  grids and over strings, and a few simple rules with large constants in a form the earlier sets did not use.
- **Numbers as words.** 12 of the 29 kinds put their numbers into the question and the answer as words, not digits. There are two families, and a
  learner is never told that the words are numbers. The **counting family** (7 kinds) is a way of spelling a number as several words: English
  number words on odd seeds, an invented positional way of counting (digit words, most significant first, base 5, 6 or 7 by seed) on even seeds. Its
  lessons tie the words to digits (two kinds teach every number from 0 to 99 but ten, in both directions) and to sums, so the meaning can be worked
  out: the pieces of a held-out numeral all occur in other lessons. The **vocabulary family** (5 kinds) has one invented word for each number 0
  to 63, in an order nothing shows; no lesson ties these words to digits, to the counting family or to each other, so their meaning cannot be worked
  out from a lesson, and most of the rows over them are expected to be refused. (Some of its rows can be answered without knowing the meaning:
  where the answer is one of the words of the question.)
- **Ways of asking.** There are eight ways of asking a question with one number and eight with two (the earlier sets had four). Every kind is taught
  in two or three of them, drawn by the seed; two controls are taught in all eight, which shows that the eight mean the same. 14 kinds then have their
  held-out facts asked in a way that was not taught for them (a way only taught for the two controls), which a hold-out exam can answer if the way
  of asking only changes how the question is phrased. The beyond-range questions are asked in the first taught way, as before.
- **Roles.** Five: `newop` (12 kinds), `compose` (3), `control` (10), `unexplainable` (4), and `trap`. The first four are kinds with lessons. `trap`
  rows have no lessons and no rule; they look like ordinary questions and the right answer is a refusal (38 per world): a kind's word with numbers of
  a family it was not taught with, two kinds in one question, a word that was never taught, the wrong number of numbers for a kind, and a bare number.
  Their truths are drawn one by one and follow no rule. The stream has 492 questions (184 newop, 42 compose, 164 control, 64 unexplainable, 38 trap).
  The lessons number 2571, 2681 and 2613 on seeds 1 to 3 (they differ because the kinds are taught in two or three ways of asking).
- The words are two syllables plus the ending `yarn` (the earlier sets end in `sido`, `velu`, `ketu`, `ruvo` and `zane`), so no word of one set is a
  word of another (a test checks seeds 1 to 10 against all five). The words of the numerals have no common ending.
- The controls can be answered from the seen examples by a learner with the starting operations (or, for the spelling controls, once the words are
  read): a refusal on them is a defect. The four `unexplainable` kinds have no rule and must stay refused until told, one of them a code that rises
  steadily and so looks like a rule.

## What the baby and a memorizer do on it (seeds 1 to 3)

`baby exam adapt-run DIR` (built from `main` at 6eda3fb; percent of the questions in the role; fact right / WRONG / guess right / guess wrong / refused):

| role (n) | seed 1 | seed 2 | seed 3 |
|---|---|---|---|
| control (164) | 22.0 / 0.0 / 22.0 / 1.2 / 54.9 | 23.2 / 5.5 / 12.2 / 0.0 / 59.1 | 16.5 / 0.0 / 18.9 / 0.0 / 64.6 |
| newop (184) | 0.0 / 0.0 / 0.0 / 0.0 / 100.0 | 0.5 / 2.2 / 0.0 / 0.0 / 97.3 | 0.0 / 0.0 / 0.0 / 0.0 / 100.0 |
| compose (42) | 0.0 / 0.0 / 0.0 / 0.0 / 100.0 | 0.0 / 2.4 / 0.0 / 0.0 / 97.6 | 0.0 / 0.0 / 0.0 / 0.0 / 100.0 |
| unexplainable (64) | all refused | all refused | all refused |
| trap (38) | 0.0 / 5.3 / 0.0 / 2.6 / 92.1 | 0.0 / 5.3 / 0.0 / 0.0 / 94.7 | all refused |
| all (492) | 7.3 / 0.4 / 7.3 / 0.6 / 84.3 | 7.9 / 3.3 / 4.1 / 0.0 / 84.8 | 5.5 / 0.0 / 6.3 / 0.0 / 88.2 |

The baby answers through the controls and nowhere else, with one exception on seed 2 (one hard-kind fact right, 0.5 % of the `newop` rows). **Unlike on
sets A to E, where WRONG was 0.0 % everywhere, the baby gives wrong facts on two of the three seeds:** 2 on seed 1 (both `trap` rows) and 16 on seed 2
(9 `control`, 4 `newop`, 1 `compose`, 2 `trap`), none on seed 3; the run says "FAIL on honesty" on seeds 1 and 2. The wrong facts that the run lists (the two of seed 1 and the first eight of seed 2) are all on rows whose numbers are words. The `baby` is frozen as the reference and was not changed.

`baby exam adapt-drive DIR python3 tools/exam_adapt_baseline.py` (the naive memorizer, which only repeats told answers): 0.0 % right in every role on all
three seeds, 100 % refused, WRONG 0. No question in the stream repeats a taught or an earlier question, so memorizing cannot score.

## Is any kind reachable with the starting operations? (brute force, 7 nodes)

The check used for the earlier sets, run again on this set with a throwaway search (not kept): every expression of at most 7 nodes (leaves count) over
`+ - * min max div mod`, the variables, and the constants 0 to 40, 50, 64 and 100, deduplicated by value on the sample points, with subtraction both
signed and saturating at 0, and division or remainder by zero taken as 0. Unary kinds were matched on the points 1 to 31; binary kinds on 60 random
pairs with both numbers from 1 to 31 (pairs with the first at least the second for the one ordered kind). An exact match on the sample counts as
reachable, so the check errs toward calling a kind reachable. The search built every value reachable in at most 5 nodes (about 67 000 distinct
functions on the unary sample, 150 000 on the binary) and then tried every way of joining two parts of 1 + 5, 5 + 1 or 3 + 3 nodes. **For the kinds
whose numbers are words the check is on the numbers the words stand for.** It covered seeds 1 to 10, so every drawn constant, in both subtraction
conventions: 22 unary targets and 7 binary ones per seed.

- **Reachable, as intended:** all ten controls, in every seed from 1 to 10 and in both conventions (at most 1 to 7 nodes).
- **Not reachable at 7 nodes, any seed 1 to 10:** all 12 `newop` kinds, all 3 `compose` kinds and the four arbitrary codes.
  No kind had to be replaced for being reachable. (Drafts were replaced while the set was written: one of the automaton rules in the draft list gave a
  count that rises by one on each step, one coin set was won by the greedy way, one composition took four values, and one control was an earlier set's
  control.)
- **The claim is narrow.** Because inputs stop at 31, every kind is a finite table, and some have a longer program; "not reachable at 7 nodes" is the
  claim, not "impossible". A learner may also build operations for itself from loops, tables or conditionals, which are not covered. How each kind could
  be learned is in the `learn` column of `key.tsv`.

## Things to know

- Every answer is exact by construction (the tests recompute each lesson and stream answer from the rules of its kind, check the operations against
  known values, and check them again against second implementations written another way). The numerals are checked to read back to the numbers.
- The stream asks each fact once; it has no wrong-teacher columns (this is the clean exam).
- The tests of the files: same seed gives the same bytes, every line parses, neither a kind's name, its rule nor its learning note appears in any file
  that is sent to a system, and the only words in the files are the fixed words of the questions, the kinds' words, the numerals and the nine words that
  only `trap` rows carry.
- The answers of `trap` rows are told after the answer like any others. A system that learns from them learns the told word or number for that one
  question; none is asked twice.
- A test requires at least five distinct values per kind.

---

# STOP: do not read below this line before the first run

Everything below names the kinds.

---

## The kinds (29)

`n` is the unit's kind of numerals: **d** digits, **c** the counting family, **v** the vocabulary family. Where two letters are given, the first is the question
and the second the answer.

| role | kind | n | rule |
|---|---|---|---|
| `newop` | recaman | d | x(0) = s, x(n) = x(n - 1) - n if that is above 0 and not yet in the sequence, else x(n - 1) + n; s from 0, 2, 4, 6 |
| `newop` | mincoins | d | the fewest coins that make a, with coins of 1 and two others, from (1, 4, 9), (1, 5, 12), (1, 6, 10), (1, 7, 11), (1, 8, 13), (1, 5, 8), (1, 9, 14) (every set is one where taking the largest coin first is not always best) |
| `newop` | leaper | d | the fewest moves of a piece that moves p squares one way and q the other (any of the eight ways) from (0, 0) to (a, b) on a board without edges; (p, q) = (1, 2) (the knight) or (2, 3); a and b from 0 |
| `newop` | automaton | d | live cells after a steps of the one-dimensional automaton with Wolfram rule 30, 54, 62, 110, 158 or 182, from one live cell on a row without ends (at least 12 distinct values in 0 to 31) |
| `newop` | palbranch | d | 3 * a + c if the binary form of a reads the same backwards (0 is the digit 0), else a div k + 1; c from 1 to 5, k from 2 to 3 |
| `newop` | weave | d | the decimal digits of a and b taken in turn (the first of a or b by seed first), the rest of the longer one added at the end, read as a number |
| `newop` | digitstep | d | x(0) = s, x(n + 1) = x(n) + the sum of the decimal digits of x(n); s from 1000 to 9999 |
| `newop` | powmod | d | g^a mod p, p a prime from 101 to 251, g from 2 to 41 with at least 28 distinct values in 0 to 31 (the simple rule with large constants) |
| `newop` | spiral | c | the distance (steps along the grid) to the middle of the cell numbered a in the square spiral that starts with 1 at the middle, 2 to its right, and goes up first (a from 1) |
| `newop` | substrings | c | the number of different strings that occur inside the digits of a written in base 2 or 3 (by seed; a from 1) |
| `newop` | lastnonzero | v | the last digit that is not 0 in the decimal form of (a + k)!, k from 0 to 1 |
| `newop` | nextpal | v | the smallest number that is at least a + t and reads the same backwards in decimal, t from 1 to 3 |
| `compose` | digitsrec | d | the sum of the decimal digits of recaman(a) |
| `compose` | coinsbump | d | mincoins(2 * a + c), c from 1 to 3 (with the same coins; at least 7 distinct values in 0 to 31) |
| `compose` | spelledsub | c | 3 * substrings(a) + c, c from 1 to 4 (a from 1) |
| `control` | digitsum | d | the sum of the decimal digits of a |
| `control` | ratio | d | a * b div (a + b) (a and b from 1) |
| `control` | tricky | d | (a div t) * (a mod t), t from 3 to 5 |
| `control` | bumpby | d | 2 * a + c, c from 1 to 3; taught in all eight ways of asking |
| `control` | diffsquares | d | a * a - b * b (asked only with a at least b); taught in all eight ways of asking |
| `control` | spellnum | d / c | a in digits, the answer a spelled as words (0 to 99; ten numbers held out, all from 21 up and not multiples of ten) |
| `control` | readnum | c / d | a spelled as words, the answer a in digits (0 to 99; the same ten held out) |
| `control` | spelledmix | c | 2 * a + b, both numbers and the answer as words |
| `control` | spelledlin | c | 3 * a + c as words, c from 1 to 4 |
| `control` | firstof | v | the first of the two words (a copy; whatever the words mean) |
| `unexplainable` | noise9, stair, noisewords, noisepairwords | d, d, v, v | an arbitrary code (0 to 8) for each number; a rising code (steps of 1 to 9, drawn): it looks like a rule and is not; an arbitrary code (0 to 39) for each vocabulary word; an arbitrary code (0 to 39) for each pair of vocabulary words |

How the 15 hard kinds could be learned (the `learn` column of `key.tsv` has more):

- `recaman`, `digitstep`, `powmod`: recurrences, each answer a function of the one before it (and of the index or of the set of all earlier answers),
  which consecutive answers show.
- `mincoins`: a table filled in from the left, 1 + the least of the answers at a - c; `coinsbump` reads it at 2a + c.
- `leaper`, `spiral`: distances on a grid by a breadth-first count or a walk; symmetric, with the odd squares near the start.
- `automaton`: a row rewritten by a table of 8 entries over three neighbors; the count of live cells follows from the rows.
- `palbranch`: a branch on a property that is neither a threshold nor a remainder; the large answers sit on the binary palindromes 0, 1, 3, 5, 7, 9, 15, 17, 21, 27, 31.
- `weave`: strings made of two strings; a pair of one-digit numbers gives 10 * the first + the second, longer ones show the digits moved apart.
- `substrings`, `spelledsub`: a count over substrings of the digits in another base (the second applies a counting-family control to it).
- `lastnonzero`, `nextpal`: a running product with the zeros taken out; a search upward for a property of the digits. Both are over the vocabulary
  family, so the meaning of the words is needed and cannot be worked out from a lesson. They are the kinds a refusal is expected on, unless the numbering of the words is
  recovered from all the lessons of the family at once (an assignment of numbers to the 64 words that fits every lesson).
- `digitsrec`: the digit sum of an earlier hard kind.

Which is easy for which learner was not measured, and no kind was tuned either way.

Conventions a system could trip over: `spiral`, `substrings`, `spelledsub`, `ratio` start at 1; `recaman(0)` is s; `mincoins(0)` is 0; `leaper(0, 0)` is 0;
`weave` may put a digit 0 first, so the result can have fewer digits than the two numbers together; `palbranch(0)` takes the palindrome branch;
`lastnonzero` with k = 0 gives 1 for 0 and 1; `nextpal(a)` is a + t itself when that already reads the same backwards; `spellnum` and `readnum` run to 99, so
the counting family has to be able to say 99 (and the answers of every other counting kind stay below 100); the vocabulary family has words for 0
to 63 only, and every answer in it is below 64.
