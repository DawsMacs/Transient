// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! A baby that chooses what to learn.
//!
//! Adaptive Learning is not only learning from experience but steering it. Here the baby overhears a pile of questions (it never
//! hears the answers) and has a teacher who will tell it the answer to a question *if it asks*. Every answer costs the teacher an
//! effort, so the question is how few it needs. Three ways of deciding what to ask:
//!
//!   random   ask about questions in a random order, whether or not it could already answer them
//!   stream   questions come in a random order; it asks only when it turns out to be wrong (a teacher correcting mistakes)
//!   curious  it looks for the overheard question least like anything it has been told, and asks about that one; when every
//!            overheard question resembles something it knows, it stops asking. No teacher needed to judge its mistakes.
//!   trusting stream, but it *learns* how alike two questions must be to trust a match, from how often its matches turned out right
//!   checked  curious, plus doubt: when nothing seems new it spot-checks a few answers it believes it knows, and each time it
//!            finds one wrong it learns the right answer and goes back to being curious. It stops after a clean round.
//!
//! Learning is by writing into the fact store (no gradients), so the cost is just the answers requested.

use crate::data::{self, Dataset, Example};
use crate::ledger::Ledger;
use crate::rng::Rng;
use crate::store::Store;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pick {
    Random,
    Stream,
    Curious,
    Checked,
    Trusting,
}

impl Pick {
    pub const ALL: [Pick; 5] = [Pick::Random, Pick::Stream, Pick::Curious, Pick::Checked, Pick::Trusting];

    pub fn name(self) -> &'static str {
        match self {
            Pick::Random => "random",
            Pick::Stream => "stream",
            Pick::Curious => "curious",
            Pick::Checked => "checked",
            Pick::Trusting => "trusting",
        }
    }
}

/// How many things it spot-checks when nothing seems new.
const CHECKS: usize = 10;

/// Labels at which accuracy is recorded.
pub const MARKS: [usize; 8] = [20, 40, 60, 80, 100, 143, 200, 300];

pub struct Curve {
    /// (labels asked, accuracy on the overheard pool, accuracy on rewordings never overheard)
    pub points: Vec<(usize, f64, f64)>,
    pub labels: usize,
    pub stopped_early: bool,
    /// The match threshold it ended up trusting (the fixed one for the other methods).
    pub final_threshold: f32,
}

struct Lexicon {
    ids: HashMap<String, usize>,
    df: Vec<u32>,
    heard: u32,
    answers: HashMap<String, usize>,
}

impl Lexicon {
    fn new(pool: &[Example], tests: &[Example]) -> Lexicon {
        let mut lex = Lexicon { ids: HashMap::new(), df: vec![0], heard: 0, answers: HashMap::new() };
        for e in pool.iter().chain(tests) {
            for w in data::tokenize(&e.query) {
                let next = lex.ids.len() + 1;
                lex.ids.entry(w).or_insert(next);
            }
            let next = lex.answers.len();
            lex.answers.entry(e.answer.clone()).or_insert(next);
        }
        lex.df = vec![0; lex.ids.len() + 1];
        for e in pool {
            let mut seen: Vec<usize> = lex.key(&e.query);
            seen.dedup();
            for i in seen {
                lex.df[i] += 1;
            }
            lex.heard += 1;
        }
        lex
    }

    fn key(&self, q: &str) -> Vec<usize> {
        let mut k: Vec<usize> = data::tokenize(q).iter().filter_map(|w| self.ids.get(w).copied()).collect();
        k.sort_unstable();
        k.dedup();
        k
    }

    fn weight(&self, id: usize) -> f32 {
        (((1 + self.heard) as f32) / ((1 + self.df[id]) as f32)).ln().max(0.0) + 0.05
    }
}

/// How well it answers: (best answer's id, similarity), or none if the store is empty.
fn best(store: &Store, lex: &Lexicon, q: &str) -> Option<(usize, f32)> {
    let w = |id: usize| lex.weight(id);
    store.lookup(&lex.key(q), &w, &mut Ledger::default())
}

fn accuracy(store: &Store, lex: &Lexicon, set: &[Example], threshold: f32) -> f64 {
    let right = set
        .iter()
        .filter(|e| best(store, lex, &e.query).map_or(false, |(a, sim)| sim >= threshold && a == lex.answers[&e.answer]))
        .count();
    right as f64 / set.len().max(1) as f64
}

pub fn run(ds: &Dataset, pick: Pick, seed: u64, threshold: f32) -> Curve {
    let pool = &ds.train;
    let tests = &ds.paraphrase_test;
    let lex = Lexicon::new(pool, tests);
    let mut store = Store::new(0.0); // raw similarity is read out; the threshold is applied here
    let mut rng = Rng::new(seed.wrapping_add(4242));
    let mut order: Vec<usize> = (0..pool.len()).collect();
    rng.shuffle(&mut order);
    let mut remaining: Vec<usize> = order.clone();
    let mut points = Vec::new();
    let mut labels = 0usize;
    let mut stopped_early = false;

    let record = |store: &Store, labels: usize, points: &mut Vec<(usize, f64, f64)>, t: f32| {
        points.push((labels, accuracy(store, &lex, pool, t), accuracy(store, &lex, tests, t)));
    };
    let mut final_threshold = threshold;
    let teach = |store: &mut Store, i: usize| {
        let key = lex.key(&pool[i].query);
        store.insert(&key, &key, lex.answers[&pool[i].answer]);
    };

    match pick {
        Pick::Random => {
            for &i in &order {
                teach(&mut store, i);
                labels += 1;
                if MARKS.contains(&labels) {
                    record(&store, labels, &mut points, threshold);
                }
            }
        }
        Pick::Stream => {
            for &i in &order {
                let right = best(&store, &lex, &pool[i].query)
                    .map_or(false, |(a, sim)| sim >= threshold && a == lex.answers[&pool[i].answer]);
                if !right {
                    teach(&mut store, i);
                    labels += 1;
                    if MARKS.contains(&labels) {
                        record(&store, labels, &mut points, threshold);
                    }
                }
            }
        }
        Pick::Trusting => {
            // Ten bins of "how alike was the best match" -> how often that match was right. The teacher reveals the truth for
            // every question, so it can learn from matches it did NOT trust as well as from the ones it did.
            let mut bins = [(0u32, 0u32); 10];
            let mut t = 0.8f32;
            for &i in &order {
                let truth = lex.answers[&pool[i].answer];
                let hit = best(&store, &lex, &pool[i].query);
                if let Some((a, sim)) = hit {
                    let b = ((sim * 10.0) as usize).min(9);
                    bins[b].1 += 1;
                    bins[b].0 += (a == truth) as u32;
                }
                let right = hit.map_or(false, |(a, sim)| sim >= t && a == truth);
                if !right {
                    teach(&mut store, i);
                    labels += 1;
                    if MARKS.contains(&labels) {
                        record(&store, labels, &mut points, t);
                    }
                }
                t = trusted_threshold(&bins, 1.0, t);
            }
            final_threshold = t;
        }
        Pick::Curious | Pick::Checked => loop {
            // Ask about the overheard question least like anything it has been told. No answers are looked at to decide.
            let mut choice: Option<(usize, f32)> = None;
            for (pos, &i) in remaining.iter().enumerate() {
                let sim = best(&store, &lex, &pool[i].query).map_or(-1.0, |x| x.1);
                if sim < threshold && choice.map_or(true, |(_, s)| sim < s) {
                    choice = Some((pos, sim));
                }
            }
            match choice {
                None if pick == Pick::Checked => {
                    // Nothing seems new. Doubt: ask the teacher to confirm a handful of things it believes it knows.
                    let mut found_wrong = false;
                    for _ in 0..CHECKS.min(remaining.len()) {
                        let pos = rng.below(remaining.len());
                        let i = remaining[pos];
                        labels += 1;
                        let right = best(&store, &lex, &pool[i].query)
                            .map_or(false, |(a, sim)| sim >= threshold && a == lex.answers[&pool[i].answer]);
                        if !right {
                            remaining.swap_remove(pos);
                            teach(&mut store, i);
                            found_wrong = true;
                        }
                        if MARKS.contains(&labels) {
                            record(&store, labels, &mut points, threshold);
                        }
                    }
                    if !found_wrong {
                        stopped_early = true;
                        break;
                    }
                }
                None => {
                    stopped_early = true;
                    break;
                }
                Some((pos, _)) => {
                    let i = remaining.swap_remove(pos);
                    teach(&mut store, i);
                    labels += 1;
                    if MARKS.contains(&labels) {
                        record(&store, labels, &mut points, threshold);
                    }
                }
            }
        },
    }
    // Always record where it ended up.
    points.push((labels, accuracy(&store, &lex, pool, final_threshold), accuracy(&store, &lex, tests, final_threshold)));
    Curve { points, labels, stopped_early, final_threshold }
}

/// The least alike a match may be and still be worth answering, judged from what it has seen so far.
///
/// Answering wrongly costs the question the teacher then has to correct (1) plus the harm of having said something wrong
/// (`harm`); not answering costs the question (1). Answering pays when the chance of being right exceeds harm / (1 + harm).
/// So the target is not a fixed "90% sure": it follows from what a mistake costs. Each band of similarity is judged on its own
/// record, scanning from the most alike downward, and it stops at the first band whose record says "not worth it".
fn trusted_threshold(bins: &[(u32, u32); 10], harm: f32, current: f32) -> f32 {
    let seen: u32 = bins.iter().map(|b| b.1).sum();
    if seen < 20 {
        return current;
    }
    let target = harm / (1.0 + harm);
    let mut edge = 10;
    for b in (0..10).rev() {
        let (right, total) = bins[b];
        if total >= 3 && (right as f32 / total as f32) < target {
            break;
        }
        edge = b;
    }
    (edge as f32 / 10.0).min(0.95)
}

/// Accuracy (pool, rewordings) after at most `n` labels, taken from the last recorded point not beyond n.
pub fn at(curve: &Curve, n: usize) -> (f64, f64) {
    curve
        .points
        .iter()
        .filter(|p| p.0 <= n)
        .last()
        .map(|p| (p.1, p.2))
        .unwrap_or((0.0, 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_curious_baby_asks_fewer_questions_than_there_are_and_still_learns() {
        let ds = data::build(1);
        let c = run(&ds, Pick::Curious, 1, 0.6);
        assert!(c.stopped_early);
        assert!(c.labels < ds.train.len() * 3 / 4, "asked {} of {}", c.labels, ds.train.len());
        let (pool, para) = (c.points.last().unwrap().1, c.points.last().unwrap().2);
        assert!(pool > 0.8 && para > 0.75, "pool {pool} para {para}");
    }

    #[test]
    fn a_baby_that_learns_how_far_to_trust_a_match_beats_the_one_it_was_told_to_use() {
        for seed in 1..=5 {
            let ds = data::build(seed);
            let told = run(&ds, Pick::Stream, seed, 0.75);
            let learned = run(&ds, Pick::Trusting, seed, 0.75);
            assert!(learned.labels < told.labels, "seed {seed}: learned asked {} vs told {}", learned.labels, told.labels);
            assert!((0.3..=0.7).contains(&learned.final_threshold), "seed {seed}: trusts at {}", learned.final_threshold);
            let last = learned.points.last().unwrap();
            assert!(last.1 > 0.95 && last.2 > 0.9, "seed {seed}: ended at {:?}", last);
        }
    }

    #[test]
    fn asking_about_everything_in_random_order_reaches_full_marks() {
        let ds = data::build(1);
        let c = run(&ds, Pick::Random, 1, 0.75);
        assert_eq!(c.labels, ds.train.len());
        assert!(c.points.last().unwrap().1 > 0.97);
    }
}
