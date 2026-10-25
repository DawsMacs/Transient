// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! Explanations as programs, and a search for them that is priced in bits.
//!
//! A program is a small expression over the numbers in a question. Its cost is the bits needed to write it down: every node picks one symbol out
//! of the alphabet (a variable, an operation, or "a constant"), and a constant also pays for its value. The search builds expressions from the
//! smallest up and keeps one of any two that compute the same numbers on every column it looks at (observational equivalence), so the space it
//! walks is the space of different behaviors, not of different spellings.
//!
//! The columns are the points the program must fit (the data) followed by probe points the program is not asked to fit. Two programs that agree
//! on every data point but differ on a probe are both kept. That is what lets the learner say how sure it is about a question it has not seen:
//! the believable programs that disagree are exactly the doubt.
//!
//! Everything is integer arithmetic. A value outside a million either way, a division by zero or an overflow makes a program undefined at that
//! point; a program that is undefined at any column is dropped.

use crate::bits::{self, Cost};
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::rc::Rc;

/// Values stay within this bound; beyond it a program counts as undefined there.
const LIMIT: i64 = 1 << 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Min,
    Max,
    Div,
    Mod,
}

/// The operations Novice starts with. This list is a hand-built choice (written down as such in the README), and the first thing the language
/// growing is meant to make less important.
pub const OPS: [Op; 7] = [Op::Add, Op::Sub, Op::Mul, Op::Min, Op::Max, Op::Div, Op::Mod];

impl Op {
    fn commutative(self) -> bool {
        matches!(self, Op::Add | Op::Mul | Op::Min | Op::Max)
    }

    pub fn apply(self, a: i64, b: i64) -> Option<i64> {
        let v = match self {
            Op::Add => a.checked_add(b)?,
            Op::Sub => a.checked_sub(b)?,
            Op::Mul => a.checked_mul(b)?,
            Op::Min => a.min(b),
            Op::Max => a.max(b),
            Op::Div => a.checked_div(b)?,
            Op::Mod => a.checked_rem(b)?,
        };
        (v.abs() <= LIMIT).then_some(v)
    }

    fn symbol(self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Min => "min",
            Op::Max => "max",
            Op::Div => "div",
            Op::Mod => "mod",
        }
    }
}

/// A word the language has learned: the program that explains one kind of question, made callable so that later kinds can be built from it.
#[derive(Debug, PartialEq, Eq)]
pub struct Lib {
    pub name: String,
    pub arity: usize,
    pub body: Expr,
}

/// A program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Var(usize),
    Const(i64),
    Bin(Op, Box<Expr>, Box<Expr>),
    /// A learned word applied to its arguments.
    Call(Rc<Lib>, Vec<Expr>),
}

impl Expr {
    /// The value at `args`, or `None` where the program is undefined.
    pub fn eval(&self, args: &[i64]) -> Option<i64> {
        match self {
            Expr::Var(i) => args.get(*i).copied(),
            Expr::Const(c) => Some(*c),
            Expr::Bin(op, a, b) => op.apply(a.eval(args)?, b.eval(args)?),
            Expr::Call(lib, xs) => {
                let mut vals = [0i64; 2];
                for (slot, x) in vals.iter_mut().zip(xs) {
                    *slot = x.eval(args)?;
                }
                lib.body.eval(&vals[..xs.len()])
            }
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Expr::Bin(_, a, b) => 1 + a.size() + b.size(),
            Expr::Call(_, xs) => 1 + xs.iter().map(Expr::size).sum::<usize>(),
            _ => 1,
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Var(i) => write!(f, "{}", ["x", "y", "z", "w"].get(*i).copied().unwrap_or("v")),
            Expr::Const(c) => write!(f, "{c}"),
            Expr::Bin(op, a, b) if matches!(op, Op::Add | Op::Sub | Op::Mul) => write!(f, "({a} {} {b})", op.symbol()),
            Expr::Bin(op, a, b) => write!(f, "{}({a}, {b})", op.symbol()),
            Expr::Call(lib, xs) => {
                write!(f, "{}(", lib.name)?;
                for (i, x) in xs.iter().enumerate() {
                    write!(f, "{}{x}", if i > 0 { ", " } else { "" })?;
                }
                write!(f, ")")
            }
        }
    }
}

/// Bits for one node: which symbol of the alphabet it is. The alphabet is the variables, the operations, the learned words and "a constant".
pub fn node_cost(arity: usize, words: usize) -> Cost {
    bits::log2((arity + OPS.len() + words + 1) as u64)
}

/// Bits for writing a whole program down. (A learned word's own definition is paid for once, where it was learned, not at every use.)
pub fn cost_of(e: &Expr, arity: usize, words: usize) -> Cost {
    let node = node_cost(arity, words);
    match e {
        Expr::Var(_) => node,
        Expr::Const(c) => node + bits::uint(c.unsigned_abs()),
        Expr::Bin(_, a, b) => node + cost_of(a, arity, words) + cost_of(b, arity, words),
        Expr::Call(_, xs) => node + xs.iter().map(|x| cost_of(x, arity, words)).sum::<Cost>(),
    }
}

/// What a search is asked to do.
pub struct Task<'a> {
    /// Points the program must reproduce, and the answers there.
    pub inputs: &'a [Vec<i64>],
    pub target: &'a [i64],
    /// Points the program is not asked to fit, kept so that programs that disagree there stay apart.
    pub probes: &'a [Vec<i64>],
    pub arity: usize,
    /// Constants that may appear as leaves.
    pub consts: &'a [i64],
    /// Learned words (of one or two arguments) the program may use.
    pub words: &'a [Rc<Lib>],
    /// Largest program, in nodes.
    pub max_size: usize,
    /// After the first fit, how many more nodes to keep looking for rivals.
    pub rival_size: usize,
    /// Most expressions held at once; reaching it ends the search and sets `Outcome::exhausted`.
    pub max_items: usize,
}

/// A program that fits the data.
#[derive(Clone, Debug)]
pub struct Found {
    pub expr: Expr,
    pub cost: Cost,
    /// Its values on the probe points.
    pub probe_values: Vec<i64>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Effort {
    /// Expression-point evaluations (one per column per candidate): a count of the work done, comparable across runs and machines.
    pub evaluations: u64,
}

pub struct Outcome {
    /// Programs that fit every data point, cheapest first; one per different behavior on the probes.
    pub found: Vec<Found>,
    pub exhausted: bool,
    pub effort: Effort,
}

#[derive(Default)]
struct Identity(u64);

impl Hasher for Identity {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, _: &[u8]) {
        unreachable!("keys are u64");
    }
    fn write_u64(&mut self, x: u64) {
        self.0 = x;
    }
}

#[derive(Clone, Copy)]
struct Item {
    /// 0 variable, 1 constant, 2 operation, 3 learned word.
    kind: u8,
    /// The operation, or the index of the word.
    op: Op,
    word: u16,
    a: u32,
    b: u32,
    cost: Cost,
}

struct Space {
    cols: usize,
    vals: Vec<i32>,
    items: Vec<Item>,
    seen: HashMap<u64, u32, BuildHasherDefault<Identity>>,
}

impl Space {
    fn row(&self, i: u32) -> &[i32] {
        &self.vals[i as usize * self.cols..(i as usize + 1) * self.cols]
    }

    fn hash(row: &[i32]) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for &v in row {
            h ^= u64::from(v as u32);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
            h ^= h >> 29;
        }
        h
    }

    /// Add a new behavior; `None` if the same numbers were already reached by something at least as cheap.
    fn add(&mut self, row: &[i32], item: Item) -> Option<u32> {
        let h = Self::hash(row);
        if self.seen.contains_key(&h) {
            return None;
        }
        let id = self.items.len() as u32;
        self.seen.insert(h, id);
        self.vals.extend_from_slice(row);
        self.items.push(item);
        Some(id)
    }

    fn expr(&self, i: u32, words: &[Rc<Lib>]) -> Expr {
        let it = self.items[i as usize];
        match it.kind {
            0 => Expr::Var(it.a as usize),
            1 => Expr::Const(i64::from(it.a as i32)),
            2 => Expr::Bin(it.op, Box::new(self.expr(it.a, words)), Box::new(self.expr(it.b, words))),
            _ => {
                let lib = words[it.word as usize].clone();
                let args = if lib.arity == 1 { vec![self.expr(it.a, words)] } else { vec![self.expr(it.a, words), self.expr(it.b, words)] };
                Expr::Call(lib, args)
            }
        }
    }
}

/// Look for programs that fit the task. Smaller programs are built before larger ones; the search stops `rival_size` nodes after the first fit.
pub fn search(task: &Task) -> Outcome {
    let n = task.inputs.len();
    let cols = n + task.probes.len();
    let mut effort = Effort::default();
    let mut space = Space { cols, vals: Vec::new(), items: Vec::new(), seen: HashMap::default() };
    let node = node_cost(task.arity, task.words.len());
    let mut found: Vec<Found> = Vec::new();
    let mut by_size: Vec<Vec<u32>> = vec![Vec::new(); task.max_size + 1];
    let mut first_fit_size: Option<usize> = None;
    let mut exhausted = false;
    let want: Vec<i32> = task.target.iter().map(|&t| t as i32).collect();
    let points = || task.inputs.iter().chain(task.probes.iter());
    let blank = Item { kind: 0, op: Op::Add, word: 0, a: 0, b: 0, cost: node };

    let check_fit = |space: &Space, id: u32, size: usize, found: &mut Vec<Found>, first: &mut Option<usize>| {
        let row = space.row(id);
        if row[..n] == want[..] {
            let probe_values = row[n..].iter().map(|&v| i64::from(v)).collect();
            found.push(Found { expr: space.expr(id, task.words), cost: space.items[id as usize].cost, probe_values });
            first.get_or_insert(size);
        }
    };

    // Size 1: the variables and the constants.
    let mut row = vec![0i32; cols];
    for v in 0..task.arity {
        for (c, p) in points().enumerate() {
            row[c] = p[v] as i32;
        }
        if let Some(id) = space.add(&row, Item { kind: 0, a: v as u32, ..blank }) {
            by_size[1].push(id);
            check_fit(&space, id, 1, &mut found, &mut first_fit_size);
        }
    }
    for &k in task.consts {
        if k.abs() >= LIMIT {
            continue;
        }
        row.iter_mut().for_each(|r| *r = k as i32);
        if let Some(id) = space.add(&row, Item { kind: 1, a: k as i32 as u32, cost: node + bits::uint(k.unsigned_abs()), ..blank }) {
            by_size[1].push(id);
            check_fit(&space, id, 1, &mut found, &mut first_fit_size);
        }
    }

    // An operation is a primitive (two arguments) or a learned word (one or two).
    enum Fun {
        Prim(Op),
        Word(usize),
    }
    let mut funs: Vec<Fun> = OPS.iter().map(|&o| Fun::Prim(o)).collect();
    funs.extend((0..task.words.len()).map(Fun::Word));

    'sizes: for size in 2..=task.max_size {
        if let Some(f) = first_fit_size {
            if size > f + task.rival_size {
                break;
            }
        }
        for fun in &funs {
            let (arity, commutative) = match fun {
                Fun::Prim(op) => (2, op.commutative()),
                Fun::Word(w) => (task.words[*w].arity, false),
            };
            let pairs: Vec<(usize, usize)> = if arity == 1 { vec![(size - 1, 0)] } else { (1..size - 1).map(|l| (l, size - 1 - l)).collect() };
            for (left, right) in pairs {
                if arity == 2 && commutative && left > right {
                    continue;
                }
                let la = by_size[left].clone();
                let lb = if arity == 1 { vec![0u32] } else { by_size[right].clone() };
                for (ia, &a) in la.iter().enumerate() {
                    for (ib, &b) in lb.iter().enumerate() {
                        if arity == 2 && commutative && left == right && ib < ia {
                            continue;
                        }
                        effort.evaluations += cols as u64;
                        let mut ok = true;
                        for c in 0..cols {
                            let x = i64::from(space.row(a)[c]);
                            let y = if arity == 1 { 0 } else { i64::from(space.row(b)[c]) };
                            let v = match fun {
                                Fun::Prim(op) => op.apply(x, y),
                                Fun::Word(w) => {
                                    let lib = &task.words[*w];
                                    let vals = [x, y];
                                    lib.body.eval(&vals[..lib.arity])
                                }
                            };
                            match v {
                                Some(v) => row[c] = v as i32,
                                None => {
                                    ok = false;
                                    break;
                                }
                            }
                        }
                        if !ok {
                            continue;
                        }
                        let sub = space.items[a as usize].cost + if arity == 1 { 0 } else { space.items[b as usize].cost };
                        let item = match fun {
                            Fun::Prim(op) => Item { kind: 2, op: *op, a, b, cost: node + sub, ..blank },
                            Fun::Word(w) => Item { kind: 3, word: *w as u16, a, b, cost: node + sub, ..blank },
                        };
                        if let Some(id) = space.add(&row, item) {
                            by_size[size].push(id);
                            check_fit(&space, id, size, &mut found, &mut first_fit_size);
                            if space.items.len() >= task.max_items {
                                exhausted = true;
                                break 'sizes;
                            }
                        }
                    }
                }
            }
        }
    }

    // One program per different behavior on the probes: the cheapest.
    found.sort_by(|x, y| x.cost.cmp(&y.cost).then_with(|| x.expr.size().cmp(&y.expr.size())).then_with(|| x.expr.to_string().cmp(&y.expr.to_string())));
    let mut kept: Vec<Found> = Vec::new();
    for f in found {
        if !kept.iter().any(|k| k.probe_values == f.probe_values) {
            kept.push(f);
        }
    }
    Outcome { found: kept, exhausted, effort }
}

/// What the believable programs say at one point: the most believed answer, how much of the belief it carries, and the total belief (both in
/// the fixed-point units of [`bits::exp2_neg`]). Programs that are undefined at the point count toward the total but support no answer.
pub fn weigh(found: &[Found], at: &[i64]) -> Option<(i64, u64, u64)> {
    let best = found.first()?.cost;
    let mut votes: Vec<(i64, u64)> = Vec::new();
    let mut total = 0u64;
    for f in found {
        let w = bits::exp2_neg(f.cost - best);
        total += w;
        if let Some(v) = f.expr.eval(at) {
            match votes.iter_mut().find(|(a, _)| *a == v) {
                Some(slot) => slot.1 += w,
                None => votes.push((v, w)),
            }
        }
    }
    votes.into_iter().max_by(|x, y| x.1.cmp(&y.1).then(y.0.cmp(&x.0))).map(|(a, w)| (a, w, total))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(points: &[(&[i64], i64)], consts: &[i64], max_size: usize) -> Outcome {
        let inputs: Vec<Vec<i64>> = points.iter().map(|p| p.0.to_vec()).collect();
        let target: Vec<i64> = points.iter().map(|p| p.1).collect();
        let arity = inputs[0].len();
        search(&Task { inputs: &inputs, target: &target, probes: &[], arity, consts, words: &[], max_size, rival_size: 0, max_items: 1_000_000 })
    }

    #[test]
    fn finds_a_sum() {
        let o = run(&[(&[1, 2], 3), (&[4, 7], 11), (&[0, 5], 5), (&[9, 3], 12)], &[0, 1, 2], 5);
        assert_eq!(o.found[0].expr.to_string(), "(x + y)");
    }

    #[test]
    fn finds_a_digit_rule_with_the_constant_it_needs() {
        let pts: Vec<(Vec<i64>, i64)> = (0..30).map(|x| (vec![x], x / 10)).collect();
        let refs: Vec<(&[i64], i64)> = pts.iter().map(|(a, t)| (a.as_slice(), *t)).collect();
        let o = run(&refs, &[0, 1, 2, 10], 3);
        assert_eq!(o.found[0].expr.to_string(), "div(x, 10)");
    }

    #[test]
    fn a_saturating_sum_needs_its_ceiling() {
        // The ceiling shows only in the told answers; with it among the constants the cap is found, without it nothing fits.
        let pts: Vec<(Vec<i64>, i64)> = vec![(vec![20, 25], 30), (vec![10, 5], 15), (vec![30, 3], 30), (vec![2, 3], 5), (vec![18, 17], 30)];
        let refs: Vec<(&[i64], i64)> = pts.iter().map(|(a, t)| (a.as_slice(), *t)).collect();
        assert!(run(&refs, &[0, 1, 2], 5).found.is_empty());
        assert_eq!(run(&refs, &[0, 1, 2, 30], 5).found[0].expr.to_string(), "min(30, (x + y))");
    }

    #[test]
    fn programs_that_differ_only_off_the_data_are_both_kept() {
        let inputs: Vec<Vec<i64>> = vec![vec![1, 2], vec![3, 4], vec![5, 1]];
        let target = vec![3, 7, 6];
        let probes: Vec<Vec<i64>> = vec![vec![40, 40]];
        let o = search(&Task { inputs: &inputs, target: &target, probes: &probes, arity: 2, consts: &[0, 1, 30], words: &[], max_size: 5, rival_size: 2, max_items: 1_000_000 });
        let at_probe: Vec<i64> = o.found.iter().map(|f| f.probe_values[0]).collect();
        assert!(at_probe.contains(&80) && at_probe.contains(&30), "{at_probe:?}");
        let (answer, top, total) = weigh(&o.found, &[40, 40]).unwrap();
        assert_eq!(answer, 80);
        assert!(top < total, "the rival must cost some belief");
    }

    #[test]
    fn undefined_points_drop_a_program() {
        // x / y cannot be used where y is 0.
        let o = run(&[(&[4, 0], 4), (&[6, 0], 6)], &[0, 1], 3);
        assert!(o.found.iter().all(|f| !f.expr.to_string().contains("div")));
    }

    #[test]
    fn costs_are_bits_of_writing_the_program() {
        let e = Expr::Bin(Op::Add, Box::new(Expr::Var(0)), Box::new(Expr::Const(2)));
        // 3 nodes at log2(10) each, plus the constant 2 (Elias gamma of 3 = 3 bits).
        assert_eq!(cost_of(&e, 2, 0), 3 * node_cost(2, 0) + 3 * bits::BIT);
    }

    #[test]
    fn a_learned_word_reaches_programs_that_are_too_long_to_enumerate() {
        use std::rc::Rc;
        // "square" as a word: the kind that squares its number.
        let square = Rc::new(Lib { name: "square".into(), arity: 1, body: Expr::Bin(Op::Mul, Box::new(Expr::Var(0)), Box::new(Expr::Var(0))) });
        // sum of two squares plus one: nine nodes without the word, five with it.
        let pts: Vec<(Vec<i64>, i64)> = [(1, 2), (3, 4), (5, 0), (2, 2), (6, 1), (4, 5), (0, 3)].iter().map(|&(x, y)| (vec![x, y], x * x + y * y + 1)).collect();
        let inputs: Vec<Vec<i64>> = pts.iter().map(|p| p.0.clone()).collect();
        let target: Vec<i64> = pts.iter().map(|p| p.1).collect();
        let words = vec![square];
        let run = |words: &[Rc<Lib>]| search(&Task { inputs: &inputs, target: &target, probes: &[], arity: 2, consts: &[0, 1, 2], words, max_size: 7, rival_size: 0, max_items: 1_000_000 });
        assert!(run(&[]).found.is_empty());
        let with = run(&words);
        assert_eq!(with.found[0].expr.size(), 7, "{}", with.found[0].expr);
        assert!(with.found[0].expr.to_string().contains("square"));
    }
}
