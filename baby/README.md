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

1. **Does learning stay light when the world gets bigger?** Test it: many more units, noisy lessons, overlapping phrasings, a vocabulary of thousands. The cheap path and the pen are untested there.
2. **Compress what it keeps without forgetting anything**: a compact binary format, and the phrasings of a fact stored as templates plus facts.
3. **Incremental and exact updates**, sparse updates, small integers, and the real energy meter to replace time and memory as the measure.
4. **Let it propose its own mechanisms** when nothing generalises (the units that stay "memorise only").
5. **Digits and number-like words**; a richer space to build rules in; discover the number line without counting lessons.
