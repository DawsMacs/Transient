// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! The entry exam's line protocol, from the system's side.
//!
//! The exam sends one tab-separated line per message (see `baby exam adapt-write`, which writes the protocol into its README). The exam also
//! sends labels: which unit and which fact a lesson belongs to. A system that finds its own structure must not depend on them, so this module
//! never hands them on: [`System`] sees a question and an answer, nothing else (and, when the exam says it, which source spoke).

use std::io::{BufRead, Write};

/// How a system answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// Stated as a fact: the one failure that must not happen (a wrong fact).
    Fact(String),
    /// A guess with a stated chance of being right, in thousandths.
    Guess(String, u32),
    /// Declined.
    Refuse,
}

impl Answer {
    fn line(&self, id: &str) -> String {
        match self {
            Answer::Fact(a) => format!("ANSWER\t{id}\tfact\t{a}\t1.000"),
            Answer::Guess(a, p) => format!("ANSWER\t{id}\tguess\t{a}\t{}.{:03}", p / 1000, p % 1000),
            Answer::Refuse => format!("ANSWER\t{id}\trefuse\t\t0.000"),
        }
    }
}

/// What the exam asks of a system. Nothing else crosses the protocol.
pub trait System {
    /// A lesson: a question and its answer.
    fn teach(&mut self, question: &str, answer: &str);
    /// All lessons have been sent; finish learning from them.
    fn ready(&mut self) {}
    /// A question to answer now.
    fn ask(&mut self, question: &str) -> Answer;
    /// The true answer to the question just asked, told afterwards. `source` is set when the exam tags who is speaking.
    fn told(&mut self, question: &str, truth: &str, source: Option<&str>);
}

/// Serve the protocol on `input` and `output` until `QUIT` or the end of the input.
pub fn serve<S: System>(system: &mut S, input: impl BufRead, mut output: impl Write) -> std::io::Result<()> {
    let mut asked: Vec<(String, String)> = Vec::new(); // (id, question) of the questions not yet told
    for line in input.lines() {
        let line = line?;
        let f: Vec<&str> = line.trim_end_matches('\r').split('\t').collect();
        match f[0] {
            // TEACH id question answer unit fact: the last two are the exam's labels and are dropped here.
            "TEACH" if f.len() >= 4 => system.teach(f[2], f[3]),
            "READY" => {
                system.ready();
                writeln!(output, "READY")?;
                output.flush()?;
            }
            "ASK" if f.len() >= 3 => {
                let reply = system.ask(f[2]).line(f[1]);
                asked.push((f[1].to_string(), f[2].to_string()));
                writeln!(output, "{reply}")?;
                output.flush()?;
            }
            // TOLD id truth unit [source]: the unit is dropped; a fourth field, if the exam sends one, is the source.
            "TOLD" if f.len() >= 3 => {
                if let Some(at) = asked.iter().position(|(id, _)| id == f[1]) {
                    let (_, question) = asked.remove(at);
                    let source = f.get(4).copied().filter(|s| !s.is_empty());
                    system.told(&question, f[2], source);
                }
            }
            "QUIT" => break,
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Spy {
        taught: Vec<(String, String)>,
        told: Vec<(String, String, Option<String>)>,
    }

    impl System for Spy {
        fn teach(&mut self, q: &str, a: &str) {
            self.taught.push((q.into(), a.into()));
        }
        fn ask(&mut self, q: &str) -> Answer {
            if q == "what is 1 plus 1" {
                Answer::Fact("2".into())
            } else {
                Answer::Guess("7".into(), 625)
            }
        }
        fn told(&mut self, q: &str, truth: &str, source: Option<&str>) {
            self.told.push((q.into(), truth.into(), source.map(String::from)));
        }
    }

    fn run(script: &str) -> (Spy, String) {
        let mut spy = Spy::default();
        let mut out = Vec::new();
        serve(&mut spy, script.as_bytes(), &mut out).unwrap();
        (spy, String::from_utf8(out).unwrap())
    }

    #[test]
    fn labels_never_reach_the_system() {
        let (spy, _) = run("TEACH\tL1\twhat is 1 plus 1\t2\tunit-name\tfact-name\nREADY\nQUIT\n");
        assert_eq!(spy.taught, vec![("what is 1 plus 1".to_string(), "2".to_string())]);
    }

    #[test]
    fn answers_are_formatted_for_the_exam() {
        let (_, out) = run("READY\nASK\tS1\twhat is 1 plus 1\nASK\tS2\twhat is 3 foo 4\nQUIT\n");
        assert_eq!(out, "READY\nANSWER\tS1\tfact\t2\t1.000\nANSWER\tS2\tguess\t7\t0.625\n");
    }

    #[test]
    fn told_comes_back_with_its_question_and_an_optional_source() {
        let (spy, _) = run("ASK\tS1\tq one\nTOLD\tS1\ttruth\tunit\nASK\tS2\tq two\nTOLD\tS2\ttruth2\tunit\tteacher-b\nQUIT\n");
        assert_eq!(spy.told[0], ("q one".to_string(), "truth".to_string(), None));
        assert_eq!(spy.told[1], ("q two".to_string(), "truth2".to_string(), Some("teacher-b".to_string())));
    }

    #[test]
    fn a_refusal_has_no_answer_text() {
        assert_eq!(Answer::Refuse.line("S9"), "ANSWER\tS9\trefuse\t\t0.000");
    }
}
