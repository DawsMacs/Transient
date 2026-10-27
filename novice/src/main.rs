// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! `novice serve` speaks the entry exam's line protocol on standard input and output, so `baby exam adapt-drive DIR novice serve` can judge it.
//! `novice serve --refuse` is rung 0, kept as the floor: it learns nothing and refuses everything.
//! `novice static DIR [--out FILE]` takes the static entry exam (`baby exam write DIR`) and writes its predictions for `baby exam score DIR FILE`.

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
        Some("static") => {
            let Some(dir) = args.get(1) else {
                eprintln!("usage: novice static DIR [--out FILE]");
                std::process::exit(2);
            };
            let read = |name: &str| {
                std::fs::read_to_string(std::path::Path::new(dir).join(name)).unwrap_or_else(|e| {
                    eprintln!("novice: cannot read {dir}/{name}: {e}");
                    std::process::exit(1);
                })
            };
            let (lessons, questions) = (read("lessons.tsv"), read("questions.tsv"));
            let predictions = novice::static_exam::run(&mut Novice::default(), &lessons, &questions);
            match args.iter().position(|a| a == "--out").and_then(|i| args.get(i + 1)) {
                Some(out) => std::fs::write(out, predictions).unwrap_or_else(|e| {
                    eprintln!("novice: cannot write {out}: {e}");
                    std::process::exit(1);
                }),
                None => print!("{predictions}"),
            }
        }
        _ => eprintln!("usage: novice serve [--refuse]    |    novice static DIR [--out FILE]    (judge with `baby exam adapt-drive DIR novice serve`; the static exam: `novice static DIR --out FILE`, then `baby exam score DIR FILE`)"),
    }
}
