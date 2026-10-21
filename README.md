<p align="center">
  <img src="assets/banner.svg" alt="TransientAI" width="100%">
</p>

<p align="center">
  <a href="LICENSE"><img alt="License: AGPL-3.0-or-later" src="https://img.shields.io/badge/license-AGPL--3.0--or--later-blue"></a>
  <a href="https://github.com/DawsMacs/TransientAI/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/DawsMacs/TransientAI/actions/workflows/ci.yml/badge.svg"></a>
  <img alt="Rust, zero dependencies" src="https://img.shields.io/badge/rust-zero%20dependencies-orange">
</p>

**TransientAI is an experimental learning system written in Rust, built to run on ordinary consumer hardware.** It explores
*adaptive learning*: a system that keeps learning while it is used, decides for itself *how* to learn each kind of thing, and is
measured by what it costs (operations, memory, estimated joules) rather than by parameter count alone.

The current artefact is `baby/`: about 8k lines of Rust, **no dependencies**, builds offline, 83 tests. It is a research prototype
evaluated on a synthetic curriculum. It is not a language model and makes no claim to be one yet.

> **Status:** active research. Every number below comes from tidy synthetic lessons and *estimated* energy. Read
> [Limitations](#limitations) before drawing conclusions.

## What `baby` does

1. **Learns from units of lessons.** A unit is a group of related question/answer pairs (capitals, sums, "which is bigger", ...).
   The teacher can group them, or `baby discover` finds the units itself from an unlabelled heap.
2. **Chooses how to learn each unit, from evidence.** For every unit it picks among five mechanisms using cross-validation on facts
   it holds back, preferring the cheaper on ties:

   | mechanism | what it does | typical use |
   |---|---|---|
   | `remember` | store the lesson, answer by weighted similarity (inverted index) | facts: capitals, colours, codes |
   | `rule` | exact ridge fit of a per-word linear combination of quantities | sums, differences, doubling, halving |
   | `select` | pick the larger or smaller of two numbers | comparisons |
   | `program` | enumerative search over small expressions (`+ - x max min apart half`, up to 7 pieces) | multiplication, "how far apart" |
   | `network` | small bag-of-words net | rarely needed |

3. **Routes questions with a precomputed table** of how telling each word's *shape* is for each unit (numbers count as "a number"),
   then lets that unit's mechanism answer or decline.
4. **Explains every answer and every refusal.** It says how it knew, and refuses (with a reason) on unmet words, unrecognised
   kinds, memorise-only units it was not told the fact for, and mechanism-level declines (negative results, out-of-line numbers,
   missing operands).
5. **Keeps learning after training.** A surprise gate skips lessons it can already work out; consistent lessons refit a unit in
   place; unrecognised lessons wait in a *pen* and open a new unit once enough gather; `baby sleep` merges units that follow one
   rule, splits units that stopped following one, and can forget lessons that are fully explained (`--compact`).

## Measurements

All on the synthetic curriculum in `baby/src/kinds.rs` (13 units, 792 lessons), one laptop core, no GPU.

| claim | result |
|---|---|
| scorecard: taught / reworded / never taught / beyond range | right 100 / 100 / 95.1 / 100 %, **0 % wrong**, the rest refused; identical on seeds 1-5, for the taught school and the self-organised one |
| answering effort | 8,977 -> 57 operations per question (159x fewer); about 85x less *estimated* energy; same answers |
| fitting a rule | exact solve vs 800-pass SGD: 392x fewer operations, ~60x faster, same rule |
| teaching 792 lessons from nothing | 0.34 s, 11.9 MB peak |
| a lesson it already understands / a new phrasing / a new kind | 0.19 ms (not stored) / 0.4 ms / 1 ms (a unit opens at the 6th) |
| **scale** (clean synthetic world, 20 -> 600 units, 828 -> 25,776 lessons, up to 10,000-word vocabulary) | teaching 26k lessons 2.65 s and 56 MB; learning one new fact **0.4-0.6 ms at every size**; answering ~10 us; reorganising 1.35 s; 100 % right, **0 % wrong**, all traps refused |
| **inventing mechanisms** (16 hidden + 9 stage-2 numeric kinds in digits, 20 seeds) | all 16 hidden kinds worked out every time; 7 of 9 stage-2 kinds every time using constants and programs it learned earlier (taught in the wrong order: 8 of 72); 0 of 120 unexplainable kinds explained; 0 wrong; digit sum declined as ambiguous |
| **asking** (digit sum taught on 0-15, 6 seeds) | where two explanations fit, it asks the number that separates them: 100 % after one question where random numbers need three; it asks nothing about random codes, primes or xor (0 wrong) |
| **its own reliability bar** | measured from the noise in its lessons (it estimates 9.6 % where the world has 9.7 %) and set just under what a correct rule can reach; at 10 % label noise unseen questions answered 98 % instead of 42 %; clean worlds unchanged |
| **label noise** (2 % and 5 % of taught answers wrong, 60-200 units, words and digits) | **0 % wrong** on unseen questions; at 2 % essentially as good as clean data (97-100 % of unseen questions right); wrongly taught lessons answered with the truth by voting memory, robust rule fitting and cross-checks |
| storage | school file 29 KB, of which 1.3 KB is what was learned; `--compact` keeps 49 % of the file, still answers all 792 taught lessons, 0 % wrong |
| unit discovery from 792 unlabelled, shuffled lessons | 9 units found (13 named by the teacher), scorecard identical |

The scorecard reports **WRONG separately from refused**: a refusal is an acceptable answer, a bluff is not.

## Quick start

Requires a stable Rust toolchain (edition 2021). No other dependencies, no network.

```bash
cargo test --release --offline         # 83 tests (about 15 s)
cargo build --release --offline
./target/release/baby school           # teach 13 units (~4 s), print how each was learned and the scorecard
./target/release/baby ask "how far apart are two and nine"   # answers, and says how it knew
./target/release/baby ask "three minus five"                 # refuses: negative result
./target/release/baby discover --out runs/discovered.school  # find the units from unlabelled lessons
./target/release/baby cost             # time and memory of learning
./target/release/baby scale --sweep 20,60,200,600   # does learning stay light as the world grows? (about 5 min)
./target/release/baby invent --seeds 20            # can it work out kinds of lesson nobody wrote a mechanism for? (about 8 min)
./target/release/baby wonder [--check]             # what would it like to be told? (--check: where would it test its own explanations?)
```

If the scorecard's WRONG column is ever above 0.0 %, or a refusal battery in `school.rs` starts answering, treat it as a
regression: honesty is the property this project protects first.

## Code map

All under `baby/src/`.

| file | role |
|---|---|
| `invent.rs` | the invention experiment: hidden kinds nobody wrote a mechanism for, and kinds that must stay unexplained |
| `scale.rs` | the scale test: a generator of large synthetic worlds (units, vocabulary, noise, traps) and the measurements |
| `school.rs` | the unified system: units, routing table, recognition floor, `ask`, `learn`, pen, `reorganise`, `compact`, save/load |
| `mechanisms.rs` | the five mechanisms, cross-validation, mechanism choice, the inverted-index memory |
| `discover.rs` | unit discovery from unlabelled lessons (anchor words, fact and rule-coherence merges) |
| `program.rs` | expression search and ambiguity detection (by behaviour, not by taught examples) |
| `quantity.rs` | number line, per-word operator weights, exact ridge fit |
| `kinds.rs` | the synthetic curriculum |
| `ledger.rs` | counted operations and words moved, priced with published 45 nm figures; optional RAPL reader |
| `memory.rs`, `store.rs` | user-taught open-vocabulary facts and the weighted-Jaccard fact store |
| `net.rs`, `model.rs`, `data.rs`, `know.rs`, `life.rs`, `curious.rs`, `rules.rs`, `rng.rs` | the earlier from-scratch network and its experiments, plumbing |
| `main.rs` | the CLI (`baby <command>`) |

## How results are judged

- **WRONG is reported apart from refused.** Accuracy alone hides bluffing.
- **Adversarial batteries are part of the tests.** The same trap (`|a-b|`, `a-b` and `10-n` agree on every taught lesson) caught
  the system three times; ambiguity is now defined by behaviour, merges are checked per group, and operand order is kept.
- **Energy is estimated**, as counted operations times published per-operation costs. Ratios are the trustworthy part. A real
  meter (RAPL) is wired in but needs a one-time permission change on the host.
- **Dead ends are written down.** `baby/README.md` records wrong hypotheses and bugs next to the results.

## Limitations

- Lessons are synthetic and regular (numbers 0-10, a handful of templates). "0 % wrong" is a spot check on in-style questions
  plus hand-written must-refuse batteries, not a proof.
- The five mechanisms, the expression language, the number line (taught by counting lessons) and the thresholds are the author's
  design choices. The system selects and fits among them; it does not invent a sixth.
- **Only label noise was tested** (2 % and 5 % of taught answers wrong, up to 200 units). Where most of one fact's own phrasings are wrong nothing can know, and at 5 % a few such answers remain;
  other noise kinds (ambiguous entities, real misspellings, noise correlated across phrasings) were not tested.
- Nothing here yet shows behaviour on real text, at millions of lessons, or at "frontier" quality. Those are the open questions the
  roadmap exists to answer.

## Roadmap

1. **Scale test, continued:** clean worlds are done to 600 units (learning one lesson is flat). Next: asking about more than numeric programs, discovery at scale, and numeric learning knobs adapted from measurement, a Zipf-shaped vocabulary, discovery at scale (its pair loops are still quadratic).
2. **Compact lossless storage format**, then incremental and exact updates, then the real energy meter.
3. **Beyond expressions:** mechanisms of a different kind (not a formula), and the baby adapting its own learning knobs; a number
   line it discovers rather than is taught. (Building programs from a learned library, and digits as numbers, are done.)
4. **Bridge to real data:** small factoid and word-problem corpora, to see which of units, mechanisms, discovery and refusal survive.
5. **Controlled comparison:** a matched mini transformer against `baby`-style systems at three or more sizes, on quality per joule,
   retention after new learning, adaptation speed and scaling slope.
6. **Growth:** a unit that starts tiny and adds capacity only when it saturates (function-preserving growth).

## Ambition

The long-term target is capable, continually learning AI that fits a consumer device's RAM, storage and power budget, and that
anyone can run and teach on their own hardware. Whether an architecture like this can reach that is unknown; the repository exists
to find out by measurement. The reasoning behind the project, in the author's own words, is in
[`TransientVision.md`](TransientVision.md) and [`docs/Direction.md`](docs/Direction.md).

## Documents

| file | contents |
|---|---|
| [`baby/README.md`](baby/README.md) | the experiment log: every result, dead end and bug, newest first |
| [`LEARNING.md`](LEARNING.md) | learning-efficiency research notes and sources |
| [`docs/Direction.md`](docs/Direction.md) | strategy, hardware terrain, language choice, candidate architectures |
| [`TransientVision.md`](TransientVision.md) | the author's dated vision entries |
| [`TransientHandover.md`](TransientHandover.md) | current state and next steps, for resuming work |

## Licence

Copyright (C) 2026 DawsMacs. Licensed under the **GNU Affero General Public License, version 3 or (at your option) any later
version** (`AGPL-3.0-or-later`). The aim is that this work stays open: anyone may use, study, change and share it, and anyone who
distributes it, or lets others use a modified version over a network, must offer their users the corresponding source under the
same terms. See [`LICENSE`](LICENSE); per-file licensing is declared in [`REUSE.toml`](REUSE.toml) and SPDX headers.

Contributions are welcome under the same licence; see [`CONTRIBUTING.md`](CONTRIBUTING.md).
