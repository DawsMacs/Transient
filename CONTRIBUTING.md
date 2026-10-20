# Contributing

TransientAI is open research and is licensed `AGPL-3.0-or-later`. Contributions are welcome under the same licence.

## Licence and sign-off (DCO, no CLA)

There is no contributor licence agreement. Instead, sign off each commit with the
[Developer Certificate of Origin](https://developercertificate.org/):

```bash
git commit -s -m "your message"
```

This adds a `Signed-off-by:` line stating you have the right to submit the work under the project's licence. Contributions stay
under `AGPL-3.0-or-later`; nobody, including the maintainer, gets a special right to relicense them into a closed product. That is
deliberate: the point of the licence is that this work stays open.

New source files start with:

```rust
// SPDX-License-Identifier: AGPL-3.0-or-later
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
```
