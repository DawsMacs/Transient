<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 DawsMacs -->
# Hidden worlds K: an eleventh set of exam worlds, with closed forms of new shapes and constants, loops over digits, grids, strings and divisors, six more numeral grammars, mixed forms, and rewordings next to the traps that look like them

`tools/hidden_worlds_k.py` writes an eleventh family of adaptive exams. The files, the line protocol, the shape of a world (invented words for the kinds,
held-out and beyond-range questions with the truth told after each answer) and the roles are those of `tools/hidden_worlds.py` and its nine
successors (see `tools/hidden_worlds.md`). The kinds are all different from those of sets A to J, and the way numbers are written in the questions is new again.

**Why an eleventh set.** Sets A to J are development sets once their kinds have been read. This set was written from the format and the earlier sets alone, without
looking at any system it will be run on. **To keep it a blind test, do not read the list of kinds and the descriptions of the numeral grammars below the marked line (or
`make_kinds` and the grammar classes in the script, or the test file, which checks kinds and grammars by name) before the first run**: run it, look at the numbers, then read on.

```bash
python3 tools/hidden_worlds_k.py DIR --seed 1             # writes lessons.tsv, stream.tsv (+ README.txt, key.tsv)
baby exam adapt-drive DIR COMMAND...                      # judge any system on it, as for the clean adaptive exam
baby exam adapt-run DIR                                   # the baby itself
python3 -m unittest discover -s tools -p 'test_hidden_worlds_k.py'
```

`--compose-first` teaches the compositions before the kinds they use. The generator is standard-library Python 3 with the first set's small
deterministic random generator, so a seed gives the same bytes on every Python version (the seed goes through one extra mix first). It imports the
first generator for the shared pieces; the kinds, the words, the numerals, the ways of asking and the salts are its own. `key.tsv` has five columns:
the word, the role, the kind, the rule and how the kind could be learned (for the numeral families also how the meaning of their words can be worked out, and for the kinds a
short program reaches how that program could be found). It is never sent to a system.

## What differs from the earlier sets (without naming a kind)

- **The kinds.** None of the kinds of sets A to J is reused, by name or by table (a test compares every table of this set with every table of the ten earlier sets,
  controls and codes included, seeds 1 to 5 against seeds 1 to 5; the kinds that read and write words are the identity on a domain of their own, from 1). **Closed forms
  are in**, in shapes none of the earlier sets used, with one, two and **three numbers** and with **one, two and three constants** in different places (a shift inside each of two
  factors, a modulus that depends on a number, a divisor that depends on a remainder of a number, a largest constant of a few hundred, a number that a minimum is taken with, a constant deep
  inside a product): five that need nine or eleven nodes, and, among the controls, five plain closed forms of five to seven nodes that a short program reaches. The other hard kinds are
  structures that no earlier set used (loops with a state over decimal digits, a count on a grid, an order on strings of digits, a number defined from its values at the divisors of
  it, a branch on a property of the number that is neither a threshold nor a remainder; they are listed below the marked line), and two compositions of kinds of this set.
- **Numbers as words, in six more numeral grammars.** 19 of the 32 kinds put their numbers into the question or the answer as words, in numbers up to 999 (333 to 984 where the arithmetic
  of the kind would pass the largest number the grammar writes). Sets F to J used English-style sums of words, positional digit words in many bases, mixed radices, words that multiply a base word,
  bijective and balanced numeration, tally words, per-place digit sets, order-signed words, counts in twenties, fives and ones, bi-quinary places, a word for each two-digit chunk, doubling,
  digit words that change shape before the place words, three remainders, run lengths, a combinatorial number system, Fibonacci words, place value in base minus three and a count down from a landmark.
  **This set uses six grammars none of which is a relative of those**: none is a count of units, a sum of words or place value in a base, and each has a structure of its own (they are
  described below the marked line). Each seed uses three, one for each of three families of words, and a fourth that no kind reads (the grammar of a family is the same for every kind of the
  family; its words are invented, have five letters, begin with a `w`, and are all different from those of the other families and from every word of an earlier set). Over the seeds 1 to 6
  each grammar is the first family once and the second family once; a family in which words repeat inside a numeral is among the three of every seed. A learner is never told that
  the words are numbers. The families differ in what the lessons say about them:
  - **First family (9 kinds, and 1 that draws the form of each number)**: one kind reads a numeral and writes it in digits and one writes a numeral for a number in digits, each
    taught on 110 to 114 numbers (from 1 to a few hundred, with the numbers up to 999 held out or beyond), so the meaning of the words can be worked
    out; the other kinds of the family are computed over the numerals. **A test writes a small decoder that knows only the kind of grammar a family might be (not the values of its words, nor
    the variant of the grammar, of which each grammar has two), finds from those two kinds' lessons alone every assignment of values to the words under which all of the lessons come out true, and checks that there
    is exactly one, that it is the true one, and that it gives every number up to the largest the grammar writes its numeral back**, for each of the seeds 1 to 12 (each grammar twice).
  - **Second family (6 kinds)**: no kind reads or writes it in digits. Kinds computed over the numerals (a sum with the second number tripled and a third of a number) fix the meaning only through the arithmetic. **The
    same decoder, given the lessons of those two kinds alone (no number in digits), finds exactly one assignment and it is the true one**, for each of the seeds 1 to 12 (each grammar twice; a test also
    shows that for two of the six grammars the lessons of the second of the two kinds alone leave more than one). The lessons of the two kinds show every word of the family.
  - **Third family (4 kinds)**: nothing in the lessons can fix the meaning: the kinds over it rearrange or copy words, are arbitrary codes, or are a hard kind defined over
    numbers whose answer is a number that only a learner who already has its rule can relate to the words (a test checks that each answer of the copying kinds follows from the words of
    the question alone).
  - **Fourth family (no kind)**: its words occur only in trap rows, after a question about numbers or in the place of a number.

  Every word of a held-out or beyond row of the first two families occurs in the lessons that tie that family (a test checks every word).
- **Mixed forms.** Three kinds take one number as digits and another as words in the same question (two in one order, one in the other, one of them with the second family) and one kind draws the form of each number
  (as digits or as words) in lessons and stream alike. Twelve kinds answer in another form than the question is in (digits to words, words to digits, words of the first family to words
  of the second, a pair of a number in digits and one in words answered in digits or in words, words of the second or third family to digits). The traps include a kind taught in one
  form and asked in another, and a kind with one number in a form it was never taught with.
- **Rewordings, and which words of a way can go.** There are eight ways of asking a question with one number, eight with two numbers and six with three numbers, with
  words that occur nowhere in the earlier sets' ways (a test compares every text and the start of every way up to the kind's word); some repeat a word of the question, one repeats the number itself, in five of the eight
  (four of the six) one word is a filler that can be left out, and a filler of one way is a necessary word of another. None puts two numbers side by side. Every kind is
  taught in two or three of them, drawn by the seed; **three controls (one for each count of numbers) are taught in all of them and, for each way that has a filler, also with the filler left out for the same facts**
  (13, 13 and 10 ways of asking for the same fact): that is the evidence in the lessons which tells a word that can be dropped from one that cannot. The held-out
  facts of the other 24 kinds with lessons are then asked in one of four ways, six kinds each (three controls and three hard kinds in each): **in a taught way**; **in a way that
  was not taught for the kind** (one only taught for the three controls); **in a mix of the start of one taught way and the end of another** (up to the kind's word the first,
  after it the second); **in a taught way with a filler word left out** (`swiftly`, `again`, `slowly`, `exactly`, `truly`). All four are answerable by a system that treats the wording as phrasing.
  The beyond-range questions are asked in the first taught way. Next to them stand trap rows that look just like these rewordings and cannot be answered: a taught way with a start
  that no way of asking has, with an end that none has, with a number or a word of the ways where the kind's word should be, with the kind's word in the middle of a numeral, with the
  two words before the numbers exchanged (for kinds where the order of the numbers matters), with the start of a way for one count of numbers and the end of a way for another, with the word of another kind where a filler is or after the way, and with
  a necessary word left out. **Rows that leave a word out, by count:** 56 to 60 held-out rows per world (seeds 1 to 6; the number depends on the seed) leave a filler out (answerable: those fillers are shown dropped in the lessons of the three kinds that are taught in every way), and 4 trap rows leave a word out that
  no lesson shows dropped (the kind's word in 2 of them, the word before a number in 2; they must be refused). No other row leaves a word out of a way.
- **Repeats.** Numerals have words that occur more than once (in every one of the six grammars, in a few to most of the numerals), the held-out rows of each kind over many-word numerals include numerals with a repeated word
  (four of them), every kind that takes two or three numbers has held-out rows that name the same
  number more than once (at least three for two numbers, two for three), and a way of asking that names its number twice and ways that repeat a word are among the eight. A system that reads a question as a set of words cannot tell these apart from their shorter relatives.
- **Roles.** Five: `newop` (10 kinds), `compose` (2), `control` (15), `unexplainable` (5), and `trap`. The first four are kinds with lessons. `trap` rows have no
  lessons and no rule; they look like ordinary questions and the right answer is a refusal (126 per world): two kinds in one question (8), a word that was never
  taught (8), a word one letter away from a kind word (4), the wrong number of numbers for a kind (8), numbers in a form the kind was never taught with (12: words
  of the third family for a kind over digits or over another family, digits for a kind over the third), one number of several in such a form (4), a bare number (6), a run of words
  that is no numeral of the grammar, as a run of the wrong length or of a value no number has, a word repeated, or a word one letter away from a number word (18, six of each), a question about numbers followed by a numeral of the fourth family (6) or with one in the place of a number (4), the units of a taught question
  in the opposite order (6), and 42 rows that look like the rewordings above (6 + 6 + 4 + 6 + 6 + 6 + 4 + 4). Their truths are drawn one by one and follow no rule. The `unexplainable` kinds include numerals taught nowhere (arbitrary codes over words of three families, one a pair of
  numbers) and one code that follows a quotient for most numbers and not for the others.
- The stream holds 736 questions on every seed (290 control, 200 newop, 38 compose, 82 unexplainable and 126 trap), after 8217, 7950 and 8260 lessons on seeds 1 to 3.
- The words are two syllables plus the ending `pesu` (the earlier sets end in `sido`, `velu`, `ketu`, `ruvo`, `zane`, `yarn`, `tole`, `dopa`, `cuwe` and `bimo`), so no word of one set is a
  word of another (a test checks seeds 1 to 6 against all ten, and checks that no invented word of this set is any word at all of an earlier exam). The words of the
  numerals have five letters: a `w`, a vowel, a consonant, a vowel and a consonant.
- The controls can be answered from the seen examples by a learner with the starting operations (or, for the spelling and word controls, once the words are
  read or copied): a refusal on them is a defect. The five `unexplainable` kinds have no rule and must stay refused until told, one of them a code that follows a
  quotient for most numbers and so looks like a rule.

## What the baby and a memorizer do on it (seeds 1 to 3)

`baby exam adapt-run DIR` (built from `main` at ae3f297; percent of the questions in the role; fact right / WRONG / guess right / guess wrong / refused):

| role (n) | seed 1 | seed 2 | seed 3 |
|---|---|---|---|
| control (290) | 0.0 / 0.0 / 0.0 / 0.0 / 100.0 | 4.5 / 0.0 / 0.0 / 0.0 / 95.5 | 0.3 / 0.3 / 0.0 / 0.0 / 99.3 |
| newop (200) | 0.0 / 1.5 / 0.0 / 0.0 / 98.5 | 0.0 / 8.0 / 0.0 / 0.0 / 92.0 | 0.0 / 3.0 / 0.0 / 0.0 / 97.0 |
| compose (38) | 0.0 / 0.0 / 0.0 / 0.0 / 100.0 | 0.0 / 0.0 / 0.0 / 0.0 / 100.0 | 0.0 / 0.0 / 0.0 / 0.0 / 100.0 |
| unexplainable (82) | 0.0 / 9.8 / 0.0 / 0.0 / 90.2 | 0.0 / 13.4 / 0.0 / 0.0 / 86.6 | 0.0 / 4.9 / 0.0 / 0.0 / 95.1 |
| trap (126) | 0.0 / 0.8 / 0.0 / 0.0 / 99.2 | 0.0 / 5.6 / 0.0 / 0.0 / 94.4 | 0.0 / 1.6 / 0.0 / 0.0 / 98.4 |
| all (736) | 0.0 / 1.6 / 0.0 / 0.0 / 98.4 | 1.8 / 4.6 / 0.0 / 0.0 / 93.6 | 0.1 / 1.8 / 0.0 / 0.0 / 98.1 |

The baby answers 0.0 to 1.8 % of the questions as facts (0.0 %, 4.5 % and 0.3 % of the controls, 0.0 % of the newop kinds on every seed) and gives no guesses. **It gives wrong facts
on all three seeds**: 12 on seed 1, 34 on seed 2 and 13 on seed 3 (mostly unexplainable kinds, newop kinds and traps), and it answers 1, 7 and 2 trap questions; the run says "FAIL on honesty" on each.
The `baby` is frozen as the reference and was not changed. The energy lines of the run are not recorded here (the machine was not quiet).

`baby exam adapt-drive DIR python3 tools/exam_adapt_baseline.py` (the naive memorizer, which only repeats told answers): 0.0 % right in every role on all
three seeds, 100 % refused, WRONG 0 ("PASS on honesty"). No question in the stream repeats a taught or an earlier question, so memorizing cannot score. The
lessons number 8217, 7950 and 8260 on seeds 1 to 3 (they differ because the kinds are taught in two or three ways of asking and in different grammars).

## Is any kind reachable with the starting operations? (brute force, 7 nodes)

The check used for the earlier sets, kept as a test (`test_no_hard_kind_is_reachable_and_the_controls_are`, which uses the search of the set I test; it needs numpy and takes about eleven minutes for six seeds, so it
is skipped without numpy; `HW_K_SEARCH_SEEDS=N` limits it to seeds 1 to N, the default is 6): every expression of at most 7 nodes (leaves count) over `+ - * min max div mod`, the
variables, and the constants 0 to 40, 50, 64 and 100, deduplicated by value on the sample points, with subtraction both signed and saturating at 0, and division or
remainder by zero taken as 0. Unary kinds were matched on the points 1 to 31, 1 to 63, 1 to 99 or a fixed spread up to 394, by the size of the domain; binary kinds on 60
fixed pairs, half with both numbers up to 24; kinds with three numbers on 40 fixed triples. An exact match on the sample counts as reachable, so the check errs toward calling
a kind reachable. The search builds every function of 1, 3 and 5 nodes (deduplicated by value), then tries every way of joining two parts of 1 + 5, 5 + 1 or 3 + 3 nodes on the
first ten points and checks the full sample where those agree with a target. **For the kinds whose numbers are words the check is on the numbers the words stand for.** The
two kinds that copy or rearrange words are not functions of the numbers and are not searched.

It covered seeds 1 to 6 (every drawn constant, in both subtraction conventions; 30 searched targets per seed, 180 in all):

- **Reachable, as intended:** all thirteen searched controls, in every seed from 1 to 6 and in both conventions, at 1 to 7 nodes (the reading and writing kinds are one node; the five plain closed forms are five or seven nodes; the controls that carry a constant stay within the constants the search offers).
- **Not reachable at 7 nodes, any seed 1 to 6:** all 10 `newop` kinds, both `compose` kinds and the five arbitrary codes (the codes and kinds over words were checked on the numbers the words stand for).
  No kind had to be replaced for being reachable in the end.
- **The claim is narrow.** Because inputs stop at 15, 24, 30, 31, 60, 99 or a few hundred, every kind is a finite table, and some have a longer program; "not reachable at
  7 nodes" is the claim, not "impossible". The constants of the hard closed forms (a prime above 100, a modulus of a few hundred) are partly outside the constants the search offers; those kinds
  are not close to seven nodes either (the five closed forms count to nine or eleven nodes by hand, and the others need a loop, a table, a square test or a recursion over divisors, none of which is among the operations), so the claim does not rest on the constants. A learner may also build
  operations for itself from loops, tables or conditionals, which are not covered. The search finds what it should (the test checks a seven-node kind, a five-node program with
  three numbers, and nothing for a nine-node program or for the median of three). How each kind could be learned is in the `learn` column of `key.tsv`.

## Things to know

- Every answer is exact by construction (the tests recompute each lesson and stream answer from the rules of its kind, check the operations against known values,
  and check them again against second implementations written another way: a walk on complex numbers, Pick's formula, a sort of strings, a list of every ordered product, a bit string).
  The numerals are checked to read back to the numbers against a second reading of
  each grammar, every number up to the cap of its grammar has a numeral of its own, and every run of words made for a trap row is checked to be no numeral of any family.
- The stream asks each fact once; it has no wrong-teacher columns (this is the clean exam).
- The tests of the files: same seed gives the same bytes, every line parses, neither a kind's name, its rule nor its learning note appears in any file that is sent to
  a system, and the only words in the files are the fixed words of the questions, the kinds' words, the numerals, the nine words that only `trap` rows carry, the four
  words that are one letter away from a kind word, the six that are one letter away from a number word and the words of the fourth family (trap rows only).
- The answers of `trap` rows are told after the answer like any others. A system that learns from them learns the told word or number for that one question; none is
  asked twice.
- The "other form" traps are chosen so that no lesson can bridge the forms: a kind over digits, the first or the second family is asked with words of the third
  family, and a kind over the third family is asked with digits. The words of the fourth family are read by no kind.
- A test requires at least five distinct values per kind (three for the two kinds that copy or rearrange words).
- The decoders of the test are not a claim about any learner: they say that the lessons of the first family (110 to 114 facts each of two kinds) and of the second family (about 70 and 110 facts of two kinds) leave one meaning of the words for a reader that has the hypothesis of the grammar, and so that a rule that no line of code names can in
  principle be acquired from them. A learner without the hypothesis has more to do.

---

# STOP: do not read below this line before the first run

Everything below names the kinds and the grammars.

---

## The kinds (32)

`n` is the unit's family of numerals: **d** digits, **m** digits or words of the first family (drawn for each number of a fact), **1**, **2**, **3** the first, second and
third family of words, **w** a run of words (copied from a numeral or rearranged). Where two are given, the first is the question and the second the answer; a pair of letters
before a slash gives the form of each number.

| role | kind | n | rule |
|---|---|---|---|
| `newop` | clipmix | d | (a mod p) * q + a div r, p from 7 to 31, q from 3 to 12, r a prime from 47 to 211 (nine nodes, three constants); from 0 to 999 |
| `newop` | shiftprod | 22 / d | (a + p) * (b + q) mod r, p and q from 3 to 60, r a prime from 211 to 499 (nine nodes, a constant beside each factor and a large one outside); a, b from 0 to 99, in words of the second family |
| `newop` | varquot | 2d / d | (a * p + b) div (b mod q + 1), p from 3 to 30, q from 4 to 12 (eleven nodes, a divisor that depends on a remainder of b); a in words of the second family, b in digits; from 0 to 99 |
| `newop` | capmod | d | min(a * p + b, c * q mod r), p and q from 2 to 9, r a prime from 59 to 199 (eleven nodes, three constants); from 0 to 30 |
| `newop` | modmod | d | (a * b + c) mod (a + p), p from 1 to 30 (nine nodes, a modulus that depends on a number, one constant); from 0 to 30 |
| `newop` | divways | 3 / d | the number of ways to write a as a product of numbers above 1, order counted (a from 1): the sum of the values at the divisors of a below a; a from 1 to 299, in words of the third family |
| `newop` | rootfork | m / d | p * (the integer square root of a) + q if a is a perfect square, else (a mod r) + a div s; p from 3 to 9, q from 1 to 40, r from 7 to 19, s from 9 to 60; a from 0 to 999 |
| `newop` | textrank | d | where a stands among the numbers 1 to b when they are put in the order of the alphabet (as strings of decimal digits): how many of them come at or before it; a, b from 1 to 99 |
| `newop` | gridwalk | d / 1 | a walk on a grid told by the digits: start at (0, 0) facing north; for each digit d from the left turn right d quarter turns and step d cells forward; the answer is |x| + |y| of the end, in words of the first family; a from 0 to 999 |
| `newop` | belowline | d | the points (x, y) of the grid with x, y at least 1 strictly under the line from (a, 0) to (0, b); a, b from 1 to 60 |
| `compose` | chainratio | d | ratshift(offmod(a), offmod(b)); a, b from 0 to 399 |
| `compose` | walkshift | d / 1 | gridwalk(offmod(a)), the answer in words of the first family; a from 0 to 999 |
| `control` | offmod | d | (a + p) mod m, p from 3 to 40, m from 13, 17, 19, 23, 29, 31, 37; from 0 to 399; taught in all eight ways of asking with one number, five of them also with the filler left out |
| `control` | subwrap | d | (a - b) mod m, m from those; from 0 to 31; a difference below zero is taken as the number from 0 up; taught in all eight ways with two numbers, five of them also with the filler left out |
| `control` | peakmod | d | max(a, b) * c mod m; from 0 to 31; taught in all six ways with three numbers, four of them also with the filler left out |
| `control` | ratshift | d | a * p div (b + q), p from 2 to 9, q from 1 to 9; from 0 to 99 |
| `control` | sumquot | d | (a + b) div (c + p), p from 1 to 9; from 0 to 24 |
| `control` | numin1 | 1 / d | a spelled in the first family, the answer a in digits (a from 1 to the cap of the grammar) |
| `control` | numout1 | d / 1 | a in digits, the answer a spelled in the first family |
| `control` | bumpf1 | 1 | a + b div 2, the numbers and the answer spelled in the first family; as far as the answer stays within the cap |
| `control` | blendpair | d1 / d | a + b mod p, p from 7 to 31, a in digits and b in words of the first family, the answer in digits; from 0 to 299 |
| `control` | mixback | 1d / 1 | min(a + p, 3 * b), p from 3 to 40, a in words of the first family and b in digits, the answer in words of the first family |
| `control` | shiftf12 | 1 / 2 | a * p div q, (p, q) one of (5, 4), (7, 6), (3, 4), (5, 6), (2, 3), (7, 8), (8, 9), the number in words of the first family and the answer in words of the second |
| `control` | skewsumf2 | 2 | a + 3 * b, the numbers and the answer spelled as words of the second family, as far as the answer stays within the cap |
| `control` | thirdf2 | 2 | a div 3, a up to the cap, the number and the answer in words of the second family |
| `control` | echo3 | 3 / w | the words of the numeral written twice in a row (a run of words, not a numeral) |
| `control` | oddplaces3 | 3 / w | the words of the numeral that stand in the odd places (the first, the third, ...) |
| `unexplainable` | tagpair, nearquot, noisef1, noisef2, noisef3 | d, d, 1, 2 / d, 3 | an arbitrary code (0 to 19) for each pair of numbers 0 to 15; a div q (q from 3 to 6) for about 60 % of the numbers 0 to 31 and an arbitrary code for the rest (at least 8 of them differ from the quotient); an arbitrary code (0 to 299) for each number 0 to 31 in the first family; an arbitrary code (0 to 54) for each number 0 to 31 in the second family, in digits; an arbitrary code (a numeral for a number from 0 to 199) for each number 0 to 31 in the third family |

How each kind could be learned is in the `learn` column of `key.tsv`; in short, the closed forms are read off rows and columns by differences, by factors or by cycles; the others are a loop with
a state of a position and a heading over digits, a count of points of a grid under a line (a loop over columns), an order on strings of digits (a count of the numbers that come before), a number checked from its values at the divisors of it, a branch on whether the number is a square whose two
branches are a line in the root and a wrapped line, and the controls are short programs found by a search over small expressions or, over words, copies and rearrangements. `divways` is the hard kind
over the third family: a refusal is expected unless an assignment of numbers to its words that fits every lesson of the family is recovered. The `unexplainable` kinds over words are the "numeral
taught nowhere" rows: a valid numeral for which no lesson gives an answer.

## The six numeral grammars

Every grammar writes the numbers 0 up to 999 (the cap) and has a numeral for each; the variants change the order of the words, chosen by the seed (two for each grammar).

- **Quot** (cap 999): the number is named by two factors and a remainder: n = q * q2 + r with q the integer square root of n, q2 = n div q (so q, q + 1 or q + 2) and r = n mod q; three words from one set of
  thirty-three digit words (values 0 to 32) for the three places, the remainder first or the root first. Zero is the zero word three times. Words repeat for the squares (q, q, 0) and wherever q2 = q.
- **Morton** (cap 999): the bits of the number shared out between two words: the bits at the even places (counted from the least) make the first number, those at the odd places the second; one set of thirty-two
  words (values 0 to 31), the even places first or the odd places first. Words repeat when the two share the same bits.
- **Hilbert** (cap 999): the number is a place along a Hilbert curve that fills a square of 32 by 32 cells, written as the coordinates of the cell (x first or y first); one set of thirty-two words. Neighboring numbers are neighboring cells (one word changes by one).
- **Gray** (cap 999): the reflected Gray code in base four: four digit words, the first digit as in the number and each next digit reflected when the digit before it in the number is odd, so that neighboring numbers differ in one word; no zero in front;
  the highest digit first or the lowest first. Up to five words, most numerals with a word that repeats.
- **Skew** (cap 999): place value with the weights 1, 3, 7, 15, 31, ... (2 to the power k, minus 1) and the digits 0, 1, 2, chosen greedily from the heaviest weight, so that a 2 can stand only at the lowest digit that is not 0; three words; up to nine words, most
  with words that repeat; the heaviest digit first or the lightest first.
- **Diffs** (cap 999): the base-seven digits of the number, each written as how far it is above the digit before it modulo 7 (the first as it is); seven words; up to four words; from the top, or from the bottom with each digit a step above the one below.

Seeds 1 to 3 use for the first, second and third family: Morton, Quot and Gray (fourth: Hilbert); Quot, Gray and Hilbert (fourth: Skew); Gray, Hilbert and Skew (fourth: Diffs).
Seeds 4 to 6 go on round the same ring (Hilbert, Skew, Diffs / Skew, Diffs, Morton / Diffs, Morton, Quot).

In the key, the notes of a kind over the first or the second family say how the meaning of the words can be worked out: for the first family from the lessons that give the numeral and the number (every word
is lined up with its place in the formula of the grammar), for the second from the arithmetic lessons alone (the numerals of small numbers name the words
in order, and the sums carry as the grammar carries). The decoder of the test does that search by trying values for the words of the lesson with fewest unknown words, and finds one assignment.

Which is easy for which learner was not measured, and no kind was tuned either way.

Conventions a system could trip over: `numin1` and `numout1` start at 1 (the zero word is shown by the other kinds of the family); `divways` and `textrank` start at 1; `textrank(a, b)` counts the numbers 1 to b that come at or before a, whether or not a is in 1 to b; `gridwalk` reads the decimal digits of the number without zeros in front
(the number 0 is the digit 0: no turn, no step); `divways(1)` is 1 (the empty product) and `divways(p)` is 1 for a prime p; `belowline` counts only points with both coordinates at least 1 and strictly under the line (so 0 for a = 1 or b = 1);
`subwrap` takes a remainder of a negative difference as the number from 0 up (-1 mod 31 is 30); `bumpf1`, `skewsumf2`, `mixback` and `shiftf12` stay within the cap of their grammars; `echo3` and `oddplaces3` give a run of words, which is not always a numeral;
`rootfork` takes each number either way, drawn for each number of each fact (about half and half, the same in the lessons and in the stream); in the Skew grammar the heaviest weight that fits is always used first, so 4 is 3 + 1 and 2 is the digit 2 alone; in the Gray grammar 4 is the words 1 3; in the Quot grammar 3 is 1 3 0 (q = 1, q2 = 3, r = 0)
and 960 is 30 32 0.
