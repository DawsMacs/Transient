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
use crate::program::{Library, ProgramRule};
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
///
/// Kept so that one more lesson can be added in time proportional to its words (`add`), not to everything ever taught: the weights of
/// the whole world used to be recomputed from every lesson on every new one, which made learning slower the more it knew.
#[derive(PartialEq, Debug)]
pub struct Weights {
    df: HashMap<String, u32>,
    n: u32,
    /// In which units of lessons each word has turned up (sorted). How many different ones is the word's *spread*.
    units: HashMap<String, Vec<usize>>,
    /// What it has learned to build with (see `program::Library`): shared by every unit's search, grown as units settle.
    pub lib: Library,
}

impl Weights {
    pub fn from(items: &[Item]) -> Weights {
        Weights::from_lists(items.iter().map(|i| (i.unit, tokenize(&i.query))).collect())
    }

    /// From (unit, words) lists, one per taught question.
    pub fn from_lists(lists: Vec<(usize, Vec<String>)>) -> Weights {
        let mut w = Weights { df: HashMap::new(), n: 0, units: HashMap::new(), lib: Library::default() };
        for (unit, words) in &lists {
            w.add(*unit, words);
        }
        w
    }

    /// One more taught question: its words, and the unit it was taught in.
    pub fn add(&mut self, unit: usize, words: &[String]) {
        self.n += 1;
        let mut seen: Vec<&String> = words.iter().collect();
        seen.sort();
        seen.dedup();
        for w in seen {
            *self.df.entry(w.clone()).or_insert(0) += 1;
            let in_units = self.units.entry(w.clone()).or_default();
            if let Err(at) = in_units.binary_search(&unit) {
                in_units.insert(at, unit);
            }
        }
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
        self.units.get(word).map_or(false, |s| s.len() >= 2)
    }

    pub fn w(&self, word: &str) -> f32 {
        let d = self.df.get(word).copied().unwrap_or(0);
        (((1 + self.n) as f32) / ((1 + d) as f32)).ln().max(0.0) + 0.05
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
    /// Each remembered question's words in the order they were said (`memory` keeps only the set, so rewordings match).
    order: Vec<Vec<String>>,
    /// For each non-glue word, which remembered questions contain it: lets recall look at a handful of questions, not all of them.
    index: HashMap<String, Vec<usize>>,
    /// Words that turn up across many different facts of the unit ("capital", "of", "code", "for"): they set the kind of
    /// question, not which one. Every OTHER word of a question must match the remembered question it is answered from.
    pub glue: std::collections::HashSet<String>,
}

impl Remembered {
    pub fn new(memory: Vec<(Vec<String>, String)>, order: Vec<Vec<String>>, glue: std::collections::HashSet<String>) -> Remembered {
        let mut index: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, (k, _)) in memory.iter().enumerate() {
            for t in k {
                if !glue.contains(t) {
                    index.entry(t.clone()).or_default().push(i);
                }
            }
        }
        Remembered { memory, order, index, glue }
    }

    /// Glue = a word found in at least three different facts and in at least two fifths of them: "what", "is", "the", and the unit's own
    /// word ("capital"). It used to be a tenth, which made the number words of a numeric unit glue too, so the fill-the-blank check
    /// could not see that "two three four" is not "two ... four". A number word is never glue, however common it is in a small numeric
    /// unit ("one" is in over 40% of the times facts, and is not "what is").
    pub fn from_items(items: &[Item]) -> Remembered {
        let mut facts: HashMap<String, std::collections::BTreeSet<usize>> = HashMap::new();
        for it in items {
            for w in tokenize(&it.query) {
                facts.entry(w).or_default().insert(it.fact);
            }
        }
        let n_facts = items.iter().map(|i| i.fact).collect::<std::collections::BTreeSet<_>>().len();
        let bar = 3.max((n_facts * 2 + 4) / 5);
        let glue = facts.into_iter().filter(|(w, f)| f.len() >= bar && !crate::quantity::is_number(w)).map(|(w, _)| w).collect();
        Remembered::new(items.iter().map(|i| (key(&i.query), i.answer.clone())).collect(), items.iter().map(|i| tokenize(&i.query)).collect(), glue)
    }

    /// The content words (not glue) of a question, in the order said, repeats kept ("three times three" is not "three times two").
    fn content<'a>(&self, tokens: &'a [String]) -> Vec<&'a str> {
        tokens.iter().filter(|t| !self.glue.contains(*t)).map(|t| t.as_str()).collect()
    }

    /// The range of each operand over the lessons that share an operation word with `q_words` (every lesson if it has none): for each operand
    /// position, the smallest and largest number the unit was taught there. `value` says whether a word is a number. Lessons listed in `skip`
    /// (sorted indices) are ignored: those are the ones the unit's rule does not explain, and they do not show where it can be trusted.
    pub fn operand_hull(&self, q_words: &[String], value: &dyn Fn(&str) -> Option<f32>, skip: &[usize]) -> Vec<(f32, f32)> {
        let mut candidates: Vec<usize> = q_words
            .iter()
            .filter(|w| value(w).is_none() && !self.glue.contains(*w))
            .flat_map(|w| self.index.get(w).into_iter().flatten().copied())
            .collect();
        if candidates.is_empty() {
            candidates = (0..self.order.len()).collect();
        }
        candidates.sort_unstable();
        candidates.dedup();
        let mut hull: Vec<(f32, f32)> = Vec::new();
        for i in candidates {
            if skip.binary_search(&i).is_ok() {
                continue;
            }
            let mut pos = 0;
            for t in &self.order[i] {
                if let Some(v) = value(t) {
                    match hull.get_mut(pos) {
                        Some(h) => {
                            h.0 = h.0.min(v);
                            h.1 = h.1.max(v);
                        }
                        None => hull.push((v, v)),
                    }
                    pos += 1;
                }
            }
        }
        hull
    }

    /// Is the point `asked` (the numbers of a question, in order) covered by what was taught: some lesson sharing an operation word with the question,
    /// not in `skip`, has every operand at least as large? A box of per-operand ranges alone vouches for corners nobody looked at (two confirmed
    /// edges, (20, 2) and (2, 20), would stretch it to (18, 18)). None if covered; else how far it is from the nearest taught point, in number units
    /// (the largest shortfall among the operands, for the lesson where that is smallest).
    pub fn uncovered_by(&self, q_words: &[String], value: &dyn Fn(&str) -> Option<f32>, skip: &[usize], asked: &[f32]) -> Option<f32> {
        let mut candidates: Vec<usize> = q_words
            .iter()
            .filter(|w| value(w).is_none() && !self.glue.contains(*w))
            .flat_map(|w| self.index.get(w).into_iter().flatten().copied())
            .collect();
        if candidates.is_empty() {
            candidates = (0..self.order.len()).collect();
        }
        candidates.sort_unstable();
        candidates.dedup();
        let mut nearest: Option<f32> = None;
        for i in candidates {
            if skip.binary_search(&i).is_ok() {
                continue;
            }
            let taught: Vec<f32> = self.order[i].iter().filter_map(|t| value(t)).collect();
            if taught.len() != asked.len() {
                continue;
            }
            let shortfall = asked.iter().zip(&taught).map(|(a, t)| (a - t).max(0.0)).fold(0.0f32, f32::max);
            if shortfall <= 0.0 {
                return None;
            }
            nearest = Some(nearest.map_or(shortfall, |n| n.min(shortfall)));
        }
        nearest
    }

    /// What was taught for exactly this question (the same words in the same order), by vote; None if it was never taught as written
    /// or the lessons for it tie. Not a similarity match: a neighbouring lesson is no witness.
    pub fn exact(&self, q: &str) -> Option<String> {
        let tokens = tokenize(q);
        let mut candidates: Vec<usize> = tokens
            .iter()
            .filter(|t| !self.glue.contains(*t))
            .flat_map(|t| self.index.get(t).into_iter().flatten().copied())
            .collect();
        if candidates.is_empty() {
            candidates = (0..self.order.len()).collect();
        }
        candidates.sort_unstable();
        candidates.dedup();
        let mut votes: Vec<(&str, usize)> = Vec::new();
        for i in candidates {
            if self.order[i] == tokens {
                match votes.iter_mut().find(|(a, _)| *a == self.memory[i].1) {
                    Some(v) => v.1 += 1,
                    None => votes.push((&self.memory[i].1, 1)),
                }
            }
        }
        votes.sort_by(|a, b| b.1.cmp(&a.1));
        match votes.as_slice() {
            [] => None,
            [(a, _)] => Some(a.to_string()),
            [(a, x), (_, y), ..] => (x > y).then(|| a.to_string()),
        }
    }

    pub fn recall(&self, q: &str, w: &Weights) -> Option<String> {
        self.recall_counted(q, w, &mut Ledger::default(), false)
    }

    /// Answer from memory, counting the work. With `indexed`, only remembered questions that share a content word with this one are
    /// compared (the others could never be answered from anyway: every content word of the question must be in the one it matches).
    pub fn recall_counted(&self, q: &str, w: &Weights, ledger: &mut Ledger, indexed: bool) -> Option<String> {
        let k = key(q);
        let asked = tokenize(q);
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
        // Every remembered lesson that is close enough and has the question's content words gets a vote, weighted by how close. The plurality
        // answers; a tie refuses. (Taking only the single closest let one wrong lesson out of three phrasings of a fact win, and let two
        // lessons with the same words but different answers, "five minus two" and "two minus five", be settled by their order.)
        // (Voting uses a looser closeness than answering: a wordier phrasing of the same fact scores about 0.45, and must still be heard;
        // but nothing is answered unless at least one lesson is close, 0.5.)
        let mut votes: Vec<(&str, f32)> = Vec::new();
        let mut any_close = false;
        for i in candidates {
            let (sim, steps) = w.similarity_steps(&k, &self.memory[i].0);
            ledger.int_ops += steps;
            ledger.words_read += steps;
            ledger.macs += steps / 2;
            // The word that fills the blank must be the same one. A question about "five" is not answered with what was taught about "one".
            if sim < 0.3 || k.iter().any(|t| !self.glue.contains(t) && !self.memory[i].0.contains(t)) {
                continue;
            }
            // Same words in a different order is a different question: "two minus five" is not "five minus two". The content words
            // (not the glue) must be the remembered question's, in its order and with its repeats.
            if self.content(&asked) != self.content(&self.order[i]) {
                continue;
            }
            any_close |= sim >= 0.5;
            match votes.iter_mut().find(|(a, _)| *a == self.memory[i].1) {
                Some(v) => v.1 += sim,
                None => votes.push((&self.memory[i].1, sim)),
            }
        }
        votes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        if !any_close {
            return None;
        }
        match votes.as_slice() {
            [] => None,
            [(a, _)] => Some(a.to_string()),
            [(a, x), (_, y), ..] => (x - y > 0.05 * x).then(|| a.to_string()),
        }
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
            let (rule, _ops) = Quantity::fit_ridge_robust(&as_examples(items), &|w| if weights.is_filler(w) { 5.0 } else { 0.0 });
            Fitted::Rule(rule)
        }
        Mech::Select => Fitted::Select(SelectRule::fit(items)),
        // The smallest explanations that fit, answering only where they all agree: when several agree on every lesson but differ elsewhere,
        // the data cannot say which was meant there, and building on a guess is how a coincidence gets believed.
        Mech::Program => Fitted::Program(ProgramRule::fit_trusted_with(items, &weights.lib)),
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
    /// The reliability bar this baby measured for its lessons.
    pub(crate) bar: f64,
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

/// Which facts memory cannot tell apart: two facts are one group if any phrasing of one has the same bag of words as a phrasing of the
/// other (union-find). Returns each fact's group (the smallest fact id in it).
fn fact_groups(items: &[Item]) -> HashMap<usize, usize> {
    let mut parent: HashMap<usize, usize> = items.iter().map(|i| (i.fact, i.fact)).collect();
    fn find(p: &mut HashMap<usize, usize>, x: usize) -> usize {
        let mut r = x;
        while p[&r] != r {
            r = p[&r];
        }
        let mut c = x;
        while p[&c] != r {
            let n = p[&c];
            p.insert(c, r);
            c = n;
        }
        r
    }
    let mut seen: HashMap<Vec<String>, usize> = HashMap::new();
    for i in items {
        match seen.get(&key(&i.query)) {
            Some(&other) => {
                let (a, b) = (find(&mut parent, i.fact), find(&mut parent, other));
                if a != b {
                    parent.insert(a.max(b), a.min(b));
                }
            }
            None => {
                seen.insert(key(&i.query), i.fact);
            }
        }
    }
    let facts: Vec<usize> = parent.keys().copied().collect();
    facts.into_iter().map(|f| (f, find(&mut parent, f))).collect()
}

/// Hold back some facts of one unit, learn with each of `mechs` from the rest, and say which held-back items each got right.
/// Result: for each mechanism, one flag per item (items that were never held back count as wrong, but with five folds all are).
pub fn cv_correct(unit_items: &[Item], weights: &Weights, seed: u64, mechs: &[usize]) -> Vec<Vec<bool>> {
    cv_correct_strata(unit_items, weights, seed, mechs, &vec![0; unit_items.len()])
}

/// As `cv_correct`, with the folds stratified: `strata[i]` says which part item `i` belongs to, and each part's facts are spread
/// evenly over the folds. Needed when one part is small next to the other (testing whether a unit of five lessons fits a rule learned
/// with a big family): a random split can leave a fold holding most of the small part, so it is judged on what it was barely taught.
pub fn cv_correct_strata(unit_items: &[Item], weights: &Weights, seed: u64, mechs: &[usize], strata: &[usize]) -> Vec<Vec<bool>> {
    let mut facts: Vec<usize> = unit_items.iter().map(|i| i.fact).collect();
    facts.sort();
    facts.dedup();
    // Facts that memory cannot tell apart belong in the same fold: "two or three" and "three or two" are different facts but the
    // same bag of words, so if one is taught and the other held back, memory "predicts" it by looking it up. Without this a
    // commutative rule looks exactly like a table whenever the twins happen to fall in different folds.
    let group = fact_groups(unit_items);
    let mut groups: Vec<usize> = facts.iter().map(|f| group[f]).collect();
    groups.sort();
    groups.dedup();
    Rng::new(seed.wrapping_add(31)).shuffle(&mut groups);
    let folds = groups.len().min(5).max(2);
    // Deal each stratum's groups round-robin over the folds (one stratum: the plain shuffled split).
    let stratum_of: HashMap<usize, usize> = unit_items.iter().enumerate().map(|(i, it)| (group[&it.fact], strata[i])).collect();
    let mut order: Vec<usize> = (0..groups.len()).collect();
    order.sort_by_key(|&i| stratum_of[&groups[i]]); // stable: keeps the shuffled order inside a stratum
    let mut fold_of: HashMap<usize, usize> = HashMap::new();
    for (k, &i) in order.iter().enumerate() {
        fold_of.insert(groups[i], k % folds);
    }
    let mut correct = vec![vec![false; unit_items.len()]; mechs.len()];
    for f in 0..folds {
        let held_groups: std::collections::HashSet<usize> = groups.iter().filter(|g| fold_of[g] == f).copied().collect();
        let is_held = |fact: usize| held_groups.contains(&group[&fact]);
        let test_idx: Vec<usize> = (0..unit_items.len()).filter(|&i| is_held(unit_items[i].fact)).collect();
        let learn: Vec<Item> = unit_items.iter().filter(|i| !is_held(i.fact)).cloned().collect();
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

/// As `evidence`, but the expensive mechanism (the network) is only tried when the cheaper ones show some structure without being
/// reliable (25% up to the bar). Where they already generalise it is waste, and where they show nothing at all (an arbitrary table of
/// facts) a network cannot do better on facts it was not shown, but cross-validating one costs about 100 ms a unit.
pub fn evidence_thrifty(unit_items: &[Item], weights: &Weights, seed: u64) -> [f64; 5] {
    // Cheapest first. Building a program is the dearest of the cheap ones (a search, now over a library too), so it is only tried when
    // memory, a rule and a selection did not already explain the unit: where one does, the cheaper one wins any tie anyway.
    let first = cv_accuracy(unit_items, weights, seed, &[0, 1, 2]);
    let simple_best = first.iter().cloned().fold(0.0, f64::max);
    // And measured cheaply before spent on: if the search over ALL the lessons finds no program, searching five subsets of them cannot do
    // better (a fold that finds one the whole does not is a coincidence), so a unit with no explanation pays for one failed search, not six.
    let program = if simple_best >= reliable() || matches!(fit(Mech::Program, unit_items, weights, seed), Fitted::Program(None)) {
        0.0
    } else {
        cv_accuracy(unit_items, weights, seed, &[3])[0]
    };
    let best = simple_best.max(program);
    let network = if !(0.25..reliable()).contains(&best) { 0.0 } else { cv_accuracy(unit_items, weights, seed, &[4])[0] };
    [first[0], first[1], first[2], program, network]
}

// The bar in force. It is not a constant: how often a correct rule is right on facts held back cannot exceed about one minus the share of wrong
// lessons, so a fixed 0.9 collapses coverage in a noisy world (42-58 % of unseen questions answered at 10 % noise, against 99-100 % at a bar of 0.7);
// and it cannot go low, because a mechanism right only 72 % of the time on held-back facts does answer wrongly (28 wrong answers at 0.7 on the clean
// battery, 0 at 0.75). So it is `bar_for_noise` of the noise the baby measures in its own lessons. Per thread, set by whoever is about to
// learn (a school installs its own); `--bar` overrides it for measurement.
thread_local! {
    static BAR: std::cell::Cell<f64> = const { std::cell::Cell::new(0.9) };
    static BAR_FIXED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub fn reliable() -> f64 {
    BAR.with(|b| b.get())
}

/// Set the bar (unless a fixed bar was asked for with `--bar`).
pub fn set_reliable(bar: f64) {
    if !BAR_FIXED.with(|f| f.get()) {
        BAR.with(|b| b.set(bar));
    }
}

/// Force a bar for every school (measurement only).
pub fn fix_reliable(bar: f64) {
    BAR_FIXED.with(|f| f.set(true));
    BAR.with(|b| b.set(bar));
}

/// The bar for a world whose lessons are wrong this often: just under the ceiling a correct rule can reach, between 0.8 (a safe floor: the
/// failures were at 72 %) and 0.9 (a clean world, as before).
pub fn bar_for_noise(noise: f64) -> f64 {
    (0.98 - 2.5 * noise).clamp(0.8, 0.9)
}

/// How often lessons that say the same thing (the same content words in the same order, so one fact in several phrasings) disagree with the vote of
/// their own group: an estimate of the share of wrong lessons. Only groups of three or more count (a vote needs voters).
pub fn estimate_noise(units: &[Vec<Item>]) -> f64 {
    let (mut minority, mut total) = (0usize, 0usize);
    for items in units {
        let mem = Remembered::from_items(items);
        let mut groups: HashMap<Vec<&str>, Vec<&str>> = HashMap::new();
        for (i, it) in items.iter().enumerate() {
            groups.entry(mem.content(&mem.order[i])).or_default().push(it.answer.as_str());
        }
        for answers in groups.values().filter(|a| a.len() >= 3) {
            let mut counts: HashMap<&str, usize> = HashMap::new();
            for a in answers {
                *counts.entry(a).or_default() += 1;
            }
            minority += answers.len() - counts.values().max().copied().unwrap_or(0);
            total += answers.len();
        }
    }
    if total == 0 { 0.0 } else { minority as f64 / total as f64 }
}

/// The best mechanism; a tie (within two points) goes to the cheaper one. A mechanism must clear a bar to replace plain memory:
/// if nothing shows a reliable ability to answer facts it was not taught, the safe thing is to remember what it was told.
/// With fewer than five distinct facts there is not enough evidence to trust anything but memory.
pub fn choose_mechanism(ev: &[f64; 5], facts: usize) -> Mech {
    let best = ev.iter().cloned().fold(0.0, f64::max);
    if best < 0.25 || facts < 5 {
        return Mech::Remember;
    }
    let (i, m) = Mech::ALL.iter().enumerate().find(|(i, _)| ev[*i] >= best - 0.02).map(|(i, m)| (i, *m)).unwrap();
    if m != Mech::Remember && ev[i] < reliable() {
        Mech::Remember
    } else {
        m
    }
}

/// Add what a settled unit learned to the library: the constants in a rule, the program of a built unit.
pub fn grow_library(lib: &mut Library, state: &Fitted, origin: usize) {
    match state {
        Fitted::Rule(q) => lib.learn_from_terms(&q.terms()),
        // only a settled program is vocabulary: one that still has a competing explanation is a guess, and building on a guess is how a
        // coincidence spreads (the ambiguous digit sum once entered the library and then hid its own ambiguity)
        Fitted::Program(Some(p)) if p.alts.is_empty() => {
            lib.learn_program(&p.expr, p.arity(), origin);
        }
        _ => {}
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
    let per_unit: Vec<Vec<Item>> = (0..n_units).map(|u| train.iter().filter(|i| i.unit == u).cloned().collect()).collect();
    let bar = bar_for_noise(estimate_noise(&per_unit));
    set_reliable(bar);
    let mut weights = Weights::from(train);
    let line = NumberLine::from_counting(&counting_lessons());
    let mut units = Vec::new();
    let mut taught = Vec::new();
    for u in 0..n_units {
        let items: Vec<Item> = train.iter().filter(|i| i.unit == u).cloned().collect();
        let t_unit = std::time::Instant::now();
        let ev = if thrifty { evidence_thrifty(&items, &weights, seed) } else { evidence(&items, &weights, seed) };
        if std::env::var("BABY_PROF").is_ok() {
            eprintln!("PROF unit {u} ({} lessons): evidence in {:.2}s -> {:?}", items.len(), t_unit.elapsed().as_secs_f64(), ev.map(|e| (e * 100.0).round()));
        }
        let n_facts = items.iter().map(|i| i.fact).collect::<std::collections::BTreeSet<_>>().len();
        let chosen = choose_mechanism(&ev, n_facts);
        let fitted: Vec<Fitted> = Mech::ALL
            .iter()
            .map(|m| if !thrifty || *m == chosen || *m == Mech::Remember { fit(*m, &items, &weights, seed) } else { Fitted::Program(None) })
            .collect();
        let program = fitted.iter().find_map(|f| if let Fitted::Program(Some(p)) = f { Some(p.expr.show()) } else { None });
        // What a reliable unit worked out becomes something later units can build with.
        if ev[Mech::ALL.iter().position(|m| *m == chosen).unwrap()] >= reliable() {
            grow_library(&mut weights.lib, &fitted[Mech::ALL.iter().position(|m| *m == chosen).unwrap()], u);
        }
        units.push(UnitModel { chosen, evidence: ev, program, fitted });
        taught.push(items.iter().map(|i| shape(&i.query, &line)).collect::<Vec<_>>());
    }
    let route_weights =
        Weights::from_lists(taught.iter().enumerate().flat_map(|(u, qs)| qs.iter().map(move |q| (u, q.clone()))).collect());
    Baby { units, taught, route_weights, line, weights, bar }
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

    fn lesson(fact: usize, t: usize, q: &str, a: &str) -> Item {
        Item { unit: 0, fact, template: t, query: q.to_string(), answer: a.to_string() }
    }

    #[test]
    fn memory_outvotes_one_wrong_phrasing_and_refuses_when_lessons_tie() {
        let mut items = Vec::new();
        let countries = [("peru", "lima"), ("chad", "ndjamena"), ("cuba", "havana"), ("iran", "tehran"), ("mali", "bamako"), ("laos", "vientiane"),
                         ("togo", "lome"), ("fiji", "suva"), ("niue", "alofi"), ("oman", "muscat"), ("qatar", "doha"), ("kenya", "nairobi")];
        for (f, (country, capital)) in countries.iter().enumerate() {
            items.push(lesson(f, 0, &format!("capital of {country}"), capital));
            items.push(lesson(f, 1, &format!("what is the capital of {country}"), capital));
            items.push(lesson(f, 2, &format!("{country} capital"), capital));
        }
        // one phrasing of one fact was taught wrongly
        items[0].answer = "quito".into();
        let w = Weights::from(&items);
        let mem = Remembered::from_items(&items);
        assert_eq!(mem.recall("capital of peru", &w).as_deref(), Some("lima"), "two phrasings out of three outvote the wrong one");
        assert_eq!(mem.recall("peru capital", &w).as_deref(), Some("lima"));
        assert_eq!(mem.recall("capital of chad", &w).as_deref(), Some("ndjamena"));
        // the same words in a different order are a different question: each gets its own lesson's answer, never the other's
        let tie = vec![lesson(0, 0, "five minus two", "three"), lesson(1, 0, "two minus five", "negative"), lesson(2, 0, "ten minus one", "nine"),
                       lesson(3, 0, "ten minus two", "eight"), lesson(4, 0, "nine minus one", "eight"), lesson(5, 0, "four minus one", "three")];
        let w = Weights::from(&tie);
        let mem = Remembered::from_items(&tie);
        assert_eq!(mem.recall("five minus two", &w).as_deref(), Some("three"));
        assert_eq!(mem.recall("two minus five", &w).as_deref(), Some("negative"));
        assert_eq!(mem.recall("three minus five", &w), None, "never taught, and the words of two taught ones, in another order");
        // a repeated operand is not the operand once: "three minus three" is not "three minus two"
        let rep = vec![lesson(0, 0, "three minus two", "one"), lesson(1, 0, "four minus two", "two"), lesson(2, 0, "five minus two", "three"), lesson(3, 0, "five minus one", "four")];
        let w = Weights::from(&rep);
        assert_eq!(Remembered::from_items(&rep).recall("three minus three", &w), None);
        // two lessons that say the same words in the same order with different answers are a genuine tie, and it refuses
        let same = vec![lesson(0, 0, "ten minus one", "nine"), lesson(1, 0, "ten minus one", "eight"), lesson(2, 0, "ten minus two", "eight"), lesson(3, 0, "nine minus one", "eight")];
        let w = Weights::from(&same);
        assert_eq!(Remembered::from_items(&same).recall("ten minus one", &w), None);
    }

    #[test]
    fn the_noise_in_its_own_lessons_is_measured_from_lessons_that_say_the_same_thing_and_sets_the_bar() {
        // 40 facts, three phrasings each
        let make = |wrong_every: usize| -> Vec<Item> {
            let mut v = Vec::new();
            for f in 0..40usize {
                for (t, q) in ["what is the capital of {c}", "{c} capital", "the capital of {c}"].iter().enumerate() {
                    let mut answer = format!("city{f}");
                    if wrong_every > 0 && (f * 3 + t) % wrong_every == 0 {
                        answer = "wrongtown".into();
                    }
                    v.push(Item { unit: 0, fact: f, template: t, query: q.replace("{c}", &format!("country{f}")), answer });
                }
            }
            v
        };
        assert_eq!(estimate_noise(&[make(0)]), 0.0, "clean lessons never disagree with their group");
        let eight = estimate_noise(&[make(12)]); // 10 of 120 lessons wrong, each in its own group of three: 8.3 %
        assert!((eight - 10.0 / 120.0).abs() < 0.005, "{eight}");
        assert_eq!(bar_for_noise(0.0), 0.9, "a clean world keeps the bar it always had");
        assert!(bar_for_noise(0.02) > 0.9 - 0.06 && bar_for_noise(0.02) < 0.9 + 1e-9);
        assert_eq!(bar_for_noise(0.083), 0.8);
        assert_eq!(bar_for_noise(0.4), 0.8, "never below the floor: a mechanism right 72 % of the time answers wrongly");
        // and a unit of three groups too small to vote counts for nothing
        assert_eq!(estimate_noise(&[make(0).into_iter().take(2).collect()]), 0.0);
    }

    #[test]
    fn a_mechanism_that_is_usually_wrong_on_facts_it_was_not_shown_is_not_allowed_to_answer() {
        // [remember, rule, select, program, network] held-back accuracy
        assert_eq!(choose_mechanism(&[0.0, 0.0, 0.28, 0.2, 0.0], 29), Mech::Remember, "28% is not a basis for answering");
        assert_eq!(choose_mechanism(&[0.0, 0.5, 0.4, 0.3, 0.0], 30), Mech::Remember);
        assert_eq!(choose_mechanism(&[0.0, 0.0, 0.0, 0.0, 0.30], 22), Mech::Remember);
        assert_eq!(choose_mechanism(&[0.1, 0.89, 0.2, 0.0, 0.0], 30), Mech::Remember);
        assert_eq!(choose_mechanism(&[0.1, 0.90, 0.2, 0.0, 0.0], 30), Mech::Rule);
        assert_eq!(choose_mechanism(&[0.0, 1.0, 0.0, 1.0, 0.0], 30), Mech::Rule, "ties still go to the cheaper");
        assert_eq!(choose_mechanism(&[0.6, 0.0, 0.0, 0.0, 0.0], 30), Mech::Remember);
        assert_eq!(choose_mechanism(&[0.0, 1.0, 0.0, 0.0, 0.0], 4), Mech::Remember, "too few facts");
    }

    /// "which is bigger two or three" and "... three or two" are different facts but the same bag of words, so memory (which ignores
    /// order) answers one from the other. Held-back evidence must not credit memory for that, or a commutative rule is mistaken for
    /// a table whenever its twins happen to land in different folds. (Found at 200 units: 3 of 11 such units were mislabelled.)
    #[test]
    fn a_commutative_rule_is_never_mistaken_for_a_table_by_where_the_folds_fall() {
        let line = ["zero", "one", "two", "three", "four", "five"];
        let mut items = Vec::new();
        let mut fact = 0;
        for a in 0..=5usize {
            for b in 0..=5usize {
                if a != b {
                    for (t, form) in ["which is zorbat {a} or {b}", "zorbat {a} or {b}", "{a} or {b} which is zorbat"].iter().enumerate() {
                        let q = form.replace("{a}", line[a]).replace("{b}", line[b]);
                        items.push(Item { unit: 0, fact, template: t, query: q, answer: line[a.max(b)].to_string() });
                    }
                    fact += 1;
                }
            }
        }
        let weights = Weights::from(&items);
        for seed in 1..=12u64 {
            let ev = evidence_thrifty(&items, &weights, seed);
            assert_eq!(choose_mechanism(&ev, fact), Mech::Select, "seed {seed}: evidence {ev:?}");
        }
    }
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


