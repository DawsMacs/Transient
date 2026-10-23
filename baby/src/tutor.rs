// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! The baby asks, a person answers.
//!
//! Until now the questions the baby asks itself (`check_myself`: the edges and corners of what it was taught, where its explanation might be wrong)
//! were answered only by simulated teachers. `session` runs the same loop with a person: it shows a question, why it asks, and what it would guess,
//! takes the answer, and learns from it; if the answer contradicts a rule the baby has good evidence for it says so and asks the person to confirm
//! (the baby is skeptical of a source until it repeats itself, see `School::set_skeptical`). The person is a function, so a terminal and a test use
//! the same code. Free-form teaching goes through the same door: `question => answer` is placed by `School::learn_told`.

use crate::school::{Learned, School};
use std::io::Write;

/// What the person is shown for one question.
pub struct Prompt {
    pub number: usize,
    pub unit: String,
    pub question: String,
    pub why: String,
    /// What the baby would say now: a fact, a labelled guess, or nothing.
    pub would_say: String,
    /// Set when this is the second look at an answer that contradicted a rule: what the rule says and what the person said.
    pub confirming: Option<(String, String)>,
}

pub enum Said {
    Answer(String),
    Skip,
    Quit,
}

#[derive(Default, Debug)]
pub struct Summary {
    pub asked: usize,
    pub learned: usize,
    pub skipped: usize,
    /// answers held for a second opinion that were then confirmed / not confirmed
    pub confirmed: usize,
    pub unconfirmed: usize,
    pub told: usize,
    pub quit: bool,
}

/// The answer in the style the unit's lessons use: a number typed in digits becomes a number word when the unit speaks in words (and the word exists).
fn in_the_units_style(school: &School, unit: &str, said: &str) -> String {
    let said = said.trim().to_lowercase();
    let Ok(n) = said.parse::<usize>() else { return said };
    let words = school.units.iter().find(|u| u.name == unit).map_or(false, |u| u.speaks_in_words());
    match crate::data::NUM.get(n) {
        Some(w) if words => w.to_string(),
        _ => said,
    }
}

fn describe(school: &School, q: &str) -> String {
    let r = school.ask(q);
    match (r.answer, r.guess) {
        (Some(a), _) => format!("{a} (as fact)"),
        (None, Some(g)) => format!("probably {} (a guess, {:.0}% sure)", g.answer, 100.0 * g.rate),
        (None, None) => "nothing, I would refuse".into(),
    }
}

fn outcome(l: &Learned) -> String {
    if !l.surprised {
        "I already agreed with that; it extends where I may state things as fact.".to_string()
    } else if l.corrected {
        format!("corrected a lesson in '{}' (now by {}, {:.0}% on facts held back).", l.unit, l.now.name(), 100.0 * l.held_back)
    } else {
        format!("learned in '{}' ({} facts, now by {}, {:.0}% on facts held back).", l.unit, l.facts, l.now.name(), 100.0 * l.held_back)
    }
}

/// Up to `rounds` rounds of up to `limit` questions per unit; the person answers, skips, or quits. A free-form `question => answer` is taken in too.
pub fn session(school: &mut School, person: &mut dyn FnMut(&Prompt) -> Said, out: &mut dyn Write, rounds: usize, limit: usize) -> Summary {
    let mut sum = Summary::default();
    let mut already: Vec<String> = Vec::new();
    'rounds: for round in 1..=rounds {
        let questions: Vec<_> = school.check_myself(limit).into_iter().filter(|w| !already.contains(&w.question)).collect();
        if questions.is_empty() {
            let _ = writeln!(out, "{}", if round == 1 { "There is nothing I want to check: where I can explain a unit there is one explanation, and where I cannot there is nothing to choose between." } else { "That is all I wanted to check for now." });
            break;
        }
        let _ = writeln!(out, "\nRound {round}: {} things I would like to check.", questions.len());
        for w in questions {
            already.push(w.question.clone());
            sum.asked += 1;
            let prompt = Prompt { number: sum.asked, unit: w.unit.clone(), question: w.question.clone(), why: w.why.clone(), would_say: describe(school, &w.question), confirming: None };
            let said = person(&prompt);
            match said {
                Said::Quit => {
                    sum.quit = true;
                    break 'rounds;
                }
                Said::Skip => {
                    sum.skipped += 1;
                    let _ = writeln!(out, "  skipped.");
                }
                Said::Answer(text) => {
                    if let Some((q, a)) = text.split_once("=>") {
                        // taught something of the person's own choosing
                        match school.learn_told(q.trim(), a.trim()) {
                            Ok(l) => {
                                sum.told += 1;
                                let _ = writeln!(out, "  taken in: {}", outcome(&l));
                            }
                            Err(e) => {
                                let _ = writeln!(out, "  {e}");
                            }
                        }
                        continue;
                    }
                    let answer = in_the_units_style(school, &w.unit, &text);
                    match school.learn_wondered(&w.question, &answer, &w.unit) {
                        Ok(l) if l.disputed => {
                            let rule_says = describe(school, &w.question);
                            let _ = writeln!(out, "  That contradicts what I am confident about (I would have said {rule_says}). I will not believe it on one word.");
                            let again = person(&Prompt { confirming: Some((rule_says, answer.clone())), ..Prompt { number: prompt.number, unit: prompt.unit.clone(), question: prompt.question.clone(), why: prompt.why.clone(), would_say: prompt.would_say.clone(), confirming: None } });
                            match again {
                                Said::Answer(t2) => {
                                    let second = in_the_units_style(school, &w.unit, &t2);
                                    match school.learn_wondered(&w.question, &second, &w.unit) {
                                        Ok(l2) if !l2.disputed => {
                                            sum.confirmed += 1;
                                            sum.learned += 1;
                                            let _ = writeln!(out, "  Repeated, so I believe it: {}", outcome(&l2));
                                        }
                                        Ok(_) => {
                                            sum.unconfirmed += 1;
                                            let _ = writeln!(out, "  A different answer the second time: I am leaving it unsettled and keeping my rule.");
                                        }
                                        Err(e) => {
                                            let _ = writeln!(out, "  {e}");
                                        }
                                    }
                                }
                                Said::Skip => {
                                    sum.unconfirmed += 1;
                                    let _ = writeln!(out, "  Not repeated: keeping my rule; the answer stays held.");
                                }
                                Said::Quit => {
                                    sum.quit = true;
                                    break 'rounds;
                                }
                            }
                        }
                        Ok(l) => {
                            sum.learned += 1;
                            let _ = writeln!(out, "  {}", outcome(&l));
                        }
                        Err(e) => {
                            let _ = writeln!(out, "  {e}");
                        }
                    }
                }
            }
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capped() -> School {
        let mut items = Vec::new();
        for a in 0..=12i64 {
            for b in 0..=12i64 {
                items.push(crate::kinds::Item { unit: 0, fact: items.len(), template: 0, query: format!("what is {a} zork {b}"), answer: (a + b).min(30).to_string() });
            }
        }
        let mut s = School::from_items(&items, vec!["zork".into()], None, 1, true);
        s.set_careful(true);
        s
    }

    fn truth(q: &str) -> String {
        let n: Vec<i64> = q.split_whitespace().filter_map(|w| w.parse().ok()).collect();
        (n[0] + n[1]).min(30).to_string()
    }

    #[test]
    fn a_person_who_answers_what_it_asks_teaches_it_the_edge_it_did_not_know() {
        let mut s = capped();
        assert!(s.ask("what is 24 zork 12").answer.is_none(), "unchecked beyond what it was taught");
        let mut out = Vec::new();
        let mut person = |p: &Prompt| Said::Answer(truth(&p.question));
        let sum = session(&mut s, &mut person, &mut out, 3, 3);
        assert!(sum.asked >= 2 && sum.learned >= 2, "{sum:?}");
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("Round 1"), "{text}");
        // what it asked about is now known as fact, and the cap it found by being told is guessed correctly
        assert_eq!(s.ask("what is 24 zork 12").answer.as_deref(), Some("30"));
        // the corner it asked about covers this one too, so it is stated, not guessed
        assert_eq!(s.ask("what is 20 zork 15").answer.as_deref(), Some("30"));
    }

    #[test]
    fn an_answer_that_contradicts_a_settled_rule_is_held_until_the_person_repeats_it_and_a_skip_keeps_the_rule() {
        let mut s = capped();
        // the person is wrong once about the first question, then right when asked to confirm
        let mut calls = 0;
        let mut slip = |p: &Prompt| {
            calls += 1;
            match (calls, &p.confirming) {
                (1, None) => Said::Answer("99".into()),
                (_, Some(_)) => Said::Answer(truth(&p.question)),
                _ => Said::Answer(truth(&p.question)),
            }
        };
        let mut out = Vec::new();
        let sum = session(&mut s, &mut slip, &mut out, 1, 1);
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("will not believe it on one word"), "{text}");
        // the slip ("99") was followed by a different answer, so it is unsettled and never believed; the next question genuinely contradicts the uncapped rule
        // (the truth is capped at 30), is repeated, and is believed
        assert_eq!((sum.unconfirmed, sum.confirmed), (1, 1), "{sum:?}\n{text}");
        // never a wrong fact from the slip
        assert!(s.ask("what is 24 zork 12").answer.as_deref() != Some("99"));
        // the person skips the confirmation: rule kept
        let mut s2 = capped();
        let mut calls2 = 0;
        let mut person2 = |p: &Prompt| {
            calls2 += 1;
            if calls2 == 1 { Said::Answer("99".into()) } else if p.confirming.is_some() { Said::Skip } else { Said::Quit }
        };
        let mut out2 = Vec::new();
        let sum2 = session(&mut s2, &mut person2, &mut out2, 1, 1);
        assert_eq!(sum2.unconfirmed, 1, "{sum2:?}");
        assert!(s2.ask("what is 24 zork 12").answer.as_deref() != Some("99"));
    }

    #[test]
    fn a_free_form_question_and_answer_is_placed_and_a_digit_answer_is_given_in_the_units_style() {
        let mut s = crate::school::School::train(&crate::kinds::world(1), 1, true);
        let mut out = Vec::new();
        let mut person = |_: &Prompt| Said::Answer("what is two times three and also five => six".into());
        // (the tidy world's own check questions come first; one free-form answer is enough to see the route)
        let sum = session(&mut s, &mut person, &mut out, 1, 1);
        let text = String::from_utf8(out).unwrap();
        assert!(sum.told >= 1, "{sum:?}\n{text}");
        assert_eq!(in_the_units_style(&s, "times", "6"), "six");
        assert_eq!(in_the_units_style(&s, "times", "twelve"), "twelve");
    }
}
