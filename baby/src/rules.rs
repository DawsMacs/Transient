// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DawsMacs
//! Rules, not tables, and a baby that decides for itself which way of learning to trust.
//!
//! Two ways of answering a question that contains numbers: a network that has memorised sums, and a quantity unit that has fitted
//! a rule (see `quantity.rs`). Neither is declared the winner in advance. The baby holds back a fifth of the sums it was taught,
//! learns from the rest with each mechanism, checks both on the sums it held back, and uses whichever generalises better for
//! questions of that kind. A tie goes to the cheaper one. The decision and the evidence are kept, so it can say why.

use crate::data::{self, Dataset, Example, Family, NUM};
use crate::know::fit;
use crate::model::Model;
use crate::net::{Reader, Scratch};
use crate::quantity::{counting_lessons, NumberLine, Quantity};
use crate::rng::Rng;
use std::collections::BTreeSet;

pub fn is_sum(e: &Example) -> bool {
    matches!(e.family, Family::Add | Family::Sub)
}

/// Which sum a question is about, regardless of phrasing: (adds?, first number, second number).
fn fact_of(e: &Example) -> (bool, usize, usize) {
    let nums: Vec<usize> = data::tokenize(&e.query).iter().filter_map(|w| NUM.iter().position(|n| n == w)).collect();
    (e.family == Family::Add, nums[0], nums[1])
}

pub fn net_right(model: &Model, set: &[Example]) -> (usize, usize) {
    let mut s = Scratch::new(&model.net.cfg);
    let sums: Vec<&Example> = set.iter().filter(|e| is_sum(e)).collect();
    let right = sums
        .iter()
        .filter(|e| {
            let p = model.net.probs(&model.encode(&e.query), &mut s);
            let best = p.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
            model.answer_id(&e.answer) == Some(best)
        })
        .count();
    (right, sums.len())
}

pub fn rule_right(q: &Quantity, set: &[Example]) -> (usize, usize) {
    let sums: Vec<&Example> = set.iter().filter(|e| is_sum(e)).collect();
    (sums.iter().filter(|e| q.answer(&e.query).map_or(false, |(a, _)| a == e.answer)).count(), sums.len())
}

pub fn fit_rule(train: &[Example]) -> Quantity {
    Quantity::fit(train, NumberLine::from_counting(&counting_lessons()), 800, 0.05)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mechanism {
    Network,
    Rule,
}

pub struct Decision {
    pub chosen: Mechanism,
    pub network_on_held_back: f64,
    pub rule_on_held_back: f64,
}

/// Hold back a fifth of the sums, learn with both mechanisms from the rest, test on what was held back.
pub fn decide(ds: &Dataset, reader: Reader, seed: u64) -> Decision {
    let facts: BTreeSet<(bool, usize, usize)> = ds.train.iter().filter(|e| is_sum(e)).map(fact_of).collect();
    let mut facts: Vec<_> = facts.into_iter().collect();
    Rng::new(seed.wrapping_add(55)).shuffle(&mut facts);
    let held: BTreeSet<_> = facts[..(facts.len() / 5).max(1)].iter().copied().collect();
    let (mut fit_set, mut held_back) = (Vec::new(), Vec::new());
    for e in &ds.train {
        if is_sum(e) && held.contains(&fact_of(e)) {
            held_back.push(e.clone());
        } else {
            fit_set.push(e.clone());
        }
    }
    let smaller = Dataset {
        train: fit_set.clone(),
        paraphrase_test: ds.paraphrase_test.clone(),
        item_test: ds.item_test.clone(),
        typo_test: Vec::new(),
        typo_entity_test: Vec::new(),
    };
    let net = fit(&smaller, reader, seed);
    let rule = fit_rule(&fit_set);
    let (nr, nn) = net_right(&net, &held_back);
    let (rr, rn) = rule_right(&rule, &held_back);
    let (net_acc, rule_acc) = (nr as f64 / nn.max(1) as f64, rr as f64 / rn.max(1) as f64);
    Decision {
        chosen: if rule_acc >= net_acc { Mechanism::Rule } else { Mechanism::Network },
        network_on_held_back: net_acc,
        rule_on_held_back: rule_acc,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_baby_picks_the_rule_for_sums_because_the_evidence_says_so() {
        let ds = data::build(1);
        let d = decide(&ds, Reader::Bag, 1);
        assert_eq!(d.chosen, Mechanism::Rule);
        assert!(d.rule_on_held_back > d.network_on_held_back + 0.3, "{} vs {}", d.rule_on_held_back, d.network_on_held_back);
    }

    #[test]
    fn a_fact_is_the_same_fact_in_every_phrasing() {
        let ds = data::build(1);
        let a = ds.train.iter().find(|e| e.family == Family::Add && e.template == 0).unwrap();
        let b = ds.train.iter().find(|e| e.family == Family::Add && e.template == 1 && fact_of(e) == fact_of(a)).unwrap();
        assert_eq!(fact_of(a), fact_of(b));
    }
}
