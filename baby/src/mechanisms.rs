// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DawsMacs
//! A library of ways to learn, and a baby that picks among them by evidence.
//!
//!   Remember  write the questions down; answer a new one with the answer of the most alike (cheap, learns nothing general)
//!   Rule      a learned combination of quantities (sums, "what comes after", "double"); declines what it does not understand
//!   Select    learn whether a question wants the bigger or the smaller of two numbers
//!   Program   search for a small expression over the question's numbers that reproduces every answer (builds the rule)
//!   Network   a small network (the bag reader) trained on the unit's questions
//!
//! For each unit of lessons the baby asks: which of these, trained on most of what I was taught, answers best on the facts I held
//! back? That is a cross-validation it runs itself. The best one wins; if two are about equally good the cheaper one does. Nothing
//! in the code says which mechanism suits which unit, and a new unit gets the same treatment.
//!
//! Questions are sent to a unit by similarity to the questions each unit was taught, rare words counting for more.

use crate::data::{tokenize, Example, Family};
use crate::kinds::{Item, World, UNITS};
use crate::ledger::Ledger;
use crate::know;
use crate::model::Model;
use crate::net::{Reader, Scratch};
use crate::program::ProgramRule;
use crate::quantity::{counting_lessons, NumberLine, Quantity};
use crate::rng::Rng;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mech {
    Remember,
    Rule,
    Select,
    Program,
    Network,
}

impl Mech {
    /// Cheapest first: ties go to the earlier one.
    pub const ALL: [Mech; 5] = [Mech::Remember, Mech::Rule, Mech::Select, Mech::Program, Mech::Network];

    pub fn name(self) -> &'static str {
        match self {
            Mech::Remember => "remember",
            Mech::Rule => "rule",
            Mech::Select => "select",
            Mech::Program => "program",
            Mech::Network => "network",
        }
    }
}

/// How much each word counts: rare across everything taught counts more; a word never met counts the most.
pub struct Weights {
    df: HashMap<String, u32>,
    n: u32,
    /// In how many different units of lessons each word has turned up.
    spread: HashMap<String, usize>,
}

impl Weights {
    pub fn from(items: &[Item]) -> Weights {
        Weights::from_lists(items.iter().map(|i| (i.unit, tokenize(&i.query))).collect())
    }

    /// From (unit, words) lists, one per taught question.
    pub fn from_lists(lists: Vec<(usize, Vec<String>)>) -> Weights {
        let mut df: HashMap<String, u32> = HashMap::new();
        let mut units: HashMap<String, std::collections::BTreeSet<usize>> = HashMap::new();
        let n = lists.len() as u32;
        for (unit, mut seen) in lists {
            for w in &seen {
                units.entry(w.clone()).or_default().insert(unit);
            }
            seen.sort();
            seen.dedup();
            for w in seen {
                *df.entry(w).or_insert(0) += 1;
            }
        }
        let spread = units.into_iter().map(|(w, u)| (w, u.len())).collect();
        Weights { df, n, spread }
    }

    pub fn knows(&self, word: &str) -> bool {
        self.df.contains_key(word)
    }

    pub fn words(&self) -> Vec<String> {
        self.df.keys().cloned().collect()
    }

    /// A word that turns up in several different units (what, is, the...) carries no instruction of its own: an operation word
    /// belongs to one kind of lesson. (Frequency alone would call "plus" filler just because there are many sums.)
    pub fn is_filler(&self, word: &str) -> bool {
        self.spread.get(word).map_or(false, |s| *s >= 2)
    }

    pub fn w(&self, word: &str) -> f32 {
        let d = self.df.get(word).copied().unwrap_or(0);
        (((1 + self.n) as f32) / ((1 + d) as f32)).ln().max(0.0) + 0.05
    }

    /// Save as lines: a header, then one line per word.
    pub fn dump(&self, tag: &str, out: &mut String) {
        use std::fmt::Write;
        let mut words: Vec<&String> = self.df.keys().collect();
        words.sort();
        writeln!(out, "{tag} {} {}", self.n, words.len()).unwrap();
        for w in words {
            writeln!(out, "w {w} {} {}", self.df[w], self.spread.get(w).copied().unwrap_or(0)).unwrap();
        }
    }

    pub fn load<'a>(tag: &str, lines: &mut impl Iterator<Item = &'a str>) -> Result<Weights, String> {
        let head = lines.next().ok_or("missing weights header")?;
        let p: Vec<&str> = head.split_whitespace().collect();
        if p.len() != 3 || p[0] != tag {
            return Err(format!("expected '{tag}' header, found '{head}'"));
        }
        let n: u32 = p[1].parse().map_err(|e| format!("{e}"))?;
        let count: usize = p[2].parse().map_err(|e| format!("{e}"))?;
        let (mut df, mut spread) = (HashMap::new(), HashMap::new());
        for _ in 0..count {
            let l = lines.next().ok_or("truncated weights")?;
            let f: Vec<&str> = l.split_whitespace().collect();
            if f.len() != 4 || f[0] != "w" {
                return Err(format!("bad weights line '{l}'"));
            }
            df.insert(f[1].to_string(), f[2].parse().map_err(|e| format!("{e}"))?);
            spread.insert(f[1].to_string(), f[3].parse().map_err(|e| format!("{e}"))?);
        }
        Ok(Weights { df, n, spread })
    }

    /// Weighted Jaccard of two sorted, de-duplicated word lists.
    pub fn similarity(&self, a: &[String], b: &[String]) -> f32 {
        self.similarity_steps(a, b).0
    }

    /// The same, and how many comparison steps it took (so the work can be counted).
    pub fn similarity_steps(&self, a: &[String], b: &[String]) -> (f32, u64) {
        let (mut i, mut j, mut inter, mut union) = (0, 0, 0.0f32, 0.0f32);
        let mut steps = 0u64;
        while i < a.len() || j < b.len() {
            steps += 1;
            if j >= b.len() || (i < a.len() && a[i] < b[j]) {
                union += self.w(&a[i]);
                i += 1;
            } else if i >= a.len() || b[j] < a[i] {
                union += self.w(&b[j]);
                j += 1;
            } else {
                let x = self.w(&a[i]);
                inter += x;
                union += x;
                i += 1;
                j += 1;
            }
        }
        (if union > 0.0 { inter / union } else { 0.0 }, steps)
    }
}

pub fn key(q: &str) -> Vec<String> {
    let mut k = tokenize(q);
    k.sort();
    k.dedup();
    k
}

/// Learn whether a question wants the bigger or the smaller of its two numbers, one score per other word.
pub struct SelectRule {
    line: NumberLine,
    s: HashMap<String, f32>,
}

impl SelectRule {
    pub fn scores(&self) -> Vec<(String, f32)> {
        let mut v: Vec<(String, f32)> = self.s.iter().map(|(w, x)| (w.clone(), *x)).collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    }

    pub fn from_scores(scores: Vec<(String, f32)>) -> SelectRule {
        SelectRule { line: NumberLine::from_counting(&counting_lessons()), s: scores.into_iter().collect() }
    }

    fn operands(&self, q: &str) -> Option<([String; 2], [f32; 2], Vec<String>)> {
        let mut words = Vec::new();
        let mut vals = Vec::new();
        let mut rest = Vec::new();
        for w in tokenize(q) {
            match self.line.value(&w) {
                Some(v) => {
                    words.push(w);
                    vals.push(v);
                }
                None => rest.push(w),
            }
        }
        if words.len() != 2 || vals[0] == vals[1] {
            return None;
        }
        Some(([words[0].clone(), words[1].clone()], [vals[0], vals[1]], rest))
    }

    fn fit(items: &[Item]) -> SelectRule {
        let mut r = SelectRule { line: NumberLine::from_counting(&counting_lessons()), s: HashMap::new() };
        let mut data = Vec::new();
        for it in items {
            if let Some((words, vals, rest)) = r.operands(&it.query) {
                if let Some(pos) = words.iter().position(|w| *w == it.answer) {
                    let y = if vals[pos] > vals[1 - pos] { 1.0f32 } else { -1.0 };
                    data.push((rest, y));
                }
            }
        }
        for (rest, _) in &data {
            for w in rest {
                r.s.entry(w.clone()).or_insert(0.0);
            }
        }
        for _ in 0..100 {
            for (rest, y) in &data {
                let score: f32 = rest.iter().map(|w| r.s[w]).sum();
                if y * score < 1.0 {
                    for w in rest {
                        *r.s.get_mut(w).unwrap() += 0.2 * y;
                    }
                }
            }
        }
        r
    }

    pub fn predict(&self, q: &str, filler: &dyn Fn(&str) -> bool) -> Option<String> {
        let (words, vals, rest) = self.operands(q)?;
        if rest.iter().any(|w| !self.s.contains_key(w) && !filler(w)) {
            return None;
        }
        let score: f32 = rest.iter().map(|w| self.s.get(w).copied().unwrap_or(0.0)).sum();
        if score.abs() < 0.5 {
            return None;
        }
        let bigger = if vals[0] > vals[1] { 0 } else { 1 };
        Some(words[if score > 0.0 { bigger } else { 1 - bigger }].clone())
    }
}

/// Plain memory of the questions it was taught, with a note of which words are glue.
#[derive(Clone)]
pub struct Remembered {
    pub memory: Vec<(Vec<String>, String)>,
    /// For each non-glue word, which remembered questions contain it: lets recall look at a handful of questions, not all of them.
    index: HashMap<String, Vec<usize>>,
    /// Words that turn up across many different facts of the unit ("capital", "of", "code", "for"): they set the kind of
    /// question, not which one. Every OTHER word of a question must match the remembered question it is answered from.
    pub glue: std::collections::HashSet<String>,
}

impl Remembered {
    pub fn new(memory: Vec<(Vec<String>, String)>, glue: std::collections::HashSet<String>) -> Remembered {
        let mut index: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, (k, _)) in memory.iter().enumerate() {
            for t in k {
                if !glue.contains(t) {
                    index.entry(t.clone()).or_default().push(i);
                }
            }
        }
        Remembered { memory, index, glue }
    }

    /// Glue = a word found in at least three different facts (and a tenth of them).
    pub fn from_items(items: &[Item]) -> Remembered {
        let mut facts: HashMap<String, std::collections::BTreeSet<usize>> = HashMap::new();
        for it in items {
            for w in tokenize(&it.query) {
                facts.entry(w).or_default().insert(it.fact);
            }
        }
        let n_facts = items.iter().map(|i| i.fact).collect::<std::collections::BTreeSet<_>>().len();
        let bar = 3.max(n_facts / 10);
        let glue = facts.into_iter().filter(|(_, f)| f.len() >= bar).map(|(w, _)| w).collect();
        Remembered::new(items.iter().map(|i| (key(&i.query), i.answer.clone())).collect(), glue)
    }

    pub fn recall(&self, q: &str, w: &Weights) -> Option<String> {
        self.recall_counted(q, w, &mut Ledger::default(), false)
    }

    /// Answer from memory, counting the work. With `indexed`, only remembered questions that share a content word with this one are
    /// compared (the others could never be answered from anyway: every content word of the question must be in the one it matches).
    pub fn recall_counted(&self, q: &str, w: &Weights, ledger: &mut Ledger, indexed: bool) -> Option<String> {
        let k = key(q);
        let candidates: Vec<usize> = if indexed {
            let mut v: Vec<usize> = k
                .iter()
                .filter(|t| !self.glue.contains(*t))
                .flat_map(|t| self.index.get(t).into_iter().flatten().copied())
                .collect();
            v.sort_unstable();
            v.dedup();
            ledger.int_ops += (k.len() + v.len()) as u64;
            ledger.words_read += (k.len() + v.len()) as u64;
            v
        } else {
            (0..self.memory.len()).collect()
        };
        let mut best: Option<(f32, usize)> = None;
        for i in candidates {
            let (sim, steps) = w.similarity_steps(&k, &self.memory[i].0);
            ledger.int_ops += steps;
            ledger.words_read += steps;
            ledger.macs += steps / 2;
            if sim >= 0.5 && best.map_or(true, |(s, _)| sim > s) {
                best = Some((sim, i));
            }
        }
        let (_, i) = best?;
        let (best_key, answer) = (&self.memory[i].0, &self.memory[i].1);
        // The word that fills the blank must be the same one. A question about "five" is not answered with what was taught about "one".
        if k.iter().any(|t| !self.glue.contains(t) && !best_key.contains(t)) {
            return None;
        }
        Some(answer.clone())
    }
}

pub enum Fitted {
    Remember(Remembered),
    Rule(Quantity),
    Select(SelectRule),
    Program(Option<ProgramRule>),
    Network(Box<Model>),
}

fn as_examples(items: &[Item]) -> Vec<Example> {
    // The family field belongs to the older benchmark; it is plumbing here and nothing reads it.
    items.iter().map(|i| Example { family: Family::Color, template: i.template, query: i.query.clone(), answer: i.answer.clone() }).collect()
}

pub fn fit(mech: Mech, items: &[Item], weights: &Weights, seed: u64) -> Fitted {
    match mech {
        Mech::Remember => Fitted::Remember(Remembered::from_items(items)),
        Mech::Rule => {
            // Solved exactly (see `Quantity::fit_ridge`): no passes, no learning rate. Filler words are pulled hard towards zero.
            let (rule, _ops) = Quantity::fit_ridge(
                &as_examples(items),
                NumberLine::from_counting(&counting_lessons()),
                &|w| if weights.is_filler(w) { 5.0 } else { 0.0 },
            );
            Fitted::Rule(rule)
        }
        Mech::Select => Fitted::Select(SelectRule::fit(items)),
        Mech::Program => Fitted::Program(ProgramRule::fit(items)),
        Mech::Network => Fitted::Network(Box::new(know::fit_examples(&as_examples(items), Reader::Bag, seed))),
    }
}

pub fn predict(f: &Fitted, q: &str, w: &Weights) -> Option<String> {
    match f {
        Fitted::Remember(r) => r.recall(q, w),
        Fitted::Rule(r) => r.answer_ignoring(q, &|x| w.is_filler(x)).filter(|(_, d)| *d < 0.25).map(|x| x.0),
        Fitted::Select(s) => s.predict(q, &|x| w.is_filler(x)),
        Fitted::Program(p) => p.as_ref().and_then(|p| p.predict(q, &|x| w.is_filler(x))),
        Fitted::Network(m) => {
            let mut s = Scratch::new(&m.net.cfg);
            m.ask(q, 1, &mut s).into_iter().next().map(|x| x.0)
        }
    }
}

/// What the baby learned about one unit of lessons.
pub struct UnitModel {
    pub chosen: Mech,
    /// Held-back accuracy of each mechanism, in `Mech::ALL` order.
    pub evidence: [f64; 5],
    /// The expression the search built for this unit, if it found one.
    pub program: Option<String>,
    pub fitted: Vec<Fitted>,
}

pub struct Baby {
    pub units: Vec<UnitModel>,
    /// For sending a question to a unit: what each unit was taught, by *shape* (every number is just "a number").
    pub(crate) taught: Vec<Vec<Vec<String>>>,
    pub(crate) route_weights: Weights,
    pub(crate) line: NumberLine,
    pub(crate) weights: Weights,
}

/// A question reduced to its shape: numbers become numbered placeholders, so "zero plus six" and "two plus four" look alike and
/// the operation word is what stands out.
pub fn shape(q: &str, line: &NumberLine) -> Vec<String> {
    let mut k = 0;
    let mut out: Vec<String> = tokenize(q)
        .into_iter()
        .map(|w| {
            if line.value(&w).is_some() {
                k += 1;
                format!("<num>{k}")
            } else {
                w
            }
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Hold back some facts of one unit, learn with each of `mechs` from the rest, and say which held-back items each got right.
/// Result: for each mechanism, one flag per item (items that were never held back count as wrong, but with five folds all are).
pub fn cv_correct(unit_items: &[Item], weights: &Weights, seed: u64, mechs: &[usize]) -> Vec<Vec<bool>> {
    let mut facts: Vec<usize> = unit_items.iter().map(|i| i.fact).collect();
    facts.sort();
    facts.dedup();
    Rng::new(seed.wrapping_add(31)).shuffle(&mut facts);
    let folds = facts.len().min(5).max(2);
    let mut correct = vec![vec![false; unit_items.len()]; mechs.len()];
    for f in 0..folds {
        let held: Vec<usize> = facts.iter().enumerate().filter(|(i, _)| i % folds == f).map(|(_, x)| *x).collect();
        let test_idx: Vec<usize> = (0..unit_items.len()).filter(|&i| held.contains(&unit_items[i].fact)).collect();
        let learn: Vec<Item> = unit_items.iter().filter(|i| !held.contains(&i.fact)).cloned().collect();
        for (k, mi) in mechs.iter().enumerate() {
            let fitted = fit(Mech::ALL[*mi], &learn, weights, seed);
            for &t in &test_idx {
                correct[k][t] = predict(&fitted, &unit_items[t].query, weights).as_deref() == Some(unit_items[t].answer.as_str());
            }
        }
    }
    correct
}

pub fn cv_accuracy(unit_items: &[Item], weights: &Weights, seed: u64, mechs: &[usize]) -> Vec<f64> {
    cv_correct(unit_items, weights, seed, mechs)
        .iter()
        .map(|c| c.iter().filter(|x| **x).count() as f64 / unit_items.len().max(1) as f64)
        .collect()
}

pub fn evidence(unit_items: &[Item], weights: &Weights, seed: u64) -> [f64; 5] {
    let all = cv_accuracy(unit_items, weights, seed, &[0, 1, 2, 3, 4]);
    [all[0], all[1], all[2], all[3], all[4]]
}

/// As `evidence`, but the expensive mechanism (the network) is only tried if the cheaper ones did not already generalise
/// (98% or better on facts held back). A network trained and cross-validated for nothing is pure waste.
pub fn evidence_thrifty(unit_items: &[Item], weights: &Weights, seed: u64) -> [f64; 5] {
    let cheap = cv_accuracy(unit_items, weights, seed, &[0, 1, 2, 3]);
    let best = cheap.iter().cloned().fold(0.0, f64::max);
    let network = if best >= 0.98 { 0.0 } else { cv_accuracy(unit_items, weights, seed, &[4])[0] };
    [cheap[0], cheap[1], cheap[2], cheap[3], network]
}

/// The best mechanism; a tie (within two points) goes to the cheaper one. A mechanism must clear a small bar to replace plain
/// memory: if nothing shows any real ability to answer facts it was not taught, the safe thing is to remember what it was told.
/// With fewer than five distinct facts there is not enough evidence to trust anything but memory.
pub fn choose_mechanism(ev: &[f64; 5], facts: usize) -> Mech {
    let best = ev.iter().cloned().fold(0.0, f64::max);
    if best < 0.25 || facts < 5 {
        Mech::Remember
    } else {
        Mech::ALL.iter().enumerate().find(|(i, _)| ev[*i] >= best - 0.02).map(|(_, m)| *m).unwrap()
    }
}

pub fn learn(world: &World, seed: u64) -> Baby {
    learn_with(world, seed, false)
}

/// `thrifty`: skip the network where cheaper mechanisms already generalise, and fit only what will be used. The unused
/// mechanisms are left as placeholders that decline.
pub fn learn_with(world: &World, seed: u64, thrifty: bool) -> Baby {
    learn_items(&world.train, UNITS.len(), seed, thrifty)
}

/// As `learn_with`, for any lessons already grouped into `n_units` units (`Item::unit` says which).
pub fn learn_items(train: &[Item], n_units: usize, seed: u64, thrifty: bool) -> Baby {
    let weights = Weights::from(train);
    let line = NumberLine::from_counting(&counting_lessons());
    let mut units = Vec::new();
    let mut taught = Vec::new();
    for u in 0..n_units {
        let items: Vec<Item> = train.iter().filter(|i| i.unit == u).cloned().collect();
        let ev = if thrifty { evidence_thrifty(&items, &weights, seed) } else { evidence(&items, &weights, seed) };
        let n_facts = items.iter().map(|i| i.fact).collect::<std::collections::BTreeSet<_>>().len();
        let chosen = choose_mechanism(&ev, n_facts);
        let fitted: Vec<Fitted> = Mech::ALL
            .iter()
            .map(|m| if !thrifty || *m == chosen || *m == Mech::Remember { fit(*m, &items, &weights, seed) } else { Fitted::Program(None) })
            .collect();
        let program = fitted.iter().find_map(|f| if let Fitted::Program(Some(p)) = f { Some(p.expr.show()) } else { None });
        units.push(UnitModel { chosen, evidence: ev, program, fitted });
        taught.push(items.iter().map(|i| shape(&i.query, &line)).collect::<Vec<_>>());
    }
    let route_weights =
        Weights::from_lists(taught.iter().enumerate().flat_map(|(u, qs)| qs.iter().map(move |q| (u, q.clone()))).collect());
    Baby { units, taught, route_weights, line, weights }
}

impl Baby {
    /// Which unit a question belongs to: the one whose taught questions it most resembles.
    pub fn unit_of(&self, q: &str) -> usize {
        let k = shape(q, &self.line);
        (0..self.taught.len())
            .map(|u| {
                let best = self.taught[u].iter().map(|t| self.route_weights.similarity(&k, t)).fold(0.0f32, f32::max);
                (u, best)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|x| x.0)
            .unwrap()
    }

    /// Answer with the chosen mechanism (or, for the comparison systems, one fixed mechanism for every unit).
    pub fn answer(&self, q: &str, forced: Option<Mech>) -> Option<String> {
        let um = &self.units[self.unit_of(q)];
        let m = forced.unwrap_or(um.chosen);
        let idx = Mech::ALL.iter().position(|x| *x == m).unwrap();
        predict(&um.fitted[idx], q, &self.weights)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds;

    #[test]
    fn the_baby_picks_a_sensible_mechanism_for_each_unit_without_being_told() {
        let w = kinds::world(1);
        let b = learn(&w, 1);
        let chosen: Vec<Mech> = b.units.iter().map(|u| u.chosen).collect();
        assert_eq!(chosen[0], Mech::Remember, "facts are remembered");
        assert_eq!(chosen[1], Mech::Rule, "sums are a rule");
        assert_eq!(chosen[2], Mech::Select, "comparison is a selection");
        assert_eq!(chosen[3], Mech::Rule);
        assert_eq!(chosen[4], Mech::Rule);
        assert_eq!(chosen[5], Mech::Rule, "halving, added as data only");
        assert_eq!(chosen[6], Mech::Program, "multiplying needs a rule it has to build");
        assert_eq!(chosen[7], Mech::Program, "how far apart needs a rule it has to build");
        assert!(b.units[8].evidence.iter().cloned().fold(0.0, f64::max) < 0.5, "an arbitrary code must not be 'explained'");
    }

    #[test]
    fn select_learns_bigger_and_smaller_from_examples() {
        let w = kinds::world(1);
        let items: Vec<Item> = w.train.iter().filter(|i| i.unit == 2).cloned().collect();
        let s = SelectRule::fit(&items);
        assert_eq!(s.predict("which is bigger seven or nine", &|_| false).as_deref(), Some("nine"));
        assert_eq!(s.predict("smaller ten or four", &|_| false).as_deref(), Some("four"));
        assert_eq!(s.predict("capital of three or four", &|_| false), None);
    }
}


