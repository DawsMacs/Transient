# Matter repo instructions

**Naming (2026-10-24):** the project and the repo are **Matter** (`Matterworm/Matter`). The **Matterworm Foundation** is the *creator* of Matter (as OpenAI is to ChatGPT); Daws is its owner, founder and creator, so
copyright and license headers naming `DawsMacs` are correct. Never call the project "Matterworm" or "Matterworm Foundation"; the meaning of the name "Matter" is Daws's to tell, so never guess it in docs.
The project and its creator were renamed on 2026-10-24 after a possible trademark conflict (unconfirmed). **The two earlier names are retired: do not use them in code, docs, site copy, commit messages or replies**
(they are written down once, in `MatterVision.md` Entry 11, and in the dated rename notes of `MatterHandover.md`; the site is served from the GitHub address `matterworm.github.io` for now, because Daws postponed buying a domain).

Matter is Daws's long-horizon project: a light, adaptive "system of intelligence" meant to run on ordinary consumer hardware, winning differently from cloud models (not out-scaling them). It is pursued
whether or not it works. The code is `baby/` (pure Rust, zero dependencies, builds offline).

## Start of session

**The rename (2026-10-24) is prepared on the local branch `rename` and waits for Daws's approval** (see "THE RENAME" at the top of `MatterHandover.md` for what was done, what is left and what only he can do). Until it is merged, `main` still carries the old names.

1. Read `MatterHandover.md` in full (rewritten 2026-10-21: state, decisions that are Daws's, options, guards, working lessons). It has the state, the numbers, the code map, the plan and what is next, the honest caveats, and the working habits. Then `MatterVision.md` (an **internal** document, rewritten 2026-10-24 at Daws's request in plain, practical
   wording: append dated entries for new decisions, label the assistant's context, tag ideas Decision / Hypothesis / Measured / Open; never link it from public pages).
2. Run `git pull --ff-only` and `gh pr list --state all` (pull requests wait for the user to merge them), then the two-minute check in the handover (`cargo test --release --offline`; `./target/release/baby school`; the scorecard's WRONG column must read 0.0%; `bash golden/check.sh`).

## Rules

- **Communication protocol: en-US.** All writing (code, identifiers, comments, runtime strings, docs, site copy, commit messages, replies) is en-US: spelling and vocabulary (color, behavior, license, organize, recognize, center, percent, toward). This holds from now on for all work, not only for Novice (which, rebuilt from scratch, is en-US from its first line). Stick with en-US; check new text before committing.
- **Goldens and pull requests.** The exam and adaptive-stream answers are pinned in `golden/` (`golden/check.sh`); a change that is meant to change answers runs `golden/check.sh update` in the same commit and says why. Changes to `.github/workflows` go through a pull request, and the user merges.
- **Honesty over polish.** Report WRONG separately from refused. Record mistakes and rejected hypotheses in `baby/README.md` as they happen. State caveats plainly (synthetic tidy lessons; estimated joules; mechanisms and thresholds are
  the assistant's, not discovered).
- **Pessimism is input, never a brake.** Daws wants risks phrased as design constraints/terrain/checkpoints that redirect the work. Do not propose kill criteria or stop the project over difficulty; do not claim certainty of success.
- **Vocabulary (2026-10-24).** Daws retired three labels he had coined: the architecture acronym beginning with U ("UMA" and its expansion), "Maximization", and "ultragenerative". Do not use or reintroduce them in code, docs, site copy or replies; say what is meant in plain, testable terms (capability per byte, parameter and joule). **Adaptive Learning** is the exception and stays. "Frontier" is an ambition, not a claim: say so wherever it appears.
- **Absolute freedom, within the stated goals** (frontier-class intelligence on typical hardware as the ambition, light on RAM/memory, built around Adaptive Learning, Rust-first, measured in joules not just parameters). Default to doing the plan.
- **After every new capability, ask adversarial questions** (the apart/minus/"N less than ten" ambiguity bit three times) and keep batteries of must-refuse questions in the tests.
- **Do not run system-setting commands** (e.g. the RAPL `chmod`): give Daws the command instead. **This is a git repo** (`origin` = `git@github.com:Matterworm/Matter.git`, public, AGPL-3.0-only; use his global git account). He has authorized commits and pushes to `origin main` for this work; still ask before force-pushing, adding other remotes or changing repo settings. The repo is public: never commit `.local/` or anything private.
