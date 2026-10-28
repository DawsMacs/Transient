<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 DawsMacs -->
# Hidden worlds J: a tenth set of exam worlds, with closed forms of new shapes, loops over digits, grids and subsets, six more numeral grammars, mixed forms, and rewordings next to the traps that look like them

`tools/hidden_worlds_j.py` writes a tenth family of adaptive exams. The files, the line protocol, the shape of a world (invented words for the kinds,
held-out and beyond-range questions with the truth told after each answer) and the roles are those of `tools/hidden_worlds.py` and its eight
successors (see `tools/hidden_worlds.md`). The kinds are all different from those of sets A to I, and the way numbers are written in the questions is new again.

**Why a tenth set.** Sets A to I are development sets once their kinds have been read. This set was written from the format and the earlier sets alone, without
looking at the learner it will be run on. **To keep it a blind test, do not read the list of kinds and the descriptions of the numeral grammars below the marked line (or
`make_kinds` and the grammar classes in the script, or the test file, which checks kinds and grammars by name) before the first run**: run it, look at the numbers, then read on.

```bash
python3 tools/hidden_worlds_j.py DIR --seed 1             # writes lessons.tsv, stream.tsv (+ README.txt, key.tsv)
baby exam adapt-drive DIR COMMAND...                      # judge any system on it, as for the clean adaptive exam
baby exam adapt-run DIR                                   # the baby itself
python3 -m unittest discover -s tools -p 'test_hidden_worlds_j.py'
```

`--compose-first` teaches the compositions before the kinds they use. The generator is standard-library Python 3 with the first set's small
deterministic random generator, so a seed gives the same bytes on every Python version (the seed goes through one extra mix first). It imports the
first generator for the shared pieces; the kinds, the words, the numerals, the ways of asking and the salts are its own. `key.tsv` has five columns:
the word, the role, the kind, the rule and how the kind could be learned (for the numeral families also how the meaning of their words can be worked out). It is never sent to a system.

## What differs from the earlier sets (without naming a kind)

- **The kinds.** None of the kinds of sets A to I is reused, by name or by table (a test compares every table of this set with every table of the nine earlier sets,
  controls and codes included, seeds 1 to 5 against seeds 1 to 5; the kinds that read and write words, add them and halve them are the identity, a sum and a floor of a
  half on a larger domain than the spelling controls of earlier sets, so on the first 32 numbers those agree with an earlier control by construction). **Closed forms
  are in**, in shapes none of the earlier sets used, with one, two and **three numbers**, some with large constants, next to controls of the same families that a short
  program reaches. The other hard kinds are structures that no earlier set used (loops with a state over decimal digits, a table built over a grid, an operation on the
  string of digits, a number defined by its values at fractions of it, a conditional on a property of the number that is neither a threshold nor a remainder, a count made
  by a loop over the elements of a set; listed below the marked line), and two compositions of kinds of this set.
- **Numbers as words, in six more numeral grammars.** 18 of the 32 kinds put their numbers into the question or the answer as words, in numbers up to 990 to 999. Sets F to I
  used English-style sums of words, positional digit words in many bases, mixed radices, words that multiply a base word, bijective and balanced numeration, tally words,
  per-place digit sets, order-signed words, counts in twenties, fives and ones, bi-quinary places, a word for each two-digit chunk, doubling, and digit words that change
  shape before the place words. **This set uses six grammars of which four are of another structure than any of those** (a numeral that is not a count of units, not a sum of
  words and not place value; they are described below the marked line); the other two are variations on a sum of words and on place value. Each seed uses three, one for each of three families of words, and a fourth that no kind reads (the grammar of a family is the same for every kind of the family; its
  words are invented, have five letters, begin with an `f`, and are all different from those of the other families and from every word of an earlier set). Over the seeds 1 to 6
  each grammar is the first family once and the second family once; a family in which words repeat inside a numeral is among the three of every seed. A learner is never told that
  the words are numbers. The families differ in what the lessons say about them:
  - **First family (9 kinds, and 1 that draws the form of each number)**: one kind reads a numeral and writes it in digits and one writes a numeral for a number in digits, each
    taught on 110 to 115 numbers (110 and the few more that show every word, with the numbers up to a few hundred or beyond held out), so the meaning of the words can be worked
    out; the other kinds of the family are computed over the numerals. **A test writes a small decoder that knows only the kind of grammar a family might be (not the values of its words, nor
    the variant of the grammar), finds from those two kinds' lessons alone every assignment of values to the words under which all of the lessons come out true, and checks that there
    is exactly one and that it gives every number up to the largest the grammar writes its numeral back**, for each of the six seeds (each grammar once).
  - **Second family (5 kinds)**: no kind reads or writes it in digits. Kinds computed over the numerals (a sum and a halving, and others) fix the meaning only through the arithmetic. **The
    same decoder, given the lessons of the sum and the halving alone (no number in digits), finds exactly one assignment and it is the true one**, for each of the six seeds (each grammar
    once). The lessons of the sum and the halving show every word of the family.
  - **Third family (4 kinds)**: nothing in the lessons can fix the meaning: the kinds over it rearrange or copy words, are arbitrary codes, or are a hard kind defined over
    numbers whose answer is a number that only a learner who already has its rule can relate to the words (a test checks that each answer of the copying kinds follows from the words of
    the question alone).
  - **Fourth family (no kind)**: its words occur only in trap rows, after a question about numbers.

  Every word of a held-out or beyond row of the first two families occurs in the lessons that tie that family (a test checks every word, the zero word of a grammar that has one included).
- **Mixed forms.** Three kinds take one number as digits and another as words in the same question (in either order) and one kind draws the form of each number (as digits
  or as words) in lessons and stream alike. Eleven kinds answer in another form than the question is in (digits to words, words to digits, words of the first family to words
  of the second, a pair of a number in digits and one in words answered in digits or in words, words of the second or third family to digits). The traps include a kind taught in one
  form and asked in another, and a kind with one number in a form it was never taught with.
- **Rewordings, and traps that look like them.** There are eight ways of asking a question with one number, eight with two numbers and six with three, with
  words that occur nowhere in the earlier sets' ways (a test compares every text and the start of every way up to the kind's word); some repeat a word of the question, and none puts two numbers side by side. Every kind is
  taught in two or three of them, drawn by the seed; three controls (one for each count of numbers) are taught in all of them, which shows that the ways mean the same. The held-out
  facts of the other 24 kinds with lessons are then asked in one of four ways, six kinds each (three controls and three hard kinds in each): **in a taught way**; **in a way that
  was not taught for the kind** (one only taught for the three controls); **in a mix of the start of one taught way and the end of another** (up to the kind's word the first,
  after it the second); **in a taught way with a filler word left out** (`promptly`, `meanwhile`, `please`, `what`). All four are answerable by a system that treats the wording as phrasing.
  The beyond-range questions are asked in the first taught way. Next to them stand trap rows that look just like these rewordings and cannot be answered: a taught way with a start
  that no way of asking has, with an end that none has, with a number or a word of the ways where the kind's word should be, with the kind's word in the middle of a numeral, with the
  two words before the numbers exchanged (for kinds where the order of the numbers matters), with the start of a way for one count of numbers and the end of a way for another, and with
  a necessary word left out.
- **Repeats.** Numerals have words that occur more than once (in three of the six grammars; in the other three no word occurs twice in a numeral, one numeral apart, and a test counts the held-out rows that do), the held-out rows of each kind over many-word numerals include numerals with a repeated word
  (four of them, for the grammars that have any), and every kind that takes two or three numbers has held-out rows that name the same
  number more than once (at least three for two numbers, two for three). A system that reads a question as a set of words cannot tell these apart from their shorter relatives.
- **Roles.** Five: `newop` (10 kinds), `compose` (2), `control` (15), `unexplainable` (5), and `trap`. The first four are kinds with lessons. `trap` rows have no
  lessons and no rule; they look like ordinary questions and the right answer is a refusal (118 per world): two kinds in one question (8), a word that was never
  taught (8), a word one letter away from a kind word (4), the wrong number of numbers for a kind (8), numbers in a form the kind was never taught with (12: words
  of the third family for a kind over digits or over another family, digits for a kind over the third), one number of several in such a form (4), a bare number (6), a run of words
  that is no numeral of the grammar, as a word repeated, an order or a word the grammar does not allow, or a word one letter away from a number word (18, six of each), a question about numbers followed by a numeral of the fourth family (6), the units of a taught question
  in the opposite order (6), and 38 rows that look like the rewordings above (6 + 6 + 6 + 6 + 6 + 4 + 4). Their truths are drawn one by one and follow no rule. The `unexplainable` kinds include numerals taught nowhere (arbitrary codes over words of three families, one a pair of
  numerals) and one code that follows a remainder for most numbers and not for the others.
- The stream holds 726 questions on every seed (289 control, 200 newop, 37 compose, 82 unexplainable and 118 trap), after 6622, 6367 and 6231 lessons on seeds 1 to 3.
- The words are two syllables plus the ending `bimo` (the earlier sets end in `sido`, `velu`, `ketu`, `ruvo`, `zane`, `yarn`, `tole`, `dopa` and `cuwe`), so no word of one set is a
  word of another (a test checks seeds 1 to 6 against all nine, and checks that no invented word of this set is any word at all of an earlier exam). The words of the
  numerals have five letters: an `f`, a vowel, two different consonants and a vowel.
- The controls can be answered from the seen examples by a learner with the starting operations (or, for the spelling and word controls, once the words are
  read or copied): a refusal on them is a defect. The five `unexplainable` kinds have no rule and must stay refused until told, one of them a code that follows a
  remainder for most numbers and so looks like a rule.

## What the baby and a memorizer do on it (seeds 1 to 3)

`baby exam adapt-run DIR` (built from `main` at 572a6a5; percent of the questions in the role; fact right / WRONG / guess right / guess wrong / refused):

| role (n) | seed 1 | seed 2 | seed 3 |
|---|---|---|---|
| control (289) | 0.7 / 2.4 / 0.0 / 0.0 / 96.9 | 0.3 / 3.8 / 0.0 / 0.0 / 95.8 | 0.3 / 8.0 / 0.0 / 0.0 / 91.7 |
| newop (200) | 0.5 / 4.5 / 0.0 / 0.0 / 95.0 | 1.5 / 2.5 / 0.0 / 0.0 / 96.0 | 0.0 / 6.0 / 0.0 / 0.0 / 94.0 |
| compose (37) | 0.0 / 0.0 / 0.0 / 0.0 / 100.0 | 0.0 / 0.0 / 0.0 / 0.0 / 100.0 | 0.0 / 0.0 / 0.0 / 0.0 / 100.0 |
| unexplainable (82) | 0.0 / 14.6 / 0.0 / 0.0 / 85.4 | 0.0 / 14.6 / 0.0 / 0.0 / 85.4 | 0.0 / 4.9 / 0.0 / 0.0 / 95.1 |
| trap (118) | 0.0 / 2.5 / 0.0 / 0.0 / 97.5 | 0.0 / 2.5 / 0.0 / 0.0 / 97.5 | 0.0 / 4.2 / 0.0 / 0.0 / 95.8 |
| all (726) | 0.4 / 4.3 / 0.0 / 0.0 / 95.3 | 0.6 / 4.3 / 0.0 / 0.0 / 95.2 | 0.1 / 6.1 / 0.0 / 0.0 / 93.8 |

The baby answers 0.1 to 0.6 % of the questions as facts (0.7 %, 0.3 % and 0.3 % of the controls, 0.5 %, 1.5 % and 0.0 % of the newop kinds) and gives no guesses. **It gives wrong facts
on all three seeds**: 31 on seed 1, 31 on seed 2 and 44 on seed 3 (spread over every role, traps and codes included), and it answers 3, 3 and 5 trap questions; the run says "FAIL on honesty" on each.
The `baby` is frozen as the reference and was not changed. Its wrong facts are rows whose question or answer holds words of the families as well as rows over digits; the offenders the run lists
(the first eight in the stream) say it answers a copy or a rearrangement of words, a code, or a number that follows no lesson. The energy lines of the run are not recorded here (the machine was not quiet).

`baby exam adapt-drive DIR python3 tools/exam_adapt_baseline.py` (the naive memorizer, which only repeats told answers): 0.0 % right in every role on all
three seeds, 100 % refused, WRONG 0 ("PASS on honesty"). No question in the stream repeats a taught or an earlier question, so memorizing cannot score. The
lessons number 6622, 6367 and 6231 on seeds 1 to 3 (they differ because the kinds are taught in two or three ways of asking and in different grammars).

## Is any kind reachable with the starting operations? (brute force, 7 nodes)

The check used for the earlier sets, kept as a test (`test_no_hard_kind_is_reachable_and_the_controls_are`, which uses the search of the set I test; it needs numpy and takes about a quarter of an hour for six seeds, so it
is skipped without numpy; `HW_J_SEARCH_SEEDS=N` limits it to seeds 1 to N, the default is 6): every expression of at most 7 nodes (leaves count) over `+ - * min max div mod`, the
variables, and the constants 0 to 40, 50, 64 and 100, deduplicated by value on the sample points, with subtraction both signed and saturating at 0, and division or
remainder by zero taken as 0. Unary kinds were matched on the points 1 to 31, 1 to 63, 1 to 99 or a fixed spread up to 394, by the size of the domain; binary kinds on 60
fixed pairs, half with both numbers up to 24; kinds with three numbers on 40 fixed triples. An exact match on the sample counts as reachable, so the check errs toward calling
a kind reachable. The search builds every function of 1, 3 and 5 nodes (deduplicated by value), then tries every way of joining two parts of 1 + 5, 5 + 1 or 3 + 3 nodes on the
first ten points and checks the full sample where those agree with a target. **For the kinds whose numbers are words the check is on the numbers the words stand for.** The
two kinds that copy a word or rearrange words are not functions of the numbers and are not searched.

It covered seeds 1 to 6 (every drawn constant, in both subtraction conventions; 30 searched targets per seed):

- **Reachable, as intended:** all thirteen searched controls, in every seed from 1 to 6 and in both conventions (at most 7 nodes; the controls that carry a constant stay within the constants the search offers).
- **Not reachable at 7 nodes, any seed 1 to 6:** all 10 `newop` kinds, both `compose` kinds and the five arbitrary codes (the codes and kinds over words were checked on the numbers the words stand for).
  No kind had to be replaced for being reachable in the end. (Drafts were changed while the set was written for other reasons: a numeral grammar with a word set of 29 words was widened to 45 so that
  it writes numbers to 990, and one of the hard kinds over the second family was moved from digits to words of that family so that the family has a hard kind computed over it.)
- **The claim is narrow.** Because inputs stop at 15, 20, 30, 31, 60, 99 or a few hundred, every kind is a finite table, and some have a longer program; "not reachable at
  7 nodes" is the claim, not "impossible". The constants of the hard closed forms (an offset, a prime) are partly outside the constants the search offers; those kinds
  are not close to seven nodes either (the two polynomial closed forms count to well over ten nodes by hand, and the others need a root, a power, a loop or a table, none of which is among the operations), so the claim does not rest on the constants. A learner may also build
  operations for itself from loops, tables or conditionals, which are not covered. The search finds what it should (the test checks a seven-node kind, a five-node program with
  three numbers, and nothing for a nine-node program or for the median of three). How each kind could be learned is in the `learn` column of `key.tsv`.

## Things to know

- Every answer is exact by construction (the tests recompute each lesson and stream answer from the rules of its kind, check the operations against known values,
  and check them again against second implementations written another way: a sum over the periods, a walk over every path of a small grid, every subset of a small set, every rotation,
  every stretch of digits, a sieve, trial division). The numerals are checked to read back to the numbers against a second reading of
  each grammar, every number up to the cap of its grammar has a numeral of its own, and every run of words made for a trap row is checked to be no numeral of any family.
- The stream asks each fact once; it has no wrong-teacher columns (this is the clean exam).
- The tests of the files: same seed gives the same bytes, every line parses, neither a kind's name, its rule nor its learning note appears in any file that is sent to
  a system, and the only words in the files are the fixed words of the questions, the kinds' words, the numerals, the nine words that only `trap` rows carry, the four
  words that are one letter away from a kind word, the six that are one letter away from a number word and the words of the fourth family (trap rows only).
- The answers of `trap` rows are told after the answer like any others. A system that learns from them learns the told word or number for that one question; none is
  asked twice.
- The "other form" traps are chosen so that no lesson can bridge the forms: a kind over digits, the first or the second family is asked with words of the third
  family, and a kind over the third family is asked with digits. The trailing words of the fourth family are read by no kind.
- A test requires at least five distinct values per kind (three for the two kinds that copy or rearrange words).
- The decoders of the test are not a claim about any learner: they say that the lessons of the first family (110 to 115 facts each of two kinds) and of the second family (the
  sum and the halving, 70 to 76 and 110 to 115 facts) leave one meaning of the words for a reader that has the hypothesis of the grammar, and so that a rule that no line of code names can in
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
| `newop` | rootprod | 1d / d | the integer square root of a * b + k, k from 100 to 999; a in words of the first family, b in digits; from 0 to 99 |
| `newop` | powthird | d | a to the power b modulo c; a, b, c from 2 to 30 |
| `newop` | remtotal | d | the sum of k mod b for k from 1 to a; a from 1 to 60, b from 1 to 60 |
| `newop` | progsum | d | the first a terms of the progression that starts at b and grows by c: a * b + c * (a * (a - 1) div 2); from 0 to 30 |
| `newop` | risedigits | d | the largest total of a stretch of decimal digits that rise strictly from left to right (one digit counts); a from 0 to 999 |
| `newop` | gridblocked | 2 / d | the paths of unit steps right or up from (0, 0) to (a, b) that avoid every cell (i, j) with i * j mod m = r, counted modulo a prime p; m 5 or 7, r from 1 to m - 1, p from 59 to 211; a, b from 0 to 30, in words of the second family |
| `newop` | turnring | d | the smallest number that the decimal digits of a make when they are turned round as a ring (a rotation may begin with 0); a from 0 to 999 |
| `newop` | splitcoin | 3 / d | f(a) = a for a < 12, else max(a, f(a div 2) + f(a div 3) + f(a div 4)); a from 0 to 399, in words of the third family |
| `newop` | primefork | m / d | p * a + c if a is prime, else a div (its least prime factor) (a for 0 and 1); p from 2 to 6, c from 1 to 30; a from 0 to 199 |
| `newop` | subsetcount | d | how many subsets of {1, ..., a} (the empty one included) have a sum divisible by b; a, b from 1 to 20 |
| `compose` | weighed | d | rankword(topper(a), topper(b)); a, b from 0 to 99 |
| `compose` | remtop | d / 1 | remtotal(a, topper(a)), the answer in words of the first family; a from 1 to the largest whose answer the grammar writes |
| `control` | rankword | d | a mod p + b div q, p from 3 to 12, q from 2 to 9; from 0 to 99 |
| `control` | mulacc | d | (a * b + c) mod m, m from 13, 17, 19, 23, 29, 31, 37; from 0 to 15 |
| `control` | sheaf | d | (a + b) * c mod m, m another of those; from 0 to 31; taught in all six ways of asking with three numbers |
| `control` | topper | d | max(a mod p, a div q), p from 5 to 12, q from 3 to p - 1; a from 0 to 99; taught in all eight ways of asking with one number |
| `control` | foldpair | d | p * a + max(a, b), p from 2 to 9; from 0 to 31; taught in all eight ways of asking with two numbers |
| `control` | readf1 | 1 / d | a spelled in the first family, the answer a in digits (up to the cap of the grammar) |
| `control` | writef1 | d / 1 | a in digits, the answer a spelled in the first family |
| `control` | addf1 | 1 | a + b, each up to half the cap |
| `control` | quarterpair | d1 / d | a + b div q, q from 3 to 6, a in digits and b in words of the first family, the answer in digits |
| `control` | capwrite | 1d / 1 | min(a, m * b), m from 2 to 4, a in words of the first family and b in digits, the answer in words of the first family |
| `control` | stepf12 | 1 / 2 | a + a div d + c, d from 3 to 5, c from 1 to 9, the number in words of the first family and the answer in words of the second |
| `control` | joinf2 | 2 | a + b, each up to half the cap |
| `control` | halvef2 | 2 | a div 2, a up to the cap |
| `control` | swapends3 | 3 / w | the words of the numeral with the first and the last exchanged (a run of words, not always a numeral; the word itself for a numeral of one word) |
| `control` | penult3 | 3 / w | the second word from the end of the numeral, copied (the word itself for a numeral of one word) |
| `unexplainable` | tagcode, nearmod6, scatterf1, scatterf2, scatterpairf3 | d, d, 1, 2 / d, 3 | an arbitrary code (0 to 18) for each number 0 to 31; a mod m (m from 5, 6, 8, 9) for about 60 % of the numbers 0 to 31 and an arbitrary code for the rest (at least 8 of them differ from the remainder); an arbitrary code (0 to 299) for each number 0 to 31 in the first family; an arbitrary code (0 to 54) for each number 0 to 31 in the second family, in digits; an arbitrary code for each pair of numbers 0 to 15 in the third family (0 to 199) |

How each kind could be learned is in the `learn` column of `key.tsv`; in short, the closed forms are read off rows and columns by differences, by factors or by cycles; the others are loops with
a state over digits, a table built over a grid, an operation on the string of digits, a number checked from its values at fractions of it, a conditional on a property of the number whose two
branches are a line and a cofactor, and a count with a state of counts over a set, and the controls are short programs or, over words, copies and rearrangements. `splitcoin` is the hard kind
over the third family: a refusal is expected unless an assignment of numbers to its words that fits every lesson of the family is recovered. The `unexplainable` kinds over words are the "numeral
taught nowhere" rows: a valid numeral for which no lesson gives an answer.

## The six numeral grammars

Every grammar writes the numbers 0 up to 990 or 999 (the cap) and has a numeral for each; the variants change the order of the words (or of the moduli), chosen by the seed.

- **Residue** (cap 999): three words, the remainders of the number modulo 7, 11 and 13 (in one of three orders); one set of thirteen digit words for the three places, so the place says the modulus
  and a word for 12 cannot stand where the modulus is 7 or 11. A numeral is always three words; words repeat when two remainders are equal. Not place value, not a sum, not a count and its unit.
- **Runs** (cap 999): a word for each run length 1 to 9 and a word for zero that stands alone; the numeral is the lengths of the runs of equal digits in the binary form from the top (the first
  run is of 1s), or from the bottom in the other variant. Up to ten words; words repeat.
- **Comb** (cap 999): the combinatorial number system C(x, 3) + C(y, 2) + z with x > y > z >= 0; twenty digit words 0 to 19 for all three places, written falling or rising. No word repeats
  inside a numeral.
- **Countdown** (cap 990): a number n is T(k) - r with T(k) = k * (k + 1) div 2 the least triangular number at or above n and r below k; forty-five digit words 0 to 44 for the two places, the
  landmark first or the shortfall first; the numeral of 0 is the word for 0 twice. A word repeats only there.
- **Zeckendorf** (cap 999): one word for each of the Fibonacci numbers 1, 2, 3, 5, ..., 987 and a word for zero that stands alone; the numeral lists the words of the Fibonacci numbers whose
  sum is the number, no two neighbors of the sequence together (largest first, or smallest first). No word repeats inside a numeral.
- **Negabase** (cap 999): place value in base minus three with a word for each of 0, 1, 2, highest digit first or lowest digit first, no zero in front; up to seven words, most with a word
  that repeats.

Seeds 1 to 3 use for the first, second and third family: Residue, Comb and Runs (fourth: Zeckendorf); Comb, Runs and Zeckendorf (fourth: Negabase); Runs, Zeckendorf and Negabase (fourth: Countdown).
Seeds 4 to 6 go on round the same ring (Zeckendorf, Negabase, Countdown / Negabase, Countdown, Residue / Countdown, Residue, Comb).

In the key, the notes of a kind over the first or the second family say how the meaning of the words can be worked out: for the first family from the lessons that give the numeral and the number (every word
is lined up with its place in the formula of the grammar), for the second from the sums and halvings alone (the numeral of 0 is the one that adds nothing, the numerals of small numbers name the words
in order, and the sums carry as the grammar carries). The decoder of the test does that search by trying values for the words of the lesson with fewest unknown words, and finds one assignment.

Which is easy for which learner was not measured, and no kind was tuned either way.

Conventions a system could trip over: `remtotal` and `powthird` start at 1 and 2 (`remtotal(a, b)` with b from 1); `subsetcount` counts the empty subset (so `subsetcount(a, 1)` is 2 to the power a); `primefork` takes
the second branch for 0 and 1 and gives the number itself there; `risedigits` and `turnring` read the decimal digits of the number without zeros in front; `turnring` may give a rotation that begins with 0 as a smaller
number; `splitcoin(a)` is `a` for a below 12; `gridblocked(0, 0)` is 1 (the start cell is never blocked) and a blocked end cell gives 0; `readf1` and `writef1` run to the cap of the first family's grammar;
`addf1`, `joinf2`, `halvef2` and `stepf12` stay within the cap of their grammars; `swapends3` and `penult3` give a run of words, which is not always a numeral, and keep a one-word numeral as it is;
`quarterpair`, `rootprod` and `capwrite` take one number in digits and one in words; `primefork` takes each number either way, drawn for each number of each fact (about half and half, the same in the lessons and in the stream);
in the Zeckendorf grammar the largest Fibonacci number that fits is always used first, so 4 is 3 + 1 and never 2 + 2; in the Negabase grammar 3 is the digits 1 2 0 and a numeral of two words is always below zero (so it is no numeral).
