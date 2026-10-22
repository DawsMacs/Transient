# Transient: session handover

Rewritten 2026-10-21 at the end of a very long session (two days of work), for a fresh session with Daws; **updated later the same day** after the second stretch (careful default, labelled guesses, the record of its own guessing; section 2b and the changed sections 3, 7, 8). **Read this first.** It is meant to be enough to continue without re-deriving anything.
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

Everything is in `~/Transient/baby/` (pure Rust, **zero dependencies**, builds offline, **102 tests pass**, ~11k lines). Git repo, `origin` = `git@github.com:DawsMacs/Transient.git` (**public**, Daws's global account `DawsMacs`, SSH),
branch `main`, **CI green** (`.github/workflows/ci.yml`: build, tests, and the scorecard's WRONG column must be 0.0 %). Licence `AGPL-3.0-only` (changed from `-or-later` on 2026-10-21, Vision Entry 8) (SPDX headers, `REUSE.toml`, `LICENSE`, `LICENSES/`), DCO not CLA (`CONTRIBUTING.md`).
All work from this session is committed and pushed (last code commit: "It keeps and states a record of its own guessing; reach is a staircase, not a box").

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

### 2b. The second stretch (2026-10-21, after Daws answered the open decisions): careful by default, guessing like a scientist

Daws's framing (Vision Entry 7, his words): a scientist guesses, forms hypotheses, theories, then laws; being wrong and updating *is* learning; **"never silently wrong"** replaces "never wrong"; real-time updating is the advantage over LLMs. What was built, in the order of the plan he approved:

1. **Careful is the default** (`--permissive` opts out; `--careful` is a no-op; a saved school writes `careful 0` only when permissive, a file with no line loads careful). Capability batteries (`invent` default run, many tests) run permissive explicitly on purpose.
2. **Labelled guesses.** `Reply.guess: Option<Guess {answer, stretch, rate, tried}>` beyond the range taught or checked; **never counted as `answer`**; the CLI prints `probably X`; the scorecard reports guesses apart (right/wrong of the refused). Must-refuse questions get no guess (a test).
3. **A miss ends the guessing out there.** Units record `exceptions` (lessons the rule does not explain; reach is read from the explained ones only) and are `refuted` if an exception lies outside what the explained lessons span (`settle_reach`, called wherever a unit is fitted). A refuted unit states facts only inside its explained range and does not even guess beyond; what it was told wins over the refuted rule. `learn` now keeps a confirmed guess (it was discarded as "nothing to learn"). Measured (`invent --corrections`): before, 303 of 305 other wrong guesses stayed wrong and 2 became WRONG facts; after, 0 and 0 (all refused; it stops, it does not learn the new regime from one point).
4. **A record of its own guessing** (`guess_record`, saved as a `guessed` line): hits/misses per distance bucket; a guess carries `rate = (h+1)/(n+2)` and `tried`; withheld where it has been wrong more often than right. `invent --calibrate`: stated rate below the realised one in every bucket (under-confident, never over), 8,884 facts, **0 wrong**.
5. **Reach is a staircase, not a box** (`Remembered::uncovered_by`): found because the calibration run stated 4 wrong facts of 9,309 (two confirmed edges vouched for the unchecked corner between them). Cost: scale never-taught 96.94 % to 94.90 % at 60 units (90.99 % at 200), scorecard never-taught 92.7 % to 90.2 %, careful recovery after 2 rounds genuine 4,674 to 4,668; WRONG 0.00 everywhere (clean, 2 %, 5 % noise).

New experiment commands: `baby invent --guesses | --corrections | --calibrate [--seeds K]`.
6. **The controlled comparison has begun (Vision Entry 2): `baby compare [--sizes 16,32,64] [--layers N] [--epochs N] [--adapt N | --scale 20,60,200]`** (`compare.rs`, `transformer.rs`: an attention transformer written from scratch, gradient-checked). Real joules measured with the energy counter. Headline (details, tables, caveats: first section of `baby/README.md`): the baby is never wrong where the transformer is wrong 0.3-40 % of the time on unseen questions, extrapolates (labelled guess 100 % right vs the transformer 19-28 % right, the rest confidently wrong, at every scale), adds 20 new facts with zero forgetting for ~63 mJ vs a forgetful 120-145 mJ, a 10 %-replay 190-250 mJ that still loses 3-8 points, or a full replay 1.3-1.6 J; teaching from scratch 10-50x cheaper (measured). **Against the simple story:** the energy advantage shrinks with scale (45x, 14x, 10x at 20, 60, 200 units) and the transformer overtakes the baby on raw unseen accuracy (99.7 vs 91.0 % at 200 units; the baby refuses 9 %). The transformer is untuned plain Rust; no sweep yet.
7. **First contact with real text (SVAMP, 1,000 word problems; `baby bridge --svamp FILE`; the file is in scratch space, not in the repo; re-download from `arkilpatel/SVAMP`, MIT, 339,970 bytes, only with Daws's approval).** First run: learned nothing, 0 wrong. Built **partial programs** (a relation fitting >= 80 % of >= 24 lessons, MDL-checked; answers are always labelled guesses; stated rate = held-back score x measured routing precision). Two-number problems: told the kind, guesses 67-90 % right; routed from words alone only 49-62 % reach the right unit (bag of words tops out at 57-65 % on this corpus, measured offline); 0 right and **0 wrong as facts**, 20-35 % of questions get a wrong *labelled* guess; the whole file (3-4 numbers per problem) still learns nothing. Full numbers, mistakes and limits: the first section of `baby/README.md`. `CAREFUL_TALLY=1 baby invent --careful ...` prints each deceptive kind's final state and tally.

## 3. How to verify the state in two minutes

```
cd ~/Transient
cargo test --release --offline          # 102 passed (about 16 s)
cargo build --release --offline
./target/release/baby school            # teaches 13 units (~0.4 s); the honest scorecard (WRONG must be 0.0 % on every row; beyond the taught range now reads 0 % right as fact, 100 % refused, 100 % of them a correct guess)
./target/release/baby school --permissive   # (optional) the old showcase; the default is careful
./target/release/baby ask "how far apart are two and nine"      # seven, with how it knew
./target/release/baby ask "what is seven times one"             # probably seven (a labelled guess: multiplication was taught on 0..5)
./target/release/baby ask "three minus five"                    # ?  (negative result: must refuse)
./target/release/baby ask "what is -3 plus 5"                   # ?  (unreadable number: must refuse)
./target/release/baby scale --sweep 20,60,200                   # ~1 min; every WRONG column 0.00
./target/release/baby invent --seeds 4                          # ~1 min; hidden 64/64, 0 false inventions, 0 WRONG (deceptive kinds reported apart)
```

If `wrong` is ever above 0.0 % on the scorecard (outside the deceptive kinds, which are answered permissively by design), or a refusal battery in the tests starts answering, **stop**: a regression in honesty is the failure that matters most here.
`runs/` is gitignored and regenerable (`baby school`, `baby discover --out ...`); `runs/*.txt` hold experiment outputs.

## 4. Commands (`baby <command>`; usage at the top of `main.rs`)

`school [--permissive]`, `ask "..." [--permissive]` (careful is the default; `--careful` is a no-op), `learn "<q>" <a> [--unit NAME] [--wondered]`, `wonder [--check] [--limit N]`, `sleep [--compact]`, `discover`, `grade`, `cost`, `storage`, `effort`, `teach`, and the experiments:
`scale [--units N --entities N --noise F --digits --typos F --no-reorg | --sweep 20,60,200,600]`, `invent [--seed N | --seeds K] [--noise F] [--bar F] [--stage2-first] [--active | --check | --careful [--rounds N --limit N] | --guesses | --corrections | --calibrate]`; add `CALIB_DEBUG` to list wrong facts.
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

## 7. Decisions that are Daws's (answered 2026-10-21; recorded in Vision Entry 7; do not reopen without him)

1. **Careful is the default**: decided (and it must not make the baby timid: labelled guesses, trial and error, see 2b).
2. **RAPL**: Daws made `/sys/class/powercap/intel-rapl:0/energy_uj` readable himself (it reads; it resets on reboot). Real joule measurements are now possible and **not yet used**: wire them into `baby effort`/`cost` next time energy is discussed. Never run the `chmod` yourself.
3. **GPU**: Daws will **rent one** for the controlled comparison (small transformer vs the baby) and will say when it is set up; the iMac (macOS, AMD 4 GB, 40 GB RAM) was set aside. Nothing may be uploaded that is private; code and synthetic data are fine. Write the comparison model in pure Rust (or Burn) so operations and joules count the same way.
4. **"Frontier"** = an ambition, not a destined goal: general-purpose intelligence you would trust in work and school at the range of trillion-parameter models, on ordinary hardware. **The measure that matters is capability per joule as the machine measures it**; benchmarks and chat quality are trend-based, optional, later. **The name "Transient" stays a mystery** ("lore"): never guess in docs.
5. **GitHub UI** (social preview, topics): done by Daws. **The repo was renamed to `DawsMacs/Transient`** by another agent (commit `ecae36a`, `origin` updated); "TransientAI" is now the creator's name. Old-name mentions in Vision entries are historical on purpose.
6. **Licence** is `AGPL-3.0-only` (Vision Entry 8, 2026-10-21; was `-or-later`). To let future AGPL versions in again, name a proxy (AGPLv3 §14) rather than restoring "or later"; revisit when outside contributors arrive. **Git**: commits and pushes to `origin main` authorised; be careful, precise, plan, ask before anything risky (force-push, new remotes, repo settings).

## 8. Options for the next session (my recommendation first)

Daws approved the plan: careful default, guess gear, update after a miss, adaptive boldness, **then the real-data bridge**. The first four are done (2b). Remaining, in order:

1. **Bridge to real data (the biggest open scientific question).** Everything above is synthetic and tidy (made-up words, numbers <= 30, a handful of phrasings). Take a small real factoid corpus and simple word problems and see which of units, mechanisms, discovery, refusal, the noise estimate, guesses and their stated rates survive. Be ready for the first version to fail and learn from how. Decide with Daws which corpus (public, small, licence-clean) before downloading anything.
2. **The controlled comparison (Vision Entry 2, still owed; GPU to be rented by Daws):** a matched mini transformer vs the baby at three or more sizes on quality at equal joules (measured: RAPL is readable now), retention after new learning, adaptation speed (guess, one correction, repeat mistakes: the new `--corrections` experiment is the baby's side of it), joules per answer, and how quality moves with a small scale-up.
3. **The comparison, next:** (a) a fair best-effort transformer (a hyperparameter sweep, digit-aware numbers, regularisation) so the 'untuned' criticism cannot stand; (b) 600 units and beyond to see where the energy lines cross; (c) a Rust-optimised transformer (SIMD) to separate algorithm from implementation; (d) beyond the synthetic curriculum. **This is where Daws's rented GPU would help** (sweeps and larger sizes); it is not needed for anything done so far.
4. **Real text, next barriers in order** (all measured in the first section of `baby/README.md`): (a) which numbers matter when a problem has three or four (the tolerant search currently refuses a unit where more than 20 % of the lessons hold more than two numbers); (b) routing a question to the right kind from compositional cues, not word counts (bag of words caps at ~60 % here); (c) discovery on real text (159 units named after story words); (d) a second corpus, a factoid one (`question TAB answer` loader is ready), so the finding is not about SVAMP alone.
5. **Make "update after a miss" learn the new regime, not only stop** (one correction currently turns the whole unit's guessing off; coverage returns only with more checks): let a refuting lesson count as must-explain in the program search so `min(a+b, 30)` can be found from a few points; unit- or kind-specific guess records.

**Smaller, concrete OPEN items (each is in the READMEs with its evidence):**
- Coverage cost of the staircase reach at scale (200 units: never-taught 91 %); the reach check is not counted in the operation ledger; compacted schools (`sleep --compact`) read their reach from the lessons that remain (untested under careful; likely too narrow).
- A real teacher is sometimes wrong (every experiment uses a simulated, always-right one): a wrong correction now switches a unit's guessing off.
- The `asked` count of the careful recovery experiment (498 over 2 rounds) disagrees with the older table (about 230 over 3 rounds); not explained.
- **Library capacity / pinning:** 8 function slots vs ~14 program units. **Asking about facts and phrasings** (today only numeric programs, rules and selections); choosing questions by expected information. **Merged rule families** are self-checked through one operator word only.
- **Discovery at scale** (quadratic pair loops), the pen with thousands of waiting lessons, Zipf-shaped vocabulary, entity/word ambiguity, misspellings that land on other words.
- **Adapting more knobs from measurement** (folds, outlier allowance, search budget); a compact lossless storage format; **wiring the real energy meter into the ledger**.
- **The UMA growth path** and **mechanisms of a different *kind***: not started. **Rust proof points** from `docs/Direction.md`: not started. The number line is still *taught* by counting lessons.

## 9. Doubts and honest caveats (state these when reporting; they are not boilerplate)

- **Everything is synthetic and tidy.** "0 % wrong" holds on in-style questions plus hand-written must-refuse batteries; it is a spot check, not a proof, and I have found a new way for it to fail every time I built a harder test (regime changes, data gaps, circular library entries). Expect more.
- **The language is mine.** Mechanisms, operations, terminals, the 7-piece limit, thresholds (0.8/0.9, one bit, one million, 10 % outliers, V/100) are my choices. The baby builds and selects inside them; "invention" here means expressions and vocabulary, not a new kind of learner.
- **Evidence for some constants is thin:** the 0.8 floor rests on one 72 % failure and six clean seeds at 0.75; V/100 on three tried values.
- **Data gaps:** a lone explanation that fits everything taught can still be wrong at a number nobody taught (digit sum explained as `first mod 9` with 8 and 9 missing). Checks and review catch it in the battery; nothing structural *prevents* it.
- **Noise:** only label noise at 2 %-15 %; where most of one fact's own phrasings are wrong nothing can know (about 0.4 % of facts at 5 %). Not tested: ambiguous entities, correlated errors, real misspellings.
- **Experiments use a simulated, always-right teacher.** The self-check "never refuted anything" once the review pass existed: it is insurance, not a measured gain (except in the constructed unit test and the careful-mode experiment).
- **Energy figures are estimates** (counted operations x published 45 nm costs; trust the ratios). Real measurements so far: wall time and peak memory on one laptop core.
- **Speed:** `reorganise` 3.5 s at 600 units (1.35 s before the robust fit); the careful/check/active experiments are slow (see section 10).
- **Guess statistics are from the simulated world.** The stated hit rate is a smoothed count per distance bucket, shared across all units, and was shown honest (under-confident) only in the invention battery; the 4 wrong facts found by the calibration run (a box reach vouching for an unchecked corner) show that a harder test finds a new hole each time. "0 wrong facts" is 0 of 8,884 in that run, not a proof.
- **Careful costs coverage** (never-taught right 96.94 % to 94.90 % at 60 units, 91 % at 200): the price of not vouching for what was not checked; labelled guesses recover some of it as *guesses*, not facts.
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
9. **Gate every commit on the tests passing** (`cargo test ... | grep -q "test result: ok"` in the `if`, not just a pipe: a pipe into `grep` returns 0 even when tests fail, and I once pushed a failing test that way). Build the failing test first when fixing a hole; run long experiments in the background and wait with `Monitor` (an until-loop on a `FINISHED` marker); a monitor expires after 10 minutes, re-arm it.
10. Commit with the `Co-Authored-By` trailer the harness specifies; push to `origin main`, then check `gh run list` for CI (wait for the run whose `headSha` equals `git rev-parse HEAD`).

## 11. Sibling project

A separate, private project of Daws's lives in `~/Hiraketa` (a chat client with a small offline model) with its own `CLAUDE.md` and handover. It is not part of this repository and its notes are not published here
(the former section 8 is kept locally in `.local/hiraketa-notes.md`, git-ignored). Read it only if Daws asks you to work on that project; its offline-mode work is its own private research.

## 12. First moves for the next session

1. Read this file, then `TransientVision.md` (end: progress log), then the top three sections of `baby/README.md` (Rules that change regime; Tuning its own bar; Adapting its effort...).
2. Run the verification in section 3.
3. Tell Daws briefly where things stand, ask decision 7.1 (careful default), and propose the order in section 8 (real-data bridge next). Then do it: he has repeatedly said "go ahead with your plan" and given absolute freedom, so default to doing, with honest reporting.
4. Keep the habits in section 10, and update this file at the end of the stretch.
