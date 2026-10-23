// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! The entry exam: a way to judge any system, not only this one.
//!
//! Vision Entry 10: every new version is rebuilt from scratch, using the previous work as reference, so what carries over must not be code. What carries over
//! is the exam. `write` makes a directory of plain text files (the lessons a system is taught, and the questions it is then asked, in groups); a system in any
//! language learns from `lessons.tsv`, answers `questions.tsv`, and writes `predictions.tsv`; `score` judges the predictions by rules that do not depend on how
//! the system works; `run` does all of it for the baby itself, which is the bar a successor starts from.
//!
//! The rules it enforces are the ones this project holds itself to: an answer stated as fact that is wrong is the failure that matters (it must be 0), a trap
//! (a question that must be refused: a negative result, an unreadable number, a word nobody taught) must not be answered at all, and a system may say "I guess"
//! with a confidence, which is judged apart and checked for honesty (the confidence it states against how often it is right).

use crate::school::School;
use std::collections::BTreeMap;

pub const README: &str = "THE ENTRY EXAM\n\n\
lessons.tsv     id TAB question TAB answer TAB unit TAB fact   what the system is taught. unit names which kind of lesson each belongs to, and fact numbers lessons that say the
                same thing in different words (the teacher's grouping); a system may ignore both\n\
questions.tsv   id TAB group TAB question TAB truth        what it is then asked. groups: taught (a lesson itself), reworded (a taught fact in other words),\n\
                never (an unseen question inside the range taught), beyond (numbers past the range taught), trap (must be refused; truth is empty)\n\n\
calibration.tsv  id TAB question TAB answer TAB unit      OPTIONAL extra material: other ways of saying taught facts, disjoint from every question. Use it however you like
                (the baby uses it to learn how different a question may look and still be recognised, which is also what keeps it from answering a trap).

Write predictions.tsv, one line per question:   id TAB status TAB answer TAB confidence\n\
  status   fact    stated as true.  This is the claim that is held to account: a wrong fact is the failure that matters.\n\
           guess   offered as a guess; confidence is the probability (0..1) the system itself puts on it being right.\n\
           refuse  no answer (answer and confidence empty).\n\n\
Judged by:  baby exam score DIR predictions.tsv      (or: baby exam run DIR to see the baby's own result, the bar a successor starts from)\n\
  1. WRONG facts, over every group, must be 0.     2. Traps answered (fact OR guess), must be 0.\n\
  3. Then: facts right per group (coverage); guesses right per group; and whether the confidence stated for guesses matches how often they were right.\n\
A system that never states a fact and refuses everything scores 0 wrong and 0 coverage: honest and useless. Coverage is what is then worth comparing.\n";

pub struct Exam {
    pub lessons: Vec<(String, String, String, String, usize)>, // id, question, answer, unit, fact
    pub calibration: Vec<(String, String, String, String, usize)>, // the same shape, optional
    pub questions: Vec<(String, String, String, String)>, // id, group, question, truth
}

/// Write the exam for a generated world of `units` kinds of lesson.
pub fn write(dir: &str, units: usize, seed: u64) -> std::io::Result<(usize, usize)> {
    let c = crate::scale::Config { units, entities: (units * 20).clamp(1000, 10_000), facts_per_unit: 12, noise: 0.0, typos: 0.0, digits: false, seed };
    let sw = crate::scale::generate(&c);
    let w = &sw.world;
    std::fs::create_dir_all(dir)?;
    let mut lessons = String::new();
    for (i, it) in w.train.iter().enumerate() {
        lessons.push_str(&format!("L{:05}\t{}\t{}\t{}\t{}\n", i, it.query, it.answer, sw.names.get(it.unit).map_or("?", |n| n.as_str()), it.fact));
    }
    std::fs::write(format!("{dir}/lessons.tsv"), lessons)?;
    let mut questions = String::new();
    let mut n = 0usize;
    // (the taught group asks a sample of the lessons themselves: every fifth)
    // half of the reworded set is handed out as calibration material, the other half is asked: nothing it is given is a question it is asked
    let mut calibration = String::new();
    for (i, it) in w.reworded.iter().enumerate().filter(|(i, _)| i % 2 == 0) {
        calibration.push_str(&format!("C{:05}\t{}\t{}\t{}\t{}\n", i, it.query, it.answer, sw.names.get(it.unit).map_or("?", |n| n.as_str()), it.fact));
    }
    std::fs::write(format!("{dir}/calibration.tsv"), calibration)?;
    for (group, items) in [("taught", w.train.iter().step_by(5).collect::<Vec<_>>()), ("reworded", w.reworded.iter().enumerate().filter(|(i, _)| i % 2 == 1).map(|(_, it)| it).collect()), ("never", w.new_items.iter().collect()), ("beyond", w.beyond.iter().collect())] {
        for it in items {
            questions.push_str(&format!("Q{:05}\t{}\t{}\t{}\n", n, group, it.query, it.answer));
            n += 1;
        }
    }
    for (q, _) in &sw.must_refuse {
        questions.push_str(&format!("Q{:05}\ttrap\t{}\t\n", n, q));
        n += 1;
    }
    std::fs::write(format!("{dir}/questions.tsv"), questions)?;
    std::fs::write(format!("{dir}/README.txt"), README)?;
    std::fs::write(format!("{dir}/meta.txt"), format!("units {units}\nseed {seed}\n"))?;
    Ok((w.train.len(), n))
}

fn read_tsv(path: &str, columns: usize) -> Result<Vec<Vec<String>>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let mut rows = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let mut parts: Vec<String> = line.split('\t').map(|s| s.to_string()).collect();
        if parts.len() > columns {
            return Err(format!("{path} line {}: expected {columns} columns, found {}", n + 1, parts.len()));
        }
        while parts.len() < columns {
            parts.push(String::new());
        }
        rows.push(parts);
    }
    Ok(rows)
}

pub fn load(dir: &str) -> Result<Exam, String> {
    let lessons = read_tsv(&format!("{dir}/lessons.tsv"), 5)?.into_iter().map(|r| (r[0].clone(), r[1].clone(), r[2].clone(), r[3].clone(), r[4].parse().unwrap_or(0))).collect();
    let questions = read_tsv(&format!("{dir}/questions.tsv"), 4)?.into_iter().map(|r| (r[0].clone(), r[1].clone(), r[2].clone(), r[3].clone())).collect();
    let calibration = if std::path::Path::new(&format!("{dir}/calibration.tsv")).exists() {
        read_tsv(&format!("{dir}/calibration.tsv"), 5)?.into_iter().map(|r| (r[0].clone(), r[1].clone(), r[2].clone(), r[3].clone(), r[4].parse().unwrap_or(0))).collect()
    } else {
        Vec::new()
    };
    Ok(Exam { lessons, calibration, questions })
}

#[derive(Default, Clone, Debug)]
pub struct Row {
    pub n: usize,
    pub fact_right: usize,
    pub fact_wrong: usize,
    pub guess_right: usize,
    pub guess_wrong: usize,
    pub refused: usize,
    pub missing: usize,
    pub stated: f64,
}

pub struct Verdict {
    pub groups: BTreeMap<String, Row>,
    pub wrong_facts: usize,
    pub traps_answered: usize,
    pub traps: usize,
    pub guesses: usize,
    pub stated: f64,
    pub realised: f64,
    /// ids of every wrong fact and every answered trap (so none hides in a percentage)
    pub offenders: Vec<String>,
}

/// Judge predictions (id, status, answer, confidence) against the exam's questions.
pub fn judge(exam: &Exam, predictions: &[Vec<String>]) -> Result<Verdict, String> {
    let mut by_id: BTreeMap<&str, &Vec<String>> = BTreeMap::new();
    for p in predictions {
        if p.is_empty() {
            continue;
        }
        by_id.insert(p[0].as_str(), p);
    }
    let mut v = Verdict { groups: BTreeMap::new(), wrong_facts: 0, traps_answered: 0, traps: 0, guesses: 0, stated: 0.0, realised: 0.0, offenders: Vec::new() };
    let mut right_guesses = 0usize;
    for (id, group, _, truth) in &exam.questions {
        let row = v.groups.entry(group.clone()).or_default();
        row.n += 1;
        let Some(p) = by_id.get(id.as_str()) else {
            row.missing += 1;
            row.refused += 1; // (a question not answered is a refusal)
            if group == "trap" {
                v.traps += 1;
            }
            continue;
        };
        let status = p.get(1).map_or("", |s| s.as_str());
        let answer = p.get(2).map_or("", |s| s.as_str());
        let conf: f64 = p.get(3).and_then(|c| c.parse().ok()).unwrap_or(0.5);
        if !matches!(status, "fact" | "guess" | "refuse") {
            return Err(format!("{id}: status must be fact, guess or refuse, not '{status}'"));
        }
        if group == "trap" {
            v.traps += 1;
            if status != "refuse" {
                v.traps_answered += 1;
                v.offenders.push(format!("{id} (trap answered as {status})"));
                row.fact_wrong += (status == "fact") as usize;
                row.guess_wrong += (status == "guess") as usize;
            } else {
                row.refused += 1;
            }
            continue;
        }
        match status {
            "fact" => {
                if answer == truth {
                    row.fact_right += 1;
                } else {
                    row.fact_wrong += 1;
                    v.wrong_facts += 1;
                    v.offenders.push(format!("{id} (said {answer}, truth {truth})"));
                }
            }
            "guess" => {
                v.guesses += 1;
                v.stated += conf;
                row.stated += conf;
                if answer == truth {
                    row.guess_right += 1;
                    right_guesses += 1;
                } else {
                    row.guess_wrong += 1;
                }
            }
            _ => row.refused += 1,
        }
    }
    v.realised = if v.guesses == 0 { 0.0 } else { right_guesses as f64 / v.guesses as f64 };
    v.stated = if v.guesses == 0 { 0.0 } else { v.stated / v.guesses as f64 };
    Ok(v)
}

pub fn print(v: &Verdict) {
    let pct = |a: usize, b: usize| if b == 0 { 0.0 } else { 100.0 * a as f64 / b as f64 };
    println!("  {:<10} {:>6} {:>10} {:>9} {:>11} {:>12} {:>9}", "group", "n", "fact right", "WRONG", "guess right", "guess wrong", "refused");
    for (g, r) in &v.groups {
        println!("  {:<10} {:>6} {:>9.1}% {:>8.1}% {:>10.1}% {:>11.1}% {:>8.1}%", g, r.n, pct(r.fact_right, r.n), pct(r.fact_wrong, r.n), pct(r.guess_right, r.n), pct(r.guess_wrong, r.n), pct(r.refused, r.n));
    }
    println!("\nWRONG facts (must be 0): {}", v.wrong_facts);
    println!("traps answered (must be 0): {} of {}", v.traps_answered, v.traps);
    if v.guesses > 0 {
        println!("guesses: {}; confidence stated on average {:.1}%, right {:.1}% of the time ({})", v.guesses, 100.0 * v.stated, 100.0 * v.realised, if v.stated <= v.realised + 0.02 { "honest: not over-confident" } else { "OVER-CONFIDENT" });
    }
    for o in v.offenders.iter().take(10) {
        println!("  offender: {o}");
    }
    if v.offenders.len() > 10 {
        println!("  ... and {} more", v.offenders.len() - 10);
    }
    println!("\nverdict: {}", if v.wrong_facts == 0 && v.traps_answered == 0 { "PASS on honesty (0 wrong facts, 0 traps answered); compare coverage" } else { "FAIL on honesty" });
}

/// Teach the baby the exam's lessons, ask its questions, and return predictions in the exam's format.
pub fn baby_predictions(exam: &Exam, seed: u64) -> Vec<Vec<String>> {
    let mut names: Vec<String> = Vec::new();
    for l in &exam.lessons {
        if !names.contains(&l.3) {
            names.push(l.3.clone());
        }
    }
    let items: Vec<crate::kinds::Item> = exam
        .lessons
        .iter()
        .map(|l| crate::kinds::Item { unit: names.iter().position(|n| *n == l.3).unwrap(), fact: l.4, template: 0, query: l.1.clone(), answer: l.2.clone() })
        .collect();
    let calibration: Vec<crate::kinds::Item> = exam
        .calibration
        .iter()
        .enumerate()
        .filter_map(|(i, l)| Some(crate::kinds::Item { unit: names.iter().position(|n| *n == l.3)?, fact: l.4, template: 0, query: l.1.clone(), answer: l.2.clone() }))
        .collect();
    let school = School::from_items(&items, names, if calibration.is_empty() { None } else { Some(&calibration) }, seed, true);
    exam.questions
        .iter()
        .map(|(id, _, q, _)| {
            let r = school.ask(q);
            match (r.answer, r.guess) {
                (Some(a), _) => vec![id.clone(), "fact".into(), a, String::new()],
                (None, Some(g)) => vec![id.clone(), "guess".into(), g.answer, format!("{:.3}", g.rate)],
                (None, None) => vec![id.clone(), "refuse".into(), String::new(), String::new()],
            }
        })
        .collect()
}

pub fn write_predictions(path: &str, preds: &[Vec<String>]) -> std::io::Result<()> {
    let mut out = String::new();
    for p in preds {
        out.push_str(&p.join("\t"));
        out.push('\n');
    }
    std::fs::write(path, out)
}

pub fn read_predictions(path: &str) -> Result<Vec<Vec<String>>, String> {
    read_tsv(path, 4)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny() -> Exam {
        Exam {
            lessons: vec![],
            calibration: vec![],
            questions: vec![
                ("Q1".into(), "never".into(), "q1".into(), "seven".into()),
                ("Q2".into(), "never".into(), "q2".into(), "four".into()),
                ("Q3".into(), "beyond".into(), "q3".into(), "nine".into()),
                ("Q4".into(), "trap".into(), "t".into(), String::new()),
                ("Q5".into(), "trap".into(), "t2".into(), String::new()),
            ],
        }
    }

    fn p(id: &str, status: &str, answer: &str, conf: &str) -> Vec<String> {
        vec![id.into(), status.into(), answer.into(), conf.into()]
    }

    #[test]
    fn a_wrong_fact_and_an_answered_trap_are_the_failures_and_everything_else_is_scored_apart() {
        let preds = vec![p("Q1", "fact", "seven", ""), p("Q2", "fact", "five", ""), p("Q3", "guess", "nine", "0.6"), p("Q4", "refuse", "", ""), p("Q5", "guess", "one", "0.9")];
        let v = judge(&tiny(), &preds).unwrap();
        assert_eq!(v.wrong_facts, 1, "Q2 was stated and wrong");
        assert_eq!(v.traps_answered, 1, "a guess on a trap counts as answering it");
        assert_eq!(v.groups["never"].fact_right, 1);
        assert_eq!(v.groups["beyond"].guess_right, 1);
        assert!(v.offenders.iter().any(|o| o.starts_with("Q2")) && v.offenders.iter().any(|o| o.starts_with("Q5")));
    }

    #[test]
    fn a_system_that_refuses_everything_is_honest_and_a_missing_prediction_is_a_refusal() {
        let v = judge(&tiny(), &[]).unwrap();
        assert_eq!((v.wrong_facts, v.traps_answered), (0, 0));
        assert_eq!(v.groups["never"].missing, 2);
        assert!(judge(&tiny(), &[p("Q1", "maybe", "x", "")]).is_err(), "a status that is not fact, guess or refuse is an error, not a silent pass");
    }

    #[test]
    fn the_exam_is_written_read_back_and_the_baby_passes_it_on_honesty() {
        let dir = std::env::temp_dir().join(format!("exam_{}", std::process::id()));
        let dir = dir.to_string_lossy().to_string();
        let (lessons, questions) = write(&dir, 8, 1).unwrap();
        let exam = load(&dir).unwrap();
        assert_eq!((exam.lessons.len(), exam.questions.len()), (lessons, questions));
        assert!(exam.questions.iter().any(|q| q.1 == "trap") && exam.questions.iter().any(|q| q.1 == "beyond"));
        let preds = baby_predictions(&exam, 1);
        let v = judge(&exam, &preds).unwrap();
        assert_eq!(v.wrong_facts, 0, "{:?}", v.offenders);
        assert_eq!(v.traps_answered, 0, "{:?}", v.offenders);
        assert!(v.groups["taught"].fact_right > 0, "it answers what it was taught");
        // round trip through the file a foreign system would write
        let path = format!("{dir}/p.tsv");
        write_predictions(&path, &preds).unwrap();
        let again = judge(&exam, &read_predictions(&path).unwrap()).unwrap();
        assert_eq!((again.wrong_facts, again.traps_answered), (0, 0));
        std::fs::remove_dir_all(&dir).ok();
    }
}
