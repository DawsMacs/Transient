# TransientVision

A living record of why Transient exists and what Daws is betting on. Entries are dated and kept in his framing. Where I (the
assistant) add context, it is in a separate, labelled section so his reasoning stays his. **This is a hunch, not a verified claim,
and Transient pursues it regardless of how long it takes.**

Started 2026-10-19.

**Vocabulary Daws has coined** (directions, not exact definitions; terminology is allowed to shift):
- **Adaptive Learning**: learning that goes beyond conventional machine learning: continuous, from interaction, with the internals changing.
- **Maximization**: optimising a system for the most intelligence in a given computational space.
- **UMA (Ultragenerative Maximized Architecture)**: the architecture Transient is aiming at. A *bare* UMA is the tiny starting unit;
  a *learned* UMA has accumulated what it learned (the counterpart of a trained GPT, a usable LLM).

---

## Entry 1 (2026-10-19): The iceberg

### The bet

There is an iceberg under the question of how to run frontier intelligence on hardware. Ordinary, low-end hardware has probably been
underestimated, and not by a little: by an enormous cliff. If AI is built from scratch again, and the right implementation is found,
the path to an architecture that is far more efficient may be like a maze, with a real way through it that nobody has taken because
the industry has not been looking there.

Daws is explicit about what kind of belief this is: **a hunch, not a verified claim.** It rests on his personal judgement, including
non-technical factors, and not on what current research officially establishes. Researchers today mostly bet on smaller scaled
benefits. Transient goes by the hunch anyway, **however long it takes.**

### The reasoning (Daws's)

1. **History keeps shrinking computers.** In the 1960s and 70s you rented machines the size of furniture or rooms. Today a smartphone
   in a pocket is more powerful than what NASA flew in the Apollo missions. What looked like it required a building now fits in a hand.
2. **Hardware will keep improving, and that is good, but it is not the whole story.** There are mathematical ceilings and natural
   limits we have reached with current implementations. Something like vertical stacking may help, but it is not the kind of
   breakthrough the single-chip processor was.
3. **Software has probably been neglected relative to hardware.** Investment and attention have gone far more to hardware than to
   rethinking the software side.
4. **Today's models share one inherited design.** Most descend from the transformer architecture Google published in 2017, which
   OpenAI built ChatGPT on. It has evolved, but almost nothing big has been started from scratch lately; the work has been improving
   existing implementations.
5. **The industry went all-in on scale.** Compute exploded, driven by NVIDIA, with skyrocketing demand for GPUs and infrastructure to
   run AI at mass scale, and with the intensity and energy cost that comes with it. New hardware followed, such as NPUs. But the
   models were mostly scaled, trained and refined, not reinvented.
6. **Conclusion:** the neglected area is enormous. Rebuild AI from scratch, find the breakthrough implementation, and the efficiency
   gain could be the size of an iceberg, not an increment.

### Decision

Transient goes by this hunch. It does not need to win by out-scaling the current approach; it exists to look where the industry has
mostly not looked, and it will keep looking for as long as it takes.

---

## Context added by the assistant (labelled, so it can be checked and removed)

These are things that rhyme with the hunch. They do not prove it, and some point the other way; both kinds are listed.

**Evidence that points the same direction**
- **Software has already out-run hardware at times.** OpenAI's "AI and Efficiency" study found that the compute needed to train a
  classifier to AlexNet-level accuracy on ImageNet fell **44x between 2012 and 2019**, with algorithmic efficiency doubling about
  every 16 months. Moore's Law over the same stretch would have given about 11x
  ([source](https://openai.com/index/ai-and-efficiency/)). For tasks with heavy investment, algorithms gave more than hardware did.
- **A single optimiser-and-architecture rework cut a benchmark's cost by an order of magnitude.** The nanoGPT speedrun took GPT-2-small
  training from 45 minutes to about 2 min 20 s on the same hardware class, using about 10x fewer tokens in an early version
  ([source](https://github.com/KellerJordan/modded-nanogpt)). The "iceberg" at small scale, found by implementation work, not new chips.
- **Capability density keeps rising.** The size of model needed for a given quality has been halving roughly every 3.5 months
  ([source](https://alphaxiv.org/abs/2412.04315)). That is the same cliff seen from the model side.
- **What Transient's own baby measured.** In `baby/`, answering a question is ~14,000 multiply-adds, and **91% of the estimated
  energy is moving memory, not arithmetic**; one change to how a dot product is written made it 2.5x faster with identical results.
  A 7-billion-parameter model does roughly 500,000x more arithmetic per token and must pay DRAM prices for every weight. Estimated
  figures, tiny world, but the shape (energy lives in data movement, and implementation choices move it a lot) fits the hunch.
- **The transformer's inheritance is real.** Variants (mixture-of-experts, grouped-query attention, FlashAttention, rotary
  embeddings) are refinements of the 2017 design; alternatives such as state-space models and RWKV exist but have not displaced it
  at the frontier.

**Things that cut the other way (terrain, not reasons to stop)**
- Hardware and software have co-evolved: transformers won partly because they map very well onto GPUs. A new architecture has to be
  efficient on real chips, not just on paper, and may need new kernels.
- Many ideas that look excellent at small scale fade at large scale; the scaling curve is the test, not a single demo.
- The "iceberg" could be real yet narrower than hoped (for example, large for knowledge lookup and routing, smaller for open-ended
  reasoning). The hunch does not say which parts; finding that out is part of the work.

### What the hunch predicts (so it can be tested without ending it)

If there is an iceberg, then, roughly:
1. Some architecture reaches a given quality using **orders of magnitude less energy per token** than a dense transformer.
2. The gain comes mostly from **software and representation**, not from new silicon.
3. That architecture runs on **ordinary low-end hardware**, such as a phone or an old laptop.

Signals to watch rather than pass/fail gates: how far Transient's results sit from the current quality-per-joule frontier; whether
independent small designs keep finding large efficiency jumps; whether the jumps survive more scale. Each tells us where to dig next.

---

## Entry 2 (2026-10-19): A working unit, Adaptive Learning, and the UMA

### The shape of the solution

Probably not a heavy model but a **working unit**: something small that does real work and can grow. The current baby is not the
breakthrough, only a rough picture of the *variety* worth trying when you refuse the conventional route. Being in Rust and designed
efficiently, it can be rewarding in itself. Daws invites suggestions: if the assistant finds its own candidate, say so.

### Adaptive Learning (Daws's term)

The next big objective, a gamble: learning that surpasses machine learning as practised and how LLMs learn today.

- It **continuously learns and builds abstract representations**, as LLMs do, **without being fed a dataset**. It learns through
  interaction: autonomous browsing, different modalities (language first), discovering on its own instead of humans formulating the
  training. **Control is still needed in the early stages**, but it should look nothing like conventional training.
- It can be **efficient**: it grows meaningfully instead of absorbing redundancy. It learns rather than being dumped on.
- It may build a **solid foundation first** (math, physics). **The complexity of reality sets reliability**: math or grammar syntax
  is simple and can be reliable; something like literature has far more room to be interpreted differently depending on what the
  learner already knows and has grown into.
- **The internals adapt and change** instead of staying frozen, including in real time while running. That **blurs the line between
  a provider training a model and a user simply interacting with one**. It is closer to how humans learn.

### Maximization and the UMA

- **Maximization** is Daws's term for optimising a system or model to hold the most intelligence in a given computational space. It is
  a direction, not a perfect definition, and "intelligence" has subjective parts; both are accepted.
- The claim: if something is made *densely* intelligent, a tiny increase in scale could produce a dramatic increase in capability,
  proportionally, like nudging the accelerator and suddenly doing 120 mph. Like a singularity before the big bang: very small, very
  dense, then growth.
- The architecture is called a **UMA, Ultragenerative Maximized Architecture**. A bare UMA is the tiny original; a **learned UMA**
  has accumulated what it learned, like an already-trained GPT or a usable LLM. A learned UMA can be **scaled up** from the original
  tiny one for greater intelligence, depending on what it can learn.

### How to proceed (Daws's guidance)

- **Flexible, not dogma.** The hypothesis is a vision, not an absolute rule. It could end up a model, a system, or a working unit;
  the assistant has the freedom to explore.
- **Run several experiments with different architectures.** Because the units are tiny, running one is cheap, especially if the
  learning process needs less compute.
- **Controlled comparison at small scale.** Build or find a **mini LLM of roughly the same size and learning/data as a UMA** and
  compare them: how much computation each uses, how each works, how each generates. The differences should apply proportionally to
  larger versions if scaled, imperfectly, but it is a pretty good testing method at this scale.

---

## Context and suggestions added by the assistant (labelled; check, edit or delete freely)

### Where the ideas meet existing research

- **Learning from interaction instead of a dataset** is close to *continual / lifelong learning*, *curiosity- and surprise-driven
  learning*, and *open-ended learning*. Google's Titans update memory when a surprise signal is high, and the continuum-memory
  work updates different parts at different speeds (see `LEARNING.md`, sources there). Adaptive Learning goes further by also
  choosing *what to look at*.
- **"Internals change in real time"** is the territory of fast weights and test-time training, and of sparse updates that touch few
  parameters to avoid forgetting. The baby's `life` experiment already shows the energy side: skipping lessons it can already
  answer meant only a third to a half of the lessons needed learning at all (285 of 858 heard for the store method, 402 of 858 for
  sleep).
- **"Scale a tiny learned UMA up"** has a concrete technical cousin: *function-preserving growth* (the Net2Net idea of widening or
  deepening a network while keeping what it computes, from my recollection of Chen et al. 2015). It would let a bare UMA grow into
  a bigger learned UMA without starting over.
- **"Complexity of reality sets reliability"** maps onto calibration: the baby's confidence already tracks how well-defined a
  question is (unknown or unseen questions came out below 0.5). A reliability signal that varies by domain (high for arithmetic,
  lower for interpretive text) is measurable.

### The "tiny nudge, huge gain" prediction is testable and sharp

Standard scaling studies find *smooth* power-law improvement with size, so a steep response to a small scale-up would stand out
against them, which is what makes it a good bet to test. Related phenomena exist: sudden generalisation after long training in
small networks ("grokking", from my recollection of Power et al. 2022) and abilities that appear to switch on at a size (whether
those jumps are real or an artefact of the metric is debated). None of this confirms the UMA claim; it says where to look.

### Suggested candidate working units (the assistant's picks, in the order I would try them)

1. **The two-speed unit (episodic store + slow network + sleep).** The baby already is one: write while awake, consolidate while
   asleep. It reached retrain-level retention at about 1/5 of the energy. Fix its known weakness (the network reads by position) with
   a position-free reader and it becomes the minimal Adaptive Learning unit. *Experiment:* does consolidation keep rewording
   robustness; how many lessons per joule.
2. **The surprise-gated learner.** Learn only when prediction error is high; do nothing when the world matches expectation. This is
   "no redundancy" as a mechanism. *Measure:* capability gained per joule, and what fraction of experience was skipped.
3. **The growing unit.** Start tiny; add capacity (hidden units, embedding rows, new answer slots) only when the unit saturates,
   using function-preserving growth. This is the literal "bare UMA to learned UMA" path. *Experiment:* grown unit vs a unit built
   large from the start, at equal energy spent.
4. **The loop-with-memory unit.** A small block reused for a variable number of steps (more steps for harder inputs) plus a store
   to read and write. Parameters stay tiny while compute adapts per input (looped transformers and tiny recursive models point here).
   *Experiment:* accuracy vs steps vs joules on rule-like tasks the baby fails (sums).

Candidate (3) with (1) is the closest match to the vision as described; (2) is the cheapest to add to the existing code.

### How to run the controlled experiment Daws describes

Build a **matched mini transformer** and a UMA candidate and hold fixed what we can: parameter count, bytes of weights, and the
*energy the learning spent* (the ledger already counts it) as well as data seen. Compare, at **three or more sizes** each, not one:
quality at equal joules, retention after new learning, adaptation speed to new facts, joules per answer, and how quality changes
as each is scaled a little. Slopes across sizes are what extrapolate; a single size can mislead. Keep a log of every config so the
comparison cannot be accidentally tilted toward the idea we like.

### A definition we can start measuring

For Maximization, a first operational proxy (to be argued with): **capability per unit of resource**, where capability is a score on
a fixed suite and resource is any of parameters, bytes, or estimated joules, reported as separate numbers rather than one
(because "intelligence" has subjective parts). The suite itself is a design decision to make together.

### Design notes for autonomous learning (constraints, not reasons to hold back)

Learning from autonomous browsing means learning from unvetted text. That needs provenance (where each lesson came from), a way to
unlearn or quarantine, and a rule for how much a single source can change the unit. Early on, as Daws says, a teacher stays in the
loop. These are the knobs that make the autonomy safe to turn up gradually.

---

## Entry 3 (2026-10-19): The goal, stated plainly, and freedom to explore

By any means: **run frontier intelligence on low-end or typical consumer hardware without overwhelming the device**, in RAM,
memory or otherwise. The architecture that does this is likely **different from today's, and more effective with Adaptive
Learning (AL)**. The assistant keeps freedom to choose and change approaches, models, systems or working units along the way; the
suggestions in Entry 2 are starting points, not a plan to follow.

### What "not overwhelming the device" means (the assistant's working reading, to be corrected)

The device must stay usable while Transient runs: it is a guest, not a tenant that takes over. The constraints are measured, not felt:
- **Resident RAM** (what must stay loaded), and **peak RAM** during learning.
- **Storage footprint**, and how much of it is read sparsely (cheap) versus loaded whole (expensive).
- **Memory bandwidth** per answer and per lesson, since moving data is where the energy goes (the baby: ~91% of estimated energy).
- **Energy and heat**: joules per answer, per lesson, and sustained temperature on a phone.
- **Background behaviour**: nothing that makes the device slow when the user is doing something else; learning gets scheduled for idle time.

Proposed reference envelope, to give the work a concrete target (every number negotiable): a *budget* of roughly 1 GB resident RAM
and a few GB of storage for the learned unit on a 8-12 GB phone or laptop (Daws's S24 Ultra and this 7.5 GB laptop are the real
test devices), with bigger stores allowed on disk because disk is read sparsely. Smaller tiers (a 2-4 GB phone) are the stretch
goal. These budgets belong in the experiment logs next to every result.

---

## Entry 4 (2026-10-19): Not even tokens, and absolute freedom

Working on this and on Adaptive Learning will be a challenge. The deviation may go deeper than architecture: **it might not even
be the tokens the industry has normalised.** The goal is a completely different route, taking nothing for granted. Daws does not
oppose using what the industry has learned; the point is to deviate *from scratch* and keep only what earns its place.

**Absolute freedom** is given to the assistant on this project, under the stated directives and goals of what Transient must be
(frontier-class intelligence on ordinary consumer hardware, light on RAM and memory, built around Adaptive Learning, in Rust,
pursued whether or not it succeeds, honest about evidence).

### What this changes in practice (the assistant's reading)

- **Words as units are one choice among several.** The baby already reads whole words from a fixed vocabulary, a smaller version of
  the industry's token habit. Candidates to try instead: reading *letters* or hashed letter-groups (open vocabulary, no tokenizer,
  typo-tolerant), reading *meaning-carrying chunks it discovers itself*, or other representations that are learned rather than
  imposed. The baby's own results already show the representation matters more than the learning trick (the reader change
  beat every learning-method change).
- **Industry knowledge is a toolbox, not a template.** Anything borrowed (embeddings, softmax, Adam) is kept only while it earns its
  place in a measured comparison, and is replaced when something better shows up.

---

## Entry 5 (2026-10-19): The learning process itself adapts

Adaptive Learning is really important here. It is not only that the system adapts to *what* it learns: **the process of learning
itself can change internally, how it learns**, and that is the real adaptation and the dynamic part. Daws asks the assistant to
keep going with its own suggestions and not give up.

### What this changes in practice (the assistant's reading)

Three levels of adaptation, each a different depth:
1. **Adapting the knowledge**: new facts, new words, corrections (the baby does this today).
2. **Adapting how much and when it learns**: steps per lesson, when to sleep, what to ask, how far to trust a match. Today these are
   constants chosen by the programmer (steps per lesson 10, sleep every 60, match threshold 0.6-0.75). The step toward Entry 5 is
   to make each a quantity the baby *measures and adjusts from its own experience*.
3. **Adapting the learning mechanism itself**: for each kind of thing it meets, try different ways of learning it (write it down,
   adjust the weights, fit a rule) and keep whichever generalises, deciding from evidence the baby gathers itself. That is learning
   to learn at the level of mechanism, not only of numbers.

The baby's own results already point there: no single mechanism wins everywhere (the fact store wins on facts, the network on
rewordings after consolidation, and nothing yet on rules). A baby that can pick, and justify the pick, is closer to the
description than one with a single fixed learner.

---

## Progress log (assistant-maintained; details live in `baby/README.md`)

- **2026-10-15** The baby: a 16k-parameter, from-scratch, zero-dependency Rust net that answers a query with one word. Joule ledger.
  `life`: five ways of learning compared (a fact store matched retraining's retention at ~1/900 the estimated energy).
- **2026-10-19** Position-free readers (the reader mattered more than any learning trick: bag reader 9k params, 100% on rewordings).
  Knowing what it doesn't know (familiarity beats confidence; calibrated refusal) and `teach` (open-vocabulary memory, no weight change).
- **2026-10-19** Letters instead of words (Entry 4): spelling-based reader generalises to misspellings it never met (37% vs ~11% on
  misspelt key words; 61% with noisy training). `curious`: a baby that chooses what to ask needs about half the answers of an
  undirected one, but stops short because familiar is not the same as correct. Still open: rules instead of tables (sums), sparse
  updates, growth, discovery without a teacher.
- **2026-10-19** Entry 5 (the learning process adapts): the baby fitted a rule for sums from examples (100% on sums it was never
  taught and beyond the range it was taught, where memorising networks get 39% and 5%) and chose it over its network from
  held-back evidence; it learned how far to trust a match (settled at 0.50, 193 answers instead of 285). Honest limits: the rule's
  form and the counting lessons were supplied by me, and it chooses only among mechanisms I gave it.
- **2026-10-19** A library of ways to learn (`baby kinds`): for each unit of lessons the baby measures four mechanisms on facts it held
  back and uses the best. 100% / 96% / 82% (reworded / new items / beyond the taught range) versus at most 100% / 38% / 19% for any one
  mechanism everywhere; a new kind of lesson (halving) found its own mechanism as data only; multiplication, which nothing in the
  library can express, was flagged as memorise-only instead of bluffed.
- **2026-10-20** It builds mechanisms (Entry 5, level 3): a search over small expressions found `first x second` for multiplying and
  the absolute difference for "how far apart" (composed from subtraction and max when the convenient operation was removed),
  100% beyond the taught range where every fixed mechanism failed; and it found nothing for an arbitrary code, so it memorised
  instead of inventing an explanation. Limits: the expression language and the number line are mine. Next: unify with `ask`,
  a richer space to build in, adapt the remaining knobs, discover the number line.
- **2026-10-20** One baby (`baby school` / `baby ask`): 13 units, five mechanisms chosen per unit by evidence, a question pipeline
  that answers with a stated reason or refuses with one. Scorecard on five seeds: 0% wrong (right, or refused). A bluff found by
  asking for every code from zero to ten (it answered the code for "one" when asked about "five") was fixed by requiring the
  word that fills the blank to match. Next: keep learning after school (re-test how a unit is learned as lessons arrive, open
  new units), digits and new phrasings.
- **2026-10-20** Less effort, same answers (Daws: use the mathematics of the computation): answering fell from ~8,977 operations to 85 (106x),
  ~56x less estimated energy, identical answers (99.7% right, 0% wrong), by routing with a precomputed table instead of comparing with every
  lesson, indexing memory, and not training a network where cheaper ways already work. And it keeps learning after school: a new phrasing joined
  a unit, and a kind of lesson it had never seen opened a unit, remembered until it had five facts, then worked out a rule that reaches numbers
  it was not taught. Next: let it find the units itself from unlabelled lessons.
- **2026-10-20** It finds the units itself (Entry 5, pushing adaptive learning): given 792 shuffled lessons with no grouping it recovered 9 units (every
  fact kind pure, the rule family merged, `times` and `apart` kept apart) with a scorecard identical to the teacher-organised baby, 0% wrong, on five seeds.
  Three mistakes found by asking adversarial questions (a thing-word becoming an anchor, `minus` merged into `apart` so "three minus five" was answered, a small
  group swallowed by a big one) are recorded in the README. And a different way of computing: fitting a rule exactly instead of nudging it 800 times is 392x
  fewer operations and 56x faster for the same rule, which cut discovery from ~30 s to 3.7 s. Next: discovery that keeps going on every new lesson; incremental
  and exact updates; mechanisms it proposes itself.
- **2026-10-20** Storage (Daws: the heavy part of conventional training is the data, not the model): the baby's school is 93% lessons and 1.3 KB of
  what was learned (a sum rule: 87 lessons, 2,780 bytes -> 240 bytes; multiplication: 66 lessons -> a 58-byte program). It can forget what is explained (`sleep --compact`):
  49% of the file, still answers every lesson it was ever taught, 0% wrong, refusals only on unseen phrasings (-2.3 points). And lessons that fit nothing wait in a pen and open
  new units on their own (`less/than` from six lessons, no name given; "multiplied by" joined `times`). The apart/minus trap appeared a third time and was closed by defining ambiguity
  as "expressions that behave differently somewhere".
- **2026-10-20** Learning is light (Daws: training is intensive; AL should let nearly anyone do it, no data centre): measured on one laptop core, no GPU: teaching 792 lessons in
  13 units from nothing = 3.8 s and 11.9 MB; a lesson it already understands = 0.18 ms and nothing kept; a new way of saying something it knows = 3 ms; a new kind of lesson, no unit named = 1 ms
  and a unit opens at the sixth. It also reorganises itself over time (units that follow one rule merge, 13 -> 9; a polluted unit splits). A bug that treated "five minus two" and "two minus five" as one
  lesson had inflated the first cost numbers (93 ms per lesson) and is fixed. Honest limit: 800 tidy lessons is not the world; nothing yet says how this grows.

- **2026-10-20** Published (Daws): the work now lives in a public GitHub repo (`DawsMacs/TransientAI`) under AGPL-3.0-or-later, with a technical README, a banner, SPDX headers, a DCO instead of a CLA, and CI that fails if the scorecard's WRONG column rises above 0.0 %. See Entry 6.

- **2026-10-20** Scale (Daws: nearly anyone, no data centre): `baby scale` grew the world to 600 units / 26k lessons / a 10,000-word vocabulary. Learning one more fact stayed flat at ~0.5 ms after the per-lesson table rebuild
  (O(all lessons), 40 ms at 200 units) was made incremental; going back over everything went 99.9 s -> 0.3 s; teaching 26k lessons 127 s -> 2.65 s (no network trained for plain tables). WRONG stayed 0 % on clean lessons. Found and fixed on the way: a cross-validation flaw
  that let memory peek at swapped twins of commutative facts, a merge test that depended on seed luck, and a bar for "may answer" (25 %) that clean data never tested. **Not solved: 2 % wrong lessons give ~1 % wrong answers.** Next: robust fitting and voting memory.

- **2026-10-20** Inventing mechanisms (Daws was told it would be interesting; direction left to the assistant): `baby invent` teaches kinds nobody wrote a mechanism for. The baby worked out all 16 hidden numeric kinds and, using constants and programs it had learned from earlier kinds as new vocabulary, 7 of 9 harder ones, every time over 20 seeds, with 0 wrong and 0 of 120 unexplainable kinds (random codes, primes) explained. Taught in the wrong order it fails (8 of 72), so the gain is learning, not luck. Along the way the harder kinds exposed two honesty bugs (a coincidence "explaining" a tens digit, 60 wrong answers; primality "explained" by chance, a multiple-comparisons problem) fixed with a uniqueness rule and an information-vs-search-effort check. Honest limits: the operations and thresholds are the assistant's; it invents expressions, not new kinds of learner.

- **2026-10-20** Noise, finished: with 2 % of taught answers wrong the baby is now nearly as good as on clean data (97-100 % of unseen questions right, 0 % wrong) in both the words and the digit worlds, and 0 % wrong at 5 %; under noise it still works out 127 of 128 hidden kinds. Six changes (robust rule fitting that protects a genuine second rule, majority-per-input before building programs, a rule-versus-memory cross-check, splits that must improve things, merges at the reliability bar, routing smoothing scaled to the vocabulary), three of which were wrong first. It also exposed an old hidden weakness (clean 600-unit worlds were only 89.7 % right *after* reorganising) that is now fixed.

- **2026-10-20** It asks what it wants to know (Entry 5: the learning process adapts; Entry 2: it chooses what to look at): when its lessons leave several explanations equally good, the baby now keeps them, answers only where they agree, and can name the number that would tell them apart (`baby wonder`). With digit sum taught on 0-15, one question took it from 88.9 % to 100 % of unseen questions where random numbers needed three; it asked nothing about random codes, primes or xor (0 wrong). Found on the way: an ambiguous guess had entered its vocabulary and hidden its own ambiguity, and units left open were never revisited when its vocabulary grew. Honest limits: a simulated, always-right teacher; numeric programs only; questions chosen by disagreement, not by expected information.

- **2026-10-21** Adapting effort, reviewing itself, checking itself: the baby no longer spends cross-validation effort on a unit whose whole-data search found nothing (battery 23.5 s -> 14 s, identical results), reviews settled-looking programs when its vocabulary changes (stage-2 kinds 61 -> 66 of 72), never lets a unit be explained by vocabulary that unit contributed itself (a circle found by a test), and can name where it would check its own explanation. The self-check found nothing to refute across 105 units (87 at 5 % noise), so it is insurance, not a measured gain. Honest limit: adapting its own knobs (folds, bars, budgets) from measurement is still open.

- **2026-10-21** Tuning its own bar: the baby now measures the share of wrong lessons in its own data (it estimates 9.6 % where the world has 9.7 %) and sets how reliable a mechanism must be before answering from that, between a floor found by experiment (a mechanism right 72 % of the time does answer wrongly) and the old 0.9 for clean worlds. At 10 % label noise it answers 98 % of unseen questions instead of 42 %, with 0 false inventions and 0 wrong in the adversarial battery at 0, 5 and 10 % noise. A negative result is recorded too: the search budget cannot be cut by watching partial progress (doomed and deep searches look alike). Honest limit: one constant of several; the floor rests on thin evidence.

- **2026-10-21** A hole found and a choice for Daws: a rule that fits everything taught and changes regime outside it (a sum that saturates or wraps) is answered confidently and wrongly (53 wrong answers per seed in the new deceptive kinds); no data in hand can reveal it. A `--careful` mode answers only inside the range it was taught or has had checked, and the baby's self-checks (now covering rules and selections) win the coverage back: after two rounds 0 wrong and 99.2 % of the extrapolation kept. **Default is still permissive; whether careful should be the default is Daws's call** (extrapolation is the showcase, and the hazard). Tracing a third-round regression also found and fixed a library bug (a unit re-adding rewritten copies of itself, evicting others).

- **2026-10-21** Guessing like a scientist (Daws, Entry 7): careful became the default, and the baby no longer just refuses beyond what it was taught: it offers a *labelled guess* ("probably 42"), never stated as fact, scored apart. A miss ends the guessing out there (before: one correction fixed 2 of 307 neighbouring wrong guesses and turned 2 into wrong *facts*; after: 0 repeats, 0 wrong facts, though it stops rather than learning the new regime), a confirmed guess extends where it may speak as fact, and it keeps and states a record of its own guessing per distance (stated rate below the realised rate everywhere: under-confident, never over). Testing that found a hole: two confirmed edges made a box-shaped "reach" vouch for an unchecked corner (4 wrong facts of 9,309); the reach is now a staircase, 0 wrong of 8,884, at a measured cost in coverage. Honest limits: simulated always-right teacher; synthetic worlds; one correction teaches it where it is wrong, not what is right. Next: the real-data bridge, then the controlled comparison (GPU to be rented by Daws).

- **2026-10-21** First contact with real text: SVAMP, 1,000 one-step arithmetic word problems somebody else wrote. The baby learned nothing at first (0 wrong, everything refused) and 159 "units" named after story words. Measuring why showed the right expressions already existed in its language but the search needed one to fit *every* lesson; it now accepts a program that fits most lessons and answers only as a labelled guess with a stated rate. When told a question's kind its guesses are 67-90 % right; recognising the kind from the words alone is the wall (49-62 %, and a tuned bag-of-words classifier reaches 57-65 % on the same data), so about a fifth to a third of all questions get a wrong *labelled* guess while 0 are stated as wrong facts. A long battery caught a regression the short ones missed (a partial program froze a unit that used to be revisited), now fixed. Honest limits: one adversarial corpus; problems with three or four numbers still learn nothing; discovery on real text is unsolved. Next: which numbers matter, routing by compositional cues, a second (factoid) corpus.

- **2026-10-21** The controlled comparison begins (Entry 2): matched attention transformers (a plain one written in this repo, and a conventional PyTorch one tuned by a 24-setting search whose cost is counted against it), same lessons, same questions, joules measured with the CPU's own counter. The baby is never wrong on unseen questions where the transformers are wrong up to 40 %, tuned or not; it extrapolates (right every time, as a labelled guess) where they stay at 14-28 % right at every scale tried (20 to 600 units); it adds new facts with no forgetting (a cheap fine-tune forgets heavily, a safe one costs 3-64x more); teaching from scratch is 9-44x cheaper than the plain transformers and orders of magnitude cheaper than the tuned PyTorch route, and its cost per lesson is flat. Against the simple story, kept in: from 60 units on a plain transformer is more accurate than the baby on unseen questions (100.0 vs 89.7 % at 600 units, with 0 wrong), because the baby refuses about 8-11 % of unseen questions (each with a right labelled guess; it does not grow with scale: I had mistaken one lucky small world for a trend); tuning did not help the transformer; the energy ratio across sizes is not monotone (20x, 9x, 10x up to 200 units, then 32-253x at 400-1,000 units over three seeds; the baby teaches 43,023 lessons in 6 s while the transformer needs 28 minutes), and an early claim of mine that the advantage shrinks with scale was an artefact of noisy readings and is withdrawn. Mistakes recorded: the energy meter was wrong four times (a stale idle baseline that produced impossible 0.00 J readings, a CPU-share correction that under-counted, an uncorrected busy desktop, and a zero that came back when the background went quiet mid-run); all tables were re-measured on a quiet machine, with an uncertainty of about plus or minus 30 %. Honest limits: tidy rule-shaped lessons (the baby's home ground), a small search, one to three seeds. A GPU would not help the small-model comparison; it would help a pretrained-model comparison on real text.

- **2026-10-22** Why the baby refuses, and asking a teacher: I had believed it refused more as the world grows; printing every refusal showed it does not (about 8-11 % at every size; one lucky small world had fooled me). Each refusal is a question one step above the top edge or corner of what its unit was taught, and each carried a right labelled guess. Making the baby's own check questions automatic (a simulated teacher) exposed four defects in turn (probes dropped when their answer could not be said, no probes of the frontier, a starved question budget, and a reach that shrank as it learned because it leaned on a moving glue heuristic) and after fixing them about 5-7 questions per numeric unit make every unseen and every beyond-range question a stated fact at 20 and 200 units, with no wrong fact and no trap answered. What was left (2.6 % at 1,000 units) was recognition: learning in one unit shrinks the margin of small neighbours below a floor fixed at teaching time. My first fix did not work and was reverted; the second, keeping the floor at a fixed ratio to the leave-one-out margin (what an unseen question of a known unit looks like to the table), works, and brings it to 1.6 %, the weakest tail the floor is designed to leave, with no wrong answer and no trap answered. Honest limits: simulated always-right teacher, number-word worlds, tidy rule-shaped units.

- **2026-10-22** A teacher who is sometimes wrong (Daws: reinforcing from true facts, trusting new sources the way a person does): simulated teachers who are wrong 1-20 % of the time, by slips (asking again gets a fresh answer) or consistently (the same wrong answer every time). A single wrong answer used to turn an exact program into one that only guesses (a loss of up to a fifth of its coverage); an established program now survives a few contradicting lessons, as a rule already did, and an opt-in skepticism (hold an answer that contradicts a settled rule, ask again, believe it only when repeated) defends against slips almost for free: 99.9-100 % of unseen and beyond-range questions stay right up to 20 % slips, against 76.8 % for a trusting baby. Consistent errors cannot be told from the truth by asking again: 99.6 % at 5 %, 88 % at 20 %, and every wrong fact it states is a fact it was wrongly told. Honest limits: simulated errors, independent of difficulty; a source wrong about a whole kind of thing is not tested and cannot be caught from inside; one world and seed.

---

## More to come

Daws has more decisions to record. Entries below, newest last.

---

## Entry 6 (2026-10-20): Open, and kept open

Daws made a public GitHub repository for the work (`DawsMacs/TransientAI`) and chose the **AGPLv3** on purpose. One internal goal is to make sure this
work **stays open for everyone, and is not hoarded or closed.** The README is to be technical rather than philosophical (the ambition may be mentioned,
not spelled out as it is in this file), with a banner built from the logo and the Lexend typeface. The assistant keeps the freedom to suggest whether
"AGPL, or version 3 and later" serves that goal better.

### Context added by the assistant (labelled; check, edit or delete freely)

- **Why a network-copyleft licence fits the goal.** The GPL obliges you to share source only when you *distribute* a program. A company can run a modified
  version as a hosted service, never distribute it, and keep its changes. The AGPL closes that gap: users who interact with a modified version over a
  network are entitled to its source. For a project whose point is that capable AI should not be the property of a few providers, that is the clause that matters.
- **"Or later" was chosen (`AGPL-3.0-or-later`).** It lets the FSF's future versions apply, which is the practical way to close loopholes that do not exist yet
  (the 2007 text predates today's AI services) and keeps the code combinable with other copyleft code as licences evolve. The cost is trusting the FSF's stewardship
  of future versions; if that trust ever changes, `-only` can be adopted from then on for new files (a licence already granted cannot be taken back). Easy to revert now if Daws prefers `-only`.
- **A licence binds others, not the copyright holder.** As sole copyright holder Daws could still relicense his own code. What prevents anyone, including a future
  Daws, from closing *other people's* contributions is that contributions come in under the same licence with a DCO sign-off and **no CLA** (a CLA is the usual
  mechanism by which a project is later relicensed into a closed product).
- **What the AGPL does not cover.** It governs the software. It does not by itself say anything about trained or learned files (`.school`, model weights) or data
  that people produce with it; if a released learned file should also stay open, that wants its own explicit statement when we first publish one.
- **Public means public.** Anything committed, including its history, is permanently visible and may be cached or forked. Private-project notes were kept out
  of the first commit for that reason (`.local/` is git-ignored).


---

## Entry 7 (2026-10-21): Frontier, guessing like a scientist, and what is trusted

### What "frontier" means here

The ambition is to bring the level of models with trillions of parameters (proprietary ones run through APIs in large data centres, or open-weight ones such as Kimi K3, over 2 trillion parameters, about 40 billion active) to
ordinary hardware, locally. "Frontier" simply means what is in that range: **"I trust this AI in my workflow, education, coding, job, school"**, general-purpose intelligence. It is vague on purpose. It is **an ambition, not a destined goal**, and it
must not falsely lead the work elsewhere. Daws still bets on it because it could change how intelligence is distributed globally, so that disadvantaged people can have it too one day without expensive hardware.
Of the possible measures, the one that matters is **capability per unit of computation as the machine itself measures it (joules)**. Benchmarks and chat quality are trend-based and can change; they come later, optionally, if the joule measure is substantially met.

### Careful, but not timid: the baby should guess like a scientist

Careful becomes the default. But an intelligent system that cannot find patterns, predict or hypothesise would be a very smart and very restrictive system, and one that is always confident would be a chatbot that is wrong all the time.
The human parallel is the way out: scientists guess, form hypotheses, then theories, then establish laws or facts, and do it rigorously. Risk-taking sometimes pays more than being overly cautious, because restraint keeps you in the same loop; **guess, get something wrong, update: that is learning.**
The baby does not have to be perfect. It has to understand **what is true, the range of complexity, and why something is true or false**, and then, through trial and error, learn using Adaptive Learning. LLMs need extensive training or post-training to update something; **ours can update in real time, which is the biggest advantage.**
Keep that in mind in handling this.

### What is trusted, and the licence

The licence stays **AGPL-3.0-or-later**. Daws accepts the trust it involves (the FSF, and GitHub, and the assistant: "we have to trust many things here"). If the AGPL's nature ever changed in a way he disliked, new releases could go out under AGPL-3.0-only, and older ones remain under what they were granted.
He trusts the GNU foundation: Stallman went through a lot to invent copyleft. On git: freedom was given so that there is progress without him having to spoon-feed or sit through everything; risk is real, so **being careful, precise, planning, and asking in risky situations is still the right call.**
**The name "Transient" stays a mystery for now** ("for the lore").

### Decisions in this entry
- `--careful` becomes the default (permissive is `--permissive`).
- GPU: Daws will rent one for the controlled comparison with a small transformer (he will say when it is set up); a local iMac (macOS, AMD Radeon 4 GB, 40 GB RAM) was considered and set aside as complicated.
- The energy meter (RAPL) was made readable by Daws himself.
- GitHub social preview and topics were set by Daws.

### Context added by the assistant (labelled)

- Daws's ladder (hypothesis, theory, law) is implemented as: a labelled **guess** (a trusted rule used outside the range taught or checked), a **checked** range (a confirmed answer extends where it may speak as fact), and **taught** facts. See the 2026-10-21 section of `baby/README.md`.
- The honesty rule became "never *silently* wrong": WRONG (a fact stated and false) must stay at 0, a labelled guess may be wrong, and what is measured is whether guesses are calibrated and whether one correction fixes the next mistake.

### Context added by the assistant (labelled): the rename (2026-10-21)

Daws renamed the GitHub repository from `DawsMacs/TransientAI` to `DawsMacs/Transient`. **TransientAI is now the name of the creator of the Transient project** (Daws is its owner and founder); the project, the repo, the README and the banners are called **Transient**. The copyright and licence holder is unchanged: DawsMacs, AGPL-3.0-or-later. Earlier entries above that say `DawsMacs/TransientAI` record the name at the time and are left as written; GitHub redirects the old URL.


---

## Entry 8 (2026-10-21): The licence becomes AGPL-3.0-only

Daws decided to move from `AGPL-3.0-or-later` to **`AGPL-3.0-only`**, to rely less on the FSF or any institution: "or later" means every future version the FSF publishes is automatically available to every recipient, which fails open if something goes wrong there. He noted that nobody had yet viewed the repository, so it was an easy moment to change it.

### Context added by the assistant (labelled; check, edit or delete freely)

- **What "or later" does and does not do.** The licence text never changes; AGPLv3 stays AGPLv3 and any copy can always be used under it. "Or later" is an *option each recipient may take*, and it attaches to old releases too: a recipient of 2026 code could choose a weaker "AGPLv4" in the future for their own copy and derivatives. That was the exposure.
- **Why `-only` rather than a proxy now.** AGPLv3 §14 lets a notice name a *proxy* whose public acceptance of a version permanently authorises it. That fails closed (no proxy statement means v3 stays), but its benefit is adopting a future legal fix without contacting every contributor, which matters only once there are outside contributors. As sole copyright holder, Daws can add a proxy or "or later" for *new* releases at any time; tightening the other way cannot be undone for copies already distributed. So `-only` is the reversible choice.
- **Honest caveat.** Commits published before this change carried `AGPL-3.0-or-later`; anyone who already pulled them keeps that option on those copies. The change applies from this commit onward.
- **Mechanics.** SPDX headers, `Cargo.toml`, `REUSE.toml`, README, CONTRIBUTING, CLAUDE.md and the handover now say `AGPL-3.0-only`; `LICENSES/AGPL-3.0-or-later.txt` was renamed `AGPL-3.0-only.txt`. The `LICENSE` file is the unmodified AGPLv3 text, whose generic "any later version" wording is part of the licence text itself and is left as is. Earlier entries above that say `-or-later` record the decision at the time and are left as written. The website text was updated to match.
