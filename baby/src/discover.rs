// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! Finding the units without being told.
//!
//! Given a heap of (question, answer) lessons with no grouping, work out which belong together:
//!
//! 1. **Anchor.** Each lesson's anchor is its most telling word: the rarest word (among those frequent enough to be a pattern, not
//!    a one-off thing being asked about) that is not a number. "what is two times three" anchors on "times".
//! 2. **Same facts, other words.** Two anchor groups of *things* that hold the same facts (the same thing asked about, the same
//!    answer) are phrasings of one unit ("what does the cat say" and "cat sound"). Not for numbers: see 3.
//! 3. **One rule.** Two groups of number questions merge only if a way of learning that keeps a separate setting for each
//!    operation word (a rule, or a selection) explains both on facts held back, or if each group, alone, built the very same
//!    expression. "plus" and "minus" merge (a rule has a weight for each word); "times" and "plus" do not (no rule does both).
//!    A single built expression is NOT enough on its own: it ignores which word was used, so one expression that happens to fit
//!    "minus" and "apart" lessons would also make "minus" quietly behave like "apart" where it was never shown (and answer
//!    "three minus five" with "two"). That merge was made once, caught by asking that question, and is refused now.
//!
//! Along the way it also works out which lessons are the same *fact*, which makes later cross-validation honest: asking about "two
//! plus three" and "three plus two" is one fact, so one cannot be used to test the other.

use crate::data::tokenize;
use crate::kinds::Item;
use crate::mechanisms::{cv_accuracy, cv_correct, shape, Weights};
use crate::program::ProgramRule;
use crate::quantity::{counting_lessons, NumberLine};
use std::collections::{BTreeMap, HashMap, HashSet};

pub struct Discovered {
    /// The lessons with the unit and fact they were assigned, in unit order.
    pub items: Vec<Item>,
    /// For each item, the index of the lesson it came from.
    pub origin: Vec<usize>,
    pub names: Vec<String>,
}

struct Ctx<'a> {
    lessons: &'a [(String, String)],
    line: NumberLine,
    raws: Vec<Vec<String>>,
    shapes: Vec<Vec<String>>,
    raw_df: HashMap<String, usize>,
    min_df: usize,
}

impl<'a> Ctx<'a> {
    /// A word that identifies *which* fact (a number, or a rare word such as a place) rather than what kind of question.
    fn is_content(&self, w: &str) -> bool {
        self.line.value(w).is_some() || self.raw_df.get(w).copied().unwrap_or(0) < self.min_df
    }

    fn fact_key(&self, i: usize) -> String {
        let mut k: Vec<&str> = self.raws[i].iter().map(String::as_str).filter(|w| self.is_content(w)).collect();
        k.sort();
        format!("{}=>{}", k.join(" "), self.lessons[i].1)
    }

    /// As `fact_key`, but keeping the order of the numbers: "five minus two" and "two minus five" are different facts here. Used to
    /// decide whether two groups hold the same facts (are phrasings of one unit); the sorted key above is for testing honestly.
    fn ordered_key(&self, i: usize) -> String {
        let k: Vec<&str> = self.raws[i].iter().map(String::as_str).filter(|w| self.is_content(w)).collect();
        format!("{}=>{}", k.join(" "), self.lessons[i].1)
    }

    fn items(&self, idxs: &[usize], unit: usize, facts: &mut HashMap<String, usize>) -> Vec<Item> {
        idxs.iter()
            .map(|&i| {
                let next = facts.len();
                let fact = *facts.entry(format!("{unit}:{}", self.fact_key(i))).or_insert(next);
                Item { unit, fact, template: 0, query: self.lessons[i].0.clone(), answer: self.lessons[i].1.clone() }
            })
            .collect()
    }

    fn numeric(&self, idxs: &[usize]) -> bool {
        let with = idxs.iter().filter(|&&i| self.raws[i].iter().any(|w| self.line.value(w).is_some())).count();
        with * 10 >= idxs.len() * 8
    }
}

/// How well the best of the given mechanisms (1 rule, 2 selection, 3 built expression) does on facts held back from these lessons.
fn generalises(ctx: &Ctx, groups: &[Vec<usize>], which: &[usize], all: &Weights, seed: u64, mechs: &[usize]) -> f64 {
    let idxs: Vec<usize> = which.iter().flat_map(|g| groups[*g].iter().copied()).collect();
    let mut facts = HashMap::new();
    let items = ctx.items(&idxs, 0, &mut facts);
    let n_facts = items.iter().map(|i| i.fact).collect::<HashSet<_>>().len();
    if n_facts < 5 {
        return 0.0;
    }
    cv_accuracy(&items, all, seed, mechs).into_iter().fold(0.0, f64::max)
}

/// The best, over the mechanisms, of the *worst* group's held-back accuracy when these groups are learned together.
fn each_group(ctx: &Ctx, groups: &[Vec<usize>], which: &[usize], all: &Weights, seed: u64, mechs: &[usize]) -> f64 {
    let mut facts = HashMap::new();
    let mut items = Vec::new();
    let mut owner = Vec::new();
    for (k, g) in which.iter().enumerate() {
        let part = ctx.items(&groups[*g], 0, &mut facts);
        owner.extend(std::iter::repeat(k).take(part.len()));
        items.extend(part);
    }
    if items.iter().map(|i| i.fact).collect::<HashSet<_>>().len() < 5 {
        return 0.0;
    }
    cv_correct(&items, all, seed, mechs)
        .iter()
        .map(|c| {
            (0..which.len())
                .map(|k| {
                    let mine: Vec<usize> = (0..items.len()).filter(|&i| owner[i] == k).collect();
                    mine.iter().filter(|&&i| c[i]).count() as f64 / mine.len().max(1) as f64
                })
                .fold(1.0, f64::min)
        })
        .fold(0.0, f64::max)
}

/// Would one rule or selection (a mechanism with a setting per operation word) be right for EACH of these two sets of lessons, on
/// facts held back? Used to decide whether two units that were taught separately are really one.
#[cfg(test)]
pub fn can_merge(a: &[Item], b: &[Item], all: &Weights, seed: u64) -> bool {
    can_merge_many(&[a, b], all, seed).is_some()
}

/// The same question for any number of parts at once: is there ONE rule (or one selection) that is right, on facts held back, for
/// every part? Each part is judged on its own share of the folds (see `cv_correct_strata`), so a small part cannot hide behind a big
/// one, and one test over many units costs about what one test over two does, instead of a test per pair. Returns the mechanism that
/// fits them all (an index into `Mech::ALL`: the rule before the selection) and its accuracy on held-back facts over every part, so
/// the merged unit can be settled from this evidence without working it all out again.
pub fn can_merge_many(parts: &[&[Item]], all: &Weights, seed: u64) -> Option<(usize, f64)> {
    let mut items = Vec::new();
    let mut owner = Vec::new();
    let mut base = 0usize;
    for (k, part) in parts.iter().enumerate() {
        let mut map: HashMap<usize, usize> = HashMap::new();
        for it in part.iter() {
            let next = base + map.len();
            let fact = *map.entry(it.fact).or_insert(next);
            items.push(Item { fact, ..it.clone() });
            owner.push(k);
        }
        base += map.len();
    }
    if items.iter().map(|i| i.fact).collect::<HashSet<_>>().len() < 5 {
        return None;
    }
    let mechs = [1usize, 2];
    let correct = crate::mechanisms::cv_correct_strata(&items, all, seed, &mechs, &owner);
    correct.iter().enumerate().find_map(|(m, c)| {
        let every_part_fits = (0..parts.len()).all(|k| {
            let mine: Vec<usize> = (0..items.len()).filter(|&i| owner[i] == k).collect();
            mine.iter().filter(|&&i| c[i]).count() as f64 / mine.len().max(1) as f64 >= crate::mechanisms::reliable()
        });
        every_part_fits.then(|| (mechs[m], c.iter().filter(|x| **x).count() as f64 / items.len().max(1) as f64))
    })
}

/// The expression this group alone builds, if it builds one.
fn own_program(ctx: &Ctx, groups: &[Vec<usize>], g: usize) -> Option<String> {
    let mut facts = HashMap::new();
    let items = ctx.items(&groups[g], 0, &mut facts);
    ProgramRule::fit_unique(&items).map(|p| p.expr.to_text())
}

fn describe(g: &[usize], anchors: &[String]) -> String {
    let mut c: BTreeMap<&str, usize> = BTreeMap::new();
    for &i in g {
        *c.entry(anchors[i].as_str()).or_default() += 1;
    }
    format!("[{}]", c.iter().map(|(k, v)| format!("{k}:{v}")).collect::<Vec<_>>().join(" "))
}

pub fn discover(lessons: &[(String, String)], seed: u64) -> Discovered {
    let n = lessons.len();
    let line = NumberLine::from_counting(&counting_lessons());
    let raws: Vec<Vec<String>> = lessons.iter().map(|(q, _)| tokenize(q)).collect();
    let shapes: Vec<Vec<String>> = lessons.iter().map(|(q, _)| shape(q, &line)).collect();
    let mut raw_df: HashMap<String, usize> = HashMap::new();
    for r in &raws {
        let mut seen = r.clone();
        seen.sort();
        seen.dedup();
        for w in seen {
            *raw_df.entry(w).or_default() += 1;
        }
    }
    let min_df = 5.max(n / 200);
    let ctx = Ctx { lessons, line, raws, shapes, raw_df, min_df };

    // 1. anchors
    let mut shape_df: HashMap<&str, usize> = HashMap::new();
    for s in &ctx.shapes {
        for t in s {
            if !t.starts_with("<num>") {
                *shape_df.entry(t.as_str()).or_default() += 1;
            }
        }
    }
    // A word that is a pattern of the question, not a thing being asked about, shows up across many different facts.
    let mut fact_spread: HashMap<&str, HashSet<String>> = HashMap::new();
    for (i, s) in ctx.shapes.iter().enumerate() {
        let key = ctx.fact_key(i);
        for t in s {
            if !t.starts_with("<num>") {
                fact_spread.entry(t.as_str()).or_default().insert(key.clone());
            }
        }
    }
    let anchors: Vec<String> = ctx
        .shapes
        .iter()
        .map(|s| {
            s.iter()
                .filter(|t| !t.starts_with("<num>") && shape_df[t.as_str()] >= min_df && fact_spread[t.as_str()].len() >= 4)
                .min_by_key(|t| (shape_df[t.as_str()], t.to_string()))
                .cloned()
                .unwrap_or_else(|| "misc".to_string())
        })
        .collect();
    let mut by_anchor: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, a) in anchors.iter().enumerate() {
        by_anchor.entry(a.clone()).or_default().push(i);
    }
    let mut groups: Vec<Vec<usize>> = by_anchor.values().cloned().collect();
    if std::env::var("BABY_DEBUG").is_ok() {
        for g in &groups {
            eprintln!("anchor group {} numeric={}", describe(g, &anchors), ctx.numeric(g));
        }
    }

    // 2. merge groups that hold the same facts, both ways round (phrasings of one unit hold the same facts; a group that merely
    //    includes another's facts, like "how far apart" including "minus", is a different unit that happens to agree on them)
    loop {
        let pairs: Vec<HashSet<String>> = groups.iter().map(|g| g.iter().map(|&i| ctx.ordered_key(i)).collect()).collect();
        let mut best: Option<(usize, usize, f64)> = None;
        for a in 0..groups.len() {
            for b in a + 1..groups.len() {
                let inter = pairs[a].intersection(&pairs[b]).count();
                let union = pairs[a].union(&pairs[b]).count();
                if inter >= 3 && inter * 5 >= union * 4 {
                    let score = inter as f64 / union as f64;
                    if best.map_or(true, |x| score > x.2) {
                        best = Some((a, b, score));
                    }
                }
            }
        }
        match best {
            Some((a, b, s)) => {
                if std::env::var("BABY_DEBUG").is_ok() {
                    eprintln!("step 2 merge {} + {} (same facts {:.2})", describe(&groups[a], &anchors), describe(&groups[b], &anchors), s);
                }
                let moved = groups.remove(b);
                groups[a].extend(moved);
            }
            None => break,
        }
    }

    // 3. merge number groups that one rule explains
    loop {
        let provisional: Vec<Item> = {
            let mut facts = HashMap::new();
            groups.iter().enumerate().flat_map(|(u, g)| ctx.items(g, u, &mut facts)).collect()
        };
        let all = Weights::from(&provisional);
        let numeric: Vec<usize> = (0..groups.len()).filter(|&g| ctx.numeric(&groups[g])).collect();
        let alone: HashMap<usize, f64> = numeric.iter().map(|&g| (g, generalises(&ctx, &groups, &[g], &all, seed, &[1, 2, 3]))).collect();
        let mut best: Option<(usize, usize, f64)> = None;
        for (x, &a) in numeric.iter().enumerate() {
            for &b in &numeric[x + 1..] {
                if alone[&a] < 0.9 || alone[&b] < 0.9 {
                    continue;
                }
                // Through a mechanism with a setting per operation word that is right for EACH group on facts held back (an overall
                // average would let a big group swallow a small one), or through the same expression built twice.
                let mut both = each_group(&ctx, &groups, &[a, b], &all, seed, &[1, 2]);
                if both < 0.95 {
                    if let (Some(x), Some(y)) = (own_program(&ctx, &groups, a), own_program(&ctx, &groups, b)) {
                        if x == y {
                            both = 0.95;
                        }
                    }
                }
                if both >= 0.95 && best.map_or(true, |t| both > t.2) {
                    best = Some((a, b, both));
                }
            }
        }
        match best {
            Some((a, b, s)) => {
                if std::env::var("BABY_DEBUG").is_ok() {
                    eprintln!("step 3 merge {} + {} (held back {:.2})", describe(&groups[a], &anchors), describe(&groups[b], &anchors), s);
                }
                let moved = groups.remove(b);
                groups[a].extend(moved);
            }
            None => break,
        }
    }

    // Name the groups by their commonest anchors, and number the facts.
    let mut names = Vec::new();
    let mut items = Vec::new();
    let mut origin = Vec::new();
    let mut facts: HashMap<String, usize> = HashMap::new();
    groups.sort_by_key(|g| g.iter().copied().min().unwrap_or(0));
    for (u, g) in groups.iter().enumerate() {
        let mut count: BTreeMap<&str, usize> = BTreeMap::new();
        for &i in g {
            *count.entry(anchors[i].as_str()).or_default() += 1;
        }
        let mut top: Vec<(&str, usize)> = count.into_iter().collect();
        top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        let mut name = top.iter().take(3).map(|t| t.0).collect::<Vec<_>>().join("/");
        if names.contains(&name) {
            name = format!("{name}{u}");
        }
        names.push(name);
        items.extend(ctx.items(g, u, &mut facts));
        origin.extend(g.iter().copied());
    }
    Discovered { items, origin, names }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds;
    use crate::rng::Rng;

    fn unlabelled(seed: u64) -> (Vec<(String, String)>, Vec<usize>) {
        let w = kinds::world(seed);
        let mut order: Vec<usize> = (0..w.train.len()).collect();
        Rng::new(seed + 5).shuffle(&mut order);
        (order.iter().map(|&i| (w.train[i].query.clone(), w.train[i].answer.clone())).collect(), order.iter().map(|&i| w.train[i].unit).collect())
    }

    #[test]
    fn it_never_mixes_things_that_need_different_rules_and_keeps_facts_apart() {
        let (lessons, truth) = unlabelled(1);
        let d = discover(&lessons, 1);
        let names = kinds::UNITS;
        // Facts that cannot be worked out (capital, color, sound, opposite, code) must each stay in a group of their own kind.
        for (u, item) in d.items.iter().zip(&d.origin).map(|(it, o)| (truth[*o], it)) {
            let kind = names[u];
            if ["capital", "color", "sound", "opposite", "code"].contains(&kind) {
                let group_truths: HashSet<usize> =
                    d.items.iter().zip(&d.origin).filter(|(i, _)| i.unit == item.unit).map(|(_, o)| truth[*o]).collect();
                assert!(group_truths.iter().all(|t| *t == u), "'{kind}' got mixed with others in group '{}'", d.names[item.unit]);
            }
        }
        // times and apart need rules of their own: they must not be merged into the linear-rule group or each other.
        let unit_of = |kind: &str| {
            let k = names.iter().position(|n| *n == kind).unwrap();
            let o = truth.iter().position(|t| *t == k).unwrap();
            d.items[d.origin.iter().position(|x| *x == o).unwrap()].unit
        };
        assert_ne!(unit_of("times"), unit_of("sum"));
        let mut school = crate::school::School::from_items(&d.items, d.names.clone(), None, 1, true);
        school.set_careful(false);
        assert!(school.ask("three minus five").answer.is_none(), "answered a minus question with a negative result");
        assert!(school.ask("two minus nine").answer.is_none());
        assert_eq!(school.ask("how far apart are two and nine").answer.as_deref(), Some("seven"));
        assert_ne!(unit_of("apart"), unit_of("minus"));
        assert_ne!(unit_of("times"), unit_of("apart"));
        assert!(d.names.len() >= 6 && d.names.len() <= 16, "found {} units: {:?}", d.names.len(), d.names);
    }
}
