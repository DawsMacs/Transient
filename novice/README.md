# Novice

The main line of work in Matter, **rebuilt from scratch**. [`baby`](../baby/README.md) is frozen as the reference and the measuring stick; Novice shares no code with it. What carries over is the [entry exam](../baby/README.md) (a plain-file and line-protocol judge that any system, in any language, can take), the goldens-and-platform habits, and the rules below.

> **Status (2026-10-25): started; rungs 0 to 2 done on the tidy adaptive exam.** It finds the kinds of question itself, explains most of them with programs priced in bits, builds later explanations from earlier ones, and states no wrong fact. See "Results so far" for the numbers and, just as important, for what they do not show. Joules are not measured yet (the energy counter was unreadable this session); work is reported as counted evaluations and wall time.

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
| 3 | rival explanations scored in bits | on the 14 streams of baby's wrong-teacher work: as few believed mistakes as baby's, with **no tuned count** | Open |
| 4 | sources | with a second, right source added, repeats fall in kinds that have no law to check against | Open |
| 5 | the language grows | a kind that needs an operation Novice does not start with, from worlds written apart from the learner, solved by a primitive it made | first step only: earlier kinds become words (see below); the test itself is open |
| 6 | answers get cheaper | joules per answer on a fixed set of questions fall as it learns, wrong facts stay 0, and the break-even point (learning joules spent against answer joules saved) is reported | Open |

Rungs 0 to 4 are the first milestone (the tidy exam, with a source tag). Rung 5 is the test of "genuine"; the worlds for it are written by a process that the learner's author does not read until the review, so the learner cannot be tuned to them. Rung 6 is the bet itself.

## Results so far (2026-10-25)

Measured on the adaptive exam (`baby exam adapt-write DIR --seed S`: 45 kinds, 3,048 lessons, a stream of 787 questions with the truth told after each answer), the same files for both systems, one desktop core, no joules. "Right" = stated as a fact and right, plus right guesses; the judge's thirds are the first, middle and last third of the stream.

| seed | system | wrong facts | facts right | wrong guesses | refused | right, 1st third → last third | wall time |
|---|---|---|---|---|---|---|---|
| 1 | Novice | **0** | 67.7 % | 0.8 % | 11.2 % | 86.7 → 88.5 % | 5.3 s |
| 1 | baby | 0 | 64.2 % | 1.5 % | 11.4 % | 85.2 → 87.8 % | 15.6 s |
| 2 | Novice | **0** | 68.1 % | 0.8 % | 11.2 % | 85.9 → 91.2 % | 5.3 s |
| 2 | baby | 0 | 64.8 % | 1.5 % | 11.2 % | 84.4 → 90.8 % | 17.5 s |
| 3 | Novice | **0** | 68.6 % | 0.8 % | 11.2 % | 87.5 → 90.1 % | 5.3 s |
| 3 | baby | 0 | 64.4 % | 1.5 % | 11.2 % | 86.7 → 89.7 % | 16.9 s |

The kinds that change their rule outside what was taught (a sum that saturates at 30, one that wraps at 40) climb from 88.6 / 80.0 % right (seeds 2, 3) to 100 % in Novice, from 71.4 / 68.0 % to 96.4 / 97.4 % in baby. The 11.2 % refused are the same in both: the six kinds that have no rule it can say (a prime test, an exclusive-or, four arbitrary codes), which cannot be answered before they are told.

**What it found, with no labels and no menu of mechanisms** (seed 1): `x + y`, `x - y`, `x + 1`, `div(x, 2)`, `max(x, y) - min(x, y)`, `x * 3`, `div(x, 10) + mod(x, 10)`, `mod(x + y, 40)` (the 40 is the number a refuted `x + y` got wrong: it said 45 where the truth was 5), `min(x + y, 3 * 10)`, and, from words it had earned, `successor(sum_of_squares(x, y))`, `halve(sum_of_squares(x, y))` and `square(average(x, y))`. Five kinds became words that the language then used; the other 34 explained kinds did not.

**Caveats, plainly.**
- **The exam is tidy and was written by the same author as the learner.** The kinds are small arithmetic rules over the integers 0 to 15. The operations Novice starts with (`+ - * min max div mod`) are my choice and fit that domain; that the exam is passed says nothing about worlds the starting operations cannot express. Two of the six unexplainable kinds (a prime test and an exclusive-or) have a rule that the starting operations cannot say; rung 5 is where that is tested properly.
- **Words are only a first step toward a language that grows.** Earlier explanations become words; no new primitive is ever made. The five words in the table are compositions of the starting operations.
- **Comparison with baby is not a fair fight on cost.** baby was built to run the same exam with a menu of mechanisms and many thresholds; Novice's 5.3 s still hides about 480 million counted evaluations, almost all of it a search for rival explanations that is repeated for every kind. That is the largest thing to fix (rung 6), and nothing here is evidence yet that adapting makes anything cheaper.
- **No joules.** The energy counter was unreadable this session. Wall time on one core is the only cost number, and it is not an energy number.
- **Only the adaptive exam, only a clean teacher.** The static exam (with its traps) and the wrong-teacher tier have not been run on Novice. A source that is consistently wrong would be believed today.
- **Guesses are labeled but crudely calibrated.** Outside the region it has seen, the confidence it states is how often its guesses at that distance have been right so far, shared across all kinds.

## What is hand-built (each to be removed by the rung named)

| choice | where | removed by |
|---|---|---|
| the starting operations `+ - * min max div mod` | `program.rs` | rung 5 (a language that grows) |
| the constants `0 1 2 3 10`, then the numbers in the data and the ones a refuted program got wrong | `learner.rs` | rung 5 |
| a fact needs all but 2^-10 of the belief; a table is kept unless a program is cheaper | `learner.rs` (`RISK_BITS`) | the error budget is a requirement, not a mechanism; kept |
| search sizes (7 nodes, then 5 with data constants, 5 with words), rivals within 2 nodes, 200,000 expressions | `learner.rs` | rung 6 (search effort proportional to what it can win) |
| probe points around the region, to keep rivals apart | `learner.rs` | rung 6 |
| a group is searched once it has four points; a table group is searched again when it has doubled | `learner.rs` | rung 6 |
| guesses outside the region: confidence = the shared record of right guesses at that distance | `learner.rs` | rung 3 |
| a kind is named by the word that makes the answer a function of the numbers and covers the most questions | `learner.rs` | breaks with noisy lessons; rung 3 |

## Mistakes and dead ends (newest first)

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
