<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 DawsMacs -->
# Hidden worlds I: a ninth set of exam worlds, with new closed forms, rules over the digits, six more kinds of numeral, mixed forms, and rewordings next to the traps that look like them

`tools/hidden_worlds_i.py` writes a ninth family of adaptive exams. The files, the line protocol, the shape of a world (invented words for the kinds,
held-out and beyond-range questions with the truth told after each answer) and the roles are those of `tools/hidden_worlds.py` and its seven
successors (see `tools/hidden_worlds.md`). The kinds are all different from those of sets A to H, and five things are new in how the exam is put.

**Why a ninth set.** Sets A to H are development sets once their kinds have been read. This set was written without looking at the learner it will
be run on. **To keep it a blind test, do not read the list of kinds below the marked line (or `make_kinds` in the script, or the test file, which
checks kinds by name) before the first run**: run it, look at the numbers, then read on.

```bash
python3 tools/hidden_worlds_i.py DIR --seed 1             # writes lessons.tsv, stream.tsv (+ README.txt, key.tsv)
baby exam adapt-drive DIR COMMAND...                      # judge any system on it, as for the clean adaptive exam
baby exam adapt-run DIR                                   # the baby itself
python3 -m unittest discover -s tools -p 'test_hidden_worlds_i.py'
```

`--compose-first` teaches the compositions before the kinds they use. The generator is standard-library Python 3 with the first set's small
deterministic random generator, so a seed gives the same bytes on every Python version (the seed goes through one extra mix first). It imports the
first generator for the shared pieces; the kinds, the words, the numerals, the ways of asking and the salts are its own. `key.tsv` has five columns:
the word, the role, the kind, the rule and how the kind could be learned. It is never sent to a system.

## What differs from the earlier sets (without naming a kind)

- **The kinds.** None of the kinds of sets A to H is reused, by name or by table (a test compares every table of this set with every table of the eight
  earlier sets, controls and codes included, seeds 1 to 5 against seeds 1 to 5; the kinds that read and write words, add them and halve them are the identity, a
  sum and a floor of a half on a larger domain than the spelling controls of earlier sets, so on the first 32 numbers those agree with an earlier control by
  construction). **Closed forms are in**, in shapes none of the earlier sets used, with one, two and **three numbers**, next to controls of the same families that a
  short program reaches; their constants are drawn from the seed and several are large. The other hard kinds are structures that no earlier set used (loops with a
  state over binary digits and over a pair, a count over a property of the base-3 digits, a conditional on a property of the binary digits, a number defined by an
  equation it satisfies, a sum over the sizes of square on a grid; listed below the marked line), and two compositions of kinds of this set.
- **Numbers as words, in six more kinds of numeral.** 20 of the 32 kinds put their numbers into the question or the answer as words, in numbers up to 199 to
  999. Sets F, G and H used English-style sums of words, positional digit words, a mixed radix of three words, words that multiply a base word, bijective numeration,
  balanced digits, tally words, per-place digit sets, order-signed words and a count in twenties with a leftover. This set uses grammars of other kinds, six of
  which exist; each seed uses three, one for each of three families of words, and a fourth that no kind reads (the grammar of a family is the same for every kind of
  the family; its words are invented, have four letters, begin with a letter that no word of an earlier set has and are all different from those of the other families):
  - **twenties with a ten word and a fifteen word**: digits of base 20 (1 to 9, a word for ten and one for fifteen, 11 to 14 as ten and a digit, 16 to 19 as fifteen
    and a digit), a word for twenty that closes the count of twenties; in one variant the leftover comes first and the word for twenty stands in front of its count.
    Numbers up to 399.
  - **bi-quinary places**: decimal places of up to four hundreds, a word for each of 1 to 4, a word for five of its own in the units and **another for the five of the
    tens**, a word that closes the tens and one for the hundreds; a digit from 5 to 9 is the five and the word of the rest (the rest first in one variant). Numbers up to 499.
  - **doubling**: a word for one, a word for "times two" and a word for "and one": the binary digits of the number from the second on, so that a **repeated word means
    doubling again** (numerals of up to 17 words). Numbers up to 999.
  - **a word for each two-digit chunk**: 100 invented words for the chunks 0 to 99; a number below 100 is one word, a larger one the word of the hundreds and the word of
    the last two digits (in the other order in one variant). Numbers up to 999.
  - **agreement**: decimal places with a word for each digit, for ten and for hundred, each place a digit and its place word; **a digit word changes its last vowel**
    (to i before the ten word, to e before the hundred word), so each digit has three forms. Numbers up to 999.
  - **twenties, fives, ones**: a count of twenties and the word for twenty, a count of fives and the word for five, a lone digit word for the ones, and a link word
    between the twenties and the rest (the rest first in one variant); the surface form is not that of the earlier twenties grammar. Numbers up to 199.

  Seed 1 uses twenties with ten and fifteen words, doubling and bi-quinary places for its first, second and third family (and chunk words for the fourth); seed 2
  uses agreement, twenties-fives-ones and chunk words (fourth: bi-quinary); seed 3 uses twenties-fives-ones, bi-quinary places and doubling (fourth: agreement). A
  learner is never told that the words are numbers. The families differ in what the lessons say about them:
  - **First family (10 kinds, and 1 that draws the form of each number)**: one kind reads a numeral and writes it in digits and one writes a numeral for a number in digits, taught on
    a sample of numbers that covers every word, so the meaning of the words can be worked out; the other kinds of the family are computed over the numerals.
  - **Second family (6 kinds)**: no kind reads or writes it. Kinds computed over the numerals (a sum, a halving and others) fix the meaning only through the arithmetic. The lessons of the
    sum and the halving show every word of the family.
  - **Third family (4 kinds)**: nothing in the lessons can fix the meaning: the kinds over it rearrange or copy words, are arbitrary codes, or are a hard kind defined over
    numbers that no lesson ties to the words.
  - **Fourth family (no kind)**: its words occur only in trap rows, after a question about numbers.

  Every word of a held-out or beyond row of the first two families occurs in the lessons that tie that family (a test checks every word, the zero word included).
- **Mixed forms.** Three kinds take one number as digits and another as words in the same question (in either order) and one kind draws the form of each number (as digits
  or as words) in lessons and stream alike. Six kinds answer in another form than the question is in (digits to words, words to digits, words of the first family to words
  of the second, words of the second to digits). The traps include a kind taught in one form and asked in another, and a kind with one number in a form it was never taught with.
- **Rewordings, and traps that look like them.** There are eight ways of asking a question with one number, eight with two numbers and six with three, with
  words that occur nowhere in the earlier sets' ways (a test compares every text); some repeat a word of the question, and none puts two numbers side by side. Every kind is
  taught in two or three of them, drawn by the seed; three controls (one for each count of numbers) are taught in all of them, which shows that the ways mean the same. The held-out
  facts of the other 24 kinds with lessons are then asked in one of four ways, six kinds each (three controls and three hard kinds in each): **in a taught way**; **in a way that
  was not taught for the kind** (one only taught for the three controls); **in a mix of the start of one taught way and the end of another** (up to the kind's word the first,
  after it the second); **in a taught way with a filler word left out** (`kindly`, `today`, `pray`, `what`). All four are answerable by a system that treats the wording as phrasing.
  The beyond-range questions are asked in the first taught way. Next to them stand trap rows that look just like these rewordings and cannot be answered: a taught way with a start
  that no way of asking has, with an end that none has, with a number or a word of the ways where the kind's word should be, with the kind's word in the middle of a numeral, with the
  two words before the numbers exchanged (for kinds where the order of the numbers matters), with the start of a way for one count of numbers and the end of a way for another, and with
  a necessary word left out.
- **Repeats.** Numerals have words that occur more than once, the held-out rows of each kind over many-word numerals include numerals with a repeated word (four of them, except for the agreement grammar,
  where no word can occur twice, and the chunk grammar, where a word repeats only where the hundreds equal the last two digits), and every kind that takes two or three numbers has held-out rows that name the same
  number more than once (at least three for two numbers, two for three). A system that reads a question as a set of words cannot tell these apart from their shorter relatives.
- **Roles.** Five: `newop` (10 kinds), `compose` (2), `control` (15), `unexplainable` (5), and `trap`. The first four are kinds with lessons. `trap` rows have no
  lessons and no rule; they look like ordinary questions and the right answer is a refusal (118 per world): two kinds in one question (8), a word that was never
  taught (8), a word one letter away from a kind word (4), the wrong number of numbers for a kind (8), numbers in a form the kind was never taught with (12: words
  of the third family for a kind over digits or over another family, digits for a kind over the third), one number of several in such a form (4), a bare number (6), a run of words
  that is no numeral of the grammar, as a word repeated, an order or a word the grammar does not allow (for the agreement grammar also a digit word in its plain form before a place
  word), or a word one letter away from a number word (18, six of each), a question about numbers followed by a numeral of the fourth family (6), the units of a taught question
  in the opposite order (6), and 38 rows that look like the rewordings above (6 + 6 + 6 + 6 + 6 + 4 + 4). Their truths are drawn one by one and follow no rule. The `unexplainable` kinds include numerals taught nowhere (arbitrary codes over words of three families, one a pair of
  numerals) and one code that follows a remainder for most numbers and not for the others.
- The stream holds 705, 707 and 709 questions (282, 284 and 286 control, 193 newop, 30 compose, 82 unexplainable and 118 trap) on seeds 1 to 3, after 6396, 6227 and 6715 lessons.
- The words are two syllables plus the ending `cuwe` (the earlier sets end in `sido`, `velu`, `ketu`, `ruvo`, `zane`, `yarn`, `tole` and `dopa`), so no word of one set is a
  word of another (a test checks seeds 1 to 6 against all eight, and checks that no invented word of this set is any word at all of an earlier exam). The words of the
  numerals have four letters and begin with one of five letters that no invented word of an earlier set contains.
- The controls can be answered from the seen examples by a learner with the starting operations (or, for the spelling and word controls, once the words are
  read or copied): a refusal on them is a defect. The five `unexplainable` kinds have no rule and must stay refused until told, one of them a code that follows a
  remainder for most numbers and so looks like a rule.

## What the baby and a memorizer do on it (seeds 1 to 3)

`baby exam adapt-run DIR` (built from `main` at 31bfc93; percent of the questions in the role; fact right / WRONG / guess right / guess wrong / refused):

| role (n) | seed 1 | seed 2 | seed 3 |
|---|---|---|---|
| control (282, 284, 286) | 0.4 / 5.3 / 0.0 / 0.0 / 94.3 | 0.4 / 5.3 / 0.0 / 0.0 / 94.4 | 1.7 / 9.4 / 0.0 / 0.0 / 88.8 |
| newop (193) | 2.6 / 1.6 / 0.0 / 0.0 / 95.9 | 0.0 / 9.3 / 0.0 / 0.0 / 90.7 | 0.5 / 13.5 / 0.0 / 0.0 / 86.0 |
| compose (30) | 0.0 / 0.0 / 0.0 / 0.0 / 100.0 | 0.0 / 0.0 / 0.0 / 0.0 / 100.0 | 0.0 / 0.0 / 0.0 / 0.0 / 100.0 |
| unexplainable (82) | 0.0 / 8.5 / 0.0 / 0.0 / 91.5 | 0.0 / 7.3 / 0.0 / 0.0 / 92.7 | 0.0 / 23.2 / 0.0 / 0.0 / 76.8 |
| trap (118) | 0.0 / 6.8 / 0.0 / 0.0 / 93.2 | 0.0 / 7.6 / 0.0 / 0.0 / 92.4 | 0.0 / 6.8 / 0.0 / 0.0 / 93.2 |
| all (705, 707, 709) | 0.9 / 4.7 / 0.0 / 0.0 / 94.5 | 0.1 / 6.8 / 0.0 / 0.0 / 93.1 | 0.8 / 11.3 / 0.0 / 0.0 / 87.9 |

The baby answers 0.1 to 0.9 % of the questions as facts (0.4 %, 0.4 % and 1.7 % of the controls, 2.6 %, 0.0 % and 0.5 % of the newop kinds) and gives no guesses. **It gives wrong facts
on all three seeds**: 33 on seed 1, 48 on seed 2 and 80 on seed 3 (spread over every role, traps and codes included), and it answers 8, 9 and 8 trap questions; the run says "FAIL on honesty" on each.
The `baby` is frozen as the reference and was not changed. Its wrong facts are rows whose question or answer holds words of the families as well as rows over digits; the offenders the run lists
(the first eight in the stream) say it answers a copy or a count of words, or a number, that follows no lesson. The energy lines of the run are not recorded here (the machine was not quiet).

`baby exam adapt-drive DIR python3 tools/exam_adapt_baseline.py` (the naive memorizer, which only repeats told answers): 0.0 % right in every role on all
three seeds, 100 % refused, WRONG 0 ("PASS on honesty"). No question in the stream repeats a taught or an earlier question, so memorizing cannot score. The
lessons number 6396, 6227 and 6715 on seeds 1 to 3 (they differ because the kinds are taught in two or three ways of asking and in different grammars).

## Is any kind reachable with the starting operations? (brute force, 7 nodes)

The check used for the earlier sets, kept as a test (`test_no_hard_kind_is_reachable_and_the_controls_are`; it needs numpy and takes some minutes, so it
is skipped without numpy; `HW_I_SEARCH_SEEDS=N` limits it to seeds 1 to N, the default is 6): every expression of at most 7 nodes (leaves count) over `+ - * min max div mod`, the
variables, and the constants 0 to 40, 50, 64 and 100, deduplicated by value on the sample points, with subtraction both signed and saturating at 0, and division or
remainder by zero taken as 0. Unary kinds were matched on the points 1 to 31, 1 to 63, 1 to 99 or a fixed spread up to 394, by the size of the domain; binary kinds on 60
fixed pairs, half with both numbers up to 24; kinds with three numbers on 40 fixed triples. An exact match on the sample counts as reachable, so the check errs toward calling
a kind reachable. The search builds every function of 1, 3 and 5 nodes (deduplicated by value), then tries every way of joining two parts of 1 + 5, 5 + 1 or 3 + 3 nodes on the
first ten points and checks the full sample where those agree with a target. **For the kinds whose numbers are words the check is on the numbers the words stand for.** The
two kinds that copy a word or rearrange words are not functions of the numbers and are not searched.

It covered seeds 1 to 6 (every drawn constant, in both subtraction conventions; 30 searched targets per seed):

- **Reachable, as intended:** all thirteen searched controls, in every seed from 1 to 6 and in both conventions (at most 1 to 7 nodes).
- **Not reachable at 7 nodes, any seed 1 to 6:** all 10 `newop` kinds, both `compose` kinds and the five arbitrary codes (the codes over words were checked on the numbers).
  No kind had to be replaced for being reachable in the end. (Drafts were changed while the set was written for other reasons: a machine over binary digits with four states
  took only four values and was given five states, a count over the words of a numeral that was constant for one grammar was replaced by a copy of the middle word, and a pair of
  remainders that took too few values is now redrawn until it takes eight.)
- **The claim is narrow.** Because inputs stop at 15, 31, 40, 99, 127 or a few hundred, every kind is a finite table, and some have a longer program; "not reachable at
  7 nodes" is the claim, not "impossible". The constants of the hard closed forms (a prime, an offset, a divisor) are partly outside the constants the search offers; those kinds
  are not close to seven nodes either (the shortest program written out for each has nine or more nodes), so the claim does not rest on the constants. A learner may also build
  operations for itself from loops, tables or conditionals, which are not covered. The search finds what it should (the test checks a seven-node kind, a five-node program with
  three numbers, and nothing for a nine-node program or for the median of three). How each kind could be learned is in the `learn` column of `key.tsv`.

## Things to know

- Every answer is exact by construction (the tests recompute each lesson and stream answer from the rules of its kind, check the operations against known values,
  and check them again against second implementations written another way). The numerals are checked to read back to the numbers against a second reading of
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

---

# STOP: do not read below this line before the first run

Everything below names the kinds.

---

## The kinds (32)

`n` is the unit's family of numerals: **d** digits, **m** digits or words of the first family (drawn for each number of a fact), **1**, **2**, **3** the first, second and
third family of words, **w** a run of words (copied from a numeral or rearranged). Where two are given, the first is the question and the second the answer; a pair of letters
before a slash gives the form of each number.

| role | kind | n | rule |
|---|---|---|---|
| `newop` | quadfloor | m / d | (a * a + k * a + c) div d, k from 11 to 40, c from 100 to 899, d from 7 to 31; a from 0 to 99 |
| `newop` | absring | d | the sum of the three absolute differences of a, b and c (twice the largest minus the smallest); from 0 to 40 |
| `newop` | cubepair | dt1 / d | (a * a * a + k * a * b + b * b) mod p, k from 5 to 44, p a prime from 59 to 211; a, b from 0 to 60; a in digits, b in words of the first family |
| `newop` | trimin | 2 / d | the least of three linear forms of the pairs (a, b), (b, c), (c, a) with coefficients from 1 to 6; from 0 to 40 |
| `newop` | dfa2 | d | the output of a five-state machine after the binary digits of a (none for 0), from the largest; no function of a mod 2 to 8 or of the number of 1 digits; a from 0 to 127 |
| `newop` | cantorcount | 3 / d | how many of the numbers 0 to a have no digit 2 in base 3; a up to 242 (or the cap of the third family) |
| `newop` | tailpair | 2 / 2 | how many different pairs the walk (x, y) -> (y, \|x - y\|) visits from (a, b) before a pair comes round again; from 0 to 40 |
| `newop` | adjbranch | d | p * a + c if the binary form of a has two 1 digits side by side, else (a div d) + e; a from 0 to 199 |
| `newop` | modinv | d | the smallest x from 1 with a * x = 1 modulo a prime p from 101 to 211; a from 1 to 100 |
| `newop` | boardsquares | 1 / d | the squares of every size on a board of a by b cells; from 1 to 40, numbers in words of the first family |
| `compose` | farthin | d | farness(twinmod(a)); a from 0 to 99 |
| `compose` | dfafar | d / 1 | dfa2(farness(a)), the answer in words of the first family; a from 0 to 99 |
| `control` | cubeprod | d | a * b * c mod m, m from 13, 17, 19, 23, 29, 31, 37; from 0 to 15 |
| `control` | capsum | d | min(a + b, c); from 0 to 31; taught in all six ways of asking with three numbers |
| `control` | farness | d | \|a - k\|, k from 5 to 34; a from 0 to 99; taught in all eight ways of asking with one number |
| `control` | twinmod | d | (a mod p) * (a mod q), p and q different from 3 to 12, taking at least eight values; a from 0 to 99 |
| `control` | slanted | d | p * a + b div q, p and q from 2 to 9; from 0 to 31; taught in all eight ways of asking with two numbers |
| `control` | decode1 | 1 / d | a spelled in the first family, the answer a in digits (up to the cap of the grammar) |
| `control` | encode1 | d / 1 | a in digits, the answer a spelled in the first family |
| `control` | sumw1 | 1 | a + b, each up to half the cap |
| `control` | mixpair | dt1 / d | a + 2 * b, a in digits and b in words of the first family |
| `control` | mixwrite | t1d / 1 | max(a, 2 * b), a in words of the first family and b in digits, the answer in words of the first family |
| `control` | step12 | 1 / 2 | (3 * a) div 2 + c, c from 1 to 9, the number in words of the first family and the answer in words of the second |
| `control` | join2 | 2 | a + b, each up to half the cap |
| `control` | halve2 | 2 | a div 2, a up to the cap |
| `control` | rot3 | 3 / w | the words of the numeral with the first moved to the end (a run of words, not always a numeral) |
| `control` | middle3 | 3 / w | the middle word of the numeral (the later of the two in the middle for an even count), copied |
| `unexplainable` | dotcode, nearmod5, scatterw1, scatterw2, scatterpair3 | d, d, 1, 2 / d, 3 | an arbitrary code (0 to 16) for each number 0 to 31; a mod m (m from 5, 6, 8, 9) for about 60 % of the numbers 0 to 31 and an arbitrary code for the rest (at least 8 of them differ from the remainder); an arbitrary code (0 to 299) for each number 0 to 31 in the first family; an arbitrary code (0 to 49) for each number 0 to 31 in the second family, in digits; an arbitrary code for each pair of numbers 0 to 15 in the third family (0 to 199) |

How each kind could be learned is in the `learn` column of `key.tsv`; in short, the closed forms are read off rows and columns by differences or by factors, the others are loops with a state over
digits or over a pair, a count over a property of the digits, a conditional on a property of the digits, a number checked by one multiplication, a sum over sizes, and the controls are short programs
or, over words, copies and rearrangements. The hard kinds over the third family (`cantorcount`) are the ones where a refusal is expected unless an assignment of numbers to its words that fits every
lesson of the family is recovered. The `unexplainable` kinds over words are the "numeral taught nowhere" rows: a valid numeral for which no lesson gives an answer.

Which is easy for which learner was not measured, and no kind was tuned either way.

Conventions a system could trip over: `tailpair(0, 0)` is 1 (one pair); `cantorcount(0)` is 1 (the number 0 has no digit 2); `dfa2(0)` is the output of the start state; `modinv` and `boardsquares`
start at 1; `adjbranch(0)` takes the second branch; `decode1` and `encode1` run to the cap of the first family's grammar; `sumw1`, `join2`, `halve2` and `step12` stay within the cap of their grammars;
`rot3` and `middle3` give a run of words, which is not always a numeral; `quadfloor` takes each number either way, drawn for each number of each fact (about half and half, the same in the lessons and in
the stream); in the agreement grammar a digit word before the ten or hundred word is in its changed form and a lone zero word is the number zero; the bi-quinary grammar has a five word of its own for the
tens; in the doubling grammar every word after the first is a doubling, a doubling and an "and one", or a doubling again; in the chunk grammar the zero chunk is the word that follows the hundreds in 100, 200, ....

