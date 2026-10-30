# Novice 0.0.1: research snapshot

Published 2026-10-30. Novice takes the spotlight in Matter; Baby remains the frozen reference with its documentation and historical artifacts intact. Development is pausing while research and foundation maintenance continue. The core stays Rust-first, CPU-accessible, dependency-free, and AGPL-3.0-only.

## Build and run

Use a recent stable Rust toolchain and Python 3 for the exam generators. Novice is a separate Cargo workspace; the root workspace supplies Baby's independent judge. This is a source release, not a crates.io package or a pretrained download.

```bash
git clone https://github.com/Matterworm/Matter.git
cd Matter
git checkout novice-v0.0.1
cargo test --manifest-path novice/Cargo.toml --release --offline
cargo build --manifest-path novice/Cargo.toml --release --offline
cargo build --release --offline
./target/release/baby exam adapt-write /tmp/novice-exam --seed 1
EXAM_NO_JOULES=1 ./target/release/baby exam adapt-drive /tmp/novice-exam "$PWD/novice/target/release/novice" serve
bash novice/golden/check.sh
```

Choose a new output directory if `/tmp/novice-exam` already exists. `EXAM_NO_JOULES=1` disables energy measurements for this correctness check. The `serve` command speaks an exam protocol; it is not an interactive chat interface. The static exam is also available (`novice static DIR --out FILE`), with the same Baby judge.

## What the release supports

Novice finds question groups without teacher-supplied unit labels, compares table and program explanations in bits, learns reusable program pieces and number-word grammars, and tracks sources. On the latest initially blind set K, the three seeds recorded zero wrong facts and no answered traps, but acquired none of the twelve hard kinds. Correct control facts covered 15.5%, 14.5%, and 14.5% of control questions. Constant inference added one kind per seed; evaluations rose 1.8% overall relative to the earlier build. Refusal is reported separately from error.

## Known limitations

- **Internal validation is not independent evaluation:** this release removes target-dependent exclusions in `holds_out`. Every reachable held-out prediction counts, including errors. However, recurrence shapes were selected on the full table. A new outer split independent of candidate selection is still needed; historical acquisition claims retain this limitation.
- **Transfer:** the latest blind set acquired no hard kind. Good scores on the tidy exam do not establish broad reasoning. Every already-read exam is development material now.
- **Efficiency:** a 19% evaluation reduction on earlier development sets did not transfer to K. Counted evaluations are not measured joules. Earlier energy comparisons concern particular builds, exams, and noisy CPU-package windows; no decreasing lifetime joule cost per useful answer has been established.
- **Language:** supplied primitives, grammar search limits, budgets, and protocol choices remain programmed. This is not a general language assistant.

The release tightens recurrence validation, updates the public introduction, and adds a research retrospective. No new capability or benchmark improvement is claimed. Historical set-K figures above describe the earlier build; current verification is recorded below. See the [complete experiment log](../novice/README.md), [Baby reference](../baby/README.md), and [project website](https://matterworm.org/).

## Publication verification (2026-10-30)

The corrected build passes all 124 Novice tests in debug and optimized release builds (Rust 1.98.1, Linux x86_64). Baby's 155 tests and its three-run golden check pass unchanged. The new recurrence regression fails on the old implementation and passes after the correction; a larger-table positive control remains accepted. In the existing cost-allocation fixture, accepted recurrences decrease from four to three because the small popcount table no longer passes by excluding errors.

Set K was replayed with the before-fix and corrected release binaries on the same generated worlds, seeds 1 through 3, with energy measurement disabled. **All replies, judge output, and counted-work reports are byte-identical before and after on each seed.** These are development replays of an already-read set, not a new blind test.

| Seed | Wrong facts | Traps answered | Correct control facts | All questions refused | Counted evaluations |
|---|---:|---:|---:|---:|---:|
| 1 | 0 | 0 | 15.5% | 91.8% | 49,845,200 |
| 2 | 0 | 0 | 14.5% | 92.1% | 55,992,141 |
| 3 | 0 | 0 | 14.5% | 92.1% | 46,833,651 |

No hard question was answered on any seed. The historical set-K coverage figures therefore also describe this corrected build on these same files. They do not resolve the independent-evaluation limitation or imply generalization to a fresh task. No new energy result is claimed.

The 13-case golden comparison generated two byte-identical runs. Twelve complete golden files were unchanged. Only set A's work report changed; its judge table and every reply remained identical. Set A's evaluations rose from 49,408,765 to 52,690,213 (6.6% more), accepted recurrences requiring skipped errors fell from two to zero, and fewer told answers narrowed doubt. Goldens were deliberately regenerated for that stricter validation behavior. This correction is an honesty improvement in the internal check, not an efficiency win.
