<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 DawsMacs -->
# Hidden worlds H: an eighth set of exam worlds, with three-number kinds, six new kinds of numeral, and rewordings next to the traps that look like them

`tools/hidden_worlds_h.py` writes an eighth family of adaptive exams. The files, the line protocol, the shape of a world (invented words for the kinds,
held-out and beyond-range questions with the truth told after each answer) and the roles are those of `tools/hidden_worlds.py` and its six
successors (see `tools/hidden_worlds.md`). The kinds are all different from those of sets A to G, and four things are new in how the exam is put.

**Why an eighth set.** Sets A to G are development sets once their kinds have been read. This set was written without looking at the learner it will
be run on. **To keep it a blind test, do not read the list of kinds below the marked line (or `make_kinds` in the script, or the test file, which
checks kinds by name) before the first run**: run it, look at the numbers, then read on.

```bash
python3 tools/hidden_worlds_h.py DIR --seed 1             # writes lessons.tsv, stream.tsv (+ README.txt, key.tsv)
baby exam adapt-drive DIR COMMAND...                      # judge any system on it, as for the clean adaptive exam
baby exam adapt-run DIR                                   # the baby itself
python3 -m unittest discover -s tools -p 'test_hidden_worlds_h.py'
```

`--compose-first` teaches the compositions before the kinds they use. The generator is standard-library Python 3 with the first set's small
deterministic random generator, so a seed gives the same bytes on every Python version (the seed goes through one extra mix first). It imports the
first generator for the shared pieces; the kinds, the words, the numerals, the ways of asking and the salts are its own. `key.tsv` has five columns:
the word, the role, the kind, the rule and how the kind could be learned. It is never sent to a system.

## What differs from the earlier sets (without naming a kind)

- **The kinds.** None of the kinds of sets A to G is reused, by name or by table (a test compares every table of this set with every table of the seven
  earlier sets, controls and codes included, seeds 1 to 5 against seeds 1 to 5; the kinds that read and write words, add them, double them and
  multiply them by a constant are the identity, a sum, a double and a multiple on a larger domain than the spelling controls of earlier sets, so on the first
  32 numbers those agree with an earlier control by construction). **Closed forms are in**, in shapes none of the earlier sets used, with one, two and **three numbers**, next to controls of the same family that a short program reaches;
  their constants are drawn from the seed and several are large. The other hard kinds are structures that no earlier set used (listed below the marked line), and two
  compositions of kinds of this set.
- **Numbers as words, in six new kinds of numeral.** 19 of the 32 kinds put their numbers into the question or the answer as words, in numbers up to 399
  to 999 (inputs of sets A to E stopped at 31 or 99). Sets F and G used English-style sums of words, positional digit words (most or least significant
  first, bases 6 to 20), a mixed radix of three words, and words that multiply a base word with or without a word for "minus". This set uses grammars of other
  kinds, six of which exist and each seed uses three, one for each of three families of words (the grammar of a family is the same for every kind of the
  family; its words are invented, have four letters and are all different from those of the other families and of every earlier set):
  - **bijective numeration**: positional, one word for each digit 1 to b (b is 5, 6 or 8 by seed) and none for zero, most significant first; a word of its
    own for the number zero that stands alone.
  - **balanced digits**: positional, one word for each digit from minus h to h (b = 2h + 1 is 3, 5 or 7 by seed), a word for minus one that has nothing to do
    with the word for one, most significant first.
  - **tally words**: a word for each place value (1, b, b squared, ... with b 4, 5 or 6 by seed) repeated as often as the digit says, the largest place first;
    numerals of up to 19 words in which one word occurs many times.
  - **twenties with a leftover** (the French style, `four twenties and ten`): digit words 1 to 9, a word for ten, a word for twenty and a link word; the
    count of twenties (a lone twenty word for one) is written as a ten and a digit, then the link, then the leftover below twenty; the leftover comes first in
    one variant. Numbers up to 399.
  - **order-signed words**: words of value 1, 5, 10, 50, 100 and 500 are added, largest first, and a smaller word in front of a larger one is taken off
    instead (for 4, 9, 40, 90 and 400 in one variant, for 4 and 9 only in the other, where 40 is four tens). Numbers up to 899.
  - **place-bound digit words**: positional with three places, and a different set of ten digit words for each place (units, tens, hundreds), none for a zero
    in front; the smallest place first in one variant. A word tells its place by itself, so no word occurs twice in a numeral.

  Seed 1 uses bijective, balanced and tally words for its first, second and third family; seed 2 uses twenties with a leftover, place-bound digit words and
  order-signed words; seed 3 uses balanced digits, tally words and twenties with a leftover. A learner is never told that the words are numbers. The families
  differ in what the lessons say about them:
  - **First family (7 kinds, and 2 that mix it with digits)**: one kind reads a numeral and writes it in digits and one writes a numeral for a number in digits,
    taught on a sample of numbers that covers every word, so the meaning of the words can be worked out; the other kinds of the family are computed over the
    numerals. **Two kinds are taught with digits and words of this family mixed inside their lessons** (each number of a fact is drawn as digits or as words),
    and asked the same way.
  - **Second family (5 kinds)**: no kind reads or writes it. Kinds computed over the numerals (a sum, a double and others) fix the meaning only through the
    arithmetic: the sums of small numbers name the words one by one and the sums that carry show the places. The lessons of the sum and the double show every
    word of the family.
  - **Third family (5 kinds)**: nothing in the lessons can fix the meaning: the kinds over it rearrange or count the words, copy one, are arbitrary codes, or are a hard kind
    defined over numbers that no lesson ties to the words.
  
  Every word of a held-out or beyond row of the first two families occurs in the lessons that tie that family (a test checks the zero word and every other
  word). Beyond-range questions use numbers above the largest taught one, so words occur there in combinations no lesson showed.
- **Rewordings, and traps that look like them.** There are eight ways of asking a question with one number, eight with two numbers and six with three, with
  words that occur nowhere in the earlier sets' ways. Every kind is taught in two or three of them, drawn by the seed; three controls (one for each count of numbers) are
  taught in all of them, which shows that the ways mean the same. The held-out facts of the other 24 kinds with lessons are then asked in one of four ways, six
  kinds each (three controls and three hard kinds in each): **in a taught way**; **in a way that was not taught for the kind** (one only taught for the three
  controls); **in a mix of the start of one taught way and the end of another** (up to the kind's word the first, after it the second); **in a taught way with a
  filler word left out** (`please`, `now`, `me`). All four are answerable by a system that treats the wording as phrasing. The beyond-range questions are asked in
  the first taught way, as before. Next to them stand trap rows that look just like these rewordings and cannot be answered: a taught way with a start that no
  way of asking has, with an end that none has, with a number or a word of the ways where the kind's word should be, with the kind's word in the middle of a
  numeral, with the two words before the numbers exchanged (for kinds where the order of the numbers matters), with the start of a way for one count of numbers
  and the end of a way for another, and with a necessary word left out.
- **Repeats.** Numerals have words that occur more than once (tally words, bijective and balanced numerals), the held-out rows of each kind over many-word
  numerals include four numerals with a repeated word (except for the place-bound grammar, where no word can occur twice), and every kind that takes two or
  three numbers has held-out rows that name the same number more than once (at least three for two numbers, two for three). A system that reads a question as a
  set of words cannot tell these apart from their shorter relatives.
- **Roles.** Five: `newop` (10 kinds), `compose` (2), `control` (15), `unexplainable` (5), and `trap`. The first four are kinds with lessons. `trap` rows have no
  lessons and no rule; they look like ordinary questions and the right answer is a refusal (108 per world): two kinds in one question (8), a word that was never
  taught (8), a word one letter away from a kind word (4), the wrong number of numbers for a kind (8), numbers in a form the kind was never taught with (12: words
  of the family that nothing can explain for a kind over digits or over another family, digits for a kind over that family), a bare number (6), a run of words
  that is no numeral of the grammar, as a word repeated, an order or a word the grammar does not allow, or a word one letter away from a number word (18, six of
  each), the units of a taught question in the opposite order (6), and 38 rows that look like the rewordings above (a start that no way has, an end that none has, a number or a word of the ways in place of the kind's word, the kind's word inside a numeral, the words before two numbers exchanged, mismatched ends, a necessary word left out: 6 + 6 + 6 + 6 + 6 + 4 + 4). Their truths are drawn one by one and
  follow no rule. The `unexplainable` kinds include numerals taught nowhere (arbitrary codes over words of three families, one a pair of numerals) and one code that
  follows a remainder for most numbers and not for the others. The stream holds 675, 672 and 675 questions (280, 277 and 280 control, 175 newop, 30 compose, 82 unexplainable and 108 trap) on seeds 1 to 3 questions per world.
- The words are two syllables plus the ending `dopa` (the earlier sets end in `sido`, `velu`, `ketu`, `ruvo`, `zane`, `yarn` and `tole`), so no word of one set is a
  word of another (a test checks seeds 1 to 6 against all seven, and checks that no invented word of this set is any word at all of an earlier exam). The words of the
  numerals have four letters and no common ending.
- The controls can be answered from the seen examples by a learner with the starting operations (or, for the spelling and word controls, once the words are
  read or counted): a refusal on them is a defect. The five `unexplainable` kinds have no rule and must stay refused until told, one of them a code that follows a
  remainder for most numbers and so looks like a rule.

## What the baby and a memorizer do on it (seeds 1 to 3)

`baby exam adapt-run DIR` (built from `main` at ddf1e51; percent of the questions in the role; fact right / WRONG / guess right / guess wrong / refused):

| role (n) | seed 1 | seed 2 | seed 3 |
|---|---|---|---|
| control (280, 277, 280) | 3.2 / 5.7 / 0.0 / 0.0 / 91.1 | 3.2 / 11.2 / 0.0 / 0.0 / 85.6 | 2.5 / 9.3 / 0.0 / 0.0 / 88.2 |
| newop (175) | 0.6 / 9.7 / 0.0 / 0.0 / 89.7 | 0.0 / 8.0 / 0.0 / 0.0 / 92.0 | 1.7 / 11.4 / 0.0 / 0.0 / 86.9 |
| compose (30) | 3.3 / 26.7 / 0.0 / 0.0 / 70.0 | 3.3 / 16.7 / 0.0 / 0.0 / 80.0 | 0.0 / 26.7 / 0.0 / 0.0 / 73.3 |
| unexplainable (82) | 0.0 / 9.8 / 0.0 / 0.0 / 90.2 | 0.0 / 6.1 / 0.0 / 0.0 / 93.9 | 0.0 / 12.2 / 0.0 / 0.0 / 87.8 |
| trap (108) | 0.0 / 6.5 / 0.0 / 0.0 / 93.5 | 0.0 / 7.4 / 0.0 / 0.0 / 92.6 | 0.0 / 7.4 / 0.0 / 0.0 / 92.6 |
| all (675, 672, 675) | 1.6 / 8.3 / 0.0 / 0.0 / 90.1 | 1.5 / 9.4 / 0.0 / 0.0 / 89.1 | 1.5 / 10.7 / 0.0 / 0.0 / 87.9 |

The baby answers 2 to 3 % of the controls as facts, 0 to 3 % of the compositions and next to nothing of the hard kinds (0 to 2 %). **It gives wrong facts on all three
seeds**: 56 on seed 1, 63 on seed 2 and 72 on seed 3 (spread over every role, traps and codes included); the run says "FAIL on honesty" on each. The `baby` is
frozen as the reference and was not changed. The WRONG column is far above the 0.0 % that sets A to E had; the eight offenders that the run lists for each seed (the first eight
in the stream) are rows over digits as well as rows whose question or answer holds words.

`baby exam adapt-drive DIR python3 tools/exam_adapt_baseline.py` (the naive memorizer, which only repeats told answers): 0.0 % right in every role on all
three seeds, 100 % refused, WRONG 0 ("PASS on honesty"). No question in the stream repeats a taught or an earlier question, so memorizing cannot score. The
lessons number 5621, 5797 and 6092 on seeds 1 to 3 (they differ because the kinds are taught in two or three ways of asking and in different grammars).

## Is any kind reachable with the starting operations? (brute force, 7 nodes)

The check used for the earlier sets, kept as a test (`test_no_hard_kind_is_reachable_and_the_controls_are`; it needs numpy and takes some minutes, so it
is skipped without numpy; `HW_H_SEARCH_SEEDS=N` limits it to seeds 1 to N, the default is 6): every expression of at most 7 nodes (leaves count) over `+ - * min max div mod`, the
variables, and the constants 0 to 40, 50, 64 and 100, deduplicated by value on the sample points, with subtraction both signed and saturating at 0, and division or
remainder by zero taken as 0. Unary kinds were matched on the points 1 to 31, 1 to 63, 1 to 99 or a fixed spread up to 394, by the size of the domain; binary kinds on 60
fixed pairs, half with both numbers up to 24; kinds with three numbers on 40 fixed triples. An exact match on the sample counts as reachable, so the check errs toward calling
a kind reachable. The search builds every function of 1, 3 and 5 nodes (deduplicated by value), then tries every way of joining two parts of 1 + 5, 5 + 1 or 3 + 3 nodes on the
first ten points and checks the full sample where those agree with a target. **For the kinds whose numbers are words the check is on the numbers the words stand for.** The
three kinds that copy a word, reverse words or count words are not functions of the numbers and are not searched.

It covered seeds 1 to 6 (every drawn constant, in both subtraction conventions; 29 searched targets per seed, 11 minutes on one core):

- **Reachable, as intended:** all twelve searched controls, in every seed from 1 to 6 and in both conventions (at most 1 to 7 nodes).
- **Not reachable at 7 nodes, any seed 1 to 6:** all 10 `newop` kinds, both `compose` kinds and the five arbitrary codes (the codes over words were checked on the numbers).
  No kind had to be replaced for being reachable in the end. (Drafts were changed while the set was written for other reasons: a first conditional on the digits had the same
  branches as an earlier set's conditional on the bits and was given branches of another shape, and several names and a remainder-of-a-quotient control that took four values
  were changed after the tests that compare names, tables and counts of values failed.)
- **The claim is narrow.** Because inputs stop at 15, 31, 99 or a few hundred, every kind is a finite table, and some have a longer program; "not reachable at
  7 nodes" is the claim, not "impossible". A learner may also build operations for itself from loops, tables or conditionals, which are not covered. The search finds
  what it should (the test checks a seven-node kind, a five-node program with three numbers, and nothing for a nine-node program or for the median of three).
  How each kind could be learned is in the `learn` column of `key.tsv`.

## Things to know

- Every answer is exact by construction (the tests recompute each lesson and stream answer from the rules of its kind, check the operations against known values,
  and check them again against second implementations written another way). The numerals are checked to read back to the numbers against a second reading of
  each grammar, every number up to the cap of its grammar has a numeral of its own, and every run of words made for a trap row is checked to be no numeral of any family.
- The stream asks each fact once; it has no wrong-teacher columns (this is the clean exam).
- The tests of the files: same seed gives the same bytes, every line parses, neither a kind's name, its rule nor its learning note appears in any file that is sent to
  a system, and the only words in the files are the fixed words of the questions, the kinds' words, the numerals, the nine words that only `trap` rows carry, the four
  words that are one letter away from a kind word and the six that are one letter away from a number word.
- The answers of `trap` rows are told after the answer like any others. A system that learns from them learns the told word or number for that one question; none is
  asked twice.
- The "other form" traps are chosen so that no lesson can bridge the forms: a kind over digits, the first or the second family is asked with words of the third
  family, and a kind over the third family is asked with digits.
- A test requires at least five distinct values per kind (three for the kinds that copy, reverse or count words and for the pair code).

---

# STOP: do not read below this line before the first run

Everything below names the kinds.

---

## The kinds (32)

`n` is the unit's family of numerals: **d** digits, **m** digits or words of the first family (drawn for each number of a fact), **1**, **2**, **3** the first, second and
third family of words, **w** a run of words (copied from a numeral or reversed). Where two are given, the first is the question and the second the answer.

| role | kind | n | rule |
|---|---|---|---|
| `newop` | fourthmod | m / d | a * a * a * a mod p, p a prime from 59 to 211; a from 0 to 63 |
| `newop` | trimod | d | (a * b + b * c + c * a) mod p, p a prime from 59 to 211; three numbers from 0 to 15 |
| `newop` | prodquotmod | 2 | ((a * b + k) div d) mod m; k from 100 to 399, d from 7 to 31, m from 17 to 53; a and b from 0 to 60 |
| `newop` | gcd3 | d | the greatest common divisor of a, b and c, from 1 to 24 |
| `newop` | median | d | the middle one of a, b and c in size; from 0 to 31 |
| `newop` | bitdouble | d | the binary digits of a, each written twice, read as a number (0 stays 0); a from 0 to 31 |
| `newop` | vaneck | 1 | the a-th term (a from 1 to 99) of the sequence that starts with 0 and goes on with 0 if the last term is new and with how long ago it last occurred if not: 0, 0, 1, 0, 2, 0, 2, 2, 1, 6, ... |
| `newop` | ant | 2 | the number of black cells after a steps (0 to 99) of an ant on an endless white grid (on a white cell it turns right, flips the cell and steps forward; on a black cell it turns left, flips it and steps forward; it starts facing up) |
| `newop` | kaprekar | 3 / d | how many times the step "digits in descending order minus digits in ascending order" (a with zeros in front to three digits) is applied before the value stays: 0 for 0 and 495, 1 for 111 and the other repdigits, at most 6; a up to the cap of the third family (399, 899 or 999) |
| `newop` | ascbranch | d | p * (the sum of the digits of a) + c if the decimal digits of a strictly rise from left to right (a single digit counts), else (a mod q) + r; p 2 to 5, c 1 to 9, q 6 to 13, r 0 to 4; a from 0 to 99 |
| `compose` | cubequot | d | cubemod(modquot(a)); a from 0 to 99 |
| `compose` | vanbits | 1 / d | bitdouble(vaneck(a)); a from 1 to 99 |
| `control` | muldiv3 | d | a * b div c; three numbers from 1 to 15 |
| `control` | avg3 | d | (a + b + c) div 3; from 0 to 15; taught in all six ways of asking with three numbers |
| `control` | modquot | d | (a div p) mod q, p 3 to 9, q 5 to 9; a from 0 to 99; taught in all eight ways of asking with one number |
| `control` | cubemod | d | a * a * a mod m, m from 13, 17, 19, 23, 29, 31, 37; a from 0 to 31 |
| `control` | lopsided | d | min(p * a, q * b), p and q different from 2 to 9; from 0 to 31; taught in all eight ways of asking with two numbers |
| `control` | unspell1 | 1 / d | a spelled in the first family, the answer a in digits (up to the cap of the grammar, 399 to 999) |
| `control` | spell1 | d / 1 | a in digits, the answer a spelled in the first family |
| `control` | plus1 | 1 | a + b, each up to half the cap |
| `control` | times1 | 1 | k * a, k from 6 to 12, a up to cap div k |
| `control` | mixsum | m / d | a + b, each a number from 0 to 99 written as digits or as words of the first family, the answer in digits |
| `control` | plus2 | 2 | a + b, each up to half the cap |
| `control` | twice2 | 2 | 2 * a, a up to half the cap |
| `control` | flip3 | 3 / w | the words of the numeral in the opposite order (a run of words, not always a numeral) |
| `control` | distinct3 | 3 / d | the number of different words of the numeral, in digits |
| `control` | firstword3 | 3 / w | the first word of the numeral, copied |
| `unexplainable` | scatterdig, nearmod, scatter1, scatter3, scatterpair2 | d, d, 1, 3, 2 | an arbitrary code (0 to 16) for each number 0 to 31; a mod 7 for about 60 % of the numbers 0 to 31 and an arbitrary code from 0 to 9 for the rest (at least 8 of them differ from the remainder); an arbitrary code (0 to 299) for each number 0 to 31 in the first family; an arbitrary code (0 to 199) in the third family; an arbitrary code (0 to 299) for each pair of numbers 0 to 15 in the second family |

How each kind could be learned is in the `learn` column of `key.tsv`; in short, the closed forms are read off rows and columns by differences or by factors, the
others are recurrences, simulations, searches over digits and tables filled in from the left, and the controls are short programs or, over words, copies and counts. The
hard kinds over the third family (`kaprekar`) are the ones where a refusal is expected unless an assignment of numbers to its words that fits every lesson of the family is
recovered. The `unexplainable` kinds over words are the "numeral taught nowhere" rows: a valid numeral for which no lesson gives an answer.

Which is easy for which learner was not measured, and no kind was tuned either way.

Conventions a system could trip over: `vaneck` starts at 1 (its first term is 0); `kaprekar(0)` and `kaprekar(495)` are 0 and a repdigit is 1; `gcd3` and `muldiv3` start at 1;
`ant(0)` is 0; a single digit counts as rising in `ascbranch`; `bitdouble(0)` is 0; `unspell1` and `spell1` run to the cap of the first family's grammar (399, 899 or 999);
`plus1`, `plus2` and `twice2` stay within the cap of their grammar; `flip3` gives a run of words in the opposite order, which is not a numeral; `distinct3` counts the different
words of the numeral; `mixsum` and `fourthmod` take each number either way, drawn for each number of each fact (about half and half, the same in the lessons and in the stream);
in the order-signed grammar a lone zero word is the number zero and a smaller word before a larger one is taken off; the twenties grammar counts the twenties with a digit word for 2 to 9, the ten word for 10 and the ten word and a digit word for 11 to 19, writes a
lone twenty word for one twenty, and puts the link word only between the two parts.

