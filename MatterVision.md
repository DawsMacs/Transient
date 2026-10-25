# MatterVision

**Internal document.** It records why Matter exists, what has been decided, and how each idea is meant to be tested. It is a working
document for Daws and the assistant, not a public page, and nothing on the public site should depend on it. The public record of the
work is `README.md`, `docs/Direction.md` and `baby/README.md`; the state for resuming work is `MatterHandover.md`.

Started 2026-10-19. **Rewritten 2026-10-24 at Daws's request:** the wording was made practical and tied to what has been measured. Decisions, dates and
entry numbers are unchanged (other files cite "Entry 7" and so on). Earlier labels coined for the architecture and for optimizing it were retired
and are not used; **Adaptive Learning** is the one project term kept. Entries are dated decisions. Where an entry also holds the assistant's
context, it says so.

**How to read a claim here.** Each idea carries one of four labels: **Decision** (taken by Daws, not reopened without him), **Hypothesis** (believed,
not shown), **Measured** (a number from a run, with its caveat in `baby/README.md`), **Open** (not started or not resolved).

---

## Where each idea stands (2026-10-24)

| idea | label | where it stands |
|---|---|---|
| A differently designed system can reach useful capability on ordinary hardware (the founding hypothesis) | Hypothesis | Untested at scale. Everything built so far is a small system on tidy, synthetic lessons, plus two real-text corpora where it only gives labeled guesses. |
| Capability per joule, measured by the machine, is the figure of merit | Decision | In use: RAPL counter on Linux; the entry exam reports joules for teaching, answering and adapting. |
| Adaptive Learning: learning continues from use, and the way of learning adapts too | Decision, partly built | Level 1 (knowledge) works; level 2 (when and how much) is partly measured instead of fixed; level 3 (the mechanism itself) works only inside a language the assistant wrote. See Entry 5. |
| Capability may respond steeply to small increases in capacity, if the design is dense in what it holds | Hypothesis | Untested. Standard scaling results are smooth, so a steep response would stand out; this needs the controlled comparison at several sizes. |
| A unit that starts small and grows capacity only when it saturates | Open | Not started: baby's capacity is fixed up front. |
| Representation need not be tokens | Decision, partly explored | The letters reader beat the word reader on misspellings (Entry 4); answers still come from a fixed list. |
| Controlled comparison against a matched transformer | Measured in part | Plain Rust transformer and a tuned PyTorch one on the same lessons and exam, at several sizes. A pretrained-model comparison on real text is not done and needs a rented GPU. |
| "Frontier-class" capability on consumer hardware | Hypothesis, ambition only | Not a target to be claimed; no result is anywhere near it. Judged by capability per joule first, benchmarks later (Entry 7). |
| Successor system, Novice, rebuilt from scratch | Decision | Not started; waits for a measured ceiling in baby (Entry 10). |

---

## Entry 1 (2026-10-19): The founding hypothesis

**Decision.** Matter pursues one hypothesis, for as long as it takes and whether or not it holds: **a large share of what it costs to run capable AI comes
from inherited design choices and from software, not from physics, so a differently designed system could reach useful capability on ordinary hardware.**
Daws is explicit that this is a hunch resting on judgment, not an established result. Most current research bets on smaller gains from scale. Matter goes the other way
because the question is worth answering by measurement, and a negative answer redirects the work instead of ending it.

**The reasoning behind the hunch (Daws's, restated).**
1. Computing keeps shrinking: machines that filled rooms now fit in a pocket, and a phone outperforms the Apollo-era computers.
2. Hardware will keep improving, but some limits are close (physical and mathematical), and no second jump on the scale of the single-chip processor is in sight.
3. Software has had less attention than hardware. This is a judgment, not something measured here.
4. Most deployed models descend from one design, the 2017 transformer. Recent work mostly refines and scales it; little large-scale work starts from a blank page.
5. The industry committed to scale, with GPU demand and energy use to match. Models were mostly scaled, trained and refined, not redesigned.
6. So a from-scratch design might find efficiency gains of a different order than tuning does. Daws's image is an iceberg: most of the possible gain lies below what current practice looks at.

**Evidence that points the same way** (assistant's context; sources cited where they exist, figures recalled from them and worth rechecking before quoting externally).
- OpenAI's "AI and Efficiency" study: the compute to reach AlexNet-level ImageNet accuracy fell 44x between 2012 and 2019 (algorithmic efficiency doubling about every 16 months), against about 11x from Moore's Law
  over the same stretch ([source](https://openai.com/index/ai-and-efficiency/)).
- The nanoGPT speedrun cut GPT-2-small training from 45 minutes to about 2 min 20 s on the same hardware class by reworking the optimizer and architecture ([source](https://github.com/KellerJordan/modded-nanogpt)).
- The size of model needed for a given quality has been roughly halving every 3.5 months ([source](https://alphaxiv.org/abs/2412.04315)).
- **Measured here:** answering a question in baby costs 59 uJ and adapting to one told answer costs 36 mJ (RAPL, upper bounds on a desktop). Early counted-operation estimates said most of the energy was moving memory, but those estimates later
  proved about 100x below the measurements, so that *share* is unverified. One change in how a dot product was written made it 2.5x faster with identical results, which is the kind of implementation gain the hypothesis predicts.
- The transformer's inheritance is real: mixture-of-experts, grouped-query attention, FlashAttention and rotary embeddings refine the 2017 design. State-space models and RWKV exist but have not displaced it at the frontier.

**Evidence and terrain that cut the other way** (design constraints, not reasons to stop).
- Hardware and software co-evolved: transformers won partly because they map well onto GPUs. A new design must be efficient on real chips, not only on paper, and may need new kernels.
- Ideas that look excellent at small scale often fade at large scale. The scaling curve is the test, not one demo.
- The gain could be real but narrower than hoped (large for lookup and routing, smaller for open-ended reasoning). Finding out which parts is part of the work.

**What the hypothesis predicts, so it can be tested one piece at a time.**
1. Some design reaches a given quality with **orders of magnitude less energy per answer** than a dense transformer.
2. The gain comes mostly from **software and representation**, not new silicon.
3. That design runs on **ordinary low-end hardware**: a phone, an old laptop.

These are signals, not pass/fail gates: how far Matter's results sit from the current quality-per-joule frontier; whether independent small designs keep finding large efficiency jumps; whether jumps survive more scale. Each says where to dig next.

---

## Entry 2 (2026-10-19): A small working unit, Adaptive Learning, and how to test it

### The shape of the solution (Decision: stay flexible)
Probably not one heavy model but a **small working unit** that does real work and can grow. baby is not the claimed breakthrough; it is a first probe of the kind of thing worth building when the conventional route is not assumed.
The unit could end up a model, a system, or a set of units. The assistant has freedom to explore and to change approach, and is asked to say when it has its own candidate.

### Adaptive Learning (the project's term)
Learning that goes past conventional training:
- It **continues from interaction** instead of stopping at a fixed training run, building abstractions from what it meets. Early on a teacher stays in the loop; the aim is that the learning looks nothing like a conventional dataset run.
- It **spends effort where something is new**: a lesson it can already answer is not learned from or stored, so it does not absorb redundancy.
- Its **internals change while it runs**, which blurs the line between a provider training a model and a user using one.
- It may **build firmer foundations first** (arithmetic, physics) and treat reliability as depending on how well-defined a domain is: arithmetic can be exact, interpretive text leaves more room for disagreement. This is measurable as calibration by domain.
- **The learning process itself adapts** (Entry 5).

### Density and growth (Hypothesis)
If a design holds more capability per byte, parameter or joule, a small increase in capacity might buy a disproportionate gain in capability. Standard scaling studies find smooth power laws, so a steep response would stand out, which makes this a sharp
prediction to test and not an assumption. Related phenomena exist (sudden generalization after long training, "grokking"; abilities that switch on at a size, though whether those are real or an artifact of the metric is debated). None of it supports the claim; it only says where to look.
The working picture is a **starting unit** that is tiny, and a **learned state** that accumulates what it learned, which can be scaled up when it saturates. A concrete cousin is function-preserving growth (widen or deepen a network while keeping what it computes; the Net2Net idea, from the assistant's recollection of Chen et al. 2015).
**Status: Open, not started.**

### Operational measure for "more capability in the same space"
**Capability per unit of resource**, where capability is a score on a fixed suite and the resource is parameters, bytes or estimated joules, reported as separate numbers (not one blended figure, because "capability" has subjective parts). The suite is a shared design decision. In practice the entry exam (Entry 10) is that suite.

### How to proceed (Decision)
- Flexible, not dogma. The hypothesis is a direction, not a rule.
- Run several experiments with different designs. Units are tiny, so each is cheap.
- **Controlled comparison at small scale:** a matched small transformer against a candidate, holding fixed parameters, bytes of weights, the *energy learning spent* (the ledger counts it) and data seen. Compare at **three or more sizes** each, since slopes extrapolate and a single size can mislead: quality at equal joules, retention after new learning, adaptation speed to new facts, joules per answer, and how quality changes with a little more scale. Log every configuration so the comparison cannot drift toward the idea we like.
  **Status: Measured in part** (Progress log, 2026-10-21 and 2026-10-23).

### Where this meets existing research (assistant's context)
- Learning from interaction instead of a dataset is close to continual and lifelong learning, curiosity- and surprise-driven learning, and open-ended learning. Google's Titans update memory when a surprise signal is high; continuum-memory work updates parts at different speeds (`LEARNING.md` has the sources). Adaptive Learning goes further by also choosing what to look at.
- "Internals change in real time" is the territory of fast weights, test-time training and sparse updates that touch few parameters to avoid forgetting. baby's `life` experiment showed the energy side: skipping lessons it can already answer meant only a third to a half of the lessons needed learning at all.
- Reliability varying by domain maps onto calibration. baby's confidence already tracks how well-defined a question is.

### Candidate working units (the assistant's picks, with status)
1. **Two-speed unit (episodic store, slow network, sleep).** Built in the first week; reached retrain-level retention at about a fifth of the energy. Later work moved most of the load to mechanisms chosen per unit (Entry 5).
2. **Surprise-gated learner:** learn only when prediction error is high. Built (the surprise gate); skips what the unit can already answer.
3. **Growing unit:** start tiny, add capacity only on saturation, function-preserving. Not started.
4. **Loop-with-memory unit:** a small block reused for a variable number of steps plus a store (looped transformers and tiny recursive models point here). Not tried.

### Design constraints for autonomous learning
Learning from browsing means learning from unvetted text. That needs provenance (where each lesson came from), a way to unlearn or quarantine, and a rule for how much a single source can change the unit. A teacher stays in the loop early on. These are the controls that let the autonomy be turned up gradually. Sources wrong about a whole kind of thing cannot be caught from inside the lessons, so per-source track records belong here (handover, next steps item 4).

---

## Entry 3 (2026-10-19): The goal, stated plainly

By any means: **run strong AI on typical consumer hardware without overwhelming the device**, in RAM, storage or otherwise. The architecture that does this is likely different from today's and more effective with Adaptive Learning. The assistant may choose and change approaches, models and units; Entry 2's candidates are starting points, not a plan to follow.

### "Not overwhelming the device" (working reading, to be corrected)
The device stays usable while Matter runs. It is a guest, not a tenant. The constraints are measured, not felt:
- **Resident RAM** (what must stay loaded) and **peak RAM** during learning.
- **Storage footprint**, and how much of it is read sparsely (cheap) versus loaded whole (expensive).
- **Memory bandwidth** per answer and per lesson, because data movement is expected to dominate energy (to be confirmed by measurement; see Entry 1).
- **Energy and heat:** joules per answer and per lesson; sustained temperature on a phone.
- **Background behavior:** nothing that slows the device while the user is doing something else; learning is scheduled for idle time.

Reference envelope (every number negotiable, to appear next to results): about **1 GB resident RAM** and a few GB of storage for the learned unit, on an 8-12 GB phone or laptop. Daws's S24 Ultra and the 7.5 GB development laptop are the real test devices. Bigger stores are allowed on disk because disk is read sparsely. A 2-4 GB phone is the stretch tier.

---

## Entry 4 (2026-10-19): Tokens are one choice among several

The deviation from convention may go deeper than architecture: the units the model reads need not be the tokens the industry normalized. The goal is a route that takes nothing for granted, keeps what the industry learned only while it earns its place in a measured comparison, and replaces it when something better shows up. Daws does not oppose using what the field has learned.

**Absolute freedom** is given to the assistant under the stated goals: strong capability on ordinary hardware, light on RAM and memory, built around Adaptive Learning, in Rust, pursued whether or not it succeeds, honest about evidence.

**What this meant in practice (assistant's reading).** Words are one choice. baby reads whole words from a fixed vocabulary, a small version of the token habit. Alternatives: letters or hashed letter-groups (open vocabulary, no tokenizer, typo-tolerant), or chunks the system discovers itself. **Measured:** the spelling-based reader generalized to misspellings it never met (37% against about 11% on misspelled key words; 61% with noisy training), and the choice of reader mattered more than any learning trick. Borrowed pieces (embeddings, softmax, Adam) stay only while they earn their place.

---

## Entry 5 (2026-10-19): The learning process itself adapts

The point of Adaptive Learning is not only that the system adapts to *what* it learns: **how it learns also changes**, and that is the dynamic part. Three levels, each deeper than the last:

| level | what adapts | status (2026-10-24) |
|---|---|---|
| 1. Knowledge | new facts, words, corrections | Works in real time. One new lesson costs about half a millisecond, flat from 20 to 600 units. |
| 2. How much and when | steps per lesson, when to sleep, what to ask, how far to trust a match | Partly. The reliability bar is set from the noise the system measures in its own lessons and its boldness from its own track record; several constants (folds, outlier allowance, search budget) are still the assistant's. |
| 3. The mechanism | try different ways to learn each kind of thing and keep whichever generalizes on held-back evidence | Works **inside a language the assistant wrote** (remember, rule, select, program, network; an expression search of seven pieces). It invents expressions and vocabulary, not new kinds of learner. |

No single mechanism wins everywhere: the fact store wins on facts, the network on rewordings after consolidation, programs on rules. A system that can pick, and justify the pick from its own evidence, is closer to the goal than one fixed learner. Entry 10 sets the bar for going further.

---

## Entry 6 (2026-10-20): Open, and kept open

Daws made a public repository and chose the **AGPL** on purpose. One goal is that this work **stays open for everyone and is not hoarded or closed**. The README is technical rather than philosophical (the ambition may be mentioned, not spelled out), with a banner built from the logo in the Lexend typeface.

**Context (assistant, labeled):**
- The GPL obliges sharing source only when a program is *distributed*. A company could run a modified version as a hosted service and keep its changes. The AGPL closes that gap: users interacting over a network are entitled to the source. For a project whose point is that capable AI should not be the property of a few providers, that is the clause that matters.
- A license binds others, not the copyright holder. What stops anyone from closing *other people's* contributions is that contributions arrive under the same license with a DCO sign-off and **no CLA** (the usual route by which projects are later relicensed into closed products).
- The AGPL governs the software. It does not by itself cover trained or learned files or data people produce with it; if a released learned file should also stay open, that needs its own statement when one is first published.
- Public means public: anything committed, history included, is permanently visible and may be cached or forked. Private notes were kept out for that reason (`.local/` is git-ignored).
- "Or later" was chosen first; Entry 8 reversed it.

---

## Entry 7 (2026-10-21): What "frontier" means, guessing like a scientist, and what is trusted

### "Frontier" (Decision: an ambition, not a destination)
The ambition is to bring the *capability level* of very large models (trillions of parameters, run in data centers or as open weights; Daws cited Kimi K3, over 2 trillion parameters with about 40 billion active, a figure not checked here) to ordinary hardware, locally.
"Frontier" here means: **"I trust this AI in my workflow, education, coding, job or school"**: general-purpose capability. It is vague on purpose, it is an ambition and not a destined goal, and it must not mislead the work. Daws bets on it anyway
because it could change how intelligence is distributed, so that people without expensive hardware can have it too.
Of the possible measures, the one that matters is **capability per unit of computation as the machine measures it (joules)**. Benchmarks and chat quality are trend-based and come later, optionally, if the joule measure is substantially met.
No current result is near this ambition, and public text should say so.

### Careful, but not timid: guess like a scientist (Decision)
Careful is the default. But a system that cannot find patterns, predict or hypothesize is very restrictive, and one that is always confident is wrong all the time. Scientists guess, form hypotheses, then theories, then establish laws or facts, rigorously. Restraint keeps you in the same loop; **guess, get something wrong, update: that is learning.**
The system does not have to be perfect. It has to understand what is true, the range of complexity and why, and then learn through trial and error. Today's models need extensive training or post-training to update; **ours can update in real time, which is the main advantage to protect.**

Implemented as a ladder: a labeled **guess** (a trusted rule used beyond what was taught or checked), a **checked** range (a confirmed answer extends where it may speak as fact), and **taught** facts. The honesty rule became "never *silently* wrong": WRONG (a stated fact that is false) must stay at 0; a labeled guess may be wrong; what is measured is whether guesses are calibrated and whether one correction fixes the next mistake.

### What is trusted, and the license
The license stays AGPL (then still "or later"; Entry 8). Daws accepts the trust involved (the FSF, GitHub, the assistant). He values copyleft's origin. On git: freedom was given so that progress does not need hand-holding, but risk is real, so being careful, precise, planning and asking in risky situations remains the right call. The meaning of the name "Matter" is Daws's to tell, "for the lore".

### Decisions in this entry
- `--careful` is the default (permissive is `--permissive`).
- GPU: Daws will rent one for the pretrained-model comparison and says when it is ready. A local iMac (macOS, AMD Radeon 4 GB, 40 GB RAM) was considered and set aside.
- The energy meter (RAPL) was made readable by Daws himself.
- GitHub social preview and topics were set by Daws.

### Rename (2026-10-21)
On 2026-10-21 the repository was renamed so that the project and its creator had separate names: the project, repo, README and banners carried the project's name, and the creator's name stayed with the organization (Daws is its owner and founder). Copyright and license holder unchanged: DawsMacs. Both names were changed again on 2026-10-24 (Entry 11); earlier text that names the old repository records the name at the time, and where that text survives it is written without the old names.

---

## Entry 8 (2026-10-21): The license becomes AGPL-3.0-only

Daws moved from `AGPL-3.0-or-later` to **`AGPL-3.0-only`** to rely less on any single institution: "or later" makes every future FSF version available to every recipient automatically, which fails open if something goes wrong there. Nobody had viewed the repository yet, so it was an easy moment.

**Context (assistant, labeled):**
- The license text never changes. "Or later" is an option each recipient may take, and it attaches to old releases too: a recipient of 2026 code could someday choose a weaker "AGPLv4" for their own copy. That was the exposure.
- `-only` rather than a named proxy now. AGPLv3 §14 allows a notice to name a proxy whose public acceptance of a version permanently authorizes it; it fails closed, but its benefit (adopting a legal fix without contacting every contributor) matters only with outside contributors. As sole copyright holder Daws can add a proxy or "or later" for new releases at any time; tightening the other way cannot be undone for copies already distributed. So `-only` is the reversible choice.
- Caveat: commits before the change carried `or-later`; anyone who already pulled them keeps that option on those copies.
- Mechanics: SPDX headers, `Cargo.toml`, `REUSE.toml`, README, CONTRIBUTING, CLAUDE.md and the handover say `AGPL-3.0-only`; `LICENSES/AGPL-3.0-or-later.txt` became `AGPL-3.0-only.txt`. `LICENSE` is the unmodified AGPLv3 text, whose generic "any later version" wording is part of the license itself.

---

## Entry 9 (2026-10-22): Reinforcing from true facts, and making fewer mistakes

Adaptive Learning includes connecting the dots and reinforcing from true facts, **similar to how a person learns and decides which sources to trust**. The system does not have to be right the first time. As it learns better it makes fewer mistakes; models also get things wrong during training and need feedback or retraining.
Skepticism toward a source that contradicts what the system has good evidence for (hold the answer, ask again, believe it when repeated) is **on by default**, on the assistant's recommendation.

**What was built (assistant, labeled).** An established exact program survives a few contradicting lessons (one lesson against dozens that agree is an outlier, not a new law), and answers that contradict a settled rule are held for a second opinion.
**Measured:** with slips up to 20% baby keeps 99.9-100% of unseen and beyond-range answers right; against a source that is consistently wrong about particular things it keeps 99.6% at 5% and 88% at 20%, and every wrong fact it states is a fact it was wrongly told.
**Limit:** a source wrong about a whole kind of thing cannot be caught from inside, because all its lessons agree. Numbers and failures: first section of `baby/README.md`.

---

## Entry 10 (2026-10-22): Genuine Adaptive Learning, and Novice

**Direction (Decision).** baby and its Adaptive Learning keep improving for a good while, and AL should become *genuine* and not a trick: real invention, dynamic architecture, internal rearrangement, efficiency changes, real-time learning and adaptation.
A future successor is named **Novice**. Every new version is **rebuilt and redesigned from scratch with the previous work used only as reference**, so the work reinvents instead of inheriting inertia. Systems should get lighter, faster, more efficient, more capable, more dynamic, with deeper AL.
Long-term direction for the assistant to weigh. Tell Daws when it is time to rent the GPU.

**Reading (assistant, labeled).**
- *Genuine, not a trick.* Today's adaptation is real but chosen by the assistant: it picks among mechanisms written for it, with thresholds set for it, and its invention is a search in a small expression language. The repair step shows the gap: it can rebuild `min(a+b, 30)` only because the language has `min` and the contradiction names a 30.
  More genuine means the system changing its own language and structure when the old one fails: new operations, new kinds of mechanism, thresholds set from measurement, units split and merged on their own evidence, with each change's cost in joules.
  **The test: a capability the system gets that no line of code names in advance, shown on a task it was not designed for.**
- *Novice.* baby stays as the reference and measuring stick. Novice may share nothing with it but goals and benchmarks. baby's batteries therefore become Novice's **entry exam** (`baby exam`, independent of baby's internals, any language through a line protocol): a successor is worth building when it passes what baby passes at lower cost and does something baby cannot.
- *When.* Starting Novice is a judgment, not a schedule. The assistant says so when baby stops yielding (a ceiling named from measurements), and proposes what to rebuild around. The handover's lists of failures are the design notes. The ceiling named so far: hand-built mechanisms and thresholds, the wording limit on real text, and adapting at roughly 600 times the cost of answering.
- *GPU.* Not needed yet. The first stage that wants one is the pretrained-model comparison on real text (a 0.5-8B model on SVAMP and a second corpus). The assistant says so before that stage starts.

---

## Entry 11 (2026-10-24): The project and its creator are renamed

**Decision (Daws).** After a possible conflict with a registered trademark (unconfirmed: nothing here claims that the old names infringe anything, and this repository makes no legal claim), the project **Transient** is now **Matter**, and its creator **TransientAI** is now the **Matterworm Foundation**. `DawsMacs` (Daws's account, owner, founder and copyright holder) is unchanged. This entry and the dated rename notes in `MatterHandover.md` are the only places that write the two earlier names; they are retired everywhere else, as the three coined labels were (Entry 10 and `CLAUDE.md`). The domain that carried the old creator name (`transientai.org`) is dropped: Daws postponed buying a new one, and the site is served from `https://matterworm.github.io` (GitHub Pages of the `Matterworm` organization) for now. The repositories live in that organization: `Matterworm/Matter` and `Matterworm/matterworm.github.io`.

**What changed (assistant, same day, on the local branch `rename`, approved by Daws).** Every mention in this repository and in the site repository; the banners, the social preview and the site's share image (re-drawn in the same typeface); the two documents `TransientHandover.md` and `TransientVision.md`, now `MatterHandover.md` and `MatterVision.md`; the repository URLs in the docs and in `baby/Cargo.toml` and `REUSE.toml` (they point at `Matterworm/Matter`); on the site, the absolute URLs (canonical links, share tags, sitemap, feed, `robots.txt`) use `https://matterworm.github.io` and the `CNAME` file is removed. The three blog posts were edited to the new names and carry a note saying so.

**What did not change.** The data folder `~/Matterworm/Data` (a local name). Daws later moved the local clones to `~/Matterworm/Matter` and `~/Matterworm/Website`. The git history, merged pull request titles and CI run names keep the old names for good. No measurement, threshold or recorded bar moved.

**Context (assistant).** The rule that the meaning of the project's name is Daws's to tell stays in force for "Matter". One thing for Daws to check, not a finding: "Matter" is also the name of a widely used smart-home connectivity standard, so a clearance search for the new name is as worthwhile as it was for the old one (not legal advice).

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

- **2026-10-20** Published (Daws): the work now lives in a public GitHub repo (under the repository's first name, later renamed; see Entries 7 and 11) under AGPL-3.0-or-later, with a technical README, a banner, SPDX headers, a DCO instead of a CLA, and CI that fails if the scorecard's WRONG column rises above 0.0 %. See Entry 6.

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

- **2026-10-23** The adaptive exam put to the matched transformer (`baby exam adapt-transformer`): told each truth as the stream goes on, the transformer does not get better (frozen 19 % right, 81 % wrong facts; fine-tuned 11-16 %, no setting tried beat frozen), while the baby holds 0 wrong facts and 64 % facts right plus 24 % right labeled guesses. The measured weak side is the baby's own: adapting costs 0.22 J per told answer against 1.6-34 mJ for fine-tuning (6.5-140x more), 174-183 J for the 787-question stream. That is the number a cheaper, more genuine Adaptive Learning has to beat.

- **2026-10-23** The cost of adapting cut 3.7x with the same answers: stage timers showed every one of 56 periodic re-decisions ended where it started, so a settled exact program that still explains every lesson is no longer refitted or re-decided. The 787-question adaptive stream went from 175-179 J to 48 J (61 mJ per told answer; was 0.22 J), the whole run from 210-217 J to 81 J, replies byte-identical on five streams and the whole battery unchanged. Still ~2x the transformer's replay fine-tuning per told answer, and the transformer does not learn from being told.

- **2026-10-23** The cost of adapting cut a second time (36 mJ per told answer, 28.5 J for the 787-question stream, 6-7x below the start of the day; teaching + stream 65 J): the same unexplainable units were being re-decided twice per told answer. One more cut I tried (re-deciding memorize-only units less often) was byte-identical on every clean stream and the whole battery but changed two answers on a wrong-teacher stream, one of them a new wrong fact, so it was dropped.

- **2026-10-24** Matter on other platforms, measured instead of assumed (a CI matrix of Linux x86_64 and aarch64, musl, macOS arm64 and Windows, plus a fingerprint and an accuracy audit of each platform's math library, plus golden answers): the math libraries differ by one last-bit step on under 1 % of inputs, and on all five platforms the exam predictions, the adaptive replies and the scorecard are byte-identical to the Linux reference. Say "CI-tested", not "supported"; joules remain Linux with a readable RAPL counter. The open questions (deterministic in-repo math, a hash-order fix, WASI as reference) are listed in `baby/README.md`.

- **2026-10-24** More machines, and the checks that keep them honest: WebAssembly under Wasmtime (no change to the logic), and 32-bit x86, s390x (big-endian) and riscv64, all give answers byte-identical to the Linux reference, so CI now has thirteen platform rows; a difference on the reference row fails the build, and the answers under a wrong teacher are re-checked weekly. DragonFly BSD was tried and dropped: its tests pass but its packaged Rust crashes the program at startup. The first guide to getting and running it (`docs/Running.md`) was written, and the site's documentation updated.

- **2026-10-24** Renamed (Entry 11): the project is now Matter and its creator the Matterworm Foundation, after a possible trademark conflict (unconfirmed). Done on the local branch `rename` in both repositories, with the banners, social preview and share image redrawn; awaiting Daws's approval. Still open: the site repository's move into the organization (named `matterworm.github.io`), the pushes and pull requests, and a custom domain whenever Daws buys one.

- **2026-10-25** Believed mistakes under a wrong teacher cut by half or more, and the reason was partly my own harness. The baby has been skeptical of its teacher since 2026-10-22 (an answer that contradicts a settled rule is held until repeated), but the adaptive exam's driver used a call that repeats every held answer on the baby's behalf, so every wrongly told fact was believed at once. Using the hold plainly cut the repeats (41 -> 10 at 20 % wrong) but also stopped the baby learning a rule that really changes (the rule-changing kinds fell from 90.6 -> 93.8 -> 100 % to 71.9 -> 68.8 -> 50.0 % right): I almost made that the default and reverted when I read the per-kind table. The fix that kept both: three or more different held answers that one exact program explains together with all the lessons are believed together (isolated slips do not cohere; a real change does). Result on 14 streams (clean, 5-40 % wrong): clean facts right unchanged within 0.4 points, rule-changing kinds climb again (a little slower), repeats 20/14/10 -> 5/7/6 (10 %), 41/25/28 -> 10/10/18 (20 %), 105/92 -> 20/38 (40 %). What is left is almost all in kinds with no law to check an answer against (17 of 18 on one stream): that needs a second independent source and a source tag in the exam protocol. This is the baby's own policy (Hypothesis -> Measured on these streams; not Daws's decision), changes the adaptive exam's bar for the baby, and regenerated the goldens. Open: one wrong fact elsewhere at 40 % (also in the old default), and an unexplained drift of the old default on seed 1 (20 and 41 repeats where the notes said 14 and 39).

