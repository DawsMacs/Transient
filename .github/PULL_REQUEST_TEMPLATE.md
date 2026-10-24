## What this changes and why

<!-- One or two sentences. If it moves a number, say which and how you measured it. -->

## Checks (tick what you ran)

- [ ] `cargo test --release --offline` passes
- [ ] `./target/release/baby school`: the WRONG column reads 0.0 % on every row
- [ ] `bash golden/check.sh`: the exam and adaptive answers match `golden/` byte for byte, **or** this change is meant to change the answers and `golden/check.sh update` is in this commit with the reason in the message
- [ ] a new capability comes with must-refuse questions (what could be answered but should not be)
- [ ] mistakes and rejected ideas are written down in `baby/README.md`
- [ ] new text is en-US and new files carry the SPDX header; commits are signed off (`git commit -s`)

## Caveats

<!-- Synthetic lessons, estimated joules, one seed, one platform: say what this does not show. -->
