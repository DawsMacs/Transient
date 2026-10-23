// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! Building a rule instead of choosing one.
//!
//! The rule and select mechanisms are fixed forms with numbers to fit. This one searches for the form. It enumerates tiny
//! expressions over the numbers in a question (the first number, the second, the constants 1 and 2; add, subtract, multiply,
//! the bigger, the smaller, how far apart, and half), smallest first, and keeps the first that reproduces every answer it was
//! taught. Expressions that compute the same thing on the taught questions are kept once (the shortest). Whether to *trust* a
//! program is not decided here: the baby tests it on facts it held back, like every other mechanism, which is also what stops
//! it from believing a coincidence.

use crate::data::tokenize;
use crate::kinds::Item;
use crate::quantity::{counting_lessons, NumberLine};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Max,
    Min,
    AbsDiff,
    /// Whole-number division, rounded down (not defined for a divisor of zero or less).
    Div,
    /// What is left over after dividing (not defined for a divisor of zero or less).
    Mod,
}

#[derive(Clone, Debug)]
pub enum Expr {
    A,
    B,
    K(i32),
    Bin(Op, Box<Expr>, Box<Expr>),
    Half(Box<Expr>),
}

impl Expr {
    /// Number of pieces in the expression.
    pub fn size(&self) -> usize {
        match self {
            Expr::A | Expr::B | Expr::K(_) => 1,
            Expr::Half(e) => 1 + e.size(),
            Expr::Bin(_, l, r) => 1 + l.size() + r.size(),
        }
    }

    pub fn eval(&self, a: f32, b: f32) -> f32 {
        match self {
            Expr::A => a,
            Expr::B => b,
            Expr::K(k) => *k as f32,
            Expr::Half(e) => e.eval(a, b) / 2.0,
            Expr::Bin(op, l, r) => apply(*op, l.eval(a, b), r.eval(a, b)),
        }
    }

    pub fn show(&self) -> String {
        match self {
            Expr::A => "first".into(),
            Expr::B => "second".into(),
            Expr::K(k) => k.to_string(),
            Expr::Half(e) => format!("half({})", e.show()),
            Expr::Bin(op, l, r) => {
                let sym = match op {
                    Op::Add => "+",
                    Op::Sub => "-",
                    Op::Mul => "x",
                    Op::Max => "max",
                    Op::Min => "min",
                    Op::AbsDiff => "apart",
                    Op::Div => "div",
                    Op::Mod => "mod",
                };
                format!("({} {} {})", l.show(), sym, r.show())
            }
        }
    }
}

impl Expr {
    /// This expression with `first` replaced by `a` and `second` by `b` (a library function applied to something).
    pub fn subst(&self, a: &Expr, b: &Expr) -> Expr {
        match self {
            Expr::A => a.clone(),
            Expr::B => b.clone(),
            Expr::K(k) => Expr::K(*k),
            Expr::Half(e) => Expr::Half(Box::new(e.subst(a, b))),
            Expr::Bin(op, l, r) => Expr::Bin(*op, Box::new(l.subst(a, b)), Box::new(r.subst(a, b))),
        }
    }
}

impl Expr {
    /// A saved form with no spaces inside words: (mul first second), (half first), K1.
    pub fn to_text(&self) -> String {
        match self {
            Expr::A => "A".into(),
            Expr::B => "B".into(),
            Expr::K(k) => format!("K{k}"),
            Expr::Half(e) => format!("(half {})", e.to_text()),
            Expr::Bin(op, l, r) => {
                let name = match op {
                    Op::Add => "add",
                    Op::Sub => "sub",
                    Op::Mul => "mul",
                    Op::Max => "max",
                    Op::Min => "min",
                    Op::AbsDiff => "apart",
                    Op::Div => "div",
                    Op::Mod => "mod",
                };
                format!("({name} {} {})", l.to_text(), r.to_text())
            }
        }
    }

    pub fn from_text(text: &str) -> Option<Expr> {
        let spaced = text.replace('(', " ( ").replace(')', " ) ");
        let tokens: Vec<&str> = spaced.split_whitespace().collect();
        let (e, used) = Expr::parse(&tokens, 0)?;
        if used == tokens.len() { Some(e) } else { None }
    }

    fn parse(t: &[&str], at: usize) -> Option<(Expr, usize)> {
        match *t.get(at)? {
            "A" => Some((Expr::A, at + 1)),
            "B" => Some((Expr::B, at + 1)),
            k if k.starts_with('K') => Some((Expr::K(k[1..].parse().ok()?), at + 1)),
            "(" => {
                let name = *t.get(at + 1)?;
                if name == "half" {
                    let (e, next) = Expr::parse(t, at + 2)?;
                    return if *t.get(next)? == ")" { Some((Expr::Half(Box::new(e)), next + 1)) } else { None };
                }
                let op = match name {
                    "add" => Op::Add,
                    "sub" => Op::Sub,
                    "mul" => Op::Mul,
                    "max" => Op::Max,
                    "min" => Op::Min,
                    "apart" => Op::AbsDiff,
                    "div" => Op::Div,
                    "mod" => Op::Mod,
                    _ => return None,
                };
                let (l, n1) = Expr::parse(t, at + 2)?;
                let (r, n2) = Expr::parse(t, n1)?;
                if *t.get(n2)? == ")" { Some((Expr::Bin(op, Box::new(l), Box::new(r)), n2 + 1)) } else { None }
            }
            _ => None,
        }
    }
}

fn apply(op: Op, x: f32, y: f32) -> f32 {
    match op {
        Op::Add => x + y,
        Op::Sub => x - y,
        Op::Mul => x * y,
        Op::Max => x.max(y),
        Op::Min => x.min(y),
        Op::AbsDiff => (x - y).abs(),
        Op::Div => if y > 0.0 { (x / y).floor() } else { f32::NAN },
        Op::Mod => if y > 0.0 { x - y * (x / y).floor() } else { f32::NAN },
    }
}

/// How many genuinely different behaviours these expressions have: two that agree on every pair of numbers from 0 to 10 are one.
fn behaviours(exprs: &[Expr]) -> usize {
    let mut seen: HashSet<Vec<i32>> = HashSet::new();
    for e in exprs {
        // As wide as the numbers it will meet (taught small, asked large): two expressions that agree on 0-10 but part ways at 20 are two.
        let mut sig = Vec::with_capacity(31 * 31);
        for a in 0..=30 {
            for b in 0..=30 {
                let v = e.eval(a as f32, b as f32);
                sig.push(if v.is_finite() { (v * 100.0).round() as i32 } else { i32::MIN });
            }
        }
        seen.insert(sig);
    }
    seen.len()
}

/// What the baby has learned to build with, beyond the operations it was given. Two kinds, both bounded:
///
/// * **constants** that proved useful: an integer coefficient or bias of at least 3 in a rule it learned ("plus ten" has a ten in it)
///   becomes a one-piece terminal for later searches (the base language can only make 3 from 1 + 2, and 10 not at all within its limit);
/// * **functions**: a program it built for a reliable unit becomes callable as one piece, however big it was. A search for a later unit
///   that needs "the sum of squares, plus one" then finds it at 4 pieces instead of 9.
///
/// A function is inlined into whatever program uses it, so a saved program is always a plain expression. The library is small on
/// purpose (6 constants, 8 functions, oldest dropped first) and only reliable units add to it.
#[derive(Clone, Default, Debug)]
pub struct Library {
    pub consts: Vec<i32>,
    pub funcs: Vec<Func>,
    /// (unit, behaviour) pairs already contributed, kept even after the function is evicted: a unit that refits to an equivalent program (or to a
    /// rewrite of it through another unit's function) adds nothing. Without this a library with fewer slots than program units thrashes: each
    /// refit re-adds its unit and evicts another, whose unit refits and evicts the next, and the entries grow into nested equivalents.
    contributed: Vec<(usize, Vec<i32>)>,
}

#[derive(Clone, Debug)]
pub struct Func {
    pub expr: Expr,
    pub arity: usize,
    /// The unit it was learned from (`NO_ORIGIN` if none): a unit is never explained by vocabulary it contributed itself.
    pub origin: usize,
}

pub const NO_ORIGIN: usize = usize::MAX;

impl PartialEq for Library {
    fn eq(&self, other: &Library) -> bool {
        self.to_lines() == other.to_lines()
    }
}

const MAX_CONSTS: usize = 6;
const MAX_FUNCS: usize = 8;

impl Library {
    /// The library as a unit may use it: without what that unit itself contributed.
    pub fn without(&self, origin: usize) -> Library {
        Library { consts: self.consts.clone(), funcs: self.funcs.iter().filter(|f| f.origin != origin).cloned().collect(), contributed: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.consts.is_empty() && self.funcs.is_empty()
    }

    /// A number that proved useful. Returns whether it was new.
    pub fn learn_constant(&mut self, c: i32) -> bool {
        if !(3..=1000).contains(&c) || self.consts.contains(&c) {
            return false;
        }
        if self.consts.len() >= MAX_CONSTS {
            self.consts.remove(0);
        }
        self.consts.push(c);
        true
    }

    /// Learn the constants in a fitted rule: every integer, at least 3, among the coefficients it found. `terms` are
    /// (first, second, bias) in the units of the answer.
    pub fn learn_from_terms(&mut self, terms: &[[f32; 3]]) {
        for t in terms {
            for v in t {
                let r = v.round();
                if (v - r).abs() < 0.05 {
                    self.learn_constant(r.abs() as i32);
                }
            }
        }
    }

    /// A program it built, as something to build with. Only if it is a bigger step than a single operation on the numbers themselves
    /// (`first x second` is already one piece; `first div 10` is not, it carries a ten), and only if it does something no function it
    /// already has does.
    pub fn learn_program(&mut self, expr: &Expr, arity: usize, origin: usize) -> bool {
        let variable = |e: &Expr| matches!(e, Expr::A | Expr::B);
        if expr.size() < 3 || matches!(expr, Expr::Bin(_, l, r) if variable(l) && variable(r)) {
            return false;
        }
        let sig = ProgramRule::grid_signature(expr, arity);
        if self.funcs.iter().any(|f| ProgramRule::grid_signature(&f.expr, f.arity) == sig) {
            return false;
        }
        if origin != NO_ORIGIN {
            if self.contributed.iter().any(|(o, g)| *o == origin && *g == sig) {
                return false;
            }
            self.contributed.push((origin, sig.clone()));
        }
        if self.funcs.len() >= MAX_FUNCS {
            self.funcs.remove(0);
        }
        self.funcs.push(Func { expr: expr.clone(), arity, origin });
        true
    }

    /// Saved as lines: `library <constants...>`, then `func <arity> <expression>` per function.
    pub fn to_lines(&self) -> String {
        let mut out = format!("library {}\n", self.consts.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" "));
        for f in &self.funcs {
            out.push_str(&format!("func {} {} {}\n", f.arity, if f.origin == NO_ORIGIN { -1 } else { f.origin as i64 }, f.expr.to_text()));
        }
        out
    }

    /// Read what `to_lines` wrote: the `library` line, then however many `func` lines follow.
    pub fn from_lines<'a>(lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>) -> Option<Library> {
        let head = lines.peek()?.strip_prefix("library")?;
        let consts = head.split_whitespace().filter_map(|c| c.parse().ok()).collect();
        lines.next();
        let mut lib = Library { consts, funcs: Vec::new(), contributed: Vec::new() };
        while let Some(l) = lines.peek() {
            let Some(rest) = l.strip_prefix("func ") else { break };
            let mut parts = rest.splitn(3, ' ');
            let (arity, origin, text) = (parts.next()?, parts.next()?, parts.next()?);
            let origin: i64 = origin.parse().ok()?;
            lib.funcs.push(Func { expr: Expr::from_text(text)?, arity: arity.parse().ok()?, origin: if origin < 0 { NO_ORIGIN } else { origin as usize } });
            lines.next();
        }
        Some(lib)
    }
}

const OPS: [Op; 8] = [Op::Add, Op::Sub, Op::Mul, Op::Max, Op::Min, Op::AbsDiff, Op::Div, Op::Mod];
const MAX_SIZE: usize = 7;
const CAP_PER_LEVEL: usize = 40_000;
/// How many candidate expressions one search may evaluate before it gives up. A search that finds nothing would otherwise try every
/// combination of everything smaller; with a library of constants and functions that is minutes. Bounded effort is part of being light.
/// Bits of information the lessons must carry beyond the bits of searching spent finding the program.
const INFO_MARGIN_BITS: f32 = 1.0;
/// The most different explanations that may fit the lessons before they are taken to say nothing.
const MAX_AMBIGUITY: usize = 6;
const WORK_BUDGET: usize = 1_000_000;
/// A partial program may leave at most this share of the lessons unexplained, and only for a unit with at least `MIN_PARTIAL_LESSONS` of them.
const PARTIAL_SHARE: f32 = 0.2;
const MIN_PARTIAL_LESSONS: usize = 24;

struct Cand {
    expr: Expr,
    vals: Vec<f32>,
}

fn signature(vals: &[f32]) -> Vec<i32> {
    vals.iter().map(|v| (v * 100.0).round() as i32).collect()
}

/// A 64-bit fingerprint of `signature(vals)` without building it (FNV-1a over the same rounded integers): two expressions that behave alike on the lessons
/// get the same one. The search asks this of every candidate it makes, and nearly all are duplicates that are thrown away.
fn fingerprint(vals: &[f32]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in vals {
        h ^= ((v * 100.0).round() as i32) as u32 as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h ^ (vals.len() as u64)
}

pub struct ProgramRule {
    pub expr: Expr,
    /// Other explanations (of the smallest size) that fit every lesson but behave differently somewhere. The program answers only
    /// where all of them agree: where they part ways the lessons do not decide the answer.
    pub alts: Vec<Expr>,
    /// How many candidate expressions had been tried when this one fit (the effort of the search, for judging whether the lessons carry
    /// enough information to believe it).
    pub work: usize,
    /// None: the program reproduces every lesson. Some(share): it reproduces only that share of them and the rest are exceptions; such a rule
    /// is fitted only when no exact program exists, and its answers are guesses, never facts.
    pub partial: Option<f32>,
    /// The questions whose lessons this program was repaired to fit (see `fit_repair_with`): a program built around a contradiction is
    /// only as good as one point, and the point is consistent with many laws (a cap at 24 explains 24 + 12 = 24 as well as one at 30 would
    /// explain 30), so those lessons are told facts but never vouch for the questions below them.
    pub repaired: Vec<String>,
    arity: usize,
    line: NumberLine,
    vocab: HashSet<String>,
}

impl ProgramRule {
    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn vocab(&self) -> Vec<String> {
        let mut v: Vec<String> = self.vocab.iter().cloned().collect();
        v.sort();
        v
    }

    /// Rebuild a program that was saved as text.
    pub fn rebuild(expr: Expr, arity: usize, vocab: Vec<String>) -> ProgramRule {
        ProgramRule { expr, alts: Vec::new(), partial: None, repaired: Vec::new(), work: 0, arity, line: NumberLine::from_counting(&counting_lessons()), vocab: vocab.into_iter().collect() }
    }

    fn read(line: &NumberLine, q: &str) -> (Vec<f32>, Vec<String>) {
        let mut nums = Vec::new();
        let mut rest = Vec::new();
        for w in tokenize(q) {
            match line.value(&w) {
                Some(v) => nums.push(v),
                None => rest.push(w),
            }
        }
        (nums, rest)
    }

    /// Search for the smallest expression that reproduces every taught answer. None if there is none within the size limit.
    pub fn fit(items: &[Item]) -> Option<ProgramRule> {
        ProgramRule::fit_with(items, &OPS)
    }

    /// As `fit_unique`, building with what has been learned (`lib`) as well as the given operations.
    #[cfg(test)]
    pub fn search_pub(items: &[Item], lib: &Library) -> (Option<ProgramRule>, usize, Vec<Expr>) {
        ProgramRule::search(items, &OPS, true, lib, 0.0)
    }

    /// The program for these lessons, with the other smallest explanations that also fit kept as `alts` (answers are given only where
    /// they all agree). More than a handful of different explanations means the lessons say next to nothing: no program then.
    pub fn fit_trusted_with(items: &[Item], lib: &Library) -> Option<ProgramRule> {
        let own = items.first().map_or(NO_ORIGIN, |i| i.unit);
        let (found, hits, exprs) = ProgramRule::search(items, &OPS, true, &lib.without(own), 0.0);
        // no program fits every lesson: real data is a majority that follows one relation and a minority that does not
        let Some(mut rule) = found else { return ProgramRule::fit_partial_with(items, lib) };
        if hits == 0 || hits > MAX_AMBIGUITY {
            return None;
        }
        let mut seen: HashSet<Vec<i32>> = HashSet::new();
        seen.insert(ProgramRule::grid_signature(&rule.expr, rule.arity));
        for e in exprs {
            if seen.insert(ProgramRule::grid_signature(&e, rule.arity)) {
                rule.alts.push(e);
            }
        }
        rule.is_believable(items).then_some(rule)
    }

    /// An exact program for lessons a rule has been shown wrong on, allowed the numbers that the contradicting lessons themselves point to
    /// (`extra`: the answer they gave, and how far the old rule was from it: a cap of 30 or a wrap at 40 is not made of 1 and 2). The extra
    /// numbers are terminals for this one search only and are never added to the library. It has to fit EVERY lesson and clear the same
    /// believability and ambiguity bars as any program; otherwise None, and nothing changes.
    pub fn fit_repair_with(items: &[Item], lib: &Library, extra: &[i32]) -> Option<ProgramRule> {
        let own = items.first().map_or(NO_ORIGIN, |i| i.unit);
        let mut lib = lib.without(own);
        for &c in extra {
            if !lib.consts.contains(&c) {
                lib.consts.push(c);
            }
        }
        let (found, hits, exprs) = ProgramRule::search(items, &OPS, true, &lib, 0.0);
        let mut rule = found?;
        if hits == 0 || hits > MAX_AMBIGUITY {
            return None;
        }
        let mut seen: HashSet<Vec<i32>> = HashSet::new();
        seen.insert(ProgramRule::grid_signature(&rule.expr, rule.arity));
        for e in exprs {
            if seen.insert(ProgramRule::grid_signature(&e, rule.arity)) {
                rule.alts.push(e);
            }
        }
        rule.is_believable(items).then_some(rule)
    }

    /// A program that reproduces most but not all of the lessons (at most `PARTIAL_SHARE` of them unexplained), for a unit where no exact
    /// program exists. It must still carry more information than it cost: the bits the explained lessons hold, minus the bits it takes to
    /// say which lessons are the exceptions, must exceed the bits of searching. A constant that fits the 80 % of a primality table that is
    /// "no" fails that; a relation that fits 150 different sums out of 175 passes. Its answers are guesses (`partial`).
    fn fit_partial_with(items: &[Item], lib: &Library) -> Option<ProgramRule> {
        if items.len() < MIN_PARTIAL_LESSONS {
            return None;
        }
        let own = items.first().map_or(NO_ORIGIN, |i| i.unit);
        let (found, hits, exprs) = ProgramRule::search(items, &OPS, true, &lib.without(own), PARTIAL_SHARE);
        let mut rule = found?;
        if hits == 0 || hits > MAX_AMBIGUITY {
            return None;
        }
        // which lessons does it explain?
        let line = NumberLine::from_counting(&counting_lessons());
        let mut explained_inputs: HashSet<(i32, i32)> = HashSet::new();
        let mut answers: HashSet<i32> = HashSet::new();
        let (mut explained, mut total) = (0usize, 0usize);
        for it in items {
            total += 1;
            let (nums, _) = ProgramRule::read(&line, &it.query);
            let Some(y) = line.value(&it.answer) else { continue };
            if nums.len() != rule.arity {
                continue;
            }
            let (a, b) = (nums[0], *nums.get(1).unwrap_or(&0.0));
            let v = rule.expr.eval(a, b);
            if v.is_finite() && (v - y).abs() < 0.01 {
                explained += 1;
                explained_inputs.insert((a as i32, b as i32));
                answers.insert(y as i32);
            }
        }
        let share = explained as f32 / total.max(1) as f32;
        if explained == total || share < 1.0 - PARTIAL_SHARE {
            return None; // (all explained: the exact search would have found it; too few: it is not a relation)
        }
        let mut seen: HashSet<Vec<i32>> = HashSet::new();
        seen.insert(ProgramRule::grid_signature(&rule.expr, rule.arity));
        for e in exprs {
            if seen.insert(ProgramRule::grid_signature(&e, rule.arity)) {
                rule.alts.push(e);
            }
        }
        let bits_per_answer = (answers.len().max(2) as f32).log2();
        let data_bits = explained_inputs.len() as f32 * bits_per_answer;
        let exception_bits = (total - explained) as f32 * ((total as f32).log2() + bits_per_answer);
        let search_bits = (rule.work.max(1) as f32).log2();
        if data_bits < search_bits + exception_bits + INFO_MARGIN_BITS {
            return None;
        }
        rule.partial = Some(share);
        Some(rule)
    }

    /// Mark an exact program as provisional: it is the best explanation the lessons give, but the baby has seen it fail too often on facts it held back to
    /// state anything as fact, so its answers are guesses only (the same flag as a program that fits most lessons). `partial` becomes the share of lessons it explains.
    pub fn make_provisional(&mut self, items: &[Item]) {
        if self.partial.is_some() {
            return;
        }
        let line = NumberLine::from_counting(&counting_lessons());
        let (mut explained, mut total) = (0usize, 0usize);
        for it in items {
            total += 1;
            let (nums, _) = ProgramRule::read(&line, &it.query);
            let Some(y) = line.value(&it.answer) else { continue };
            if nums.len() != self.arity {
                continue;
            }
            let v = self.expr.eval(nums[0], *nums.get(1).unwrap_or(&0.0));
            if v.is_finite() && (v - y).abs() < 0.01 {
                explained += 1;
            }
        }
        self.partial = Some((explained as f32 / total.max(1) as f32).min(0.999));
    }

    pub fn fit_unique_with(items: &[Item], lib: &Library) -> Option<ProgramRule> {
        let own = items.first().map_or(NO_ORIGIN, |i| i.unit);
        let (found, hits, _) = ProgramRule::search(items, &OPS, true, &lib.without(own), 0.0);
        if hits != 1 {
            return None;
        }
        found.filter(|p| p.is_believable(items))
    }

    /// The same search with a chosen set of operations (to see what it can compose when a convenient one is taken away).
    pub fn fit_with(items: &[Item], ops: &[Op]) -> Option<ProgramRule> {
        ProgramRule::search(items, ops, false, &Library::default(), 0.0).0
    }

    /// Only if exactly ONE smallest expression fits. When two different expressions of the same size reproduce every lesson (for
    /// instance `first apart second` and `second - first`, which agree whenever the second number is the larger), the data cannot
    /// say which one is meant, and nothing should be built on the guess.
    pub fn fit_unique(items: &[Item]) -> Option<ProgramRule> {
        ProgramRule::fit_unique_with(items, &Library::default())
    }

    /// The search. With `count_all`, it finishes the size at which the first expression fits and reports how many fit there.
    fn search(items: &[Item], ops: &[Op], count_all: bool, lib: &Library, tolerance: f32) -> (Option<ProgramRule>, usize, Vec<Expr>) {
        let line = NumberLine::from_counting(&counting_lessons());
        let mut points: Vec<(f32, f32, f32)> = Vec::new();
        let mut vocab = HashSet::new();
        let mut arity = 0;
        let mut skipped = 0usize; // lessons this kind of program cannot read at all (tolerant search only)
        for it in items {
            let (nums, rest) = ProgramRule::read(&line, &it.query);
            let y = match line.value(&it.answer) {
                Some(y) => y,
                None if tolerance > 0.0 => {
                    skipped += 1;
                    continue;
                }
                None => return (None, 0, Vec::new()),
            };
            if nums.is_empty() || nums.len() > 2 {
                if tolerance > 0.0 {
                    skipped += 1;
                    continue;
                }
                return (None, 0, Vec::new());
            }
            arity = arity.max(nums.len());
            let p = (nums[0], *nums.get(1).unwrap_or(&0.0), y);
            if !points.contains(&p) {
                points.push(p);
            }
            vocab.extend(rest);
        }
        // The same numbers with two different answers (one phrasing of a fact was taught wrongly): the plurality of the lessons decides,
        // and a tie drops the point. No program can fit both, so without this one wrong lesson makes everything unexplainable.
        let mut by_input: Vec<((f32, f32), Vec<(f32, usize)>)> = Vec::new();
        for it in items {
            let (nums, _) = ProgramRule::read(&line, &it.query);
            if nums.is_empty() || nums.len() > 2 || line.value(&it.answer).is_none() {
                continue; // (only possible in a tolerant search: the others returned above)
            }
            let y = line.value(&it.answer).unwrap_or(0.0);
            let key = (nums[0], *nums.get(1).unwrap_or(&0.0));
            match by_input.iter_mut().find(|(k, _)| *k == key) {
                Some((_, ys)) => match ys.iter_mut().find(|(v, _)| *v == y) {
                    Some(c) => c.1 += 1,
                    None => ys.push((y, 1)),
                },
                None => by_input.push((key, vec![(y, 1)])),
            }
        }
        points = by_input
            .into_iter()
            .filter_map(|(k, mut ys)| {
                ys.sort_by(|a, b| b.1.cmp(&a.1));
                (ys.len() == 1 || ys[0].1 > ys[1].1).then(|| (k.0, k.1, ys[0].0))
            })
            .collect();
        if points.is_empty() {
            return (None, 0, Vec::new());
        }
        let mut first: Option<ProgramRule> = None;
        let mut hits_n = 0usize;
        let mut work = 0usize;
        let mut hit_exprs: Vec<Expr> = Vec::new();
        let ys: Vec<f32> = points.iter().map(|p| p.2).collect();
        // how many points an expression may get wrong and still count as fitting: none, unless this is the tolerant search
        let budget = if tolerance > 0.0 { ((tolerance * (points.len() + skipped) as f32).floor() as usize).saturating_sub(skipped) } else { 0 };
        if tolerance > 0.0 && (skipped as f32) > tolerance * (points.len() + skipped) as f32 {
            return (None, 0, Vec::new());
        }
        let hits = |vals: &[f32]| {
            let mut wrong = 0usize;
            for (v, y) in vals.iter().zip(&ys) {
                if (v - y).abs() >= 0.01 {
                    wrong += 1;
                    if wrong > budget {
                        return false;
                    }
                }
            }
            true
        };

        let mut seen: HashSet<u64> = HashSet::new();
        let mut scratch: Vec<f32> = Vec::with_capacity(points.len());
        let mut levels: Vec<Vec<Cand>> = (0..=MAX_SIZE).map(|_| Vec::new()).collect();
        let mut terminals = vec![
            Cand { expr: Expr::A, vals: points.iter().map(|p| p.0).collect() },
            Cand { expr: Expr::K(1), vals: vec![1.0; points.len()] },
            Cand { expr: Expr::K(2), vals: vec![2.0; points.len()] },
        ];
        if arity == 2 {
            terminals.push(Cand { expr: Expr::B, vals: points.iter().map(|p| p.1).collect() });
        }
        for &k in &lib.consts {
            terminals.push(Cand { expr: Expr::K(k), vals: vec![k as f32; points.len()] });
        }
        for c in terminals {
            if hits(&c.vals) {
                hits_n += 1;
                hit_exprs.push(c.expr.clone());
                if first.is_none() {
                    first = Some(ProgramRule { expr: c.expr.clone(), arity, line: NumberLine::from_counting(&counting_lessons()), vocab: vocab.clone(), work, alts: Vec::new(), partial: None, repaired: Vec::new() });
                }
                if !count_all {
                    return (first, behaviours(&hit_exprs), hit_exprs);
                }
            }
            if seen.insert(fingerprint(&c.vals)) {
                levels[1].push(c);
            }
        }
        if first.is_some() {
            return (first, behaviours(&hit_exprs), hit_exprs);
        }
        for size in 2..=MAX_SIZE {
            let mut made: Vec<Cand> = Vec::new();
            // Half of a smaller expression.
            for c in &levels[size - 1] {
                scratch.clear();
                scratch.extend(c.vals.iter().map(|v| v / 2.0));
                if hits(&scratch) {
                    hits_n += 1;
                    hit_exprs.push(Expr::Half(Box::new(c.expr.clone())));
                    if first.is_none() {
                        first = Some(ProgramRule { expr: Expr::Half(Box::new(c.expr.clone())), arity, line: NumberLine::from_counting(&counting_lessons()), vocab: vocab.clone(), work, alts: Vec::new(), partial: None, repaired: Vec::new() });
                    }
                    if !count_all {
                        return (first, behaviours(&hit_exprs), hit_exprs);
                    }
                }
                if seen.insert(fingerprint(&scratch)) {
                    made.push(Cand { expr: Expr::Half(Box::new(c.expr.clone())), vals: scratch.clone() });
                }
            }
            // Two smaller expressions joined by an operation.
            for left_size in 1..size - 1 {
                let right_size = size - 1 - left_size;
                if right_size < 1 {
                    continue;
                }
                for l in &levels[left_size] {
                    for r in &levels[right_size] {
                        for &op in ops {
                            work += 1;
                            if work > WORK_BUDGET {
                                return (first, behaviours(&hit_exprs), hit_exprs);
                            }
                            scratch.clear();
                            scratch.extend(l.vals.iter().zip(&r.vals).map(|(x, y)| apply(op, *x, *y)));
                            if scratch.iter().any(|v| !v.is_finite()) {
                                continue;
                            }
                            if hits(&scratch) {
                                hits_n += 1;
                                hit_exprs.push(Expr::Bin(op, Box::new(l.expr.clone()), Box::new(r.expr.clone())));
                                if first.is_none() {
                                    let expr = Expr::Bin(op, Box::new(l.expr.clone()), Box::new(r.expr.clone()));
                                    first = Some(ProgramRule { expr, arity, line: NumberLine::from_counting(&counting_lessons()), vocab: vocab.clone(), work, alts: Vec::new(), partial: None, repaired: Vec::new() });
                                }
                                if !count_all {
                                    return (first, behaviours(&hit_exprs), hit_exprs);
                                }
                            }
                            if made.len() < CAP_PER_LEVEL && seen.insert(fingerprint(&scratch)) {
                                made.push(Cand { expr: Expr::Bin(op, Box::new(l.expr.clone()), Box::new(r.expr.clone())), vals: scratch.clone() });
                            }
                        }
                    }
                }
            }
            // A learned function applied to smaller expressions costs one piece, however big the program it came from.
            for f in &lib.funcs {
                let splits: Vec<(usize, usize)> = if f.arity == 1 { vec![(size - 1, 0)] } else { (1..size - 1).map(|l| (l, size - 1 - l)).filter(|(_, r)| *r >= 1).collect() };
                for (ls, rs) in splits {
                    for (li, l) in levels[ls].iter().enumerate() {
                        let rights: &[Cand] = if f.arity == 1 { &levels[ls][li..li + 1] } else { &levels[rs] };
                        for r in rights {
                            work += 1;
                            if work > WORK_BUDGET {
                                return (first, behaviours(&hit_exprs), hit_exprs);
                            }
                            scratch.clear();
                            if f.arity == 1 {
                                scratch.extend(l.vals.iter().map(|x| f.expr.eval(*x, 0.0)));
                            } else {
                                scratch.extend(l.vals.iter().zip(&r.vals).map(|(x, y)| f.expr.eval(*x, *y)));
                            }
                            if scratch.iter().any(|v| !v.is_finite()) {
                                continue;
                            }
                            let expr = f.expr.subst(&l.expr, &r.expr);
                            if hits(&scratch) {
                                hits_n += 1;
                                hit_exprs.push(expr.clone());
                                if first.is_none() {
                                    first = Some(ProgramRule { expr: expr.clone(), arity, line: NumberLine::from_counting(&counting_lessons()), vocab: vocab.clone(), work, alts: Vec::new(), partial: None, repaired: Vec::new() });
                                }
                                if !count_all {
                                    return (first, behaviours(&hit_exprs), hit_exprs);
                                }
                            }
                            if made.len() < CAP_PER_LEVEL && seen.insert(fingerprint(&scratch)) {
                                made.push(Cand { expr, vals: scratch.clone() });
                            }
                        }
                    }
                }
            }
            levels[size] = made;
            if first.is_some() {
                return (first, behaviours(&hit_exprs), hit_exprs);
            }
            if work > WORK_BUDGET {
                break;
            }
        }
        (None, 0, Vec::new())
    }

    /// Where this program and its own competing explanations part ways (a repaired program's alternatives are built with numbers the library does not
    /// have, so `ambiguity` cannot find them again): up to three untaught inputs, the nearest to the taught range first among those where the most
    /// explanations differ.
    pub fn own_ambiguity(&self, items: &[Item]) -> Option<Ambiguity> {
        if self.alts.is_empty() {
            return None;
        }
        let arity = self.arity;
        let mut seen: HashSet<Vec<i32>> = HashSet::new();
        let mut hypotheses: Vec<Expr> = Vec::new();
        for e in std::iter::once(&self.expr).chain(self.alts.iter()) {
            if seen.insert(ProgramRule::grid_signature(e, arity)) && hypotheses.len() < 8 {
                hypotheses.push(e.clone());
            }
        }
        let taught: HashSet<(i32, i32)> = points_of(items).iter().map(|p| (p.0 as i32, p.1 as i32)).collect();
        let mut scored: Vec<((i32, i32), usize)> = Vec::new();
        for a in 0..=30i32 {
            for b in 0..=if arity == 2 { 30 } else { 0 } {
                if taught.contains(&(a, b)) {
                    continue;
                }
                let answers: HashSet<i32> = hypotheses.iter().map(|e| { let v = e.eval(a as f32, b as f32); if v.is_finite() { (v * 100.0).round() as i32 } else { i32::MIN } }).collect();
                if answers.len() >= 2 {
                    scored.push(((a, b), answers.len()));
                }
            }
        }
        scored.sort_by(|x, y| y.1.cmp(&x.1).then((x.0 .0 + x.0 .1).cmp(&(y.0 .0 + y.0 .1))));
        let inputs: Vec<(f32, f32)> = scored.iter().take(3).map(|(k, _)| (k.0 as f32, k.1 as f32)).collect();
        (!inputs.is_empty()).then_some(Ambiguity { hypotheses, inputs, arity })
    }

    /// When the lessons leave several explanations equally good (they agree on every lesson and part ways elsewhere), which numbers would
    /// settle it? Returns the explanations that fit, and the questions that tell them apart: whole numbers nobody taught, best first (the
    /// ones on which the explanations give the most different answers, then the smallest numbers). None if one explanation fits, or none.
    pub fn ambiguity(items: &[Item], lib: &Library) -> Option<Ambiguity> {
        let own = items.first().map_or(NO_ORIGIN, |i| i.unit);
        let (found, behaviours, hits) = ProgramRule::search(items, &OPS, true, &lib.without(own), 0.0);
        let rule = found?;
        if behaviours < 2 {
            return None;
        }
        let arity = rule.arity;
        let mut seen: HashSet<Vec<i32>> = HashSet::new();
        let mut hypotheses: Vec<Expr> = Vec::new();
        for e in hits {
            if seen.insert(ProgramRule::grid_signature(&e, arity)) && hypotheses.len() < 8 {
                hypotheses.push(e);
            }
        }
        let taught: HashSet<(i32, i32)> = points_of(items).iter().map(|p| (p.0 as i32, p.1 as i32)).collect();
        let mut scored: Vec<((i32, i32), usize)> = Vec::new();
        for a in 0..=30i32 {
            for b in 0..=if arity == 2 { 30 } else { 0 } {
                if taught.contains(&(a, b)) {
                    continue;
                }
                let answers: HashSet<i32> = hypotheses.iter().map(|e| { let v = e.eval(a as f32, b as f32); if v.is_finite() { (v * 100.0).round() as i32 } else { i32::MIN } }).collect();
                if answers.len() >= 2 {
                    scored.push(((a, b), answers.len()));
                }
            }
        }
        scored.sort_by(|x, y| y.1.cmp(&x.1).then((x.0 .0 + x.0 .1).cmp(&(y.0 .0 + y.0 .1))));
        let inputs: Vec<(f32, f32)> = scored.iter().take(3).map(|(k, _)| (k.0 as f32, k.1 as f32)).collect();
        (!inputs.is_empty()).then_some(Ambiguity { hypotheses, inputs, arity })
    }

    /// Do the lessons carry enough information to believe this program? A search that tries N candidates will, by luck alone, find one that
    /// fits any pattern with fewer than about log2(N) bits in it; a unit whose answers are only 0 and 1 gives one bit a fact. So the bits in
    /// the lessons (facts x log2 of the distinct answers) must exceed the bits of searching (log2 of the candidates tried) by a margin.
    /// A short program found early passes (`first div 10`, found among about a thousand); a long one found late in a huge search over thin
    /// data does not. (Found by a primality unit being "explained" by an expression of nine pieces over thirteen 0/1 answers.)
    pub fn is_believable(&self, items: &[Item]) -> bool {
        let line = NumberLine::from_counting(&counting_lessons());
        let mut facts: HashSet<(i32, i32)> = HashSet::new();
        let mut answers: HashSet<i32> = HashSet::new();
        for it in items {
            let (nums, _) = ProgramRule::read(&line, &it.query);
            if let (Some(y), false) = (line.value(&it.answer), nums.is_empty()) {
                facts.insert((nums[0] as i32, *nums.get(1).unwrap_or(&0.0) as i32));
                answers.insert(y as i32);
            }
        }
        let data_bits = facts.len() as f32 * (answers.len().max(2) as f32).log2();
        let search_bits = (self.work.max(1) as f32).log2();
        data_bits >= search_bits + INFO_MARGIN_BITS
    }

    /// How an expression behaves over the numbers 0-30 (for a unary one only the first matters).
    pub(crate) fn grid_signature(e: &Expr, arity: usize) -> Vec<i32> {
        let mut sig = Vec::new();
        for a in 0..=30 {
            for b in 0..=if arity == 2 { 30 } else { 0 } {
                let v = e.eval(a as f32, b as f32);
                sig.push(if v.is_finite() { (v * 100.0).round() as i32 } else { i32::MIN });
            }
        }
        sig
    }

    /// Answer a question with the program. Declines unless the question has the numbers it needs, every other word is one it has
    /// met in this unit's lessons (or is harmless filler), and the result lands on a number it knows.
    pub fn predict(&self, q: &str, filler: &dyn Fn(&str) -> bool) -> Option<String> {
        let (nums, rest) = ProgramRule::read(&self.line, q);
        if nums.len() != self.arity || (self.partial.is_none() && rest.iter().any(|w| !self.vocab.contains(w) && !filler(w))) {
            return None;
        }
        let (a, b) = (nums[0], *nums.get(1).unwrap_or(&0.0));
        let y = self.expr.eval(a, b);
        // Where explanations that fit every lesson part ways, the lessons do not decide the answer: decline.
        if self.alts.iter().any(|e| {
            let z = e.eval(a, b);
            !(z.is_finite() && (z - y).abs() < 0.01)
        }) {
            return None;
        }
        let (word, dist) = self.line.say(y, crate::quantity::uses_digits(q))?;
        if dist < 0.2 { Some(word) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds;

    fn unit_items(w: &kinds::World, unit: usize) -> Vec<Item> {
        w.train.iter().filter(|i| i.unit == unit).cloned().collect()
    }

    #[test]
    fn a_long_fit_found_late_over_thin_data_is_not_believed_but_a_short_one_found_early_is() {
        let thin: Vec<Item> = (0..13).map(|a: i64| Item { unit: 0, fact: a as usize, template: 0, query: format!("zork {a}"), answer: ((a % 3 == 0) as i64).to_string() }).collect();
        let rich: Vec<Item> = (0..12).flat_map(|a: i64| (0..4).map(move |b: i64| (a, b))).map(|(a, b)| Item { unit: 0, fact: (a * 4 + b) as usize, template: 0, query: format!("zork {a} {b}"), answer: (a * a + b * b).to_string() }).collect();
        let rule = |work| ProgramRule { expr: Expr::A, alts: Vec::new(), partial: None, repaired: Vec::new(), work, arity: 1, line: NumberLine::from_counting(&counting_lessons()), vocab: HashSet::new(), };
        assert!(!rule(100_000).is_believable(&thin), "13 bits of 0/1 against 17 bits of searching: luck explains it");
        assert!(rule(500).is_believable(&thin), "9 bits of searching against 13 bits of data");
        assert!(rule(3_000_000).is_believable(&rich), "48 facts with many answers carry far more than 22 bits");
    }

    #[test]
    fn a_unit_contributes_a_behaviour_to_the_library_once_even_after_it_is_evicted() {
        let mut lib = Library::default();
        let sumsq = Expr::from_text("(add (mul A A) (mul B B))").unwrap();
        let bloated = Expr::from_text("(add (mul A A) (mul (sub A (add A B)) (sub A (add A B))))").unwrap();
        assert!(lib.learn_program(&sumsq, 2, 7));
        assert!(!lib.learn_program(&bloated, 2, 7), "an equivalent rewrite adds nothing");
        // fill the library until the first function is evicted
        for k in 0..MAX_FUNCS {
            assert!(lib.learn_program(&Expr::from_text(&format!("(add (mul A K{}) B)", 3 + k)).unwrap(), 2, 100 + k));
        }
        assert!(!lib.funcs.iter().any(|f| f.origin == 7), "evicted");
        assert!(!lib.learn_program(&sumsq, 2, 7), "and its unit refitting does not put it back and push another out");
        assert!(!lib.learn_program(&bloated, 2, 7));
        assert_eq!(lib.funcs.len(), MAX_FUNCS);
    }

    #[test]
    fn far_probes_do_not_make_a_program_unfittable() {
        let mut lib = Library::default();
        lib.learn_constant(10);
        lib.learn_program(&Expr::from_text("(add (mul A A) (mul B B))").unwrap(), 2, 5);
        let mk = |a: i64, b: i64| Item { unit: 0, fact: (a * 1000 + b) as usize, template: 0, query: format!("what is {a} zork {b}"), answer: ((a * a + b * b) / 2).to_string() };
        let mut items: Vec<Item> = (0..=12).flat_map(|a| (0..=12).map(move |b| mk(a, b))).collect();
        assert!(ProgramRule::fit_trusted_with(&items, &lib).is_some(), "taught range");
        for (a, b) in [(24, 12), (12, 24), (24, 24), (48, 24), (24, 48), (48, 48)] {
            items.push(mk(a, b));
        }
        let before = ProgramRule::fit_trusted_with(&items, &lib);
        assert!(before.is_some(), "after round 2 probes");
        items.push(mk(96, 48));
        let (found, behaviours, hits) = ProgramRule::search_pub(&items, &lib);
        println!("DEBUG found {:?} behaviours {behaviours} hits {}", found.as_ref().map(|f| f.expr.show()), hits.len());
        let after = ProgramRule::fit_trusted_with(&items, &lib);
        assert!(after.is_some(), "after the (96, 48) probe: found {:?}", found.map(|f| (f.expr.show(), f.work)));
    }

    #[test]
    fn a_library_function_is_one_piece_so_deeper_things_come_within_reach() {
        let items: Vec<Item> = (0..=25).filter(|a| *a != 8 && *a != 22).map(|a: i64| Item { unit: 0, fact: a as usize, template: 0, query: format!("what is zork {a}"), answer: (a / 10 + a % 10).to_string() }).collect();
        // without the pieces, ten cannot be built in seven pieces, so there is no program
        assert!(ProgramRule::fit_unique(&items).is_none());
        let mut lib = Library::default();
        lib.learn_constant(10);
        let tens = ProgramRule::fit_unique_with(&items.iter().map(|i| Item { answer: (i.fact / 10).to_string(), ..i.clone() }).collect::<Vec<_>>(), &lib).expect("tens");
        assert_eq!(tens.expr.show(), "(first div 10)");
        let units = ProgramRule::fit_unique_with(&items.iter().map(|i| Item { answer: (i.fact % 10).to_string(), ..i.clone() }).collect::<Vec<_>>(), &lib).expect("units");
        assert!(lib.learn_program(&tens.expr, 1, NO_ORIGIN) && lib.learn_program(&units.expr, 1, NO_ORIGIN));
        let digit_sum = ProgramRule::fit_unique_with(&items, &lib).expect("digit sum from tens and units");
        assert_eq!(digit_sum.predict("what is zork 29", &|_| false).as_deref(), Some("11"));
        // and when the numbers taught leave the answer open it declines: up to 15, "(first + first div 10) mod 10" fits as well
        let narrow: Vec<Item> = items.iter().filter(|i| i.fact <= 15).cloned().collect();
        assert!(ProgramRule::fit_unique_with(&narrow, &lib).is_none(), "two explanations fit; the data cannot say which");
    }

    #[test]
    fn when_the_lessons_leave_it_open_it_can_say_which_number_would_settle_it() {
        let mut lib = Library::default();
        lib.learn_constant(10);
        let tens_items: Vec<Item> = (0..=25).map(|a: i64| Item { unit: 0, fact: a as usize, template: 0, query: format!("what is zork {a}"), answer: (a / 10).to_string() }).collect();
        let units_items: Vec<Item> = (0..=25).map(|a: i64| Item { unit: 0, fact: a as usize, template: 0, query: format!("what is zork {a}"), answer: (a % 10).to_string() }).collect();
        lib.learn_program(&ProgramRule::fit_unique_with(&tens_items, &lib).unwrap().expr, 1, NO_ORIGIN);
        lib.learn_program(&ProgramRule::fit_unique_with(&units_items, &lib).unwrap().expr, 1, NO_ORIGIN);
        // digit sum taught on 0-15 only: "tens + units" and "(first + tens) mod 10" fit every lesson and part ways at 19
        let items: Vec<Item> = (0..=15).map(|a: i64| Item { unit: 0, fact: a as usize, template: 0, query: format!("what is zork {a}"), answer: (a / 10 + a % 10).to_string() }).collect();
        assert!(ProgramRule::fit_unique_with(&items, &lib).is_none());
        let amb = ProgramRule::ambiguity(&items, &lib).expect("it is ambiguous");
        assert!(amb.hypotheses.len() >= 2);
        assert!(amb.inputs.iter().all(|i| i.0 > 15.0), "asks about numbers nobody taught: {:?}", amb.inputs);
        // asking about the number it names settles it
        let n = amb.inputs[0].0 as i64;
        let mut more = items.clone();
        more.push(Item { unit: 0, fact: 99, template: 0, query: format!("what is zork {n}"), answer: (n / 10 + n % 10).to_string() });
        let settled = ProgramRule::fit_unique_with(&more, &lib).expect("one question settles it");
        assert_eq!(settled.predict("what is zork 29", &|_| false).as_deref(), Some("11"));
        // nothing ambiguous, nothing to ask: an exactly explained kind, and an arbitrary table
        let exact: Vec<Item> = (0..=20).map(|a: i64| Item { unit: 0, fact: a as usize, template: 0, query: format!("what is zork {a}"), answer: (a * 2).to_string() }).collect();
        assert!(ProgramRule::ambiguity(&exact, &lib).is_none());
        let table: Vec<Item> = (0..=15).map(|a: i64| Item { unit: 0, fact: a as usize, template: 0, query: format!("what is zork {a}"), answer: ((a * 7 + 3) % 11).to_string() }).collect();
        assert!(ProgramRule::ambiguity(&table, &lib).is_none() || ProgramRule::fit_unique_with(&table, &lib).is_none());
    }

    #[test]
    fn it_builds_the_rule_for_multiplying_and_for_how_far_apart_which_nothing_else_could() {
        let w = kinds::world(1);
        let times = ProgramRule::fit(&unit_items(&w, 6)).expect("times");
        assert_eq!(times.predict("what is seven times one", &|_| false).as_deref(), Some("seven"));
        assert_eq!(times.predict("what is two times five", &|_| false).as_deref(), Some("ten"));
        let apart = ProgramRule::fit(&unit_items(&w, 7)).expect("apart");
        assert_eq!(apart.predict("how far apart are nine and four", &|_| false).as_deref(), Some("five"));
        assert!(apart.expr.show().contains("apart"), "{}", apart.expr.show());
    }

    #[test]
    fn it_finds_nothing_when_there_is_nothing_to_find() {
        let w = kinds::world(1);
        assert!(ProgramRule::fit(&unit_items(&w, 8)).is_none(), "a code with no formula must not be 'explained'");
        assert!(ProgramRule::fit(&unit_items(&w, 2)).is_none(), "bigger and smaller together are not one formula");
        assert!(ProgramRule::fit(&unit_items(&w, 0)).is_none(), "capitals have no numbers");
    }

    #[test]
    fn without_the_convenient_operation_it_composes_the_same_thing_from_simpler_ones() {
        let w = kinds::world(1);
        let ops = [Op::Add, Op::Sub, Op::Mul, Op::Max, Op::Min];
        let apart = ProgramRule::fit_with(&unit_items(&w, 7), &ops).expect("apart from max and min");
        println!("built: {}", apart.expr.show());
        assert_eq!(apart.predict("how far apart are nine and four", &|_| false).as_deref(), Some("five"));
        assert_eq!(apart.predict("how far apart are two and ten", &|_| false).as_deref(), Some("eight"));
        let shown = apart.expr.show();
        assert!(!shown.contains("apart"), "it must not have been given the operation: {shown}");
        assert!(shown.contains("max") && shown.contains('-'), "built from subtraction and max: {shown}");
    }

    #[test]
    fn a_program_survives_being_saved_as_text() {
        for text in ["(mul A B)", "(half A)", "(add A K1)", "(max (sub A B) (sub B A))", "(apart A B)"] {
            let e = Expr::from_text(text).unwrap();
            assert_eq!(e.to_text(), text);
        }
        assert!(Expr::from_text("(mul A").is_none() && Expr::from_text("(nope A B)").is_none());
    }

    #[test]
    fn when_two_different_expressions_fit_equally_it_refuses_to_pick() {
        // "N less than ten": `second - first` and `first apart second` agree on every lesson (the second number is always larger).
        let mut items = Vec::new();
        let nums = crate::data::NUM;
        for n in 0..=6usize {
            items.push(Item { unit: 0, fact: n, template: 0, query: format!("what is {} less than ten", nums[n]), answer: nums[10 - n].to_string() });
        }
        assert!(ProgramRule::fit(&items).is_some(), "an expression fits");
        assert!(ProgramRule::fit_unique(&items).is_none(), "but it is ambiguous, so no unique answer");
        let w = kinds::world(1);
        let times: Vec<Item> = w.train.iter().filter(|i| i.unit == 6).cloned().collect();
        assert!(ProgramRule::fit_unique(&times).is_some(), "multiplication is unambiguous");
    }

    #[test]
    fn small_rules_come_out_small() {
        let w = kinds::world(1);
        let after = ProgramRule::fit(&unit_items(&w, 3)).unwrap();
        assert_eq!(after.expr.show(), "(first + 1)");
        let half = ProgramRule::fit(&unit_items(&w, 5)).unwrap();
        assert_eq!(half.expr.show(), "half(first)");
    }
}

/// Several explanations fit every lesson; these numbers would tell them apart.
pub struct Ambiguity {
    pub hypotheses: Vec<Expr>,
    /// (first, second) to ask about; the second is 0 for a one-number kind.
    pub inputs: Vec<(f32, f32)>,
    pub arity: usize,
}

/// The (first, second, answer) of each distinct taught lesson.
fn points_of(items: &[Item]) -> Vec<(f32, f32, f32)> {
    let line = NumberLine::from_counting(&counting_lessons());
    let mut points: Vec<(f32, f32, f32)> = Vec::new();
    for it in items {
        let (nums, _) = ProgramRule::read(&line, &it.query);
        if let (Some(y), false) = (line.value(&it.answer), nums.is_empty()) {
            let p = (nums[0], *nums.get(1).unwrap_or(&0.0), y);
            if !points.contains(&p) {
                points.push(p);
            }
        }
    }
    points
}
