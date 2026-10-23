# TransientVision

A living record of why Transient exists and what Daws is betting on. Entries are dated and kept in his framing. Where I (the
assistant) add context, it is in a separate, labeled section so his reasoning stays his. **This is a hunch, not a verified claim,
and Transient pursues it regardless of how long it takes.**

Started 2026-10-19.

**Vocabulary Daws has coined** (directions, not exact definitions; terminology is allowed to shift):
- **Adaptive Learning**: learning that goes beyond conventional machine learning: continuous, from interaction, with the internals changing.
- **Maximization**: optimizing a system for the most intelligence in a given computational space.
- **UMA (Ultragenerative Maximized Architecture)**: the architecture Transient is aiming at. A *bare* UMA is the tiny starting unit;
  a *learned* UMA has accumulated what it learned (the counterpart of a trained GPT, a usable LLM).

---

## Entry 1 (2026-10-19): The iceberg

### The bet

There is an iceberg under the question of how to run frontier intelligence on hardware. Ordinary, low-end hardware has probably been
underestimated, and not by a little: by an enormous cliff. If AI is built from scratch again, and the right implementation is found,
the path to an architecture that is far more efficient may be like a maze, with a real way through it that nobody has taken because
the industry has not been looking there.

Daws is explicit about what kind of belief this is: **a hunch, not a verified claim.** It rests on his personal judgment, including
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

## Context added by the assistant (labeled, so it can be checked and removed)

These are things that rhyme with the hunch. They do not prove it, and some point the other way; both kinds are listed.

**Evidence that points the same direction**
- **Software has already out-run hardware at times.** OpenAI's "AI and Efficiency" study found that the compute needed to train a
  classifier to AlexNet-level accuracy on ImageNet fell **44x between 2012 and 2019**, with algorithmic efficiency doubling about
  every 16 months. Moore's Law over the same stretch would have given about 11x
  ([source](https://openai.com/index/ai-and-efficiency/)). For tasks with heavy investment, algorithms gave more than hardware did.
- **A single optimizer-and-architecture rework cut a benchmark's cost by an order of magnitude.** The nanoGPT speedrun took GPT-2-small
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

The next big objective, a gamble: learning that surpasses machine learning as practiced and how LLMs learn today.

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

- **Maximization** is Daws's term for optimizing a system or model to hold the most intelligence in a given computational space. It is
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

## Context and suggestions added by the assistant (labeled; check, edit or delete freely)

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
against them, which is what makes it a good bet to test. Related phenomena exist: sudden generalization after long training in
small networks ("grokking", from my recollection of Power et al. 2022) and abilities that appear to switch on at a size (whether
those jumps are real or an artifact of the metric is debated). None of this confirms the UMA claim; it says where to look.

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
- **Background behavior**: nothing that makes the device slow when the user is doing something else; learning gets scheduled for idle time.

Proposed reference envelope, to give the work a concrete target (every number negotiable): a *budget* of roughly 1 GB resident RAM
and a few GB of storage for the learned unit on a 8-12 GB phone or laptop (Daws's S24 Ultra and this 7.5 GB laptop are the real
test devices), with bigger stores allowed on disk because disk is read sparsely. Smaller tiers (a 2-4 GB phone) are the stretch
goal. These budgets belong in the experiment logs next to every result.

---

## Entry 4 (2026-10-19): Not even tokens, and absolute freedom

Working on this and on Adaptive Learning will be a challenge. The deviation may go deeper than architecture: **it might not even
be the tokens the industry has normalized.** The goal is a completely different route, taking nothing for granted. Daws does not
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
   adjust the weights, fit a rule) and keep whichever generalizes, deciding from evidence the baby gathers itself. That is learning
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
- **2026-10-19** Letters instead of words (Entry 4): spelling-based reader generalizes to misspellings it never met (37% vs ~11% on
  misspelled key words; 61% with noisy training). `curious`: a baby that chooses what to ask needs about half the answers of an
  undirected one, but stops short because familiar is not the same as correct. Still open: rules instead of tables (sums), sparse
  updates, growth, discovery without a teacher.
- **2026-10-19** Entry 5 (the learning process adapts): the baby fitted a rule for sums from examples (100% on sums it was never
  taught and beyond the range it was taught, where memorizing networks get 39% and 5%) and chose it over its network from
  held-back evidence; it learned how far to trust a match (settled at 0.50, 193 answers instead of 285). Honest limits: the rule's
  form and the counting lessons were supplied by me, and it chooses only among mechanisms I gave it.
- **2026-10-19** A library of ways to learn (`baby kinds`): for each unit of lessons the baby measures four mechanisms on facts it held
  back and uses the best. 100% / 96% / 82% (reworded / new items / beyond the taught range) versus at most 100% / 38% / 19% for any one
  mechanism everywhere; a new kind of lesson (halving) found its own mechanism as data only; multiplication, which nothing in the
  library can express, was flagged as memorize-only instead of bluffed.
- **2026-10-20** It builds mechanisms (Entry 5, level 3): a search over small expressions found `first x second` for multiplying and
  the absolute difference for "how far apart" (composed from subtraction and max when the convenient operation was removed),
  100% beyond the taught range where every fixed mechanism failed; and it found nothing for an arbitrary code, so it memorized
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
  it was not taught. Next: let it find the units itself from unlabeled lessons.
- **2026-10-20** It finds the units itself (Entry 5, pushing adaptive learning): given 792 shuffled lessons with no grouping it recovered 9 units (every
  fact kind pure, the rule family merged, `times` and `apart` kept apart) with a scorecard identical to the teacher-organized baby, 0% wrong, on five seeds.
  Three mistakes found by asking adversarial questions (a thing-word becoming an anchor, `minus` merged into `apart` so "three minus five" was answered, a small
  group swallowed by a big one) are recorded in the README. And a different way of computing: fitting a rule exactly instead of nudging it 800 times is 392x
  fewer operations and 56x faster for the same rule, which cut discovery from ~30 s to 3.7 s. Next: discovery that keeps going on every new lesson; incremental
  and exact updates; mechanisms it proposes itself.
- **2026-10-20** Storage (Daws: the heavy part of conventional training is the data, not the model): the baby's school is 93% lessons and 1.3 KB of
  what was learned (a sum rule: 87 lessons, 2,780 bytes -> 240 bytes; multiplication: 66 lessons -> a 58-byte program). It can forget what is explained (`sleep --compact`):
  49% of the file, still answers every lesson it was ever taught, 0% wrong, refusals only on unseen phrasings (-2.3 points). And lessons that fit nothing wait in a pen and open
  new units on their own (`less/than` from six lessons, no name given; "multiplied by" joined `times`). The apart/minus trap appeared a third time and was closed by defining ambiguity
  as "expressions that behave differently somewhere".
- **2026-10-20** Learning is light (Daws: training is intensive; AL should let nearly anyone do it, no data center): measured on one laptop core, no GPU: teaching 792 lessons in
  13 units from nothing = 3.8 s and 11.9 MB; a lesson it already understands = 0.18 ms and nothing kept; a new way of saying something it knows = 3 ms; a new kind of lesson, no unit named = 1 ms
  and a unit opens at the sixth. It also reorganizes itself over time (units that follow one rule merge, 13 -> 9; a polluted unit splits). A bug that treated "five minus two" and "two minus five" as one
  lesson had inflated the first cost numbers (93 ms per lesson) and is fixed. Honest limit: 800 tidy lessons is not the world; nothing yet says how this grows.

- **2026-10-20** Published (Daws): the work now lives in a public GitHub repo (`DawsMacs/TransientAI`) under AGPL-3.0-or-later, with a technical README, a banner, SPDX headers, a DCO instead of a CLA, and CI that fails if the scorecard's WRONG column rises above 0.0 %. See Entry 6.

- **2026-10-20** Scale (Daws: nearly anyone, no data center): `baby scale` grew the world to 600 units / 26k lessons / a 10,000-word vocabulary. Learning one more fact stayed flat at ~0.5 ms after the per-lesson table rebuild
  (O(all lessons), 40 ms at 200 units) was made incremental; going back over everything went 99.9 s -> 0.3 s; teaching 26k lessons 127 s -> 2.65 s (no network trained for plain tables). WRONG stayed 0 % on clean lessons. Found and fixed on the way: a cross-validation flaw
  that let memory peek at swapped twins of commutative facts, a merge test that depended on seed luck, and a bar for "may answer" (25 %) that clean data never tested. **Not solved: 2 % wrong lessons give ~1 % wrong answers.** Next: robust fitting and voting memory.

- **2026-10-20** Inventing mechanisms (Daws was told it would be interesting; direction left to the assistant): `baby invent` teaches kinds nobody wrote a mechanism for. The baby worked out all 16 hidden numeric kinds and, using constants and programs it had learned from earlier kinds as new vocabulary, 7 of 9 harder ones, every time over 20 seeds, with 0 wrong and 0 of 120 unexplainable kinds (random codes, primes) explained. Taught in the wrong order it fails (8 of 72), so the gain is learning, not luck. Along the way the harder kinds exposed two honesty bugs (a coincidence "explaining" a tens digit, 60 wrong answers; primality "explained" by chance, a multiple-comparisons problem) fixed with a uniqueness rule and an information-vs-search-effort check. Honest limits: the operations and thresholds are the assistant's; it invents expressions, not new kinds of learner.

- **2026-10-20** Noise, finished: with 2 % of taught answers wrong the baby is now nearly as good as on clean data (97-100 % of unseen questions right, 0 % wrong) in both the words and the digit worlds, and 0 % wrong at 5 %; under noise it still works out 127 of 128 hidden kinds. Six changes (robust rule fitting that protects a genuine second rule, majority-per-input before building programs, a rule-versus-memory cross-check, splits that must improve things, merges at the reliability bar, routing smoothing scaled to the vocabulary), three of which were wrong first. It also exposed an old hidden weakness (clean 600-unit worlds were only 89.7 % right *after* reorganizing) that is now fixed.

- **2026-10-20** It asks what it wants to know (Entry 5: the learning process adapts; Entry 2: it chooses what to look at): when its lessons leave several explanations equally good, the baby now keeps them, answers only where they agree, and can name the number that would tell them apart (`baby wonder`). With digit sum taught on 0-15, one question took it from 88.9 % to 100 % of unseen questions where random numbers needed three; it asked nothing about random codes, primes or xor (0 wrong). Found on the way: an ambiguous guess had entered its vocabulary and hidden its own ambiguity, and units left open were never revisited when its vocabulary grew. Honest limits: a simulated, always-right teacher; numeric programs only; questions chosen by disagreement, not by expected information.

- **2026-10-21** Adapting effort, reviewing itself, checking itself: the baby no longer spends cross-validation effort on a unit whose whole-data search found nothing (battery 23.5 s -> 14 s, identical results), reviews settled-looking programs when its vocabulary changes (stage-2 kinds 61 -> 66 of 72), never lets a unit be explained by vocabulary that unit contributed itself (a circle found by a test), and can name where it would check its own explanation. The self-check found nothing to refute across 105 units (87 at 5 % noise), so it is insurance, not a measured gain. Honest limit: adapting its own knobs (folds, bars, budgets) from measurement is still open.

- **2026-10-21** Tuning its own bar: the baby now measures the share of wrong lessons in its own data (it estimates 9.6 % where the world has 9.7 %) and sets how reliable a mechanism must be before answering from that, between a floor found by experiment (a mechanism right 72 % of the time does answer wrongly) and the old 0.9 for clean worlds. At 10 % label noise it answers 98 % of unseen questions instead of 42 %, with 0 false inventions and 0 wrong in the adversarial battery at 0, 5 and 10 % noise. A negative result is recorded too: the search budget cannot be cut by watching partial progress (doomed and deep searches look alike). Honest limit: one constant of several; the floor rests on thin evidence.

- **2026-10-21** A hole found and a choice for Daws: a rule that fits everything taught and changes regime outside it (a sum that saturates or wraps) is answered confidently and wrongly (53 wrong answers per seed in the new deceptive kinds); no data in hand can reveal it. A `--careful` mode answers only inside the range it was taught or has had checked, and the baby's self-checks (now covering rules and selections) win the coverage back: after two rounds 0 wrong and 99.2 % of the extrapolation kept. **Default is still permissive; whether careful should be the default is Daws's call** (extrapolation is the showcase, and the hazard). Tracing a third-round regression also found and fixed a library bug (a unit re-adding rewritten copies of itself, evicting others).

- **2026-10-21** Guessing like a scientist (Daws, Entry 7): careful became the default, and the baby no longer just refuses beyond what it was taught: it offers a *labeled guess* ("probably 42"), never stated as fact, scored apart. A miss ends the guessing out there (before: one correction fixed 2 of 307 neighboring wrong guesses and turned 2 into wrong *facts*; after: 0 repeats, 0 wrong facts, though it stops rather than learning the new regime), a confirmed guess extends where it may speak as fact, and it keeps and states a record of its own guessing per distance (stated rate below the realized rate everywhere: under-confident, never over). Testing that found a hole: two confirmed edges made a box-shaped "reach" vouch for an unchecked corner (4 wrong facts of 9,309); the reach is now a staircase, 0 wrong of 8,884, at a measured cost in coverage. Honest limits: simulated always-right teacher; synthetic worlds; one correction teaches it where it is wrong, not what is right. Next: the real-data bridge, then the controlled comparison (GPU to be rented by Daws).

- **2026-10-21** First contact with real text: SVAMP, 1,000 one-step arithmetic word problems somebody else wrote. The baby learned nothing at first (0 wrong, everything refused) and 159 "units" named after story words. Measuring why showed the right expressions already existed in its language but the search needed one to fit *every* lesson; it now accepts a program that fits most lessons and answers only as a labeled guess with a stated rate. When told a question's kind its guesses are 67-90 % right; recognizing the kind from the words alone is the wall (49-62 %, and a tuned bag-of-words classifier reaches 57-65 % on the same data), so about a fifth to a third of all questions get a wrong *labeled* guess while 0 are stated as wrong facts. A long battery caught a regression the short ones missed (a partial program froze a unit that used to be revisited), now fixed. Honest limits: one adversarial corpus; problems with three or four numbers still learn nothing; discovery on real text is unsolved. Next: which numbers matter, routing by compositional cues, a second (factoid) corpus.

- **2026-10-21** The controlled comparison begins (Entry 2): matched attention transformers (a plain one written in this repo, and a conventional PyTorch one tuned by a 24-setting search whose cost is counted against it), same lessons, same questions, joules measured with the CPU's own counter. The baby is never wrong on unseen questions where the transformers are wrong up to 40 %, tuned or not; it extrapolates (right every time, as a labeled guess) where they stay at 14-28 % right at every scale tried (20 to 600 units); it adds new facts with no forgetting (a cheap fine-tune forgets heavily, a safe one costs 3-64x more); teaching from scratch is 9-44x cheaper than the plain transformers and orders of magnitude cheaper than the tuned PyTorch route, and its cost per lesson is flat. Against the simple story, kept in: from 60 units on a plain transformer is more accurate than the baby on unseen questions (100.0 vs 89.7 % at 600 units, with 0 wrong), because the baby refuses about 8-11 % of unseen questions (each with a right labeled guess; it does not grow with scale: I had mistaken one lucky small world for a trend); tuning did not help the transformer; the energy ratio across sizes is not monotone (20x, 9x, 10x up to 200 units, then 32-253x at 400-1,000 units over three seeds; the baby teaches 43,023 lessons in 6 s while the transformer needs 28 minutes), and an early claim of mine that the advantage shrinks with scale was an artifact of noisy readings and is withdrawn. Mistakes recorded: the energy meter was wrong four times (a stale idle baseline that produced impossible 0.00 J readings, a CPU-share correction that under-counted, an uncorrected busy desktop, and a zero that came back when the background went quiet mid-run); all tables were re-measured on a quiet machine, with an uncertainty of about plus or minus 30 %. Honest limits: tidy rule-shaped lessons (the baby's home ground), a small search, one to three seeds. A GPU would not help the small-model comparison; it would help a pretrained-model comparison on real text.

- **2026-10-22** Why the baby refuses, and asking a teacher: I had believed it refused more as the world grows; printing every refusal showed it does not (about 8-11 % at every size; one lucky small world had fooled me). Each refusal is a question one step above the top edge or corner of what its unit was taught, and each carried a right labeled guess. Making the baby's own check questions automatic (a simulated teacher) exposed four defects in turn (probes dropped when their answer could not be said, no probes of the frontier, a starved question budget, and a reach that shrank as it learned because it leaned on a moving glue heuristic) and after fixing them about 5-7 questions per numeric unit make every unseen and every beyond-range question a stated fact at 20 and 200 units, with no wrong fact and no trap answered. What was left (2.6 % at 1,000 units) was recognition: learning in one unit shrinks the margin of small neighbors below a floor fixed at teaching time. My first fix did not work and was reverted; the second, keeping the floor at a fixed ratio to the leave-one-out margin (what an unseen question of a known unit looks like to the table), works, and brings it to 1.6 %, the weakest tail the floor is designed to leave, with no wrong answer and no trap answered. Honest limits: simulated always-right teacher, number-word worlds, tidy rule-shaped units.

- **2026-10-22** A teacher who is sometimes wrong (Daws: reinforcing from true facts, trusting new sources the way a person does): simulated teachers who are wrong 1-20 % of the time, by slips (asking again gets a fresh answer) or consistently (the same wrong answer every time). A single wrong answer used to turn an exact program into one that only guesses (a loss of up to a fifth of its coverage); an established program now survives a few contradicting lessons, as a rule already did, and an opt-in skepticism (hold an answer that contradicts a settled rule, ask again, believe it only when repeated) defends against slips almost for free: 99.9-100 % of unseen and beyond-range questions stay right up to 20 % slips, against 76.8 % for a trusting baby. Consistent errors cannot be told from the truth by asking again: 99.6 % at 5 %, 88 % at 20 %, and every wrong fact it states is a fact it was wrongly told. Honest limits: simulated errors, independent of difficulty; a source wrong about a whole kind of thing is not tested and cannot be caught from inside; one world and seed.

- **2026-10-22** After a miss, the new regime (Entry 7: guess, get it wrong, update): until now one correction only made the baby stop guessing where its rule had been wrong; it learned that it was wrong, not what was true. It now offers the numbers the contradiction itself names (the answer it was told, and how far its rule was) to one exact program search over everything it was told, so a sum that caps at 30 or wraps at 40 is rebuilt from a single correction. After one correction per kind all 157 formerly wrong guesses are right guesses (they were refused) and the 137 right ones stay (64 were refused); the long careful battery stays at 0 wrong facts and, after a last fix (a contradiction at a corner no lesson covers had been hidden inside the box of what had been seen), its stated facts on the deceptive kinds rose from 521 to 704 right of 720 (a sum that wraps at 40 went from 0 of 62 to 62 of 62); the price is about half a point of accuracy against a teacher who is consistently wrong, since a wrongly-labeled corner now looks like a real regime change. Along the way it needed three more fixes found by tracing one kind: repair before a small unit is re-decided into plain memory, keep a repaired law that has rivals and ask exactly where they differ, and my own harness had been hiding that question. Two things I got wrong: one point is consistent with many laws (a cap at 24 explains "24 + 12 = 24" as well as a reflection at 30 does), so a repaired program never states as fact what its repair lessons would vouch for, only labeled guesses; and the sometimes-wrong-teacher run caught a regression no other battery saw (240 wrong facts elsewhere, because a coincidental program rewrote a selection unit inside its taught range), fixed by requiring a repair to agree with the old rule wherever the other lessons cover, after which that run reads as before (87.9 % right, every wrong fact a repeat of one it was wrongly told). Honest limits: simulated teacher, tidy kinds, my own language of operations; a regime it cannot express in 7 pieces is still only "refuted".

- **2026-10-22** Real text, the wall was not the words: I had concluded that a story problem's kind cannot be told from its words beyond about 60 % ("a bag of words cannot see the compositional cue"). That was a fact about the counting classifiers I had tried, not about the words: a learner that corrects itself on its own mistakes reaches 78 % on the same bag of words, 80-82 % when the last few words (where a story asks its question) are marked. The baby now trains such a layer on its own lessons when its routing is measurably poor and the layer measurably helps (tidy worlds are untouched by construction, and every honesty battery reads as before). On SVAMP's two-number problems, routing to the right kind went 57 % to 76 % (random split) and 50 % to 73 % (blocks), and labeled guesses 50 % to 68 % and 38 % to 59 % right; still nothing is stated as fact and the stated hit rate stays honest in five of six splits (8 points over in the sixth). My first version trained in class order, a perceptron forgets all but the last class, and scored below the majority class: the signature of a bug, not of a weak method. Honest limits: one adversarial corpus, 340 two-number problems, about a quarter of all questions still get a wrong labeled guess, the 3-4-number problems still learn nothing.

- **2026-10-22** Which numbers matter: two thirds of the story problems hold more numbers than the baby's rules use, and until now it could not read them at all. It now learns which numbers matter from its own programs (the one pair of numbers that its unit's program turns into the given answer is a label: connecting the dots) and from one general cue, that a number counts when the words beside it also occur in the question; it keeps the best two and blanks the rest. The questions it could not touch before (0 % right) now get a right labeled guess 27-38 % of the time, still never stated as fact, and wrong 60-72 % of the time because choosing the pair is right only about half the time. The part I care about: it first stated 77 % for guesses that were right 32 %, so it now asks itself questions it was given answers for but not taught from, and scales what it claims to what it achieved; for these questions it claims 16-31 % and achieves 27-39 %, while for two-number questions its claim is within 10 points in five of six splits (over by 4-10 in three, 19 under in one). Honest limits: one adversarial corpus, answers that need three numbers are not handled, and much of this rests on labels that exist for only about 60 % of the lessons.

- **2026-10-22** A second corpus (Daws approved the download: ASDiv, CC BY-NC 4.0, used privately, never committed): on its two-number problems, with nothing retuned, the baby routes 76 % to the right kind and gives a right labeled guess 58-60 % of the time, never stated as fact: the same picture as SVAMP's. Two things I must report against myself: a re-count (the baby reads number words, my problem counter did not) changed which SVAMP problems form the two-number set and showed that the figures I gave earlier today were optimistic and, more importantly, fragile: across six splits labeled guesses are right 35-73 % of the time (mean 56 %), swinging on whether a kind gets a partial program at all (I first blamed a guess bar I set myself, then a work budget, then an ambiguity cap: each tested, none made any difference; printing the decision path found it, and a program that is right on most inputs but not reliable enough to state facts now gives labeled guesses instead of being thrown away, a modest gain: 56 to 58 %). And it was twice as slow as it had to be (a review that repeated the whole cross-validation for nothing): halved with identical output.

- **2026-10-22** Learning from being told (Entry 9: fewer mistakes by learning from true facts): when the baby is told the true answer to a question it guessed at, it now files the question where a program reproduces that answer from some of its numbers, which also says which numbers mattered, and learns the selector and the routing again from it. My first version filed every correction wherever the question routed and made things worse late in a stream (37 down to 15 % right), because lessons a unit's program cannot explain dragged it below 80 % explained and it lost its program; it now learns only what it can place (49 to 90 % of the answers). The result is small and honest: over streams of 240 to 324 held-out questions guesses right rise by 0.3 to 2.5 points and routing by up to 4, in all five streams, never wrong as fact; the errors that remain are mostly wording, not a shortage of lessons.

- **2026-10-22** A person can now answer the baby's own questions (`baby tutor`): it shows each question, why it asks and what it would say now; you type the answer. It is skeptical of you the way it is of any source: an answer that contradicts a rule it has good evidence for is held until you repeat it (a different second answer leaves it unsettled, Enter keeps its rule), and `a question => the answer` teaches it something of your own, filed where a program reproduces the answer. Until now only simulated and scripted teachers had answered it. Honest limit: no real person has yet used it beyond me piping in a few answers.

- **2026-10-22** Cost, measured: building the baby on a real-text corpus took 111 s and I had said the program search was the reason; timing each mechanism showed the program search takes one second and the rule mechanism, which fits a weight to every word and so scales with the cube of the vocabulary, took ninety percent. With more unknowns than lessons the same fit can be solved from the other side (a system the size of the lessons): the build now takes 13.7 s, the results are identical line for line, and every honesty battery reads as before. A first attempt on the part I had wrongly blamed changed nothing. Light matters here, and being wrong about where the cost is cost an hour.

- **2026-10-22** The entry exam (Entry 10: each new version is rebuilt from scratch with the old one only as reference): since code will not carry over, the judging does. `baby exam write` makes plain files (lessons, optional calibration material, questions in groups, and traps that must be refused); any system in any language answers with a predictions file (a fact, a guess with its own confidence, or a refusal); `baby exam score` judges by the project's own rules (no wrong facts, no traps answered, a guess's stated confidence at or below how often it is right) without caring how the system works. The baby's own result is the bar a successor starts from, and a naive Python memorizer is the floor. On its first run the exam found two traps the baby answered, and the cause was in my exam: it had left out the teacher's grouping of lessons, which the baby's honesty depends on. Honest limits: tidy synthetic worlds, static, no joules, no real text (the licenses forbid redistributing it).

- **2026-10-23** The exam learns to test learning: an adaptive tier gives a system a stream of questions about kinds that look like a simple rule and secretly change regime, and the exam itself runs the system as a child process over a small line protocol (teach, ask, answer, told), so any language can take part and no answer can be seen before it has been given. It is judged on wrong facts (none) and on how much better it does in the last third of the stream than in the first. The baby: no wrong facts in 787 questions, and on the rule-changing kinds right 90.6, then 93.8, then 100 percent as it is told the truth; a lookup-only Python system stays at nothing. It also costs the baby seven minutes, because every told answer makes it think its unit through again, which is the first number a successor should beat.

- **2026-10-23** Joules in the exam (Entry 7: capability per joule as the machine itself measures it). The exam now reports what teaching and what answering cost on the CPU's own counter, for the baby and for any system run through the pipe protocol. The baby teaches 2,520 lessons for 0.68 J and answers 4,832 questions for 0.28 J: lookup after learning is close to free, about 4,800 right answers per joule overall. But on the adaptive stream, learning from every told answer, it spent 1,419 J, 1.8 J a question, forty times what teaching everything cost, for half a right answer per joule. Adapting is where the baby's energy goes, and doing it without thinking the whole unit through again is the next thing to beat. Honest limits: one run each for the adaptive figures, desktop counter to about thirty percent, only the processor package.

- **2026-10-23** Beaten by 3.5 times the same day, by reading the code rather than guessing: for every answer it was told, the baby re-decided how to learn the whole unit (a cross-validation of every mechanism), twice when the answer surprised it. A rule it has already settled on now simply takes the lesson in and extends where it may speak, and thinks it through afresh every twenty-five lessons. The same stream costs 400 joules instead of 1,419 (half a joule a question instead of 1.8), with the same answers and nothing wrong stated as fact; the rule-changing kinds even came out a little better in the long careful battery (718 right of 720 against 704). That is still thousands of times what answering without learning costs, so there is more to find.

- **2026-10-23** Timed the rest of that cost stage by stage rather than guessing: sixty percent of what remained was the baby re-thinking, for every answer it was told, the units that can only memorize, which already had a rule for being looked at only every ten facts elsewhere. Following that rule, the same stream costs 268 joules (a third of a joule a question; it was 1,419 at the start of the day, 5.3 times more) with the same answers and nothing wrong stated as fact. What is left is mostly real work for real changes (a new building block means re-examining the open units). Honest limits: one run per figure, desktop counter, processor only.

- **2026-10-23** An honesty hole, found by accident and closed: asking the baby to check its own explanations with fewer questions (one per kind instead of four) made it state six wrong facts, which the usual gate, always run at four, had never shown. It was mine: when a rule was shown wrong and it rebuilt the law from the correction, the next lesson let it throw that law away for a different one that fitted the same two probes by coincidence (a cap built out of threes where the truth is a cap at thirty), and those probes then vouched for everything below them. It now keeps a rebuilt law that still explains everything it has been told. No wrong facts at one, two or four questions; the price is 34 facts of 720 on the rule-changing kinds in the standard battery, coverage a fresh fit used to find by luck. The standing gate now includes the one-question budget.

- **2026-10-23** The same lesson, a second time, from a different direction: varying the seed of the experiment with a teacher who is consistently wrong (not the budget) showed the repair step turning one wrong lesson beyond the taught range into a rebuilt law, and the lessons that merely agreed with both laws then vouched for it; on one seed that meant 68 wrong facts about things the baby had never been wrongly told, where there had been none, and a trap answered. The cure is a principle the project already had: the law that was replaced remains a competing explanation, and the new one may only state where they agree until two lessons told afterwards have separated them (and where the old one is silent, no endorsement, so nothing). No damage elsewhere in nine teacher runs now, at a cost in coverage under that teacher (about 4 to 7 points on two seeds). Also found, and not mine: label noise lets rewordings of wrongly-told facts through at about half a percent on other seeds, identical in the build before today.

- **2026-10-23** A tried idea that failed, recorded: to win back the coverage the new caution costs, I made the baby ask exactly where its rebuilt law and the law it replaced disagree. Under a consistently wrong teacher it recovered some (up to ten points), but the long careful battery's rule-changing kinds fell at every question budget, and on one seed 32 wrong facts about things never wrongly told came back, because a wrong teacher's answer can coincide with the rebuilt law at a probed point and be taken as proof. Reverted. The lesson is about what counts as evidence: a lesson can retire a rival law only if it cannot be a wrong teacher's coincidence, and nothing inside the lessons themselves can promise that.

- **2026-10-23** The exam's wrong-teacher tier (`adapt-write --wrong P --repeat F`): the adaptive stream's truths are told wrongly for a share of the facts, consistently, and the judge separates wrong facts that repeat a believed mistake from wrong facts about things never wrongly told (must be 0). Its first run found a hole at once: one wrongly told lesson that a coincidence law agreed with replaced a correct tens-digit law, and the baby then stated a wrong fact the teacher never touched (0/1/1 and 0/1/0 on three seeds). Fixed by adopting a fresh fit that explains a contradicting lesson only as a repair, keeping the replaced law as a rival (`baby/README.md`, first section). Now 0 elsewhere on all six runs; the baby still believes a teacher who repeats a wrong answer (14-39 repeats; a naive memorizer 32-59).

- **2026-10-23** The repo and the site moved to en-US, a communication protocol for all writing and code. Because it renamed identifiers and printed strings in baby, I ran the tests (142 pass) and compared the `baby school` scorecard (identical but for spelling). For Novice it is a requirement from the first line. Details in `baby/README.md` ("en-US") and the handover.

- **2026-10-23** The entry exam put to the matched transformer (`baby exam transformer`): same lessons, same judge, same meter, three seeds. The transformer fits what it was taught and then states 1,500-2,800 wrong facts and answers 43-100 % of the traps depending on whether it refuses below its own confidence; the baby states 0 and answers 0. Teaching cost is 11-24x the baby's and answering 2.2-2.7x (upper bounds on a busy machine). The adaptive tier for the transformer is the open next step.

---

## More to come

Daws has more decisions to record. Entries below, newest last.

---

## Entry 6 (2026-10-20): Open, and kept open

Daws made a public GitHub repository for the work (`DawsMacs/TransientAI`) and chose the **AGPLv3** on purpose. One internal goal is to make sure this
work **stays open for everyone, and is not hoarded or closed.** The README is to be technical rather than philosophical (the ambition may be mentioned,
not spelled out as it is in this file), with a banner built from the logo and the Lexend typeface. The assistant keeps the freedom to suggest whether
"AGPL, or version 3 and later" serves that goal better.

### Context added by the assistant (labeled; check, edit or delete freely)

- **Why a network-copyleft license fits the goal.** The GPL obliges you to share source only when you *distribute* a program. A company can run a modified
  version as a hosted service, never distribute it, and keep its changes. The AGPL closes that gap: users who interact with a modified version over a
  network are entitled to its source. For a project whose point is that capable AI should not be the property of a few providers, that is the clause that matters.
- **"Or later" was chosen (`AGPL-3.0-or-later`).** It lets the FSF's future versions apply, which is the practical way to close loopholes that do not exist yet
  (the 2007 text predates today's AI services) and keeps the code combinable with other copyleft code as licenses evolve. The cost is trusting the FSF's stewardship
  of future versions; if that trust ever changes, `-only` can be adopted from then on for new files (a license already granted cannot be taken back). Easy to revert now if Daws prefers `-only`.
- **A license binds others, not the copyright holder.** As sole copyright holder Daws could still relicense his own code. What prevents anyone, including a future
  Daws, from closing *other people's* contributions is that contributions come in under the same license with a DCO sign-off and **no CLA** (a CLA is the usual
  mechanism by which a project is later relicensed into a closed product).
- **What the AGPL does not cover.** It governs the software. It does not by itself say anything about trained or learned files (`.school`, model weights) or data
  that people produce with it; if a released learned file should also stay open, that wants its own explicit statement when we first publish one.
- **Public means public.** Anything committed, including its history, is permanently visible and may be cached or forked. Private-project notes were kept out
  of the first commit for that reason (`.local/` is git-ignored).


---

## Entry 7 (2026-10-21): Frontier, guessing like a scientist, and what is trusted

### What "frontier" means here

The ambition is to bring the level of models with trillions of parameters (proprietary ones run through APIs in large data centers, or open-weight ones such as Kimi K3, over 2 trillion parameters, about 40 billion active) to
ordinary hardware, locally. "Frontier" simply means what is in that range: **"I trust this AI in my workflow, education, coding, job, school"**, general-purpose intelligence. It is vague on purpose. It is **an ambition, not a destined goal**, and it
must not falsely lead the work elsewhere. Daws still bets on it because it could change how intelligence is distributed globally, so that disadvantaged people can have it too one day without expensive hardware.
Of the possible measures, the one that matters is **capability per unit of computation as the machine itself measures it (joules)**. Benchmarks and chat quality are trend-based and can change; they come later, optionally, if the joule measure is substantially met.

### Careful, but not timid: the baby should guess like a scientist

Careful becomes the default. But an intelligent system that cannot find patterns, predict or hypothesize would be a very smart and very restrictive system, and one that is always confident would be a chatbot that is wrong all the time.
The human parallel is the way out: scientists guess, form hypotheses, then theories, then establish laws or facts, and do it rigorously. Risk-taking sometimes pays more than being overly cautious, because restraint keeps you in the same loop; **guess, get something wrong, update: that is learning.**
The baby does not have to be perfect. It has to understand **what is true, the range of complexity, and why something is true or false**, and then, through trial and error, learn using Adaptive Learning. LLMs need extensive training or post-training to update something; **ours can update in real time, which is the biggest advantage.**
Keep that in mind in handling this.

### What is trusted, and the license

The license stays **AGPL-3.0-or-later**. Daws accepts the trust it involves (the FSF, and GitHub, and the assistant: "we have to trust many things here"). If the AGPL's nature ever changed in a way he disliked, new releases could go out under AGPL-3.0-only, and older ones remain under what they were granted.
He trusts the GNU foundation: Stallman went through a lot to invent copyleft. On git: freedom was given so that there is progress without him having to spoon-feed or sit through everything; risk is real, so **being careful, precise, planning, and asking in risky situations is still the right call.**
**The name "Transient" stays a mystery for now** ("for the lore").

### Decisions in this entry
- `--careful` becomes the default (permissive is `--permissive`).
- GPU: Daws will rent one for the controlled comparison with a small transformer (he will say when it is set up); a local iMac (macOS, AMD Radeon 4 GB, 40 GB RAM) was considered and set aside as complicated.
- The energy meter (RAPL) was made readable by Daws himself.
- GitHub social preview and topics were set by Daws.

### Context added by the assistant (labeled)

- Daws's ladder (hypothesis, theory, law) is implemented as: a labeled **guess** (a trusted rule used outside the range taught or checked), a **checked** range (a confirmed answer extends where it may speak as fact), and **taught** facts. See the 2026-10-21 section of `baby/README.md`.
- The honesty rule became "never *silently* wrong": WRONG (a fact stated and false) must stay at 0, a labeled guess may be wrong, and what is measured is whether guesses are calibrated and whether one correction fixes the next mistake.

### Context added by the assistant (labeled): the rename (2026-10-21)

Daws renamed the GitHub repository from `DawsMacs/TransientAI` to `DawsMacs/Transient`. **TransientAI is now the name of the creator of the Transient project** (Daws is its owner and founder); the project, the repo, the README and the banners are called **Transient**. The copyright and license holder is unchanged: DawsMacs, AGPL-3.0-or-later. Earlier entries above that say `DawsMacs/TransientAI` record the name at the time and are left as written; GitHub redirects the old URL.


---

## Entry 8 (2026-10-21): The license becomes AGPL-3.0-only

Daws decided to move from `AGPL-3.0-or-later` to **`AGPL-3.0-only`**, to rely less on the FSF or any institution: "or later" means every future version the FSF publishes is automatically available to every recipient, which fails open if something goes wrong there. He noted that nobody had yet viewed the repository, so it was an easy moment to change it.

### Context added by the assistant (labeled; check, edit or delete freely)

- **What "or later" does and does not do.** The license text never changes; AGPLv3 stays AGPLv3 and any copy can always be used under it. "Or later" is an *option each recipient may take*, and it attaches to old releases too: a recipient of 2026 code could choose a weaker "AGPLv4" in the future for their own copy and derivatives. That was the exposure.
- **Why `-only` rather than a proxy now.** AGPLv3 §14 lets a notice name a *proxy* whose public acceptance of a version permanently authorizes it. That fails closed (no proxy statement means v3 stays), but its benefit is adopting a future legal fix without contacting every contributor, which matters only once there are outside contributors. As sole copyright holder, Daws can add a proxy or "or later" for *new* releases at any time; tightening the other way cannot be undone for copies already distributed. So `-only` is the reversible choice.
- **Honest caveat.** Commits published before this change carried `AGPL-3.0-or-later`; anyone who already pulled them keeps that option on those copies. The change applies from this commit onward.
- **Mechanics.** SPDX headers, `Cargo.toml`, `REUSE.toml`, README, CONTRIBUTING, CLAUDE.md and the handover now say `AGPL-3.0-only`; `LICENSES/AGPL-3.0-or-later.txt` was renamed `AGPL-3.0-only.txt`. The `LICENSE` file is the unmodified AGPLv3 text, whose generic "any later version" wording is part of the license text itself and is left as is. Earlier entries above that say `-or-later` record the decision at the time and are left as written. The website text was updated to match.

---

## Entry 9 (2026-10-22): Reinforcing from true facts, and learning to make fewer mistakes

On the baby learning from a teacher who is sometimes wrong: this is the AL idea, connecting the dots and reinforcing from true facts, **similar to how a human learns and trusts new sources**. The baby does not have to get it right the first time. As AL improves it gets better at learning, so it makes fewer mistakes, rather than needing to be right at once: even models get things wrong during training, and they need to be retrained or go through some feedback process. Skepticism of a source that contradicts what the baby has good evidence for (hold it, ask again, believe it when it is repeated) is to be **on by default**, on the assistant's own recommendation.

### Context added by the assistant (labeled)

- What was built from this: an established exact program survives a few contradicting lessons (one lesson against dozens that agree is an outlier, not a new law), and answers that contradict a settled rule are held for a second opinion. With slips up to 20 % the baby keeps 99.9-100 % of its unseen and beyond-range answers right; against a source that is consistently wrong about particular things it keeps 99.6 % at 5 % and 88 % at 20 %, and every wrong fact it states is a fact it was wrongly told. What it cannot do from inside is catch a source that is wrong about a whole kind of thing: all of that source's lessons would agree with each other. Numbers and failures: first section of `baby/README.md`.


---

## Entry 10 (2026-10-22): Genuine AL, and Novice, the successor to baby

Daws, on where baby and its Adaptive Learning go (his words, lightly tidied):

We will keep working on baby for quite a while, and I want baby and its AL to improve. As we improve in that field, **AL should become more genuine than a mere trick**: real invention, dynamic architecture, internal rearrangements, efficiency alterations, real-time learning and adaptation, and so on.

There will also be a future successor to baby, which I am naming **Novice**. For every new version we will **rebuild and redesign from scratch and use the previous work only as reference**. This is to make sure we reinvent and genuinely improve rather than build on the inertia of what exists. Our systems should get lighter, faster, more efficient, more intelligent, more capable, more dynamic, with deeper AL.

This is a long-term direction for the assistant to consider; the assistant is still driving the project with the freedom it was given. Also: tell Daws when it is time to rent a GPU.

### Context added by the assistant (labeled; check, edit or delete freely)

- **How I read "genuine, not a trick".** Today the baby's adaptation is real but chosen by me: it picks among mechanisms I wrote (remember, rule, select, program, network), with thresholds I set, and its "invention" is a search inside a language of eight operations I defined. The repair step (2026-10-22) is a small example of the gap: it can rebuild `min(a+b, 30)` only because the language has `min` and the contradiction names a 30. "More genuine" would mean the baby changing *its own language and its own structure* when the old one fails: new operations, new kinds of mechanism, its own thresholds set from measurement, units split, merged and re-routed on their own evidence, and the cost of each such change measured in joules. The test of genuineness I will hold myself to: **a capability the baby gets that no line of the code names in advance**, shown on a task I did not design it for.
- **How I read "Novice".** Baby stays alive as the reference and the measuring stick; Novice is a clean-sheet design that may share nothing with it but the goals and the benchmarks. The honest consequence is that baby's batteries (the invention battery, the scale test, the teacher experiments, the controlled comparison) become Novice's **entry exam**: a successor is worth building when it passes what baby passes at lower cost, and does something baby cannot. I will keep those tests runnable and independent of baby's internals (the `--export` TSVs and `--pairs` loader already point that way) so they can examine a system with a different design.
- **When a Novice starts is a judgment, not a schedule.** I will say so when baby's design has stopped yielding (a ceiling I can name from measurements, not a feeling), and propose what Novice should be rebuilt around, rather than carry baby's structure over by habit. Until then baby is where the learning about AL happens, and its failures (the handover's lists) are the design notes for Novice.
- **GPU:** not needed yet. The first stage that wants one is the pretrained-model comparison on real text (a 0.5-8B model on the SVAMP bridge and a second corpus); I will say so before that stage starts.
