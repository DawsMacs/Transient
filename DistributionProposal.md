<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 DawsMacs -->

# Distribution proposal (version 2): running Transient on other platforms without touching its logic

Drafted 2026-10-23, revised 2026-10-24. **Status:** Phase 1 is built and had its first CI run on 2026-10-24 (PR #1; results in section 3.1).
Phases 2 to 4 are proposals and are not approved. Claims are marked as *read from the source*, *cited*, or *not verified*. This
is the assistant's reasoning; the decisions are Daws's.

This version merges version 1 with the review in `DPVerdict.md` and a read-only source review. Where they disagreed, the decision
is stated once, in section 5.

## 1. Position

The logic files stay exactly as they are. The port is new files, a few relocated measurement functions, and a test matrix. Every
other platform is compared against what Linux/x86_64 produces today. Differences are reported and never absorbed. The recorded
bars (the exam numbers) stay as measured on Linux/x86_64. A platform adds a row to a "tested on" table and nothing else. No
threshold is widened to make a platform pass.

## 2. What the source and the research say

| Question | Finding |
|---|---|
| Will the core build elsewhere? | Very likely. No dependencies, no `unsafe`, no `libc`, no `cfg(target_os)`. *(source)* |
| Which arithmetic is the same everywhere? | `+ - * /` and `sqrt` are [exact by specification](https://doc.rust-lang.org/std/primitive.f32.html). Rust fuses multiply-add only through an explicit `mul_add`, which the source never calls. The RNG (`rng.rs`) is integer-only. |
| Which is not? | **24 calls** to `exp`, `ln`, `log2`, `tanh` and `powi`: five each in `transformer.rs`, `net.rs`, `program.rs` and `school.rs`, one each in `model.rs`, `mechanisms.rs`, `life.rs` and `curious.rs` (nine more calls are `sqrt`, which is exact; there is no `powf`). The Rust docs call their precision [non-deterministic](https://doc.rust-lang.org/std/primitive.f32.html): it varies by platform and by Rust version. `tanh` maps to the C library's `tanhf` on Unix and Windows. |
| Do any sit on the honesty guards' decision path? | Yes. `ln` is in the routing scores (`school.rs` lines 247, 250, 280, 298, 332). `log2` is in the program-believability test, which compares the lessons' bits against the search's bits (`program.rs` lines 503, 505, 506, 862, 863). `ln` also sets the word weights used to match questions (`model.rs:162`, `mechanisms.rs:111`, `life.rs:176`, `curious.rs:101`). A last-bit difference could flip a near-tie. This is rare, and it is why the WRONG gates run on every platform and not only a byte comparison. |
| Do the save files depend on byte order or word size? | No. `Model::save` and `School::save` write text, and the source has no `to_*_bytes` calls. All hashes are explicit `u64` arithmetic reduced with `% buckets as u64`. *(source)* |
| What is Linux-only today? | Four measurement functions: `peak_memory_kb` (`main.rs`), `mem_kb` (`scale.rs`), `cpu_seconds` (`compare.rs`), and the RAPL path (`ledger.rs`). Elsewhere they fail quietly: energy becomes n/a and peak memory prints `0 KB`. |
| Is zero-dependency still possible? | Yes. The pure-Rust [`libm` crate](https://docs.rs/libm/latest/libm/) would remove the math differences but breaks the zero-dependency rule. The alternative is a small in-repo implementation built only from the exact operations, as [other deterministic-physics projects do](https://rapier.rs/docs/user_guides/rust/determinism/). |

Corrections to earlier assumptions, so they are not repeated:

- Linux's own bits are not permanent. Because precision is tied to the Rust version, a toolchain upgrade alone can shift a last
  bit. Every output records the compiler and OS.
- Fused multiply-add is not a risk here. The math library is.
- A calls-with-literal-argument case matters: `school.rs:250` has `3.0f32.ln()`. The compiler may fold such a call using the math
  library of the machine that *builds* the program, not the one that runs it (suspected, not verified). The fingerprint tests it.
- `windows-2025` is not a pin: it moved to Visual Studio 2026 in June 2026
  ([changelog](https://github.blog/changelog/2026-05-14-github-actions-upcoming-image-migrations)). No label is a true pin, because
  runner images update underneath it. Recording versions in the output is the real fix.

### 2.1 Findings from the read-only source review (2026-10-24)

**Paths: low risk.** Paths are relative (`runs/baby.school`) or built as `format!("{dir}/file")`. On Windows `temp_dir()` yields
backslashes, so tests build mixed-separator paths, which Windows accepts (not run). `memory.rs:127` calls `.to_str().unwrap()` in
a test, which would panic on a non-UTF-8 temp path. The CI scorecard step uses `tee`, `grep` and `test`, so Windows rows set
`shell: bash`.

**Hash-map ordering: two latent dependencies.** Rust randomizes hash-map iteration order per run. Most of the code handles this
(`selector.rs` sorts its keys, `known_words` sorts, `discover.rs` uses a `BTreeMap`, other loops only sum integers). Two places
depend on it:

1. `quantity.rs` `fit_ridge_dual` adds into an f64 matrix while looping over a `HashMap` of features. The last bits may differ
   between runs; after rounding to f32 this should almost never show.
2. `Quantity::terms()` → `learn_from_terms` → `learn_constant` pushes constants into a 6-slot first-in-first-out list in hash
   order, and the list is saved in that order. If a rule yields more than six distinct integers, which ones survive could depend
   on the hash seed. The scorecard's saved library is empty, so it never reaches this.

Tested: eight runs of `baby school` gave byte-identical saved files, and six runs of `baby scale --units 40` were identical once
timing and memory fields were removed. So the existing batteries do not trigger either. This is evidence, not proof, and it means
a golden mismatch on another platform could be a false alarm from hash order. **Each golden is therefore run at least three times
per row before comparing.** Changing to ordered maps is a logic change and is not part of this proposal.

**Casts and word size: low risk.** The narrowing casts are mostly float-to-integer (saturating, identical everywhere) or small
indexes. One width-dependent spot: `school.rs:336` uses `size_of::<Cell>()` in the memory ledger, which affects memory and
estimated-energy accounting if the struct layout differs, not any answer.

*Limits:* the review covered `.keys()`, `.values()`, `.drain()` and loops over the named hash collections, not every `.iter()`
over each of the 148 hash-collection uses, and `school.rs` was not read in full.

## 3. Phase 1 as built (PR #1)

No existing source file changes. Three files:

| File | What it does |
|---|---|
| `baby/examples/mathprint.rs` | The **math fingerprint**. Evaluates `ln`, `log2`, `exp`, `tanh` and `powi` over the ranges the code reaches (about 60 million evaluations) and prints a 64-bit hash of all result bits, plus one hash per 262,144 inputs to show where a difference starts. Ranges: word weights `ln((1+heard)/(1+df))` with heard and df each up to 5,000 (real-text runs teach thousands of lessons); routing gain up to 200,000 counts; routing prior with up to 10,000 rows and 3,200 lessons per unit; `log2` of 1 to 2,000,000 (the search budget is 1,000,000); softmax `exp` down to underflow; `tanh` over -12 to 12; loss `ln` of probabilities; and the optimizer's 0.9^t and 0.999^t. Runs under 2 seconds. It also prints each literal-argument call twice, as the compiler may fold it and forced at run time. "Identical" means identical on those inputs only; a run larger than these ranges is not covered. `mathprint dump NAME FROM COUNT` lists the actual values in a block, and `mathprint audit` measures accuracy (section 3.2). |
| `baby/examples/mathprint.expected` | The reference, generated on Linux x86_64, glibc 2.39, rustc 1.98.1. Lines starting with `#` are ignored when comparing. |
| `.github/workflows/ci.yml` | The existing workflow, extended into a matrix. Actions pinned by commit, `permissions: contents: read`, no secrets. Each row records `rustc -Vv`, the OS and whether `/sys/class/powercap` exists, then runs build, tests, the scorecard WRONG gate (unchanged), and the fingerprint comparison. A fingerprint difference produces a warning and a job summary and does not fail the job; a failed test or a WRONG above 0.0% still fails it. |

Rows: `ubuntu-24.04` (reference), `ubuntu-24.04-arm`, `x86_64-unknown-linux-musl` on `ubuntu-24.04` (same CPU, different C
library), `macos-15` (Apple silicon), `windows-2022`.

Checked locally before the PR: the fingerprint was identical across three runs and matched the stored reference, the scorecard
step logic passed, `cargo test --release --offline` passed (146 tests), and the workflow parsed as valid YAML.

### 3.1 First results (PR #1, CI run of 2026-10-24)

**Every gate passed on all five rows.** 146 tests passed on each (Windows and macOS included), the scorecard WRONG gate held, and
the scorecard output (17 lines) was identical to the reference on all five. All rows used rustc 1.99.1. The reference was built
with 1.98.1, and the reference row matched it exactly, so the toolchain bump moved nothing.

**The math fingerprint differs on four of five rows.** `ok` means identical to the Linux x86_64 reference, `DIFF` means different bits.

| Row | Word weights, routing, `log2` (decision path) | `exp` | `tanh` | Loss `ln` | `powi` |
|---|---|---|---|---|---|
| Linux x86_64 glibc (reference) | reference | reference | reference | reference | reference |
| Linux aarch64 glibc | ok | ok | DIFF | ok | ok |
| Linux x86_64 musl | ok | ok | DIFF | ok | ok |
| macOS arm64 | DIFF | DIFF | DIFF | DIFF | ok |
| Windows x86_64 | DIFF | DIFF | DIFF | DIFF | DIFF |

How to read it:

- **A green check does not mean bit-identical math.** The fingerprint step only warns; the differences appear in each job's
  summary and annotations. Any README or website text must say "CI-tested on GitHub's runners", never "supported" or
  "identical".
- On macOS and Windows every block of inputs differs on the decision-path functions. **How many inputs differ and by how many
  bits is measured by the accuracy audit (section 3.2), whose other-platform results are pending.**
- The identical scorecards are evidence only for the tidy curriculum. The exam and the adaptive streams (Phase 2 goldens) are the
  real test of whether the near-tie flip can happen. They are compared on Windows and macOS first.
- `tanh` lives in the old network, not on the decision path. aarch64 and musl differ from glibc x86_64 only there.
- **Folding, partly confirmed:** on musl the compiler-folded `tanh(0.5)` (`3eec9a9f`, the build machine's glibc) differs from
  the run-time value (`3eec9a9e`). Every other literal-argument call, including `ln(3)`, gave the same bits folded and at run time
  on all five rows.
- `/sys/class/powercap` exists but is empty on the x86_64 runners, so that log line shows the directory, not readable RAPL. A
  better check is `/sys/class/powercap/intel-rapl:0`.
- Not covered by CI: other hardware and OS versions, bare-metal energy, and real-world sandboxes. The runners are VMs running the
  real operating systems, not emulation.

### 3.2 The accuracy audit (`mathprint audit`)

Compares every result with the f64 result of the same function rounded to f32. The f64 value is accurate to far below half an f32
step, so the rounded value is the correctly rounded f32 result except for inputs within about one part in a billion of a rounding
boundary. It needs no stored reference: it says how often, and by how many steps, a platform is off. The CI matrix runs it on every
row and puts the table in the job summary (reported, never failing).

**Reference platform (Linux x86_64, glibc 2.39, rustc 1.98.1), measured 2026-10-24:**

| Section | Inputs | Correctly rounded | One step off | More than one step off | Worst |
|---|---|---|---|---|---|
| word weights `ln` | 25,010,001 | 99.14% | 215,705 | 0 | 1 step |
| routing gain `ln` | 200,001 | 99.98% | 32 | 0 | 1 step |
| routing prior `ln` | 32,013,201 | 99.97% | 9,681 | 0 | 1 step |
| `log2` | 2,000,000 | 99.99% | 177 | 0 | 1 step |
| `exp` | 92,161 | 99.94% | 57 | 0 | 1 step |
| `tanh` | 24,577 | 95.45% | 1,102 | 16 | 2 steps |
| loss `ln` | 100,000 | 99.28% | 722 | 0 | 1 step |
| `powi` 0.9^t | 100,000 | 99.14% | 42 | 821 | 46 steps |
| `powi` 0.999^t | 100,000 | 2.92% | 1,386 | 95,698 | 12,119 steps |

All exactly representable results came out exact on the reference: `log2(2^k) = k` for all 25 values of k, `ln(1) = 0`,
`exp(0) = 1`, `tanh(0) = 0`. Ties built on those are safe there.

What this says:

- **Even the reference library is not correctly rounded.** It is within one step for `ln`, `log2` and `exp`, and off by one on
  roughly 0.9% of word-weight inputs. A different library that is also within one step can therefore disagree with it by up to two.
  Differences of this size are not a bug in either platform; they are what "unspecified precision" means.
- `powi` is by design not correctly rounded (it multiplies repeatedly), and its error in 0.999^t grows with the exponent. The
  optimizer uses `1 - 0.999^t`: at huge exponents the value is tiny and that rounds to 1, but at moderate exponents the error can
  survive the subtraction. It sits in the old network's training (Adam), not on the decision path, and its effect on any result
  is not measured.
- Whether a one-step difference can change an answer depends on a comparison landing within one step of a tie. The audit's
  exact-case line checks the most likely tie sources. The exam and the Phase 2 goldens test the rest.

## 4. Later phases (not approved)

**Phase 2.** `platform.rs`: the four measurement functions behind `cfg`. Linux code is moved unchanged; elsewhere they return
"unavailable" and memory prints `n/a`, with no `unsafe`. Then goldens: the scorecard table rows, `exam run --out` predictions and
`adapt-run --out` replies, never the timings, peak memory, joules or meter warnings. Compare only after three runs per row.
`golden/**` is marked as binary or LF in `.gitattributes` so Windows checkout cannot look like a platform difference. Each golden
header records toolchain, OS, CPU and build host. Also one test-only change: the Python pipe test's skip check should skip when
`python3` cannot actually run (on Windows it may be a Store stub that starts but fails; a guess, unverified). Revert condition for
`platform.rs`: Linux output must stay byte-identical to the goldens and all tests must pass.

**Phase 3.** More rows: FreeBSD, NetBSD, OpenBSD and DragonFly via
[cross-platform-actions](https://api.github.com/repos/cross-platform-actions/action/releases/latest) (v1.6.0), i686, s390x and
riscv64 under QEMU, Windows arm64 (`windows-11-arm`), macOS Intel (`macos-15-intel`). A scheduled wrong-teacher stream (about 20 to
30 minutes, because a byte comparison on that stream once caught a change that the clean streams and the regression battery
missed), skipped on emulated rows. GitHub may disable scheduled workflows on inactive public repositories after about 60 days
(not verified), so check that it still fires.

**Phase 4.** The published results table, the README section and the website text, using the levels in section 6.

## 5. Decisions, and what is conditional

| Question | Decision |
|---|---|
| Order of work | Measure first, change later. Phase 1 changes no existing source. |
| WASI | **Conditional.** A WASI build carries its own math code, so it may be bit-identical across hosts, which would give a reproducible reference build with no logic change. It enters only if the fingerprint shows disagreement. Its goldens will probably differ from glibc Linux's (it links wasi-libc, which I believe derives from musl), so it needs its own golden set, validated by agreement across hosts (and a second engine if cheap), not by matching Linux. If it ever became the reference, the bars would have to be re-earned on it. That is a version decision for Daws. `scale --sweep` re-launches the program, `adapt-drive` spawns processes, and the Python test spawns `python3`, so the WASI row would run a subset (that WASI blocks process spawning is not confirmed). |
| Deterministic in-repo math | **Conditional** on the fingerprint disagreeing on a decision-path input. **That trigger is now met on macOS arm64 and Windows** (section 3.1), but no answer has changed yet, so the decision waits for the accuracy audit and the Phase 2 goldens. The smallest version replaces the `ln` and `log2` calls on the decision path. It changes Linux's numbers too, so it is a new, labeled version with the bars re-earned, not a port. |
| Real peak-memory measurement on non-Linux systems | Not proposed. It needs small `unsafe` blocks, and the units differ ([bytes on macOS, kilobytes on Linux and FreeBSD](https://developer.apple.com/library/ios/documentation/system/conceptual/manpages_iphoneos/man2/getrusage.2.html)). FreeBSD is the cheapest, as its unit matches Linux. |
| Can a platform port change a recorded bar? | No. |
| Hash-order dependencies | Logged for Daws; not fixed here. |

## 6. Linux and BSD in particular

Neither is one platform.

### Linux: four independent axes

| Axis | Finding | Plan |
|---|---|---|
| **CPU** | Rust [tier 1](https://doc.rust-lang.org/rustc/platform-support.html): x86_64, aarch64, i686. Tier 2 with host tools: riscv64gc, powerpc64le, powerpc64, s390x, loongarch64. | Native rows for x86_64 and aarch64. riscv64 and s390x emulated (tests and behavior only; emulated timing and energy mean nothing). |
| **Byte order, word size** | s390x and ppc64 are big-endian; save files are text, so it should not matter. 32-bit risk is low (section 2.1). | s390x covers big-endian, i686 covers 32-bit. |
| **C library** | glibc (Debian, Fedora, Arch, NixOS) versus musl (Alpine). musl targets are tier 2 with host tools. | The musl row. A static musl build may also work as one binary for any distro; not tested here. |
| **Environment** | RAPL generally exists only on bare metal, so VMs and containers do not expose it, and in containers `/proc/stat` usually shows the whole host (search summaries; no primary source found). | Joules apply only where RAPL opens. Otherwise the meter is off and the report says why. |

GitHub's runners are VMs, so **CI cannot test the energy code**; it covers behavior only. The joule path is verified by hand on
bare-metal machines. Distro differences are small: `/proc` and `/sys` are kernel interfaces. RAPL also needs the `chmod` Daws runs
by hand, because the [2020 power side-channel fix](https://ubuntu.com/security/CVE-2020-8694) made `energy_uj` root-only.

### BSD: three levels of Rust support

| System | Rust's support | How people build | Plan |
|---|---|---|---|
| **FreeBSD** | [Tier 2 with host tools](https://doc.rust-lang.org/rustc/platform-support.html) (x86_64, aarch64) | Packages, rustup | CI row; promise "builds and passes" as tested. |
| **NetBSD** | Tier 2 with host tools for x86_64 only; other CPUs tier 3 | [pkgsrc `lang/rust`](https://cdn.netbsd.org/pub/pkgsrc/current/pkgsrc/lang/rust/index.html) | CI row for x86_64. |
| **OpenBSD** | Tier 3 everywhere (no official prebuilt standard library) | `pkg_add rust`, [per the handbook](https://www.openbsdhandbook.com/ports/) | CI row using the distro's Rust. Zero dependencies and an offline build suit it. |
| **DragonFly** | Tier 3 | Unclear | Attempt a row if a Rust toolchain installs; otherwise best effort. |

FreeBSD and NetBSD do not mount `/proc` by default ([FreeBSD's `linprocfs`](https://man.freebsd.org/linprocfs(4)) is a
compatibility layer at a different path), so treat `/proc` as absent on every BSD. Peak memory reads n/a there after Phase 2.

### Support levels for the documentation

Each promise is tied to Rust's own tier. **Level A means: the gates pass, and the bytes are identical or the difference is shown.**

| Level | Meaning | Platforms |
|---|---|---|
| **A: tested on every push** | Tests, scorecard WRONG 0.0%, fingerprint, and (from Phase 2) golden diff | Linux x86_64 glibc, Linux aarch64, Linux musl, macOS arm64, Windows x64, FreeBSD x86_64 |
| **B: tested, but emulated or on a distro toolchain** | Same checks, may be slower or flakier | s390x, riscv64, i686, NetBSD, OpenBSD, Windows arm64, macOS Intel |
| **C: expected to work, unverified** | No CI | DragonFly, other CPUs, Android, WSL, containers |

Joules are **Linux with RAPL only**, at every level, and stay "measured with RAPL on this hardware." macOS `powermetrics`
[needs root](https://hazyresearch.stanford.edu/intelligence-per-watt/benchmarking/platform-support/), and Windows'
[energy metering interface](https://learn.microsoft.com/en-us/windows/win32/api/emi) depends on the hardware's drivers. The
counted-operations estimate must not be quoted as joules, because it was 34 to 116 times off.

## 7. Not verified

- Whether the compiler folds literal-argument calls with the build machine's library for functions other than `tanh` (it does for `tanh` on musl; no other case showed it).
- The exact math code path for `wasm32-wasip1`, whether hosts or engines agree, and whether WASI blocks process spawning.
- Whether the Python pipe test ran or was skipped on Windows (all 146 tests passed there; the log does not say which). The Windows scorecard matched, which is some evidence that CRLF is not a problem.
- How many of the 24 calls reach the scorecard or exam path (the fingerprint and goldens will show).
- Whether the BSD VM runs are fast enough for the full exam, and the QEMU setup for s390x and riscv64.
- OpenBSD and NetBSD package versions against this project's minimum Rust.
- That RAPL is absent in VMs and containers, and `/proc/stat` is host-wide in containers (secondary sources only).
- Mixed-separator paths on Windows (not run), and the 32-bit casts.
- The scheduled-workflow expiry rule.

## 8. What approval covers

- **Phase 1 (built, PR #1):** the files in section 3, merged by a normal merge commit once the results section is in.
- **Next, as a separate PR:** the accuracy audit in `mathprint` (section 3.1).
- **Phases 2 to 4:** separate approvals, each as its own commit, after the first results table exists.
