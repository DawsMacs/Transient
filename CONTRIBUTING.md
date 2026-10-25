# Contributing

Matter is open research and is licensed `AGPL-3.0-only`. Contributions are welcome under the same license.

## License and sign-off (DCO, no CLA)

There is no contributor license agreement. Instead, sign off each commit with the
[Developer Certificate of Origin](https://developercertificate.org/):

```bash
git commit -s -m "your message"
```

This adds a `Signed-off-by:` line stating you have the right to submit the work under the project's license. Contributions stay
under `AGPL-3.0-only`; nobody, including the maintainer, gets a special right to relicense them into a closed product. That is
deliberate: the point of the license is that this work stays open.

New source files start with:

```rust
// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) <year> <your name or handle>
```

## Ground rules that protect the results

- **Report WRONG separately from refused.** A new capability must not raise the WRONG column above 0.0 % (`baby school`).
- **Add must-refuse questions** for every new capability, and think adversarially about what could be answered but should not be.
- **Write down mistakes and rejected hypotheses** in `baby/README.md` as they happen; dead ends are part of the record.
- **State caveats plainly:** synthetic lessons, estimated (not measured) joules, thresholds and mechanisms that were chosen by hand.
- **Keep it light and dependency-free.** A dependency has to earn its place with a measurement.

## Before opening a pull request

```bash
cargo build --release --offline
cargo test --release --offline
./target/release/baby school      # WRONG must read 0.0 % on every row
golden/check.sh                   # the exam and adaptive-stream answers must match golden/ byte for byte
```

The goldens record what Matter answers on the reference platform (Linux x86_64). If your change is *meant* to change the answers, run `golden/check.sh update` in the same commit and say why in the message; if it is not, a difference is a regression. Changes to `.github/workflows` go through a pull request first (a new platform row can fail for reasons nobody has met yet), so `main` never carries the experiment.

## Pull requests and CI

Work on a branch and open a pull request; the maintainer merges. CI builds and tests every change on fourteen platform rows (Linux in several flavors, macOS, Windows, four BSDs, WebAssembly, and 32-bit and big-endian Linux). The reference row (Linux x86_64) must be green, and a golden difference there fails the build. On any other row a failure is a **finding**: say what the platform did and why, rather than widening a threshold or skipping the row. [`docs/Running.md`](docs/Running.md) explains how to build and run Matter on each platform. A pull request template and issue templates ask for the details that matter (platform, `rustc -Vv`, the exact question).
