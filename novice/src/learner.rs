// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! The learner of rungs 1 and 2: it finds the kinds of question itself, remembers what it was told, and explains what it can with a program.
//!
//! **Kinds.** A question is its numbers plus its words. A word can name a kind only if the answer is a function of the numbers among the
//! questions that hold it (wording shared by many kinds fails this at once, because the same numbers get different answers). Among the words that
//! pass, the one that covers the most questions names them first. All questions that share a kind word form a group, and a group has one
//! explanation at a time: a program priced in bits, or just the table of what it has been told.
//!
//! **Answers.** What was told is a fact. Otherwise the believable programs vote, each with the belief its cost allows (`2^-bits`). A fact is
//! stated only inside the region where the group has been taught or told, and only when the programs that disagree with the favorite carry less
//! than `2^-RISK_BITS` of the belief. Outside that region it can only guess, and the confidence it states is how often its guesses at that
//! distance have turned out right so far.
//!
//! **Being told.** A told answer that some believable program already gives only narrows the doubt (the others that disagree are dropped) and
//! widens the region. One that none gives starts the search again with the new point included, which is the expensive case. The numbers the
//! refuted programs got wrong (what they said minus what was true) are offered to that search as constants: a contradiction names the ceiling.
//!
//! **Words.** A kind that has a program becomes a word the language can use, so a later kind can be built from earlier ones. A word has to
//! earn its place: all of them are offered to the kinds still unexplained, and only the ones that turn up in a new explanation are kept.

use crate::bits::{self, Cost};
use crate::program::{self, Expr, Found, Lib, Task};
use crate::protocol::{Answer, System};
use crate::question::{parse, Parsed};
use std::collections::BTreeMap;
use std::rc::Rc;

/// A fact is stated only when the favorite program carries all but `2^-RISK_BITS` of the belief. Chosen, not found: the error rate a fact may have.
pub const RISK_BITS: u32 = 10;
/// A group is searched for a program only once it holds this many distinct points.
const MIN_POINTS: usize = 4;
/// Constants every search may use (a hand-built choice). Later searches add the numbers that appear in the data.
const BASE_CONSTS: [i64; 5] = [0, 1, 2, 3, 10];
const MAX_ITEMS: usize = 200_000;

struct Fit {
    found: Vec<Found>,
    lo: Vec<i64>,
    hi: Vec<i64>,
}

struct Group {
    key: String,
    arity: usize,
    /// Every distinct point (the numbers of a question) with the answers it has been given and how often.
    points: BTreeMap<Vec<i64>, BTreeMap<String, u32>>,
    fit: Option<Fit>,
    /// How many points the group held when it was last searched.
    searched_at: usize,
    /// How many words were on offer the last time the group was searched with words.
    words_tried: usize,
    /// Whether any answer it has been given was negative. A negative result for a kind that never showed one is not stated.
    negative: bool,
}

struct Said {
    answer: Option<String>,
    /// Distance bucket of a guess made outside the taught region.
    bucket: Option<usize>,
}

/// Work done, for the report at the end.
#[derive(Default, Debug, Clone, Copy)]
pub struct Stats {
    pub searches: u64,
    pub evaluations: u64,
    pub exhausted: u64,
    pub narrowed: u64,
    pub words_earned: u64,
}

pub struct Novice {
    lessons: Vec<(Parsed, String)>,
    df: BTreeMap<String, u32>,
    groups: Vec<Group>,
    by_key: BTreeMap<String, usize>,
    said: BTreeMap<String, Said>,
    /// Guesses made outside the taught region, by distance bucket: (right, made).
    record: [(u32, u32); 8],
    /// Words that earned their place.
    words: Vec<Rc<Lib>>,
    pub stats: Stats,
}

impl Default for Novice {
    fn default() -> Self {
        Novice { lessons: Vec::new(), df: BTreeMap::new(), groups: Vec::new(), by_key: BTreeMap::new(), said: BTreeMap::new(), record: [(0, 0); 8], words: Vec::new(), stats: Stats::default() }
    }
}

fn majority(answers: &BTreeMap<String, u32>) -> Option<&str> {
    let top = answers.iter().max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))?;
    let ties = answers.values().filter(|&&c| c == *top.1).count();
    (ties == 1).then_some(top.0.as_str())
}

/// Probe points just outside and inside the region the group has seen, where rival programs are made to show themselves.
fn probes(lo: &[i64], hi: &[i64]) -> Vec<Vec<i64>> {
    let mut out: Vec<Vec<i64>> = Vec::new();
    let arity = lo.len();
    let choices = |i: usize| -> Vec<i64> {
        let mut c = vec![(lo[i] + hi[i]) / 2, hi[i] + 1, hi[i] + 3, 2 * hi[i] + 1];
        c.sort_unstable();
        c.dedup();
        c
    };
    if arity == 1 {
        for x in choices(0) {
            out.push(vec![x]);
        }
        out.push(vec![2 * hi[0] + 5]);
    } else if arity == 2 {
        let (a, b) = (choices(0), choices(1));
        for &x in &a {
            for &y in &b {
                out.push(vec![x, y]);
            }
        }
        out.push(vec![hi[0] + 1, 0]);
        out.push(vec![0, hi[1] + 1]);
    }
    out.sort();
    out.dedup();
    out
}

/// A few points to compare two words on.
fn grid(arity: usize) -> Vec<Vec<i64>> {
    if arity == 1 {
        (0..24).map(|x| vec![x]).collect()
    } else {
        (0..12).flat_map(|x| (0..12).map(move |y| vec![x, y])).collect()
    }
}

fn mentions(e: &Expr, word: &str) -> bool {
    match e {
        Expr::Call(lib, xs) => lib.name == word || xs.iter().any(|x| mentions(x, word)),
        Expr::Bin(_, a, b) => mentions(a, word) || mentions(b, word),
        _ => false,
    }
}

fn bucket(dist: i64) -> usize {
    ((64 - (dist.max(1) as u64).leading_zeros()) as usize).min(7)
}

impl Novice {
    /// The group a question belongs to: the rarest of its words that names a group.
    fn group_of(&self, p: &Parsed) -> Option<usize> {
        p.words.iter().filter_map(|w| self.by_key.get(w).map(|&g| (std::cmp::Reverse(self.groups[g].points.len()), w.clone(), g))).min().map(|(_, _, g)| g)
    }

    fn add_point(&mut self, g: usize, nums: &[i64], answer: &str) {
        self.groups[g].negative |= answer.starts_with('-');
        *self.groups[g].points.entry(nums.to_vec()).or_default().entry(answer.to_string()).or_insert(0) += 1;
    }

    fn inputs_and_targets(&self, g: usize) -> Option<(Vec<Vec<i64>>, Vec<i64>)> {
        let grp = &self.groups[g];
        let mut inputs: Vec<Vec<i64>> = Vec::new();
        let mut target: Vec<i64> = Vec::new();
        for (nums, answers) in &grp.points {
            if let (Some(a), true) = (majority(answers).and_then(|a| a.parse::<i64>().ok()), nums.len() == grp.arity) {
                inputs.push(nums.clone());
                target.push(a);
            }
        }
        (grp.arity > 0 && inputs.len() >= MIN_POINTS && inputs.len() == grp.points.len()).then_some((inputs, target))
    }

    /// Run one search for the group and keep the result if it is cheaper to write than the table of answers.
    fn search_group(&mut self, g: usize, consts: &[i64], words: &[Rc<Lib>], max_size: usize, label: &str) -> bool {
        let Some((inputs, target)) = self.inputs_and_targets(g) else { return false };
        let arity = self.groups[g].arity;
        let (mut lo, mut hi) = (inputs[0].clone(), inputs[0].clone());
        for p in &inputs {
            for i in 0..arity {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
        let table: Cost = target.iter().map(|&a| bits::uint(a.unsigned_abs())).sum();
        let probe = probes(&lo, &hi);
        let out = program::search(&Task { inputs: &inputs, target: &target, probes: &probe, arity, consts, words, max_size, rival_size: 2, max_items: MAX_ITEMS });
        if std::env::var_os("NOVICE_TRACE").is_some() {
            let shown = out.found.first().map(|f| f.expr.to_string()).unwrap_or_else(|| "nothing".into());
            eprintln!("search {:<14} {label:<9} points {:>3} consts {:>2} words {:>2} size {max_size} -> {:>11} evaluations, {shown}", self.groups[g].key, inputs.len(), consts.len(), words.len(), out.effort.evaluations);
        }
        self.stats.searches += 1;
        self.stats.evaluations += out.effort.evaluations;
        self.stats.exhausted += u64::from(out.exhausted);
        if out.found.first().is_some_and(|f| f.cost < table) {
            self.groups[g].fit = Some(Fit { found: out.found, lo, hi });
            return true;
        }
        false
    }

    /// Look for a program that explains the group's points. `refuted` are programs the latest told answer contradicted: the numbers they got
    /// wrong are offered to the search as constants.
    fn refit(&mut self, g: usize, refuted: &[Expr]) {
        self.groups[g].searched_at = self.groups[g].points.len();
        self.groups[g].fit = None;
        let Some((inputs, target)) = self.inputs_and_targets(g) else { return };
        // First the plain constants, deeper; then, only if that fails, the numbers that occur in the data and the ones the refuted programs got wrong.
        let mut consts: Vec<i64> = BASE_CONSTS.to_vec();
        if self.search_group(g, &consts, &self.words.clone(), 7, "plain") {
            return;
        }
        for v in target.iter().chain(inputs.iter().flatten()) {
            consts.push(*v);
        }
        for e in refuted {
            for (x, &t) in inputs.iter().zip(&target) {
                if let Some(v) = e.eval(x) {
                    if v != t {
                        consts.extend([(v - t).abs(), v]);
                    }
                }
            }
        }
        consts.retain(|c| (0..=200).contains(c));
        consts.sort_unstable();
        consts.dedup();
        self.search_group(g, &consts, &self.words.clone(), 5, "data");
    }

    /// Offer every explained kind, as a word, to the kinds that still have none; keep the words that turn up in a new explanation.
    fn sleep(&mut self) {
        for _ in 0..3 {
            let mut offer: Vec<Rc<Lib>> = self.words.clone();
            for g in &self.groups {
                let Some(fit) = &g.fit else { continue };
                let body = &fit.found[0].expr;
                if body.size() < 3 || offer.iter().any(|w| w.name == g.key) {
                    continue;
                }
                let lib = Rc::new(Lib { name: g.key.clone(), arity: g.arity, body: body.clone() });
                let same = |a: &Lib, b: &Lib| a.arity == b.arity && grid(a.arity).iter().all(|p| a.body.eval(p) == b.body.eval(p));
                if !offer.iter().any(|w| same(w, &lib)) {
                    offer.push(lib);
                }
            }
            let mut changed = false;
            for g in 0..self.groups.len() {
                if self.groups[g].fit.is_some() || self.groups[g].words_tried >= offer.len() || self.inputs_and_targets(g).is_none() {
                    continue;
                }
                self.groups[g].words_tried = offer.len();
                let mut consts: Vec<i64> = BASE_CONSTS.to_vec();
                let (inputs, target) = self.inputs_and_targets(g).unwrap();
                consts.extend(target.iter().chain(inputs.iter().flatten()).copied().filter(|c| (0..=3).contains(c)));
                if self.search_group(g, &consts, &offer, 5, "words") {
                    let used = self.groups[g].fit.as_ref().map(|f| f.found[0].expr.clone());
                    for w in &offer {
                        if used.as_ref().is_some_and(|e| mentions(e, &w.name)) && !self.words.iter().any(|x| x.name == w.name) {
                            self.words.push(w.clone());
                            self.stats.words_earned += 1;
                        }
                    }
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }

    fn make_groups(&mut self) {
        let lessons = std::mem::take(&mut self.lessons);
        for (p, _) in &lessons {
            for w in &p.words {
                *self.df.entry(w.clone()).or_insert(0) += 1;
            }
        }
        // Which words could name a kind: those whose questions never give one set of numbers two answers.
        let mut holding: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
        for (i, (p, _)) in lessons.iter().enumerate() {
            for w in &p.words {
                holding.entry(w).or_default().push(i);
            }
        }
        let mut candidates: Vec<(&str, &Vec<usize>)> = holding
            .iter()
            .filter(|(_, idx)| {
                let mut seen: BTreeMap<&[i64], &str> = BTreeMap::new();
                idx.iter().all(|&i| *seen.entry(&lessons[i].0.nums).or_insert(&lessons[i].1) == lessons[i].1)
            })
            .map(|(w, idx)| (*w, idx))
            .collect();
        candidates.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(b.0)));
        let mut key_of: Vec<Option<String>> = vec![None; lessons.len()];
        for (w, idx) in candidates {
            for &i in idx {
                key_of[i].get_or_insert_with(|| w.to_string());
            }
        }
        for (i, (p, answer)) in lessons.iter().enumerate() {
            // A question no word can claim falls back to its rarest word.
            let Some(key) = key_of[i].clone().or_else(|| p.words.iter().map(|w| (self.df[w], w.clone())).min().map(|(_, w)| w)) else { continue };
            let g = match self.by_key.get(&key) {
                Some(&g) => g,
                None => {
                    self.groups.push(Group { key: key.clone(), arity: p.nums.len(), points: BTreeMap::new(), fit: None, searched_at: 0, words_tried: 0, negative: false });
                    self.by_key.insert(key, self.groups.len() - 1);
                    self.groups.len() - 1
                }
            };
            if self.groups[g].arity == p.nums.len() {
                self.add_point(g, &p.nums, answer);
            }
        }
        for g in 0..self.groups.len() {
            self.refit(g, &[]);
        }
        self.sleep();
    }
}

impl System for Novice {
    fn teach(&mut self, question: &str, answer: &str) {
        self.lessons.push((parse(question), answer.trim().to_string()));
    }

    fn ready(&mut self) {
        self.make_groups();
    }

    fn ask(&mut self, question: &str) -> Answer {
        let p = parse(question);
        let mut said = Said { answer: None, bucket: None };
        let reply = (|| {
            // A word it has never met may change what is asked: nothing is stated about such a question.
            if p.words.iter().any(|w| !self.df.contains_key(w)) {
                return None;
            }
            let g = &self.groups[self.group_of(&p)?];
            if g.arity != p.nums.len() {
                return None;
            }
            if let Some(answers) = g.points.get(&p.nums) {
                return majority(answers).map(|a| Answer::Fact(a.to_string()));
            }
            let fit = g.fit.as_ref()?;
            let (value, top, total) = program::weigh(&fit.found, &p.nums)?;
            if value < 0 && !g.negative {
                return None;
            }
            let inside = p.nums.iter().enumerate().all(|(i, &n)| fit.lo[i] <= n && n <= fit.hi[i]);
            if inside {
                // All but 2^-RISK_BITS of the belief on this answer?
                let sure = u128::from(top) << RISK_BITS >= u128::from(total) * ((1u128 << RISK_BITS) - 1);
                if sure {
                    return Some(Answer::Fact(value.to_string()));
                }
                let permille = (u128::from(top) * 1000 / u128::from(total.max(1))) as u32;
                return (permille >= 500).then(|| Answer::Guess(value.to_string(), permille.min(999)));
            }
            let dist = p.nums.iter().enumerate().map(|(i, &n)| (n - fit.hi[i]).max(fit.lo[i] - n)).max().unwrap_or(1);
            let b = bucket(dist);
            let (right, made) = self.record[b];
            let permille = (1000 * (right + 1) / (made + 2)).min(999);
            said.bucket = Some(b);
            (permille >= 500).then(|| Answer::Guess(value.to_string(), permille))
        })();
        let reply = reply.unwrap_or(Answer::Refuse);
        if let Answer::Fact(a) | Answer::Guess(a, _) = &reply {
            said.answer = Some(a.clone());
        } else {
            said.bucket = None;
        }
        self.said.insert(question.to_string(), said);
        reply
    }

    fn told(&mut self, question: &str, truth: &str, _source: Option<&str>) {
        let p = parse(question);
        let truth = truth.trim();
        if let Some(said) = self.said.remove(question) {
            if let (Some(b), Some(a)) = (said.bucket, said.answer) {
                self.record[b].1 += 1;
                self.record[b].0 += u32::from(a == truth);
            }
        }
        let g = match self.group_of(&p) {
            Some(g) => g,
            None => {
                // A kind word it has never met names a new group.
                let Some(key) = p.words.iter().filter(|w| !self.df.contains_key(*w)).min().cloned() else { return };
                self.groups.push(Group { key: key.clone(), arity: p.nums.len(), points: BTreeMap::new(), fit: None, searched_at: 0, words_tried: 0, negative: false });
                self.by_key.insert(key, self.groups.len() - 1);
                self.groups.len() - 1
            }
        };
        for w in &p.words {
            self.df.entry(w.clone()).or_insert(1);
        }
        if self.groups[g].arity != p.nums.len() {
            return;
        }
        self.add_point(g, &p.nums, truth);
        let value = truth.parse::<i64>().ok();
        let mut refuted: Option<Vec<Expr>> = None;
        match (&mut self.groups[g].fit, value) {
            (Some(fit), Some(v)) => {
                // Programs that already say this stay; the rest are refuted by it.
                let before = fit.found.len();
                let old: Vec<Expr> = fit.found.iter().take(4).map(|f| f.expr.clone()).collect();
                fit.found.retain(|f| f.expr.eval(&p.nums) == Some(v));
                if fit.found.is_empty() {
                    refuted = Some(old);
                } else {
                    self.stats.narrowed += u64::from(fit.found.len() < before);
                    for (i, &n) in p.nums.iter().enumerate() {
                        fit.lo[i] = fit.lo[i].min(n);
                        fit.hi[i] = fit.hi[i].max(n);
                    }
                }
            }
            (Some(fit), None) => refuted = Some(fit.found.iter().take(4).map(|f| f.expr.clone()).collect()),
            (None, _) => {
                // Only a table so far: look again when the group has doubled since the last search.
                let grp = &self.groups[g];
                if grp.points.len() >= 2 * grp.searched_at.max(MIN_POINTS / 2) {
                    refuted = Some(Vec::new());
                }
            }
        }
        if let Some(old) = refuted {
            self.refit(g, &old);
            if self.groups[g].fit.is_some() {
                self.sleep();
            }
        }
    }

    fn quit(&mut self) {
        let s = self.stats;
        let fits = self.groups.iter().filter(|g| g.fit.is_some()).count();
        eprintln!(
            "novice: {} groups, {} explained by a program; {} searches, {} evaluations, {} hit the size limit; {} told answers narrowed the doubt; {} words earned: {}",
            self.groups.len(),
            fits,
            s.searches,
            s.evaluations,
            s.exhausted,
            s.narrowed,
            s.words_earned,
            self.words.iter().map(|w| w.name.as_str()).collect::<Vec<_>>().join(" ")
        );
        for g in &self.groups {
            if let Some(f) = &g.fit {
                eprintln!("  {:<14} {} ({} rival behaviors)", g.key, f.found[0].expr, f.found.len() - 1);
            } else {
                eprintln!("  {:<14} table of {}", g.key, g.points.len());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny world: "x minus y" (never negative in the lessons), "x plus y" and a unary doubling, each under an invented word.
    fn taught() -> Novice {
        let mut n = Novice::default();
        for (x, y) in [(5, 1), (7, 2), (9, 4), (3, 3), (8, 0), (6, 5), (10, 7), (4, 2)] {
            for template in ["what is {x} kigisido {y}", "{x} kigisido {y}", "{x} kigisido {y} equals"] {
                n.teach(&template.replace("{x}", &x.to_string()).replace("{y}", &y.to_string()), &(x - y).to_string());
            }
            n.teach(&format!("what is {x} fegisido {y}"), &(x + y).to_string());
        }
        for x in [1, 2, 4, 5, 7, 9] {
            n.teach(&format!("what is pugisido {x}"), &(2 * x).to_string());
        }
        n.ready();
        n
    }

    fn fact(a: &Answer) -> Option<&str> {
        match a {
            Answer::Fact(s) => Some(s),
            _ => None,
        }
    }

    #[test]
    fn it_explains_the_kinds_it_was_taught_without_being_told_which_they_are() {
        let mut n = taught();
        assert_eq!(fact(&n.ask("what is 6 fegisido 3")), Some("9"));
        assert_eq!(fact(&n.ask("what is 9 kigisido 3")), Some("6"));
        assert_eq!(fact(&n.ask("what is pugisido 8")), Some("16"));
    }

    #[test]
    fn a_word_it_has_never_met_means_no_fact() {
        let mut n = taught();
        assert_eq!(n.ask("what is 3 blorp 4"), Answer::Refuse);
        // A known kind with an extra, unknown word in the question may mean something else entirely.
        assert_eq!(n.ask("what is 6 fegisido 3 never"), Answer::Refuse);
        assert_eq!(n.ask("what is not pugisido 8"), Answer::Refuse);
    }

    #[test]
    fn the_wrong_number_of_numbers_is_refused() {
        let mut n = taught();
        assert_eq!(n.ask("what is fegisido 4"), Answer::Refuse);
        assert_eq!(n.ask("what is 1 pugisido 4"), Answer::Refuse);
        assert_eq!(n.ask("what is 1 fegisido 2 3"), Answer::Refuse);
        assert_eq!(n.ask("what is fegisido"), Answer::Refuse);
    }

    #[test]
    fn an_answer_of_a_kind_never_seen_is_not_stated() {
        // No lesson showed a negative result for "kigisido"; 3 - 5 is inside the region of the numbers taught.
        let mut n = taught();
        assert!(matches!(n.ask("what is 3 kigisido 5"), Answer::Refuse));
    }

    #[test]
    fn what_it_was_told_is_a_fact_and_a_contradiction_is_learned_from() {
        let mut n = taught();
        assert_eq!(fact(&n.ask("what is 6 fegisido 3")), Some("9"));
        n.told("what is 6 fegisido 3", "9", None);
        // A kind it knows nothing about yet becomes a group when it is told.
        n.told("what is 4 zorbosido 2", "8", None);
        assert_eq!(fact(&n.ask("what is 4 zorbosido 2")), Some("8"));
    }

    #[test]
    fn a_ceiling_is_found_from_the_numbers_the_mistakes_name() {
        let mut n = Novice::default();
        for (x, y) in [(1, 2), (3, 4), (5, 5), (8, 1), (6, 6), (2, 9), (0, 7), (9, 3)] {
            n.teach(&format!("what is {x} nakisido {y}"), &(x + y).to_string());
        }
        n.ready();
        // Beyond what it was taught it can only guess, and it says so.
        assert!(!matches!(n.ask("what is 25 nakisido 20"), Answer::Fact(_)));
        n.told("what is 25 nakisido 20", "30", None);
        n.told("what is 22 nakisido 21", "30", None);
        // The first contradiction left a rival standing by luck (`y + min(x, 10)` also fits); the second removed it and the search found the
        // ceiling. Inside the region it has now seen, the ceiling is a fact.
        assert_eq!(fact(&n.ask("what is 23 nakisido 20")), Some("30"));
        assert_eq!(fact(&n.ask("what is 3 nakisido 4")), Some("7"));
    }
}
