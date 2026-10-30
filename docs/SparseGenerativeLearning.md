# Sparse generative learning: what the attempt taught us

Research retrospective, 2026-10-30. Matter Creation 1 (MC1) explored CPU-first sparse generative learning, starting with a 10 MB pilot objective. It is a research attempt, distinct from the Novice source release. Development is pausing; research and foundation maintenance continue. The long-term ambition remains useful learning on ordinary hardware. The evidence below does not establish that ambition.

## The approach

The first path used byte-context prediction: recent text selects sparse count tables, and a learned mixer combines their predictions with recent matches. A rebuild replaced hashed contexts with explicit context trees, added update exclusion, fitted count discounts, a document-local cache, learned mixing weights, and capped word-context stores. These are established statistical prediction and compression ideas implemented and compared under a CPU and memory budget. They are not evidence of a new general learning principle.

Frozen evaluation held the long-term trained counts fixed. Some variants still adapted a separate document-local cache and mixing state while reading a document. That distinction matters: frozen long-term parameters do not imply a stateless predictor. The reported metric is negative log probability in **bits per original byte**, including an end-of-document symbol; lower is better. It is not a conversation or reasoning score.

## Historical measurements

These are recorded experimental results, not benchmarks rerun for Novice's publication. Book-level splits and registered finalist checkpoints were used for the fresh corpora. Corpus text and learned checkpoints are not distributed with this source release.

| Experiment | Evaluation size | System | Bits per byte |
|---|---:|---|---:|
| Fresh Stage A, 10,386,184 cleaned bytes total | 6 held-out books, 1,431,657 bytes | plain count predictor | 2.2959 |
| same | same | static count predictor | 2.0912 |
| same | same | document-adapting milestone | 1.8058 |
| Fresh Stage B, 100,527,708 cleaned bytes total | 28 test documents, 10,388,729 bytes | static count predictor | 1.9913 |
| same | same | document-adapting milestone | 1.7761 |
| same | same | plus word contexts, 256 MiB store cap | 1.7255 |
| same | same | plus word contexts, 1,024 MiB store cap | 1.7135 |

Stage A trained on 8,347,592 bytes; Stage B on 79,322,718 bytes. Books were selected by a seeded content-blind rule from English Project Gutenberg text, then split by whole book; large books could form several documents within their assigned split. The material was a mixture of books, largely nonfiction, not a fiction benchmark. The test sets were scored after finalist registration; they are spent for further selection on these books.

On Stage B, recorded training time and peak process memory were 24.8 seconds / 489 MB for the milestone, 60.2 seconds / 822 MB with the 256 MiB word store, and 80.3 seconds / 1,897 MB with the 1,024 MiB store. These are single-machine measurements, not platform guarantees. The retained final scoring records contain no package-joule reading. No energy advantage is inferred from those times.

## Failures and rejected interpretations

1. **A better predictor was not necessarily a better energy trade.** The initial hashed mixer bought about a 2% improvement in frozen development loss over a PPM baseline while using about 14 times its learning energy and 7.6 times its serving energy on that experiment. Better quality alone did not meet the efficiency objective.
2. **More data gave diminishing gains.** Stage B's milestone validation loss went from 2.0780 at 1 MB to 1.8885 at 79 MB. The last step, 30 to 79 MB, improved from 1.9180 to 1.8885. The curve was flatter than predicted; it does not justify extrapolating to general capability.
3. **Word contexts cost real memory and work.** The 256 MiB store improved Stage B test loss 2.85% relative to the milestone but more than doubled training time. A larger cap improved loss further at higher process memory. Store capacity and quality must be reported together.
4. **Compression did not produce coherent conversation.** Generated samples still contained invented words, fragments, and control bytes. A lower next-byte loss did not establish usable language generation, factual reliability, or reasoning.
5. **Template tests overstated language handling.** A fact-memory path reached zero wrong answers on a generated 264-question bench after programmed repairs. A separate 45-question hand-written development set still had 6 wrong answers after repair (13.3% of all questions). The first registered changed run had 7 wrong (15.6%); retesting the repair did not make that file fresh evidence.
6. **A learned reader still depended on taught constructions.** On its supplied-template bench the reader answered 740 of 977 questions correctly with no wrong reader-path answers. On 17 items with unseen wording it answered only 3 correctly. A fallback based on shared words also gave wrong answers on role reversal and negation. Template consistency is not general language understanding.
7. **Some explanations of performance were wrong.** Instrumentation found duplicate computations after several incorrect guesses about the bottleneck. Timing under a slowed machine required alternating controls; normalized times were inferences, not direct measurements. A previously described 22% cache improvement at 100 KB was against an already-adaptive match-only control, not a static control.

## What we learned

Keep prediction quality, factual answering, linguistic transfer, memory, wall time, and joules as separate measurements. A cache can cheaply exploit repeated vocabulary without understanding roles or meaning. Training faster within one count-based family is useful engineering, but it does not by itself discover a stronger learning mechanism. A saturated generator should become development material, and unseen wording needs an independently designed test.

For the next research phase: register predictions and finalists before evaluation; keep the final test closed during tuning; score wrong answers and refusals separately; test role reversal, negation, and ambiguous wording; profile before optimizing; and report the total cost of learning plus use. Whether sparse learning can deliver broad useful capability, and whether experience can reduce lifetime energy per useful answer, remain open questions. These are research directions to investigate, not claims of success or a ceiling on every sparse design.
