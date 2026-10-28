<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 DawsMacs -->
# Hidden worlds G: a seventh set of exam worlds, with closed forms, numbers spelled in several grammars and questions that repeat words

`tools/hidden_worlds_g.py` writes a seventh family of adaptive exams. The files, the line protocol, the shape of a world (invented words for the kinds,
held-out and beyond-range questions with the truth told after each answer) and the roles are those of `tools/hidden_worlds.py` and its five
successors (see `tools/hidden_worlds.md`). The kinds are all different from those of sets A to F, and four things are new in how the exam is put.

**Why a seventh set.** Sets A to F are development sets once their kinds have been read. This set was written without looking at the learner it will
be run on. **To keep it a blind test, do not read the list of kinds below the marked line (or `make_kinds` in the script, or the test file, which
checks kinds by name) before the first run**: run it, look at the numbers, then read on.

```bash
python3 tools/hidden_worlds_g.py DIR --seed 1             # writes lessons.tsv, stream.tsv (+ README.txt, key.tsv)
baby exam adapt-drive DIR COMMAND...                      # judge any system on it, as for the clean adaptive exam
baby exam adapt-run DIR                                   # the baby itself
python3 -m unittest discover -s tools -p 'test_hidden_worlds_g.py'
```

`--compose-first` teaches the compositions before the kinds they use. The generator is standard-library Python 3 with the first set's small
deterministic random generator, so a seed gives the same bytes on every Python version (the seed goes through one extra mix first). It imports the
first generator for the shared pieces; the kinds, the words, the numerals, the ways of asking and the salts are its own. `key.tsv` has five columns:
the word, the role, the kind, the rule and how the kind could be learned. It is never sent to a system.

## What differs from the earlier sets (without naming a kind)

- **The kinds.** None of the kinds of sets A to F is reused, by name or by table (a test compares every table of this set with every table of the six
  earlier sets, controls and codes included, seeds 1 to 5 against seeds 1 to 5; the two kinds that read and write words are the identity on a larger
  domain than the spelling controls of set F, and the sum over words is the sum of set A on a larger domain, so on the first 32 numbers those three agree
  with an earlier control by construction). **Closed forms are back**: floors, remainders and a minimum, which sets A to F kept out, appear here in
  shapes none of them used (a floor of a ratio of two linear forms, a sum of two multiples modulo a prime, a quadratic form in two numbers modulo a
  prime, the smaller of two linear forms plus a constant, a five-digit constant divided by a shifted number), next to controls of the same family that a
  short program reaches. Their constants are drawn from the seed and several are large. The other hard kinds are structures that no earlier set used: a
  number defined from all the earlier ones, a string rewritten by a rule, a grid in which each cell reacts to its count, a running total of a property of
  pairs, a loop over digits with a state, a conditional on a property that is neither a threshold nor a remainder, and a string made of the digits of a power.
- **Numbers as words, in grammars of their own.** 20 of the 33 kinds put their numbers into the question or the answer as words, not digits, in numbers up to
  995 to 999 (inputs of the earlier sets stopped at 31 or 99). There are four grammars, and each seed uses three of them, one for each of three families of
  words (the grammar of a family is the same for every kind of the family; its words are invented and are all different from those of the other families):
  - **multiplier words**: a word for each digit 1 to base - 1, a word for the base and a word for its square that multiply the digit word before them (a
    missing digit word means 1), a word for zero that stands alone; the base is 10, 12 or 20 by seed. Sums of parts, but the words multiply.
  - **least significant digit first**: a positional numeral, one word per digit, the last place first; base 7, 12 or 20 by seed, with a second word for two of
    the digits (either is read, the answers use the first), no word for a trailing zero.
  - **subtractive**: multiplier words in base 10 plus a word for "minus": a last digit of 8 (or 7, or 6, by seed) or more is written as the next ten (or
    hundred) minus what is missing, `twenty minus two` for 18.
  - **mixed radix**: three words, least significant first, with 7 places in one position and 12 in the other two (the position is drawn by seed): every
    number is written with three words.

  Seed 1 uses multiplier words, least significant first and subtractive for its first, second and third family; seed 2 uses least significant first,
  subtractive and multiplier words; seed 3 uses subtractive, mixed radix and multiplier words (the mixed radix is never the third family, which has kinds that
  count and copy words). A learner is never told that the words are numbers. The families differ in what the lessons say about them:
  - **First family (8 kinds)**: one kind reads a numeral and writes it in digits and one writes a numeral for a number in digits, taught on a sample of numbers that
    covers every word, so the meaning of the words can be worked out; the other kinds of the family are computed over the numerals.
  - **Second family (6 kinds)**: no kind reads or writes it. Kinds computed over the numerals (a sum, a double and others) fix the meaning only through the arithmetic:
    the sums of small numbers name the words one by one and the sums that carry show the places.
  - **Third family (6 kinds)**: nothing in the lessons can fix the meaning: the kinds over it copy a word, count its words, or are arbitrary codes, and one hard
    kind is defined over the numbers that no lesson ties to the words (a table that only an assignment of numbers to the words can explain).
  
  Every word of a held-out or beyond row of the first two families occurs in the lessons that tie that family (a test checks the zero word and every other word).
  Beyond-range questions use numbers above the largest taught one (600 to 999 on the wide kinds), so words occur there in combinations no lesson showed.
- **Repeats.** Numerals have words that occur more than once (many numbers of every grammar), the held-out rows of each kind over many-word numerals include
  four numerals with a repeated word, and every kind that takes two numbers has at least three held-out rows that name the same number twice. A system that
  reads a question as a set of words cannot tell these apart from their shorter relatives. Three controls (a copy, a count and a last word, all of the third
  family) are answerable only by looking at the words themselves and their order.
- **Ways of asking.** Eight ways of asking a question with one number and eight with two (new wording, with words that occur nowhere in the earlier sets' ways).
  Every kind is taught in two or three of them, drawn by the seed; two controls are taught in all eight, which shows that the eight mean the same. 16 kinds then
  have their held-out facts asked in a way that was not taught for them (a way only taught for the two controls), which a hold-out exam can answer if the
  way of asking only changes how the question is phrased. The beyond-range questions are asked in the first taught way, as before.
- **Roles.** Five: `newop` (12 kinds), `compose` (3), `control` (13), `unexplainable` (5), and `trap`. The first four are kinds with lessons. `trap` rows have
  no lessons and no rule; they look like ordinary questions and the right answer is a refusal (63 per world): a kind's word with numbers of a family it was not
  taught with (8), two kinds in one question (8), a word that was never taught (8), the wrong number of numbers for a kind (8), a bare number (7), a run of words
  that is no numeral of the grammar, as a word repeated, an order of the words the grammar does not allow, or a word one letter away from a number word (18, six of
  each; the family without an order for its words splices in words of another family), and the words of a taught question in an order that no lesson used (6).
  Their truths are drawn one by one and follow no rule. The `unexplainable` kinds include numerals taught nowhere (arbitrary codes over words), and one code that
  rises along a line and is not one. The stream has 657 questions (204 newop, 64 compose, 248 control, 78 unexplainable, 63 trap). The lessons number 5098,
  5089 and 4719 on seeds 1 to 3 (they differ because the kinds are taught in two or three ways of asking).
- The words are two syllables plus the ending `tole` (the earlier sets end in `sido`, `velu`, `ketu`, `ruvo`, `zane` and `yarn`), so no word of one set is a word of
  another (a test checks seeds 1 to 6 against all six). The words of the numerals have three letters and no common ending.
- The controls can be answered from the seen examples by a learner with the starting operations (or, for the spelling and word controls, once the words are
  read or counted): a refusal on them is a defect. The five `unexplainable` kinds have no rule and must stay refused until told, one of them a code that rises
  steadily with small errors and so looks like a rule.

## What the baby and a memorizer do on it (seeds 1 to 3)

`baby exam adapt-run DIR` (built from `main` at d6c86b7; percent of the questions in the role; fact right / WRONG / guess right / guess wrong / refused):

| role (n) | seed 1 | seed 2 | seed 3 |
|---|---|---|---|
| control (248) | 16.5 / 4.8 / 5.6 / 0.0 / 73.0 | 15.3 / 10.5 / 8.1 / 0.0 / 66.1 | 18.1 / 8.5 / 8.1 / 0.0 / 65.3 |
| newop (204) | 1.5 / 9.8 / 0.0 / 0.0 / 88.7 | 1.0 / 10.3 / 0.0 / 0.0 / 88.7 | 0.0 / 7.4 / 0.0 / 0.0 / 92.6 |
| compose (64) | 29.7 / 7.8 / 4.7 / 0.0 / 57.8 | 3.1 / 26.6 / 0.0 / 0.0 / 70.3 | 0.0 / 12.5 / 0.0 / 0.0 / 87.5 |
| unexplainable (78) | 0.0 / 23.1 / 0.0 / 0.0 / 76.9 | 0.0 / 10.3 / 0.0 / 0.0 / 89.7 | 0.0 / 2.6 / 0.0 / 0.0 / 97.4 |
| trap (63) | 0.0 / 6.3 / 0.0 / 0.0 / 93.7 | 0.0 / 3.2 / 0.0 / 1.6 / 95.2 | 0.0 / 6.3 / 0.0 / 0.0 / 93.7 |
| all (657) | 9.6 / 9.0 / 2.6 / 0.0 / 78.8 | 6.4 / 11.3 / 3.0 / 0.2 / 79.1 | 6.8 / 7.6 / 3.0 / 0.0 / 82.5 |

The baby answers 15 to 18 % of the controls as facts, 30 % of the compositions on seed 1 (3 % and 0 % on the others) and next to nothing of the hard kinds (0 to 1.5 %).
**It gives wrong facts on all three seeds**: 59 on seed 1, 74 on seed 2 and 50 on seed 3 (spread over every role, traps and codes included); the run says "FAIL on
honesty" on each. The `baby` is frozen as the reference and was not changed. The WRONG column is far above the 0.0 % that sets A to E had; the eight offenders that the
run lists for each seed are all rows whose question or answer holds words (the rows over digits only are not among the listed ones, which are the first eight in the
stream).

`baby exam adapt-drive DIR python3 tools/exam_adapt_baseline.py` (the naive memorizer, which only repeats told answers): 0.0 % right in every role on all
three seeds, 100 % refused, WRONG 0. No question in the stream repeats a taught or an earlier question, so memorizing cannot score.

## Is any kind reachable with the starting operations? (brute force, 7 nodes)

The check used for the earlier sets, kept this time as a test (`test_no_hard_kind_is_reachable_and_the_controls_are`; it needs numpy and takes a few minutes, so it
is skipped without numpy; `HW_G_SEARCH_SEEDS=N` limits it to seeds 1 to N): every expression of at most 7 nodes (leaves count) over `+ - * min max div mod`, the
variables, and the constants 0 to 40, 50, 64 and 100, deduplicated by value on the sample points, with subtraction both signed and saturating at 0, and division or
remainder by zero taken as 0. Unary kinds were matched on the points 1 to 31, 1 to 80 or a fixed spread of 39 points up to 494 (by the size of the domain); binary kinds on 60
fixed pairs, with both numbers from 1 to 31 or, for the wide domains, half of them from 1 to 24 and half from 1 to 494. An exact match on the sample counts as
reachable, so the check errs toward calling a kind reachable. The search builds every function of 1, 3 and 5 nodes (about 45, 1 000 and 95 000 distinct ones on the unary
sample), then tries every way of joining two parts of 1 + 5, 5 + 1 or 3 + 3 nodes on the first ten points and checks the full sample where those agree with a target.
**For the kinds whose numbers are words the check is on the numbers the words stand for.** The three kinds that copy a word, count words or take a word of
a numeral are not functions of the numbers and are not searched. It covered seeds 1 to 10, so every drawn constant, in both subtraction conventions: 30 targets per seed.

- **Reachable, as intended:** all ten searched controls, in every seed from 1 to 10 and in both conventions (at most 1 to 7 nodes).
- **Not reachable at 7 nodes, any seed 1 to 10:** all 12 `newop` kinds, all 3 `compose` kinds and the five arbitrary codes.
  No kind had to be replaced for being reachable in the end. (Drafts were replaced while the set was written: a composition of the floors-of-a control with itself
  was reachable by a seven-node program on three seeds, because a function that grows slowly collapses when it is applied twice; and a copy of the first of two numerals
  had the table of a copy of set F.)
- **The claim is narrow.** Because inputs stop at 31 (99 or 999 on a few kinds), every kind is a finite table, and some have a longer program; "not reachable at
  7 nodes" is the claim, not "impossible". A learner may also build operations for itself from loops, tables or conditionals, which are not covered. How each kind
  could be learned is in the `learn` column of `key.tsv`.

## Things to know

- Every answer is exact by construction (the tests recompute each lesson and stream answer from the rules of its kind, check the operations against known values,
  and check them again against second implementations written another way). The numerals are checked to read back to the numbers, every number up to the cap of its
  grammar has a numeral of its own, and every run of words made for a trap row is checked to be no numeral of any family.
- The stream asks each fact once; it has no wrong-teacher columns (this is the clean exam).
- The tests of the files: same seed gives the same bytes, every line parses, neither a kind's name, its rule nor its learning note appears in any file that is sent to
  a system, and the only words in the files are the fixed words of the questions, the kinds' words, the numerals, the nine words that only `trap` rows carry and the
  six words that are one letter away from a number word.
- The answers of `trap` rows are told after the answer like any others. A system that learns from them learns the told word or number for that one question; none is
  asked twice.
- A test requires at least five distinct values per kind (three for the two kinds that count words and take the last word of a numeral).
- The second words of the least-significant-first grammar are used in the questions only (never in an answer, and never in the family of the copies), so a copy of a
  numeral is exact in every question.

---

# STOP: do not read below this line before the first run

Everything below names the kinds.

---

## The kinds (33)

`n` is the unit's family of numerals: **d** digits, **1**, **2**, **3** the first, second and third family of words, **w** a single word (copied from a numeral). Where two are given, the first
is the question and the second the answer.

| role | kind | n | rule |
|---|---|---|---|
| `newop` | ratiofloor | d | (p a + q b + c) div (r a + s b + t); p 7 to 14, q 2 to 8, c 1 to 9, r and s 1 to 3, t 2 to 5 (at least 6 values in 0 to 31) |
| `newop` | twomult | d | (p a + q b) mod m; p and q from 11 to 37, m a prime from 101 to 211 |
| `newop` | quadpair | 2 | (a * a + c * a * b + d * b * b) mod m; c 2 to 9, d 2 to 5, m a prime from 59 to 97 |
| `newop` | minform | 2 | min(p a + b, a + q b) + c; p and q from 2 to 6, c 1 to 9 |
| `newop` | bigquot | 1 | (K div (a + c)) mod m; K a five-digit number, c 1 to 6, m 83, 89 or 97 |
| `newop` | sidon | d | the a-th term (a from 1) of the greedy sequence in which all sums of two terms (a term twice allowed) are different: 1, 2, 4, 8, 13, 21, 31, 45, ... |
| `newop` | lookandsay | d | the number of digits of the a-th term (a from 1) of 1, 11, 21, 1211, 111221, ...: the reading aloud of the term before it as runs |
| `newop` | rightcount | 1 | the number of right triangles with whole sides and a hypotenuse of at most a (a from 1) |
| `newop` | sortcube | d | the decimal digits of a * a * a in order from the smallest to the largest, read as a number |
| `newop` | sandpile | 3 / d | a grains on a cell of an endless grid, a cell with 4 or more gives one grain to each neighbor until none has 4: the number of cells left with a grain |
| `newop` | luhn | 1 / d | the Luhn check digit of a (double every second digit from the right, starting with the last; take 9 off a double above 9; the sum's complement to a multiple of 10); a up to the grammar's cap |
| `newop` | niven | d | a div s if a is divisible by s, the sum of its decimal digits, else c * (a mod s) + s; c 2 to 4; a from 1 to 99 |
| `compose` | halvestriple | d | halves(a + b + min(a, b)) |
| `compose` | luhnsum | 1 / d | luhn(a + b), the numbers spelled in the first family, each up to half the cap |
| `compose` | quadtwice | 2 | 2 * quadpair(a, b) |
| `control` | halves | d | a div p + a div q, (p, q) from (2, 3), (2, 5), (3, 4), (3, 5), (4, 5), (2, 7); taught in all eight ways of asking |
| `control` | mix3 | d | p * a + q * b, (p, q) from (3, 2), (2, 5), (4, 3), (5, 2), (3, 4); taught in all eight ways of asking |
| `control` | lastprod | d | (a * b) mod m, m from 7, 9, 10, 11, 12 |
| `control` | triple | d | a + b + min(a, b) |
| `control` | read1 | 1 / d | a spelled in the first family, the answer a in digits (up to the cap, 995 to 999) |
| `control` | write1 | d / 1 | a in digits, the answer a spelled in the first family |
| `control` | sum1 | 1 | a + b, each up to half the cap |
| `control` | scale1 | 1 | k * a, k from 6 to 12, a up to cap div k |
| `control` | sum2 | 2 | a + b, each up to half the cap |
| `control` | double2 | 2 | 2 * a, a up to half the cap |
| `control` | second3 | 3 | the second of the two numerals, copied (numbers up to 299) |
| `control` | size3 | 3 / d | the number of words of the numeral, in digits |
| `control` | lastword3 | 3 / w | the last word of the numeral, copied |
| `unexplainable` | noisedig, noisyline, noise2, noise3, pairnoise3 | d, d, 2, 3, 3 | an arbitrary code (0 to 22) for each number; 4a plus a code of 0 to 2, a line that is not one; an arbitrary code (0 to 299) for each number 0 to 31 in the second family; an arbitrary code (0 to 199) in the third family; an arbitrary code (0 to 99) for each pair in the third family |

How each kind could be learned is in the `learn` column of `key.tsv`; in short, the closed forms are read off rows and columns by differences, the others are
recurrences, searches over small structures, simulations and loops over digits that the tables show. The hard kinds over the third family (`sandpile`) are the ones where
a refusal is expected unless an assignment of numbers to its words that fits every lesson of the family is recovered.

Which is easy for which learner was not measured, and no kind was tuned either way.

Conventions a system could trip over: `sidon`, `lookandsay`, `rightcount` and `niven` start at 1; `sandpile(0)` is 0; `sortcube(10)` is 1 (the zeros fall away at the front);
`luhn(0)` is 0; `bigquot` divides by a + c, never by 0; `read1`, `write1` and `luhn` run to the cap of the first family's grammar (995 to 999: the subtractive
grammar stops where a thousand would be needed); the second words of the least-significant-first grammar are read in every position but the answers use the first
word; the answers of `sum1`, `sum2` and `double2` stay within the cap; `second3` runs to 299; `size3` counts words with repeats; `lastword3` gives a single word, which is not a numeral.
