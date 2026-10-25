# Novice

The main line of work in Matter, **rebuilt from scratch**. [`baby`](../baby/README.md) is frozen as the reference and the measuring stick; Novice shares no code with it. What carries over is the [entry exam](../baby/README.md) (a plain-file and line-protocol judge that any system, in any language, can take), the goldens-and-platform habits, and the rules below.

> **Status (2026-10-25): started.** Rung 0 exists: a system that speaks the exam's protocol, learns nothing and refuses everything. It scores 0 wrong facts and 0 right answers, which is the floor every later rung has to beat. Nothing else works yet, and nothing here is a result.

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
| 1 | remembers what it was told (a table of facts) | 0 wrong facts; right on the facts it saw, refuses the rest; first joule numbers | Open |
| 2 | finds programs from a minimal start | right on questions it never saw, for kinds a short program can say; routing found without the labels | Open |
| 3 | rival explanations scored in bits | on the 14 streams of baby's wrong-teacher work: as few believed mistakes as baby's, with **no tuned count** | Open |
| 4 | sources | with a second, right source added, repeats fall in kinds that have no law to check against | Open |
| 5 | the language grows | a kind that needs an operation Novice does not start with, from worlds written apart from the learner, solved by a primitive it made | Open |
| 6 | answers get cheaper | joules per answer on a fixed set of questions fall as it learns, wrong facts stay 0, and the break-even point (learning joules spent against answer joules saved) is reported | Open |

Rungs 0 to 4 are the first milestone (the tidy exam, with a source tag). Rung 5 is the test of "genuine"; the worlds for it are written by a process that the learner's author does not read until the review, so the learner cannot be tuned to them. Rung 6 is the bet itself.

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
