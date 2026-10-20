// SPDX-License-Identifier: AGPL-3.0-or-later
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
                };
                format!("({} {} {})", l.show(), sym, r.show())
            }
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
    }
}

/// How many genuinely different behaviours these expressions have: two that agree on every pair of numbers from 0 to 10 are one.
fn behaviours(exprs: &[Expr]) -> usize {
    let mut seen: HashSet<Vec<i32>> = HashSet::new();
    for e in exprs {
        let mut sig = Vec::with_capacity(121);
        for a in 0..=10 {
            for b in 0..=10 {
                sig.push((e.eval(a as f32, b as f32) * 100.0).round() as i32);
            }
        }
        seen.insert(sig);
    }
    seen.len()
}

const OPS: [Op; 6] = [Op::Add, Op::Sub, Op::Mul, Op::Max, Op::Min, Op::AbsDiff];
const MAX_SIZE: usize = 7;
const CAP_PER_LEVEL: usize = 40_000;

struct Cand {
    expr: Expr,
    vals: Vec<f32>,
}

fn signature(vals: &[f32]) -> Vec<i32> {
    vals.iter().map(|v| (v * 100.0).round() as i32).collect()
}

pub struct ProgramRule {
    pub expr: Expr,
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
        ProgramRule { expr, arity, line: NumberLine::from_counting(&counting_lessons()), vocab: vocab.into_iter().collect() }
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

    /// The same search with a chosen set of operations (to see what it can compose when a convenient one is taken away).
    pub fn fit_with(items: &[Item], ops: &[Op]) -> Option<ProgramRule> {
        ProgramRule::search(items, ops, false).0
    }

    /// Only if exactly ONE smallest expression fits. When two different expressions of the same size reproduce every lesson (for
    /// instance `first apart second` and `second - first`, which agree whenever the second number is the larger), the data cannot
    /// say which one is meant, and nothing should be built on the guess.
    pub fn fit_unique(items: &[Item]) -> Option<ProgramRule> {
        let (found, hits) = ProgramRule::search(items, &OPS, true);
        if hits == 1 { found } else { None }
    }

    /// The search. With `count_all`, it finishes the size at which the first expression fits and reports how many fit there.
    fn search(items: &[Item], ops: &[Op], count_all: bool) -> (Option<ProgramRule>, usize) {
        let line = NumberLine::from_counting(&counting_lessons());
        let mut points: Vec<(f32, f32, f32)> = Vec::new();
        let mut vocab = HashSet::new();
        let mut arity = 0;
        for it in items {
            let (nums, rest) = ProgramRule::read(&line, &it.query);
            let y = match line.value(&it.answer) {
                Some(y) => y,
                None => return (None, 0),
            };
            if nums.is_empty() || nums.len() > 2 {
                return (None, 0);
            }
            arity = arity.max(nums.len());
            let p = (nums[0], *nums.get(1).unwrap_or(&0.0), y);
            if !points.contains(&p) {
                points.push(p);
            }
            vocab.extend(rest);
        }
        if points.is_empty() {
            return (None, 0);
        }
        let mut first: Option<ProgramRule> = None;
        let mut hits_n = 0usize;
        let mut hit_exprs: Vec<Expr> = Vec::new();
        let ys: Vec<f32> = points.iter().map(|p| p.2).collect();
        let hits = |vals: &[f32]| vals.iter().zip(&ys).all(|(v, y)| (v - y).abs() < 0.01);

        let mut seen: HashSet<Vec<i32>> = HashSet::new();
        let mut levels: Vec<Vec<Cand>> = (0..=MAX_SIZE).map(|_| Vec::new()).collect();
        let mut terminals = vec![
            Cand { expr: Expr::A, vals: points.iter().map(|p| p.0).collect() },
            Cand { expr: Expr::K(1), vals: vec![1.0; points.len()] },
            Cand { expr: Expr::K(2), vals: vec![2.0; points.len()] },
        ];
        if arity == 2 {
            terminals.push(Cand { expr: Expr::B, vals: points.iter().map(|p| p.1).collect() });
        }
        for c in terminals {
            if hits(&c.vals) {
                hits_n += 1;
                hit_exprs.push(c.expr.clone());
                if first.is_none() {
                    first = Some(ProgramRule { expr: c.expr.clone(), arity, line: NumberLine::from_counting(&counting_lessons()), vocab: vocab.clone() });
                }
                if !count_all {
                    return (first, behaviours(&hit_exprs));
                }
            }
            if seen.insert(signature(&c.vals)) {
                levels[1].push(c);
            }
        }
        if first.is_some() {
            return (first, behaviours(&hit_exprs));
        }
        for size in 2..=MAX_SIZE {
            let mut made: Vec<Cand> = Vec::new();
            // Half of a smaller expression.
            for c in &levels[size - 1] {
                let vals: Vec<f32> = c.vals.iter().map(|v| v / 2.0).collect();
                if hits(&vals) {
                    hits_n += 1;
                    hit_exprs.push(Expr::Half(Box::new(c.expr.clone())));
                    if first.is_none() {
                        first = Some(ProgramRule { expr: Expr::Half(Box::new(c.expr.clone())), arity, line: NumberLine::from_counting(&counting_lessons()), vocab: vocab.clone() });
                    }
                    if !count_all {
                        return (first, behaviours(&hit_exprs));
                    }
                }
                if seen.insert(signature(&vals)) {
                    made.push(Cand { expr: Expr::Half(Box::new(c.expr.clone())), vals });
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
                            let vals: Vec<f32> = l.vals.iter().zip(&r.vals).map(|(x, y)| apply(op, *x, *y)).collect();
                            if vals.iter().any(|v| !v.is_finite()) {
                                continue;
                            }
                            if hits(&vals) {
                                hits_n += 1;
                                hit_exprs.push(Expr::Bin(op, Box::new(l.expr.clone()), Box::new(r.expr.clone())));
                                if first.is_none() {
                                    let expr = Expr::Bin(op, Box::new(l.expr.clone()), Box::new(r.expr.clone()));
                                    first = Some(ProgramRule { expr, arity, line: NumberLine::from_counting(&counting_lessons()), vocab: vocab.clone() });
                                }
                                if !count_all {
                                    return (first, behaviours(&hit_exprs));
                                }
                            }
                            if made.len() < CAP_PER_LEVEL && seen.insert(signature(&vals)) {
                                made.push(Cand { expr: Expr::Bin(op, Box::new(l.expr.clone()), Box::new(r.expr.clone())), vals });
                            }
                        }
                    }
                }
            }
            levels[size] = made;
            if first.is_some() {
                return (first, behaviours(&hit_exprs));
            }
        }
        (None, 0)
    }

    /// Answer a question with the program. Declines unless the question has the numbers it needs, every other word is one it has
    /// met in this unit's lessons (or is harmless filler), and the result lands on a number it knows.
    pub fn predict(&self, q: &str, filler: &dyn Fn(&str) -> bool) -> Option<String> {
        let (nums, rest) = ProgramRule::read(&self.line, q);
        if nums.len() != self.arity || rest.iter().any(|w| !self.vocab.contains(w) && !filler(w)) {
            return None;
        }
        let y = self.expr.eval(nums[0], *nums.get(1).unwrap_or(&0.0));
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
