# baby

Transient's first working unit, written from scratch in Rust with **zero dependencies**. It started on 2026-10-15 as a tiny network
that read a short query and said one word, and has become something else: a baby that is **taught units of lessons, works out for
itself how to learn each one, answers by the way that worked, says how it knew, and refuses when it should**. The first network is
still here (`baby train`), because every later idea was tested against it.

## Try the baby (the unified one)

```
cd ~/Transient
cargo build --release --offline
./target/release/baby school                   # teach it: 13 units of lessons, about 12 seconds; prints how it learned each one
./target/release/baby ask "what is two times four"
./target/release/baby ask "how far apart are two and nine"
./target/release/baby ask "the capital of peru"
./target/release/baby ask "what is the code for five"      # it was never taught this one: it says so
./target/release/baby teach "capital of atlantis" poseidonis   # tell it something new; it keeps it in a file you can read
./target/release/baby learn "what is two multiplied by three" six --unit times   # keep learning AFTER school (a new phrasing, or a whole new unit)
./target/release/baby effort                   # what answering costs: the slow way vs the baby's way
./target/release/baby cost                     # what LEARNING costs: time and memory for teaching it, a new lesson, a new kind of lesson
./target/release/baby storage                  # what it keeps: lessons vs what it learned from them, per unit
./target/release/baby sleep --compact          # forget what is already explained (about half the file), keep every promise
./target/release/baby grade                    # re-run the honest scorecard on a saved school
./target/release/baby discover --out runs/discovered.school   # hand it a heap of UNLABELLED lessons; it finds the units itself
./target/release/baby ask --school runs/discovered.school "three minus five"   # ask the baby that organised itself
```

Every answer comes with how the baby knew (`worked out by an expression I built, (first x second), 100% on facts I held back` /
`recalled, not worked out`), and every refusal with why (`I haven't met the word...`, `I only memorise this kind of thing and
nothing I was taught is close enough`, `no kind of lesson I was taught is like this question`).

The older experiments, each of which has its own section below:

```
cargo test --offline                 # 61 tests, including numerical checks of the hand-written backprop
./target/release/baby kinds          # the nine-unit mechanism comparison behind the school (about 30 s)
./target/release/baby life           # five ways of learning, charged in estimated joules
./target/release/baby curious        # a baby that chooses what to ask a teacher
./target/release/baby rules          # sums as a rule, not a table
./target/release/baby know           # does it know what it doesn't know? three refusal signals compared
./target/release/baby train          # the original single network (--reader concat|bag|bigram|letters)
./target/release/baby energy         # what one question and one learning step cost, counted
```

## The controlled comparison (Vision Entry 2): the baby against a matched attention transformer (2026-10-21)

`baby compare` (`compare.rs`, `transformer.rs`). The transformer is written from scratch in this repo (no dependencies): pre-LayerNorm encoder, token and position embeddings, multi-head self-attention, a ReLU feed-forward layer, mean-pool, a linear layer over the answer words, trained with Adam; its backward pass is checked against finite differences (a test). It reads the same word lists as the baby, says one answer word, is trained on the **same lessons**, and is graded on the **same four question sets**; its vocabulary and answers come only from the training lessons, plus spare slots reserved for what it is taught later (a fixed network cannot represent a new word otherwise; this favours the network). Every size is trained until it answers every training lesson (or 300 epochs), with the same settings, **tuned for none**. Energy is **measured** (CPU package counter, above idle; Daws made it readable) and also counted and priced with 45 nm figures.

**1. Same lessons, three transformer sizes (13-unit curriculum, 792 lessons; seeds 1 / 2 / 3).**

| | teaching, measured | never taught: right / **WRONG** | beyond the taught numbers |
|---|---|---|---|
| baby | 0.19 / 0.28 / 0.23 J | 90.2 / **0** ; 87.8 / **0** ; 85.4 / **0** | 0 as fact, **100 % right as a labelled guess** (all seeds) |
| transformer d=16 (6k params) | 4.6 / 3.6 / 2.8 J | 84.1 / **15.9** ; 64.0 / **36.0** ; 70.7 / **29.3** | 15-18 right, **82-85 wrong** |
| transformer d=32 (16k) | 5.1 / 3.4 / 3.4 J | 81.7 / **18.3** ; 60.4 / **39.6** ; 77.4 / **22.6** | 15-21 right, 79-85 wrong |
| transformer d=64 (49k) | 12.6 / 10.5 / 10.6 J | 78.7 / **21.3** ; 62.8 / **37.2** ; 72.6 / **27.4** | 17-20 right, 80-83 wrong |

With a 0.9 confidence cut-off the transformers are wrong on 4.9-15 % of never-taught and 24-42 % of beyond-range questions while refusing 20-64 %. Taught and reworded questions: everyone about 100 %. Scaling up from 6k to 49k parameters did **not** help on unseen questions (84.1 to 78.7 on seed 1): no sign of the "tiny nudge, huge gain" jump of Entry 2.

**2. Learning something new** (20 invented capitals, 60 lessons, asked in a fourth phrasing; then the old knowledge re-graded; **median of five runs** because one run of 35 ms varied 31-127 mJ for the same operation).

| | measured energy | new capitals | old knowledge afterwards |
|---|---|---|---|
| baby learns the 60 lessons | **63 mJ** | 100 % | **unchanged exactly** (100 / 100 / 90.2) |
| transformer, fine-tune on the new only | 120-145 mJ | 95-100 % | **forgets**: taught 100 to 49-85 %, never-taught to 37-68 % |
| transformer, new + a 10 % sample of the old | 190-250 mJ | 90-100 % | loses 3-8 points of taught/reworded, 5 points of never-taught |
| transformer, new + all the old | 1.3-1.6 J (20-25x the baby) | 100 % | keeps everything |

**3. How the two grow** (the scale generator, trained from scratch at each size; seed 1).

| units (lessons) | baby teaching | transformer d=32 teaching | ratio | never taught right / **WRONG**: baby, transformer | beyond the taught numbers: baby (as a guess), transformer right / **WRONG** |
|---|---|---|---|---|---|
| 20 (828) | 0.10 J | 4.3 J | 45x | 96.9 / **0**, 74.2 / **25.8** | 100 % right, 27.5 / **72.5** |
| 60 (2,520) | 0.51 J | 7.4 J | 14x | 94.9 / **0**, 98.5 / **1.5** | 100 % right, 18.8 / **81.2** |
| 200 (8,580) | 2.01 J | 19.6 J | 10x | 91.0 / **0**, 99.7 / **0.3** | 100 % right, 22.0 / **78.0** |

**What this supports, and what it does not.**
- **Supported:** (a) the baby is never wrong where the transformers are wrong 0.3-40 % of the time on unseen questions; (b) the baby extrapolates (a labelled guess, right every time) where an attention model with ten times the lessons stays at 19-28 % right and confidently wrong the rest, so extrapolation does not come from scale here, it comes from the rule-learning mechanism; (c) the baby adds knowledge without forgetting anything, while a cheap fine-tune forgets a lot and a safe one costs 3-25 times more; (d) teaching it from scratch costs 10-50x less measured energy.
- **Against the simple story (kept in, on purpose):** (a) the baby's energy advantage **shrinks** with scale: 45x, 14x, 10x (its cost per lesson rises, 0.12 to 0.23 mJ, while the transformer needs fewer epochs as lessons grow, 5.2 to 2.3 mJ per lesson); I have not tested beyond 200 units, so I cannot say if or where the lines cross. (b) On **raw accuracy on unseen questions the transformer overtakes the baby** at 60 and 200 units (98.5 and 99.7 % against 94.9 and 91.0 %), because careful mode makes the baby refuse more as units grow (refusals 3.1, 5.1, 9.0 %) while a big enough network learns the small-number tables from examples. (c) The cut-off transformer's 24-42 % wrong beyond the range is still a real number, but it is a number about a network with no way to say "I have no basis".
- **Caveats on the whole comparison.** (1) The curriculum is tidy and rule-shaped: the baby's home ground; a transformer with more data, regularisation, digit-aware number encoding or a hyperparameter sweep would do better, and I ran no sweep. (2) My transformer is plain Rust loops (no SIMD, no BLAS): its absolute joules are an upper estimate, perhaps several times too high; I do not know how much of the 10-50x is algorithm and how much is implementation. The baby's code is not optimised either. (3) Measured energy is the CPU package minus a 2-second idle reading; windows under 200 ms are noisy (one baby operation read 31 and 127 mJ in two runs); the five-run medians are the figures to use. (4) **My counted/priced estimates are off from the measurements by one to two orders of magnitude and not uniformly** (transformer teaching: 39 mJ estimated, 4.6 J measured, 116x; the older network answering: 0.8 uJ, 27 uJ measured, 34x), so ratios taken from estimates alone are not to be trusted; the teaching cost of the baby is not counted at all. (5) One world per row except where stated (seeds 1-3 for table 1). (6) Everything is synthetic; nothing here says what happens on real language.

**Mistakes and surprises.** The gradient check first failed at 7.4 % relative error; it was float32 round-off on gradients of about 0.001 (absolute error about 0.0001, both signs, no pattern across blocks), so the test now measures the whole gradient vector (relative L2 under 3 %) and entries above 0.01 individually, not single tiny entries. I expected the transformer to degrade with scale; it **improved** on unseen questions (74 to 99.7 %), and I had left the extrapolation column out of the first scale run, which would have made the transformer look better than it is; adding it is what separates the two.

## First contact with real text: SVAMP word problems (2026-10-21)

Everything before this was tidy synthetic lessons. `baby bridge` (`bridge.rs`) takes question/answer pairs somebody else wrote, says what the baby cannot read at all, teaches a share, grades the rest, and reports right / refused / **WRONG** / labelled-guess right and wrong by kind, why it refused, and every wrong fact by name. Corpus: **SVAMP** (arkilpatel/SVAMP, MIT licence, `SVAMP.json`, 339,970 bytes, 1,000 one-step arithmetic word problems built so that small wording changes flip the operation); downloaded with Daws's approval to scratch space, **not committed**. Kinds come from the file (Addition, Subtraction, Multiplication, Common-Division) and are used only as the units a teacher would give.

**What it can read.** 971 of 1,000 problems (29 set aside: numbers or answers of more than four digits; decimals/negatives were rare). Only 351 have exactly two numbers; 491 have three and 155 four (distractors).

**First run: it learned nothing, and said nothing wrong.** Every unit "remember, 0 % on held-back facts"; all 296 test questions refused (261 "memorise-only unit", 35 "a word it has never met"); **0 wrong**. With units found by `discover` it shattered 971 lessons into 159 units named after story words ("jerry", "storks", "salt"): discovery assumes operation words anchor a unit, and a story has dozens of words that look just as consistent.

**Why, measured before building anything.** (1) On the two-number problems the baby's own language *can* express the relations: `first apart second` explains 152 of 174 subtraction problems, "larger divided by smaller" 90 of 91 division problems; but the program search wanted a program that fits **every** lesson, and real data is a majority plus exceptions (e.g. "458 more girls than boys, 692 girls, how many pupils" typed Subtraction). (2) A program declined any question with a word not seen in that unit, and the same check zeroed the unit's own held-back score. (3) The search gave up if any lesson held more than two numbers.

**What was built: partial programs.** When no exact program exists (and only then, so every earlier result is unchanged by construction), `ProgramRule::fit_partial_with` accepts the smallest expression that explains at least 80 % of at least 24 lessons, if it still passes an MDL test: the bits in the explained lessons minus the cost of naming each exception (log2 of the lessons plus the answer's bits) must exceed the bits of searching. A constant that fits the four-in-five of a primality table that is "no" fails it (a test). A partial program's answers are **always labelled guesses, never facts**, whose stated rate is its held-back score times the unit's **routing precision** (measured by 5-fold routing of its own lessons, only when a partial unit exists). Unmet words no longer block such a guess; a unit whose best way to learn is a partial program is taken at a guess bar of 0.6 instead of the fact bar. It is not library vocabulary, is not edge-checked like an exact rule, and is revisited whenever the library grows.

**Results (two-number problems, 3 seeds, random split; blocks split in brackets).**

| | |
|---|---|
| held-back score of the program per kind | subtraction 88 %, addition 94 %, division 92 %, multiplication 70 % (24 lessons) |
| **told each question's kind (oracle)** | guess right **67-90 %**, guess wrong 5-9 %, nothing 4-24 % |
| **routed from the words alone** | only **49-62 %** of questions reach the unit of their own kind (the biggest kind alone is 51 %) |
| labelled guesses made | 61-97 per ~100 questions; right 58-64 % of them; **wrong as a labelled guess 20-35 % of all questions** |
| stated hit rate vs realised | 55.5 vs 61.9, 55.7 vs 62.2, 64.5 vs 63.9 (blocks: 74.0 vs 64.6, 54.6 vs 58.1) |
| **answers stated as fact** | **0 right, 0 wrong** (every answer is a guess) |
| the whole file (three and four numbers included) | still nothing learned: more than 20 % of the lessons hold more than two numbers, so the tolerant search refuses; 0 wrong |

**Reading it honestly.** The way of learning is sound (oracle 67-90 %, and 50 of 51 right in the first trial). The loss is routing: a prototype of three bag-of-words classifiers (naive Bayes with and without class prior, counts of one to five) tops out at 57-65 % on the same split, so this is not a bug in the baby's routing: bag-of-words cannot see the compositional cue that SVAMP is built around. The stated rates are honest in aggregate (within 1-7 points on four of five splits) and over-confident by 9 points on one blocks split. Nothing is stated as a fact, so the WRONG column the project cares about stays at 0, but **20-35 % of all questions get a wrong labelled guess**: that is what it costs to guess at all when you cannot tell which kind a question is.

**Mistakes and surprises (recorded as they happened).**
- My first idea was that number words in the stories ("one", "two") broke the operand count. Measured: 1 of 351 two-number problems. Not it.
- `--tsv` was already a boolean flag of another command and swallowed the file name; the option is `--pairs`.
- A newline inside a JSON string survived into the query; the loader now normalises whitespace.
- **A regression the long battery caught and the short ones did not:** `baby invent --careful --seeds 3 --rounds 2` read deceptive **309 / 411 / 0** where it had read **521 / 199 / 0**. A bisect (`NO_PARTIAL`, `NO_GUESSBAR` switches, since removed) showed the partial program itself. Cause: `capsum` (a sum capped at 30) used to be a plain-memory "open" unit until the library grew a piece that let `reconsider_open` find the exact program; a partial program counts as a settled program, so it was never revisited and stayed a guess. Fix: a partial program is an open unit. Now 520 / 200 / 0 (one answer differs: `wrapsum` guesses where it used to remember).
- The 2 % label-noise row of `baby scale` reads never-taught 95.9 % (was 98.0 %): I built the previous commit separately and got the same 95.9 %, so it comes from the staircase reach of the previous step, not from partial programs. I had documented the staircase's cost at 60 and 200 units but had not measured it at 2 % noise.

**Honest limits.** Four kinds from one adversarially built corpus; the kinds are the file's labels; a second look at 3-4-number problems (which numbers matter) has not started; the stated rate uses a single routing precision per unit, not one per question; discovery on real text is unsolved (159 units); nothing was tuned on SVAMP beyond the two thresholds above (0.8 allowance, 0.6 guess bar), which are mine.

## Careful by default, labelled guesses, and what one correction does (2026-10-21; Daws's call, Vision Entry 7)

**The decision.** Daws chose `careful` as the default, and asked that it not make the baby timid: a scientist guesses, then hypothesis, theory, law; a guess that is wrong and corrected is learning, a *silent* wrong answer is the failure. So the honesty rule became: **WRONG (a fact stated and false) stays at 0; a labelled guess may be wrong; what is measured is whether guesses are calibrated and whether one correction fixes the next mistake.**

**What was built.**
1. `careful` is on by default (`--permissive` opts out; `--careful` is accepted and does nothing; a saved school records `careful 0` only when permissive, and a file with no line loads careful).
2. `Reply.guess`: beyond the range taught or checked, a trusted rule's value is offered as a *labelled guess* with its `stretch` (how far past the range, in widths of that range). It is never `answer`, so every harness still counts right/wrong on facts alone. The CLI prints `probably 42` with the reason on stderr. The scorecard reports, apart, how many of the refused had a guess and whether it was right. The must-refuse battery (negative results, unreadable numbers, unmet words, memorise-only units) gets no guess; it is a test.
3. **A miss ends the guessing out there.** Each numeric unit now records the lessons its rule does not explain (`exceptions`) and reads its reach from the lessons it *does* explain. If an unexplained lesson lies outside what the explained ones span, the unit is `refuted`: it still speaks as fact inside its explained range, it no longer guesses beyond it, and what it was told wins over the refuted rule. `learn` now keeps a confirmed guess (it used to throw it away as "nothing to learn", so a confirmed guess never extended the reach).

**Numbers (simulated always-right teacher; the invention battery).**
- Guesses by stretch (`baby invent --guesses --seeds 6`, 5,672 guesses): genuine kinds **4,921 of 4,921 right at every distance**; deceptive kinds (a rule that secretly changes regime) 79.5 % right close in, about 50 % further out. The baby cannot tell the two apart from its lessons, so a distance label is all it can say honestly.
- One correction (`baby invent --corrections --seeds 6`, one correction per deceptive kind, the wrong guess nearest the taught range): **before** this change 303 of the other 305 wrong guesses stayed wrong and **2 became WRONG facts** (the correction widened the reach of a rule it had just contradicted). **After**: 0 and 0, all 305 refused. Honest limit: the 251 *right* guesses of those kinds are refused too. It learned "my rule is wrong out there", not what the new regime is; coverage comes back with more checks.
- Recovery (`baby invent --careful --seeds 3 --rounds 2 --limit 4`): after 1 round of self-checks the deceptive kinds' wrong facts fell **96 to 4** (right 471 to 433); after 2 rounds the table is identical to the earlier one (deceptive 521 / 199 / 0, genuine 4,674 / 36 / 0). Noise batteries (`scale --noise 0.02 / 0.05`, `invent --noise 0.05`) stay at 0 wrong.
- The tidy curriculum under the new default: beyond the taught range 0 % right as fact, 100 % refused, 0 % wrong, **100 % of them offered as a correct guess**; never-taught 92.7 % right (was 95.1 %: some unseen questions have an operand past the taught range) and 7.3 % refused, 0 % wrong.

**It keeps a record of its own guessing, and states it (step 4).** Each time a guess is checked (an answer it is told for a question it would have guessed at) it counts a hit or a miss in a bucket by distance (`stretch` under 0.25, to 0.5, to 1, to 2, over 2), and saves the counts. A guess now carries `rate = (hits + 1) / (tried + 2)` and `tried`; the CLI says "my guesses this far out have been right 94 % of the time, over 120 checked", or that too few have been checked to say. **Where its guesses at that distance have been wrong more often than right it withholds them** ("so I won't guess"); a unit test builds a hostile world of eight units that all saturate and shows the fifth unit's guess withheld after four misses, while a nearer distance is still offered. This is the "boldness adapts from its own record" part, in the same spirit as the bar it sets from the noise it measures.

**Is the stated rate honest? (`baby invent --calibrate --seeds 6`: guess, be told the truth, learn, in random order over all unseen questions of the 33 kinds that have a mechanism.)** 1,436 guesses; **stated rate below the realised rate in every distance bucket** (96.8 % said vs 99.9 % right under 0.25; 87.6 vs 97.2; 86.0 vs 94.0; 77.5 vs 91.5), so it is under-confident (the prior (1, 1) makes early guesses humble), never over-confident. 8,884 questions were stated as fact in the same run, **0 of them wrong**. The withholding never triggered in this battery (hits far outnumber misses, because a miss turns a unit off after one).

**A hole this found, and its fix (the corner).** The first version of that run stated **4 wrong facts out of 9,309**, all in `wrapsum` and `capsum`. The reach was a *box* (each operand's range on its own): two confirmed guesses, (20, 2) and (2, 20), stretched it to 20 on both axes, so (18, 18) counted as "checked" though nothing near that corner was. Confirmation, which I had just made extend the reach, was the cause. The reach is now a *staircase*: a question is covered only if some explained lesson is at least as large on every operand (`Remembered::uncovered_by`); otherwise it is a labelled guess. A test pins the corner (it failed first), and the old test that expected two edge probes to vouch for (20, 15) now needs the third probe, the corner, as it should.
Cost of the stricter reach (measured, not hidden): the tidy scorecard's never-taught row 92.7 % to 90.2 % right (9.8 % refused, 4.9 % of the questions offered as a correct guess); `baby scale` never-taught 96.94 % to 94.90 % at 60 units and 90.99 % at 200 units, **WRONG 0.00 in every column** (clean and at 2 % and 5 % noise); careful recovery after 1 round: deceptive kinds' wrong facts **96 (original) to 4 to 0**, after 2 rounds deceptive 521 / 199 / 0 and genuine 4,668 / 42 / 0 (was 4,674 / 36 / 0).

**Mistakes and surprises (recorded as they happened).**
- Flipping the default made the invention battery read "hidden kinds worked out 0 of 32" with no failure anywhere: it grades beyond-range questions, which careful now refuses. It measures what can be *worked out*, so the capability batteries now run permissive explicitly and say so; `run_careful`'s first row sets permissive itself.
- I pushed one commit with a failing test (my command chain did not gate on the test result); fixed in the next commit. Commits are now gated on the tests passing.
- The first version of the correction experiment built a fresh school per kind and timed out; it now builds one per seed.
- A blanket text replace added a `guess` field to function *signatures*; the compiler caught it.
- The baseline of the correction experiment was the surprise: the 5 % allowance for unexplained lessons, built for wrong labels *inside* the taught range, absorbed a teacher's correction *outside* it as one more wrong label.
- The cross-check "rule vs the exact question taught" refused the very question the teacher had just corrected (the rule says 36, memory 30); out where the rule is refuted, what it was told now wins.

**Honest limits.**
- The teacher is simulated and always right; a real teacher is sometimes wrong, and a wrong correction now makes a unit stop guessing where it should not.
- `stretch` is a distance; the stated `rate` is a smoothed count of past checks, per distance bucket and across all units (a unit-specific or kind-specific record is not kept). It is under-confident early on by construction.
- The reach check scans the lessons that share the question's operation word (the same order of work as the box did) and is not yet counted in the operation ledger, so the "57 operations per answer" headline excludes it.
- One correction stops the mistakes; it does not teach the new regime. `check_myself` rounds win coverage back, a few points at a time.
- After `sleep --compact` the reach is read from the lessons that remain (the unexplained ones stay, the explained ones are forgotten): not tested under careful; likely too narrow, which errs safe.
- The `asked` count of the recovery experiment is 498 over 2 rounds against "about 230" over 3 rounds in the older table below; I have not found out why.

## Rules that change regime: a hole, a careful mode, and a library bug it exposed (2026-10-21)

**The hole.** Plan item: extend asking from numeric programs to rules and selections. Before building, I asked whether rules and selections can be *confidently wrong at all*. They can, in a way no earlier battery tested: a rule that fits everything taught perfectly and
changes regime outside the taught numbers. `baby invent` now has four such kinds (a sum capped at 30, a sum that wraps at 40, a double capped at 30, a triple that wraps at 50) and a control (`firstalways`, whose truth really is the first operand, which it learns as a rule and gets right). **Permissively, as before: 53 wrong answers per seed
on those four kinds** (169 over 3 seeds, 187 of 240 unseen questions right). Each unit has 100 % held-back evidence and no competing explanation in its language, so every guard passes it, and the change lies outside everything taught, so no amount of data in hand can reveal it. The project's "0 % wrong" fails on this class.

**A policy, not a patch.** Whether a rule may extrapolate is a values trade-off (its extrapolation is the project's showcase, and the thing that is wrong here), so it is a switch: `School::set_careful`, `baby ask --careful`, `baby school --careful`; **the default stayed permissive when this was written; Daws chose careful as the default on 2026-10-21** (`--permissive` is now the opt-out; `--careful` is accepted and does nothing).
Careful: a rule, selection or program answers only for operands inside the range it was taught *or has had checked* for the operation named (per-operand ranges from the memory index), and otherwise says why ("for this kind I was only taught (or have checked) numbers from 0 to 12, and this asks about 20: I won't extrapolate unchecked").

**Checks extend the reach.** `check_myself` now covers rules and selections as well as programs and asks: at most two gap questions (programs), the floor (0, if nothing near it was taught), the two numbers swapped when the order never varied, each operand pushed to about twice its largest taught value, and **the corner** (both large at once: a wrapping sum shows itself only there).
Never an answer it could not read back (over four digits). Every answer it asked for is kept even if it already predicted it, since a confirmed check is what extends the reach; a refuted one makes the unit refit.

**Result (`baby invent --careful --seeds 3 --rounds 3 --limit 4`, simulated always-right teacher; unseen = never taught + beyond the taught numbers):**

| policy | deceptive kinds right / refused / **WRONG** | genuine kinds right / refused / **WRONG** |
|---|---|---|
| permissive (as before) | 551 / 0 / **169** | 4,701 / 9 / 0 |
| careful, nothing asked | 340 / 380 / **0** | 2,224 / 2,486 / 0 |
| careful, after 1 round of checks | 471 / 153 / **96** | 3,889 / 821 / 0 |
| careful, after 2 rounds | 521 / 199 / **0** | **4,674 / 36 / 0** |
| careful, after 3 rounds | 521 / 199 / 0 | 4,306 / 404 / 0 |

Careful mode with no questions removes every wrong answer and costs more than half the genuine extrapolation; **two rounds of checks (about 230 questions over 3 seeds, a few per unit) win nearly all of it back (99.2 %) and still leave 0 wrong.** The row after one round shows the limit of a probe: a wrap that only shows at the corner of the reach box survives until the corner is probed.

**Mistakes and what they uncovered.**
- My first probes tested one operand at a time, so a wrapping sum survived the first round (96 wrong); the corner probe and a cap of two gap questions per unit (hundreds of untaught pairs had crowded the edge probes out) fixed that.
- `tenfold` taught from 1 refused a question about 0: below the smallest taught number is as unchecked as above the largest. Added a floor probe.
- **A third round made things worse, and tracing it found a library bug that has nothing to do with careful mode.** The library holds at most 8 functions but a battery has ~14 program units. Every refit of a unit re-added its program and evicted the oldest function, whose unit then refit and evicted the next, in a rotation; and because a unit is fitted with its own function excluded, it rediscovered
  itself as a *rewrite through another unit's function*, so entries grew into nested equivalents (`a^2 + b^2` became `a^2 + (a - (a + b))^2`, then deeper). Dependents then lost their function and fell back to memory. Fix: **a unit contributes a given behaviour to the library once**, even after it is evicted (a test pins it); the clean battery is identical (8 seeds: 128/128 hidden, 66/72 stage-2),
  and round 3 now produces 0 wrong. It still gives back some coverage (4,306 of 4,710), because 8 slots remain fewer than the units that depend on them: open.
- I first guessed the round-3 problem was answers too long to read back. That was a real but different bug (fixed, and worth having); the evidence for the real cause came from logging each library addition and eviction.

**What this is not.** The default is permissive: out of the box the baby still answers wrongly on a rule that changes regime. A simulated, always-right teacher answers the checks. A regime change between probes (a wrap at 40 with probes at 36 and 60 ... ) can still slip through until the probe lands on it; the reach is a per-operand box.
A merged family of rule units is checked through one operator word only. Only whole numbers up to four digits.

## Tuning its own bar from what it measures (2026-10-21; Entry 5)

Plan item: let the baby set some of my constants from measurement. Two attempts, one negative.

**Negative: the search budget cannot be cut by watching progress.** Most successful program searches are cheap (median 2,223 candidates, 90th percentile 31,253) but the deepest need ~700,000, so a smaller fixed budget would lose solves. A cut-off would need a signal that separates a doomed search from a deep one. The obvious one, the best fraction of lessons any candidate explains,
does not: at ~100,000 candidates, 28 searches that went on to find a program and 29 that found nothing both sit around 0.2 (a correct deep program shows nothing until it appears). Nothing was built on it.

**Positive: the reliability bar (the 90 % a mechanism must reach on held-back facts before it may answer).** First I made it adjustable (`--bar`) and swept 0.5 to 0.95. What it does:

| bar | 10 % label noise, unseen questions answered (words / digits) | clean adversarial battery, wrong answers (4-6 seeds) |
|---|---|---|
| 0.5 | | **92** |
| 0.6, 0.7 | 100 % / 99 % | **28** |
| 0.75, 0.8, 0.85 | 96 % / 95 % at 0.8 | 0 |
| 0.9 (what it was) | **42 % / 58 %** | 0 |
| 0.95 | 13 % / 5 % (already 43-49 % at 5 % noise) | 0 |

Two facts. **The ceiling is noise:** a correct rule cannot score above about one minus the share of wrong lessons on held-back facts, so a fixed bar falls off a cliff when the world is noisier than it assumed. **The floor is real:** the wrong answers at low bars were a `select` mechanism that was right 72 % of the time on held-back facts
("remainder" explained as "the bigger or smaller of the two") and answered anyway. The noisy sweeps (3 benign seeds) had not shown that; the clean adversarial battery did, which is why it was run before anything was built.

**What it does now.** The baby measures the noise in its own lessons: lessons that say the same thing (the same content words in the same order, i.e. one fact in several phrasings) must agree, and the share that disagree with their group's vote estimates the share of wrong lessons. Its bar is
`clamp(0.98 - 2.5 x noise, 0.8, 0.9)`: exactly 0.9 for a clean world (nothing changed there), 0.8 from about 7 % noise (a margin above the 72 % that failed). The bar lives in the school (saved in the file, re-measured by `reorganise`; a thread-local while learning, so schools and parallel tests cannot interfere).
`baby scale` now prints what it believes.

**It measures well.** The baby's own estimate of the noise against the world's: 2.3 / 2.3 %, 5.5 / 5.5 %, 9.6 / 9.7 %, 14.0 / 14.2 % (words), 2.0 / 2.0, 5.1 / 5.1, 9.5 / 9.5, 14.6 / 14.9 % (digits).

| unseen questions answered, 0 % wrong on every row but the two noted | fixed 0.9 | adaptive bar |
|---|---|---|
| words, 10 % noise (60 units) | 42 % | **98.0 %** (never taught) / 99.4 % (beyond) |
| digits, 10 % noise (60 units) | 58 % | **97.0 %** / 93.7 % (0.25 % wrong on the beyond questions) |
| words, 15 % noise | | 92.9 % / 97.4 % (0.05 % wrong on taught) |
| digits, 15 % noise | | 66.7 % / 66.7 % (the floor of 0.8 is above what the data can reach) |
| 200 units: words 10 %, digits 5 % | | 93 % / 93.5 %, and 98.5 % / 98.5 %; after `reorganise` 88.8 % and 98.7 % |
| adversarial invention battery, 4 seeds, hidden / stage-2 kinds worked out; false inventions; wrong | | clean 64/64, 32/36; 5 % noise 61/64, 31/36; 10 % noise 51/64, 27/36; **0 false inventions, 0 wrong at all three** |

Clean worlds are bit-for-bit unchanged (scorecard, the clean sweep to 200 units, the clean battery).

**Honest limits.** The floor 0.8 rests on one failure (72 %) and six clean seeds at 0.75: a thin margin. At 15 % noise on digits the floor still costs a third of the coverage. The noise estimate needs groups of three or more phrasings of one fact; a world taught with single phrasings gives no estimate (it then keeps 0.9). It adapts one constant of several: folds,
the outlier allowance, the search budget and the thresholds inside voting are still mine. It measures label noise only, not ambiguity or correlated errors.

## Adapting its effort, reviewing itself, checking itself (2026-10-21)

Three changes in the direction of the learning process adapting, one of which turned out to be mostly insurance.

**1. Effort follows evidence.** Every program search had the same budget, and a unit with *no* explanation paid for six full failed searches (one on all lessons, five on cross-validation folds). The baby now searches all its lessons first, and only if that finds something does it
cross-validate: a fold that finds a program the whole data does not is a coincidence. `baby invent`: **23.5 s -> 14.0 s with identical results** (8 seeds: 128/128 hidden, 61/72 stage-2, 0 false inventions, 0 wrong). It is one rule, not a general theory of how much to try; the other knobs (folds, the 90 % bar,
the search budget, the 10 % allowance for outliers) are still constants I chose.

**2. A program that looked settled is reviewed when its vocabulary changes.** A digit sum fitted before the tens digit and the units digit existed in the library held `first mod 9` as its one explanation; later two more explanations fit equally well, but nothing told the unit. Now, once teaching is finished and whenever a lesson changes the library,
each program unit is asked one cheap question ("is my explanation still the only one that fits?", a single search that stops as soon as it has found its fits); only if not is it re-fitted. **Stage-2 kinds worked out: 61 -> 66 of 72** (8 seeds), 0 false inventions, 0 wrong, 14 -> 16 s a seed.
My first version re-fitted every program unit with full cross-validation: the same results at 76 s a seed, five times slower.

**3. A unit is never explained by vocabulary it contributed itself.** Found by a test written to reproduce the digit-sum case: the unit's own `mod 9` program had entered the library as a function, so every later search for that unit found *its own earlier invention* at size 2, where nothing could compete.
Each library function now records the unit it came from, and a unit is fitted and reviewed with the library minus its own contributions (the saved format gained an origin field).

**4. It checks its own explanations where they were tested least (`School::check_myself`, `baby wonder --check`).** For a unit that holds one program, it names the numbers nobody taught inside the range it was taught (gaps) and the first one past it, and asks about them. The unit test is the whole arc: digit sum on 0-15 with 8 and 9 missing, knowing only a ten, holds `first mod 9`, confidently says 0 at 9; it asks 8 (confirmed), 9 (refuted),
refits, then (given the tens and units pieces) sees two rivals, wonders about 19, and is right everywhere after two questions of its own choosing.

**What the experiment found (`baby invent --check`):** 105 program units asked 202 questions in 6 seeds, and **all were confirmed**: 100 % right before and after, 0 wrong. At 5 % label noise, 87 units asked 167 and all were confirmed. **The self-check never refuted anything once the review pass (change 2) existed.** An earlier version of the experiment, before the review pass, found 3 digit-sum units
that were not confirmed (100 % -> 88.9 % after checking, because it had become honest about two rivals, -> 100 % after following up with `wonder`): the review pass now catches those first. So its demonstrated value is the unit test and insurance against the data-gap case, at about two questions per program unit; I have not found a battery where it matters, and I would not claim more.

**Mistakes on the way.** I read a stale results file because it still held the previous run's `FINISHED` marker (always delete the file before a rerun). The first debug runs filtered to one kind and so did not reproduce the full run, where earlier kinds' questions change the library under the later ones (confounded, now noted). The first "refuted" counter ignored declining at the number asked.
Several runs hit my own timeouts because each seed teaches whole schools (about 100 s a seed for the check experiment); the experiments now run in the background to files.

**What this is not.** Effort adaptation is one rule. The teacher is simulated and always right. Self-checks cover numeric programs only. (The reliability bar was later made adaptive; see the section above. The fold count, the outlier allowance and the search budget are still constants.)

## It asks what it wants to know (2026-10-20; Entry 5, the learning process adapting)

Until now the baby learned only what it was handed. When the lessons left two explanations equally good (digit sum taught on 0-15: `tens + units` and `(first + tens) mod 10` agree on every taught number and part ways at 19) it declined the whole unit.
It now keeps the small set of smallest explanations that fit, **answers wherever they all agree, declines where they differ, and can say which number would settle it**. `baby wonder` prints those questions; `baby learn "<q>" <a> --unit NAME --wondered` answers one
and makes it think again at once. This is the first place the baby chooses *what to look at*, which is the part of Adaptive Learning that had no implementation.

**What was built.** `ProgramRule::alts` (the other smallest fits; saved with the school), `predict` that declines where they disagree, `ProgramRule::ambiguity` (the numbers on which the explanations give the most different answers, then the smallest), `School::wonder`,
`learn_wondered`, and `reconsider_open` (below). The old rule, "a program is trusted only if its smallest fit is unique", is now "trusted where its smallest fits agree"; a unit taught only subtraction with a >= b is the model case:
`first - second` and `first apart second` agree exactly where it was taught and part ways on negative results, which is exactly where it should decline.

**The experiment (`baby invent --active`, 6 seeds).** For each kind not already perfect, the baby asks one question at a time and a simulated teacher answers from the truth; a second teacher gives random numbers instead.

| kind | what happened |
|---|---|
| digit sum (taught on 0-15), 3 of 6 seeds where explanations compete | percent of unseen questions right: **asking 88.9 -> 100.0 after ONE question**; random numbers 88.9 -> 92.6 -> 92.6 -> 100 (three) |
| digit sum, the other 3 seeds | **no question to ask** (66.7 %, 0 wrong): the unit holds `first mod 9` as its only explanation within its vocabulary, so nothing competes; a gap that only a second explanation could expose goes unseen (a known limit) |
| four random codes, primality, xor (all 6 seeds) | **it asked nothing**, stayed at 0 % right and **0 wrong**: no competing explanations, so it invents no puzzle where there is none (the random teacher "improves" these only by memorising one fact per number) |

**Mistakes on the way (recorded on purpose).**
- **An ambiguous program entered the library.** The unit fitted `tens + units` while `(first + tens) mod 10` was still a live alternative, `grow_library` added it as vocabulary, and the unit's own explanation was then found as a single library function, hiding its ambiguity from `wonder`. A guess had become a building block.
  Only a settled program (no competing explanation) is now vocabulary; a test pins it.
- **Cross-validation penalises hypotheses that rest on one fact.** After the baby asked "19" and was told 10 the unit still did not solve, because any fold that held 19 out was ambiguous again (80 % held back). Keeping the alternatives (answer where they agree) removes that cliff, and also
  raised the invention battery under 2 % noise from 56/72 to 32/36 stage-2 kinds (4 seeds, 89 %).
- **Units left open were never revisited when the vocabulary grew.** Asking about one digit sum settled it and added a clean `digitsum` to the library, but a later wider digit-sum unit kept its stale state (76 %) while `wonder` searched with the new library and saw no ambiguity. A baby that gains vocabulary should look again at what it left open:
  `reconsider_open` re-fits numeric units that were left open, only when the library has actually changed. (My first version did it on every lesson and made the experiment take over 25 minutes.)

**Checks that nothing else moved:** the school and discovery scorecards, the clean sweep, the noisy scale worlds (0 wrong), the invention battery (clean 8 seeds: 128/128 hidden, 61/72 stage-2, 0 of 48 false inventions, 0 wrong; 2 % noise, 4 seeds: 62/64 and 32/36, 0 false, 0 wrong).

**What this is not.** The teacher is a simulation that always answers correctly; a wrong answer from a real teacher would be treated as one more lesson (noise handling applies, but nothing here tests it). It asks only about numeric programs, one hypothesis family (small expressions), up to eight competing hypotheses; it does not ask about rules, selections, facts or
phrasings. "Which question" is chosen by how many different answers the hypotheses give, not by expected information; there is no cost model for the teacher's time. It is not yet curiosity in the sense of exploring: it asks only when it has found two explanations and no more.

## Noise, finished: robust fitting and cross-checks (2026-10-20)

The previous noise round left honesty solved and coverage open: with 2 % wrong lessons a few rule units fell below the bar and became memorise-only, so unseen questions were refused (55-62 % right). Closing that took six changes, three of which were
wrong first.

| 60 / 200 units, never-taught right (WRONG was 0.00 % in every cell) | before | after |
|---|---|---|
| words, 2 % noise | 62 % / 56 % | **100 % / 97.7 %** |
| digits, 2 % noise | 51 % | **100 %** (and 100 % at 200 units) |
| words, 5 % noise | not measured | 96.9 % / 97.4 % |
| after `reorganise`, 200 units, 2 % noise | 85.5 % | **98.0 %** |
| invention battery, 2 % noise: hidden kinds / stage-2 kinds worked out | 46 of 96 / 12 of 54 (6 seeds) | **127 of 128 / 56 of 72** (8 seeds) |
| invention battery, 5 % noise | | 118 of 128 / 51 of 72, 0 false inventions, 0 wrong |
| clean worlds (every row of the sweep, 20 to 600 units) | 100 % | 100 % (unchanged), reorganise at 600 units 3.5 s (was 1.35 s) |

**1. Robust rule fit (`Quantity::fit_ridge_robust`).** Fit, find the lessons the rule misses by more than half a step, set the worst aside, refit; at most a tenth of the lessons, at most 12 rounds, a few per round. *First version wrong:* it protected a second rule hiding in the unit
(six "times" lessons among sums, which must stay visible so the unit is split) by requiring each suspect's words to be "explained by other lessons", but the outliers skew the fit so that *no* lesson is well explained and nothing was ever set aside (it explained 11 of 36, same as the plain fit). The
test is now relative to peers: a lesson is an outlier only if it misses by at least three times the *median* miss of the lessons sharing each of its words. Scattered mistakes qualify; every "times" lesson misses alike, so their median is as bad as they are and they stay. A test pins both.
A greedy one-at-a-time version made `reorganise` 9x slower at 600 units (a merged family of thousands of lessons re-solved hundreds of times); it now caps the rounds.

**2. Majority per input before building programs.** One corrupted phrasing of a fact gives the *same numbers with two answers*, which no program can fit, so a single wrong lesson made a kind unexplainable. The plurality of a fact's lessons decides; a tie drops the point.
(Alone, this took hidden kinds from 46 to 93 of 96 under 2 % noise.)

**3. Two witnesses.** If a rule unit's answer disagrees with what was taught *for that exact question* (voted across the phrasings of that fact), it declines ("my rule says X, but I was taught Y for this very question"). Two versions failed first: consulting memory's *similarity* match
treated a neighbouring lesson as a witness and cut the clean school's never-taught score from 95.1 % to 75.6 % (the scorecard caught it); consulting only the exact lesson made a wrongly taught lesson its own single witness and refused 46 % of them. The check applies only to questions that were taught as written and uses the vote across their phrasings. Clean lessons never disagree, so it costs nothing there.

**4. A split must make things better.** Noise pushed some units just under the 0.95 evidence that triggers `split`, discovery re-clustered them badly, and one group, named after the filler word "is", became a memorise-only unit that took good lessons out of working rules. A split now needs the extracted group *and* what stays behind to be under a real mechanism, checked before anything moves.

**5. Merges use the reliability bar.** A part had to be 95 % right on held-back facts to merge, and a 15-lesson unit with one wrong lesson cannot clear that. Merge candidates and parts now use the same 90 % bar as everything else.

**6. Routing smoothing scales with the vocabulary.** The routing score penalised a unit by ln(lessons + 1) for each word it lacks, so a tiny unit that had never seen a rare word was penalised far less than the big merged family that had seen it rarely, and won the question. Additive smoothing with a vocabulary term fixes it, but the constant matters:
V/2 (textbook) broke the small clean world (reworded 100 -> 93.9 %), V/20 broke 600 clean units (96.9 % routed right), V/100 keeps both exact. **This exposed an old hidden weakness:** after `reorganise`, 600 *clean* units were only 89.7 % right on rewordings (merged families diluted their own words), and the earlier sweeps never printed coverage after reorganising.
V/100 makes it 100 %.

**Mistakes in the harness:** the "two attributes at once" trap sometimes picked the *same* unit's attribute, a legitimate question with a stray word, and counted as an answered trap. Fixed.

**What is left.** (a) At 5 % noise some facts have *two of three* phrasings corrupted (about 0.4 % of facts); the majority is wrong and nothing can know. 7 such answers remain in the digit world (0.04-0.2 % of taught and reworded questions); unseen questions stay at 0 wrong. (b) A gap in the taught numbers that
only one explanation in the vocabulary can see is still believed: under noise the library lost the explanation that would have objected, and `first mod 9` explained a digit sum taught on 0-15 with 8 and 9 missing (9 wrong answers once, in one seed of six; it did not recur on the final code, but nothing structural prevents it). (c) Only label noise was tested (2 % and 5 %),
not ambiguous entities, misspellings that land on other words, or noise correlated across phrasings. (d) `reorganise` is 3.5 s at 600 units (1.35 s before the robust fit).

## Can the baby invent its own way to learn something? (2026-10-20)

Daws was told it would be interesting if the baby could invent mechanisms or learning processes, and left the direction to me. "Invent" needs a definition that can be tested, or it is a word: **a kind of lesson nobody wrote a
mechanism for, taught on small numbers, that the baby works out and gets right on numbers it was never shown, while kinds that have no explanation stay unexplained.** `baby invent` (`invent.rs`) is that experiment. Everything
below is on a synthetic numeric world in digits (see "Digits" at the end), 20 seeds unless said otherwise (each seed: new random codes, new splits).

**Setup.** Nine ordinary kinds are taught first (sum, minus, after, double, half, times, apart, bigger, smaller). Then 16 *hidden* kinds ("five times", "plus ten", "remainder by three", "take away but not below zero", "sum plus ten", ...),
9 *stage-2* kinds that need something the baby could only have learned from earlier kinds (a ten; a whole earlier program as a piece), and 7 kinds that **must stay unexplained**: four random codes (a random answer for each number),
primality, digit-xor, and (in the first version) digit sum. Each is taught on small numbers only (up to 15, or up to 12 of two), a fifth of the small facts are never taught, and questions on bigger numbers test it. Scored right / refused / WRONG.

**My premise was wrong, and the baseline said so.** I expected "five times", "plus ten" and the like to need new machinery. With the current mechanisms **12 of 16 hidden kinds were already solved**: the `rule` mechanism (a free coefficient per
number plus a bias) covers every linear one. What failed was exactly the four needing integer division or remainder (`modthree`, `halfdown`, `avgdown`, `remainder`), which no mechanism can express. The false-invention side was clean (0 of 7).

**Step 1: two more operations (floor-divide, remainder).** 16 of 16 hidden kinds, and the baby *composed* the constants itself: `first mod (1 + 2)`, `(first + second) div 2`. 30 seeds: 480 of 480 solved, 0 of 210 unexplainable kinds explained.
(The operations are mine; what it builds from them is not.)

**Step 2: the harder kinds found two honesty bugs that the easy ones could not.**
- **`tensdigit` (the tens digit of a number taught up to 15) was "explained" by `half(half(first - 1)) div 2` with 92 % held-back evidence, and was wrong on half the unseen numbers: 60 WRONG answers over 20 seeds.** The taught numbers have only two answers (0 and 1)
  so many expressions fit, and held-back accuracy cannot tell them apart. The project already had the principle ("when the data cannot say which one is meant, nothing should be built on the guess") for splitting but not for building. A built program
  is now trusted only if its smallest fit is unique *in behaviour over 0 to 30*, not just on the taught range (60 -> 42 WRONG).
- I tried to catch what remained (a gap in the taught numbers at exactly 8 and 9, where `floor(a/8)` and the real `a div 10` part ways) with **version-space agreement**: keep near-minimal alternatives and answer only where they agree, first by enumerating
  one more size, then by small edits of the found program. Both were wrong turns. The first made every run 7x slower, the second 3 minutes 45 seconds per run and *worse* (8 of 16 hidden kinds, because legitimate programs abstained), and thinking it through the alternatives
  would not have fixed the case anyway (small edits of `floor(a/8)` mostly still say 1 at 9). Reverted. The real cause was that the true explanation needed a constant the vocabulary could not build cheaply, so a larger-constant coincidence won by Occam.
  That is what the library (next) fixes: with `10` a one-piece constant, `a div 10` (3 pieces) beats `floor(a/8)` (5).
- **Primality was "explained" in 1 seed of 20** by `(((first + half(first mod 10)) div 2) mod 2)` over thirteen 0/1 answers, with 22 WRONG answers. This is the multiple-comparisons problem: a search that tries tens of thousands of candidates will fit any pattern
  that has fewer than about log2(candidates) bits in it. The guard is information-theoretic: **the lessons must carry more bits (facts x log2 of the distinct answers) than the search spent (log2 of the candidates tried before the first fit), plus a bit.**
  A short program found early passes (`first div 10`, found among about a thousand); a long one found late over thin data does not. After it: 0 of 120 unexplainable kinds explained, 0 WRONG over 20 seeds.

**Step 3: a library, learned from experience.** Two kinds of thing the baby keeps after it has learned something, both small and bounded:
- **Constants.** An integer of at least 3 among the coefficients of a rule it learned becomes a one-piece terminal for later searches. "Plus ten" and "ten times" taught it a `10` that the base language could only make in nine pieces.
- **Functions.** A program it built for a reliable unit (not a plain operation on the two numbers) becomes callable as one piece, however large it was, and is inlined into whatever program uses it, so saved programs stay plain expressions. At most 6 constants and 8 functions
  (oldest dropped), behaviourally deduplicated, grown only from units that clear the 90 % bar, saved in the school file.

| 20 seeds | no library | with the library | library, taught in the wrong order |
|---|---|---|---|
| hidden kinds (16) worked out | 320 of 320 | 320 of 320 | 128 of 128 (8 seeds) |
| stage-2 kinds | 20 of 160 (and 60 WRONG) | **140 of 180**, every kind that is solvable at all 20 of 20 | **8 of 72** |
| unexplainable kinds explained | 0 of 120 | 0 of 120 | 0 of 48 |
| WRONG answers | 60 -> 42 -> 0 after the guards | **0** | 0 |

What it built that nobody wrote: `first div 10` and `first mod 10` (from a ten it learned in an earlier unit), `1 + (sum of squares)` and `(sum of squares) div 2` *using its own earlier program as one piece*, `(first x first) + ((first - second) x (first - second))`,
and for `avgsq` an odd but correct `1 apart (1 + avg x avg)`. **The wrong-order column is the control:** taught before the kinds that give it the ten and the sum of squares, the same kinds are not learned (only `tenplus`, a plain rule, is). So the gain is what it learned earlier, not luck or a bigger language.

**Declined on purpose: digit sum.** 0 of 20, both versions. Taught on numbers up to 15 the explanations `tens + units` and `(first + first div 10) mod 10` agree on every taught number (they part ways at 19), so the data cannot say which is meant and the baby declines.
Taught up to 25 it still declines whenever the random hold-out dropped the one number that separates them. More data, not more cleverness, is the answer, and the unit test pins both cases.

**Making it affordable** (the first library run took 112 s a battery, from 3.4 s): a work budget of one million candidates per search (checked *inside* the loops, an earlier version checked between levels and did nothing), and program building is only tried when memory, a rule and a selection did not already
explain the unit. Now 22 s for 36 units; a search that finds nothing fails cheaply. Nothing else moved: the school scorecard, discovery, the clean and noisy scale worlds (0 WRONG in words and digits at 2 % noise) are unchanged.

**What this is not.** The operations, the list of terminals, the size limit, the thresholds (90 %, one bit, one million) and what counts as "reliable" are mine; the baby builds expressions inside that language and keeps what proved useful. It does not invent a new *kind* of mechanism
(a different learner, not an expression), and it does not yet change *how* it learns (how many folds, how much to try) from its own experience: that is Entry 5's third level, and this is only its program-building part. The kinds are synthetic and tidy, numbers are whole numbers up to 30
(answers under 1,000), and 16 + 9 kinds is a small sample. It is also **order dependent by design**, like a curriculum: what it can learn tomorrow depends on what it learned today.

**Digits (the enabling change).** Numbers could only be the eleven words zero to ten, which made "times five" three facts. A digit string (up to four digits) is now a number; a question that uses digits is answered in digits; negative or oversized results are refused. A first adversarial test found a bluff immediately:
"what is -3 plus 5" answered 8, because reading words drops the sign. A number with an attached sign, decimal point, fraction slash, thousands comma or percent is now refused ("I only read plain whole numbers"), in `ask`, in the CLI (whose spelling step rebuilt the question from bare words *before* `ask` ever saw it), and when teaching.
Number words are never "glue" (see the noise section). `baby scale --digits` adds the digit world to the scale test: 60 units, 100 % right, 0 wrong, all traps refused.

## Does learning stay light when the world gets big? (2026-10-20)

Everything before this was measured on 13 units and 792 lessons. `baby scale` (`scale.rs`) generates a much larger synthetic world and measures it: 20 to 600 units (60% fact units, then rules,
selects and built expressions across nine semantics), vocabularies of 1,000 to 10,000 subjects shared across units, six overlapping phrasing styles, label noise, misspelt questions, and traps that must be refused.
`--sweep 20,60,200,600` runs each size in its own process so peak memory is honest. Same seed throughout. One laptop core.

**What the first run found** (before any change; WRONG was 0.00 % everywhere, all 467 traps refused):

| units | lessons | teach | one new fact | going back over everything (`reorganise`) |
|---|---|---|---|---|
| 20 | 828 | 4.6 s | 4.7 ms | 1.6 s |
| 60 | 2,520 | 13.2 s | 11.3 ms | 9.4 s |
| 200 | 8,580 | 41.8 s | **40.1 ms** | **99.9 s** |

Learning one lesson grew with the whole world (O(lessons): every `learn` recomputed the word weights and the routing table from every lesson, and cloned every lesson to find the next fact number), and reorganising
was roughly O(units^2). At 200 units quality had also slipped (3 of 18 comparison units had mislabelled themselves, rewordings 97.5 %, never-taught 94.8 %) with WRONG still 0.

**What changed, each measured or tested:**

1. **Incremental tables.** `Weights::add` and a sparse `Routing` (counts per word per unit, `add_lesson`) take one more lesson in time proportional to its words. The routing score drops two terms that are identical for every unit
   (they cancel in the margin, which is all that is used) so the table no longer stores vocabulary x units floats. A test pins that incrementally kept tables are *exactly equal* to a rebuild after a mix of real operations.
   A lesson that only corrects an answer no longer rebuilds anything. Result: **a new fact costs 0.4 ms at every size from 20 to 600 units** (was 40 ms at 200). Answering also got cheaper: 85 -> 57 counted operations
   per question (159x fewer than the slow way, was 106x), same answers.
2. **Cross-validation folds that cannot be peeked through.** Memory matches by *set* of words, so the swapped twin of a commutative fact ("which is bigger two or three" / "three or two") held in another fold let `remember` look perfect
   and tie with `select`; the cheaper one won and the unit became memorise-only. Facts that share a bag of words now go in one fold. A seed-sweep test pins it (it failed on seed 10 before the fix).
3. **Stratified folds for merges.** A unit of five lessons was judged on whatever a random split left it: it could join a rule family on only 17 of 24 seeds. Each part's facts are now spread over the folds
   (24 of 24 seeds, and multiplication is rejected on 24 of 24).
4. **Merging by set, not by pair.** `reorganise` tested every pair and started over after every merge; each merge then re-ran cross-validation on the whole growing family. Profiling (not guessing) showed the merge test cost 20-30 ms
   and the re-learning *after* it 6.9 s for 564 lessons. Now a whole set of rule units is tested at once ("does one rule fit every part?"), halved only when it does not, and the merged unit is settled from the evidence that justified the merge.
   **99.9 s -> 0.32 s at 200 units, 1.35 s at 600.**
5. **The network is only tried where it can matter** (cheap mechanisms show some structure, 25 % up to the bar). It was trained and cross-validated for every unit that was a plain table and never won. Teaching 25,776 lessons:
   127 s -> **2.65 s** (792 lessons: 3.8 s -> 0.34 s). No scorecard moved.

**Clean world after the changes** (final sweep): 

| units | lessons | teach | peak MB | new fact | repeat lesson | answer | reorganise | file | mechanism right | reworded / never taught | WRONG | traps answered |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 20 | 828 | 0.08 s | 5.5 | 0.38 ms | 0.65 ms | 6 us | 0.03 s | 33 KB | 100 % | 100 / 100 % | 0.00 | 0 |
| 60 | 2,520 | 0.35 s | 8.8 | 0.64 ms | 0.66 ms | 10 us | 0.09 s | 97 KB | 100 % | 100 / 100 % | 0.00 | 0 |
| 200 | 8,580 | 0.85 s | 21 | 0.41 ms | 0.61 ms | 7 us | 0.32 s | 331 KB | 100 % | 100 / 100 % | 0.00 | 0 |
| 600 | 25,776 | 2.65 s | 56 | 0.57 ms | 0.55 ms | 11 us | 1.35 s | 1,008 KB | 100 % | 100 / 100 % | 0.00 | 0 |

A 10,000-subject vocabulary (200 units) behaves the same (0.9 s to teach, 21 MB, 100 %, 0 wrong, 467 traps refused). Memory is about 2 KB per lesson including the test data; the file is 40 bytes per lesson.

**Mistakes made on the way (recorded on purpose):**

- **My first generator made rewordings that used a word the unit had never been taught** ("what is bafoba sedavubi" where "what" never appeared in that unit). The baby refused 30 % of them. I first read that as a weakness; it is the
  guard that closed the "code for five" bluff working as designed. The generator was wrong; a test now pins that every reworded question uses only words its unit was taught.
- **I attributed the 3 mislabelled units to scale.** They were a CV flaw (change 2) that scale merely made frequent: at 20 and 60 units the luck happened to hold.
- **A test failed after change 2 and I first suspected the change.** It was a test that passed on seed 1 by luck (the small unit joined the family on 17 of 24 seeds with the old folds, 18 with the new); the real flaw was
  variance in the merge test (change 3). Leave-one-out predictions were perfect, 5-fold ones were not: the clue.
- The "settled" shortcut and the network trial were tuned on a clean world; label noise (below) found them wanting.

**Label noise (2 % of taught answers wrong), in three rounds.** The first noisy run broke the headline claim; fixing it took three more findings, and the second round contained a mistake of mine.

| 2 % noise, 60 / 200 units | round 1 (clean-world code) | after the 90 % bar + settled fixes | after voting memory + order + number words |
|---|---|---|---|
| taught, right / WRONG | 91.0 / 1.87 % | 82.3 / 0.20 % | **100.0 / 0.00 %** (both sizes) |
| lessons taught *wrongly*, answered with the truth | (mislabelled, see below) | 40 % | **100 %**, repeated the wrong lesson 0 times |
| reworded, WRONG | 2.50 % | 0.83 % | **0.00 %** |
| never taught, right / WRONG | 77.3 / 6.12 % | 62.2 / 0.00 % | 62.2 and 55.5 / **0.00 %** |
| beyond the taught range, WRONG | 9.18 % | 0.00 % | **0.00 %** |
| must-refuse traps answered | 1 of 140 | 0 of 140, 1 of 467 | **0 of 140, 0 of 467** |

What each step was:

1. **A mechanism with 28 % held-back accuracy was allowed to answer.** The bar to replace memory was 25 %, which clean lessons (evidence about 1.0 or about 0) never test. It is now 90 % (`RELIABLE`; test pinned), and a reliable rule
   may leave one lesson in twenty unexplained instead of re-deciding everything (so a noisy-but-reliable unit stops costing 160 ms a lesson).
2. **Voting memory.** Memory used to answer from the single closest lesson, so one wrong phrasing out of three won. Now every lesson with the question's content words votes, weighted by closeness; the plurality answers, a tie refuses.
   Voters need only 0.3 closeness (a wordier phrasing of the same fact scores about 0.45 and must be heard) but something must be close (0.5). Wrongly taught facts are now corrected by their own other phrasings, which is redundancy the
   baby had all along.
3. **Memory ignored word order and repeated operands, so the same trap came back a fourth time.** In memorise-only numeric units "two nidovubi five" (a negative result) retrieved "five nidovubi two", and "three times three" retrieved
   "three times two" because the content words were compared as a *set*. Content words are now compared as a *sequence*, repeats kept. And "glue" (words ignored by the fill-the-blank guard) used to be any word in a tenth of a unit's
   facts, which made number words glue; it is now two fifths of the facts, and a number word is never glue ("one" is in over 40 % of the times facts and is not "what is"). Each of those was found by printing the wrong answers
   (`SCALE_WRONG=1`), not by reasoning: the examples named the cause every time.

**A mistake in my harness.** It graded the wrongly taught lessons against the answer they were *taught*, so "right" meant "repeated the wrong lesson". The first noisy tables therefore reported "40 % faithfully repeated" when the truth was
40 % repeated and 40 % refused. It now grades against the truth and counts repeats separately.

**What noise still costs is coverage, not honesty.** Never-taught and beyond-range questions are refused much more (55-62 % right instead of 100 %): a few wrong lessons skew the *exact* least-squares fit, so rule units fall to 70-89 %
held-back, miss the 90 % bar and become memorise-only, and the 3 program units that need an exact expression cannot find one. A **robust fit** (drop lessons that disagree *only if their operator word is otherwise consistent*, so scattered
noise is ignored while a polluting second rule is still caught by split) would recover that, and is the remaining noise work. Noisy units that are not settled still cost 80-250 ms a lesson (a new fact in a settled fact unit stays 0.5 ms).

**Honest limits.** Still synthetic and tidy in structure: made-up words, number words 0-10, four phrasings per fact, a vocabulary with no frequency distribution (no Zipf), no ambiguity between entities and words, no
real misspellings that land on other words. `typo` questions are all refused (none answered), so spelling robustness at scale is untested here (the letters reader exists in the older network but is not in the school). Discovery
(`baby discover`) still has its own quadratic pair loops (3.6 s on 792 lessons) and was not scaled. Teaching cost is now a property of the *generator* as much as of the baby: real text will have far more distinct units and
far less regular ones. 600 units / 26,000 lessons / 10,000 words is a long way from millions of sentences.

## Learning is light (2026-10-20)

Daws: training is enormously intensive today, and learning with Adaptive Learning should be light enough that nearly anyone can do it, with no data centre. That is
testable at this scale. `baby cost` (one core of this laptop, no GPU; the CPU's energy counter is off limits here, so the measures are time and memory):

| | cost |
|---|---|
| teach it from nothing: 792 lessons, 13 units | **3.8 s, 11.9 MB** peak (3.3 MB of that is the program itself) |
| a lesson of a kind it already understands | **0.18 ms**, none of them needed learning, none kept |
| a lesson that says something it knows a new way ("added to") | **3 ms** |
| a lesson of a kind it has never seen, nobody says where it belongs | **1 ms** (the pen); the 6th lesson opened a unit that learns by a rule, 100% held back |
| going back over everything: merge and split units | 0.6 s |
| forgetting what is already explained | 0.01 s |
| the whole session | **11.9 MB** peak |

Three things make learning light here, and each was measured, not assumed:

1. **Surprise-gated.** A lesson the unit can already work out is neither learned from nor stored. Of 60 lessons of kinds it already understands, 60 cost nothing.
2. **Refit, don't re-decide.** When a surprising lesson is consistent with a unit's settled rule (a new phrasing, a new fact) the baby just refits the rule exactly and
   checks that it still explains every lesson. The expensive step, cross-validating five ways of learning, only runs when something structural changed. That took a surprising lesson
   from ~93 ms to ~3 ms.
3. **Exact where it can be.** Fitting a rule is solved, not iterated (392x fewer operations), and nothing is trained for thousands of passes.

**A bug that distorted my first numbers.** A lesson's identity was the *sorted* set of its words, so "five minus two" and "two minus five" were the same lesson, and lessons like "three plus two" after
"two plus three" were treated as corrections of each other. 39 of 60 lessons were being "corrected" instead of skipped, each one re-running the full cross-validation (93 ms each), and none were counted as kept.
A lesson's identity is now its words *in order*; the same 60 lessons now cost 0.18 ms each. (That bug could also have corrupted order-sensitive units like subtraction; it did not in the data, but would have.)

Honest limits, and they are large: 800 tidy lessons is not the world. **Nothing here says how this grows with millions of real sentences**, whether the pen and the cross-validation still stay this cheap
when lessons are noisy and overlapping, or whether a language with huge vocabulary needs more than a handful of rules. Time and memory on a laptop are an honest proxy for "needs no data centre" at this scale,
and no more than that.

## Reorganising over time: merge and split (2026-10-20)

Discovery should not be a one-off. `baby sleep` now also looks again at how the lessons are grouped, in both directions:

- **Split**: a unit that is no longer fully explained (its newer lessons stopped following one rule) is taken apart if its lessons separate into groups that each *are* explained. The largest group stays; the others
  go to the unit that builds exactly the same expression, or to a unit of their own. Test: six multiplication lessons put under `sum` by mistake were taken out and went to `times`; `sum` was itself again.
- **Merge**: two units that were taught separately merge if a rule, or a rule with a setting per word, is right for **each** of them on facts held back. Test: a unit `added` ("what is a added to b") taught on the side was folded into the rule family.

On the real school this merged the five rule-like units (sum, minus, after, double, half) into one, so there are 9 units, the same 9 that discovery found from unlabelled lessons; what the merged rule learned takes 586 bytes. Scorecard unchanged:
100% / 100% / 95.1% / 100%, 0% wrong, every nonsense question still refused.

Three bugs on the way: (1) the first split moved *every* healthy group out, including the big correct one, and left the original unit empty (the largest now stays); (2) after merging, the recognition floor was still
calibrated for 13 units and the scorecard slipped to 98%/96%/92% (all refusals) until the floor was re-set from the new table; (3) the lesson-identity bug above.

Honest limits: merging needs *both* units to have settled on a rule or a selection (a built expression is never merged on its own, the apart/minus lesson); the split needs a unit with at least ten facts, numeric lessons and a clear separation; and
neither can fix lessons the teacher got wrong in a way that still follows a rule.

## What it keeps, and forgetting what is explained (2026-10-20)

Daws raised storage: a conventional model is trained on a pile of data (tens of terabytes at the large end) and the model itself is much smaller, so the pile is
the heavy part. The baby has the same shape in miniature, and `baby storage` shows it. Of the school's 29,139 bytes, **93% is the lessons it was taught and 1.3 KB is
everything it learned from them**. By unit, the rule for sums took 87 lessons (2,780 bytes) and is 240 bytes; multiplication took 66 lessons (2,189 bytes) and is a 58-byte
program. Facts do not shrink: a unit of capitals is its lessons.

So I taught it to forget what is already explained (`baby sleep --compact`). A unit that learned a rule, a selection or an expression keeps a support set: the facts it could not get right
without, and every word it needs; a unit of facts keeps two phrasings of each. Then comes the part that took three tries:

**The promise: it must still answer every lesson it was ever taught, kept or not, and never answer wrongly.** First version: the file fell to 47% but "taught" dropped to 96.3%, rewordings to 88.6%.
Two causes. (1) I tested whether the rule still worked but not whether the baby still *recognised* the question: the router had lost examples of how each unit is phrased. Compaction now
re-checks every lesson it let go and puts back any it can no longer answer, until it can answer all of them. (2) That check covered only the lessons let go, and the recognition floor was
recalibrated from them, so the *kept* lessons of two tiny units (which have weaker routing scores) fell below it. The floor is now set from every lesson it was ever taught, with room below it.

Result on seeds 1, 2 and 3 (identical): **358 of 792 lessons kept, the file at 49% of its size (29,139 to 14,274 bytes)**, with

| questions | before | after forgetting |
|---|---:|---:|
| taught (all 792, kept or forgotten) | 100% right | **100% right** |
| reworded | 100% right | 97.7% right, 2.3% refused |
| never taught | 95.1% right | 94.5% right, 5.5% refused |
| beyond the taught range | 100% right | 99.8% right, 0.2% refused |
| **wrong, anywhere** | 0% | **0%** |

Every loss is a refusal, never a wrong answer, and the nonsense battery (22 unanswerable questions) is still all refused (a test pins both). The 2.3% lost are all one phrasing, "what is after X", the least decisive
phrasing there is: unseen phrasings are less decisive than taught ones, so they sit near the floor. A lower floor recovers some; it is the same dial as before.

Honest limits: this is a 792-lesson world. The ratio (lessons about 100x the size of what a rule learned from them) shows what the structure of *rule-like* knowledge allows; it says nothing about what 100 billion parameters
of general language would compress to, and facts (the part that grows with the world) do not compress at all. The file is plain text, with no entropy coding: a real format would shrink everything a further 3 to 10x without forgetting anything.

## Lessons that fit nothing: the pen (2026-10-20)

Pushing on Adaptive Learning: `baby learn "query" answer` no longer needs the unit named. If the baby recognises a kind of lesson it joins that unit; if not, the lesson waits in a **pen**.
When enough gather (at least six lessons of at least five different facts) the same discovery used on a heap of unlabelled lessons looks at the pen, and a group that follows one rule
opens a unit by itself, named after the words that were new to it. A real session, nothing named:

```
$ baby learn "what is zero less than ten" ten        ... five more, "what is N less than ten"
learned: ... (this is not a kind of lesson I know yet: set aside with 4 others waiting)
the pen is empty now                                   (the sixth lesson opened a unit, 'less/than', which learns by a rule)
$ baby ask "what is eight less than ten"      two      (it was taught 0 to 5)
$ baby ask "what is ten less than three"      ?        (negative: it refuses)
```

And a new phrasing of something it already knows: six lessons "what is a multiplied by b" gathered, and at the sixth they **joined unit `times` by themselves**, because the group built the one expression
`first x second` that `times` builds. Along the way, "all N lessons waiting look like another way of asking unit 'times': confirm with --unit times" notes appear as a suggestion before it acts.

**The same trap, a third time.** My first pen joined the "N less than ten" lessons to unit `apart`, because the expression it built for them was identical to apart's. It is the minus/apart trap again:
for a number below ten, `ten - n` and `|n - ten|` agree on every lesson, so the search picked `apart` (enumerated first) and the baby would have answered "ten less than three" with 7 instead of refusing.
Two repairs, both general: an expression only counts as *the* explanation if exactly one behaviour fits the lessons (two expressions that agree on every pair of numbers from 0 to 10, like `a x b` and `b x a`, are one; `second - first` and `first apart second` are two),
and the same test guards suggestions. A test pins both directions: "N less than ten" is ambiguous, multiplication is not.

Honest limits: the pen needs six lessons of five facts before it acts; it can only open a unit for a kind that one rule explains (anything else stays waiting); names are the group's new words; and joining an existing unit needs
an unambiguous identical expression, so it will *not* merge a new phrasing into a rule-family unit by itself (it suggests).

## Finding the units itself (2026-10-20)

Daws: push on Adaptive Learning, the revolutionary part, not conventional training. Until now the teacher named the unit of every lesson. `baby discover`
hands the baby **792 lessons with no grouping at all** (questions and answers, shuffled) and it works out the structure itself (`src/discover.rs`):

1. **Anchor.** Each lesson's anchor is its most telling word: the rarest word frequent enough to be a pattern, which is not a number, and which spans at
   least four different facts ("times", not "peru").
2. **Same facts, other words.** Two groups that hold the *same* facts, both ways round, are phrasings of one unit ("what does the cat say" / "cat sound",
   "double" / "doubled", "what comes after" / "after" / "then what").
3. **One rule.** Two groups of number questions merge only if a mechanism with a setting *per operation word* (a rule, a selection) is right for **each group** on facts
   held back, or both built the very same expression. "plus" and "minus" merge (a rule has a weight for each word); "times" and "plus" do not.

Result, identical on seeds 1 to 5, from 792 unlabelled lessons:

```
found unit             lessons   made of (the units the teacher would have named)
times                       66   times 66
plus/minus/equals          186   after 18, double 15, half 15, minus 51, sum 87
bigger/smaller             144   compare 144
color / opposite / capital 60/120/60   each pure
and/are                     87   apart 87
make/say/sound              42   sound 42
for/code                    27   code 27
9 units for the 13 the teacher would have named, found in 3.7 s
```

Each fact kind came out pure, the five rule-like kinds (sum, minus, after, double, half) correctly became one family with a setting per word, `compare`
needed its own mechanism, and `times` and `apart`, each of which needs an expression built for it, stayed separate. Its scorecard is
**identical to the baby the teacher organised**: right 100% / 100% / 95.1% / 100% (taught, reworded, never taught, beyond the range), refused the rest, **0% wrong**.

Three mistakes on the way, because they are the useful part:

- **"night" became an anchor.** It occurs in enough lessons (a color item and half of day/night) to look like a pattern, so the opposites split. The difference is
  spread: an operation word spans dozens of different facts, a thing being asked about spans one or two. Anchors now need at least four distinct facts.
- **The "same facts" shortcut was unsafe for numbers.** "Five minus two" and "how far apart are five and two" are both three: for the bigger number first, `a - b`
  and `|a - b|` agree on every lesson, so the two groups were merged into one unit built on `|a - b|`, and the discovered baby answered **"three minus five"
  with "two"**. The organised baby correctly refused. The scorecard had not asked that question; I found it by asking. Two repairs: the same-facts test now keeps
  the order of the numbers and must hold both ways round (a group merely *including* another's facts is a different unit); and a single built expression is not
  a licence to merge, because it ignores which operation word was used. A test pins "three minus five" and "two minus nine" as refused.
- **A small group was swallowed.** Six "after" lessons joined 144 comparison lessons because the *overall* held-back score stayed above 95%: the error was
  entirely the small group. A merge now has to hold for each group separately.

Honest limits: the lessons are this tiny curated world, where numbers, shapes and phrasings are regular; the anchor test, the thresholds (4 facts, 80% number
words, 95% on held-back facts) are mine; groups with fewer than four facts cannot anchor; the names it gives units ("plus/minus/equals") are its commonest anchors and are not
pretty; it chooses among the five mechanisms and cannot invent a sixth; and "identical on five seeds" partly reflects that the structure of the world is fixed (only the
shuffle and the held-back facts change).

## A different way of computing (2026-10-20)

Daws: the mathematics can change *what* is computed or *how*, for far greater results. Fitting a rule had been done the standard way: nudge the weights
a little, 800 times over every lesson. But fitting a rule is a *linear* problem, and a linear problem has an exact answer: one small system of equations (ridge
regression, solved by elimination). No learning rate, no number of passes, the same answer every time.

| fitting the rule for sums (87 lessons) | operations | time |
|---|---:|---:|
| nudging it 800 times | ~2,505,600 | 12.3 ms |
| **solving it exactly** | **6,398** | **0.22 ms** |

**392x fewer operations, 56x faster, the same rule** (`plus` = 1.00, 1.00, 0.00; the same answers on held-back and beyond-range sums, pinned by a test). Because fitting
a rule is in the inner loop of everything (every cross-validation fold, every candidate merge in discovery), the whole thing sped up: discovery from 26-45 s to 3.7 s, teaching the school from 4.6 s to 3.9 s.
The scorecards did not move. What changed is not only speed: the iterative fit had two knobs (passes, learning rate) that were quietly part of what the baby "learned"; the
exact one has none.

Honesty: this works because the rule is linear in its weights; the built expressions (a search) and the small network are not, and are still found the slow way.
And my first attempt at swapping it in broke the file (I replaced the wrong `Mech::Select` and duplicated a third of it); it was rebuilt from the intact tail and all 55 tests pass.

## Less effort, same answers (2026-10-20)

Daws: the mathematics of the computation can make the machine do far less for results as good or better. `baby effort` counts what answering
costs (operations, 4-byte words moved, and the estimated energy of both from the 45 nm figures; absolute joules are an estimate, the
ratios are what to trust) on 2,592 questions (taught, reworded, never taught, beyond the range):

| how the baby finds and gives an answer | operations / question | words read | est. energy | right | refused | wrong |
|---|---:|---:|---:|---:|---:|---:|
| the slow way | 8,977 | 6,126 | 45.0 nJ | 99.7% | 0.3% | 0% |
| **what the baby does** | **85** | **89** | **0.80 nJ** | **99.7%** | 0.3% | **0%** |
| one small network | 7,104 | 7,104 | 388 nJ | (memorises; cannot work out a new sum) | | |

**106x fewer operations, 69x fewer words moved, about 56x less estimated energy, the same answers.** Training dropped from 12.3 s to
4.6 s as well (one seed). Where the savings come from:

1. **Finding the unit was 99% of the cost** (8,856 of 8,977 operations): the slow way compared each question with all 792 lessons ever taught.
   The baby now keeps a table of how telling each word's *shape* is for each unit, `ln(P(word | unit) / P(word))`, and a question's score for a unit is
   the sum over its words: about 63 operations. (It is a naive-Bayes-style score; the unit with the highest score wins.)
2. **Remembered questions are found through an index of their content words**, not by scanning them all. This is safe because a question can only be
   answered from a remembered one that contains all of its content words anyway.
3. **Thrifty training**: a network is only trained and cross-validated for a unit if the cheaper mechanisms did not already reach 98% on facts
   held back; and only the mechanism it will use is fitted. Most units now never train a network at all.
4. **The answering itself was never expensive**: a learned rule is ~20 operations, an expression it built ~10. The network is 7,104 and cannot even
   do the sums.

It took two honest corrections to get "the same answers" and not just "fewer operations":
- **First version: 94.2% right** (5.8% refused). The "is this a kind of question I recognise?" check compared the raw score with a floor, and raw scores grow
  with the number of words, so a three-word question like "one plus zero" looked less recognisable than a long one, though it is unmistakably a sum. The fix
  is to measure how decisively one unit beats the next (the margin), which does not depend on length.
- **Then 99.3% (not 99.7%)**: ten questions, all one reworded phrasing ("what is after five"). The floor was set so that 95% of rewordings are
  recognised, i.e. the least decisive 5% were turned away on purpose. I made it a 99% floor and re-ran the battery of 22 unanswerable questions: all still
  refused. The floor is a dial (lower recognises more, risks more), not a law.

Honesty about this: these are counted operations, not a meter (the CPU's energy counter is still off limits here). The comparison "one small network" is the
older single-network baby, not a clever alternative. Savings of this size come from an inefficient baseline: the first design compared against everything.

## Learning after school (2026-10-20)

The baby now keeps every lesson it has been taught, so it can go on learning. `baby learn "query" answer [--unit NAME]` adds a lesson, works out again how
that unit is best learned from *all* its lessons, and saves. With `--unit` it joins that unit or opens one of that name; without, it joins the unit the
question resembles if it recognises a kind of lesson and refuses (saying why) if not, rather than guessing. Fewer than five facts in a unit: it only
remembers, because that is not enough evidence to trust anything else.

A real session:

```
$ baby ask "what is four multiplied by two"
?   I haven't met the words multiplied, by. If you tell me, I'll remember...
$ baby learn "what is two multiplied by three" six --unit times      (and two more examples)
learned ... (added to unit 'times'; it now has 25 facts)   it learns this unit by program (100% on facts held back)
$ baby ask "what is four multiplied by two"
eight   (unit 'times', worked out by an expression I built, (first x second) ...)

$ baby learn "what is zero less than ten" ten --unit lessthan        (a kind it has never seen: opens a new unit)
it only remembers this unit for now: with fewer than five facts there is not enough evidence to trust anything else
   ... four more lessons ...
it used to learn this unit by remember; with the new lesson it now learns it by rule (100% on facts held back)
$ baby ask "what is eight less than ten"                             (it was taught 0 to 6 only)
two   (unit 'lessthan', worked out by a learned rule ...)
```

Honest limits: the lessons are told one at a time with the unit named (it does not yet discover that several lessons belong together, or invent a unit
for lessons that fit nothing); every new lesson is treated as its own fact, so two phrasings of the same fact count as two (the cross-validation then
sees a "twin" and is a little optimistic about remembering); and the other units' mechanisms are not re-tested when a new unit changes the word statistics.

## One baby: the school (2026-10-20)

`baby school` unifies everything that worked. A curriculum of 13 **units** (capital, color, sound, opposite: facts; sum, minus, after,
double, half: arithmetic-like rules; compare: pick the bigger or smaller; times, apart: forms that had to be *built*; code: an arbitrary
mapping) is taught. The teacher groups the lessons and never says how to learn them. For each unit the baby holds back facts, tries five
mechanisms (remember, rule, select, program, network), and keeps what generalises. A question then goes through this pipeline
(`baby ask`):

1. **What the user taught it** (`baby teach`): looked up first, any word, no weights.
2. **Spelling**: a word one or two letters from exactly one word it knows is taken to be that word, and it says so.
3. **A word it has never met**: it says so and asks to be taught.
4. **Which unit?** By a precomputed table of how telling each word's shape is for each unit (every number is a number); if no unit beats the next by enough, it says it doesn't recognise the kind of question.
5. **That unit's mechanism answers**, or declines: the rule, selection and program decline on words and shapes they have not learned; remembering
   declines unless the *word that fills the blank* matches what it was taught.

**The honest scorecard**, printed by `baby school` and identical on seeds 1 to 5 (share of questions): 

| questions | right | refused | **wrong** |
|---|---:|---:|---:|
| taught | 100% | 0% | **0%** |
| reworded | 100% | 0% | **0%** |
| never taught (held-back facts) | 95.1% | 4.9% | **0%** |
| beyond the taught range (numbers up to ten) | 100% | 0% | **0%** |

The 4.9% refused are the codes for numbers the baby was never taught: an arbitrary code cannot be known. It also refused all 27 of a
separate battery of unanswerable questions I wrote after the fact: nonsense made of familiar words ("capital of banana", "color of france"),
unrelated chat ("tell me a joke"), numbers off the line ("five times five", "three minus five"), digits ("what is 3 plus 4") and
phrasings it was not taught ("two multiplied by four"). Those last two are refused although a person could answer them: the safe failure.

**A bluff the scorecard missed, then found.** My first unified version answered "what is the code for five" with "three" (the code for
*one*), claiming it was recalled. The code unit is memorise-only, and its matching was dominated by the glue words ("code", "for"),
while number words turn up all over the curriculum and so carry little weight. The first scorecard had passed because it only asked
questions of a certain style; the check that caught it was asking for every code from zero to ten and comparing with the truth.
The fix: a word that appears across several different facts of a unit is glue; every other word of a question must match the
remembered question it is answered from. Now: 9 right, 2 refused (the two never taught), 0 wrong, and a test pins it.

Honest limits:
- **The curriculum, the units and the counting lessons are given by the teacher**, and the five mechanisms and the expression language are mine.
- **It keeps learning after `school`** (`learn`, see above), but only lessons told one at a time with the unit named.
- **The refusals are conservative**: digits, new phrasings of a kind it knows (until it is taught them with `learn`), and a negative or above-ten result are all refused.
- **"0% wrong" is on questions in or near the style it was taught.** The 27-question battery is a spot check, not a proof.

## What it is

```
query -> 8 word slots -> reader -> tanh hidden layer (64) -> softmax over 95 answer words -> one word
```

(The reader is `concat` by default, the original; `--reader bag` is smaller and generalises better, see "Position-free readers".
The numbers in this section describe the original `concat` baby.)

- **16,271 parameters** (about 65 KB as floats; the readable text file is 185 KB). **14,272 multiply-adds per query.**
- Trains in about **6.5 seconds** on one laptop CPU core. A question takes **13 us** on this laptop (it was 32 us before one small change, below).
- Everything is hand-written: forward pass, backward pass, AdamW, the tokenizer, the RNG, the file format. Gradients are verified
  against finite differences in a test, which is the check that matters most in a from-scratch net.

## Its world

Six kinds of tiny question, each with one-word answers, in four phrasings (`src/data.rs`): colour of things, animal sounds,
opposites, capitals, addition and subtraction up to 5 (number words). The data is generated in code from a seed, so every result is
reproducible. Phrasings 0-2 are for training. Phrasing 3 is a recombination of words the baby has already seen ("new-phrase"
test). For arithmetic, 15% of whole sums are held out in every phrasing ("new-sum" test: did it learn the rule or the table?).

## First results (2026-10-15)

| | train | new phrasing | new sums |
|-|-------|--------------|----------|
| shuffled, seeds 1-4 | 100% | 55 / 56 / 58 / 60 % (mean 57.3) | 19 / 6 / 25 / 44 % |
| child-like curriculum, seeds 1-4 | 100% | 61 / 69 / 62 / 62 % (mean 63.4) | 12 / 6 / 28 / 25 % |

Per family (seed 1, shuffled), new phrasing: sound 100, add 100, sub 100, color 40, capital 25, opposite 7.5.

What this teaches us, in order of confidence:

1. **It memorises perfectly and generalises where position lines up.** Every family that fails on the new phrasing is one where
   the key word lands in a slot it never occupied in training (read the templates in `src/data.rs`: "the opposite of X" puts X in
   slot 3; the training phrasings put it in slots 0, 2 and 5). Sound, add and sub succeed because their held-out phrasing keeps the
   words in familiar slots. This is the architecture's fault, not the data's: embeddings glued side by side tie every word to a
   position. It is a clean, checked reason to try position-free ways of reading (pooling, small convolutions, tiny attention),
   and the baby gives us a yardstick for it.
2. **It learned the table, not the rule, for sums.** Held-out sums land far below the training accuracy and swing 6-44% between
   seeds (only 32 test items, so noisy), and subtraction is the weakest.
3. **Curriculum may help a little.** Introducing the families in stages (colour/sound, then opposites, then capitals, then sums,
   with the earlier ones kept in the mix) scored about 6 points higher on new phrasing across four seeds. That is a hint, not a
   finding: four seeds, noisy, and the curriculum run also takes fewer gradient steps early on. BabyLM found ordering tricks
   mostly don't help, so this deserves a bigger, fairer test before anyone believes it.
4. **Its confidence is honest.** Questions it can't answer ("capital of atlantis", "what color is a zebra", an unseen phrasing)
   come out below 0.5, and `ask` says "not sure". A free "I don't know yet", which is what a child does.

## Where this goes next (candidates, not commitments) [STALE as of 2026-10-15: the authoritative list is the last "Next" section at the bottom, and TransientHandover.md]

- **Position-free readers** compared on the same harness: does the reader matter at 16k parameters? (fixes #1)
- **Learn one thing, now:** a `teach` step that adds a new fact in a few updates without wrecking the rest, and a fact *store*
  that answers with zero updates: the first two rungs of the learning ladder (`../LEARNING.md`).
- **Letters instead of words**, so it can read words it has never seen and spell: closer to how a child starts.
- **Rules from few examples** (the sums): what inductive bias makes a tiny net learn addition instead of a lookup table?
- **A scoreboard in Transient's terms:** accuracy per parameter, per multiply-add, per byte, per second of learning.

## Thinking in joules (2026-10-15)

`src/ledger.rs` counts every multiply-add and every 4-byte memory access, then prices them with the widely cited 45 nm figures from
Horowitz (ISSCC 2014): fp32 multiply 3.7 pJ, add 0.9 pJ, a 32-bit DRAM read 640 pJ (confirmed in a search; the cache prices of 5 and
50 pJ per word are my recollection of the same talk). A modern chip spends several times less in absolute terms, so treat the joules
as an **estimate** and trust the **ratios**. When the OS allows it, `baby energy` also reads the CPU's own energy counter (Intel RAPL)
to compare the estimate with reality; this machine currently denies access (see below).

What counting showed:

- **Answering one question costs ~14,000 multiply-adds, about 0.8 microjoules (estimated), and 91% of that is memory traffic, not
  arithmetic.** Even a 63 KB network is a data-movement problem. A 7-billion-parameter model does ~500,000x more arithmetic per
  token and cannot keep its weights in cache at all, so each word it reads is priced as DRAM (640 pJ vs ~50 here).
- **One single-lesson learning step costs ~12x a question (9.9 uJ), and the optimiser is most of it.** Adam touches all 16k
  parameters and keeps two extra numbers for each, which is 70% of that step's arithmetic and two thirds of its memory traffic. In a
  batch of 16 it shrinks to ~12%. So the optimiser matters exactly when updates are small and frequent, which is what *learning*
  is. (Plain SGD is cheaper per update but needed more updates or care: no clear win without tuning; see below.)
- **Implementation is a real lever.** A dot product written as eight independent sums instead of one lets the compiler use SIMD with
  no unsafe code and no special flags: **32 us -> 13 us per question (2.5x), same network, same answers.**

To get a real measurement here (until the next reboot, run once, then `baby energy` again):

```
sudo chmod a+r /sys/class/powercap/intel-rapl:0/energy_uj
```

## Learning instead of training (2026-10-15)

`baby life` gives a newborn baby (it knows its *words*, no *facts*) the same lessons in the same order, one at a time: colours and
sounds, then opposites, capitals, sums, over two "days". It tries each question, is told the right word, and has to cope. Five ways:

| method | what it does |
|--------|--------------|
| `naive` | keep nudging the weights on the new lesson only |
| `replay` | nudge the weights on the new lesson mixed with rehearsal of old lessons (memory of 128) |
| `store` | write the lesson into a fact store (word sets weighted by rarity); never touch the weights |
| `sleep` | store while awake; now and then replay the stored lessons into the weights (mixed with older ones), then forget what the weights now know |
| `retrain` | the conventional way: freeze, then rebuild from scratch on everything seen so far at the end of each stage |

Mean of 5 seeds, Adam, lr 0.01, energy estimated (`baby life --seeds 5`):

| method | retained | new phrasing | new sums | weight updates | energy (est.) | store at end |
|--------|---------:|-------------:|---------:|---------------:|--------------:|-------------:|
| naive | 9% | 6% | 6% | 6,947 | 75 mJ | 0 |
| replay | 82% | 29% | 6% | 2,908 | 103 mJ | 0 |
| **store** | **98.5%** | **97%** | 35% | **0** | **0.39 mJ** | 285 |
| **sleep** | **98.5%** | 56% | 18% | 1,200 | **66 mJ** | 42 |
| retrain | 100% | 55% | 15% | 6,600 | 366 mJ | 0 |

What it says (with the caveats that matter):

1. **Naive learning forgets almost everything** (9% retained): each new lesson overwrites the last. The classic result, and a clean
   baseline for anything cleverer.
2. **Writing it down is ~900x cheaper than retraining and forgets nothing, on a task made of facts.** The store scores 98.5% with zero
   weight updates for 0.39 mJ against retrain's 366 mJ. It also understands rewordings (97%) because it reads *sets* of words and
   weights rare ones ("peru") over common ones ("what", "the"), which is exactly the position-free reading the weights lacked. The
   caveat is fair: this world is all lookup, so a lookup structure wins. Its limits show in sums (35% on never-taught ones, likely
   mostly from commuted twins like "two plus three" after "three plus two", which a set cannot tell apart; not yet checked) and in
   its size: it grows with experience.
3. **Sleep gets retrain's retention for about 1/5 of the energy** (66 vs 366 mJ; 98.5% vs 100%): learn into the store while awake,
   then consolidate into the weights. The store shrinks from a peak of ~60 entries to 42.
4. **Consolidation moves facts but loses the store's flexibility.** Rewording scores fall from 97% (store) to 56% (after sleep),
   the same as a batch-trained network. The knowledge moved into weights; the position-free reading did not move with it. If the
   network read words the way the store does, consolidation would keep the generalisation. This is the strongest hint so far for
   what to change in the network.
5. **Replay is not enough here** (82% retained at 103 mJ): a small rehearsal buffer protects most of the past but not all, and
   it costs more than retraining per percentage point kept once sleeping is available.

Caveats, honestly: five seeds; hyper-parameters (steps per lesson, rehearsal size, sleep schedule) were set once and not tuned, so
the ranking between replay and sleep could move; the retrain baseline is a deliberately plain "start over" and a smarter conventional
pipeline would fine-tune with replay (which is what `replay`/`sleep` approximate); the energy figures are a model, not a meter;
this world is tiny and tidy.

Optimiser check (3 seeds): with plain SGD (lr 0.05-0.2) retrain needed ~338 mJ vs 366 mJ with Adam, only 8% less, because in
batches of 16 the optimiser is a small share of the work. SGD made single-lesson learning (replay) need more updates, and at low
learning rates sleeping never consolidated (the store kept everything and did the answering). So: no free win from dropping Adam
at this scale, but the *shape* of the cost (an optimiser that touches everything to learn one thing) is the target for something
smarter, such as updating only the rows a lesson actually involves.

## Position-free readers (2026-10-19)

The `life` results pointed at the reader: the store read *sets* of words and shrugged off rewording; the network glued each word to
its slot and did not. So the network now has three readers (`--reader concat|bag|bigram`, see `src/net.rs`): the original
`concat`; `bag`, the average of the words' embeddings (position-free, order-blind); and `bigram`, the bag plus hashed pairs of
neighbouring words (the fastText recipe: position-free, keeps local order). Gradients of all three are checked numerically in tests.

Batch training, four seeds (`baby train --reader R`):

| reader | parameters | new phrasing | new sums | notes |
|--------|-----------:|-------------:|---------:|-------|
| concat | 16,271 | 55-60% | 6-44% | the original |
| **bag** | **9,103** | **100%** (all four seeds) | 31-50% | smaller and better |
| bigram | 18,319 | 98.6-100% | 0% | no gain over the bag here |

Learning (`baby life --reader bag`, 5 seeds, Adam; the previous concat numbers in brackets):

| method | retained | new phrasing | updates | energy (est.) |
|--------|---------:|-------------:|--------:|--------------:|
| naive | 45% (9%) | 40% (6%) | 4,429 | 26 mJ |
| replay | 97% (82%) | 94% (29%) | 2,169 | 39 mJ (103) |
| store | 98.6% (98.5%) | 97% (97%) | 0 | 0.28 mJ (0.39) |
| **sleep** | **99.2%** (98.5%) | **98%** (56%) | 720 (1,200) | **20 mJ** (66) |
| retrain | 100% | 100% (55%) | 6,600 | 187 mJ (366) |

And one question now costs **7,104 multiply-adds (about 0.4 uJ estimated; 9 us measured here; 35 KB of weights)**, half of the
original reader's 14,272 multiply-adds, 799 nJ, 13 us and 63 KB.

What it says:

1. **The reader mattered more than any learning trick.** Changing how the network *reads* made it smaller, faster, cheaper to
   teach and fixed the flaw that consolidation had: after sleeping, the weights now answer rewordings 98% of the time (were 56%).
   Sleep with the bag reader spends about **1/18 of the energy of the original retrain** (20 vs 366 mJ) for the same retention.
2. **Position-free learning is also kinder to forgetting**: naive updates retain 45% instead of 9%, and replay 97% instead of 82%,
   probably because the lessons now share parts instead of each owning its slot (a hypothesis; not isolated).
3. **Sums are still not learned.** The bag's 31-50% on never-taught sums is mostly *commutativity*: I checked, and for three of the
   four seeds it equals exactly the share of held-out additions whose swapped twin ("two plus three" after "three plus two") was in
   training, with subtraction at 0%. The bigram reader, which can tell the order, got 0%: it memorised pair patterns. At this size
   and with ~100 examples, **none of the readers learns the addition rule**. That gap (memorising versus abstracting) is the central
   open problem for Adaptive Learning.
4. **The price is honesty about the unknown.** The old reader said "not sure" (confidence under 0.5) to "capital of atlantis". The
   bag answers *paris*, confidently, and also "what color is a zebra" -> green, "opposite of purple" -> slow. It generalises by
   association (a lone "banana" -> yellow is a nice one), and the same association makes it guess boldly when it should abstain.
   The store and a simple rule ("a word I have never met: say so and ask to be taught") are the obvious fixes, and a good first
   test of a surprise-gated learner.

## Knowing what it doesn't know, and being taught (2026-10-19)

After the bag reader made the baby guess boldly ("capital of atlantis" -> paris at 0.73), the question became: how does it know when to
hold back? `baby know` compares three signals. Each is calibrated the same way: set its threshold so that 95% of reworded
*real* questions still get answered, then count how many *unanswerable* questions it refuses. The unanswerable ones are of two
kinds: a word it has never met ("capital of atlantis", 90+ probes) and every-word-familiar nonsense ("capital of banana", "color
of france", 100+ probes). Mean of 4 seeds:

| refuses when... | reader | answers real | right when answering | refuses unmet words | refuses nonsense |
|---|---|---:|---:|---:|---:|
| (always answers) | concat / bag | 100% | 57% / 100% | 0% | 0% |
| confidence is low | concat | 95% | 60% | 25% | 1% |
| confidence is low | bag | 95% | 100% | 100% | 99.6% |
| **question is unfamiliar** | **both** | 96% | 55% / 100% | **100%** | **100%** |
| a word is unmet | both | 100% | - | 100% | 0% |
| all three | concat / bag | 91% / 92% | 58% / 100% | 100% | 100% |

What it says:

1. **My first fix was wrong, and the study caught it.** I had `ask` refuse below 0.5 confidence. For the bag reader that floor is far
   too low: it is ~1.000 confident on real questions, so the honest threshold sits right under that. A threshold that is not
   calibrated on the baby's own experience is a guess. The baby now **calibrates its own refusal thresholds at the end of training**
   and stores them in its file (`floors` line).
2. **Familiarity beats confidence, and does not care which reader is used.** Asking "has anything like this ever been taught to me?"
   (rare-word-weighted similarity to its taught questions) refused 100% of both kinds of unanswerable question for *both* readers
   while keeping 95.6% of real ones. Confidence only works if the network is well calibrated (bag: yes after calibration;
   concat: no, 25%). The unmet-word rule is free and perfect on unmet words, and blind to nonsense (0%).
3. **Caveats.** This world is tiny and the probes are clean. Familiarity may also refuse a legitimate question made of familiar
   parts it has never seen assembled (held-out sums), which this test did not measure. Combining all three costs coverage
   (92%, not additive).

And the baby can now be **taught** (`src/memory.rs`). `baby teach "capital of atlantis" poseidonis` writes a line to
`runs/<model>.memory` and the baby answers at once: no weights change, any new word, any new answer, kept across runs in a file you
can read and edit. A real session (bag reader):

```
capital of atlantis                  => ?   I haven't met the word atlantis. If you tell me, I'll remember: baby teach ...
capital of banana                    => ?   I don't know this one (nothing I was taught is like it) ...
baby teach "capital of atlantis" poseidonis
capital of atlantis                  => poseidonis   (from what you taught me, match 1.00)
what is the capital of atlantis      => poseidonis   (match 0.74)
the atlantis capital                 => poseidonis   (match 0.77)
capital of mars                      => ?   (a different place does not retrieve it)
capital of france                    => paris   (what it knew before is untouched)
```

Limits I can already see: a filler it has never met ("the capital of atlantis *please*") counts as an unmet word and makes it refuse;
the baby cannot yet learn that "please" is noise, which is something it would learn from experience. Taught facts live in the
memory, not the network: consolidating them ("sleep" for new words and answers) needs a vocabulary that can grow.

## Letters, not words (2026-10-19)

TransientVision Entry 4: the deviation may be deeper than architecture, perhaps not even tokens. The word readers carry a fixed word
list (a small cousin of the industry's token habit). `--reader letters` has none: a question is turned into hashed pieces of its own
spelling (the whole word, 3- and 4-letter groups with word-boundary marks, pairs of neighbouring words) and their embeddings are
averaged. Any word at all has a reading, including one it never met, and a misspelling keeps most of its pieces.
Gradients are checked numerically like the others (31 tests).

Misspellings: the same questions with one spelling mistake (drop, swap or replace a letter), in one word. "typos" is any word of 4+
letters; **"typo-key" is the word that carries the meaning** ("peru", "banana", "three"), which is where it matters. Mean of 3 seeds,
right answers %:

| trained with misspelt copies of each question | concat | bag | **letters** |
|---|---:|---:|---:|
| none | 11 | 10 | **37** |
| 2 copies | 14 | 12 | **44** |
| 4 copies | 14 | 12 | **48** |
| 8 copies | 15 | 12 | **61** |

(typo-key accuracy; no misspelt *word* used for testing ever appears in training, in any question). The word readers cannot learn
from misspellings they will be tested on, because each misspelling is simply a new unknown word: they stay at 10-15% however much
they see. The letters reader generalises to spellings it never met, and improves as it meets more noise. Along the way:

- **My first explanation was wrong.** I guessed the letters reader wasn't robust because the whole-word piece always appears next
  to its letter groups so the network had no reason to lean on the groups. Removing the whole-word piece changed nothing (64-67% vs
  64-65% on any-word typos). Rejected.
- **My first augmentation result was inflated.** The same misspelt word ("pru") could appear in training in one template and in the
  test in another, which helped the word readers unfairly (bag appeared to reach 42-51%). Filtering so that test misspellings never
  appear in training dropped the bag back to 12% and left the letters reader on top. Worth remembering: this kind of leak is easy
  and invisible.
- **The cost is small.** 15,455 parameters at 512 buckets (about concat's size, not bag's); a question reads only ~50 table rows.
  In the learning experiment (`baby life --reader letters`) it matches the bag on retention and rewording (sleep: 98.9% / 97.1%)
  for about 20% more energy (25 vs 20 mJ), mostly because Adam still updates the whole table to learn one thing: sparse updates are
  the obvious next saving.
- **A spelling-aware fact store is not worth it.** Letting the store compare spelling pieces (`--piece-keys`) lifts misspelling
  accuracy from ~1% to ~19% but costs about 4.5x the energy; the network after sleeping handles misspellings far better (~54%).
  Kept as an option, not the default.

## A baby that chooses what to learn (2026-10-19)

Adaptive Learning is not only learning from experience but steering it. `baby curious`: the baby overhears 429 questions (143 facts,
each asked three ways) but not their answers, and a teacher will tell it an answer *if it asks*. How few answers does it need?

- `random`: asks about everything in random order.
- `stream`: asks only after getting a question wrong (a teacher correcting mistakes, as in `life`).
- `curious`: asks about the overheard question least like anything it has been told, and stops when nothing is. It never needs the
  teacher to judge its mistakes.
- `checked`: curious, plus doubt: when nothing seems new, it asks the teacher to confirm 10 things it believes it knows, learning from
  any it got wrong.

Accuracy on the overheard questions / on rewordings it never overheard, mean of 5 seeds, similarity needed to count as a match 0.6:

| | N=60 | N=100 | N=143 | N=200 | N=300 | asked in total |
|---|---:|---:|---:|---:|---:|---:|
| random | 34/34% | 50/52% | 64/69% | 78/83% | 92/95% | 429 |
| stream | 37/37% | 58/58% | 77/77% | 96/92% | 100/93% | 215 |
| curious | 32/41% | 55/60% | 76/77% | 93/86% | 93/86% | **197** |
| checked | 32/41% | 55/60% | 76/77% | 93/87% | 94/87% | 229 |

What it says, honestly:

1. **Choosing helps a lot against not choosing**: curious needs about half the answers random does (197 vs 429) and reaches about the
   same accuracy as a teacher-corrected stream, without anyone judging its mistakes. At a stricter match (0.7) it also generalises
   to rewordings better per answer (73% vs 69% at 143 answers).
2. **But novelty is not correctness.** It stops at 93/86% because everything left *looks* familiar, including facts it has
   confused with similar ones. The stream baby is told when it is wrong, so it finishes at 100/93%.
3. **My idea of doubt did not work as built.** Ten random spot-checks cost 32 extra answers (229 vs 197) for about one point
   (94/87 vs 93/86). The errors it has left are rare and mostly confusable look-alikes (sums); random checking rarely lands on them.
   A better doubt would aim its checks at the places most likely to be confused (look-alike questions with different answers),
   which is an experiment worth running.
4. **Everything here depends on the match threshold** (how alike two questions must be). At 0.5 the stream baby finishes with 178
   answers; at 0.7 it needs 269. Calibrating that from experience, instead of choosing it, is part of making this adaptive.

## Rules, not tables (2026-10-19)

Every network so far memorised sums. Given "three plus two" it learned the pair, and "three plus one" was a stranger. `src/quantity.rs`
is a small unit with the two things a rule needs: a **number line**, learned the way a child gets one (taught to count: "what comes
after three? four"), and an **operator** per word, three learned numbers each, so a question's prediction is
`a*first + b*second + c` summed over its words. It is fitted on the same sums the networks see. `baby rules`, 5 seeds, right answers on sums:

| mechanism | taught | reworded | never taught (same numbers) | beyond the taught range (up to ten) |
|---|---:|---:|---:|---:|
| bag network (memorises) | 100% | 100% | 39% | 5% |
| letters network (memorises) | 100% | 100% | 33% | 5% |
| **quantity rule (fitted)** | 100% | 100% | **100%** | **100%** |

What the rule learned about words (nobody told it): `plus` -> (1.00, 1.00, 0.00), `minus` -> (1.00, -1.00, 0.00), and `what`, `is`,
`equals` -> (0, 0, 0): it found which words are noise, which was an open item on this list. It reaches `seven plus two` and
`nine minus four` although seven and nine never appeared in any question it was taught, because the counting lessons put them on the line.

**The baby chooses between them, from evidence it gathers itself.** While training, it holds back a fifth of the sums, learns with
both mechanisms from the rest, tests both on the held-back sums, and uses whichever does better (a tie goes to the cheaper). In every
seed: network ~41%, rule 100%, so numeric questions go to the rule. The choice and the evidence are printed and saved in the model file.

```
numeric questions: it chose the rule (on sums held back: network 11%, rule 100%)
seven plus two        => nine   (worked out by the rule, not recalled)
nine minus four       => five
what is sevn plus two => ?      I haven't met the word sevn ...        (it will not invent a missing operand)
```

Honesty about how much this is:

1. **I supplied the inductive bias.** The rule is linear in two quantities; that makes sums learnable from ~30 examples, and it is why
   nothing else here succeeds. What the baby did on its own was fit the operator, find the noise words, and decide, from held-back
   evidence, to prefer it. It did not invent the idea of quantity, and it was *taught to count first*.
2. **It chooses between mechanisms I gave it.** It cannot yet make a new one.
3. **A first version made things up.** Asked "what is sevn plus two" it answered "two" with full confidence: it had read one number
   and silently treated the other as zero. Now the rule declines unless every word is one it has met in a solved question and
   both operands are present.
4. **One rule is one rule.** Multiplication, comparison, or sequences are not covered; this is evidence that a rule *can* be learned
   from few examples with the right form, not that the baby finds forms.

## The baby adapts how it learns (2026-10-19; TransientVision Entry 5)

Entry 5: the process of learning itself should adapt. Two cases built so far, at different depths:

**How far to trust a match.** Every earlier experiment used a constant for "how alike two questions must be to trust a match" (0.6-0.75),
and results swung with it (stream baby, answers requested to finish: 178 at 0.5, 215 at 0.6, 269 at 0.7, 285 at 0.75). The `trusting`
baby learns it. Each time the teacher reveals a truth it records, per band of similarity, whether the match was right, and trusts a
band only if it was right often enough to be worth answering. "Often enough" is derived from costs: a wrong answer costs the question
that corrects it plus its own harm, so answering pays above harm/(1+harm) = 50% when harm equals one question. Across 5 seeds it
settled on **0.50 on average without being told** (the best fixed value I had found by sweeping), asked 193 answers where my hard-coded
0.75 needed 285, and ended at 100%/100%.

- **It failed first.** My first target was "right 90% of the time", which silently says a wrong answer is nine times worse than a
  question. It never moved off its starting 0.8, asked 325 answers, and answered no rewordings for the first 100. Deriving the target
  from costs fixed it. A test now pins the result across 5 seeds.

**Which mechanism to use.** The previous section: network or rule, chosen from the baby's own held-back evidence.

**Spelling.** `ask` no longer silently trips over a typo: a word it has never met that is one or two letters from exactly one word it
knows is taken to be that word, and it says so ("I took 'capitol' to mean 'capital'"). Words under five letters are left alone,
because "your" -> "four" would be a bad assumption. (The letters reader already reads some misspellings itself; this layer is
conventional, visible, and works for every reader.)

Still fixed by me and therefore not yet adaptive: steps per lesson, the sleep schedule, learning rate, the number of spot-checks.

## A library of ways to learn, chosen by evidence (2026-10-19)

`baby kinds`. Entry 5 says the mechanism of learning should adapt. Nine **units** of lessons, grouped the way a school would ("today:
comparing numbers"): `capital` (facts), `sum`, `compare` ("which is bigger three or five"), `after` ("what comes after four"), `double`,
`half`, `times`, `apart` ("how far apart are two and seven"), `code` ("what is the code for four": an arbitrary mapping with no
formula). The teacher says which lessons belong together and **never how to learn them**. Five mechanisms (`src/mechanisms.rs`):
**remember** (write questions down, answer with the most alike), **rule** (the quantity unit), **select** (bigger or smaller of two
numbers), **program** (search for an expression that reproduces every taught answer, see below), **network** (the bag net on that unit).
For each unit the baby holds back facts, learns with each mechanism from the rest, tests all five on the held-back facts (a
cross-validation it runs itself), and uses the best. A tie within two points goes to the cheaper one, and a mechanism must show
at least 25% on held-back facts to replace plain memory. Questions are sent to a unit by shape (every number counts as "a number").
Mean of 3 seeds, held-back accuracy %:

```
unit       remember   rule  select  program  network   chose
capital          0%     0%      0%       0%       0%   remember 3/3   <- nothing here generalises; it can only memorise this
sum             57%   100%     26%     100%      57%   rule 3/3
compare         83%    21%    100%       0%      86%   select 3/3
after            0%   100%      0%     100%       0%   rule 3/3
double           0%   100%      0%     100%       0%   rule 3/3
half             0%   100%      0%     100%       0%   rule 3/3
times           62%     5%     19%     100%      62%   program 3/3
apart           55%     0%      2%     100%      56%   program 3/3
code             0%     4%      0%       0%       0%   remember 3/3   <- nothing here generalises
```

| system (each unit counting equally) | reworded | new items | beyond what it was taught |
|---|---:|---:|---:|
| **the baby's own choices** | **100%** | **88%** | **100%** |
| remember everywhere | 100% | 34% | 6% |
| rule everywhere | 48% | 53% | 43% |
| select everywhere | 18% | 20% | 35% |
| program everywhere | 67% | 75% | 80% |
| network everywhere | 100% | 36% | 16% |

The 88% on new items is `code` scoring 0%: an arbitrary code cannot be known for a number never taught, and the baby says so. Every
other unit scores 100% on all three tests, and 100% of questions went to the right unit.

What it says:

1. **Nothing in the code says which mechanism suits which unit**, and every unit is treated alike. Facts go to remember, arithmetic-like
   units to the rule, comparison to select, and two kinds that the earlier four could not express (`times`, `apart`) to **program**.
   No single mechanism everywhere comes close on new and beyond-range questions.
2. **It builds a mechanism, not only chooses one.** For `times` the search wrote `(first x second)`; for `apart`, `(first apart
   second)`; for `after`, `(first + 1)`; for `half`, `half(first)`. Before `program` existed, `times` and `apart` were flagged as
   memorise-only (answers beyond what it was taught: 33% and 4%); now 100%.
3. **It did not "explain" what has no explanation.** On `code` (a fixed random-looking mapping) and `capital` the search found
   nothing, and the held-back test would have rejected a coincidence anyway.
4. **I took the convenient operation away to check it was not handed the answer.** `apart` is a primitive in the search. Without it
   (add, subtract, multiply, max and min only) it composed `((first - second) max (second - first))`: the absolute difference,
   built from subtraction and max, not the `max - min` I had predicted. My test expectation was wrong, the search was right.
5. **Two bugs the table exposed.** The small-fit default: with nothing above 25% the baby "chose" the rule for `code` on 4% noise
   and could no longer answer even the facts it was taught (0% on rewordings); the safe default is to remember. And the misrouting
   and confounded-credit fixes from the earlier version of this section, kept below.

Honesty about how much this is:
- **The language is mine.** The search composes add, subtract, multiply, max, min, absolute difference, halve, the first and second
  numbers, and the constants 1 and 2, up to seven pieces. It finds forms inside that space. Division with remainders, three
  numbers, or conditions are outside it; a unit needing them would be flagged memorise-only, as `times` was before.
- **The number line is taught**, and the units are given by the teacher.
- **Search cost grows fast with size.** The size limit of seven is what keeps the search instant (a few milliseconds per unit here).
- **"Beyond what it was taught" works only when the baby has seen the numbers somewhere** (the counting lessons put six to ten on the line).

The road there before `program`, because the dead ends are the useful part:
- **First run:** 80% on rewordings, all from `after` (0% on "what is after four"). I blamed the word "is" being unseen and
  let common words through: nothing changed.
- **Real cause:** the data cannot say whether "what" or "after" does the work (two of the three phrasings contain both), and a
  minimum-effort fit gave "what" a third of the credit. The fix is a prior: a word that turns up all over everything else is a poor
  suspect (pulled towards zero), and an operation word belongs to one kind of lesson.
- **First fix over-corrected:** I defined "all over" by frequency, which called "plus" common (there are many sums) and killed sums
  (0%). The right notion is **spread across units**, not frequency.
- **Misrouting (5%):** "zero plus six" went to `after`, because "six" happened to occur only in that unit's lessons. Matching by
  **shape** (every number is just a number) fixed it: 100% routing.

## Where the baby stands as Adaptive Learning (2026-10-19)

| AL ingredient | in the baby today |
|---|---|
| learns from interaction, not a fixed dataset | yes, in small: `teach`, `life`, `curious` |
| internals change in real time | yes: the store instantly; the weights during `sleep` |
| doesn't learn what it already knows | yes: only ~1/3 to 1/2 of lessons trigger learning |
| knows what it doesn't know | yes (familiarity, calibrated thresholds, "?" with a reason; flags units where nothing generalises) |
| chooses what to learn | partly: `curious` chooses what to ask, not what to explore |
| not tied to a token list | partly: the letters reader and spelling assumptions; answers still come from a fixed list |
| one system, honest about its limits | **yes**: `baby school`/`ask` run units, mechanisms and refusals together; 0% wrong on the scorecard, with a stated list of what it refuses |
| builds abstractions (rules, not tables) | **yes, within a small language**: it builds expressions for forms no fixed mechanism covers (`times`, `apart`) and they reach beyond what it was taught; the language of pieces and the number line are supplied |
| the learning process itself adapts | **more**: it learns how far to trust a match, picks the mechanism per kind of lesson from its own held-back evidence (new kinds included, no code), and builds a mechanism when none fits; steps, schedule and rate are still fixed |
| discovers on its own (browsing, other modalities) | **no**: a teacher answers; nothing explores |
| grows (a bare UMA becoming a learned UMA) | **no**: capacity is fixed up front |

## Next (candidates; the freedom to reorder is mine)

1. ~~Does learning stay light when the world gets bigger?~~ Answered for clean lessons up to 600 units (see the scale section; learning one lesson is flat at ~0.5 ms). Label noise is solved at 2 % and 5 % (robust fitting, cross-checks); open: other noise kinds, a Zipf-shaped vocabulary, discovery at scale, and the pen with thousands of waiting lessons.
2. **Compress what it keeps without forgetting anything**: a compact binary format, and the phrasings of a fact stored as templates plus facts.
3. **Incremental and exact updates**, sparse updates, small integers, and the real energy meter to replace time and memory as the measure.
4. **Let it propose its own mechanisms** when nothing generalises (the units that stay "memorise only").
5. ~~Digits~~ done; ~~a richer space to build rules in~~ done (floor-divide, remainder, a learned library). It now also asks questions (see "It asks what it wants to know"). Still open: discover the number line without counting lessons; mechanisms of a different *kind* (not an expression); the baby adapting its own numeric knobs (folds, search effort, thresholds) from what it measures; asking about rules, selections and facts.
