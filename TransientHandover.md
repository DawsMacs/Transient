# Transient: session handover

Rewritten 2026-10-21 at the end of a very long session (two days of work), for a fresh session with Daws. **Read this first.** It is meant to be enough to continue without re-deriving anything.
Order of reading: this file, then `TransientVision.md` (Daws's own words: never rewrite them; the progress log at its end is a dated summary of everything below), then the section of `baby/README.md` you need
(it is long, newest section first; the map is in section 5). Nothing here is aspirational unless marked PLAN or OPEN.

---

## 1. What Transient is (Daws's framing, keep it)

- **Mission:** powerful AI on anyone's own hardware, winning *differently* from cloud models rather than out-scaling them, so capable AI is not the property of a few providers with massive infrastructure.
  It may turn out not to be a model at all but a *system of intelligence*. Pursued **whether or not it works**. Made public and **kept open on purpose** (AGPL, Vision Entry 6): Daws does not want it hoarded or closed.
- **Vocabulary Daws coined** (directions, not exact definitions): **Adaptive Learning (AL)** = learning from interaction and experience, internals change in real time, and *the process of learning itself adapts*;
  **Maximization** = most intelligence in a given computational space; **UMA** (Ultragenerative Maximized Architecture) = the target architecture (a *bare UMA* is the tiny start, a *learned UMA* has accumulated learning).
- **Hunch he goes by (his words: not a verified claim):** an "iceberg": the industry scaled and refined the 2017 transformer, so low-end hardware is underestimated by a cliff. It may not even be tokens.
- **Directives:** Rust-first (not Rust-only); absolute freedom to the assistant under the stated goals (**he has now explicitly handed me the lead on direction: "go ahead with your plan", "lead, but still consider it"**);
  frontier-class intelligence on typical consumer hardware, light on RAM/memory; think in **joules**; "scale in efficiency, not size"; learning so light that nearly anyone can do it.
- **How Daws wants cautions phrased:** pessimism is welcome *as input* (it says what to measure) but never as a brake. Frame risks as terrain / design constraints / checkpoints; no kill criteria; no claims that success is certain. He values honest reporting of mistakes.

## 2. Where the work stands (2026-10-21)

Everything is in `~/Transient/baby/` (pure Rust, **zero dependencies**, builds offline, **87 tests pass**, ~10k lines). Git repo, `origin` = `git@github.com:DawsMacs/Transient.git` (**public**, Daws's global account `DawsMacs`, SSH),
branch `main`, **CI green** (`.github/workflows/ci.yml`: build, tests, and the scorecard's WRONG column must be 0.0 %). Licence `AGPL-3.0-or-later` (SPDX headers, `REUSE.toml`, `LICENSE`, `LICENSES/`), DCO not CLA (`CONTRIBUTING.md`).
All work from this session is committed and pushed (last code commit: "Rules that change regime: careful mode, checks for rules and selections").

**The baby** (`baby school` / `baby ask`) is a unified system:

1. Taught **units of lessons** (teacher groups them, never says how to learn them), or finds the units itself (`baby discover`).
2. For each unit it chooses how to learn it **from facts it holds back**: *remember* (voting memory), *rule* (exact, now **robust** ridge fit), *select* (bigger/smaller), *program* (searched expression, now with **div/mod, a learned library, digits, competing explanations**), *network* (rarely).
3. Questions are routed by a sparse, incremental table; every answer says how it knew, every refusal says why.
4. It keeps learning after school (surprise gate, pen, `reorganise`/`sleep`, `--compact`), and **now**: asks questions (`baby wonder`), checks its own explanations (`baby wonder --check`), measures the noise in its own lessons and sets its own reliability bar, and has an opt-in **careful** mode.

### What was added this session (each has a section in `baby/README.md`; the honest stories, mistakes and limits are there)

| capability | headline number |
|---|---|
| **Scale test** (`baby scale`, `scale.rs`) | clean worlds 20 -> 600 units / 26k lessons / 10k vocabulary: learning one fact flat at ~0.5 ms, teaching 26k lessons 2.65 s and 56 MB, reorganise 3.5 s, 100 % right, **0 wrong**, all traps refused. Fixed on the way: incremental weights + sparse routing, folds that cannot peek through word-bag twins, stratified merge folds, set-wise merging, network only where it can matter |
| **Digits as numbers** | a digit string (<= 4 digits) is a number; answers in the question's number style; signs/decimals/fractions/percents refused everywhere (CLI entry and teaching included: a bluff "-3 plus 5 = 8" was found and fixed) |
| **Label noise** | at 2 % and 5 % wrong lessons: **0 % wrong**, 97-100 % of unseen questions right at 2 %. Voting memory, majority-per-input before building programs, robust rule fit (peer-median outliers, protects a genuine second rule), rule-vs-memory cross-check, splits must improve things, merges at the reliability bar, routing smoothing V/100 |
| **Adaptive reliability bar** | the baby estimates its own noise (9.6 % where the world has 9.7 %) and sets `bar = clamp(0.98 - 2.5 x noise, 0.8, 0.9)`: at 10 % noise 98 % of unseen questions answered instead of 42 %; clean worlds unchanged. Floor 0.8 comes from a sweep (a select right 72 % of the time on held-back facts did answer wrongly) |
| **Mechanism invention** (`baby invent`, `invent.rs`) | all 16 hidden numeric kinds worked out every time; 7 of 9 stage-2 kinds (66/72 over 8 seeds) using constants and programs it learned earlier; taught in the wrong order 8/72 (so it is learned vocabulary); **0 false inventions, 0 wrong**; 127/128 and 118/128 hidden kinds at 2 %/5 % noise |
| **Asking** (`baby wonder`, `baby learn --wondered`) | programs keep competing smallest explanations (`alts`) and answer only where they agree; digit sum: **one question took it from 88.9 % to 100 %, random numbers needed three**; it asks nothing about random codes, primes, xor |
| **Effort / review** | no fold searches when the whole-data program search finds nothing (battery 23.5 s -> 14 s, identical results); settled programs reviewed by one cheap ambiguity search when the library changes (stage-2 61 -> 66/72); a unit is never explained by vocabulary it contributed itself |
| **Careful mode** (`--careful`) | **the hole:** a rule exact on everything taught that changes regime outside it (sum capped at 30, sum wrapping at 40, ...) is answered confidently wrongly: **53 wrong answers per seed**. Careful answers only inside the range taught *or checked*; `check_myself` (gaps, floor, swapped order, 2x edge, corner) wins the coverage back: after 2 rounds **0 wrong and 99.2 % of genuine extrapolation kept** |

Older headline numbers still hold (tidy 13-unit curriculum `kinds.rs`): scorecard right 100 / 100 / 95.1 / 100 %, 0 % wrong; answering 8,977 -> 57 operations per question (159x fewer, ~85x less *estimated* energy);
exact rule fit 392x fewer operations than 800-pass SGD; teaching 792 lessons 0.34 s (was 3.8 s: networks are no longer trained for plain tables); discovery finds 9 units from 792 unlabelled lessons.

## 3. How to verify the state in two minutes

```
cd ~/Transient
cargo test --release --offline          # 87 passed (about 12 s)
cargo build --release --offline
./target/release/baby school            # teaches 13 units (~0.4 s); the honest scorecard (WRONG must be 0.0 % on every row)
./target/release/baby ask "how far apart are two and nine"      # seven, with how it knew
./target/release/baby ask "three minus five"                    # ?  (negative result: must refuse)
./target/release/baby ask "what is -3 plus 5"                   # ?  (unreadable number: must refuse)
./target/release/baby scale --sweep 20,60,200                   # ~1 min; every WRONG column 0.00
./target/release/baby invent --seeds 4                          # ~1 min; hidden 64/64, 0 false inventions, 0 WRONG (deceptive kinds reported apart)
```

If `wrong` is ever above 0.0 % on the scorecard (outside the deceptive kinds, which are answered permissively by design), or a refusal battery in the tests starts answering, **stop**: a regression in honesty is the failure that matters most here.
`runs/` is gitignored and regenerable (`baby school`, `baby discover --out ...`); `runs/*.txt` hold experiment outputs.

## 4. Commands (`baby <command>`; usage at the top of `main.rs`)

`school [--careful]`, `ask "..." [--careful]`, `learn "<q>" <a> [--unit NAME] [--wondered]`, `wonder [--check] [--limit N]`, `sleep [--compact]`, `discover`, `grade`, `cost`, `storage`, `effort`, `teach`, and the experiments:
`scale [--units N --entities N --noise F --digits --typos F --no-reorg | --sweep 20,60,200,600]`, `invent [--seed N | --seeds K] [--noise F] [--bar F] [--stage2-first] [--active | --check | --careful [--rounds N --limit N]]`.
Global: `--bar F` forces the reliability bar (measurement only). Debug env vars that exist: `SCALE_WRONG`, `SCALE_REFUSED`, `INVENT_WRONG`, `ACTIVE_DEBUG`, `ACTIVE_ONLY`, `CHECK_DEBUG`, `CHECK_ONLY`, `CAREFUL_DEBUG`, `CAREFUL_ONLY` (each prints the cases behind a number; they are how every bug this session was found).

## 5. Code map (all under `baby/src/`)

| file | what it is |
|---|---|
| `school.rs` | the unified baby: `School` (units, sparse incremental `Routing`, `bar`, `careful`), `ask_with`, `learn`/`learn_anywhere`/`learn_wondered`, `wonder`/`wonder_about`, `check_myself`, `reconsider_open`/`review_programs`, `beyond_reach`, `reorganise` (split must improve; merges set-wise and settled from merge evidence), `compact`, save/load (**SCHOOL2** text, optional `library`, `bar`, `careful` lines, `alt` lines). Tests are the best spec |
| `mechanisms.rs` | the mechanisms, `Weights` (incremental, carries the `Library`), `Remembered` (glue, content-word order, **voting** recall, `exact`, `operand_hull`), CV (`cv_correct`, `cv_correct_strata`, word-bag groups), `evidence_thrifty`, `choose_mechanism`, **`reliable()` / `bar_for_noise` / `estimate_noise`** (thread-local bar), `grow_library` |
| `program.rs` | expression search (`+ - x max min apart div mod half`, constants, <= 7 pieces, work budget 1M), **`Library`** (constants from learned rules; functions with `origin`, contribute-once), `fit_trusted_with` (competing `alts`, `is_believable`: bits in the lessons vs bits of searching), `ambiguity`, majority-per-input |
| `quantity.rs` | number line, **digits**, `fit_ridge` and **`fit_ridge_robust`** |
| `discover.rs` | unit discovery (still quadratic pair loops), `can_merge_many` (stratified) |
| `scale.rs` | the scale-test generator and measurements (units, vocabulary, noise, digits, traps) |
| `invent.rs` | the invention battery: Base / Hidden / Stage2 / **Deceptive** / Unexplainable kinds; `run`, `run_active`, `run_check`, `run_careful` |
| `kinds.rs`, `memory.rs`, `store.rs`, `ledger.rs`, `net.rs`, `model.rs`, `data.rs`, `know.rs`, `life.rs`, `curious.rs`, `rules.rs`, `rng.rs`, `main.rs` | as before: the tidy curriculum, taught facts, the energy ledger, the older network and experiments, the CLI |

Docs: `README.md` (public technical overview, banner in `assets/`), `docs/Direction.md` (strategy, hardware terrain, language choice: the old long README), `baby/README.md` (the experiment log, newest first: Rules that change regime; Tuning its own bar; Adapting its effort...; It asks...; Noise, finished; Can the baby invent...;
Does learning stay light...; then the older ones), `LEARNING.md` (research notes), `TransientVision.md`, `CONTRIBUTING.md`. The last `## Next` list at the bottom of `baby/README.md` is partly stale: **this file's section 8 is the authoritative to-do**.

## 6. Guards that must stay (each one was learned from a failure; do not "simplify" them away)

- **Behavioural uniqueness / believability:** a built program is trusted only where its smallest fits agree (`alts`), and only if the lessons carry more bits than the search spent (`is_believable`). Search work budget is checked *inside* the loops.
- **Reliability bar** adaptive, floor 0.8, 0.9 when clean. A mechanism below it must not answer (28 wrong answers at 0.7 on the clean battery).
- **Memory:** votes among matching lessons (a tie refuses), content words compared as a *sequence with repeats*, number words never "glue"; lesson identity = words in order.
- **Library:** only *settled* programs are vocabulary (a guess once became a building block), a unit is never explained by its own contributions, a unit contributes a behaviour once, `reconsider_open`/`review_programs` when it changes.
- **Cross-validation:** facts that share a bag of words in one fold; merge parts stratified.
- **Splits must make things better** (both sides under a real mechanism); merges use the reliability bar.
- **Unreadable numbers** (sign, decimal, slash, comma, percent) are refused at `ask`, at the CLI entry and when teaching.
- **Never ask for an answer it cannot read back** (> 4 digits).
- **Report WRONG apart from refused, and deceptive kinds apart from the 0-wrong headline.** Add must-refuse questions with every capability (the same ambiguity family bit **four** times: apart/minus/ten-n, then order in memory).

## 7. Decisions that are Daws's (do not decide these for him; raise them)

1. **Should `--careful` be the default?** (new, the most important one). Permissive default = the documented showcase ("reaches numbers it was never taught"), but a rule that changes regime outside what it was taught is then confidently wrong (53 wrong/seed in the new deceptive kinds; no data in hand can reveal it).
   Careful = never confidently wrong there, loses half the extrapolation until checks win it back (99.2 % after 2 rounds with a teacher). I left it **off**. His call.
2. **The RAPL permission** for real joule measurements: one `sudo chmod a+r /sys/class/powercap/intel-rapl:0/energy_uj`. **Never run it yourself**; give him the command. (`perf` counters are blocked, `perf_event_paranoid=4`.)
3. **Rent a GPU** for architecture experiments, and what data may be uploaded: asked earlier, never answered.
4. **What "frontier" means concretely** for this project (benchmarks? chat quality like a cloud assistant?) and **what the name "Transient" means**: I deliberately did not guess.
5. **GitHub UI only he can do:** upload `assets/social-preview.png` as the social preview; set repository topics; (optionally) update the repo description. The licence is `AGPL-3.0-or-later`; he can say if he prefers `-only` (reasoning in Vision Entry 6).
6. **Git workflow:** he authorised commits and pushes to `origin main` for this work and said "go ahead with your plan" throughout; I have pushed after each stretch. Still ask before force-pushing, adding other remotes, or changing repo settings.

## 8. Options for the next session (my recommendation first; he asked to be given options, and has said "go ahead with your plan")

**Recommended order:**

1. **Settle decision 7.1 with him in one message** (careful default or not); if yes, flip `careful` to default-on, make the harnesses report beyond-range with checks, update the READMEs.
2. **Bridge to real data (the biggest open scientific question; I have suggested it since the start).** Everything above is synthetic and tidy (made-up words, numbers <= 30, a handful of phrasings). Take a small real factoid corpus and simple word problems and see which of units, mechanisms, discovery, refusal and the noise estimate survive.
   Be ready for the first version to fail and learn from how. Nothing else I could do is as informative.
3. **The controlled comparison (Vision Entry 2, still owed):** a matched mini transformer vs the baby at three or more sizes on quality at equal joules, retention after new learning, adaptation speed, joules per answer, and how quality moves with a small scale-up (his "tiny nudge, huge gain" prediction). This is where the architecture claim actually gets tested.

**Smaller, concrete OPEN items (each is in the READMEs with its evidence):**
- **Library capacity / pinning:** 8 function slots vs ~14 program units; round 3 of the careful experiment still loses some coverage (4,306 of 4,710). Evict by use (a function that existing programs contain as a subtree should be pinned) or raise capacity and measure search cost.
- **Asking about facts and phrasings** (today only numeric programs, rules and selections); choosing questions by expected information rather than by disagreement; a wrong answer from a *real* teacher (every experiment uses a simulated, always-right teacher).
- **Merged rule families** are self-checked through one operator word only.
- **Discovery at scale** (`baby discover` has quadratic pair loops, 3.6 s on 792 lessons); the pen with thousands of waiting lessons (`process_pen` re-clusters on every lesson); Zipf-shaped vocabulary; entity/word ambiguity; misspellings that land on other words (typo questions are all *refused* in the scale test, so robustness there is untested).
- **Adapting more knobs from measurement** (folds, outlier allowance, search budget): the bar is the only one adapted. A negative result is recorded: partial progress does not separate doomed from deep program searches, so the search budget cannot be cut that way.
- **Compact lossless storage format**, incremental exact updates, the real energy meter (needs decision 7.2).
- **The UMA growth path** (a unit that starts tiny and grows capacity only when it saturates) and **mechanisms of a different *kind*** (not an expression): not started.
- **Rust proof points** from `docs/Direction.md` (Rust Qwen3-0.6B forward pass vs llama.cpp logits; Burn vs PyTorch): not started.
- Number line is still *taught* by counting lessons (digits parse by themselves, words do not).

## 9. Doubts and honest caveats (state these when reporting; they are not boilerplate)

- **Everything is synthetic and tidy.** "0 % wrong" holds on in-style questions plus hand-written must-refuse batteries; it is a spot check, not a proof, and I have found a new way for it to fail every time I built a harder test (regime changes, data gaps, circular library entries). Expect more.
- **The language is mine.** Mechanisms, operations, terminals, the 7-piece limit, thresholds (0.8/0.9, one bit, one million, 10 % outliers, V/100) are my choices. The baby builds and selects inside them; "invention" here means expressions and vocabulary, not a new kind of learner.
- **Evidence for some constants is thin:** the 0.8 floor rests on one 72 % failure and six clean seeds at 0.75; V/100 on three tried values.
- **Data gaps:** a lone explanation that fits everything taught can still be wrong at a number nobody taught (digit sum explained as `first mod 9` with 8 and 9 missing). Checks and review catch it in the battery; nothing structural *prevents* it.
- **Noise:** only label noise at 2 %-15 %; where most of one fact's own phrasings are wrong nothing can know (about 0.4 % of facts at 5 %). Not tested: ambiguous entities, correlated errors, real misspellings.
- **Experiments use a simulated, always-right teacher.** The self-check "never refuted anything" once the review pass existed: it is insurance, not a measured gain (except in the constructed unit test and the careful-mode experiment).
- **Energy figures are estimates** (counted operations x published 45 nm costs; trust the ratios). Real measurements so far: wall time and peak memory on one laptop core.
- **Speed:** `reorganise` 3.5 s at 600 units (1.35 s before the robust fit); the careful/check/active experiments are slow (see section 10).
- Nothing yet shows behaviour on millions of real sentences, or that an architecture like this can reach "frontier" quality. Treat that as the open frontier, not a settled conclusion either way.

## 10. How to work on this (earned the hard way)

1. **Measure before building, and print the cases behind every number.** Almost every real finding this session came from a debug listing (`SCALE_WRONG`, `INVENT_WRONG`, library add/evict logs), not from reasoning. Several of my first explanations were wrong; the READMEs say so. Record mistakes as they happen (Daws values this).
2. **Ask adversarial questions after every capability** and keep must-refuse batteries in the tests; build the harder kind of test *before* the feature (the deceptive kinds, the unexplainable kinds, the clean adversarial battery against low bars).
3. **Long experiments:** the tool times out at ~590 s. Run them in the background to a file (`nohup sh -c '...; echo FINISHED >> file' &`), **delete the file before a rerun** (I once read a stale `FINISHED`), then wait with an until-loop. Rough costs: `invent --seeds 8` ~2.5 min; `invent --check --seeds 6` ~12 min; `invent --active --seeds 6` ~15 min; `invent --careful --seeds 3 --rounds 3 --limit 4` ~25 min; `scale --sweep 20,60,200,600` ~5 min.
4. **Never `pkill -f "<pattern>"` where the pattern appears in your own command line** (it killed the shell twice); use `pkill -x baby` or kill by pid. Foreground `sleep` is blocked; use an until-loop.
5. **Do not compile or run heavy jobs while timing something**; the machine has 8 cores, so parallel runs are fine for correctness but not for timing.
6. **Tool pitfalls:** piping a CLI into `head` closes the pipe and can kill it before it saves; scripted find-and-replace with a non-unique anchor corrupted a file once (use unique anchors, build and test straight after); the Write tool needs the file read first; `then_some` evaluates eagerly (use `then(|| ...)` when the value can panic).
7. **Keep `TransientVision.md` Daws's voice.** Append dated entries as he gives them; assistant context goes in separate labelled sections; update the progress log (it is maintained) at the end of each stretch.
8. **Privacy / repo is public:** never commit anything from `.local/` (git-ignored) or details of the sibling project (section 11). Do not run system-setting commands yourself (the RAPL `chmod`): give Daws the command.
9. Commit with the `Co-Authored-By` trailer the harness specifies; push to `origin main`, then check `gh run list` for CI (wait for the run whose `headSha` equals `git rev-parse HEAD`).

## 11. Sibling project

A separate, private project of Daws's lives in `~/Hiraketa` (a chat client with a small offline model) with its own `CLAUDE.md` and handover. It is not part of this repository and its notes are not published here
(the former section 8 is kept locally in `.local/hiraketa-notes.md`, git-ignored). Read it only if Daws asks you to work on that project; its offline-mode work is its own private research.

## 12. First moves for the next session

1. Read this file, then `TransientVision.md` (end: progress log), then the top three sections of `baby/README.md` (Rules that change regime; Tuning its own bar; Adapting its effort...).
2. Run the verification in section 3.
3. Tell Daws briefly where things stand, ask decision 7.1 (careful default), and propose the order in section 8 (real-data bridge next). Then do it: he has repeatedly said "go ahead with your plan" and given absolute freedom, so default to doing, with honest reporting.
4. Keep the habits in section 10, and update this file at the end of the stretch.
