---
name: Bug or build problem
about: It does not build, a test fails, or something behaves differently on your platform.
labels: bug
---

**What you ran** (the exact commands)

**What you expected, and what happened** (paste the output)

**Platform** (OS and version, architecture; a virtual machine or container?)

**Toolchain** (`rustc -Vv` and `cargo -V`)

**If a golden differs** (`bash golden/check.sh`): paste the printed difference. On a platform other than Linux x86_64 that is a finding we want to hear about, not a mistake on your side.
