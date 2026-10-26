# Novice

The main line of work in Matter, **rebuilt from scratch**. [`baby`](../baby/README.md) is frozen as the reference and the measuring stick; Novice shares no code with it. What carries over is the [entry exam](../baby/README.md) (a plain-file and line-protocol judge that any system, in any language, can take), the goldens-and-platform habits, and the rules below.

> **Status (2026-10-25): started; rungs 0 to 4 done on the tidy adaptive exam.** It finds the kinds of question itself, explains most of them with programs priced in bits, builds later explanations from earlier ones, states no wrong fact, knows how reliable each source has been, and with one source states a teacher's possibly wrong answers as guesses, not facts (with a second independent source, as facts when the two agree). See "Results so far" for the numbers and, just as important, for what they do not show. On the same exam it uses about 55 times less energy than baby (3.7 to 4.2 J against 221 to 226 J; see "Joules"), after its cost of learning was cut 25-fold from the first version.

Labels used below: **Hypothesis** (believed, not shown), **Measured** (a number from a run), **Open** (not started).

## What Novice is for

Two things, and a bet that they help each other.

1. **Adaptive Learning that is genuine.** baby adapts, but inside a design that was written for it: a menu of five mechanisms, an expression language of seven pieces, and thresholds chosen by hand. The test for "genuine" is a capability that no line of code names in advance, shown on a task the system was not built for. Novice is built so that its own language can change when the old one fails.
2. **Efficiency from the start.** Every milestone has a joule budget and a measurement. baby answers a question for about 59 µJ but pays about 36 mJ to absorb one told answer (about 600 times more), and nothing it learns makes later answers cheaper. **Hypothesis (the bet):** adaptation can make a system cheaper to run, not dearer, because learning is compression and a shorter explanation costs fewer operations to evaluate. If that holds, the cost of answering falls as the system gains experience, and the cost of learning pays itself back.

Where the two truly conflict, adaptation wins and the cost is written down. "Frontier" is an ambition, not a claim: nothing here is near it.

## Design (all Hypothesis until a rung measures it)

- **One currency: bits.** An explanation is a program, or a table of facts. To compare two explanations, Novice adds the bits needed to write each one down to the bits needed to write down what it gets wrong (the minimum-description-length idea). A contradicting answer is not "held" or "vetoed" by a special rule; it is data that costs the incumbent explanation some bits and may cost a rival fewer. Where baby needs a hand-set count (three coherent answers before it believes a rule has changed), the count should fall out of the costs. That is the first thing to test (rung 3).
- **Integers in every decision.** Costs are counted in 1/256 bit with integer arithmetic only ([`bits.rs`](src/bits.rs)). Nothing that decides an answer uses the math library, so results are identical on every platform by construction instead of by testing it thirteen times. Floating point appears only in reports.
- **No labels from the teacher.** The exam tells a system which unit and which fact every lesson belongs to. Novice's protocol layer drops those columns before the system sees them ([`protocol.rs`](src/protocol.rs), tested), so structure has to be found, not read.
- **Sources are first-class.** When the exam says who is speaking (an optional extra column), Novice keeps a record per source and believes by agreement of independent sources or by an explanation it can check. With one source it claims nothing more than one source can give.
- **The language is a learned thing.** The starting primitives are few. Pieces that keep appearing in good explanations are promoted into the language, which makes later explanations shorter (and cheaper to run). Which primitives it starts with is a choice of mine, the largest remaining hand-built part, and is written down as such.
- **Cost proportional to the change.** A told answer should touch only what it concerns. The measure is joules per told answer staying flat as the world grows.
- **Zero dependencies, en-US, Rust.** Builds offline.

Prior work this leans on, so nobody mistakes it for new: the minimum-description-length principle (Rissanen) for the currency, and library-learning program synthesis (the DreamCoder line of work) for a language that grows by compressing its own solutions. What is meant to be different here is the setting: online, integer-only, a joule meter on every step, source tracking, and an exam that does not hand over the structure.

## How it is judged

`baby exam adapt-write DIR` writes a stream of lessons and questions; `baby exam adapt-drive DIR COMMAND...` runs any system as a child process and judges it. For Novice:

```bash
cd novice && cargo build --release --offline
cd .. && cargo build --release --offline
./target/release/baby exam adapt-write /tmp/exam --seed 1
EXAM_NO_JOULES=1 ./target/release/baby exam adapt-drive /tmp/exam "$PWD/novice/target/release/novice" serve
```

The judge reports wrong facts (must be 0), facts and guesses right per kind of question, and how each kind changes from the first third of the stream to the last. Rung 0 refuses all 787 questions, so the judge's last two lines read:

```
WRONG facts over the whole stream (must be 0): 0
verdict: PASS on honesty; compare coverage and the climb across the thirds
```

## The ladder

Each rung has the test that would show it works, and a joule line to fill in.

| rung | what | test | status |
|---|---|---|---|
| 0 | speaks the protocol, refuses everything | 0 wrong facts; the exam can drive a system that is not baby | done |
| 1 | remembers what it was told (a table of facts) | 0 wrong facts; right on the facts it saw, refuses the rest | done (inside the learner; the four lookup kinds in the exam are tables) |
| 2 | finds programs from a minimal start | right on questions it never saw, for kinds a short program can say; routing found without the labels | done on the tidy exam (seeds 1 to 3); joules open |
| 3 | rival explanations scored in bits | on the 14 streams of baby's wrong-teacher work: as few believed mistakes as baby's, with **no tuned count** | done on the tidy exam: repeats on par with baby's, 0 elsewhere, at a cost in coverage (see above); the count of three falls out of the costs |
| 4 | sources | with a second, right source added, repeats fall in kinds that have no law to check against | done on the tidy exam: 0 wrong facts of any kind on all streams with one reliable and one unreliable source; with one source, repeats fall to 0 by stating the unreliable source's answers as guesses |
| 5 | the language grows | a kind that needs an operation Novice does not start with, from worlds written apart from the learner, solved by a primitive it made | development set: 5 of 18 hard kinds per seed acquired, all correct, 0 wrong facts; the blind test set is open |
| 6 | answers get cheaper | joules per answer on a fixed set of questions fall as it learns, wrong facts stay 0, and the break-even point (learning joules spent against answer joules saved) is reported | first steps: learning cost 25-fold lower than the first learner, composition 116 times cheaper than enumeration, joules measured once; the curriculum measurement shows capability from experience but not yet a falling cost |

Rungs 0 to 4 are the first milestone (the tidy exam, with a source tag). Rung 5 is the test of "genuine"; the worlds for it are written by a process that the learner's author does not read until the review, so the learner cannot be tuned to them. Rung 6 is the bet itself.

## Results so far (2026-10-25)

Measured on the adaptive exam (`baby exam adapt-write DIR --seed S`: 45 kinds, 3,048 lessons, a stream of 787 questions with the truth told after each answer), the same files for both systems, one desktop core. "Facts right" and "wrong guesses" are shares of all 787 questions; the judge's thirds are the first, middle and last third of the stream.

| seed | system | wrong facts | facts right | wrong guesses | refused | right, 1st third → last third | wall time |
|---|---|---|---|---|---|---|---|
| 1 | Novice | **0** | 67.0 % | 1.1 % | 11.2 % | 86.3 → 88.2 % | 0.35 s |
| 1 | baby | 0 | 64.2 % | 1.5 % | 11.4 % | 85.2 → 87.8 % | 15.6 s |
| 2 | Novice | **0** | 67.6 % | 1.3 % | 11.2 % | 84.4 → 91.2 % | 0.35 s |
| 2 | baby | 0 | 64.8 % | 1.5 % | 11.2 % | 84.4 → 90.8 % | 17.5 s |
| 3 | Novice | **0** | 67.6 % | 1.1 % | 11.2 % | 86.7 → 90.1 % | 0.35 s |
| 3 | baby | 0 | 64.4 % | 1.5 % | 11.2 % | 86.7 → 89.7 % | 16.9 s |

The kinds that change their rule outside what was taught (a sum that saturates at 30, one that wraps at 40) climb from 72.0-84.4 % right in the first third to 96.2-100 % in the last in Novice (seeds 1 to 3), and from 68.0-78.1 % to 92.3-97.4 % in baby. The 11.2 % refused are the same in both: the six kinds that have no rule it can say (a prime test, an exclusive-or, four arbitrary codes), which cannot be answered before they are told.

**What it found, with no labels and no menu of mechanisms** (seed 1): `x + y`, `x - y`, `x + 1`, `div(x, 2)`, `max(x, y) - min(x, y)`, `x * 3`, `div(x, 10) + mod(x, 10)`, `mod(x + y, 40)` (the 40 is the number a refuted `x + y` got wrong: it said 45 where the truth was 5), `min(x + y, 3 * 10)`, and, built on words it had earned, `sum_of_squares(x, y) + 1`, `div(sum_of_squares(x, y), 2)` and `average(x, y) * average(x, y)`. Two kinds became words (sum of squares, average); the other 37 explained kinds did not.

### Joules (first measurements, 2026-10-25)

The CPU package's own counter (RAPL), joules above the machine's idle draw, on the seed-1 adaptive exam, one desktop core. Novice is measured as a child process through the exam's protocol, baby in-process; "stream" is answering the 787 questions *and* absorbing the truth told after each, because for both systems that is one loop.

| | teaching (3,048 lessons) | stream (787 questions, truths told) | total | right answers per joule |
|---|---|---|---|---|
| Novice (4 runs) | 3.3 to 3.8 J (0.33 s) | 0.36 to 0.42 J (0.03 to 0.06 s), about 0.5 mJ per question | **3.7 to 4.2 J** | 164 to 184 |
| baby (3 runs) | 121 to 123 J (10.3 s) | 100 to 104 J (8.7 to 8.8 s), about 130 mJ per question | **221 to 226 J** | 3.0 to 3.1 |

So about **55 times less energy for the same exam**, with 689 right answers against 685. Caveats: these are upper bounds (the counter includes whatever else the machine was doing: it warned of other processes during every run, and Novice's runs last a third of a second, so a background process weighs heavily in them); it is one machine, one seed, and the clean exam; baby's own notes recorded 65 J on 2026-10-23, before its skeptical exam driver, and today's build measures 221 to 226 J in each of three runs; and the comparison is of two systems that learn differently, not of one idea against another. What it does show is that Novice's design is not just faster in wall time: it is cheaper in energy, by a margin that survives the noise. Nine tenths of Novice's energy is teaching (the first search for programs); absorbing a told answer costs it a fraction of a millijoule, a few thousandths of baby's. That is still not the thing the efficiency bet claims (that *adapting* makes a system cheaper): nothing here yet shows its energy per answer falling as it learns.

### Sources (rung 4)

`baby exam adapt-write DIR --second P` (the exam extension in its own pull request) adds a second source that is wrong about a fraction P of the facts, independently of the teacher's mistakes; `adapt-drive` then sends each truth twice, tagged `teacher` and `second`. Streams without it are unchanged.

**How it works.** What the lessons said and what each told source says are separate *channels*. A channel is as reliable as it has been wherever a program was confident (the only check a system can make on its own), starting from the lessons' record when it is the same teacher. In a kind with no rule, each channel that spoke contributes its reliability if it is right and `(1 - reliability) / (answers possible - 1)` if it is wrong, and the answer is believed as much as the channels jointly allow. A fact needs all but 2^-10 of the belief, as everywhere. So a single channel measured at 80 % can only be a guess; two independent channels that agree make a fact; two that disagree favor the more reliable one with a stated doubt. For a program, a second source's answer is independent evidence; the lessons followed by the same teacher's told answer are one speaker and are weighed once.

**Measured** (seeds 1 to 3, 787 questions, 0 wrong facts means none of any kind):

| streams | wrong facts | facts right | wrong guesses | refused |
|---|---|---|---|---|
| honest teacher (clean exam) | 0 | 67.0 / 67.7 / 67.9 % | 1.1 / 1.3 / 1.1 % | 11.3 / 12.2 / 11.7 % |
| teacher wrong about 10 %, 20 %, 40 %, **one source** | repeats 0 / 0 / 0 in all nine streams (before this rung: 1/0/3, 2/0/7, 9/9/7); one "elsewhere" at 40 % seed 2 | 61.8 to 63.7 / 58.1 to 61.0 / 46.1 to 52.1 % | 1.3 to 1.8 / 0.8 to 2.4 / 1.5 to 2.3 % | 11.7 to 12.6 / 14.6 to 20.3 / 28.2 to 36.5 % |
| teacher wrong about 20 %, second source wrong about 5 % | **0** on all three | 66.8 / 65.2 / 64.5 % | 0.9 / 0.9 / 0.6 % | 17.3 / 14.4 / 14.4 % |
| teacher wrong about 40 %, second source wrong about 5 % | **0** on all three | 66.5 / 64.2 / 63.4 % | 0.1 / 0.6 / 0.3 % | 29.7 / 23.0 / 20.8 % |
| honest teacher, second source wrong about 5 % | 0 | 67.2 / 67.3 / 67.6 % | 0.6 / 0.9 / 0.8 % | 11.2 / 11.3 / 11.2 % |
| teacher and second source both wrong about 20 % | 0 | 61.5 / 61.8 / 61.1 % | 1.3 / 0.9 / 0.8 % | 20.5 / 15.5 / 14.6 % |

The reliabilities it measures are close to the truth (seed 3 shown): with a teacher wrong about 20 % and a second source wrong about 5 %, it reports the second source at 478 of 497 checks (96 %; true 95 %) and the teacher at 283 of 358 (79 %; true 80 %); at 40 % the teacher at 215 of 349 (62 %; true 60 %). With an honest teacher (seed 1) it reports 367 of 367 for the teacher and 502 of 530 (95 %) for the second source.

**What this means, and what it does not.** With one source the remaining repeats disappeared because Novice no longer states what a source it has measured as unreliable says as a *fact*: it says it as a guess with that source's measured rate, so the mistakes are wrong guesses (1.5 to 2.4 %), which are allowed, and not wrong facts. That is the honest version of "a single source cannot catch these mistakes": it cannot catch them, but it can know it cannot. A second source turns many of those guesses back into facts when it agrees. The costs: under a wrong teacher with one source the facts it can state in the kinds with no rule fall, which is the correct price of not believing an unreliable source. The limits: the sources must err independently (two sources that make the same mistake are believed); the checks on reliability exist only where a program is confident, so a system with no kinds that have rules would have nothing to measure with; the answer space (100 possible answers) is a guess written into the code; and this is one tidy exam.

### Acquiring operations it does not start with (rung 5, first results on a development set)

`tools/hidden_worlds.py` (from a separate session, in its own pull request) writes exams in which every interesting kind needs an operation outside `+ - * min max div mod`: counting one bits, greatest common divisor, bitwise and/or/xor, integer square root, number of divisors, the smallest power of two above a number, trailing zero bits, bit mirror, last digit of a power, a small table; plus kinds composed on those (popcount plus a constant, parity of the bit count, gcd halved, lcm, hamming distance, sqrt plus popcount); plus controls the starting operations can say; plus arbitrary codes. Before any of this was built I ran the existing Novice on it once, unread: it refused every new-operation and composed question (0 wrong facts), and baby gave exactly the same numbers.

**What was added.** An explanation may now refer to the kind's *own values*: `f(p) = e(p, f(g(p)))`, with the base cases being the table of what was taught and told (shared and live, so what is learned later is used at once). The reduction `g` and the step `e` come from the same bit-priced search as everything else: reductions from everything three to five nodes can make, steps from a search over the arguments and the value at the nearer point. Nothing in it names counting, divisors or remainders. It is tried only when no closed form (plain search, constants, composition from learned words) explains the kind, and what it finds becomes a word like any other, so kinds built on it can be composed.

**Results on the development worlds (seeds 1 to 3):** 0 wrong facts and 0 wrong guesses on all three; the original exam is unchanged (same numbers, 0 wrong facts). Five of the 18 new-operation and composed kinds are acquired on each seed, and every one is correct (checked against the generator's `key.tsv`, which the system never sees):

| seed | acquired by a recurrence | acquired by composing on those |
|---|---|---|
| 1 | smallest power of two above x (`r + r, r = f(x - x div 2)`), count of one bits (`r + x mod 2, r = f(x div 2)`), one bits plus a constant, gcd halved (`r = f(y, x mod y)`) | parity of the bit count, as `mod(popplus(x), 2)` |
| 2 | count of one bits, one bits plus a constant, gcd and gcd halved (`r = f(mod(y, x), x)` and `r = f(y, mod(x, y))`) | parity of the bit count, as `mod(popcount(x), 2)` |
| 3 | gcd, count of one bits, smallest power of two above x, one bits plus a constant | parity of the bit count, as `mod(popcount(x), 2)` |

On seed 1 it also recovered the count of one bits from *popcount plus 2* (`popplus(x) - 2`) after finding the latter first. What it gets right is stated as a labeled guess with a measured track record, not as a fact: about 13 % of the new-operation questions and 27 to 31 % of the composed ones are answered correctly this way; the rest are refused.

**Not acquired, and why.** Trailing zeros (the step needs a conditional: seven nodes), integer square root and divisor count (no short recurrence), the table lookup (a period the reductions do not reach), bit mirror, and exclusive-or / and / or and what is built on them (the step is nine to eleven nodes). The last digit of `a^b` was found in an earlier version as `f(x, y) = f(x, y + 4)` (its true period) and then lost when I stopped reductions that point away from the table; lcm, hamming distance and sqrt-plus-popcount depend on operations it did not acquire.

**What I did to get here, and what it cost me.** The first version accepted a recurrence whenever it was cheaper than the table, and it stated **2 to 3 wrong facts per seed** from spurious fits (a recurrence that ignored the value at the nearer point and "explained" only the rows its own reduction selected; `f(x, y) = f(x + y, 3)` for bitwise-or). Each fix is recorded under "Mistakes". The three that matter: the table's cost was computed with a code that charges large numbers heavily, which overstated what a chance fit saves for kinds with small answers, so the table is now priced with a code that learns the answers' distribution; a recurrence is fitted again on three quarters of the table and must predict the hidden quarter (four rounds, at least three hidden points reached, none wrong); and an answer worked out by walking a chain of reductions is **never a fact**, only a guess whose confidence is how often such answers have been right.

**This is a development set, and I have read it.** I wrote the mechanism after reading the list of 25 kinds, so these results measure how well I did on a set I knew, not whether the mechanism generalizes. The test that counts is a second set, written without reading Novice's code and not shown to me before I run it once. It has not been run.

### Cost

Counted evaluations (one operation applied at one point), seed 1; the work is almost all in teaching, because a told answer that a believed program already gives costs a few dozen evaluations. Every row has 0 wrong facts except where noted.

| version | evaluations | wall | what changed |
|---|---|---|---|
| first learner | 486 M | 5.3 s | every rival found by enumerating programs two nodes larger than the best |
| rivals by editing the best program | 323 M | 4.4 s | **a wrong fact on seed 2** (below): reverted |
| repair first, data-constants stage dropped | 120 M | 2.7 s | a refuted program is edited with the numbers its mistakes name before anything is searched; the widest stage changed no answer on three seeds, so it went |
| the last level is checked, not built | 58 M | 1.3 s | rivals are enumerated again (completeness), but the largest level is only tested against the data, and nearly every candidate fails at its first or second point |
| a kind as `h(word(x, y))` | **19 M** | **0.33 s** | if the answers are a function of a known word's output, only a one-variable program is searched |

The last row is the one that matters for the bet below. Three stage-2 kinds are explained by composition (seeds 1 to 3: 352 K, 360 K and 356 K evaluations for the three together, about 117 K each). Enumerating with the same 29 words cost 13.6 M each: **about 116 times less for that step**, and about 20 times less for the whole kind (roughly 0.7 M against 14.2 M) once the failed plain pass that runs first is counted. This is the first evidence of the effect rung 6 is after (explaining a new kind gets cheaper when more kinds are known). It is one effect on one tidy exam, not yet a curve, and the cost of answering a question was never the expensive part.

### A teacher who is consistently wrong about some facts (rung 3)

`baby exam adapt-write DIR --seed S --wrong P [--repeat 0.3]`: with probability P a fact is one the teacher is wrong about, and every time it tells that fact (any phrasing) it gives the same wrong answer, so a system cannot tell. The judge counts a wrong fact as a **repeat** (the same fact was wrongly told earlier: the system believed its teacher) or **elsewhere** (a fact never wrongly told: the teacher's mistakes damaged what the system knew; must be 0). Seeds 1 to 3 in each cell.

| teacher wrong about | Novice, 30 % re-asks | baby, 30 % re-asks (recorded for its skeptical exam driver) | Novice, no re-asks | baby, no re-asks |
|---|---|---|---|---|
| 10 % | 4 / 4 / 6 | 5 / 7 / 6 | 1 / 0 / 3 | not run |
| 20 % | 6 / 7 / 17 | 10 / 10 / 18 | 2 / 0 / 7 | 2 (seed 1 only) |
| 40 % | not run | 20 / 38 (two seeds) | 9 / 9 / 7 | not run |

Wrong facts about facts never wrongly told ("elsewhere", must be 0): **0 in every re-ask stream and in every no-re-ask stream but one** (seed 2 at 40 %, as in baby). Read plainly, the repeats are **on par with baby's**, not below them: the like-for-like cells are close, and the one stream where both ran (seed 1, 20 %, no re-asks) gives 2 and 2. **Every remaining wrong fact I checked is in one of the six kinds that have no rule to check against** (a prime test, an exclusive-or, four arbitrary codes): a single source cannot catch a mistake there by any means, as baby's notes also found, and a second independent source is what rung 4 is for (below).

**What it costs: coverage.** On that shared stream (seed 1, 20 %, no re-asks), baby states 59.3 % as right facts and gives 25.0 % right guesses (2.9 % wrong guesses, 12.5 % refused), right across the thirds 82.9 → 84.0 %. Novice states 58.8 % as right facts, 19.1 % right guesses (1.5 % wrong), **20.3 % refused**, 69.2 → 82.4 % across the stream. So Novice is as sure of its facts and wrong less often when it guesses, but it declines where baby guesses and is right. The first version of this rung refused 33 % there: the record of how its guesses outside the taught region had turned out was shared by all kinds, so kinds whose rule really changes (and a teacher who is wrong a fifth of the time) dragged the confidence of the stable kinds below one half. Each kind now keeps its own record, shrunk toward the shared one, which brought it to 20 %. The rest is the price of not guessing from a poor record, and is open.

**How it works, and why no count is tuned.** A told answer that disagrees with a program does not remove the program. The program pays the price of an exception: the bits to say which point (`log2(points + 1)`) plus the bits to write the odd answer. Rivals that agree with the told answer gain on it. One slip costs the believed program about 14 bits, and the rival that explains a real change of rule (a ceiling at 30) costs about 16 bits more to write, so one contradiction changes nothing and two that a rival explains together win. That is baby's hand-set "three coherent answers", derived from the costs. A point where a told answer was refused is never stated as a fact, only guessed. Saying the same thing again (a re-ask, another phrasing) is counted, not weighed twice. On honest streams the learner is a little slower to adopt a rule that really changed (wrong guesses 1.1 % against 0.6 % before this rung).

**Caveats, plainly.**
- **The exam is tidy and was written by the same author as the learner.** The kinds are small arithmetic rules over the integers 0 to 15. The operations Novice starts with (`+ - * min max div mod`) are my choice and fit that domain; that the exam is passed says nothing about worlds the starting operations cannot express. Two of the six unexplainable kinds (a prime test and an exclusive-or) have a rule that the starting operations cannot say; rung 5 is where that is tested properly.
- **Words are only a first step toward a language that grows.** Earlier explanations become words; no new primitive is ever made. The two words are compositions of the starting operations.
- **Joules are upper bounds from one machine and one seed.** Nothing here yet shows that adapting makes anything cheaper in joules; the composition result is about the cost of learning a kind, not of answering a question (which was never expensive).
- **Sources must err independently.** Two sources that make the same mistake are believed, and a reliability can only be measured where some kind has a rule. The second source in the exam is generated to be independent.
- **Only the adaptive exam.** The static exam (with its traps) has not been run on Novice. A teacher who is wrong about a whole kind of thing, not about some facts, is not caught (nothing inside one source can catch it).
- **Guesses are labeled but crudely calibrated.** Outside the region it has seen, the confidence it states is the kind's own record of right guesses, shrunk toward the shared record at that distance. The judge has not yet been asked whether the stated confidences match the realized rates.
- **The failed searches are still most of what is left**: the six lookup kinds are searched, fail, and are searched again as they grow.

### Curriculum: what knowing more does to the cost of learning (rung 6, first measurement)

`cargo run --release --offline --example curriculum -- EXAM_DIR [REPEATS] [KINDS_PER_BATCH]` teaches the 45 kinds of an adaptive exam in nine batches of five, in the order of their roles (base, hidden, stage 2, deceptive, no rule), to one system that keeps what it learned. As a control it teaches each batch to a system that knows nothing. The exam's unit labels only cut the lessons into batches; Novice never sees them. Seed 1, median of 7 runs, counted evaluations and wall time (energy for these windows is about 0.17 µJ per evaluation, e.g. 548 mJ for batch 5, and too noisy to tabulate):

| batch | roles | evaluations, in order | evaluations, knowing nothing | ms, in order | ms, nothing | kinds explained, in order | kinds explained, nothing |
|---|---|---|---|---|---|---|---|
| 1 | base | 158,428 | 158,428 | 2.9 | 2.9 | 5 of 5 | 5 of 5 |
| 2 | base + hidden | 807,699 | 807,699 | 8.6 | 8.5 | 5 of 5 | 5 of 5 |
| 3 | hidden | 249,251 | 249,251 | 4.4 | 4.4 | 5 of 5 | 5 of 5 |
| 4 | hidden | 2,095,000 | 2,095,000 | 19.5 | 19.3 | 5 of 5 | 5 of 5 |
| 5 | hidden | 3,154,784 | 3,154,784 | 28.1 | 27.3 | 5 of 5 | 5 of 5 |
| 6 | stage 2 | 1,111,974 | 1,111,974 | 13.2 | 13.0 | 5 of 5 | 5 of 5 |
| 7 | stage 2 + deceptive | 2,540,805 | 2,089,929 | 25.1 | 15.3 | **5 of 5** | **1 of 5** |
| 8 | deceptive + no rule | 263,013 | 243,073 | 4.5 | 3.3 | 4 of 5 | 4 of 5 |
| 9 | no rule | 673,325 | 612,788 | 6.9 | 5.1 | 0 of 5 | 0 of 5 |

Seeds 2 and 3 have the same shape (batch 7: 2.47 M and 2.62 M evaluations against 2.02 M and 2.16 M, five kinds explained against one). **What it shows, and what it does not.**
- Knowing earlier kinds buys *capability* that the same effort cannot buy otherwise: four kinds that are built from earlier ones (sum of squares plus one, half of it, the square of an average, a difference of squares) are explained, in about 17 to 22 % more work than the system that cannot explain them. Without the library they are out of reach of any search this small.
- Learning is **not** cheaper in total: where the kinds do not depend on each other (batches 1 to 6) the cost is exactly the same, and the kinds that cannot be explained cost 4 to 10 % more with a library than without. The earlier result (composition about 116 times cheaper than enumerating with the same words) is a statement about *how* a dependent kind gets explained, not about learning getting cheaper over a curriculum. So the efficiency bet is **not shown** by this measurement; what is shown is that experience can be kept without making failure expensive, which was not true of my first version (below).

## What is hand-built (each to be removed by the rung named)

| choice | where | removed by |
|---|---|---|
| the starting operations `+ - * min max div mod` | `program.rs` | rung 5 (a language that grows) |
| the constants `0 1 2 3 10`, then the numbers a refuted program got wrong | `learner.rs` | rung 5 |
| a fact needs all but 2^-10 of the belief; a table is kept unless a program is cheaper | `learner.rs` (`RISK_BITS`) | the error budget is a requirement, not a mechanism; kept |
| rivals: every program within two nodes of the best, every program one edit away, every other word that explains the kind | `program.rs`, `learner.rs` | the cost rule "within `RISK_BITS` of the best" would replace the node count; rung 6 |
| search sizes (7 nodes; 5 once words are on offer), 200,000 expressions | `learner.rs` | rung 6 (search effort proportional to what it can win) |
| probe points around the region, to keep rivals apart | `learner.rs` | rung 6 |
| a group is searched once it has four points; a table group is searched again when it has doubled | `learner.rs` | rung 6 |
| guesses outside the region: confidence = the kind's own record of right guesses, shrunk toward the shared record at that distance | `learner.rs` | a calibration learned per kind from more than counts; open |
| the price of an exception is `log2(points + 1)` bits plus the bits of the odd answer; a repair may use programs up to 12 bits above the favorite | `learner.rs` | a teacher-reliability estimate learned per source (rung 4) |
| a kind is named by the word that makes the answer a function of the numbers and covers the most questions | `learner.rs` | breaks with noisy lessons; open |
| a word is any explained kind with a body of at least three nodes whose behavior is new | `learner.rs` | rung 5 (a compression test instead) |
| recurrences: reductions of up to five nodes (unary) or two of three (binary), constants 0 to 8 and 10, the 24 cheapest that reach 10 points; a step of at most five nodes (seven for the best six); four-fold hold-out with at least three hidden points reached; three validated recurrences kept as rivals; answers worked out by a chain are guesses | `learner.rs` | rung 5 (a language that grows beyond what a table can say) |
| the lessons count as confirmed 2^11 times before anything checks them | `learner.rs` (`LESSONS_PRIOR`) | measured, not assumed; open |
| 100 possible answers when a source is wrong (`ANSWER_SPACE`); the lessons and the exam's main teacher count as one speaker | `learner.rs` | an estimate from the answers seen; open |

## Mistakes and dead ends (newest first)

- **Rung 5, in the order I made them.** (1) A recurrence was tried before the closed-form search could use learned words, and it preempted a better explanation: for the sum-of-squares-plus-one kind it accepted a true identity that reaches few points (13 of 18 questions refused). Now a recurrence is the last resort. (2) The table was priced as if every answer cost bits in proportion to its size; kinds with small answers then looked more compressible than they were, and spurious recurrences saved 25 to 57 bits. Now the table is priced with a code that learns the distribution of the answers. (3) I ranked candidate reductions by how much they narrow the answer, which any reduction that maps every point to a different point wins trivially; they are now ranked by cost among those that reach enough points. (4) My first hold-out used a hash that hid or kept consecutive small numbers together, leaving unary kinds with no training rows at all, and a single hold-out has no power on 16 points; the check now runs over four rounds. (5) Accepting the cheapest candidate and giving up when it failed validation lost genuine recurrences whose cheaper cousins were partial truths (`r + 1, r = f(x mod min(x, 8))` is right for 9 to 15 and 17); candidates are now tried in cost order. (6) A symmetry (`f(x, y) = f(y, x)`) beat Euclid's rule on a dense table because it is cheaper and true; both are kept as rivals, and a recurrence whose chain reaches nothing abstains instead of counting as doubt. (7) I required a reduction to reach 40 % of the table, which is far above what the true reduction reaches on a sparse table of pairs.

- **Failing got dearer as the library grew, and I guessed the cause wrong twice.** In the curriculum measurement the kinds that cannot be explained cost 4.7 times as much with a library as without in the first measurement (batch 9: 2.87 M evaluations against 0.61 M; 2.19 M after my first two fixes). My first explanation (a word with all-distinct outputs makes "the answer is a function of the word's output" trivially true; unsolved kinds were retried on every new word) was partly right and moved nothing, so I added a collision requirement and a doubling back-off and measured again: barely changed. My second (every new kind's first blind search ran over a larger alphabet because earned words were in it) was also not the cause: the result was identical. Only a trace of what each kind was charged showed it: after the cheap search failed, a blind enumeration over all 33 offered words cost 433 K per unary kind and about 13 M per two-input kind. Now composition (one lookup per word) uses every word on offer, and blind enumeration only the words that have earned their place; the cost of failing is 4 to 10 % above the control. The collision requirement and the back-off stay (they are sound), and the first search no longer carries words.

- **I first compared Novice with the wrong baby numbers and claimed it beat baby on repeats.** The recorded baby figures (5/7/6, 10/10/18) are for streams with 30 % re-asks; I set them against Novice runs without re-asks. When I ran baby on a stream without re-asks, it matched Novice (2 and 2), and the like-for-like cells are close. The claim was in the commit message of the first rung-3 commit and in the description of the pull request that carried it; the pull request description is corrected, the commit message stays as pushed (history is not rewritten), and the table above states what was compared with what. The same run showed that Novice's refusals under a wrong teacher were far above baby's (33 % against 12.5 %), which led to the per-kind record.

- **A repeated answer was weighed as new evidence.** My first version of the exception price penalized the believed program every time a wrong answer was told, including when the same wrong answer came again as a re-ask, while programs found by editing were priced against it once. The believed rule (`div(x, 10)`) then lost to a rival (`div(x, 9)`) that happened to agree with some wrong answers, and the system stated a wrong fact about a fact the teacher had never got wrong (1 on one of six streams; the must-be-zero column). Fixed by weighing a (question, answer) pair once; the same stream now has 0.

- **Rivals found only by editing the best program gave a wrong fact.** To save work I replaced the enumeration of larger programs by edits of the best one (wrap it in an operation, swap a constant). Cost fell from 486 M to 323 M evaluations and the exam was unchanged on seed 1, but on seed 2 the first stream question, a digit-sum kind, was answered `0` as a fact where the truth was `9`. The lessons skipped 7, 9 and 10, `x mod 9` fits every one of them, and it is about 7 bits cheaper to write than the real rule; the real rule is not one edit away from it. The enumeration had been keeping it as a rival with enough belief to stop the fact. The right completeness rule is a cost rule: any program within `RISK_BITS` of the best can keep a fact from being stated and must be considered; edits are an addition (they find ceilings and wrap-arounds that cost many bits more), not a substitute. Restored, made cheap another way (the last level is only checked), and pinned by a test built from that exact case.
- **The widest search (every number in the data as a constant) was dead weight once contradictions are repaired by editing.** It cost 65 M evaluations, almost all failing on the lookup kinds, and removing it changed no answer on three seeds.

- **Grouping by the rarest word was wrong.** My first version named a question's kind by its rarest word. It passed the exam, because the kind word appears three times per fact while "is" appears everywhere. A small test world broke it at once: there "is" was rarer than the kind word and one kind split into three groups. Replaced by the rule in the table above, which gives byte-identical exam results and passes the small world.
- **It stated a negative result for a kind that never showed one** (`3 - 5` as `-1`, guessed at 99 %), and answered a question that contained a word it had never met. Both were caught by the tests I wrote for them before recording the results, and both now refuse.
- **My first expectation for the saturating sum was wrong.** I expected a fact one step outside the region; the system rightly guessed. The sequence it follows is worth recording: after the first contradiction a rival (`y + min(x, 10)`) that fits all the data by luck survives and is believed; the second contradiction removes it, and only then is the ceiling found.
- **First run of the learner** (before words and before constants named by refuted programs): 74.9 → 80.2 %, stage-2 kinds 52 → 72 %, deceptive kinds 66 %. Both additions were made because of these numbers.

## Rules (the ones that protect the results)

- **WRONG is reported apart from refused**, and a stated fact that is wrong must stay at 0. A guess may be wrong; it must say how sure it is.
- **Add must-refuse questions with every capability** and think adversarially about what could be answered but should not be.
- **Record mistakes and rejected hypotheses here as they happen.** Dead ends are part of the record. This file is the notebook for Novice, newest section first, once there is something to put in it.
- **Report joules as upper bounds** unless the machine was quiet; never build or compile while a measurement runs; energy exists only where the CPU's counter is readable (Linux).
- **Say what is hand-built.** Every constant that was chosen, not found, is listed with the rung that is supposed to remove it.

## Platforms

Novice is checked on Linux x86_64 and aarch64 for every change (tier 1). macOS, Windows and musl are built and tested weekly and on demand (tier 2); wider coverage (the BSDs, WebAssembly, 32-bit, big-endian) is added at milestones, once there is a result worth keeping identical. Because decisions use integers only, identical results across platforms are expected by design and checked at milestones, not on every commit. baby's own matrix is unchanged and runs when baby changes.

## License

`AGPL-3.0-only`, like the rest of Matter. See the repository's `LICENSE` and `CONTRIBUTING.md`.
