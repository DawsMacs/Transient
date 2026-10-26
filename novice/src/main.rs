// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! `novice serve` speaks the entry exam's line protocol on standard input and output, so `baby exam adapt-drive DIR novice serve` can judge it.
//! `novice serve --refuse` is rung 0, kept as the floor: it learns nothing and refuses everything.

use novice::learner::Novice;
use novice::protocol::{serve, Answer, System};

struct Refuser;

impl System for Refuser {
    fn teach(&mut self, _question: &str, _answer: &str) {}
    fn ask(&mut self, _question: &str) -> Answer {
        Answer::Refuse
    }
    fn told(&mut self, _question: &str, _truth: &str, _source: Option<&str>) {}
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("serve") => {
            let stdin = std::io::stdin();
            let result = if args.iter().any(|a| a == "--refuse") {
                serve(&mut Refuser, stdin.lock(), std::io::stdout().lock())
            } else {
                serve(&mut Novice::default(), stdin.lock(), std::io::stdout().lock())
            };
            if let Err(e) = result {
                eprintln!("novice: {e}");
                std::process::exit(1);
            }
        }
        _ => eprintln!("usage: novice serve [--refuse]    (speaks the entry exam's protocol; judge it with `baby exam adapt-drive DIR novice serve`)"),
    }
}
