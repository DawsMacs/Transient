# Transient: rethinking how a system learns

Brainstorm and research notes, 2026-10-15. Not a plan, a map. Tags: **[SOURCED]** read this session (search summaries unless noted;
check the paper before relying on a number), **[MEMORY]** from my own recollection, unverified, **[IDEA]** a proposal of ours,
untested. Pessimism below is terrain: it tells us what to measure first and never whether to proceed.

## 1. The reframe

Mainstream practice runs one pipeline: pretrain on a huge corpus, post-train, freeze the weights, serve them. "Training" is an event
that happens once, in a data centre, for everyone. Transient does not have to inherit that. Learning can instead be a *continuous,
budgeted process with several mechanisms on different timescales*, running on the owner's own hardware, where the unit that matters
is capability gained per joule, per byte read and per token seen. The size of the data is one input to that; how learning is
implemented is the lever Daws is betting on.

## 2. Where the standard paradigm is wasteful (the opening)

- **Facts are learned very expensively.** Transformers store ~2 bits of knowledge per parameter, and only after a fact has been
  seen on the order of 1,000 times **[SOURCED, Allen-Zhu & Li]**. Writing one line to a store takes one step and zero gradients.
- **Updating weights forgets.** In a continual-learning test, full fine-tuning on new facts dropped NaturalQuestions F1 by 89% and
  LoRA by 71%; updating only a few memory slots dropped it 11% at the same new-knowledge level **[SOURCED, sparse memory
  finetuning, Meta/Berkeley]**.
- **Small models are usually fed more data instead of learning better.** Sub-2B model families are trained on 2-11 trillion tokens
  to compensate for size **[SOURCED, survey]**. That is the default approach to challenge.
- **Backprop is costly on small devices.** It stores activations, runs sequentially and is non-local, which is why researchers are
  scaling alternatives like predictive coding (a 2025 result trains 100+ layer predictive-coding nets) **[SOURCED]**, though not
  at language-model scale.
- **Frozen at deployment.** The cloud model cannot learn from you privately or cheaply per person.

## 3. Evidence that implementation, not data volume, is the ceiling

- **The nanoGPT speedrun** **[SOURCED]**: training GPT-2-small (124M) to the same validation loss went from 45 minutes on 8 H100s
  (llm.c baseline) to ~3 minutes (Jan 2025) to ~2 min 20 s (Oct 2025), and one early variant reached the same loss on 0.75B tokens
  instead of 10B. Same task, same model class: better optimiser (Muon), modernised architecture, better kernels and schedules. A
  >10x cut in both time and tokens from implementation alone. (Caveat: a tuned benchmark at small scale; how much transfers
  upward is a separate question.)
- **BabyLM** **[SOURCED]**: under a 100M-word budget, winners (LTG-BERT) beat models trained on trillions of words on the
  challenge's evaluations, and a data-recombination trick (mixing chunks from different contexts, repeated 40x) matched a 4B-word
  dataset from 100M words. Honest limits: the evaluations lean on grammar and linguistic tests, not world knowledge or chat, and
  most curriculum-ordering attempts were *unsuccessful*, so "teach it like a child" by reordering data is not a free win.
- **Data pruning** **[SOURCED, Sorscher et al.]**: with a good ranking metric, error can fall exponentially instead of by a power
  law in dataset size (shown on image classifiers; not established for language models). Quality filters like FineWeb-Edu and DCLM
  already show data selection matters for small models.
- **Distillation** **[SOURCED earlier, in private research notes not in this repository]**: a teacher's per-token probabilities carry far more information
  than hard labels; on-policy distillation reportedly reaches teacher-level results in a fraction of the steps of RL.

## 4. A ladder of learning timescales (the central proposal) [IDEA, informed by sources]

Instead of one weight-update mechanism, several, matched to how fast and how permanent each piece of learning should be.

| Timescale | Mechanism | Status in the literature |
|-----------|-----------|--------------------------|
| **Instant** (zero gradients) | Write to a store: facts, preferences, episodes, corrections; retrieve later | Plain engineering (offline knowledge dumps, a personal diary) |
| **Seconds-minutes** | In-context learning and fast weights: a small memory module updated while reading (Titans, test-time training layers) | Promising; critiques exist (a Titans re-implementation analysis; a 2026 paper whose title says test-time training with KV binding is secretly linear attention; I have read neither) **[SOURCED titles only]** |
| **Hours (overnight)** | Sparse consolidation: replay the day's experiences into a small set of parameters (memory-slot or adapter updates), with rehearsal of old behaviour | Sparse memory finetuning and self-distillation reduce forgetting **[SOURCED]**; sleep-style consolidation is a conceptual match, not an established result in LMs |
| **Self-directed** | The model writes its own training examples and update directives, rewarded by later performance (SEAL) | Early research, NeurIPS 2025 **[SOURCED]** |
| **Rare** | Base training: language, reasoning, judgement; possibly distilled from a teacher once, possibly shared | The standard pipeline, kept for what must live in weights |

Google's Nested Learning / HOPE work frames this same idea as a "continuum memory system": banks that update at different
frequencies, fast ones for the immediate, slow ones for stable knowledge **[SOURCED, secondary summaries]**.

Design principle that falls out: **put each kind of knowledge at the cheapest timescale that does the job.** Facts about the world
and about Daws go in stores; habits and style go in slow, sparse updates; only language and judgement need the expensive base.

## 5. Levers for learning efficiently on small hardware

1. **Objective:** distillation signals over hard labels; on-policy samples with judge or verifier feedback; self-generated data.
2. **Optimiser and schedule:** Muon-style optimisers and tuned schedules (the speedrun) **[SOURCED]**; low-precision training
   **[MEMORY]**.
3. **Data engineering:** quality filtering, recombination, pruning, deduplication, multiple epochs, synthetic data from a teacher.
4. **Sparse and local updates:** memory layers or adapters so each lesson touches few parameters, which also limits forgetting.
5. **Memory-cheap training:** zeroth-order methods (MeZO) estimate gradients from forward passes only, with the memory of
   inference; it trained a 30B model on one A100 80GB where backprop managed 2.7B **[SOURCED]**. It typically needs many more
   steps than backprop **[MEMORY]**, so it suits small, targeted updates. Evolution-strategy variants are being applied to LLM
   reasoning too **[SOURCED, title]**.
6. **Same engine for inference and learning:** if training only needs forward passes and sparse writes, the Rust inference
   runtime can also be the learner on the phone or laptop. A forward-only learner makes a very small deployable system.
7. **Architecture co-design:** an architecture whose knowledge lives in sparse tables (memory layers, Engram-style) is *also* one
   that is cheap to update sparsely. The inference design and the learning design should be chosen together.
8. **Local learning rules:** predictive coding, target propagation and similar methods avoid storing the whole graph and
   parallelise by layer; early and unproven for language **[SOURCED]**. A long-horizon watch item.

## 6. How to measure learning (so we can tell if an idea works)

- **Capability gained per training token, per joule, per byte of weights touched.**
- **One-shot score:** after a single exposure to a new fact or preference, can it be used correctly an hour or a week later?
- **Forgetting:** capability on a fixed held-out suite before and after learning (an existing small-model evaluation suite already does this).
- **Time to adapt on the target device:** what an overnight run on the laptop or S24 can actually accomplish.
- **Stability:** does repeated nightly learning drift her voice? (Use the 24-30 turn drift set.)

## 7. Cheap first experiments

1. **Store vs weights, head to head:** teach a ~0.6B model 100 new personal facts by (a) writing them to a store, (b) LoRA, (c) sparse
   updates; compare accuracy, steps and forgetting on the use-case suite. Likely the clearest early evidence for the ladder.
2. **Nightly consolidation on a small model:** a day's conversations plus a rehearsal slice of base-model outputs and router JSON, trained
   into an adapter at night; check for voice drift and regressions. Uses the existing harness.
3. **Forward-only adapter tuning (MeZO-style) on the laptop CPU:** how much of a preference or style update fits in inference memory?
4. **Reproduce a speedrun-style 124M training in Rust (Burn/CubeCL)** and see how close it gets to PyTorch; this tests both the
   Rust-for-research bet and the "implementation is the ceiling" claim.
5. **BabyLM-style data-efficiency test for two architectures** (e.g. transformer vs a recurrent or looped design) at fixed tokens.
6. **Self-authored training data (SEAL-style) at small scale:** can a 0.6B model write useful training examples for itself, with a
   verifier from the judge? Probably hard at this size; worth a bounded try.

## 7b. First results from the baby (2026-10-15)

`baby life` (see `baby/README.md`) ran the ladder on a 16k-parameter net: a fact store (instant rung) kept 98.5% of lessons at
1/900 the estimated energy of retraining; store plus overnight-style consolidation ("sleep") matched retraining's retention at about
1/5 of the energy; naive online learning forgot ~90%. Consolidation lost the store's rewording robustness (97% -> 56%), pointing at the
network's position-bound reader rather than at the learning rule. One implementation change (an SIMD-friendly dot product) made
answering 2.5x faster with identical results. Small, tidy world, 5 seeds, estimated joules: encouraging evidence for the ladder, not
proof.

## 8. Open questions

- Is a gradient step even the right primitive? A system that mostly *writes memory* and rarely changes weights may be the right
  shape, and then "training" is mostly curation, retrieval design and consolidation policy.
- Who trains the base? One distilled base, community-trained, or each device from a shared recipe?
- What is the right forgetting policy (some things should be forgotten)?
- How should learning be governed and made inspectable so an owner can see and undo what the system learned?
- Safety of self-modifying systems (poisoning through retrieved or conversational data): needs its own design.

## 9. Link to other work

A separate, private project (a small offline model with tools and an evaluation harness) is the baseline for the old train-then-freeze pipeline and the measuring instrument for section 7. Its notes are not part of this repository.

## Sources

- Sparse memory finetuning: https://arxiv.org/pdf/2510.15103
- Knowledge capacity, ~2 bits/parameter, ~1000 exposures: https://arxiv.org/pdf/2404.05405
- modded-nanogpt speedrun: https://github.com/KellerJordan/modded-nanogpt ; record history: https://www.greaterwrong.com/posts/j3gp8tebQiFJqzBgg/how-the-nanogpt-speedrun-wr-dropped-by-20-in-3-months
- BabyLM findings: https://arxiv.org/html/2504.08165v1
- Beyond neural scaling laws (data pruning): https://arxiv.org/pdf/2206.14486v5
- Small language model survey (2T-11T token budgets): https://arxiv.org/pdf/2409.15790
- Titans: https://arxiv.org/pdf/2501.00663 ; Titans re-implementation analysis: https://arxiv.org/pdf/2510.09551 ; TTT-KV binding paper: https://arxiv.org/pdf/2602.21204
- Nested Learning / HOPE (secondary): https://aipapersacademy.com/nested-learning-hope/
- SEAL: https://arxiv.org/html/2506.10943v2
- MeZO: https://arxiv.org/pdf/2305.17333 ; ES for LLM reasoning (title only): https://arxiv.org/pdf/2608.27351
- Predictive coding at depth: https://arxiv.org/pdf/2510.23323
- Self-distillation for continual learning: https://arxiv.org/pdf/2601.19897 ; on-policy distillation: https://thinkingmachines.ai/blog/on-policy-distillation/
