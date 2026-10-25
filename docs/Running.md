<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 DawsMacs -->

# Running Matter

How to get the code, build it, run it and check it, one section per platform. Everything here was run, or comes from a CI log, on the dates given in
[`baby/README.md`](../baby/README.md) ("Running on other platforms"). Where something was **not** tried, it says so.

There are **no prebuilt binaries and no releases yet**: you build from source. The build has no dependencies and needs no network (`--offline`), takes about
a minute and a half on a recent desktop (one to four minutes on the CI machines), and the 152 tests take about 15 seconds.

## Pick your platform

| your system | section | how it is tested |
|---|---|---|
| Linux (Debian, Ubuntu, Fedora, Arch, Alpine, others), x86_64 or aarch64 | [Linux](#linux) | CI, native (glibc and musl) |
| macOS on Apple silicon or Intel | [macOS](#macos) | CI, native |
| Windows 10 or 11 | [Windows](#windows) | CI, native (MSVC toolchain, Git Bash) |
| FreeBSD | [FreeBSD](#freebsd) | CI, virtual machine |
| NetBSD | [NetBSD](#netbsd) | CI, virtual machine |
| OpenBSD | [OpenBSD](#openbsd) | CI, virtual machine |
| Any system with a WebAssembly runtime (sandboxed) | [WebAssembly (WASI)](#webassembly-wasi) | CI, under Wasmtime |
| 32-bit x86, s390x (big-endian), riscv64 | [Other processors](#other-processors) | CI, native or QEMU emulation |
| DragonFly BSD | [DragonFly BSD](#dragonfly-bsd) | not tested: the first try crashed at startup; a workaround is described, untested |

Anything not in this table is **untested**, which does not mean it fails: Matter is portable Rust with no dependencies and no `unsafe`, so a system with a
working Rust toolchain has a good chance. If you try one, please tell us how it went (see [Reporting](#reporting-a-problem-or-a-result)).

**In a hurry, and you already have Rust and git?** On Linux, macOS or a BSD:

```bash
git clone https://github.com/Matterworm/Matter.git && cd Matter
cargo build --release --offline && ./target/release/baby school
```

## Before you start: the parts every platform shares

### Getting the code

| way | what you need | size | notes |
|---|---|---|---|
| `git clone https://github.com/Matterworm/Matter.git` | git | about 1.5 MB (the whole history) | no account or key needed; use the SSH address if you have write access |
| "Code → Download ZIP" on GitHub | a browser | about 0.5 MB | no history; you can build it the same way |
| `cargo install --git https://github.com/Matterworm/Matter baby` | Rust | downloads and builds in about a minute | puts a `baby` command in `~/.cargo/bin`; tried on 2026-10-24 on Linux and it works; it installs only the `baby` program, not the tests, the goldens or the Python tools |

Within a clone, `cargo install --path baby` does the same from your local copy (about 76 seconds, tried).

### What you need to build it

A **stable Rust toolchain** (edition 2021) and the linker that Rust itself needs on your system. The platform sections below say how to get both. The CI builds
use the current stable (1.99.1 on 2026-10-24); the BSD packages we tried are older and also built it (FreeBSD 1.98.1, NetBSD 1.96.1, OpenBSD 1.94.1). The
lowest version that works has **not** been established.

### Checking your build

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

---

## Linux

**Tested in CI:** Ubuntu 24.04 on x86_64 (glibc, the **reference** every other platform is compared with), Ubuntu 24.04 on aarch64 (glibc), and an x86_64 **musl**
build. **Not tried:** Alpine itself (the musl build ran on Ubuntu), Raspberry Pi hardware, and every distribution other than Ubuntu. The package commands below
are each system's standard ones, not something I ran on a fresh machine.

### 1. Install the build tools and git

| distribution | command |
|---|---|
| Debian, Ubuntu | `sudo apt update && sudo apt install -y git curl build-essential` |
| Fedora | `sudo dnf install -y git curl gcc` |
| Arch | `sudo pacman -S --needed git curl base-devel` |
| Alpine | `apk add git curl build-base` (run as root) |

### 2. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh     # accept the defaults, then open a new terminal
rustc --version                                                     # any recent stable
```

You can use your distribution's `rust` and `cargo` packages instead; they work if they are recent enough (the lowest version that works is not established).
On Alpine, `apk add rust cargo` is the easy way and gives the musl build.

### 3. Build, test and run

```bash
git clone https://github.com/Matterworm/Matter.git
cd Matter
cargo build --release --offline
cargo test --release --offline                  # 152 tests, about 15 s
./target/release/baby school                    # teaches 13 units, prints how each was learned and the scorecard
./target/release/baby ask "how far apart are two and nine"
```

Then run [the check](#checking-your-build). **Joules** (the energy report) need a readable RAPL counter; see [Energy and memory](#energy-and-memory).

---

## macOS

**Tested in CI:** macOS 15 on Apple silicon (arm64) and on Intel (x86_64), native, both byte-identical to the Linux reference. **Not tried:** other macOS
versions. Energy is not measured on a Mac (no RAPL), and peak memory prints `n/a`.

### 1. Install the command line tools (the linker)

```bash
xcode-select --install
```

This also installs `git`. If a dialog appears, accept it and wait for it to finish.

### 2. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh     # accept the defaults, then open a new terminal
```

Or, if you use Homebrew: `brew install rust`.

### 3. Build, test and run

```bash
git clone https://github.com/Matterworm/Matter.git
cd Matter
cargo build --release --offline
cargo test --release --offline                  # 152 tests, about 15 s
./target/release/baby school
./target/release/baby ask "how far apart are two and nine"
```

Then run [the check](#checking-your-build).

---

## Windows

**Tested in CI:** Windows Server 2022 (the same code base as Windows 10 and 11), x86_64, MSVC toolchain, with the commands running in **Git Bash**. The
program itself is the same on every shell, but I have **not** run the build and run commands in PowerShell or `cmd.exe` myself. Windows on Arm was not tried.
Energy is not measured on Windows, and peak memory prints `n/a`.

### 1. Install Git for Windows

Download it from [git-scm.com](https://git-scm.com/download/win), or in PowerShell: `winget install --id Git.Git -e`. It includes **Git Bash**, which the two shell
scripts under `golden/` need.

### 2. Install Rust and its linker

Download and run `rustup-init.exe` from [rustup.rs](https://rustup.rs). If the Microsoft C++ build tools are missing, it offers to install them (choose
"Desktop development with C++"); accept, and wait: it is a large download. Then open a **new** terminal and check `rustc --version`.

### 3. Build, test and run

In Git Bash the Linux commands work as written:

```bash
git clone https://github.com/Matterworm/Matter.git
cd Matter
cargo build --release --offline
cargo test --release --offline
./target/release/baby.exe school
```

In PowerShell or `cmd.exe` the program is at `target\release\baby.exe`:

```
cargo build --release --offline
cargo test --release --offline
target\release\baby.exe school
target\release\baby.exe ask "how far apart are two and nine"
```

Then run [the check](#checking-your-build) in Git Bash (`bash golden/check.sh`).

---

## FreeBSD

**Tested in CI:** FreeBSD 15.1, x86_64, in a virtual machine, with the packaged Rust (1.98.1).

```bash
sudo pkg install -y rust git          # as root, or with doas
git clone https://github.com/Matterworm/Matter.git
cd Matter
cargo build --release --offline
cargo test --release --offline        # 152 tests
./target/release/baby school
./target/release/baby ask "how far apart are two and nine"
```

Then run [the check](#checking-your-build). Energy is not measured here, and peak memory depends on what the system reports (it prints `n/a` when it does not).

---

## NetBSD

**Tested in CI:** NetBSD 10.1, x86_64, in a virtual machine, with the packaged Rust (1.96.1). This needs `pkgin`; if it is not set up, install it
from the NetBSD pkgsrc binary packages first.

```bash
sudo pkgin -y install rust git        # as root, or with sudo if you set it up
git clone https://github.com/Matterworm/Matter.git
cd Matter
cargo build --release --offline
cargo test --release --offline        # 152 tests
./target/release/baby school
./target/release/baby ask "how far apart are two and nine"
```

Then run [the check](#checking-your-build). NetBSD's `/proc/self/status` is in a different format from Linux's, so peak memory prints `n/a` there (it never prints zero).

---

## OpenBSD

**Tested in CI:** OpenBSD 7.9, x86_64, in a virtual machine, with the packaged Rust (1.94.1). OpenBSD is a "tier 3" Rust target (no official builds, so use the
system's package, as CI does).

```bash
doas pkg_add rust git                 # or sudo, if you use it
git clone https://github.com/Matterworm/Matter.git
cd Matter
cargo build --release --offline
cargo test --release --offline        # 152 tests
./target/release/baby school
./target/release/baby ask "how far apart are two and nine"
```

Then run [the check](#checking-your-build).

---

## DragonFly BSD

**Not tested in CI.** We tried DragonFly 6.4.2, x86_64, in a virtual machine on 2026-10-24, with its packaged Rust (1.85.1, a "tier 3" target). The 149 tests
passed, but running `baby` died at startup with `fatal runtime error: assertion failed: key as usize != KEY_SENTVAL`, inside Rust's standard library, before any
of Matter's code ran. A second try on 2026-10-25 could not even install Rust: the virtual machine could not resolve DragonFly's package host that day.

**What is probably happening (an explanation from reading, not yet confirmed here).** The same error was reported to Rust as
[rust-lang/rust#112180](https://github.com/rust-lang/rust/issues/112180): on DragonFly, a program that is not linked against the thread library can be handed
thread-local key 0 by `pthread_key_create`, and Rust's standard library treats 0 as "not set yet" and aborts. A comment on that issue says that with link-time
optimization the linker drops the thread library when nothing in the program asks for it. Matter's release profile uses full link-time optimization
(`lto = true`), and the tests passed because the test program starts threads, which keeps the thread library linked. That fits everything we saw. The Rust issue was
closed as "not planned" in June 2025 without a fix. Matter's own thread-local (the reliability bar in `baby/src/mechanisms.rs`) is a plain number with no
destructor and is not what triggers it.

**Source.** The code is at [github.com/Matterworm/Matter](https://github.com/Matterworm/Matter) (AGPL-3.0-only); get it as described in
[Getting the code](#getting-the-code). There is no prebuilt binary, and the Linux build cannot be used here: DragonFly removed its Linux emulation in 2016 and
2017 ([the commit](https://lists.dragonflybsd.org/pipermail/commits/2017-February/714870.html)), and as far as we found it only ran 32-bit programs.

**To try building from source (the workaround is untested):**

```bash
sudo pkg install -y rust git          # as root, or with doas
git clone https://github.com/Matterworm/Matter.git
cd Matter
CARGO_PROFILE_RELEASE_LTO=false cargo build --release --offline     # build without link-time optimization
CARGO_PROFILE_RELEASE_LTO=false cargo test --release --offline      # 152 tests
./target/release/baby school
```

Then run [the check](#checking-your-build), including `bash golden/check.sh`: building without link-time optimization should not change any answer, but that has
not been checked on this system, and the program may run a little slower. If `baby school` still dies with the same message, please tell us (see
[Reporting](#reporting-a-problem-or-a-result)) with the output of `uname -a` and `rustc -Vv`: a report from a real DragonFly machine, or a newer Rust there, is
what we are missing.

---

## WebAssembly (WASI)

**Tested in CI:** `wasm32-wasip1` under [Wasmtime](https://wasmtime.dev) 50.0.0, on Linux. This runs Matter in a sandbox: it can see only the folders you give
it. It is the way to run it on a system with no native build.

### 1. Install Rust, the target and Wasmtime

Install Rust as in [Linux](#linux), [macOS](#macos) or [Windows](#windows), then:

```bash
rustup target add wasm32-wasip1
```

Download Wasmtime from its [GitHub releases page](https://github.com/bytecodealliance/wasmtime/releases) and check the SHA-256 that GitHub shows for the file.
CI pins version 50.0.0 and verifies it.

### 2. Build and run

```bash
cargo build --release --offline --target wasm32-wasip1
wasmtime run --dir . target/wasm32-wasip1/release/baby.wasm school
wasmtime run --dir . target/wasm32-wasip1/release/baby.wasm ask "how far apart are two and nine"
```

`--dir .` gives the program the current folder (where it keeps `runs/baby.school`). The test suite runs the same way:

```bash
CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime run --dir . --dir /tmp --env TMPDIR=/tmp" cargo test --release --offline --target wasm32-wasip1
```

**Not available there:** anything that starts another program (`baby scale --sweep`, `baby exam adapt-drive`); `baby compare` was not tried. There is no energy
or memory report.

---

## Other processors

CI also builds and tests **i686 (32-bit x86)** natively on a 64-bit runner, and **s390x (big-endian)** and **riscv64** Linux under QEMU user-mode emulation. All three
give byte-identical answers to the reference. To do the same yourself you need that target's cross toolchain and an emulator (on Ubuntu, for example):

```bash
rustup target add riscv64gc-unknown-linux-gnu
sudo apt-get install -y gcc-riscv64-linux-gnu libc6-dev-riscv64-cross qemu-user
export CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_LINKER=riscv64-linux-gnu-gcc
export CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_RUNNER="qemu-riscv64 -L /usr/riscv64-linux-gnu"
cargo test --release --offline --target riscv64gc-unknown-linux-gnu
```

The exact packages, linkers and runners for each of the three are in `.github/workflows/ci.yml` (job `emulated`). Emulated runs take minutes instead of
seconds, and any timing or energy reading from them is meaningless. Real s390x and riscv64 machines were not tried.

---

## Energy and memory

- **Measured joules need Linux on hardware with a readable RAPL counter** (a bare-metal machine whose processor exposes it at `/sys/class/powercap/intel-rapl:0`; not a
  virtual machine or container; I have only verified this on an Intel desktop). The counter is root-only on current kernels. Making it readable is a system
  setting, so Matter never does it for you; if you want measured energy, run this yourself (it resets at reboot):
  `sudo chmod a+r /sys/class/powercap/intel-rapl:0/energy_uj`. Without it the energy report says it is unavailable.
- **Peak memory** is reported only where the system says it (`/proc/self/status` on Linux). Elsewhere it prints `n/a`, never zero.

## Updating and removing

- **Update:** `git pull --ff-only`, then build again (`cargo build --release --offline`) and re-run the check. If you used `cargo install`, run it again with `--force`.
- **Remove:** delete the folder you cloned (the build is in its `target/` folder, and the saved schools are in `runs/`). If you used `cargo install`, run
  `cargo uninstall baby`. Matter keeps its files in the folder you run it in (`runs/`), plus temporary files in your system's temp folder (or `TMPDIR`) for a few commands.

## What is tested where

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

**This is "CI-tested on GitHub's runners", not a promise of support.** The runners are virtual machines running the real operating systems (or emulation);
other versions, other hardware and real s390x or riscv64 machines were not tried.

## Troubleshooting

- *`cargo: command not found`* right after installing Rust: open a new terminal (or run `source "$HOME/.cargo/env"`).
- *"linker `cc` not found"* (Linux, the BSDs) or *"link.exe not found"* (Windows): install the build tools from your platform's step 1 or 2.
- *`--offline` errors about the network*: there are no dependencies, so it should not happen; run `cargo build --release` without the flag and tell us.
- *`bash: golden/check.sh` fails on Windows*: run it in Git Bash, not PowerShell.
- *A test fails*: run it alone with `cargo test --release --offline NAME -- --nocapture` and open an issue with the platform, the Rust version (`rustc -Vv`) and the output.
- *The Python tools under `tools/`* need `python3` (and `pytorch_baseline.py` needs PyTorch); they are only for the exam's comparison systems and are not needed to run Matter.

## Reporting a problem or a result

Open an issue on [GitHub](https://github.com/Matterworm/Matter/issues) (there is a template for a wrong answer and one for a bug). For a platform, say
the operating system and version, the processor, the output of `rustc -Vv`, and what you ran. A platform that works is as useful to hear about as one that does
not, and a golden that differs from `golden/` on another platform is a finding we want.

## Coming later

- **DragonFly BSD in CI** once a build there is known to run: see [DragonFly BSD](#dragonfly-bsd) for what we know and the workaround to try.
- **Prebuilt binaries and a first release** are not decided yet; until then, build from source as above.
- **Real peak memory outside Linux** (it prints `n/a` today).
