# Transient: session handover

Written 2026-10-20 at the end of a long session, for a fresh session with Daws. Read this first. It is meant to be enough to continue without
re-deriving anything. Order of reading: this file, then `TransientVision.md` (Daws's own words, do not rewrite them), then the section of
`baby/README.md` you need (it is long; the map is in section 4 below). Nothing here is aspirational unless marked PLAN.

---

## 1. What Transient is (Daws's framing, keep it)

- **Mission:** powerful AI on anyone's own hardware, winning *differently* from cloud models rather than out-scaling them, so capable AI is not the property of a few providers
  with massive infrastructure. It may turn out not to be a model at all but a *system of intelligence*. Pursued **whether or not it works**.
- **Vocabulary Daws coined** (directions, not exact definitions): **Adaptive Learning (AL)** = learning from interaction and experience, internals change in real time, and *the process of learning itself adapts*
  (not only what is learned); **Maximization** = most intelligence in a given computational space; **UMA** (Ultragenerative Maximized Architecture) = the target architecture, a *bare UMA* is the tiny starting
  unit and a *learned UMA* has accumulated learning (analogue of a trained GPT), scaled up *from* the tiny one.
- **Hunch he goes by (not a verified claim, he says so):** an "iceberg": the industry scaled and refined the 2017 transformer and invested in hardware, so low-end hardware is underestimated by a cliff.
  Also: it may not even be tokens.
- **Directives:** Rust-first (not Rust-only, the idea is the centre, embrace better if it measures better); absolute freedom to the assistant under the stated goals; frontier-class intelligence on typical
  consumer hardware without overwhelming RAM/memory; thinking in **joules** not just parameters; "scale in efficiency, not size"; learning should be so light that nearly anyone can do it.
- **How Daws wants cautions phrased:** pessimism is fine and useful *as input* (it tells us what to measure) but must never slow or stop the work. Frame risks as terrain/design constraints/checkpoints that redirect,
  never as reasons to stop or as kill criteria. Do not claim success is certain either. He values honest reporting of mistakes, and the README records dead ends on purpose.

## 2. Where the work stands (2026-10-20)

All code is in `~/Transient/baby/` (pure Rust, **zero dependencies**, builds offline, **70 tests pass**). It is a Cargo workspace; `runs/` and `target/` are gitignored.
**Git (since 2026-10-20):** `~/Transient` is a git repo, `origin` = `git@github.com:DawsMacs/TransientAI.git` (public, Daws's global account `DawsMacs`, SSH). Daws authorised commits and pushes to that repo for this work. Licence: `AGPL-3.0-or-later` (SPDX headers in `baby/src/*.rs`, `REUSE.toml`, `LICENSE`, `LICENSES/`). Contributions use a DCO, not a CLA (`CONTRIBUTING.md`). The repo is public, so never commit anything from `.local/`, and keep the sibling project's details out.

The headline artefact is **"the baby"** (`baby school` / `baby ask`), a unified system:

1. It is **taught units of lessons** (the teacher groups them, never says how to learn them) or **finds the units itself from an unlabelled heap** (`baby discover`).
2. For each unit it **works out how to learn it, from facts it holds back** (cross-validation it runs itself), choosing among five mechanisms: *remember* (write down, answer by similarity), *rule* (exact ridge fit of a
   per-word linear combination of quantities), *select* (bigger/smaller of two numbers), *program* (enumerative search for a small expression, built not chosen), *network* (small bag-of-words net, rarely needed). Ties go to the cheaper.
3. Questions are routed to a unit by a **precomputed table** of how telling each word's *shape* is for each unit (numbers count as "a number"); the unit's mechanism answers or declines.
4. **Every answer says how it knew; every refusal says why.** Refuses: unmet words, unrecognised kinds, memorise-only units for facts it wasn't told, mechanism declines (negative results, off-the-line numbers, missing operands).
5. It **keeps learning after school** (`baby learn`): lessons it can already work out cost nothing and are not stored (surprise gate); consistent lessons refit in place; unrecognised lessons wait in a **pen**
   and open a unit by themselves once six lessons of five facts gather; `baby sleep` merges units that follow one rule and splits units that stopped following one; `--compact` forgets lessons that are explained.

**Headline measurements** (all on a tidy synthetic curriculum, `kinds.rs`; see caveats in section 6):

| claim | number |
|---|---|
| scorecard (taught / reworded / never taught / beyond range), school and self-organised baby | right 100 / 100 / 95.1 / 100 %, **0 % wrong**, rest refused; identical on seeds 1-5 |
| answering effort | 8,977 -> 85 operations per question (106x fewer), ~56x less *estimated* energy (45 nm figures), same answers |
| fitting a rule | exact solve vs 800-pass SGD: 392x fewer operations, 56x faster, same rule |
| teaching 792 lessons / 13 units from nothing | 3.8 s, 11.9 MB peak, one laptop core |
| a lesson it already understands / a new phrasing / a new kind (no unit named) | 0.18 ms (not stored) / 3 ms / 1 ms (unit opens at the 6th) |
| storage | school file 29 KB: 93 % lessons, 1.3 KB learned; `--compact` -> 49 % of the file, still answers all 792 taught lessons, 0 % wrong, rewordings 97.7 % |
| discovery from 792 unlabelled shuffled lessons | 9 units found (13 named by the teacher), scorecard identical, 3.7 s |

## 3. How to verify the state in two minutes

```
cd ~/Transient
cargo test --release --offline          # 70 passed (about 15 s)
cargo build --release --offline
./target/release/baby school            # teaches 13 units (~4 s); prints how it learned each + the honest scorecard (wrong must be 0.0%)
./target/release/baby ask "how far apart are two and nine"      # seven, with how it knew
./target/release/baby ask "three minus five"                    # ?  (negative result: must refuse)
./target/release/baby grade             # scorecard on the saved school
./target/release/baby cost              # time + memory of learning
./target/release/baby discover --out runs/discovered.school     # finds 9 units from unlabelled lessons (~4 s)
```

If `wrong` is ever above 0.0 % on the scorecard, or any of the refusal batteries in `school.rs` tests starts answering, stop: a regression in honesty is the failure that matters most here.
`runs/baby.school` is regenerable (`baby school`); `runs/baby.memory` holds facts the user taught with `baby teach` (delete it for a clean slate).

## 4. Code map (all under `baby/src/`)

| file | what it is |
|---|---|
| `school.rs` (~1,200 lines) | **the unified baby**: `School` (units, routing table, floor), `ask_with` (counted effort), `learn` / `learn_anywhere` (surprise gate, `refit_in_place`, pen), `process_pen`, `reorganise` (split then merge), `compact`, `storage`, `save`/`load` (**SCHOOL2** text format; networks in `<path>.unit<k>.model`). Tests are the best spec. |
| `mechanisms.rs` | the five mechanisms, `Weights` (df/spread), `Remembered` (glue words + inverted index), cross-validation (`cv_correct`, `cv_accuracy`), `evidence`/`evidence_thrifty`, `choose_mechanism`, `learn_items` |
| `discover.rs` | unit discovery from unlabelled lessons: anchor words, same-facts merge (ordered keys, symmetric), rule-coherence merge (per-group), `can_merge` |
| `program.rs` | expression search (`+ - x max min apart half`, constants 1 and 2, up to 7 pieces); `fit_unique` + `behaviours()` for ambiguity |
| `quantity.rs` | number line from counting lessons; per-word operator weights; `fit_ridge` (exact) and the old SGD `fit_penalized` |
| `kinds.rs` | the synthetic curriculum: 13 units (capital, color, sound, opposite, code = facts; sum, minus, after, double, half = rule; compare = select; times, apart = need a built expression) |
| `memory.rs`, `store.rs` | user-taught open-vocabulary facts (`baby teach`) and the weighted-Jaccard fact store |
| `ledger.rs` | counted operations/words moved, priced with Horowitz 45 nm figures; optional RAPL reader |
| `net.rs`, `model.rs`, `data.rs`, `know.rs`, `life.rs`, `curious.rs`, `rules.rs`, `rng.rs` | the original from-scratch network (readers: concat/bag/bigram/letters), the older experiments (`train`, `life`, `know`, `curious`, `rules`, `kinds`), and plumbing |
| `main.rs` | the CLI (`baby <command>`; usage at the top of the file) |

Docs: `baby/README.md` (64 KB, newest sections first: Try the baby, Learning is light, Reorganising, Storage/forgetting, the pen, Finding the units itself, A different way of computing, Less effort,
Learning after school, One baby: the school; then the 10-19 and 10-15 experiments in order). The **last `## Next` section at the bottom of that README is the authoritative to-do list**; the older
"Where this goes next" near line 370 is stale. `LEARNING.md` is the learning-ladder research (sources in it). `README.md` is the public, technical overview (banner in `assets/`); the former long-form README is `docs/Direction.md`. `TransientVision.md` is Daws's entries (1 to 5)
plus a progress log kept by the assistant.

## 5. The plan and what is next (PLAN, in order)

**Immediate (the scale test was built and run on 2026-10-20; see `baby/README.md`, "Does learning stay light when the world gets big?"):**

1. **Scale test, continued.** DONE for clean lessons up to 600 units / 26k lessons / 10k vocabulary (`baby scale --sweep 20,60,200,600`; learning one fact is flat at ~0.5 ms; teaching 26k lessons 2.65 s; WRONG 0). What it fixed: incremental
   weights + sparse routing, folds that cannot be peeked through (word-bag twins), stratified merge folds, set-wise merging, network only where it can matter, `RELIABLE` = 0.9 bar for any mechanism to answer. **OPEN, do next:**
   - **Label noise (2 % wrong lessons): honesty SOLVED (0 % WRONG at 60 and 200 units; voting memory denoises wrongly taught facts; memory respects content-word order and repeats; number words are never glue), coverage OPEN:** rule units fall below the 90 % bar and become memorise-only, so never-taught questions are refused (55-62 % right). Do robust rule fitting (drop lessons that disagree only if their operator word is otherwise consistent, so a polluting second rule is still caught by split).
   - Zipf-shaped vocabulary, entity/word ambiguity, misspellings that land on other words; `baby discover` at scale (still quadratic pair loops, 3.6 s on 792); the pen with thousands of waiting lessons (`process_pen` re-clusters on every lesson).
2. **Compact storage format** that loses nothing (binary, word table + ids, phrasings stored as templates plus facts); forgetting (`--compact`) then becomes a separate saving for what is truly redundant.
3. **Incremental and exact updates** (add one lesson to a fitted rule without refitting; sparse updates; small integers instead of floats) and the **real energy meter** (needs Daws's one-time
   `sudo chmod a+r /sys/class/powercap/intel-rapl:0/energy_uj`, never run it yourself; `perf` counters are blocked, `perf_event_paranoid=4`).
4. **Let it propose its own mechanisms** when nothing generalises (the units that stay "memorise only"); a richer space for built expressions (conditions, three numbers, division, composing earlier rules).
5. **Digits and number-like words** as numbers; **discover the number line without counting lessons** (it is currently taught: the counting lessons put zero to ten on the line).

**Still owed from Daws's earlier requests (do not lose these):**

- **The controlled comparison (Entry 2):** a mini transformer/LLM matched to the baby on parameters/bytes/learning energy/data, compared at **three or more sizes** each on quality at equal joules, retention after new learning,
  adaptation speed, joules per answer, and how quality moves with a small scale-up (his "tiny nudge, huge gain" prediction). Not started. This is where the architecture claim actually gets tested.
- **The UMA growth path:** a unit that starts tiny and grows capacity only when it saturates (function-preserving growth), "bare UMA to learned UMA". Not started.
- **Rust proof points from the README's language section:** a Rust Qwen3-0.6B forward pass matching llama.cpp logits; Burn vs PyTorch on a 10M model. Not started (the baby is all hand-written Rust; this is about the ecosystem question).
- **Bridge to real data.** Everything so far is a tidy synthetic curriculum. The most important open question is whether units, mechanisms, discovery and refusal survive *real* text (a small factoid corpus, simple word
  problems). Suggest doing this soon after the scale test; be ready for the first version to fail and learn from how.

## 6. Honest caveats (state these when reporting; they are not boilerplate)

- Everything is measured on **synthetic, regular lessons** (numbers 0-10, a handful of templates). "0 % wrong" is on questions in or near the taught style plus hand-written batteries of unanswerable questions; it is a spot check, not a proof.
- The **five mechanisms, the expression language, the number line (taught by counting lessons), the thresholds** (4 facts to anchor, 5 facts per unit, 95 % held-back, floors) are mine. The baby chooses among them and fits them; it does not invent a sixth.
- **Energy figures are estimates** (counted operations x published 45 nm costs; ratios are what to trust). Real measures so far: wall time and peak memory only.
- The older single-network results (`train`, `life`, `know`, `curious`) and the school are different systems; do not mix their numbers.
- Nothing yet shows how any of this behaves with millions of real sentences, or that an architecture like this can reach "frontier" quality. Treat that as the open frontier, not a settled conclusion either way.

## 7. Lessons about *how* to work on this (earned the hard way; the READMEs have the stories)

1. **Ask adversarial questions after every capability.** The same trap appeared **three times**: `|a-b|`, `a-b` and `ten-n` agree on every taught lesson, so merging units on "same facts" or "same expression" made the baby
   answer "three minus five" with "two". Ambiguity must be defined by *behaviour* (do the expressions differ somewhere on the grid), merges must hold **per group** (an overall average lets a big group swallow a small one),
   and order of operands must be kept when comparing facts. Tests pin all of this; keep adding batteries of must-refuse questions.
2. **A scorecard must report WRONG separately from refused.** Accuracy alone hides bluffing (the "code for five" -> code for "one" bluff was found by asking for every code, not by the scorecard).
3. **Lesson identity is the words in order** (`lesson_key`). A sorted-word key once made "five minus two" and "two minus five" the same lesson.
4. **Recalibrate the recognition floor after anything that changes the routing table** (merge, split, compact). `recalibrate_floor` exists; `rebuild_tables` deliberately does not call it.
5. **Record mistakes and rejected hypotheses in the README as they happen** (Daws values this). Several of my first explanations were wrong and the notes say so.
6. **Tool pitfalls I hit:** (a) piping a CLI into `head -2` closes the pipe and kills `train` before it saves; (b) a scripted find-and-replace whose anchor string occurs earlier in the file duplicated a third of `mechanisms.rs`
   (rebuilt from the intact tail): use unique anchors and rebuild/test straight after; (c) the Write tool requires reading a file this session before overwriting it; (d) the in-app `cargo test` takes ~20 s because of debug-free
   `--release` plus school training in tests, so run the whole suite sparingly and a single test by name.
7. **Keep `TransientVision.md` Daws's voice.** Append dated entries as he gives them; assistant context goes in separate labelled sections. Update the progress log at the end of each stretch of work.

## 8. Sibling project

A separate, private project of Daws's lives in `~/Hiraketa` with its own `CLAUDE.md` and handover. It is not part of this repository and its notes are not published here. The full former section 8 is kept locally in `.local/hiraketa-notes.md` (git-ignored). Read it only if Daws asks you to work on that project.

---

## 9. Pending actions that need Daws (do not do these yourself)

- The RAPL read permission (one `sudo chmod`) for real joule measurements.
- ~~Whether to `git init` / go public~~ Decided 2026-10-20: public on GitHub, AGPL-3.0-or-later, kept open on purpose. Still his to do in the GitHub UI: upload `assets/social-preview.png` as the social preview, set topics.
- Whether to rent a GPU for architecture experiments and what data may be uploaded (asked, not answered).
- What "frontier" should mean concretely for this project (benchmarks? chat quality like a cloud assistant?) and what the name "Transient" means (I deliberately did not guess).

## 10. First moves for the next session

1. Read this file, then `TransientVision.md`, then `baby/README.md` sections "Try the baby", "Learning is light" and the final "Next".
2. Run the two-minute verification in section 3.
3. Tell Daws briefly where things stand and that the scale test (plan item 1) is next; then start it. He has repeatedly said "go ahead with your plan" and given absolute freedom, so default to doing, with honest reporting.
4. Keep the three habits: adversarial batteries, wrong-vs-refused, and writing down mistakes.
