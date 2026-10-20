# Transient: direction and strategy

This is the long-form framing that used to be the front page. The technical overview is [`../README.md`](../README.md); the working state is in [`../TransientHandover.md`](../TransientHandover.md).

**Status (as of 2026-10-20):** active. `baby/` is a working, tested (61 tests) system: units of lessons, mechanisms chosen by evidence, honest refusals, learning after school, unit discovery, light cost (see `baby/README.md` and the handover); the larger architecture and the scale-up are still ahead. Founded 2026-10-15 by Daws. We pursue this whether or not it works; what we learn is
the point, and a failed hypothesis still redirects the work.

## Mission

(The why behind it, in Daws's own framing: [`TransientVision.md`](../TransientVision.md), a living document.)

Bring powerful AI to anyone's own hardware, and make it win *differently* from the cloud rather than trying to out-scale it. If a
few providers and a few countries with massive infrastructure are the only possible way to have capable AI, that is a choice
of design, not a law of nature. Transient is the attempt to find another way.

Transient may not turn out to be a model. It might be a model, a set of small models, a memory and tool architecture, a way of
spending time and compute, or all of these as one **system of intelligence** designed around consumer hardware. We do not know yet,
and that is the first thing the work has to find out.

## Winning differently (the game we choose)

The cloud's strengths come from scale: the most knowledge in the weights, the most compute per training run. Its structure also
imposes limits a personal system does not share. A light local system can win on axes where the cloud is *structurally* weak:

| Axis | Why a local system can win by construction |
|------|-------------------------------------------|
| **Compute per question** | A provider pays per token, so it spends the least compute that works. A local system's marginal cost is zero: it can think for minutes, try many candidates, check its own work, or keep working overnight. Time is the budget, not money. |
| **Knowledge** | Weights have a hard capacity; storage does not. A phone holds hundreds of GB. Knowledge in storage, read sparsely, has no practical ceiling. |
| **Personal context** | It can read what no cloud service should: your notes, history, files, habits. It can learn from you without that data ever leaving. |
| **Latency and availability** | No network, no queue, no outage, no rate limit, no policy change overnight. |
| **Cost and ownership** | Pay once; no subscription; nobody can revoke it; open to inspect. |
| **Adaptation** | It can be trained on its owner's own corrections, privately, on their own machine. |
| **Capability per watt, per byte** | The metric that decides whether "for anyone" is real. |

The goal is a system that is clearly better than a cloud model on some of these axes, on hardware people already own, while being
good enough on the rest. It does not need to win every benchmark to win where it matters to the person using it.

## Why this is reachable (reasons for optimism, sources are in private research notes that are not part of this repository)

- Capability density has been rising fast: the size needed for a given quality has been halving roughly every quarter.
- Retrieval and extra compute at answer time have let small models beat much larger ones on knowledge and math tasks.
- Respected voices (e.g. Karpathy's "cognitive core", NVIDIA's argument for teams of small models) already point at small,
  always-on, tool-using systems as a direction.
- On a phone, speed is limited by memory bandwidth, so a small model can make about ten calls in the time a 7B makes one.
  Systems that use many cheap steps fit this hardware better than one big step.
- Nobody I found has put sub-billion models, effectively unlimited external knowledge, specialist adapters and local compute
  budgets together as one designed thing. The opening is real.

## Relationship to other work

A separate, private project (a chat client with a small offline model) shares some of the lessons and the Rust tooling. It is not part of this repository.

## The terrain (facts that shape the design, not reasons to stop)

- **Decoding is memory-bandwidth-bound.** Speed is roughly bandwidth divided by bytes read per token (Snapdragon 8 Gen 3: rated
  77 GB/s). Design for arithmetic done per byte read, and for knowledge that lives in storage instead of RAM.
- **RAM is small, storage is large, compute is comparatively plentiful, heat and battery cap sustained work.**
- **Weights can hold only so much.** A model stores roughly 2 bits of facts per parameter; a 0.6B model is about 150 MB of
  facts. So facts belong in stores and the model's job is understanding, language and judgement. This is a design principle.
- **Architecture ideas earn trust by scaling behaviour.** Many look strong at 10-100M parameters. We compare how quality improves
  with size and with bytes read per token, so a good small result can be believed and a bad one explained.
- **Hardware and tooling are part of the design.** Ideas that run on ordinary matrix multiplies travel further than ones needing
  exotic kernels, so each idea is also costed in the kernels it needs.
- **We choose the metrics, but they must be honest.** Winning on cost, privacy and availability is real only if the answers are
  also good; the evaluation has to catch fluent nonsense (an earlier LLM-judge setup was lenient on it).

## Candidate directions (from memory, UNVERIFIED; phase 0 researches each properly)

| Direction | Idea | Why it may suit consumer hardware | Open question |
|-----------|------|-----------------------------------|---------------|
| Compute reuse | Looped or recurrent depth; more loops for harder tokens (Ouro, Tiny Recursive Model) | Depth without more weights to stream | Does it hold up in general chat, not just puzzles? |
| Knowledge as lookup | Memory layers, Engram-style tables, retrieval built into the architecture | Tables live in storage, read sparsely | How to train and update them |
| Non-attention mixers | State-space models, RWKV, xLSTM, gated delta nets, hybrids | Constant-size state, no growing cache | Exact recall, phone kernels |
| Ultra-low-bit weights | Ternary / BitNet-style | Far fewer bytes per token, cheap arithmetic | Training natively at that precision |
| Non-autoregressive generation | Diffusion language models | Many tokens per weight read | Quality and control |
| Adaptive compute | Early exit, mixture of depths, think only when needed | Effort where it matters | Routing cost |
| Fast weights | The model updates part of itself on context | Private, lasting personalisation | Stability, forgetting |
| System design | Router, specialists, verifier, stores and tools designed together | Matches the "many cheap steps" arithmetic | How much is architecture, how much engineering |
| Slow thinking | Asynchronous background work: research, verify, rehearse, overnight | Uses the one thing the cloud cannot give: unmetered time | What tasks benefit and how to present results |

A system answer and an architecture answer are not exclusive. Phase 0 maps which parts of each give the most per byte, per watt
and per second.

## A path (draft, to argue with)

- **Phase 0, framing and reading (now):** define "winning" as a scorecard over the axes above, choose a few concrete tasks that
  matter to a person (private recall, lookup, conversation, planning), and survey each direction with real numbers. Build a small
  cost model (tokens/s, bytes per token, watts) for a phone and a laptop so ideas compare on hardware terms.
- **Phase 1, testbeds:** (a) system experiments, many cheap calls plus stores and verifiers, using an existing evaluation harness; (b) small
  architecture experiments at 10-100M parameters on a rented GPU, tracking quality against parameters, bytes read and FLOPs.
- **Phase 2, grow what shows promise:** scale the ideas whose curves look good, and combine the architecture with the system.
- **Phase 3, real devices:** the S24 Ultra and a laptop, measuring latency, power and heat.
- **Checkpoints, not endings:** after each phase ask what the results taught. A hypothesis that fails tells us which axis to
  shift to, which idea to combine, or which piece to carry into other work. The work continues from there.

## Learning, not just training

"Pretrain, post-train, freeze" is a convention, not a law. For Transient, learning is a design question of its own: which
mechanism at which timescale (a store write, a fast-weight update, an overnight sparse consolidation, a rare base run), and how
efficiently each runs on the owner's hardware. Daws's working belief: data size is not the ceiling, implementation is. See
[`LEARNING.md`](../LEARNING.md) for the evidence, a ladder of learning timescales, efficiency levers, metrics and first experiments.

## Language and tooling (decided 2026-10-15: Rust-first, not Rust-only)

The project is the idea, not the language. Rust is the default for everything Transient owns; any layer can use something else
when a measurement says it is better. Pessimism is welcome as input to design (it tells us what to measure first); it never
decides whether to proceed.

**Why Rust-first**
- Transient's regime is small models and many small steps, where the cost around the kernels (interpreter, allocation, tokenizing,
  sampling, process startup, glue) is a bigger share of the total than it is for giant GPU-bound models. Reports of Rust
  rewrites show this: Rust inference servers beating a Python one by ~27% under load with ~20x faster startup, and Rust tokenizers
  5-10x faster [secondhand, from blog and repo claims; direction is plausible, numbers are not rigorous].
- Systems properties the design wants: one static binary, no runtime, predictable memory, zero-copy `mmap` of storage-resident
  knowledge tables (see "Knowledge as lookup"), safe concurrency for a multi-step system, cross-compilation to Android.
- The inertia argument is partly right. Frameworks and habits assume dense matmul + attention + a KV cache; a Rust stack built
  from scratch can put bandwidth, sparse reads and per-token compute budgets in the foundations. "Rust is not great for AI" often
  means "the Rust ecosystem is young", which is a different thing from a ceiling. And the author's other project is already Rust (wgpu and all), so
  code and lessons can flow between them.

**Terrain: what Rust does and does not change**
- The inner loop of bandwidth-bound decoding compiles through LLVM in Rust, C++ and Zig alike; the language does not make DRAM
  faster. Gains show up around the kernels, and in the confidence to build more aggressive designs. This matches what Daws has seen
  in application code.
- A concrete snag: the ARM instructions that fast quantized matmul uses (the int8 dot-product and matrix-multiply extensions, which I
  believe the S24's cores have) were still marked nightly-only in Rust's `std::arch` docs when checked. Options: a nightly
  toolchain, inline `asm!`, or calling existing C kernels (ggml) over FFI for the hot loop at first.
- Research velocity: most papers ship PyTorch/CUDA/Triton reference code (selective scans, FlashAttention). Burn has autodiff and
  training; its CubeCL lets one Rust kernel target CUDA, ROCm, Metal, Vulkan, WebGPU and CPU [sourced]. We did not find a ready
  state-space model or looped transformer in Burn, so expect to port ideas ourselves. Mitigation: a Python (or C) reference is
  allowed as a **test oracle** that produces golden outputs the Rust version must match, never as the runtime. Where a PyTorch
  scratch run answers a question much faster than a Rust port, use it, and note the exception.

**Layers (defaults, revisit with data)**

| Layer | Default | Note |
|-------|---------|------|
| System runtime (router, stores, tools, memory, scheduling) | Rust | Can share code with other Rust projects |
| CPU kernels | Rust + `std::arch`; FFI to ggml first | Verify against llama.cpp logits |
| Portable GPU compute | CubeCL / wgpu | wgpu is already used elsewhere; Android/Adreno path unverified |
| Training and architecture research | Burn (+ CubeCL), PyTorch as a conscious exception | Rented GPU for scale |
| Plots, notebooks, one-off analysis | Whatever is fastest | Never shipped |

**Alternatives to embrace if they measure better:** Mojo (fully open-sourced and 1.0 as of Aug 2026, built on MLIR, aimed at AI
kernels, Qualcomm-owned per the headline I found, which matters for a Snapdragon phone; compiler not yet taking outside
contributions), Zig (comptime, tiny runtime, easy C interop), MLIR/IREE/TVM-style compilers that generate kernels per target, and
Triton/CUDA for GPU kernels on rented machines. Rule: choose per layer from a micro-benchmark, not from allegiance.

**First Rust proof points (cheap, early, tell us where the wall really is)**
1. A Rust forward pass for Qwen3-0.6B that matches llama.cpp's logits to tolerance on the same GGUF.
2. Tokens/s against llama.cpp on the laptop, then the phone: how close can Rust get, and where does the gap live (kernel,
   threading, memory layout)?
3. Train a 10M-parameter model with Burn and with PyTorch on the same config and data: compare loss and throughput, and learn
   what ecosystem pieces are missing.

## Open questions

- What does "winning" mean concretely: which scorecard axes matter most, and what tasks should it excel at?
- What does the name mean, and should it shape the design? (Not known; not guessing.)
- Rust-first is decided; open: nightly toolchain or stable-with-FFI for the first kernels?
- Budget and acceptability of rented GPUs, and what data may be uploaded.
- ~~Public or private?~~ Decided 2026-10-20: public, AGPL-3.0-or-later (see `../README.md`, Licence).
