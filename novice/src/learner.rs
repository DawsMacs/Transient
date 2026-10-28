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
//! **Sources.** What the lessons said and what each told source says are separate channels, and a channel is as reliable as it has been where a
//! program was confident (a check that only kinds with a rule allow). In a kind with no rule, each channel that spoke counts for its reliability,
//! and the answer is believed as much as they jointly allow: one channel measured at 80 % can only be a guess, two independent ones that agree
//! can be a fact, two that disagree favor the more reliable one with a stated doubt.
//!
//! **Words.** A kind that has a program becomes a word the language can use, so a later kind can be built from earlier ones. A word has to
//! earn its place: all of them are offered to the kinds still unexplained, and only the ones that turn up in a new explanation are kept.

use crate::bits::{self, Cost};
use crate::fit;
use crate::frames::{self, Frames};
use crate::numerals::{self, Lexicon};
use crate::program::{self, Expr, Found, Lib, Task};
use crate::protocol::{Answer, System};
use crate::question::{parse_with, Parsed};
use std::collections::BTreeMap;
use std::rc::Rc;

/// A fact is stated only when the favorite program carries all but `2^-RISK_BITS` of the belief. Chosen, not found: the error rate a fact may have.
pub const RISK_BITS: u32 = 10;
/// A group is searched for a program only once it holds this many distinct points.
const MIN_POINTS: usize = 4;
/// Constants every search may use (a hand-built choice). Later searches add the numbers that appear in the data.
const BASE_CONSTS: [i64; 5] = [0, 1, 2, 3, 10];
const MAX_ITEMS: usize = 200_000;
/// How many answers a wrong source might have given instead (a hand-built guess of the size of the space of answers).
const ANSWER_SPACE: u64 = 100;
/// A word in this many lessons is part of the frame or the kind of a question, not the thing asked about.
const COMMON_WORD: u32 = 10;
/// The most bits a member of a fitted family is charged over the cheapest member (see `search_fitted`).
const FAMILY_RIVAL_BITS: u64 = 3;
/// Evaluations each stage of the search for a recurrence may cost per bit of what the kind's table would cost: the most a rule can save is the table,
/// so it bounds what is worth spending (hand-built).
const EFFORT_PER_BIT: u64 = 10_000;
/// The channel the lessons arrive on.
const TEACHING: &str = "teaching";
/// How many times the lessons count as already confirmed before anything has checked them (hand-built).
const LESSONS_PRIOR: u64 = 1 << (RISK_BITS + 1);
/// A probe value for a program that says nothing there.
const UNDEFINED: i64 = i64::MIN;

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
    /// How many words were on offer the last time the group was tried for a composition, and how many earned words the last blind search had.
    words_tried: usize,
    enumerated_with: usize,
    /// The size of the group, doubled, when a recurrence was last looked for (it is tried again when the group has doubled).
    recurrence_tried: usize,
    /// Counted evaluations spent on this kind until it was first explained, and its place in the order the kinds were explained.
    spent: u64,
    explained_at: Option<usize>,
    /// What is known of the kind's values (numeric, by point), shared with any recurrence that is defined in terms of them.
    table: Rc<std::cell::RefCell<BTreeMap<Vec<i64>, i64>>>,
    /// What each channel said about each point, the latest claim of a channel winning.
    claims: BTreeMap<Vec<i64>, BTreeMap<String, String>>,
    /// Points where the told answer disagreed with the program believed at the time: not believed, not forgotten.
    disputed: std::collections::BTreeSet<Vec<i64>>,
    /// This kind's own record of guesses made outside its region: (right, made).
    record: (u32, u32),
    /// The same for answers worked out by walking a chain of reductions.
    chain_record: (u32, u32),
    /// Whether any answer it has been given was negative. A negative result for a kind that never showed one is not stated.
    negative: bool,
    /// The answers of this kind were number words: a number it works out is spoken as the word for it, or not at all.
    speaks: Option<usize>,
    /// The forms the numbers of its lessons were written in (digits or a family of number words), one entry per argument, as the lessons had them.
    forms: std::collections::BTreeSet<Vec<u8>>,
}

struct Said {
    answer: Option<String>,
    /// Distance bucket of a guess made outside the taught region, and the group that made it.
    bucket: Option<usize>,
    group: Option<usize>,
    /// The kind whose chain of reductions worked this answer out.
    chain_of: Option<usize>,
}

/// Work done, for the report at the end.
#[derive(Default, Debug, Clone, Copy)]
pub struct Stats {
    pub searches: u64,
    pub evaluations: u64,
    pub exhausted: u64,
    pub narrowed: u64,
    pub words_earned: u64,
    /// Evaluations spent by compositions that found an explanation, and how many found one.
    pub composed_evaluations: u64,
    pub composed: u64,
}

/// The ways of asking that lessons used, for any kind: each arrangement of words and numbers whole, and its three parts. The core runs from the first
/// number or kind word to the last; what comes before it is the prefix and what comes after it the suffix. A new arrangement is known if its
/// core was seen in some lesson, and its prefix and suffix are ways of starting and ending that were seen, or those with words left out ("what is"
/// before, "equals" after, the same core in between; "which" for "which is").
#[derive(Default)]
struct Arrangements {
    whole: std::collections::BTreeSet<Vec<String>>,
    cores: std::collections::BTreeSet<Vec<String>>,
    prefixes: std::collections::BTreeSet<Vec<String>>,
    suffixes: std::collections::BTreeSet<Vec<String>>,
}

impl Arrangements {
    fn parts(a: &[String]) -> Option<(Vec<String>, Vec<String>, Vec<String>)> {
        let marked = |w: &String| w == "K" || w == "#";
        let (first, last) = (a.iter().position(marked)?, a.iter().rposition(marked)?);
        Some((a[..first].to_vec(), a[first..=last].to_vec(), a[last + 1..].to_vec()))
    }

    fn add(&mut self, a: Vec<String>) {
        if let Some((p, c, s)) = Self::parts(&a) {
            self.prefixes.insert(p);
            self.cores.insert(c);
            self.suffixes.insert(s);
        }
        self.whole.insert(a);
    }

    fn known(&self, a: &[String]) -> bool {
        // (a part may drop words of a way of asking that was seen: "which is" before the core, "which" before it)
        let cut = |part: &Vec<String>, seen: &std::collections::BTreeSet<Vec<String>>| {
            seen.iter().any(|whole| {
                let mut rest = whole.iter();
                part.iter().all(|w| rest.any(|x| x == w))
            })
        };
        self.whole.contains(a) || Self::parts(a).is_some_and(|(p, c, s)| self.cores.contains(&c) && cut(&p, &self.prefixes) && cut(&s, &self.suffixes))
    }
}

pub struct Novice {
    lessons: Vec<(Parsed, String)>,
    /// The lessons as they were said (question, answer), kept until the words that are numbers have been looked for.
    raw: Vec<(String, String)>,
    /// Whether the answer of each lesson was a number word that was turned into a number.
    lesson_spoke: Vec<Option<usize>>,
    /// The words that are numbers (learned: see `numerals`), and the word for each number the lessons have a word for.
    numerals: Lexicon,
    /// The arrangements (words in order, numbers as "#", the kind's own word as "K") that some lesson was taught in, for any kind.
    shapes: Arrangements,
    df: BTreeMap<String, u32>,
    groups: Vec<Group>,
    by_key: BTreeMap<String, usize>,
    said: BTreeMap<String, Said>,
    /// Guesses made outside the taught region, by distance bucket: (right, made).
    record: [(u32, u32); 8],
    /// Words that earned their place.
    words: Vec<Rc<Lib>>,
    explained: usize,
    why: [u32; 8],
    /// Per channel: how often it agreed with a confident program, and how often one could be asked.
    reliability: BTreeMap<String, (u32, u32)>,
    /// How often answers worked out by a chain of reductions have been right, over all kinds.
    chain_record: (u32, u32),
    /// How often the search for a recurrence has found one.
    recurrence_paid: Paid,
    /// The dear stages of the recurrence search (larger steps, two earlier values), which only run when the first stage found nothing.
    dear_paid: Paid,
    /// Questions with no numbers (lookups, not instances of a kind): the answers each was ever given in exactly those words.
    exact: BTreeMap<String, std::collections::BTreeSet<String>>,
    /// The words (in order) that a lesson or an unlabeled teacher has answered.
    trusted_lookups: std::collections::BTreeSet<String>,
    /// The same, by the words of the question taken as a set: every order it was ever asked in, and every answer any of them was given.
    bags: BTreeMap<String, (std::collections::BTreeSet<String>, std::collections::BTreeSet<String>)>,
    /// Every lesson or told answer to a question without numbers (the words of the question, the answer), and what was learned from them about frames.
    lookups: Vec<(Vec<String>, String)>,
    frames: Option<Frames>,
    /// How many lookups the frames were built from.
    frames_from: usize,
    /// The same for the search with the numbers a kind's shape announces.
    shape_paid: Paid,
    pub stats: Stats,
}

impl Default for Novice {
    fn default() -> Self {
        Novice { lessons: Vec::new(), raw: Vec::new(), lesson_spoke: Vec::new(), numerals: Lexicon::default(), shapes: Arrangements::default(), df: BTreeMap::new(), groups: Vec::new(), by_key: BTreeMap::new(), said: BTreeMap::new(), record: [(0, 0); 8], words: Vec::new(), explained: 0, why: [0; 8], reliability: BTreeMap::new(), chain_record: (0, 0), recurrence_paid: Paid::default(), dear_paid: Paid::default(), exact: BTreeMap::new(), trusted_lookups: Default::default(), bags: BTreeMap::new(), lookups: Vec::new(), frames: None, frames_from: 0, shape_paid: Paid::default(), stats: Stats::default() }
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

/// How often a kind of search has found something, over the kinds it has been tried on. It sets how much effort the next kind is given: a search
/// that has paid off keeps its full allowance, one that keeps failing is given less and less (never none), so that experience makes failure cheaper.
#[derive(Clone, Copy, Debug, Default)]
struct Paid {
    tried: u32,
    won: u32,
}

impl Paid {
    /// The share of the full allowance, in thousandths: twice the estimated chance of success, kept between a tenth and one and a half.
    fn share(&self) -> u64 {
        let chance = 1000 * (u64::from(self.won) + 1) / (u64::from(self.tried) + 2);
        (2 * chance).clamp(100, 1500)
    }
}

/// The words of a question as a set, in alphabetical order, one string.
fn bag_of(p: &Parsed) -> String {
    let mut words = p.seq.clone();
    words.sort();
    words.join(" ")
}

/// Bits to write down a kind's answers, one after the other, with a code that learns how often each answer comes: the first time an answer
/// appears is dear, a repeat is cheap, so answers that are concentrated in a few values cost little. This is what the answers cost if the kind
/// has no rule; a rule must beat it. (`ANSWER_SPACE` is the number of different answers the code allows for before it has seen any.)
fn table_cost(answers: &[i64]) -> Cost {
    let mut seen: BTreeMap<i64, u64> = BTreeMap::new();
    let mut cost: Cost = 0;
    for (n, a) in answers.iter().enumerate() {
        let count = seen.get(a).copied().unwrap_or(0);
        let (num, den) = (2 * count + 1, 2 * n as u64 + ANSWER_SPACE);
        cost += bits::log2(den).saturating_sub(bits::log2(num));
        *seen.entry(*a).or_insert(0) += 1;
    }
    cost
}

/// The numbers that the shape of a one-argument kind announces, if it has a regular shape: the slopes it climbs with, where the slope changes (a
/// clamp, a threshold, a wrap) and the gaps between those places (a period), and its largest answer. Data that changes its slope at about every
/// step (arbitrary codes) has no shape and gives nothing.
fn shape_constants(inputs: &[Vec<i64>], target: &[i64]) -> Option<Vec<i64>> {
    let mut pts: Vec<(i64, i64)> = inputs.iter().map(|p| p[0]).zip(target.iter().copied()).collect();
    pts.sort_unstable();
    let slopes: Vec<(i64, i64)> = pts.windows(2).filter(|w| w[1].0 == w[0].0 + 1).map(|w| (w[0].0, w[1].1 - w[0].1)).collect();
    if slopes.len() < 6 {
        return None;
    }
    let changes: Vec<i64> = slopes.windows(2).filter(|w| w[1].0 == w[0].0 + 1 && w[1].1 != w[0].1).map(|w| w[1].0).collect();
    if changes.len() * 3 > slopes.len() {
        return None;
    }
    let mut found: std::collections::BTreeSet<i64> = Default::default();
    for &(_, d) in &slopes {
        found.insert(d.abs());
    }
    for (i, &x) in changes.iter().enumerate() {
        found.insert(x);
        found.insert(x + 1);
        if i > 0 {
            found.insert(x - changes[i - 1]);
        }
    }
    // Each straight piece has a constant term (where its line would cross the axis), which a rule written with those pieces needs.
    let at: std::collections::BTreeMap<i64, i64> = pts.iter().copied().collect();
    let mut start = 0;
    while start < slopes.len() {
        let mut end = start;
        while end + 1 < slopes.len() && slopes[end + 1].0 == slopes[end].0 + 1 && slopes[end + 1].1 == slopes[start].1 {
            end += 1;
        }
        if end > start {
            let (x, d) = slopes[start];
            if let Some(&t) = at.get(&x) {
                found.insert((t - d * x).abs());
            }
        }
        start = end + 1;
    }
    let top = target.iter().copied().max().unwrap_or(0);
    found.extend([top, top + 1]);
    let out: Vec<i64> = found.into_iter().filter(|c| (2..=64).contains(c) && !BASE_CONSTS.contains(c)).take(10).collect();
    (!out.is_empty()).then_some(out)
}

/// Whether the expression mentions every variable from `from` up to (not including) `to`: for a step, each of the values at the nearer points.
fn uses_all(e: &Expr, from: usize, to: usize) -> bool {
    (from..to).all(|v| uses_from(e, v) && mentions_variable(e, v))
}

fn mentions_variable(e: &Expr, v: usize) -> bool {
    match e {
        Expr::Var(i) => *i == v,
        Expr::Const(_) | Expr::Fix(_) => false,
        Expr::Bin(_, a, b) => mentions_variable(a, v) || mentions_variable(b, v),
        Expr::Call(_, xs) => xs.iter().any(|x| mentions_variable(x, v)),
    }
}

/// Whether the expression mentions any variable numbered `from` or higher (for a step: any of the values at the nearer points).
fn uses_from(e: &Expr, from: usize) -> bool {
    match e {
        Expr::Var(v) => *v >= from,
        Expr::Const(_) | Expr::Fix(_) => false,
        Expr::Bin(_, a, b) => uses_from(a, from) || uses_from(b, from),
        Expr::Call(_, xs) => xs.iter().any(|x| uses_from(x, from)),
    }
}

/// The smallest box that holds all the points.
fn bounding_box(inputs: &[Vec<i64>], arity: usize) -> (Vec<i64>, Vec<i64>) {
    let (mut lo, mut hi) = (inputs[0].clone(), inputs[0].clone());
    for p in inputs {
        for i in 0..arity {
            lo[i] = lo[i].min(p[i]);
            hi[i] = hi[i].max(p[i]);
        }
    }
    (lo, hi)
}

/// Every point of the box from the lowest taught number to just beyond the highest, at most 40 numbers along an axis.
fn dense_probes(lo: &[i64], hi: &[i64]) -> Vec<Vec<i64>> {
    let mut out: Vec<Vec<i64>> = vec![Vec::new()];
    for i in 0..lo.len() {
        let axis: Vec<i64> = (lo[i]..=(2 * hi[i] + 5).min(lo[i] + 39)).collect();
        out = out.iter().flat_map(|prefix| axis.iter().map(move |&x| prefix.iter().copied().chain([x]).collect())).collect();
    }
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

/// `h` with its one variable replaced by `with`.
fn substitute(h: &Expr, with: &Expr) -> Expr {
    match h {
        Expr::Var(_) => with.clone(),
        Expr::Const(c) => Expr::Const(*c),
        Expr::Bin(op, a, b) => Expr::Bin(*op, Box::new(substitute(a, with)), Box::new(substitute(b, with))),
        Expr::Call(lib, xs) => Expr::Call(lib.clone(), xs.iter().map(|x| substitute(x, with)).collect()),
        Expr::Fix(_) => h.clone(),
    }
}

fn mentions(e: &Expr, word: &str) -> bool {
    match e {
        Expr::Call(lib, xs) => lib.name == word || xs.iter().any(|x| mentions(x, word)),
        Expr::Fix(_) => false,
        Expr::Bin(_, a, b) => mentions(a, word) || mentions(b, word),
        _ => false,
    }
}

fn bucket(dist: i64) -> usize {
    ((64 - (dist.max(1) as u64).leading_zeros()) as usize).min(7)
}

impl Novice {
    /// How many kinds of question it has found, and how many of them it explains with a program.
    pub fn kinds(&self) -> (usize, usize) {
        (self.groups.len(), self.groups.iter().filter(|g| g.fit.is_some()).count())
    }

    /// The group a question belongs to: the rarest of its words that names a group.
    fn group_of(&self, p: &Parsed) -> Option<usize> {
        p.words.iter().filter_map(|w| self.by_key.get(w).map(|&g| (std::cmp::Reverse(self.groups[g].points.len()), w.clone(), g))).min().map(|(_, _, g)| g)
    }

    /// The arrangement of a question with the word that names its kind written "K".
    fn arrangement(p: &Parsed, key: &str) -> Vec<String> {
        p.shape.iter().map(|w| if w == key { "K".to_string() } else { w.clone() }).collect()
    }

    fn add_point(&mut self, g: usize, nums: &[i64], answer: &str) {
        self.groups[g].negative |= answer.starts_with('-');
        *self.groups[g].points.entry(nums.to_vec()).or_default().entry(answer.to_string()).or_insert(0) += 1;
    }

    /// Run `work` for group `g`, charging the evaluations it spends to the group and noting when the group first has an explanation.
    fn charge<R>(&mut self, g: usize, work: impl FnOnce(&mut Self) -> R) -> R {
        let before = self.stats.evaluations;
        let result = work(self);
        self.groups[g].spent += self.stats.evaluations - before;
        if std::env::var_os("NOVICE_TRACE").is_some() && self.stats.evaluations - before > 100_000 {
            eprintln!("charge {:<14} {:>10} evaluations (fit: {})", self.groups[g].key, self.stats.evaluations - before, self.groups[g].fit.is_some());
        }
        if self.groups[g].fit.is_some() && self.groups[g].explained_at.is_none() {
            self.explained += 1;
            self.groups[g].explained_at = Some(self.explained);
        }
        result
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
        self.search_group_in(g, consts, words, max_size, label, None)
    }

    /// [`Self::search_group`], with the answers taken modulo a constant if one is given.
    fn search_group_in(&mut self, g: usize, consts: &[i64], words: &[Rc<Lib>], max_size: usize, label: &str, modulus: Option<i64>) -> bool {
        let Some((inputs, target)) = self.inputs_and_targets(g) else { return false };
        let arity = self.groups[g].arity;
        let (mut lo, mut hi) = (inputs[0].clone(), inputs[0].clone());
        for p in &inputs {
            for i in 0..arity {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
        let table: Cost = table_cost(&target);
        let probe = probes(&lo, &hi);
        let out = program::search_with_rivals_in(&Task { inputs: &inputs, target: &target, probes: &probe, arity, consts, words, max_size, rival_size: 2, max_items: MAX_ITEMS }, modulus);
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

    /// Look for a program that explains the group's points. `refuted` are programs the latest told answer contradicted. The cheapest guess is that
    /// the rule changed in one place, so first try editing them, with the numbers they got wrong offered as constants; then search.
    fn refit(&mut self, g: usize, refuted: &[Expr]) {
        self.groups[g].searched_at = self.groups[g].points.len();
        self.groups[g].fit = None;
        let Some((inputs, target)) = self.inputs_and_targets(g) else { return };
        let words = self.words.clone();
        if !refuted.is_empty() && self.repair(g, refuted, &inputs, &target, &words) {
            return;
        }
        // The first blind search uses the starting operations only: every word on the alphabet makes it dearer for every kind that comes after,
        // whether or not the kind has anything to do with the word. Words are used by composition and by editing a known program.
        if self.search_group(g, &BASE_CONSTS, &[], 7, "plain") || self.search_data_constants(g) || self.search_modulo(g) || self.search_shape(g) || self.search_fitted(g) || self.search_affine(g) {
            return;
        }
    }

    /// The region where the favorite has been checked: the bounding box of the points it gets right.
    fn verified_region(&mut self, g: usize) {
        let Some(obs) = self.observations(g) else { return };
        let Some(fit) = &mut self.groups[g].fit else { return };
        let best = &fit.found[0].expr;
        let right: Vec<&Vec<i64>> = obs.iter().filter(|(x, t)| best.eval(x) == Some(*t)).map(|(x, _)| x).collect();
        if let Some(first) = right.first() {
            let (mut lo, mut hi) = ((*first).clone(), (*first).clone());
            for x in &right {
                for i in 0..lo.len() {
                    lo[i] = lo[i].min(x[i]);
                    hi[i] = hi[i].max(x[i]);
                }
            }
            fit.lo = lo;
            fit.hi = hi;
        }
    }

    /// The sources that are one speaker: the lessons and the exam's main teacher are the same teacher, so what they say is not independent.
    fn family(channel: &str) -> &str {
        match channel {
            TEACHING | "told" | "teacher" => "teacher",
            other => other,
        }
    }

    /// Bits a channel pays for agreeing with the truth (`-log2` of its reliability), or for being wrong in one particular way
    /// (`-log2` of `(1 - reliability) / (ANSWER_SPACE - 1)`). Reliability is the Laplace estimate from the times it could be checked.
    fn channel_cost(&self, channel: &str, right: bool) -> Cost {
        // Before it has been checked, a channel is as reliable as the lessons were if it is the same teacher, and a coin toss otherwise; the
        // prior counts for two checks.
        let (prior_num, prior_den) = match (Self::family(channel) == "teacher" && channel != TEACHING, self.reliability.get(TEACHING)) {
            (true, Some(&(agree, asked))) => (u64::from(agree) + LESSONS_PRIOR, u64::from(asked) + LESSONS_PRIOR + 1),
            (true, None) => (LESSONS_PRIOR, LESSONS_PRIOR + 1),
            _ => (1, 2),
        };
        let (agree, asked) = self.reliability.get(channel).copied().unwrap_or((0, 0));
        // The lessons are what the system exists to trust: before anything has checked them they count as having been confirmed 2^(RISK_BITS + 1)
        // times, so one taught fact is a fact; checking can only lower that.
        let (num, den) = if channel == TEACHING {
            (u64::from(agree) + LESSONS_PRIOR, u64::from(asked) + LESSONS_PRIOR + 1)
        } else {
            (u64::from(agree) * prior_den + 2 * prior_num, (u64::from(asked) + 2) * prior_den)
        };
        if right {
            bits::surprise(num, den)
        } else {
            bits::log2(den * (ANSWER_SPACE - 1)).saturating_sub(bits::log2(den - num))
        }
    }

    /// What the channels that spoke about a point jointly make of it: the most believed answer, its share of the belief, and the whole (fixed
    /// point). An answer nobody gave carries belief too: any of the other answers in the space could be the truth if all the channels erred.
    fn believe(&self, g: usize, nums: &[i64]) -> Option<(String, u64, u64)> {
        let claims = self.groups[g].claims.get(nums)?;
        let mut candidates: Vec<&String> = claims.values().collect();
        candidates.sort();
        candidates.dedup();
        let cost = |answer: Option<&String>| -> Cost { claims.iter().map(|(c, said)| self.channel_cost(c, Some(said) == answer)).sum() };
        let costs: Vec<Cost> = candidates.iter().map(|a| cost(Some(*a))).collect();
        let other = cost(None);
        let floor = costs.iter().copied().chain([other]).min()?;
        let other_count = ANSWER_SPACE.saturating_sub(candidates.len() as u64).max(1);
        let weights: Vec<u64> = costs.iter().map(|c| bits::exp2_neg(c - floor)).collect();
        let total: u64 = weights.iter().sum::<u64>() + other_count * bits::exp2_neg(other - floor);
        let (best, top) = weights.iter().enumerate().max_by(|a, b| a.1.cmp(b.1).then(candidates[b.0].cmp(candidates[a.0])))?;
        Some((candidates[best].clone(), *top, total))
    }

    /// The program's answer at a point, if the point is inside the region it has been checked in and all but 2^-RISK_BITS of the belief is on it.
    fn confident_value(&self, g: usize, nums: &[i64]) -> Option<i64> {
        let fit = self.groups[g].fit.as_ref()?;
        if !nums.iter().enumerate().all(|(i, &n)| fit.lo[i] <= n && n <= fit.hi[i]) {
            return None;
        }
        let (value, top, total) = program::weigh(&fit.found, nums)?;
        (u128::from(top) << RISK_BITS >= u128::from(total) * ((1u128 << RISK_BITS) - 1)).then_some(value)
    }

    /// How reliable the lessons were, as far as they can be checked: the points of the kinds that have a program.
    fn score_teaching(&mut self) {
        let mut tally = (0u32, 0u32);
        for g in &self.groups {
            let Some(fit) = &g.fit else { continue };
            for (nums, said) in &g.claims {
                let Some(answer) = said.get(TEACHING).and_then(|a| a.parse::<i64>().ok()) else { continue };
                tally.1 += 1;
                tally.0 += u32::from(fit.found[0].expr.eval(nums) == Some(answer));
            }
        }
        self.reliability.insert(TEACHING.to_string(), tally);
    }

    /// Every distinct (point, numeric answer) the group has been given.
    fn observations(&self, g: usize) -> Option<Vec<(Vec<i64>, i64)>> {
        let grp = &self.groups[g];
        let mut out = Vec::new();
        for (nums, answers) in &grp.points {
            if nums.len() != grp.arity {
                return None;
            }
            for a in answers.keys() {
                out.push((nums.clone(), a.parse::<i64>().ok()?));
            }
        }
        (!out.is_empty()).then_some(out)
    }

    /// A told answer disagreed with the favorite. Look at the programs one edit away from the believed ones, with the numbers their mistakes name
    /// as constants, and let each pay for the points it still gets wrong. Anything that comes out cheaper than the favorite takes its place.
    fn repair_priced(&mut self, g: usize) {
        let Some(obs) = self.observations(g) else { return };
        let Some(fit) = &self.groups[g].fit else { return };
        let arity = self.groups[g].arity;
        let words = self.words.clone();
        let believed: Vec<Expr> = fit.found.iter().take(4).map(|f| f.expr.clone()).collect();
        let (lo, hi) = (fit.lo.clone(), fit.hi.clone());
        let best_cost = fit.found[0].cost;
        let inputs: Vec<Vec<i64>> = obs.iter().map(|o| o.0.clone()).collect();
        let target: Vec<i64> = obs.iter().map(|o| o.1).collect();
        let points = self.groups[g].points.len() as u64;
        let price: Vec<Cost> = target.iter().map(|&t| bits::log2(points + 1) + bits::uint(t.unsigned_abs())).collect();
        let mut consts: Vec<i64> = BASE_CONSTS.to_vec();
        for e in &believed {
            for (x, &t) in inputs.iter().zip(&target) {
                if let Some(v) = e.eval(x) {
                    if v != t {
                        consts.extend([(v - t).abs(), v, t]);
                    }
                }
            }
        }
        consts.retain(|c| (0..=200).contains(c));
        consts.sort_unstable();
        consts.dedup();
        let probe = probes(&lo, &hi);
        let task = Task { inputs: &inputs, target: &target, probes: &probe, arity, consts: &consts, words: &words, max_size: 0, rival_size: 0, max_items: MAX_ITEMS };
        let mut more: Vec<Found> = Vec::new();
        for e in &believed {
            let (found, effort) = program::rivals_priced(e, &task, Some((&price, best_cost + 12 * bits::BIT)));
            self.stats.evaluations += effort.evaluations;
            more.extend(found);
        }
        self.stats.searches += 1;
        if more.is_empty() {
            return;
        }
        let Some(fit) = &mut self.groups[g].fit else { return };
        for f in more {
            match fit.found.iter_mut().find(|k| k.probe_values == f.probe_values) {
                Some(k) => k.cost = k.cost.min(f.cost),
                None => fit.found.push(f),
            }
        }
        fit.found.sort_by(|x, y| x.cost.cmp(&y.cost).then_with(|| x.expr.size().cmp(&y.expr.size())).then_with(|| x.expr.to_string().cmp(&y.expr.to_string())));
        let best = fit.found[0].cost;
        fit.found.retain(|f| f.cost <= best + 32 * bits::BIT);
        fit.found.truncate(64);
    }

    /// Explain a kind by a recurrence over its own values: `f(p) = e(p, f(g(p)))`, or over two earlier values, `e(p, f(g1(p)), f(g2(p)))`. The
    /// reductions `g` are proposed from everything a few nodes can make, keeping those that bring a good share of the points down to points whose
    /// value is known; then `e` is searched on the points that have such values. The points the recurrence does not account for are paid for one
    /// by one, so it is kept only if that is cheaper than the whole table by the error budget and it passes the held-out check.
    fn recurrence(&mut self, g: usize) -> bool {
        let arity = self.groups[g].arity;
        if arity == 0 || arity > 2 {
            return false;
        }
        let Some((inputs, target)) = self.inputs_and_targets(g) else { return false };
        if inputs.len() < 8 {
            return false;
        }
        let known: BTreeMap<Vec<i64>, i64> = inputs.iter().cloned().zip(target.iter().copied()).collect();
        let whole_table: Cost = table_cost(&target);
        let consts: Vec<i64> = (0..=8).chain([10]).collect();
        let (pool, effort) = program::pool(&inputs, arity, &consts, if arity == 1 { 5 } else { 3 }, 30_000);
        self.stats.evaluations += effort.evaluations;
        let usable: Vec<usize> = (0..pool.len()).filter(|&i| pool[i].2.iter().all(|&v| v >= 0)).collect();
        struct Reduction {
            parts: Vec<usize>,
            cover: usize,
            cost: Cost,
        }
        let mut reductions: Vec<Reduction> = Vec::new();
        let at = |parts: &[usize], k: usize| -> Vec<i64> { parts.iter().map(|&i| pool[i].2[k]).collect() };
        if arity == 1 {
            for &i in &usable {
                let down = inputs.iter().enumerate().filter(|(k, p)| p[0] > 0 && pool[i].2[*k] < p[0]).count();
                let positive = inputs.iter().filter(|p| p[0] > 0).count();
                if down * 10 < positive * 9 {
                    continue;
                }
                let cover = (0..inputs.len()).filter(|&k| known.contains_key(&at(&[i], k))).count();
                if cover >= 4 {
                    reductions.push(Reduction { parts: vec![i], cover, cost: pool[i].1 });
                }
            }
        } else {
            for &i in &usable {
                for &j in &usable {
                    let parts = [i, j];
                    let moved = (0..inputs.len()).filter(|&k| at(&parts, k) != inputs[k]).count();
                    if moved < inputs.len() {
                        continue;
                    }
                    let cover = (0..inputs.len()).filter(|&k| known.contains_key(&at(&parts, k))).count();
                    if cover >= 4 {
                        reductions.push(Reduction { parts: parts.to_vec(), cover, cost: pool[i].1 + pool[j].1 });
                    }
                }
            }
        }
        // The cheapest reductions first, among those that bring a good share of the points down to a point whose value is known.
        let enough = (inputs.len() * 2 / 5).clamp(4, 10);
        reductions.retain(|r| r.cover >= enough);
        reductions.sort_by(|a, b| a.cost.cmp(&b.cost).then(b.cover.cmp(&a.cover)).then(a.parts.cmp(&b.parts)));
        self.stats.searches += 1;

        // Fit the step for a choice of one or two reductions.
        let started = std::cell::Cell::new(self.stats.evaluations);
        // The first stage (small steps, one earlier value) is where recurrences are almost always found and it is cheap: it gets its full allowance.
        // The dearer stages (larger steps, two earlier values) get an allowance scaled by how often a recurrence has been found so far.
        let full = (bits::as_bits(whole_table) as u64).max(1) * EFFORT_PER_BIT;
        let scaled = full * self.dear_paid.share() / 1000;
        let budget_now = std::cell::Cell::new(full);
        let fit = |this: &mut Self, chosen: &[&Reduction], max_size: usize, slack_cap: usize| -> Option<(Cost, Expr)> {
            if this.stats.evaluations - started.get() > budget_now.get() {
                return None;
            }
            let (mut rows, mut ts, mut row_of) = (Vec::new(), Vec::new(), Vec::new());
            'points: for k in 0..inputs.len() {
                let mut row = inputs[k].clone();
                for red in chosen {
                    let gp = at(&red.parts, k);
                    if gp == inputs[k] {
                        continue 'points;
                    }
                    match known.get(&gp) {
                        Some(&r) => row.push(r),
                        None => continue 'points,
                    }
                }
                rows.push(row);
                ts.push(target[k]);
                row_of.push(k);
            }
            if rows.len() < 4 {
                return None;
            }
            let step = this.find_step(&rows, &ts, arity, chosen.len(), max_size, slack_cap)?;
            // A point is explained if the step gives its answer from the values at the nearer points; the others are base cases and are paid for.
            let mut explained = vec![false; inputs.len()];
            for (i, &k) in row_of.iter().enumerate() {
                explained[k] = step.eval(&rows[i]) == Some(ts[i]);
            }
            let base_cost: Cost = table_cost(&(0..inputs.len()).filter(|&k| !explained[k]).map(|k| target[k]).collect::<Vec<i64>>());
            let steps: Vec<Vec<Expr>> = chosen.iter().map(|red| red.parts.iter().map(|&i| pool[i].0.clone()).collect()).collect();
            let whole = Expr::Fix(Rc::new(program::FixDef { arity, steps, e: step, table: this.groups[g].table.clone(), base_cost }));
            Some((program::cost_of(&whole, arity, 0), whole))
        };

        // One earlier value first; the larger steps only if the small ones gave nothing; two earlier values only if one is not enough.
        let mut accepted: Vec<(Cost, Expr)> = Vec::new();
        for (max_size, how_many) in [(5usize, 24usize), (7, 6)] {
            started.set(self.stats.evaluations);
            budget_now.set(if max_size == 5 { full } else { scaled });
            if max_size != 5 {
                self.dear_paid.tried += 1;
            }
            let stage_start = self.stats.evaluations;
            let mut candidates: Vec<(Cost, Expr)> = Vec::new();
            for (i, red) in reductions.iter().take(how_many).enumerate() {
                // Base cases at the start are allowed only for the cheapest few: that search costs several times an exact one.
                if let Some(c) = fit(self, &[red], max_size, if i < 6 { 1 } else { 0 }) {
                    candidates.push(c);
                }
            }
            if std::env::var_os("NOVICE_TRACE").is_some() {
                eprintln!("stage {:<12} singles size {max_size}: {} evaluations, {} candidates", self.groups[g].key, self.stats.evaluations - stage_start, candidates.len());
            }
            accepted = self.accept_recurrences(g, candidates, whole_table, &known, arity);
            if !accepted.is_empty() {
                self.dear_paid.won += u32::from(max_size != 5);
                break;
            }
        }
        if accepted.is_empty() {
            // For a pair, only reductions that keep the points apart: one that sends almost every point to the same few (a constant, a
            // remainder by two) is a base case, not a step towards the answer.
            let distinct = |r: &Reduction| (0..inputs.len()).map(|k| at(&r.parts, k)).collect::<std::collections::BTreeSet<_>>().len();
            let few: Vec<&Reduction> = reductions.iter().filter(|r| distinct(r) * 2 >= inputs.len()).take(8).collect();
            if std::env::var_os("NOVICE_TRACE").is_some() {
                eprintln!("recurrence {:<14} pairs from {} reductions of {}: {}", self.groups[g].key, few.len(), reductions.len(), few.iter().map(|r| r.parts.iter().map(|&i| pool[i].0.to_string()).collect::<Vec<_>>().join(",")).collect::<Vec<_>>().join(" | "));
            }
            let mut candidates: Vec<(Cost, Expr)> = Vec::new();
            started.set(self.stats.evaluations);
            budget_now.set(scaled);
            let pairs_start = self.stats.evaluations;
            let mut tried = 0;
            for i in 0..few.len() {
                for j in i + 1..few.len() {
                    if let Some(c) = fit(self, &[few[i], few[j]], 5, if tried < 10 { 1 } else { 0 }) {
                        candidates.push(c);
                    }
                    tried += 1;
                }
            }
            if std::env::var_os("NOVICE_TRACE").is_some() {
                eprintln!("stage {:<12} pairs: {} evaluations, {} candidates", self.groups[g].key, self.stats.evaluations - pairs_start, candidates.len());
            }
            accepted = self.accept_recurrences(g, candidates, whole_table, &known, arity);
            self.dear_paid.won += u32::from(!accepted.is_empty());
        }
        // They are rivals: each is true of the table, and which one reaches a point depends on the point.
        let mut accepted = accepted.into_iter();
        self.recurrence_paid.tried += 1;
        let Some((cost, whole)) = accepted.next() else { return false };
        self.recurrence_paid.won += 1;
        let rivals: Vec<Expr> = accepted.map(|a| a.1).collect();
        if std::env::var_os("NOVICE_TRACE").is_some() {
            eprintln!("recurrence {:<14} points {:>3} -> {whole}   saves {:.1} bits ({:.1} against a table of {:.1})", self.groups[g].key, inputs.len(), bits::as_bits(whole_table) - bits::as_bits(cost), bits::as_bits(cost), bits::as_bits(whole_table));
        }
        // The table is shared: what is known so far goes in, and what is learned later joins it.
        for (p, t) in &known {
            self.groups[g].table.borrow_mut().entry(p.clone()).or_insert(*t);
        }
        self.finish_fit(g, whole, rivals, &inputs, &target, &[])
    }

    /// The candidates, cheapest first, that beat the table by the error budget and predict the points held out; at most three.
    fn accept_recurrences(&mut self, g: usize, mut candidates: Vec<(Cost, Expr)>, whole_table: Cost, known: &BTreeMap<Vec<i64>, i64>, arity: usize) -> Vec<(Cost, Expr)> {
        candidates.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.to_string().cmp(&b.1.to_string())));
        candidates.dedup_by(|a, b| a.1.to_string() == b.1.to_string());
        let mut accepted: Vec<(Cost, Expr)> = Vec::new();
        for (cost, whole) in candidates.into_iter().take(6) {
            if accepted.len() == 3 {
                break;
            }
            // Beating the table by a hair is what a chance fit does too: the saving has to clear the error budget. A rule also answers what a table
            // cannot, the points beyond it, each of which costs a table the full price of an answer nobody can guess; half as many questions as
            // there are points in the table are assumed to come, and the rule is credited for them.
            let future = (known.len() as u64 / 2) * bits::log2(ANSWER_SPACE);
            if cost + u64::from(RISK_BITS) * bits::BIT >= whole_table + future {
                if std::env::var_os("NOVICE_TRACE").is_some() {
                    eprintln!("recurrence {:<14} {whole}: {:.1} bits against a table of {:.1}", self.groups[g].key, bits::as_bits(cost), bits::as_bits(whole_table));
                }
                continue;
            }
            // And a step fitted on the rows it explains says nothing about points it has not seen: it is fitted again on part of the table and
            // must predict the rest.
            let Expr::Fix(fix) = &whole else { continue };
            if !self.holds_out(fix, known, arity) {
                if std::env::var_os("NOVICE_TRACE").is_some() {
                    eprintln!("recurrence {:<14} {whole}: does not predict the points held out", self.groups[g].key);
                }
                continue;
            }
            accepted.push((cost, whole));
        }
        accepted
    }

    /// A step over `arity + k` variables (the arguments, then the values at the `k` nearer points) that fits the rows and uses at least one of
    /// those values. Tried as it stands, and then reduced modulo the number just above the largest answer; exactly first, and only then letting
    /// a few rows go (a rule that holds from some point on has base cases at the start, and those are paid for as such).
    fn find_step(&mut self, rows: &[Vec<i64>], ts: &[i64], arity: usize, k: usize, max_size: usize, slack_cap: usize) -> Option<Expr> {
        let task = Task { inputs: rows, target: ts, probes: &[], arity: arity + k, consts: &BASE_CONSTS, words: &[], max_size, rival_size: 0, max_items: MAX_ITEMS };
        let top = ts.iter().copied().max()?;
        let modular = ts.iter().all(|&t| t >= 0) && top >= 2;
        let allowed = (rows.len() / 6).min(slack_cap);
        for slack in 0..=allowed {
            let out = program::search_with_slack(&task, None, slack);
            self.stats.evaluations += out.effort.evaluations;
            if let Some(f) = out.found.iter().find(|f| uses_all(&f.expr, arity, arity + k)) {
                return Some(f.expr.clone());
            }
            if modular {
                let out = program::search_with_slack(&Task { max_size: max_size.min(5), ..task }, Some(top + 1), slack);
                self.stats.evaluations += out.effort.evaluations;
                if let Some(f) = out.found.iter().find(|f| uses_all(&f.expr, arity, arity + k)) {
                    return Some(f.expr.clone());
                }
            }
        }
        None
    }

    /// Whether a recurrence found on the whole table is also right about points it has not seen. Four times, a different quarter of the points is
    /// hidden; the step is found again from the other three quarters and must give the right value, through the chain of reductions, at the hidden
    /// points the chain reaches. Over the four rounds it must reach at least three of them and be right at every one.
    fn holds_out(&mut self, fix: &Rc<program::FixDef>, known: &BTreeMap<Vec<i64>, i64>, arity: usize) -> bool {
        let fold_of = |p: &Vec<i64>| -> u64 {
            let mut h: u64 = 0x9E37_79B9_7F4A_7C15;
            for &v in p {
                h = (h ^ v as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                h ^= h >> 31;
                h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
                h ^= h >> 29;
            }
            h % 4
        };
        // The points the recurrence does not account for even with the whole table are base cases: they are given, not predicted.
        let base_points: std::collections::BTreeSet<Vec<i64>> = known
            .iter()
            .filter(|(p, &t)| {
                let mut row = (*p).clone();
                for step in &fix.steps {
                    match step.iter().map(|g| g.eval(p)).collect::<Option<Vec<i64>>>().and_then(|next| known.get(&next).copied()) {
                        Some(r) => row.push(r),
                        None => return false,
                    }
                }
                fix.e.eval(&row) != Some(t)
            })
            .map(|(p, _)| p.clone())
            .collect();
        let (mut reached, mut right) = (0usize, 0usize);
        for fold in 0..4 {
            let training: BTreeMap<Vec<i64>, i64> = known.iter().filter(|(p, _)| fold_of(p) != fold).map(|(p, t)| (p.clone(), *t)).collect();
            let (mut rows, mut ts) = (Vec::new(), Vec::new());
            'points: for (p, &t) in &training {
                let mut row = p.clone();
                for step in &fix.steps {
                    let Some(next) = step.iter().map(|g| g.eval(p)).collect::<Option<Vec<i64>>>() else { continue 'points };
                    match training.get(&next) {
                        Some(&r) if &next != p => row.push(r),
                        _ => continue 'points,
                    }
                }
                rows.push(row);
                ts.push(t);
            }
            if rows.len() < 4 {
                continue;
            }
            let Some(step) = self.find_step(&rows, &ts, arity, fix.steps.len(), fix.e.size().max(5), 1) else { return false };
            let again = program::FixDef { arity, steps: fix.steps.clone(), e: step, table: Rc::new(std::cell::RefCell::new(training)), base_cost: 0 };
            for (p, &t) in known {
                if fold_of(p) != fold || base_points.contains(p) {
                    continue;
                }
                // A point this step does not explain even from its true predecessors is a base case for it, not something it failed to predict.
                let mut row = p.clone();
                let mut complete = true;
                for step in &fix.steps {
                    match step.iter().map(|g| g.eval(p)).collect::<Option<Vec<i64>>>().and_then(|next| known.get(&next).copied()) {
                        Some(r) => row.push(r),
                        None => complete = false,
                    }
                }
                if complete && again.e.eval(&row) != Some(t) {
                    continue;
                }
                if let Some(v) = again.apply(p, program::FIX_DEPTH) {
                    reached += 1;
                    right += usize::from(v == t);
                    if v != t && std::env::var_os("NOVICE_TRACE").is_some() {
                        eprintln!("    held out (fold {fold}): at {p:?} chain says {v}, truth {t}; refitted step {}", again.e);
                    }
                }
            }
        }
        if std::env::var_os("NOVICE_TRACE").is_some() {
            eprintln!("    held out over four rounds: reached {reached}, right {right}");
        }
        reached >= 3 && right == reached
    }

    /// A kind with a regular shape (a few places where its slope changes): search with the numbers that shape announces as constants.
    fn search_shape(&mut self, g: usize) -> bool {
        if self.groups[g].arity != 1 {
            return false;
        }
        let Some((inputs, target)) = self.inputs_and_targets(g) else { return false };
        let Some(salient) = shape_constants(&inputs, &target) else { return false };
        let mut consts: Vec<i64> = BASE_CONSTS.to_vec();
        consts.extend(salient);
        consts.sort_unstable();
        consts.dedup();
        self.shape_paid.tried += 1;
        let won = self.search_group(g, &consts, &[], 7, "shape");
        self.shape_paid.won += u32::from(won);
        won
    }

    /// The floor of a line or plane with large constants, and a sum of multiples (or a quadratic of one variable) reduced modulo a constant: families with a few integers that no
    /// enumeration reaches, fitted directly (see `fit`). Every member of the family that gives all the answers is kept, so the ones that disagree
    /// beyond the data stay as rivals.
    fn search_fitted(&mut self, g: usize) -> bool {
        let arity = self.groups[g].arity;
        if arity == 0 || arity > 2 {
            return false;
        }
        let Some((inputs, target)) = self.inputs_and_targets(g) else { return false };
        for (label, fitted) in [("floor", fit::floors(&inputs, &target)), ("modular", fit::modular(&inputs, &target))] {
            if self.adopt_family(g, &inputs, &target, fitted.exprs, fitted.work, label) {
                return true;
            }
        }
        false
    }

    /// Keep a family of programs that all give the answers (see `search_fitted`) as the explanation of the group if the best of them is cheaper than
    /// the table. `work` is what finding them cost.
    fn adopt_family(&mut self, g: usize, inputs: &[Vec<i64>], target: &[i64], exprs: Vec<Expr>, work: u64, label: &str) -> bool {
        let arity = self.groups[g].arity;
        self.stats.searches += 1;
        self.stats.evaluations += work;
        let (lo, hi) = bounding_box(inputs, arity);
        let probe = probes(&lo, &hi);
        // The family's members agree on the data and differ beyond it, and the differences are small: the sparse probes (which tell programs
        // apart for the enumerations) would call two slopes close to each other the same program. So the members are told apart on every
        // point of the box out to just beyond the data, and the ones that give the highest and the lowest value anywhere are always kept.
        let dense = dense_probes(&lo, &hi);
        let mut rows: Vec<(Found, Vec<i64>)> = Vec::new();
        for expr in exprs {
            self.stats.evaluations += dense.len() as u64;
            let values: Option<Vec<i64>> = dense.iter().map(|p| expr.eval(p)).collect();
            let probe_values: Option<Vec<i64>> = probe.iter().map(|p| expr.eval(p)).collect();
            if let (Some(values), Some(probe_values), true) = (values, probe_values, inputs.iter().zip(target).all(|(p, &t)| expr.eval(p) == Some(t))) {
                rows.push((Found { cost: program::cost_of(&expr, arity, 0), expr, probe_values }, values));
            }
        }
        rows.sort_by(|x, y| x.0.cost.cmp(&y.0.cost).then_with(|| x.0.expr.size().cmp(&y.0.expr.size())).then_with(|| x.0.expr.to_string().cmp(&y.0.expr.to_string())));
        let mut reps: Vec<(Found, Vec<i64>)> = Vec::new();
        for row in rows {
            if !reps.iter().any(|k| k.1 == row.1) {
                reps.push(row);
            }
        }
        let mut keep: std::collections::BTreeSet<usize> = (0..reps.len().min(64)).collect();
        for i in 0..dense.len() {
            let value = |r: usize| reps[r].1[i];
            keep.extend((0..reps.len()).min_by_key(|&r| (value(r), r)));
            keep.extend((0..reps.len()).max_by_key(|&r| (value(r), std::cmp::Reverse(r))));
        }
        let mut kept: Vec<Found> = reps.into_iter().enumerate().filter(|(r, _)| keep.contains(r)).map(|(_, (f, _))| f).collect();
        // Members that fit the data equally are not much less believable because their constants are dearer to write: the data does not
        // choose between them, so none is charged more than a few bits over the cheapest, and where they disagree the answer is in doubt.
        if let Some(best) = kept.first().map(|f| f.cost) {
            for f in &mut kept {
                f.cost = f.cost.min(best + FAMILY_RIVAL_BITS * bits::BIT);
            }
        }
        if std::env::var_os("NOVICE_TRACE").is_some() {
            let shown = kept.first().map(|f| f.expr.to_string()).unwrap_or_else(|| "nothing".into());
            eprintln!("fit {:<18} {label:<10} points {:>3} -> {:>9} checks, {shown}", self.groups[g].key, inputs.len(), work);
        }
        if kept.first().is_some_and(|f| f.cost < table_cost(target)) {
            self.groups[g].fit = Some(Fit { found: kept, lo, hi });
            return true;
        }
        false
    }

    /// A simple program times a constant plus a constant: `k * p + c` with `p` of at most five nodes over the starting operations (the pool of
    /// everything those make). The constants are not searched for; two points where `p` differs give them, and the rest only has to agree. This is
    /// what reaches `2 * min(a, b) + 5` (seven nodes, and 5 is not in the plain search's short list), and it works for kinds of two numbers,
    /// which have no stage of their own for the numbers in their data.
    fn search_affine(&mut self, g: usize) -> bool {
        let arity = self.groups[g].arity;
        if arity == 0 || arity > 2 {
            return false;
        }
        let Some((inputs, target)) = self.inputs_and_targets(g) else { return false };
        let distinct: std::collections::BTreeSet<i64> = target.iter().copied().collect();
        if inputs.len() < 10 || distinct.len() < 4 {
            return false;
        }
        let (pool, effort) = program::pool(&inputs, arity, &[1, 2, 3], if arity == 1 { 3 } else { 5 }, 12_000);
        let mut work = effort.evaluations;
        let mut exprs: Vec<Expr> = Vec::new();
        for (e, _, vals) in &pool {
            let Some(j) = (1..vals.len()).find(|&j| vals[j] != vals[0]) else { continue };
            let (dt, dv) = (target[j] - target[0], vals[j] - vals[0]);
            if dt % dv != 0 || dt / dv < 1 {
                continue;
            }
            let (k, c) = (dt / dv, target[0] - dt / dv * vals[0]);
            work += vals.len() as u64;
            if !vals.iter().zip(&target).all(|(&v, &t)| k * v + c == t) || (k == 1 && c == 0) {
                continue;
            }
            let scaled = if k == 1 { e.clone() } else { Expr::Bin(program::Op::Mul, Box::new(e.clone()), Box::new(Expr::Const(k))) };
            exprs.push(match c {
                0 => scaled,
                c if c > 0 => Expr::Bin(program::Op::Add, Box::new(scaled), Box::new(Expr::Const(c))),
                c => Expr::Bin(program::Op::Sub, Box::new(scaled), Box::new(Expr::Const(-c))),
            });
            if exprs.len() >= 600 {
                break;
            }
        }
        self.adopt_family(g, &inputs, &target, exprs, work, "affine")
    }

    /// A kind whose answers all lie below some number may be a rule reduced modulo that number: `(k * x + c) mod m`. The modulus is nearly given
    /// (just above the largest answer), and the rest only has to be right up to a multiple of it, which a search of five nodes can do. The
    /// constants are the numbers in the data.
    fn search_modulo(&mut self, g: usize) -> bool {
        let arity = self.groups[g].arity;
        if arity == 0 || arity > 2 {
            return false;
        }
        let Some((inputs, target)) = self.inputs_and_targets(g) else { return false };
        let top = target.iter().copied().max().unwrap_or(0);
        if target.iter().any(|&t| t < 0) || top < 2 {
            return false;
        }
        let mut consts: Vec<i64> = BASE_CONSTS.to_vec();
        if arity == 1 {
            consts.extend(target.iter().chain(inputs.iter().flatten()).copied().filter(|c| (0..=40).contains(c)));
        }
        consts.sort_unstable();
        consts.dedup();
        for m in top + 1..=top + 3 {
            if self.search_group_in(g, &consts, &[], 5, "modular", Some(m)) {
                return true;
            }
        }
        false
    }

    /// Unary kinds only: the numbers in the data as constants, for the programs that mention a number nobody has to be told about (a divisor).
    fn search_data_constants(&mut self, g: usize) -> bool {
        if self.groups[g].arity != 1 {
            return false;
        }
        let Some((inputs, target)) = self.inputs_and_targets(g) else { return false };
        let mut consts: Vec<i64> = BASE_CONSTS.to_vec();
        consts.extend(target.iter().chain(inputs.iter().flatten()).copied().filter(|c| (0..=40).contains(c)));
        consts.sort_unstable();
        consts.dedup();
        self.search_group(g, &consts, &[], 5, "data")
    }

    /// Edit the refuted programs: change a leaf or an operation, swap a constant, or wrap any part in an operation with a constant. The constants
    /// offered are the plain ones, the told answers, and what each refuted program got wrong (what it said, and what it said minus the truth).
    fn repair(&mut self, g: usize, refuted: &[Expr], inputs: &[Vec<i64>], target: &[i64], words: &[Rc<Lib>]) -> bool {
        let arity = self.groups[g].arity;
        let mut consts: Vec<i64> = BASE_CONSTS.to_vec();
        for e in refuted {
            for (x, &t) in inputs.iter().zip(target) {
                if let Some(v) = e.eval(x) {
                    if v != t {
                        consts.extend([(v - t).abs(), v, t]);
                    }
                }
            }
        }
        consts.retain(|c| (0..=200).contains(c));
        consts.sort_unstable();
        consts.dedup();
        let (mut lo, mut hi) = (inputs[0].clone(), inputs[0].clone());
        for p in inputs {
            for i in 0..arity {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
        let probe = probes(&lo, &hi);
        let task = Task { inputs, target, probes: &probe, arity, consts: &consts, words, max_size: 0, rival_size: 0, max_items: MAX_ITEMS };
        let mut found: Vec<Found> = Vec::new();
        for e in refuted.iter().take(4) {
            let (mut more, effort) = program::rivals(e, &task);
            self.stats.evaluations += effort.evaluations;
            found.append(&mut more);
        }
        self.stats.searches += 1;
        found.sort_by(|x, y| x.cost.cmp(&y.cost).then_with(|| x.expr.size().cmp(&y.expr.size())).then_with(|| x.expr.to_string().cmp(&y.expr.to_string())));
        let mut kept: Vec<Found> = Vec::new();
        for f in found {
            if !kept.iter().any(|k| k.probe_values == f.probe_values) {
                kept.push(f);
            }
        }
        kept.truncate(64);
        let table: Cost = table_cost(&target);
        if std::env::var_os("NOVICE_TRACE").is_some() {
            eprintln!("repair {:<14} points {:>3} consts {:>2} -> {}", self.groups[g].key, inputs.len(), consts.len(), kept.first().map(|f| f.expr.to_string()).unwrap_or_else(|| "nothing".into()));
        }
        if kept.first().is_some_and(|f| f.cost < table) {
            self.groups[g].fit = Some(Fit { found: kept, lo, hi });
            return true;
        }
        false
    }

    /// Explain a kind as a small program of one variable applied to what a known word says: `h(word(x, y))`. If the answers are a function of the
    /// word's output (no two points with the same output disagree), `h` is found by a search over one variable, which is a few thousand evaluations;
    /// the word does the hard part. Returns the cheapest explanation over all the words on offer.
    fn compose(&mut self, g: usize, offer: &[Rc<Lib>]) -> bool {
        let Some((inputs, target)) = self.inputs_and_targets(g) else { return false };
        let arity = self.groups[g].arity;
        let mut composites: Vec<(Cost, Expr)> = Vec::new();
        let table: Cost = table_cost(&target);
        for word in offer {
            // The ways to feed the word from the question's own numbers.
            let mut feeds: Vec<Vec<Expr>> = Vec::new();
            if word.arity == 1 {
                feeds.extend((0..arity).map(|i| vec![Expr::Var(i)]));
            } else {
                for i in 0..arity {
                    for j in 0..arity {
                        feeds.push(vec![Expr::Var(i), Expr::Var(j)]);
                    }
                }
            }
            for feed in feeds {
                let call = Expr::Call(word.clone(), feed);
                // The word's output at every point, and what the answer was there.
                let mut seen: BTreeMap<i64, i64> = BTreeMap::new();
                let consistent = inputs.iter().zip(&target).all(|(x, &t)| call.eval(x).is_some_and(|z| *seen.entry(z).or_insert(t) == t));
                self.stats.evaluations += inputs.len() as u64 * call.size() as u64;
                // "The answer is a function of the word's output" says something only if some outputs were met twice (otherwise any data
                // satisfy it): at least one collision, with the same answer, is the evidence.
                if !consistent || seen.len() < 2 || seen.len() >= inputs.len() {
                    continue;
                }
                let zs: Vec<Vec<i64>> = seen.keys().map(|&z| vec![z]).collect();
                let ts: Vec<i64> = seen.values().copied().collect();
                let out = program::search(&Task { inputs: &zs, target: &ts, probes: &[], arity: 1, consts: &BASE_CONSTS, words: &[], max_size: 5, rival_size: 0, max_items: MAX_ITEMS });
                self.stats.evaluations += out.effort.evaluations;
                let Some(h) = out.found.first() else { continue };
                let whole = substitute(&h.expr, &call);
                let cost = program::cost_of(&whole, arity, offer.len());
                if cost < table {
                    composites.push((cost, whole));
                }
            }
        }
        self.stats.searches += 1;
        composites.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.to_string().cmp(&b.1.to_string())));
        let Some((best_cost, best)) = composites.first().cloned() else { return false };
        if std::env::var_os("NOVICE_TRACE").is_some() {
            eprintln!("compose {:<14} points {:>3} words {:>2} -> {best} (+{} others)", self.groups[g].key, inputs.len(), offer.len(), composites.len() - 1);
        }
        // The other words that explain it as well are rivals, as long as their belief is not negligible.
        let others: Vec<Expr> = composites.into_iter().skip(1).filter(|c| c.0 <= best_cost + 32 * bits::BIT).map(|c| c.1).collect();
        self.finish_fit(g, best, others, &inputs, &target, offer)
    }

    /// Keep `best` as the explanation of the group, with the programs one edit away that fit as its rivals.
    fn finish_fit(&mut self, g: usize, best: Expr, others: Vec<Expr>, inputs: &[Vec<i64>], target: &[i64], words: &[Rc<Lib>]) -> bool {
        let arity = self.groups[g].arity;
        let (mut lo, mut hi) = (inputs[0].clone(), inputs[0].clone());
        for p in inputs {
            for i in 0..arity {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
        let probe = probes(&lo, &hi);
        let task = Task { inputs, target, probes: &probe, arity, consts: &BASE_CONSTS, words, max_size: 0, rival_size: 0, max_items: MAX_ITEMS };
        let mut found: Vec<Found> = Vec::new();
        let mut probe_values = Vec::new();
        for p in &probe {
            // Where the program says nothing (a recurrence whose chain does not reach a known value) it takes no side.
            probe_values.push(best.eval(p).unwrap_or(UNDEFINED));
        }
        found.push(Found { cost: program::cost_of(&best, arity, words.len()), expr: best.clone(), probe_values });
        for e in others {
            let probe_values: Vec<i64> = probe.iter().map(|p| e.eval(p).unwrap_or(UNDEFINED)).collect();
            found.push(Found { cost: program::cost_of(&e, arity, words.len()), expr: e, probe_values });
        }
        let (more, effort) = program::rivals(&best, &task);
        self.stats.evaluations += effort.evaluations;
        found.extend(more);
        found.sort_by(|x, y| x.cost.cmp(&y.cost).then_with(|| x.expr.size().cmp(&y.expr.size())).then_with(|| x.expr.to_string().cmp(&y.expr.to_string())));
        let mut kept: Vec<Found> = Vec::new();
        for f in found {
            if !kept.iter().any(|k| k.probe_values == f.probe_values) {
                kept.push(f);
            }
        }
        kept.truncate(64);
        self.groups[g].fit = Some(Fit { found: kept, lo, hi });
        true
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
                if self.groups[g].fit.is_some() || self.inputs_and_targets(g).is_none() {
                    continue;
                }
                let (inputs, target) = self.inputs_and_targets(g).unwrap();
                let mut consts: Vec<i64> = BASE_CONSTS.to_vec();
                consts.extend(target.iter().chain(inputs.iter().flatten()).copied().filter(|c| (0..=3).contains(c)));
                let mut solved = false;
                // Composition is a lookup per word, so it may use every word on offer. A kind that did not yield is tried again only when the
                // words on offer have doubled since: a bigger library must not make a kind that cannot be explained dearer every time.
                if self.groups[g].words_tried == 0 || offer.len() >= 2 * self.groups[g].words_tried {
                    self.groups[g].words_tried = offer.len();
                    let before = self.stats.evaluations;
                    solved = self.charge(g, |s| s.compose(g, &offer));
                    if solved {
                        self.stats.composed += 1;
                        self.stats.composed_evaluations += self.stats.evaluations - before;
                    }
                }
                // Blind enumeration is a search, so it uses only the words that have earned their place in an explanation, and only when
                // there are more of them than the last time.
                if !solved && !self.words.is_empty() && self.groups[g].enumerated_with < self.words.len() {
                    self.groups[g].enumerated_with = self.words.len();
                    let earned = self.words.clone();
                    solved = self.charge(g, |s| s.search_group(g, &consts, &earned, 5, "words"));
                }
                if solved {
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

    /// An answer that is the word of a number, as that number (and whether it was); any other answer as it is.
    fn as_number(&self, answer: &str) -> (String, Option<usize>) {
        match self.numerals.value_of_text(answer) {
            Some((n, family)) => (n.to_string(), Some(family)),
            None => (answer.to_string(), None),
        }
    }

    /// What a kind whose answers are number words says: the word for the number, and nothing if no lesson has a word for it.
    fn speak(&mut self, p: &Parsed, reply: Answer) -> Answer {
        let Some(gi) = self.group_of(p) else { return reply };
        let Some(family) = self.groups[gi].speaks else { return reply };
        let word = |this: &Self, a: &str| a.parse::<i64>().ok().and_then(|n| this.numerals.speak(family, n));
        match reply {
            Answer::Fact(a) => word(self, &a).map_or_else(|| { self.why[0] += 1; Answer::Refuse }, Answer::Fact),
            Answer::Guess(a, pm) => word(self, &a).map_or_else(|| { self.why[0] += 1; Answer::Refuse }, |w| Answer::Guess(w, pm)),
            Answer::Refuse => Answer::Refuse,
        }
    }

    /// What the lookups say about frames, built when first needed after anything new was said.
    fn ensure_frames(&mut self) {
        // Frame words change slowly: the frames are rebuilt when a twentieth more lookups have come in (and at least 25), not for every one.
        if self.frames.is_none() || self.lookups.len() >= self.frames_from + (self.frames_from / 20).max(25) {
            self.frames_from = self.lookups.len();
            self.frames = Some(frames::build(&self.lookups));
        }
    }

    /// Keep what was said to a question with no numbers, in the words it was said in. A lesson or an unlabeled teacher is trusted; an answer from a
    /// named source nobody has checked (a question with no numbers has no program to check it against) is kept apart and is only ever a guess.
    fn remember_lookup(&mut self, p: &Parsed, answer: &str, trusted: bool) {
        if p.nums.is_empty() {
            let ordered = p.seq.join(" ");
            // A second answer for words that had one contradicts what the frames were built from: they are rebuilt at once, not when enough has piled up.
            if self.exact.get(&ordered).is_some_and(|a| !a.contains(answer)) {
                self.frames = None;
            }
            self.exact.entry(ordered.clone()).or_default().insert(answer.to_string());
            if !trusted {
                return;
            }
            self.trusted_lookups.insert(ordered.clone());
            let bag = self.bags.entry(bag_of(p)).or_default();
            bag.0.insert(ordered);
            bag.1.insert(answer.to_string());
            self.lookups.push((p.seq.clone(), answer.to_string()));
        }
    }

    /// How often two orders of the same words, both taught or told, have given the same answer: `(agreements, pairs)`, counting each further order of
    /// a set of words as one pair. This is the only evidence there is about whether order matters in these questions.
    fn reordering_record(&self, signature: &str) -> (u64, u64) {
        let mut record = (0, 0);
        for (bag, (orders, answers)) in &self.bags {
            if orders.len() < 2 || self.signature(bag) != signature {
                continue;
            }
            let pairs = orders.len() as u64 - 1;
            record.1 += pairs;
            if answers.len() == 1 {
                record.0 += pairs;
            }
        }
        record
    }

    /// The words of a question that many lessons share (the frame and the kind, not the thing asked about), as one string. Two questions with the same
    /// signature are the same sort of question, so what was seen of one is evidence about the other.
    fn signature(&self, bag: &str) -> String {
        bag.split(' ').filter(|w| self.df.get(*w).copied().unwrap_or(0) >= COMMON_WORD).collect::<Vec<_>>().join(" ")
    }

    fn make_groups(&mut self) {
        // Words that are numbers, if the lessons say so: every lesson is read again with them as numbers, and the kinds of numbers are then kinds of
        // numbers, learned as any other.
        if self.numerals.is_empty() && !self.raw.is_empty() {
            let reading = numerals::learn_reading(&self.raw);
            let found = if reading.is_empty() { Lexicon::single(numerals::learn(&self.raw)) } else { reading };
            let raw = std::mem::take(&mut self.raw);
            if !found.is_empty() {
                if std::env::var_os("NOVICE_TRACE").is_some() {
                    eprintln!("numerals {} families, {} words", found.families(), found.len());
                }
                self.numerals = found;
                self.lessons.clear();
                self.lesson_spoke.clear();
                for (q, a) in raw {
                    let (answer, spoke) = self.as_number(&a);
                    self.lessons.push((parse_with(&q, &self.numerals), answer));
                    self.lesson_spoke.push(spoke);
                }
            }
        }
        let lessons = std::mem::take(&mut self.lessons);
        let mut spoke = std::mem::take(&mut self.lesson_spoke);
        spoke.resize(lessons.len(), None);
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
        // Widest cover first; among equals, a word that already names a kind (more lessons can arrive later for the same kind).
        candidates.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(self.by_key.contains_key(b.0).cmp(&self.by_key.contains_key(a.0))).then(a.0.cmp(b.0)));
        let mut key_of: Vec<Option<String>> = vec![None; lessons.len()];
        for (w, idx) in candidates {
            for &i in idx {
                key_of[i].get_or_insert_with(|| w.to_string());
            }
        }
        let mut touched: std::collections::BTreeSet<usize> = Default::default();
        for (i, (p, answer)) in lessons.iter().enumerate() {
            // A question no word can claim falls back to its rarest word.
            let Some(key) = key_of[i].clone().or_else(|| p.words.iter().map(|w| (self.df[w], w.clone())).min().map(|(_, w)| w)) else { continue };
            let g = match self.by_key.get(&key) {
                Some(&g) => g,
                None => {
                    self.groups.push(Group { key: key.clone(), arity: p.nums.len(), points: BTreeMap::new(), fit: None, searched_at: 0, words_tried: 0, enumerated_with: 0, recurrence_tried: 0, spent: 0, explained_at: None, table: Default::default(), claims: BTreeMap::new(), disputed: Default::default(), record: (0, 0), chain_record: (0, 0), negative: false, speaks: None, forms: Default::default() });
                    self.by_key.insert(key, self.groups.len() - 1);
                    self.groups.len() - 1
                }
            };
            if spoke[i].is_some() {
                self.groups[g].speaks = spoke[i];
            }
            // (only questions with numbers: without any, the word taken for the kind's name may be a digit word of numbers nobody could read)
            if !p.nums.is_empty() {
                let arrangement = Self::arrangement(p, &self.groups[g].key);
                self.shapes.add(arrangement);
            }
            if self.groups[g].arity == p.nums.len() {
                self.groups[g].forms.insert(p.forms.clone());
                self.add_point(g, &p.nums, answer);
                self.groups[g].claims.entry(p.nums.clone()).or_default().insert(TEACHING.to_string(), answer.clone());
                if let Ok(v) = answer.parse::<i64>() {
                    self.groups[g].table.borrow_mut().entry(p.nums.clone()).or_insert(v);
                }
                touched.insert(g);
            }
        }
        // Teaching can come in several rounds: only the kinds that were taught something are looked at again.
        for g in touched {
            self.charge(g, |s| s.refit(g, &[]));
        }
        self.sleep();
        self.last_resort();
        self.score_teaching();
    }

    /// For the kinds no closed form explains, even with the words learned so far: a recurrence over the kind's own values. A kind it explains
    /// becomes a word like any other, so the kinds built on it are tried again.
    fn last_resort(&mut self) {
        let mut any = false;
        for g in 0..self.groups.len() {
            if self.groups[g].fit.is_none() && self.groups[g].recurrence_tried < self.groups[g].points.len() {
                self.groups[g].recurrence_tried = self.groups[g].points.len() * 2;
                if self.charge(g, |s| s.recurrence(g)) {
                    any = true;
                }
            }
        }
        if any {
            self.sleep();
        }
    }
}

impl System for Novice {
    fn teach(&mut self, question: &str, answer: &str) {
        let p = parse_with(question, &self.numerals);
        self.remember_lookup(&p, answer.trim(), true);
        if self.numerals.is_empty() {
            self.raw.push((question.to_string(), answer.trim().to_string()));
        }
        let (answer, spoke) = self.as_number(answer.trim());
        self.lessons.push((p, answer));
        self.lesson_spoke.push(spoke);
    }

    fn ready(&mut self) {
        self.make_groups();
    }

    fn ask(&mut self, question: &str) -> Answer {
        let p = parse_with(question, &self.numerals);
        if p.nums.is_empty() {
            self.ensure_frames();
        }
        let mut said = Said { answer: None, bucket: None, group: None, chain_of: None };
        let reason = std::cell::Cell::new(0usize);
        let reply = (|| {
            // A question with no numbers is a lookup, not an instance of a kind: the words that two facts share name no kind, and the answer of
            // one must not be given for the other. It is answered only in the words it was taught or told in, and only if those never differed.
            if p.nums.is_empty() {
                let ordered = p.seq.join(" ");
                if let Some(answers) = self.exact.get(&ordered) {
                    if answers.len() != 1 {
                        return None;
                    }
                    let answer = answers.iter().next().cloned().unwrap_or_default();
                    // Said by a source nobody has checked: as sure as one unchecked statement, a guess.
                    return Some(if self.trusted_lookups.contains(&ordered) { Answer::Fact(answer) } else { Answer::Guess(answer, 500) });
                }
                // The same words in an order nobody used: a guess, as sure as the lessons have earned (two orders that agreed every time so far
                // make a good guess, never a fact), and none at all if any order of these words was ever given another answer.
                let bag = bag_of(&p);
                if let Some((_, answers)) = self.bags.get(&bag) {
                    // Only the same sort of question can vouch for it: where no order was ever taught twice, or the two orders ever disagreed, nothing
                    // shows that order does not matter, and the new order is refused.
                    let (agree, pairs) = self.reordering_record(&self.signature(&bag));
                    if pairs > 0 && agree == pairs && answers.len() == 1 {
                        let permille = (1000 * (agree + 1) / (pairs + 2)).min(999) as u32;
                        return Some(Answer::Guess(answers.iter().next().cloned().unwrap_or_default(), permille));
                    }
                }
                // The same fact in a frame it was not taught in: as sure as the frame words that are new to it have been seen to change nothing
                // (each one, the Laplace estimate of its pairs; all of them together, the product), a fact only if that leaves less than
                // 2^-RISK_BITS of doubt.
                let m = self.frames.as_ref()?.lookup(&p.seq)?;
                let (num, den) = (0..m.new_words).fold((1u128, 1u128), |(n, d), _| (n * u128::from(m.support.min(1 << 40) + 1), d * u128::from(m.support.min(1 << 40) + 2)));
                let (num, den) = m.order_support.map_or((num, den), |k| (num * u128::from(k.min(1 << 40) + 1), den * u128::from(k.min(1 << 40) + 2)));
                if num << RISK_BITS >= den * ((1u128 << RISK_BITS) - 1) {
                    return Some(Answer::Fact(m.answer));
                }
                let permille = (1000 * num / den) as u32;
                return (permille >= 500).then(|| Answer::Guess(m.answer, permille.min(999)));
            }
            // A word it has never met may change what is asked: nothing is stated about such a question.
            if p.words.iter().any(|w| !self.df.contains_key(w)) {
                reason.set(1);
                return None;
            }
            let g = &self.groups[self.group_of(&p)?];
            // Two words that each name a kind of their own: which kind is asked is not known, so nothing is stated.
            let named: std::collections::BTreeSet<usize> = p.words.iter().filter_map(|w| self.by_key.get(w).copied()).collect();
            if named.len() > 1 {
                return None;
            }
            // The words of a question in an arrangement that no lesson made up from known parts, for this kind or any other, have no known reading,
            // whatever the numbers say.
            if !self.shapes.known(&Self::arrangement(&p, &g.key)) {
                return None;
            }
            // Numbers written in a form the kind was never taught with (digits for a kind of number words, or the other way round) are not read as theirs.
            if !g.forms.is_empty() && !g.forms.contains(&p.forms) {
                return None;
            }
            if g.arity != p.nums.len() {
                reason.set(2);
                return None;
            }
            if g.fit.is_none() {
                reason.set(3);
                let gi = self.group_of(&p)?;
                let (answer, top, total) = self.believe(gi, &p.nums)?;
                if u128::from(top) << RISK_BITS >= u128::from(total) * ((1u128 << RISK_BITS) - 1) {
                    return Some(Answer::Fact(answer));
                }
                let permille = (u128::from(top) * 1000 / u128::from(total.max(1))) as u32;
                return (permille >= 500).then(|| Answer::Guess(answer, permille.min(999)));
            }
            let fit = g.fit.as_ref()?;
            let Some((value, top, total)) = program::weigh(&fit.found, &p.nums) else {
                reason.set(4);
                return None;
            };
            if value < 0 && !g.negative {
                reason.set(5);
                return None;
            }
            // A program that walks a chain of reductions is a risky kind of explanation: what it was taught or told is a fact, and what it works
            // out is a guess whose confidence is how often such answers have been right.
            if fit.found[0].expr.has_fix() {
                let gi = self.group_of(&p)?;
                if !self.groups[gi].disputed.contains(&p.nums) {
                    if let Some((answer, top, total)) = self.believe(gi, &p.nums) {
                        if u128::from(top) << RISK_BITS >= u128::from(total) * ((1u128 << RISK_BITS) - 1) {
                            return Some(Answer::Fact(answer));
                        }
                    }
                }
                let shared = (1000 * (self.chain_record.0 + 1)) / (self.chain_record.1 + 2);
                let (right, made) = g.chain_record;
                let permille = ((1000 * right + 2 * shared) / (made + 2)).min(999);
                said.chain_of = Some(gi);
                reason.set(7);
                return (permille >= 500).then(|| Answer::Guess(value.to_string(), permille));
            }
            let inside = p.nums.iter().enumerate().all(|(i, &n)| fit.lo[i] <= n && n <= fit.hi[i]);
            if inside {
                // All but 2^-RISK_BITS of the belief on this answer? (Not at a point where a told answer was refused: that stays a guess.)
                let sure = u128::from(top) << RISK_BITS >= u128::from(total) * ((1u128 << RISK_BITS) - 1);
                if sure && !g.disputed.contains(&p.nums) {
                    return Some(Answer::Fact(value.to_string()));
                }
                let permille = (u128::from(top) * 1000 / u128::from(total.max(1))) as u32;
                reason.set(6);
                return (permille >= 500).then(|| Answer::Guess(value.to_string(), permille.min(999)));
            }
            let dist = p.nums.iter().enumerate().map(|(i, &n)| (n - fit.hi[i]).max(fit.lo[i] - n)).max().unwrap_or(1);
            let b = bucket(dist);
            // The kind's own record counts for as much as its history allows; with little of it, the shared record at this distance stands in
            // (two guesses' worth of weight).
            let (right, made) = self.record[b];
            let shared = (1000 * (right + 1)) / (made + 2);
            let (own_right, own_made) = g.record;
            let permille = ((1000 * own_right + 2 * shared) / (own_made + 2)).min(999);
            said.bucket = Some(b);
            said.group = self.group_of(&p);
            reason.set(7);
            (permille >= 500).then(|| Answer::Guess(value.to_string(), permille))
        })();
        let reply = reply.unwrap_or(Answer::Refuse);
        if matches!(reply, Answer::Refuse) {
            self.why[reason.get()] += 1;
        }
        if let Answer::Fact(a) | Answer::Guess(a, _) = &reply {
            said.answer = Some(a.clone());
        } else {
            said.bucket = None;
        }
        self.said.insert(question.to_string(), said);
        self.speak(&p, reply)
    }

    fn told(&mut self, question: &str, truth: &str, source: Option<&str>) {
        let p = parse_with(question, &self.numerals);
        let truth = self.as_number(truth.trim()).0;
        let truth = truth.as_str();
        self.remember_lookup(&p, truth, source.is_none());
        if let Some(said) = self.said.remove(question) {
            if let (Some(gi), Some(a)) = (said.chain_of, said.answer.as_ref()) {
                self.chain_record.1 += 1;
                self.chain_record.0 += u32::from(a == truth);
                self.groups[gi].chain_record.1 += 1;
                self.groups[gi].chain_record.0 += u32::from(a == truth);
            }
            if let (Some(b), Some(a)) = (said.bucket, said.answer) {
                self.record[b].1 += 1;
                self.record[b].0 += u32::from(a == truth);
                if let Some(g) = said.group {
                    self.groups[g].record.1 += 1;
                    self.groups[g].record.0 += u32::from(a == truth);
                }
            }
        }
        let g = match self.group_of(&p) {
            Some(g) => g,
            None => {
                // A kind word it has never met names a new group.
                let Some(key) = p.words.iter().filter(|w| !self.df.contains_key(*w)).min().cloned() else { return };
                self.groups.push(Group { key: key.clone(), arity: p.nums.len(), points: BTreeMap::new(), fit: None, searched_at: 0, words_tried: 0, enumerated_with: 0, recurrence_tried: 0, spent: 0, explained_at: None, table: Default::default(), claims: BTreeMap::new(), disputed: Default::default(), record: (0, 0), chain_record: (0, 0), negative: false, speaks: None, forms: Default::default() });
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
        // The same answer to the same question again from the same speaker (a re-ask, another phrasing, the lessons and then the teacher) is not
        // new evidence: it is counted, not weighed twice.
        let channel = source.unwrap_or("told").to_string();
        let value = truth.parse::<i64>().ok();
        let again = self.groups[g].claims.get(&p.nums).is_some_and(|c| c.iter().any(|(ch, a)| Self::family(ch) == Self::family(&channel) && a == truth));
        if self.groups[g].forms.is_empty() {
            self.groups[g].forms.insert(p.forms.clone());
        }
        self.add_point(g, &p.nums, truth);
        if again {
            return;
        }
        // Where a program is confident, what the channel says is a check on the channel.
        if let (Some(v), Some(confident)) = (value, self.confident_value(g, &p.nums)) {
            let tally = self.reliability.entry(channel.clone()).or_insert((0, 0));
            tally.1 += 1;
            tally.0 += u32::from(confident == v);
        }
        self.groups[g].claims.entry(p.nums.clone()).or_default().insert(channel, truth.to_string());
        let points = self.groups[g].points.len() as u64;
        let mut disputed = false;
        let mut agreed = false;
        let mut changed_favorite = false;
        let mut explained = self.groups[g].fit.is_some();
        if let (Some(fit), Some(v)) = (&mut self.groups[g].fit, value) {
            // A program that says something else is not thrown away: it pays for an exception (which point, and what odd answer), and a rival
            // that says this gains on it. One slip costs the incumbent about what a ceiling costs a rival, so one contradiction changes nothing
            // and two contradictions that a rival explains together win. No count is chosen anywhere.
            let price = bits::log2(points + 1) + bits::uint(v.unsigned_abs());
            let favorite_before = fit.found[0].expr.clone();
            let mut penalized = false;
            for f in fit.found.iter_mut() {
                if f.expr.eval(&p.nums) != Some(v) {
                    f.cost += price;
                    penalized = true;
                }
            }
            fit.found.sort_by(|x, y| x.cost.cmp(&y.cost).then_with(|| x.expr.size().cmp(&y.expr.size())).then_with(|| x.expr.to_string().cmp(&y.expr.to_string())));
            if fit.found[0].expr.eval(&p.nums) == Some(v) {
                agreed = true;
                self.stats.narrowed += u64::from(penalized);
                for (i, &n) in p.nums.iter().enumerate() {
                    fit.lo[i] = fit.lo[i].min(n);
                    fit.hi[i] = fit.hi[i].max(n);
                }
            } else {
                disputed = true;
            }
            let best = fit.found[0].cost;
            fit.found.retain(|f| f.cost <= best + 32 * bits::BIT);
            changed_favorite = fit.found[0].expr != favorite_before;
        } else if self.groups[g].fit.is_some() {
            disputed = true;
        }
        if changed_favorite {
            // A new favorite may get right points the old one got wrong: the region where it has been checked grows with it.
            self.verified_region(g);
        }
        if agreed {
            self.groups[g].disputed.remove(&p.nums);
        }
        if let (false, Some(v)) = (disputed, value) {
            self.groups[g].table.borrow_mut().entry(p.nums.clone()).or_insert(v);
        }
        if disputed {
            self.groups[g].disputed.insert(p.nums.clone());
            self.charge(g, |s| s.repair_priced(g));
            self.verified_region(g);
            // With so many exceptions to pay for that a table is cheaper, the program is not an explanation.
            if let Some((_, target)) = self.observations(g).map(|o| (0, o.into_iter().map(|x| x.1).collect::<Vec<_>>())) {
                let table: Cost = table_cost(&target);
                if self.groups[g].fit.as_ref().is_some_and(|f| f.found[0].cost > table) {
                    self.groups[g].fit = None;
                }
            }
            explained = self.groups[g].fit.is_some();
        } else if !explained {
            // Only a table so far: look again when the group has doubled since the last search.
            let grp = &self.groups[g];
            if grp.points.len() >= 2 * grp.searched_at.max(MIN_POINTS / 2) {
                self.charge(g, |s| s.refit(g, &[]));
                if self.groups[g].fit.is_some() {
                    self.sleep();
                } else {
                    self.sleep();
                    self.last_resort();
                }
            }
        }
        let _ = explained;
    }

    fn quit(&mut self) {
        eprintln!("refusals by reason [other, unknown word, arity, table miss, no program value, negative, inside but unsure, outside and record low]: {:?}", self.why);
        eprintln!("channel reliability (agreed with a confident program / times it could be checked): {:?}", self.reliability);
        let s = self.stats;
        let fits = self.groups.iter().filter(|g| g.fit.is_some()).count();
        eprintln!(
            "novice: {} groups, {} explained by a program; {} searches, {} evaluations, {} hit the size limit; {} told answers narrowed the doubt; {} words earned: {}; {} kinds explained by composition for {} evaluations in all",
            self.groups.len(),
            fits,
            s.searches,
            s.evaluations,
            s.exhausted,
            s.narrowed,
            s.words_earned,
            self.words.iter().map(|w| w.name.as_str()).collect::<Vec<_>>().join(" "),
            s.composed,
            s.composed_evaluations
        );
        let mut order: Vec<&Group> = self.groups.iter().filter(|g| g.explained_at.is_some()).collect();
        order.sort_by_key(|g| g.explained_at);
        eprintln!("evaluations spent before each kind was first explained, in the order they were explained:");
        eprintln!("  {}", order.iter().map(|g| g.spent.to_string()).collect::<Vec<_>>().join(" "));
        for g in &self.groups {
            if let Some(f) = &g.fit {
                eprintln!("  {:<14} {} ({} rival behaviors; {} evaluations to explain)", g.key, f.found[0].expr, f.found.len() - 1, g.spent);
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
        // A kind it knows nothing about yet becomes a group when it is told. Told once, by a teacher whose lessons were all consistent but few, it
        // is believed, not yet a fact.
        n.told("what is 4 zorbosido 2", "8", None);
        match n.ask("what is 4 zorbosido 2") {
            Answer::Fact(a) => assert_eq!(a, "8"),
            Answer::Guess(a, p) => {
                assert_eq!(a, "8");
                assert!(p >= 900, "{p}");
            }
            Answer::Refuse => panic!("refused"),
        }
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
        // One contradiction is cheaper to explain as a slip than as a ceiling, so nothing changes yet.
        n.told("what is 25 nakisido 20", "30", None);
        assert!(!matches!(n.ask("what is 25 nakisido 20"), Answer::Fact(_)), "a disputed point is never a fact");
        // Two that a ceiling explains together are not a slip: the ceiling is believed (still short of a fact: the old rule is 9 bits behind), and
        // the region where it has been checked takes in both points.
        n.told("what is 22 nakisido 21", "30", None);
        match n.ask("what is 23 nakisido 20") {
            Answer::Fact(a) | Answer::Guess(a, _) => assert_eq!(a, "30"),
            Answer::Refuse => panic!("refused"),
        }
        // A third, and it is a fact inside the region now checked.
        n.told("what is 24 nakisido 22", "30", None);
        assert_eq!(fact(&n.ask("what is 23 nakisido 20")), Some("30"));
        assert_eq!(fact(&n.ask("what is 3 nakisido 4")), Some("7"));
    }

    #[test]
    fn one_wrong_answer_from_the_teacher_is_not_believed() {
        let mut n = taught();
        assert_eq!(fact(&n.ask("what is 6 fegisido 3")), Some("9"));
        // The teacher says 99 for a sum that is 9. Nothing explains 99, so it is a slip: the program keeps its belief, and the point is only a guess.
        n.told("what is 6 fegisido 3", "99", None);
        match n.ask("what is 6 fegisido 3") {
            Answer::Guess(a, _) => assert_eq!(a, "9"),
            other => panic!("expected a guess of 9, got {other:?}"),
        }
        // A question nearby is unaffected.
        assert_eq!(fact(&n.ask("what is 6 fegisido 4")), Some("10"));
    }

    #[test]
    fn a_cheaper_rule_that_fits_every_lesson_does_not_make_a_fact_where_a_rival_disagrees() {
        // Found by the exam (seed 2): the lessons for a digit-sum kind skip 7, 9 and 10, and "x mod 9" fits all of them and is a little cheaper to
        // write than the digit sum. At 9 the two disagree (0 against 9). The rival has to keep enough belief that 9 is not stated as a fact.
        let mut n = Novice::default();
        for x in [0, 1, 2, 3, 4, 5, 6, 8, 11, 12, 13, 14, 15] {
            let digit_sum = x / 10 + x % 10;
            for template in ["what is lokesido {x}", "lokesido {x}", "lokesido {x} equals"] {
                n.teach(&template.replace("{x}", &x.to_string()), &digit_sum.to_string());
            }
        }
        n.ready();
        assert!(!matches!(n.ask("what is lokesido 9"), Answer::Fact(_)), "{:?}", n.ask("what is lokesido 9"));
        // Where the rivals agree it is still a fact.
        assert_eq!(fact(&n.ask("what is lokesido 4")), Some("4"));
    }

    /// A kind with no rule (arbitrary codes), taught, with the reliabilities that checking elsewhere would have measured.
    fn codes() -> Novice {
        let mut n = Novice::default();
        for (x, a) in [(1, 44), (2, 97), (3, 5), (4, 63), (5, 18), (6, 71), (7, 39), (8, 52)] {
            n.teach(&format!("what is zigsido {x}"), &a.to_string());
        }
        n.ready();
        n.reliability.insert(TEACHING.to_string(), (3000, 3000));
        n.reliability.insert("teacher".to_string(), (80, 100));
        n.reliability.insert("second".to_string(), (95, 100));
        n
    }

    #[test]
    fn two_sources_that_agree_make_a_fact() {
        let mut n = codes();
        n.told("what is zigsido 9", "30", Some("teacher"));
        n.told("what is zigsido 9", "30", Some("second"));
        assert_eq!(fact(&n.ask("what is zigsido 9")), Some("30"));
    }

    #[test]
    fn two_sources_that_disagree_favor_the_more_reliable_one_with_doubt() {
        let mut n = codes();
        n.told("what is zigsido 10", "31", Some("teacher"));
        n.told("what is zigsido 10", "77", Some("second"));
        match n.ask("what is zigsido 10") {
            Answer::Guess(a, p) => {
                assert_eq!(a, "77");
                assert!((700..=950).contains(&p), "stated {p}");
            }
            other => panic!("expected a guess, got {other:?}"),
        }
    }

    #[test]
    fn one_source_measured_at_eighty_percent_can_only_guess() {
        let mut n = codes();
        n.told("what is zigsido 11", "12", Some("teacher"));
        match n.ask("what is zigsido 11") {
            Answer::Guess(a, p) => {
                assert_eq!(a, "12");
                assert!((700..=850).contains(&p), "stated {p}");
            }
            other => panic!("expected a guess, got {other:?}"),
        }
        // What the lessons taught stays a fact.
        assert_eq!(fact(&n.ask("what is zigsido 2")), Some("97"));
    }

    #[test]
    fn a_source_is_judged_where_a_program_can_check_it() {
        let mut n = taught();
        let before = n.reliability.get("second").copied().unwrap_or((0, 0));
        // 6 + 3 is 9 and the program is sure inside the taught region: a source that says 15 is caught out, one that says 9 is confirmed.
        n.told("what is 6 fegisido 3", "15", Some("second"));
        n.told("what is 5 fegisido 3", "8", Some("second"));
        let after = n.reliability["second"];
        assert_eq!((after.0 - before.0, after.1 - before.1), (1, 2));
    }
}

#[cfg(test)]
mod recurrence_tests {
    use super::*;

    fn teach_unary(n: &mut Novice, name: &str, rule: impl Fn(i64) -> i64, points: impl IntoIterator<Item = i64>) {
        for x in points {
            for template in ["what is {w} {x}", "{w} {x}", "{w} {x} equals"] {
                n.teach(&template.replace("{w}", name).replace("{x}", &x.to_string()), &rule(x).to_string());
            }
        }
    }

    fn teach_binary(n: &mut Novice, name: &str, rule: impl Fn(i64, i64) -> i64, pairs: &[(i64, i64)]) {
        for &(x, y) in pairs {
            for template in ["what is {x} {w} {y}", "{x} {w} {y}", "{x} {w} {y} equals"] {
                n.teach(&template.replace("{w}", name).replace("{x}", &x.to_string()).replace("{y}", &y.to_string()), &rule(x, y).to_string());
            }
        }
    }

    fn popcount(x: i64) -> i64 {
        i64::from((x as u32).count_ones())
    }

    fn gcd(a: i64, b: i64) -> i64 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }

    fn fact(a: &Answer) -> Option<&str> {
        match a {
            Answer::Fact(s) => Some(s),
            _ => None,
        }
    }

    fn answer_text(a: &Answer) -> String {
        match a {
            Answer::Fact(s) => format!("fact {s}"),
            Answer::Guess(s, p) => format!("guess {s} at {p}"),
            Answer::Refuse => "refuse".to_string(),
        }
    }

    #[test]
    fn it_acquires_counting_the_one_bits_from_a_table_nobody_gave_a_rule_for() {
        let mut n = Novice::default();
        // Not 0, 8, 18, 19: the lessons leave gaps, as the hidden worlds do.
        teach_unary(&mut n, "zarbosido", popcount, (1..=17).filter(|x| *x != 8));
        n.ready();
        assert_eq!(n.kinds(), (1, 1), "the kind should be explained by a recurrence");
        // What was taught is a fact.
        assert_eq!(fact(&n.ask("what is zarbosido 13")), Some("3"));
        // What the recurrence works out is a guess with a stated confidence, right here: 8 was never taught (1 bit), 25 is beyond the range (3).
        for (q, truth) in [("what is zarbosido 8", "1"), ("what is zarbosido 25", "3"), ("what is zarbosido 30", "4")] {
            match n.ask(q) {
                Answer::Guess(a, p) => {
                    assert_eq!(a, truth, "{q}");
                    assert!(p >= 500, "{q}: {p}");
                }
                other => panic!("{q}: expected a guess, got {}", answer_text(&other)),
            }
        }
    }

    #[test]
    fn it_acquires_euclids_rule_from_pairs_and_answers_a_pair_it_never_saw() {
        let mut n = Novice::default();
        let taught: Vec<(i64, i64)> = (1..=15).flat_map(|a| (1..=15).map(move |b| (a, b))).filter(|&(a, b)| (a * 7 + b * 13) % 4 != 0).collect();
        teach_binary(&mut n, "tarmosido", gcd, &taught);
        n.ready();
        assert_eq!(n.kinds(), (1, 1), "the kind should be explained by a recurrence");
        // A pair that was not taught whose next step is: Euclid's chain reaches a known value.
        let (a, b) = (1..=15)
            .flat_map(|a| (1..=15).map(move |b| (a, b)))
            .find(|&(a, b)| !taught.contains(&(a, b)) && taught.contains(&(b, a % b)))
            .expect("some pair is missing and one step from a taught one");
        match n.ask(&format!("what is {a} tarmosido {b}")) {
            Answer::Guess(g, _) => assert_eq!(g, gcd(a, b).to_string(), "({a}, {b})"),
            other => panic!("({a}, {b}): expected a guess, got {}", answer_text(&other)),
        }
    }

    #[test]
    fn arbitrary_codes_get_no_rule_however_many_recurrences_it_tries() {
        let mut n = Novice::default();
        let mut seed = 12345u64;
        let mut code = |_: i64| {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((seed >> 33) % 100) as i64
        };
        let rule: Vec<i64> = (0..=20).map(|x| code(x)).collect();
        teach_unary(&mut n, "kordulsido", |x| rule[x as usize], 0..=20);
        n.ready();
        assert_eq!(n.kinds(), (1, 0), "a table of arbitrary codes has no rule");
        assert_eq!(n.ask("what is kordulsido 27"), Answer::Refuse);
        assert_eq!(fact(&n.ask("what is kordulsido 5")), Some(rule[5].to_string().as_str()));
    }

    #[test]
    fn a_kind_built_on_an_acquired_word_is_explained_by_composition() {
        let mut n = Novice::default();
        teach_unary(&mut n, "zarbosido", popcount, (1..=17).filter(|x| *x != 8));
        // Parity of the number of one bits: a function of the first kind's value, with its own lessons.
        teach_unary(&mut n, "morpelsido", |x| popcount(x) % 2, (1..=17).filter(|x| *x != 5));
        n.ready();
        assert_eq!(n.kinds(), (2, 2));
        match n.ask("what is morpelsido 5") {
            Answer::Guess(a, _) => assert_eq!(a, "0"),
            other => panic!("expected a guess, got {}", answer_text(&other)),
        }
    }

    #[test]
    fn what_a_chain_works_out_is_never_a_fact() {
        let mut n = Novice::default();
        teach_unary(&mut n, "zarbosido", popcount, (1..=17).filter(|x| *x != 8));
        n.ready();
        for x in [8, 20, 25, 31] {
            assert!(!matches!(n.ask(&format!("what is zarbosido {x}")), Answer::Fact(_)), "{x}");
        }
        // Told, it is a fact from then on, and the record of the chain's guesses is kept.
        n.told("what is zarbosido 8", "1", Some("teacher"));
        assert_eq!(n.chain_record, (1, 1));
        assert_eq!(fact(&n.ask("what is zarbosido 8")), Some("1"));
    }

    #[test]
    fn a_rule_reduced_modulo_a_constant_is_found_from_the_size_of_the_answers() {
        // (7 * x + 1) mod 8: three constants and seven nodes, out of reach of an enumeration, but every answer is below 8.
        let mut n = Novice::default();
        teach_unary(&mut n, "morgulsido", |x| (7 * x + 1) % 8, (0..=19).filter(|x| *x != 6 && *x != 13));
        n.ready();
        assert_eq!(n.kinds(), (1, 1), "the kind should be explained");
        // A point inside the range that was never taught, and one beyond it.
        assert_eq!(fact(&n.ask("what is morgulsido 6")), Some("3"));
        assert_eq!(fact(&n.ask("what is morgulsido 13")), Some("4"));
        match n.ask("what is morgulsido 27") {
            Answer::Guess(a, _) => assert_eq!(a, "6"),
            Answer::Fact(a) => assert_eq!(a, "6"),
            Answer::Refuse => panic!("refused a point beyond the range of a closed-form rule"),
        }
    }

    #[test]
    fn a_step_over_two_earlier_values_is_found_on_a_table_with_enough_points() {
        // Fibonacci numbers mod 8: f(x) = f(x - 1) + f(x - 2), with the first two values given.
        let fib = |n: i64| -> i64 {
            let (mut a, mut b) = (0i64, 1i64);
            for _ in 0..n {
                let c = (a + b) % 8;
                a = b;
                b = c;
            }
            a
        };
        let mut n = Novice::default();
        teach_unary(&mut n, "fibulsido", fib, (0..=23).filter(|x| *x != 14 && *x != 21));
        n.ready();
        assert_eq!(n.kinds(), (1, 1), "the kind should be explained by a recurrence over two earlier values");
        // The chain: a point never taught, and one beyond the range, both worked out from the values on either side.
        for x in [14, 21, 27] {
            match n.ask(&format!("what is fibulsido {x}")) {
                Answer::Guess(a, _) => assert_eq!(a, fib(x).to_string(), "x={x}"),
                other => panic!("x={x}: expected a guess, got {}", answer_text(&other)),
            }
        }
    }

    #[test]
    fn a_step_over_two_values_must_use_both() {
        // A table where the value two steps back is irrelevant: popcount. A "two earlier values" rule that ignores one is the one-value rule.
        let mut n = Novice::default();
        teach_unary(&mut n, "zarbosido", popcount, (1..=17).filter(|x| *x != 8));
        n.ready();
        let shown = n.groups[0].fit.as_ref().map(|f| f.found[0].expr.to_string()).unwrap_or_default();
        assert!(!shown.contains(", s = f("), "{shown}");
    }

    #[test]
    fn a_threshold_rule_with_constants_out_of_reach_is_found_from_the_shape_of_the_data() {
        // min(a, 13) + 3 * max(a - 13, 0): a knee at 13 and a slope of 3 after it; two constants that a short fixed list does not have.
        let rule = |x: i64| x.min(13) + 3 * (x - 13).max(0);
        let mut n = Novice::default();
        teach_unary(&mut n, "bralmosido", rule, (0..=23).filter(|x| *x != 6 && *x != 17));
        n.ready();
        assert_eq!(n.kinds(), (1, 1), "the shape of the data should announce the knee and the slope");
        // Inside the range, never taught: a fact. Beyond it: the same rule, as a guess or a fact, but the right value.
        assert_eq!(fact(&n.ask("what is bralmosido 17")), Some(rule(17).to_string().as_str()));
        match n.ask("what is bralmosido 29") {
            Answer::Guess(a, _) | Answer::Fact(a) => assert_eq!(a, rule(29).to_string()),
            Answer::Refuse => panic!("refused a point beyond the range of a closed-form rule"),
        }
    }

    #[test]
    fn a_fold_is_found_from_its_period_and_its_plateau() {
        let rule = |x: i64| (x % 11).max(3) - 3;
        let mut n = Novice::default();
        teach_unary(&mut n, "folmursido", rule, (0..=23).filter(|x| *x != 5 && *x != 15));
        n.ready();
        assert_eq!(n.kinds(), (1, 1));
        assert_eq!(fact(&n.ask("what is folmursido 15")), Some(rule(15).to_string().as_str()));
    }

    #[test]
    fn a_floor_of_a_line_with_large_constants_is_found_from_where_its_steps_fall() {
        // (29 * x + 11) div 9: three constants that no short list has, and seven nodes; the steps are 3 or 4 about every point.
        let rule = |x: i64| (29 * x + 11) / 9;
        let mut n = Novice::default();
        teach_unary(&mut n, "flormosido", rule, (0..=19).filter(|x| *x != 6 && *x != 13));
        n.ready();
        assert_eq!(n.kinds(), (1, 1), "the steps of a floor announce its slope as a ratio");
        assert_eq!(fact(&n.ask("what is flormosido 6")), Some(rule(6).to_string().as_str()));
        match n.ask("what is flormosido 27") {
            Answer::Guess(a, _) | Answer::Fact(a) => assert_eq!(a, rule(27).to_string()),
            Answer::Refuse => panic!("refused a point beyond the range of a closed-form rule"),
        }
    }

    #[test]
    fn a_floor_of_a_plane_with_large_constants_is_found() {
        // (5 * x + 7 * y + 13) div 11: nine nodes, out of reach of an enumeration.
        let rule = |x: i64, y: i64| (5 * x + 7 * y + 13) / 11;
        let taught: Vec<(i64, i64)> = (1..=15).flat_map(|a| (1..=15).map(move |b| (a, b))).filter(|&(a, b)| (a * 7 + b * 13) % 4 != 0).collect();
        let mut n = Novice::default();
        teach_binary(&mut n, "florplasido", rule, &taught);
        n.ready();
        assert_eq!(n.kinds(), (1, 1), "a plane's floor should be fitted");
        let (a, b) = (1..=15).flat_map(|a| (1..=15).map(move |b| (a, b))).find(|p| !taught.contains(p)).expect("a pair that was not taught");
        assert_eq!(fact(&n.ask(&format!("what is {a} florplasido {b}"))), Some(rule(a, b).to_string().as_str()));
    }

    #[test]
    fn a_sum_of_two_multiples_reduced_modulo_a_constant_is_found() {
        // (13 * x + 22 * y + 5) mod 47: every residue appears, the modulus is just above the largest answer.
        let rule = |x: i64, y: i64| (13 * x + 22 * y + 5) % 47;
        let taught: Vec<(i64, i64)> = (1..=15).flat_map(|a| (1..=15).map(move |b| (a, b))).filter(|&(a, b)| (a * 7 + b * 13) % 4 != 0).collect();
        let mut n = Novice::default();
        teach_binary(&mut n, "resmodsido", rule, &taught);
        n.ready();
        assert_eq!(n.kinds(), (1, 1));
        let (a, b) = (1..=15).flat_map(|a| (1..=15).map(move |b| (a, b))).find(|p| !taught.contains(p)).expect("a pair that was not taught");
        assert_eq!(fact(&n.ask(&format!("what is {a} resmodsido {b}"))), Some(rule(a, b).to_string().as_str()));
    }

    #[test]
    fn a_quadratic_reduced_modulo_a_constant_is_found() {
        // (37 * x * x + 5 * x + 11) mod 53: nine nodes and three large constants.
        let rule = |x: i64| (37 * x * x + 5 * x + 11) % 53;
        let mut n = Novice::default();
        teach_unary(&mut n, "quadmodsido", rule, (0..=19).filter(|x| *x != 6 && *x != 13));
        n.ready();
        assert_eq!(n.kinds(), (1, 1));
        assert_eq!(fact(&n.ask("what is quadmodsido 6")), Some(rule(6).to_string().as_str()));
        match n.ask("what is quadmodsido 27") {
            Answer::Guess(a, _) | Answer::Fact(a) => assert_eq!(a, rule(27).to_string()),
            Answer::Refuse => panic!("refused a point beyond the range of a closed-form rule"),
        }
    }

    fn all_pairs() -> Vec<(i64, i64)> {
        (1..=15).flat_map(|a| (1..=15).map(move |b| (a, b))).collect()
    }

    fn some_pairs_held_out() -> (Vec<(i64, i64)>, (i64, i64)) {
        let taught: Vec<(i64, i64)> = all_pairs().into_iter().filter(|&(a, b)| (a * 7 + b * 13) % 4 != 0).collect();
        let held = all_pairs().into_iter().find(|p| !taught.contains(p)).expect("a pair that was not taught");
        (taught, held)
    }

    #[test]
    fn a_rule_of_two_numbers_with_a_constant_out_of_the_short_list_is_found() {
        // 2 * min(a, b) + 5: seven nodes, and 5 is not one of the constants the plain search has.
        let rule = |x: i64, y: i64| 2 * x.min(y) + 5;
        let (taught, (a, b)) = some_pairs_held_out();
        let mut n = Novice::default();
        teach_binary(&mut n, "offmodsido", rule, &taught);
        n.ready();
        assert_eq!(n.kinds(), (1, 1), "the answers are a simple program plus a constant");
        assert_eq!(fact(&n.ask(&format!("what is {a} offmodsido {b}"))), Some(rule(a, b).to_string().as_str()));
    }

    #[test]
    fn a_product_plus_a_large_constant_and_a_scaled_maximum_are_found() {
        let (taught, (a, b)) = some_pairs_held_out();
        for (name, rule) in [("prodoffsido", (|x: i64, y: i64| x * y + 17) as fn(i64, i64) -> i64), ("scalemaxsido", |x: i64, y: i64| 5 * x.max(y) + 9)] {
            let mut n = Novice::default();
            teach_binary(&mut n, name, rule, &taught);
            n.ready();
            assert_eq!(n.kinds(), (1, 1), "{name}");
            assert_eq!(fact(&n.ask(&format!("what is {a} {name} {b}"))), Some(rule(a, b).to_string().as_str()), "{name}");
        }
    }

    #[test]
    fn a_table_of_pairs_that_is_a_program_plus_a_constant_only_by_accident_is_refused() {
        // Arbitrary codes over pairs: no pooled program plus a constant fits sixty points.
        let (taught, _) = some_pairs_held_out();
        let mut seed = 4242u64;
        let codes: Vec<i64> = taught
            .iter()
            .map(|_| {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                ((seed >> 33) % 30) as i64
            })
            .collect();
        let mut n = Novice::default();
        for (i, &(x, y)) in taught.iter().enumerate() {
            for template in ["what is {x} {w} {y}", "{x} {w} {y}", "{x} {w} {y} equals"] {
                n.teach(&template.replace("{w}", "noisepairsido").replace("{x}", &x.to_string()).replace("{y}", &y.to_string()), &codes[i].to_string());
            }
        }
        n.ready();
        assert_eq!(n.kinds(), (1, 0));
        assert_eq!(n.ask("what is 3 noisepairsido 20"), Answer::Refuse);
    }

    /// Kinds that a recurrence explains from the first stage, then kinds of arbitrary codes that nothing explains: the spend on each.
    fn spends_on_failing_after_successes() -> (Vec<u64>, (usize, usize)) {
        let bit_length = |x: i64| i64::from(64 - (x as u64).leading_zeros());
        let digit_sum3 = |mut x: i64| {
            let mut t = 0;
            while x > 0 {
                t += x % 3;
                x /= 3;
            }
            t
        };
        let twos = |x: i64| if x == 0 { 0 } else { i64::from(x.trailing_zeros()) };
        let mut n = Novice::default();
        teach_unary(&mut n, "zarbosido", popcount, (1..=22).filter(|x| *x != 8));
        teach_unary(&mut n, "bitlensido", bit_length, (1..=22).filter(|x| *x != 9));
        teach_unary(&mut n, "tridigsido", digit_sum3, (1..=22).filter(|x| *x != 10));
        teach_unary(&mut n, "twosumsido", twos, (1..=22).filter(|x| *x != 12));
        let mut seed = 31337u64;
        for k in 0..8 {
            let rule: Vec<i64> = (0..=22)
                .map(|_| {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    ((seed >> 33) % 100) as i64
                })
                .collect();
            teach_unary(&mut n, &format!("noise{}sido", ["a", "b", "c", "d", "e", "f", "g", "h"][k]), |x| rule[x as usize], (0..=22).filter(|x| *x != 7 && *x != 14));
        }
        n.ready();
        let mut noise: Vec<(String, u64)> = n.groups.iter().filter(|g| g.key.starts_with("noise")).map(|g| (g.key.clone(), g.spent)).collect();
        noise.sort();
        (noise.into_iter().map(|(_, spent)| spent).collect(), n.kinds())
    }

    #[test]
    fn what_a_failing_kind_costs_falls_with_the_failures_that_came_before_it() {
        let (spent, kinds) = spends_on_failing_after_successes();
        assert_eq!(kinds, (12, 4), "four recurrences found, eight kinds of arbitrary codes left unexplained");
        // Same kind of data, same size: the dear stages (larger steps, two earlier values) have never paid, so after seven failures they get
        // little. Before this was a fall of a quarter; the first stage and the closed forms are paid in full every time.
        assert!(spent[7] * 10 < spent[0] * 6, "the eighth failing kind cost {} against {} for the first", spent[7], spent[0]);
    }

    #[test]
    fn a_question_without_numbers_is_answered_only_in_the_words_it_was_taught_in() {
        // The static exam's shape: invented words, no numbers, one answer per way of asking. A word shared by two facts names no kind.
        let mut n = Novice::default();
        for (q, a) in [
            ("how is bafoba sedavubi", "vetupa"),
            ("sedavubi bafoba", "vetupa"),
            ("how is podoba sedavubi", "favapa"),
            ("podoba is sedavubi how", "favapa"),
            ("what is the madovubi of podoba", "bivapa"),
            ("madovubi of podoba", "bivapa"),
        ] {
            n.teach(q, a);
        }
        n.ready();
        // Every way of asking that was taught gets its own answer, not the answer of another fact that shares a word.
        assert_eq!(fact(&n.ask("podoba is sedavubi how")), Some("favapa"));
        assert_eq!(fact(&n.ask("how is podoba sedavubi")), Some("favapa"));
        assert_eq!(fact(&n.ask("what is the madovubi of podoba")), Some("bivapa"));
        assert_eq!(fact(&n.ask("sedavubi bafoba")), Some("vetupa"));
        // A way of asking nobody taught is not answered, however close it is to a taught one, and a trap (words from two facts) is refused.
        assert_eq!(n.ask("sedavubi podoba"), Answer::Refuse);
        assert_eq!(n.ask("the madovubi of bafoba"), Answer::Refuse);
        assert_eq!(n.ask("how is bafoba madovubi"), Answer::Refuse);
    }

    #[test]
    fn a_way_of_asking_taught_with_two_answers_is_refused() {
        let mut n = Novice::default();
        n.teach("how is bafoba sedavubi", "vetupa");
        n.teach("how is bafoba sedavubi", "favapa");
        n.ready();
        assert_eq!(n.ask("how is bafoba sedavubi"), Answer::Refuse);
    }

    #[test]
    fn a_new_order_of_the_words_of_a_taught_question_is_a_guess_with_the_confidence_the_lessons_earn() {
        let mut n = Novice::default();
        // Facts taught in two orders each: in every one the two orders gave the same answer, which is the evidence that order does not matter here.
        for (a, w) in [("bafoba", "vetupa"), ("tadaba", "bavapa"), ("podoba", "favapa"), ("mifuba", "bevapa"), ("ligaba", "gutopa")] {
            n.teach(&format!("how is {a} sedavubi"), w);
            n.teach(&format!("{a} is sedavubi how"), w);
        }
        // And one fact taught in one order only.
        n.teach("how is nufeba sedavubi", "nevapa");
        n.ready();
        match n.ask("is nufeba sedavubi how") {
            Answer::Guess(a, p) => {
                assert_eq!(a, "nevapa");
                assert!((800..1000).contains(&p), "five orders agreed out of five: {p}");
            }
            other => panic!("expected a guess, got {}", answer_text(&other)),
        }
        // What was taught in exactly those words is still a fact.
        assert_eq!(fact(&n.ask("how is nufeba sedavubi")), Some("nevapa"));
        // The words of a question nobody taught, in any order, are refused.
        assert_eq!(n.ask("sedavubi how nufeba bafoba"), Answer::Refuse);
    }

    #[test]
    fn a_new_order_is_refused_where_no_order_was_ever_taught_twice() {
        // A kind of two words taught in one order per pair: nothing shows whether "two plus five" is "five plus two", so it is a trap, not a guess.
        let mut n = Novice::default();
        for (a, b, w) in [("five", "two", "three"), ("one", "four", "five"), ("six", "three", "three"), ("nine", "one", "eight"), ("four", "four", "zero"), ("seven", "two", "five"), ("eight", "five", "three"), ("two", "two", "zero"), ("three", "one", "two"), ("six", "six", "zero"), ("ten", "four", "six"), ("five", "one", "four")] {
            n.teach(&format!("what is {a} nidovubi {b}"), w);
            n.teach(&format!("{a} nidovubi {b}"), w);
        }
        // Facts of another kind that were taught in two orders, and agreed.
        for (a, w) in [("bafoba", "vetupa"), ("tadaba", "bavapa"), ("podoba", "favapa"), ("mifuba", "bevapa"), ("ligaba", "gutopa"), ("nufeba", "nevapa")] {
            n.teach(&format!("how is {a} sedavubi"), w);
            n.teach(&format!("{a} is sedavubi how"), w);
        }
        n.ready();
        assert_eq!(n.ask("what is two nidovubi five"), Answer::Refuse);
        assert_eq!(n.ask("what is five nidovubi two"), Answer::Fact("three".into()));
        // The other kind, whose orders were taught and agreed, still gets its guess for a new order.
        assert!(matches!(n.ask("is bafoba sedavubi how"), Answer::Guess(..) | Answer::Fact(..)));
    }

    #[test]
    fn when_the_order_of_the_words_changes_the_answer_a_new_order_is_refused() {
        let mut n = Novice::default();
        n.teach("man bites dog", "news");
        n.teach("dog bites man", "usual");
        n.ready();
        assert_eq!(fact(&n.ask("man bites dog")), Some("news"));
        assert_eq!(n.ask("bites dog man"), Answer::Refuse);
    }

    #[test]
    fn a_word_said_twice_is_not_the_same_question_as_the_word_said_once() {
        // Digits of an invented positional counting repeat ("sed sed" is not "sed"): the sixth blind set told a wrong fact for each repeat.
        let mut n = Novice::default();
        n.teach("what is gokiyarn sed sed", "gar");
        n.teach("what is gokiyarn sed", "sim");
        n.teach("what is gokiyarn sim sim", "bat");
        n.teach("what is henuyarn sim", "dag");
        n.ready();
        assert_eq!(fact(&n.ask("what is gokiyarn sed sed")), Some("gar"));
        assert_eq!(fact(&n.ask("what is gokiyarn sed")), Some("sim"));
        // Taught only with the digit twice, asked with it once, and the other way round: nothing was taught for these words.
        assert_eq!(n.ask("what is gokiyarn sim"), Answer::Refuse);
        assert_eq!(n.ask("what is henuyarn sim sim"), Answer::Refuse);
    }

    #[test]
    fn a_word_said_twice_does_not_make_a_frame_word_of_the_difference_between_two_wordings() {
        // "sim sim" and "sim" are different content: a pair of lessons that answer alike by chance must not show that "what is" changes nothing
        // for a fact whose words repeat.
        let mut n = Novice::default();
        for (i, k) in ["gokiyarn", "henuyarn", "luloryarn", "zareyarn", "ronirnyarn", "sukuyarn", "mamiyarn", "doniryarn"].iter().enumerate() {
            n.teach(&format!("please {k} sim sim"), &format!("a{i}"));
            n.teach(&format!("what is {k} sim"), &format!("a{i}"));
            n.teach(&format!("please {k} sed"), &format!("b{i}"));
        }
        n.ready();
        // "what is K sim sim" was never taught; with "what is" in doubt it may be a guess at most, never a fact.
        assert!(fact(&n.ask("what is gokiyarn sim sim")).is_none());
    }

    /// Teach `name` (x + k) in the wordings `frames` (with {w} and {a}) over a few numbers.
    fn teach_plus(n: &mut Novice, name: &str, k: i64, frames: &[&str]) {
        for a in 1..=12i64 {
            for f in frames {
                n.teach(&f.replace("{w}", name).replace("{a}", &a.to_string()), &(a + k).to_string());
            }
        }
    }

    #[test]
    fn an_arrangement_of_words_and_numbers_nobody_taught_is_refused_but_a_way_of_asking_taught_for_another_kind_is_not() {
        let mut n = Novice::default();
        teach_plus(&mut n, "bumpyarn", 3, &["apply {w} to {a}", "{w} {a}"]);
        teach_plus(&mut n, "hopyarn", 5, &["the {w} of {a}", "{a} under {w}"]);
        n.ready();
        // Taught for this kind.
        assert_eq!(fact(&n.ask("apply bumpyarn to 7")), Some("10"));
        // Taught only for the other kind: the same words in the same order, so it is a way of asking, and it means the same.
        assert_eq!(fact(&n.ask("the bumpyarn of 7")), Some("10"));
        assert_eq!(fact(&n.ask("7 under bumpyarn")), Some("10"));
        // The same words in an order that no lesson used for any kind: no reading is known, whatever the numbers say.
        assert_eq!(n.ask("7 bumpyarn of"), Answer::Refuse);
        assert_eq!(n.ask("of 7 bumpyarn"), Answer::Refuse);
        assert_eq!(n.ask("7 to bumpyarn apply"), Answer::Refuse);
    }

    #[test]
    fn a_prefix_and_a_suffix_that_were_each_taught_around_the_same_core_make_a_way_of_asking_that_means_the_same() {
        let mut n = Novice::default();
        teach_plus(&mut n, "bumpyarn", 3, &["what is {w} {a}", "{w} {a} equals"]);
        teach_plus(&mut n, "hopyarn", 5, &["{w} {a}", "what is {w} {a}"]);
        n.ready();
        // "what is" was taught before this core and "equals" after it, each for some kind: together they ask the same thing.
        assert_eq!(fact(&n.ask("what is hopyarn 9 equals")), Some("14"));
        // A suffix before the core, or a core that was never taught, is not a known way of asking.
        assert_eq!(n.ask("equals hopyarn 9"), Answer::Refuse);
        assert_eq!(n.ask("what is 9 hopyarn equals"), Answer::Refuse);
    }

    #[test]
    fn a_kind_taught_with_numbers_in_one_form_is_not_asked_in_another() {
        // A kind taught with written numbers is refused for number words, and the other way round, whatever the numbers say.
        let mut n = Novice::default();
        for a in 0..=99 {
            for template in ["what is {s} tehayarn", "tehayarn of {s}", "please tehayarn {s}"] {
                n.teach(&template.replace("{s}", &spelled(a)), &a.to_string());
            }
        }
        teach_plus(&mut n, "bumpyarn", 3, &["what is {w} {a}", "{w} {a}"]);
        n.ready();
        assert_eq!(fact(&n.ask("what is bumpyarn 9")), Some("12"));
        assert_eq!(n.ask("what is bumpyarn nine"), Answer::Refuse);
        assert_eq!(fact(&n.ask("tehayarn of sixty three")), Some("63"));
        assert_eq!(n.ask("tehayarn of 63"), Answer::Refuse);
    }

    #[test]
    fn a_correction_of_a_word_only_fact_is_not_hidden_by_the_cached_frames() {
        // Eight facts each taught in two wordings make "what" a frame word; a ninth, taught in one wording, is then also answered in the other.
        let mut n = Novice::default();
        for i in 0..8 {
            n.teach(&format!("item{i} kind"), &format!("answer{i}"));
            n.teach(&format!("what item{i} kind"), &format!("answer{i}"));
        }
        n.teach("what target kind", "red");
        n.ready();
        assert_eq!(fact(&n.ask("target what kind")), Some("red"));
        assert_eq!(fact(&n.ask("what target kind")), Some("red"));
        n.told("what target kind", "blue", None);
        // The exact wording sees the contradiction; the other wording must not keep stating the old answer as a fact.
        assert_eq!(n.ask("what target kind"), Answer::Refuse);
        assert!(fact(&n.ask("target what kind")).is_none(), "a contradicted fact is not a fact in another wording");
    }

    #[test]
    fn a_word_only_answer_from_one_source_nobody_has_checked_is_a_guess_not_a_fact() {
        let mut n = Novice::default();
        n.teach("color of stone", "grey");
        n.ready();
        assert_eq!(n.ask("color of pebble"), Answer::Refuse);
        n.told("color of pebble", "red", Some("untrusted"));
        match n.ask("color of pebble") {
            Answer::Guess(a, p) => {
                assert_eq!(a, "red");
                assert!(p < 900, "{p}");
            }
            other => panic!("expected a guess, got {}", answer_text(&other)),
        }
        // What a teacher says with no source, or what a lesson says, is still a fact.
        n.told("color of leaf", "green", None);
        assert_eq!(fact(&n.ask("color of leaf")), Some("green"));
    }

    fn spelled(n: usize) -> String {
        let ones = ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen"];
        let tens = ["", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety"];
        match n {
            0..=19 => ones[n].to_string(),
            _ if n % 10 == 0 => tens[n / 10].to_string(),
            _ => format!("{} {}", tens[n / 10], ones[n % 10]),
        }
    }

    #[test]
    fn numbers_spelled_in_several_words_are_read_and_a_worked_out_answer_is_spoken_again() {
        let mut n = Novice::default();
        // A kind that reads a spelled number as digits (what the words mean), and a kind about spelled numbers: three times a number, plus four.
        for a in 0..=99 {
            for template in ["what is {s} tehayarn", "tehayarn of {s}", "please tehayarn {s}"] {
                n.teach(&template.replace("{s}", &spelled(a)), &a.to_string());
            }
        }
        for a in (0..=30).filter(|a| ![12, 17, 27].contains(a)) {
            for template in ["what is {s} serusyarn", "serusyarn of {s}"] {
                n.teach(&template.replace("{s}", &spelled(a)), &spelled(3 * a + 4));
            }
        }
        n.ready();
        // Never taught, with a two-word number in the question and a two-word number in the answer.
        assert_eq!(fact(&n.ask("what is twenty seven serusyarn")), Some("eighty five"));
        assert_eq!(fact(&n.ask("serusyarn of seventeen")), Some("fifty five"));
        // Reading is a fact too.
        assert_eq!(fact(&n.ask("tehayarn of sixty three")), Some("63"));
        // A result with no words in the lessons is refused (3 * 40 + 4 is not a number the lessons spell), and so is a word that is not a number.
        assert_eq!(n.ask("what is forty serusyarn"), Answer::Refuse);
        assert_eq!(n.ask("what is twenty banana serusyarn"), Answer::Refuse);
    }

    const NUMBER_WORDS: [&str; 11] =["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten"];

    /// Teach a kind of two number words, as the static exam does: three wordings of every pair, the answer a word as well.
    fn teach_word_kind(n: &mut Novice, name: &str, pairs: &[(usize, usize)], rule: impl Fn(usize, usize) -> usize) {
        for &(a, b) in pairs {
            for template in ["what is {a} {k} {b}", "{a} {k} {b}", "{a} {k} {b} equals"] {
                n.teach(&template.replace("{a}", NUMBER_WORDS[a]).replace("{k}", name).replace("{b}", NUMBER_WORDS[b]), NUMBER_WORDS[rule(a, b)]);
            }
        }
    }

    fn word_pairs() -> Vec<(usize, usize)> {
        (0..=5).flat_map(|a| (0..=5).map(move |b| (a, b))).filter(|&(a, b)| (a, b) == (0, 0) || (a * 3 + b * 5) % 4 != 0).collect()
    }

    #[test]
    fn words_that_are_numbers_are_found_and_the_learner_works_with_them() {
        let mut n = Novice::default();
        teach_word_kind(&mut n, "bidevubi", &word_pairs(), |a, b| a + b);
        teach_word_kind(&mut n, "dufavubi", &word_pairs(), |a, b| a.abs_diff(b));
        // A kind about things, whose answers are words that repeat and are no numbers.
        for (i, w) in ["bafoba", "tadaba", "podoba", "mifuba", "ligaba", "nufeba", "pefoba", "dageba"].iter().enumerate() {
            for template in ["how is {w} sedavubi", "{w} is sedavubi how", "sedavubi {w}"] {
                n.teach(&template.replace("{w}", w), ["vetupa", "bevapa", "zovapa", "vetupa", "bevapa", "zovapa", "vetupa", "bevapa"][i]);
            }
        }
        n.ready();
        // A pair that was never taught, inside the range: worked out from the program for adding.
        let held = word_pairs();
        let (a, b) = (0..=5).flat_map(|a| (0..=5).map(move |b| (a, b))).find(|p| !held.contains(p) && p.0 + p.1 <= 8).expect("a pair that was not taught");
        match n.ask(&format!("what is {} bidevubi {}", NUMBER_WORDS[a], NUMBER_WORDS[b])) {
            Answer::Fact(w) | Answer::Guess(w, _) => assert_eq!(w, NUMBER_WORDS[a + b], "{a} + {b}"),
            Answer::Refuse => panic!("refused {a} + {b}"),
        }
        // The second kind, explained once the words are numbers.
        match n.ask("what is five dufavubi one") {
            Answer::Fact(w) | Answer::Guess(w, _) => assert_eq!(w, "four"),
            Answer::Refuse => panic!("refused five dufavubi one"),
        }
        // Beyond the numbers taught as arguments: six is only ever an answer, and is a number all the same.
        match n.ask("what is zero bidevubi six") {
            Answer::Fact(w) | Answer::Guess(w, _) => assert_eq!(w, "six"),
            Answer::Refuse => panic!("refused zero bidevubi six"),
        }
        // A result with no word is not spoken: six and six make a number that no lesson has a word for.
        assert_eq!(n.ask("what is six bidevubi six"), Answer::Refuse);
        // Things are still things.
        assert_eq!(fact(&n.ask("how is bafoba sedavubi")), Some("vetupa"));
    }

    #[test]
    fn without_a_kind_that_adds_no_word_is_a_number() {
        let mut n = Novice::default();
        teach_word_kind(&mut n, "dudovubi", &word_pairs(), |a, b| a.max(b));
        n.ready();
        assert_eq!(n.ask("what is three dudovubi three"), Answer::Refuse);
    }

    #[test]
    fn a_fact_asked_in_a_frame_it_was_not_taught_in_is_answered_as_surely_as_the_frame_has_earned() {
        let mut n = Novice::default();
        for (i, e) in ["bafoba", "tadaba", "podoba", "mifuba", "ligaba", "nufeba", "pefoba", "dageba", "kuvaba", "retiba", "gobaba", "zinuba"].iter().enumerate() {
            let (a, b) = (format!("ta{i}pa"), format!("tu{i}pa"));
            for q in [format!("how is {e} sedavubi"), format!("{e} is sedavubi how"), format!("sedavubi {e}")] {
                n.teach(&q, &a);
            }
            for q in [format!("what is the madovubi of {e}"), format!("madovubi of {e}"), format!("the madovubi of {e}")] {
                n.teach(&q, &b);
            }
            if i < 8 {
                for q in [format!("name the sedovubi of {e}"), format!("the sedovubi of {e}"), format!("sedovubi {e}")] {
                    n.teach(&q, &format!("to{i}pa"));
                }
            }
        }
        n.ready();
        // "name" was seen to change nothing in eight pairs, "what" in many more: new to the fact, they are as sure as that.
        match n.ask("name the madovubi of podoba") {
            Answer::Fact(a) | Answer::Guess(a, _) => assert_eq!(a, "tu2pa"),
            Answer::Refuse => panic!("refused a fact in a new frame"),
        }
        match n.ask("what is mifuba sedavubi") {
            Answer::Fact(a) | Answer::Guess(a, _) => assert_eq!(a, "ta3pa"),
            Answer::Refuse => panic!("refused a fact in a new frame"),
        }
        // Not facts the lessons have: two kinds at once, a thing nobody knows, a kind taught for other things only.
        assert_eq!(n.ask("what is the sedavubi of podoba madovubi"), Answer::Refuse);
        assert_eq!(n.ask("what is the madovubi of kufoba"), Answer::Refuse);
        assert_eq!(n.ask("what is the sedovubi of zinuba"), Answer::Refuse);
    }

    #[test]
    fn a_random_staircase_is_not_a_floor_of_a_line() {
        // Rising, but by steps of 0 to 3 drawn at random: monotone, and no line floors to it.
        let mut seed = 777u64;
        let mut total = 0i64;
        let table: Vec<i64> = (0..=20)
            .map(|_| {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                total += ((seed >> 33) % 4) as i64;
                total
            })
            .collect();
        let mut n = Novice::default();
        teach_unary(&mut n, "stairmosido", |x| table[x as usize], 0..=20);
        n.ready();
        assert_eq!(n.kinds(), (1, 0), "a random staircase has no rule");
        assert_eq!(n.ask("what is stairmosido 27"), Answer::Refuse);
    }

    #[test]
    fn arbitrary_codes_have_no_shape_to_search_with() {
        let mut seed = 99u64;
        let codes: Vec<i64> = (0..20)
            .map(|_| {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                ((seed >> 33) % 100) as i64
            })
            .collect();
        let inputs: Vec<Vec<i64>> = (0..20).map(|x| vec![x]).collect();
        assert!(shape_constants(&inputs, &codes).is_none());
        // A line with a knee does.
        let target: Vec<i64> = (0..20).map(|x: i64| x.min(9) + 2 * (x - 9).max(0)).collect();
        let found = shape_constants(&inputs, &target).expect("a knee is a shape");
        assert!(found.contains(&9) || found.contains(&10), "{found:?}");
    }
}
