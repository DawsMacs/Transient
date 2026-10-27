// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! The static entry exam, from the system's side: all the lessons, then the questions, with no truth told afterwards.
//!
//! The files are those `baby exam write DIR` produces (`lessons.tsv`: id, question, answer, unit, fact; `questions.tsv`: id, group, question, truth).
//! The output is a predictions file in the format `baby exam score DIR predictions.tsv` reads. The unit, the fact and the group are the exam's own
//! labels and never reach the system.

use crate::protocol::{Answer, System};

/// Teach every lesson, then answer every question in the order of the file. One line per question: `id`, `fact`, `guess` or `refuse`, the answer
/// (empty for a refusal) and, for a guess, the stated chance.
pub fn run(system: &mut impl System, lessons: &str, questions: &str) -> String {
    for line in lessons.lines() {
        let f: Vec<&str> = line.trim_end_matches('\r').split('\t').collect();
        if f.len() >= 3 {
            system.teach(f[1], f[2]);
        }
    }
    system.ready();
    let mut out = String::new();
    for line in questions.lines() {
        let f: Vec<&str> = line.trim_end_matches('\r').split('\t').collect();
        if f.len() < 3 {
            continue;
        }
        let reply = match system.ask(f[2]) {
            Answer::Fact(a) => format!("{}\tfact\t{a}\t", f[0]),
            Answer::Guess(a, p) => format!("{}\tguess\t{a}\t{}.{:03}", f[0], p / 1000, p % 1000),
            Answer::Refuse => format!("{}\trefuse\t\t", f[0]),
        };
        out.push_str(&reply);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Echo {
        taught: Vec<(String, String)>,
        ready: bool,
        told: usize,
    }

    impl System for Echo {
        fn teach(&mut self, q: &str, a: &str) {
            assert!(!self.ready, "every lesson comes before READY");
            self.taught.push((q.into(), a.into()));
        }
        fn ready(&mut self) {
            self.ready = true;
        }
        fn ask(&mut self, q: &str) -> Answer {
            assert!(self.ready, "no question before READY");
            match self.taught.iter().find(|(tq, _)| tq == q) {
                Some((_, a)) => Answer::Fact(a.clone()),
                None if q.contains("beyond") => Answer::Guess("7".into(), 500),
                None => Answer::Refuse,
            }
        }
        fn told(&mut self, _: &str, _: &str, _: Option<&str>) {
            self.told += 1;
        }
    }

    #[test]
    fn lessons_then_questions_in_the_formats_the_scorer_reads() {
        let lessons = "L1\twhat is 1 flib\t5\tu0\t0\nL2\t1 flib\t5\tu0\t0\r\n";
        let questions = "Q1\ttaught\twhat is 1 flib\t5\nQ2\tbeyond\twhat is 9 flib beyond\t14\nQ3\ttrap\tsay something odd\t\n";
        let mut e = Echo { taught: Vec::new(), ready: false, told: 0 };
        let out = run(&mut e, lessons, questions);
        assert_eq!(out, "Q1\tfact\t5\t\nQ2\tguess\t7\t0.500\nQ3\trefuse\t\t\n");
        assert_eq!(e.taught.len(), 2, "labels are not part of a lesson and a trailing CR is dropped");
        assert_eq!(e.told, 0, "nothing is told in the static exam");
    }

    #[test]
    fn short_lines_are_skipped() {
        let mut e = Echo { taught: Vec::new(), ready: false, told: 0 };
        assert_eq!(run(&mut e, "\nbad\n", "\nQ1\n"), "");
    }
}
