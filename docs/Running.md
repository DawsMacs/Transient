<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 DawsMacs -->

# Running Transient

How to get the code, build it, run it and check it, on each platform it has been tested on. Everything here was run, or comes from a CI log, on the dates given in
[`baby/README.md`](../baby/README.md) ("Running on other platforms"). Where something was **not** tried, it says so.

There are **no prebuilt binaries and no releases yet**: you build from source. The build has no dependencies and needs no network (`--offline`), takes about
a minute and a half on a recent desktop (one to four minutes on the CI machines), and the 150 tests take about 15 seconds.

## 1. Get the code

| way | what you need | size | notes |
|---|---|---|---|
| `git clone https://github.com/DawsMacs/Transient.git` | git | about 1.5 MB (the whole history) | no account or key needed; use the SSH address if you have write access |
| "Code → Download ZIP" on GitHub | a browser | about 0.5 MB | no history; you can build it the same way |
| `cargo install --git https://github.com/DawsMacs/Transient baby` | Rust | downloads and builds in about a minute | puts a `baby` command in `~/.cargo/bin`; tried on 2026-10-24 and it works; it installs only the `baby` program, not the tests, the goldens or the Python tools |

Within a clone, `cargo install --path baby` does the same from your local copy (about 76 seconds, tried).

## 2. What you need to build it

A **stable Rust toolchain** (edition 2021). Install it with [rustup](https://rustup.rs) on Linux, macOS and Windows, or from your system's package manager
on the BSDs. The CI builds use the current stable (1.99.1 on 2026-10-24); the BSD and DragonFly packages we tried are older and also built it
(FreeBSD 1.98.1, NetBSD 1.96.1, OpenBSD 1.94.1, DragonFly 1.85.1). The lowest version that works has **not** been established.

Rust itself needs a linker: on Linux a C compiler (`gcc` or `clang`), on macOS the Xcode command line tools (`xcode-select --install`), on Windows the
Microsoft C++ build tools (rustup offers to install them). These are standard Rust requirements, not something Transient adds.

## 3. Build, test and run

### Linux, macOS, and the BSDs (any Unix shell)

```bash
git clone https://github.com/DawsMacs/Transient.git
cd Transient
cargo build --release --offline
cargo test --release --offline                  # 150 tests, about 15 s
./target/release/baby school                    # teaches 13 units, prints how each was learned and the scorecard
./target/release/baby ask "how far apart are two and nine"
```

On the BSDs, install Rust from the system's repository first, as our CI does: `pkg install rust` (FreeBSD), `pkgin install rust` (NetBSD),
`pkg_add rust` (OpenBSD). As root or with `sudo`/`doas` as your system uses.

### Windows

The same commands work, with the program at `target\release\baby.exe`:

```
cargo build --release --offline
cargo test --release --offline
target\release\baby.exe school
target\release\baby.exe ask "how far apart are two and nine"
```

CI builds and tests on Windows (MSVC toolchain) with the commands running in Git Bash. The commands above are shell-independent, but I have not run them in
PowerShell or `cmd.exe` myself. The two shell scripts under `golden/` need `bash` (Git for Windows provides it).

### WebAssembly (WASI), in a sandbox

Transient compiles for `wasm32-wasip1` with no code change and runs under [Wasmtime](https://wasmtime.dev):

```bash
rustup target add wasm32-wasip1
cargo build --release --offline --target wasm32-wasip1
wasmtime run --dir . target/wasm32-wasip1/release/baby.wasm school
```

`--dir .` gives the program the current folder (where it keeps `runs/baby.school`). The test suite runs the same way:
`CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime run --dir . --dir /tmp --env TMPDIR=/tmp" cargo test --release --offline --target wasm32-wasip1`.
Not available there: anything that starts another program (`baby scale --sweep`, `baby exam adapt-drive`), and `baby compare` was not tried.
Download Wasmtime from its GitHub releases page and check the SHA-256 GitHub shows for the file; CI pins version 50.0.0 and verifies it.

### Other targets (32-bit x86, s390x, riscv64)

CI also builds and tests i686 (32-bit), s390x (big-endian) and riscv64 Linux, the last two under QEMU emulation. To do the same yourself you need that target's
cross toolchain and an emulator; the exact steps are in `.github/workflows/ci.yml` (job `emulated`). Emulated runs take minutes instead of seconds, and any timing or energy
reading from them is meaningless.

## 4. Check your build

The two-minute check: the tests pass, and the scorecard's **WRONG** column reads 0.0% on every row.

```bash
./target/release/baby school
./target/release/baby ask "what is seven times one"     # probably seven: a labeled guess, multiplication was only taught on 0..5
./target/release/baby ask "three minus five"            # ?  (must refuse: a negative result)
./target/release/baby ask "what is -3 plus 5"           # ?  (must refuse: a number it cannot read)
bash golden/check.sh                                    # about 1.5 minutes: the exam and adaptive answers must match golden/ byte for byte
```

If WRONG is ever above 0.0%, or a refusal starts answering, or a golden differs on the reference platform (Linux x86_64), treat it as a regression.
On another platform, a golden that differs is a **finding**, not something to patch over: please open an issue with the platform and the printed difference.

`baby school` writes its saved school to `runs/baby.school` in the current folder (the folder is created if needed). `baby ask` and the other commands read it.

## 5. What is tested where

CI (`.github/workflows/ci.yml`) builds and tests every push and pull request on these rows, and a separate weekly job
(`.github/workflows/wrong-teacher.yml`) checks the answers on streams with a wrong teacher:

| platform | how |
|---|---|
| Linux x86_64 (glibc), the **reference** | native |
| Linux x86_64 (musl), Linux aarch64 | native |
| macOS arm64 (Apple silicon), macOS x86_64 (Intel) | native |
| Windows x86_64 | native |
| FreeBSD 15.1, NetBSD 10.1, OpenBSD 7.9 | virtual machines |
| WebAssembly (WASI) | Wasmtime |
| Linux i686 (32-bit) | native on a 64-bit runner |
| Linux s390x (big-endian), Linux riscv64 | QEMU user-mode emulation |

On the 13 rows that run (2026-10-24): all tests pass, the scorecard reads 0.0% wrong, and the exam predictions, the adaptive-stream replies and the scorecard are
**byte-identical** to the Linux x86_64 reference, although the platforms' math libraries differ in the last bit of some results
(`cargo run --release --offline --example mathprint` prints a fingerprint and `-- audit` measures the differences: at most one step from the correctly rounded value).
DragonFly BSD was tried and dropped: its packaged Rust crashes the binary at startup inside the standard library, although the tests pass.

**This is "CI-tested on GitHub's runners", not a promise of support.** The runners are virtual machines running the real operating systems (or emulation);
other versions, other hardware and real s390x or riscv64 machines were not tried.

## 6. Energy and memory

- **Measured joules need Linux on hardware with a readable RAPL counter** (a bare-metal machine whose processor exposes it at `/sys/class/powercap/intel-rapl:0`; not a virtual machine or container; I have only verified this on an Intel desktop). The counter is
  root-only on current kernels. Making it readable is a system setting, so Transient never does it for you; if you want measured energy, run this yourself (it
  resets at reboot): `sudo chmod a+r /sys/class/powercap/intel-rapl:0/energy_uj`. Without it the energy report says it is unavailable.
- **Peak memory** is reported only where the system says it (`/proc/self/status` on Linux). Elsewhere it prints `n/a`, never zero.

## 7. Troubleshooting

- *"linker `cc` not found"* (Linux/macOS) or *"link.exe not found"* (Windows): install the linker toolchain from section 2.
- *`--offline` errors about the network*: there are no dependencies, so it should not happen; run `cargo build --release` without the flag and tell us.
- *A test fails*: run it alone with `cargo test --release --offline NAME -- --nocapture` and open an issue with the platform, the Rust version (`rustc -Vv`) and the output.
- *The Python tools under `tools/`* need `python3` (and `pytorch_baseline.py` needs PyTorch); they are only for the exam's comparison systems and are not needed to run Transient.
