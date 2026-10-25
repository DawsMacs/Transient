// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! `novice serve` speaks the entry exam's line protocol on standard input and output, so `baby exam adapt-drive DIR novice serve` can judge it.

use novice::protocol::{serve, Answer, System};

/// Rung 0: learns nothing and refuses everything. It exists to prove the harness end to end (the exam can drive a system that is not baby) and to
/// set the floor: zero wrong facts, zero right ones.
struct Refuser;

impl System for Refuser {
    fn teach(&mut self, _question: &str, _answer: &str) {}
    fn ask(&mut self, _question: &str) -> Answer {
        Answer::Refuse
    }
    fn told(&mut self, _question: &str, _truth: &str, _source: Option<&str>) {}
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("serve") => {
            let stdin = std::io::stdin();
            if let Err(e) = serve(&mut Refuser, stdin.lock(), std::io::stdout().lock()) {
                eprintln!("novice: {e}");
                std::process::exit(1);
            }
        }
        _ => eprintln!("usage: novice serve    (speaks the entry exam's protocol; judge it with `baby exam adapt-drive DIR novice serve`)"),
    }
}
