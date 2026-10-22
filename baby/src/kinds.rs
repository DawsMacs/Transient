// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DawsMacs
//! Five kinds of lesson, grouped into units the way a school would ("today: comparing numbers").
//!
//! The teacher says which lessons belong together. The teacher does NOT say how to learn them: that is the baby's decision
//! (see `mechanisms.rs`). The units are chosen so that no single way of learning suits all of them:
//!
//!   capital   facts: remember them (nothing can guess an unseen capital)
//!   sum       arithmetic: a rule with two quantities
//!   compare   "which is bigger three or five": pick one of the two (not a sum of quantities, so the sums rule cannot do it)
//!   after     "what comes after four": a rule with one quantity and a constant
//!   double    "what is double three": a rule with one quantity and a multiplier
//!   half      "what is half of six": added last, as data only, to see whether a new kind of lesson finds its own way
//!   times     "what is two times three": a kind NONE of the mechanisms can express (they add quantities; this multiplies them),
//!             there to check that the baby notices it has not learned it instead of bluffing
//!   apart     "how far apart are two and seven": the difference whichever way round, which no linear rule can express
//!   color, sound, opposite   plain facts, like capital
//!   minus     "what is five minus two": a rule, like sum
//!   code      "what is the code for four": an arbitrary mapping with no formula behind it, there to check that a search for
//!             rules does not 'explain' what has no explanation
//!
//! Each fact is asked in four phrasings. Three are used for teaching and the fourth tests rewordings. For the rule units a fifth
//! of the facts are never taught ("new items"), and some questions use numbers above anything taught ("beyond").

use crate::data::{self, NUM};
use crate::rng::Rng;

pub const UNITS: [&str; 13] =
    ["capital", "sum", "compare", "after", "double", "half", "times", "apart", "code", "color", "sound", "opposite", "minus"];

/// Units made of facts that cannot be worked out: nothing is held back for them and nothing lies beyond them.
fn is_fact_unit(unit: usize) -> bool {
    matches!(unit, 0 | 9 | 10 | 11)
}

#[derive(Clone, Debug)]
pub struct Item {
    pub unit: usize,
    pub fact: usize,
    pub template: usize,
    pub query: String,
    pub answer: String,
}

pub struct World {
    pub train: Vec<Item>,
    pub reworded: Vec<Item>,
    pub new_items: Vec<Item>,
    pub beyond: Vec<Item>,
}

struct Fact {
    slots: Vec<String>,
    answer: String,
    templates: [&'static str; 4],
}

fn fill(template: &str, slots: &[String]) -> String {
    let mut s = template.to_string();
    for (i, slot) in slots.iter().enumerate() {
        s = s.replace(&format!("{{{i}}}"), slot);
    }
    s
}

const CAPITAL_T: [&str; 4] = ["capital of {0}", "what is the capital of {0}", "{0} capital", "the capital of {0}"];
const SUM_T: [&str; 4] = ["what is {0} plus {1}", "{0} plus {1}", "{0} plus {1} equals", "what is {0} plus {1} equals"];
const BIGGER_T: [&str; 4] = ["which is bigger {0} or {1}", "bigger {0} or {1}", "{0} or {1} which is bigger", "which bigger {0} or {1}"];
const SMALLER_T: [&str; 4] = ["which is smaller {0} or {1}", "smaller {0} or {1}", "{0} or {1} which is smaller", "which smaller {0} or {1}"];
const AFTER_T: [&str; 4] = ["what comes after {0}", "after {0}", "{0} then what", "what is after {0}"];
const DOUBLE_T: [&str; 4] = ["what is double {0}", "double {0}", "{0} doubled", "what is {0} doubled"];
const TIMES_T: [&str; 4] = ["what is {0} times {1}", "{0} times {1}", "{0} times {1} equals", "what is {0} times {1} equals"];
const APART_T: [&str; 4] = ["how far apart are {0} and {1}", "{0} and {1} apart", "apart {0} and {1}", "how far are {0} and {1} apart"];
const CODE_T: [&str; 4] = ["what is the code for {0}", "code for {0}", "{0} code", "the code for {0}"];
pub const CODE: [usize; 11] = [7, 3, 9, 0, 5, 10, 2, 8, 1, 6, 4];
const COLOR_T: [&str; 4] = ["what color is the {0}", "color of {0}", "the {0} is what color", "what is the color of {0}"];
const SOUND_T: [&str; 4] = ["what does the {0} say", "{0} sound", "what sound does the {0} make", "what does the {0} make"];
const OPPOSITE_T: [&str; 4] = ["opposite of {0}", "what is the opposite of {0}", "{0} opposite", "the opposite of {0}"];
const MINUS_T: [&str; 4] = ["what is {0} minus {1}", "{0} minus {1}", "{0} minus {1} equals", "what is {0} minus {1} equals"];
const HALF_T: [&str; 4] = ["what is half of {0}", "half of {0}", "{0} halved", "what is {0} halved"];

/// (facts inside the taught range, facts beyond it) for each unit.
fn facts() -> Vec<(Vec<Fact>, Vec<Fact>)> {
    let n = |i: usize| NUM[i].to_string();
    let mut out = Vec::new();

    out.push((
        data::CAPITALS
            .iter()
            .map(|(c, a)| Fact { slots: vec![c.to_string()], answer: a.to_string(), templates: CAPITAL_T })
            .collect(),
        Vec::new(),
    ));

    let (mut sums, mut sums_far) = (Vec::new(), Vec::new());
    for a in 0..=10usize {
        for b in 0..=10usize {
            if a + b <= 10 {
                let f = Fact { slots: vec![n(a), n(b)], answer: n(a + b), templates: SUM_T };
                if a <= 5 && b <= 5 { sums.push(f) } else { sums_far.push(f) }
            }
        }
    }
    out.push((sums, sums_far));

    let (mut cmp, mut cmp_far) = (Vec::new(), Vec::new());
    for a in 0..=10usize {
        for b in 0..=10usize {
            if a == b {
                continue;
            }
            for (templates, bigger) in [(BIGGER_T, true), (SMALLER_T, false)] {
                let winner = if (a > b) == bigger { a } else { b };
                let f = Fact { slots: vec![n(a), n(b)], answer: n(winner), templates };
                if a <= 5 && b <= 5 { cmp.push(f) } else { cmp_far.push(f) }
            }
        }
    }
    out.push((cmp, cmp_far));

    let (mut after, mut after_far) = (Vec::new(), Vec::new());
    for a in 0..=9usize {
        let f = Fact { slots: vec![n(a)], answer: n(a + 1), templates: AFTER_T };
        if a <= 6 { after.push(f) } else { after_far.push(f) }
    }
    out.push((after, after_far));

    let doubles: Vec<Fact> = (0..=5usize).map(|a| Fact { slots: vec![n(a)], answer: n(2 * a), templates: DOUBLE_T }).collect();
    out.push((doubles, Vec::new()));

    let halves: Vec<Fact> = (0..=5usize).map(|a| Fact { slots: vec![n(2 * a)], answer: n(a), templates: HALF_T }).collect();
    out.push((halves, Vec::new()));

    let (mut prod, mut prod_far) = (Vec::new(), Vec::new());
    for a in 0..=10usize {
        for b in 0..=10usize {
            if a * b <= 10 {
                let f = Fact { slots: vec![n(a), n(b)], answer: n(a * b), templates: TIMES_T };
                if a <= 5 && b <= 5 { prod.push(f) } else { prod_far.push(f) }
            }
        }
    }
    out.push((prod, prod_far));

    let (mut apart, mut apart_far) = (Vec::new(), Vec::new());
    for a in 0..=10usize {
        for b in 0..=10usize {
            let f = Fact { slots: vec![n(a), n(b)], answer: n(a.abs_diff(b)), templates: APART_T };
            if a <= 5 && b <= 5 { apart.push(f) } else { apart_far.push(f) }
        }
    }
    out.push((apart, apart_far));

    let code: Vec<Fact> = (0..=10usize).map(|a| Fact { slots: vec![n(a)], answer: n(CODE[a]), templates: CODE_T }).collect();
    out.push((code, Vec::new()));

    let pairs = |list: &[(&str, &str)], templates: [&'static str; 4]| -> Vec<Fact> {
        list.iter().map(|(q, a)| Fact { slots: vec![q.to_string()], answer: a.to_string(), templates }).collect()
    };
    out.push((pairs(data::COLORS, COLOR_T), Vec::new()));
    out.push((pairs(data::SOUNDS, SOUND_T), Vec::new()));
    let mut opp = pairs(data::OPPOSITES, OPPOSITE_T);
    opp.extend(data::OPPOSITES.iter().map(|(a, b)| Fact { slots: vec![b.to_string()], answer: a.to_string(), templates: OPPOSITE_T }));
    out.push((opp, Vec::new()));

    let (mut minus, mut minus_far) = (Vec::new(), Vec::new());
    for a in 0..=10usize {
        for b in 0..=a {
            let f = Fact { slots: vec![n(a), n(b)], answer: n(a - b), templates: MINUS_T };
            if a <= 5 { minus.push(f) } else { minus_far.push(f) }
        }
    }
    out.push((minus, minus_far));
    out
}

pub fn world(seed: u64) -> World {
    let mut rng = Rng::new(seed.wrapping_add(777));
    let mut w = World { train: Vec::new(), reworded: Vec::new(), new_items: Vec::new(), beyond: Vec::new() };
    let mut fact_id = 0;
    for (unit, (mut near, far)) in facts().into_iter().enumerate() {
        rng.shuffle(&mut near);
        let held = if is_fact_unit(unit) { 0 } else { ((near.len() as f32) * 0.2).round().max(1.0) as usize };
        for (i, f) in near.iter().enumerate() {
            for (t, template) in f.templates.iter().enumerate() {
                let item = Item { unit, fact: fact_id, template: t, query: fill(template, &f.slots), answer: f.answer.clone() };
                if i < held {
                    w.new_items.push(item);
                } else if t < 3 {
                    w.train.push(item);
                } else {
                    w.reworded.push(item);
                }
            }
            fact_id += 1;
        }
        for f in &far {
            for (t, template) in f.templates.iter().enumerate() {
                w.beyond.push(Item { unit, fact: fact_id, template: t, query: fill(template, &f.slots), answer: f.answer.clone() });
            }
            fact_id += 1;
        }
    }
    w
}

/// `n` capitals nobody was taught, invented (made-up country and city names that are not in the curriculum): three phrasings to teach and the
/// fourth, never taught, to ask. For measuring what it costs to learn something NEW and what happens to the old.
pub fn new_capital_facts(seed: u64, n: usize) -> (Vec<Item>, Vec<Item>) {
    let mut rng = Rng::new(seed.wrapping_add(4242));
    let syl = ["ka", "lo", "mi", "ra", "tu", "ve", "no", "sa", "di", "pe", "zu", "fa", "bo", "gi", "he", "ju"];
    let known: std::collections::HashSet<String> = data::CAPITALS.iter().flat_map(|(c, a)| [c.to_string(), a.to_string()]).collect();
    let mut made: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut word = |rng: &mut Rng, made: &mut std::collections::HashSet<String>| loop {
        let w: String = (0..3).map(|_| syl[rng.below(syl.len())]).collect();
        if !known.contains(&w) && made.insert(w.clone()) {
            return w;
        }
    };
    let (mut teach, mut ask) = (Vec::new(), Vec::new());
    for i in 0..n {
        let (country, city) = (word(&mut rng, &mut made), word(&mut rng, &mut made));
        for (t, template) in CAPITAL_T.iter().enumerate() {
            let item = Item { unit: 0, fact: 100_000 + i, template: t, query: fill(template, &[country.clone()]), answer: city.clone() };
            if t < 3 { teach.push(item) } else { ask.push(item) }
        }
    }
    (teach, ask)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn the_answers_are_right_and_the_splits_do_not_leak() {
        let w = world(1);
        let train: HashSet<&str> = w.train.iter().map(|i| i.query.as_str()).collect();
        for i in w.reworded.iter().chain(&w.new_items).chain(&w.beyond) {
            assert!(!train.contains(i.query.as_str()), "leaked: {}", i.query);
        }
        let val = |s: &str| NUM.iter().position(|n| *n == s);
        for i in w.train.iter().chain(&w.reworded).chain(&w.new_items).chain(&w.beyond) {
            let nums: Vec<usize> = data::tokenize(&i.query).iter().filter_map(|t| val(t)).collect();
            let ans = val(&i.answer);
            match i.unit {
                1 => assert_eq!(ans, Some(nums[0] + nums[1]), "{}", i.query),
                2 => {
                    let big = i.query.contains("bigger");
                    let want = if big { nums[0].max(nums[1]) } else { nums[0].min(nums[1]) };
                    assert_eq!(ans, Some(want), "{}", i.query)
                }
                3 => assert_eq!(ans, Some(nums[0] + 1), "{}", i.query),
                4 => assert_eq!(ans, Some(2 * nums[0]), "{}", i.query),
                5 => assert_eq!(ans.map(|a| a * 2), Some(nums[0]), "{}", i.query),
                6 => assert_eq!(ans, Some(nums[0] * nums[1]), "{}", i.query),
                7 => assert_eq!(ans, Some(nums[0].abs_diff(nums[1])), "{}", i.query),
                8 => assert_eq!(ans, Some(CODE[nums[0]]), "{}", i.query),
                12 => assert_eq!(ans, Some(nums[0] - nums[1]), "{}", i.query),
                _ => {}
            }
        }
        assert!(w.beyond.iter().any(|i| i.unit == 2) && w.new_items.iter().all(|i| !is_fact_unit(i.unit)));
    }

    #[test]
    fn invented_capitals_are_new_and_split_into_teaching_and_asking_phrasings() {
        let (teach, ask) = new_capital_facts(1, 12);
        assert_eq!((teach.len(), ask.len()), (36, 12));
        let known: HashSet<String> = data::CAPITALS.iter().flat_map(|(c, a)| [c.to_string(), a.to_string()]).collect();
        let taught_queries: HashSet<&str> = teach.iter().map(|i| i.query.as_str()).collect();
        for i in teach.iter().chain(&ask) {
            assert!(data::tokenize(&i.query).iter().all(|w| !known.contains(w) || ["capital", "of", "the", "what", "is"].contains(&w.as_str())), "{}", i.query);
            assert!(!known.contains(&i.answer));
        }
        assert!(ask.iter().all(|i| !taught_queries.contains(i.query.as_str())));
    }

    #[test]
    fn every_unit_has_enough_facts_to_test_on() {
        let w = world(1);
        for u in 0..UNITS.len() {
            let facts: HashSet<usize> = w.train.iter().filter(|i| i.unit == u).map(|i| i.fact).collect();
            assert!(facts.len() >= 5, "unit {u} has only {} facts", facts.len());
        }
    }
}
