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
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
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
    /// A function defined by its own values: what has been learned about it is the base, and anywhere else it is a step from a nearer point.
    Fix(Rc<FixDef>),
}

/// How far down a chain of steps a recurrence may go before it gives up.
pub const FIX_DEPTH: u32 = 12;

/// A function that is known at some points (a table that grows as the kind is taught and told things) and elsewhere satisfies
/// `f(p) = e(p, f(g1(p)), ..., f(gk(p)))` for one or two nearer points: each `g` says which point to look at (one expression per argument) and `e` what
/// to do with the values (an expression over the arguments and, as the variables after them, the values at the nearer points). Popcount is
/// `f(x) = f(x div 2) + x mod 2`; the greatest common divisor is `f(x, y) = f(y, x mod y)`; Fibonacci numbers are `f(x) = f(x - 1) + f(x - 2)`.
pub struct FixDef {
    pub arity: usize,
    /// The nearer points to look at, each as one expression per argument.
    pub steps: Vec<Vec<Expr>>,
    pub e: Expr,
    /// The values the function is known to have. Shared with the kind it belongs to, so it is live: whatever is learned about the kind later is
    /// used the next time the function is evaluated.
    pub table: Rc<RefCell<BTreeMap<Vec<i64>, i64>>>,
    /// Bits for the points of the table that the recurrence does not account for.
    pub base_cost: Cost,
}

impl FixDef {
    pub fn apply(&self, args: &[i64], depth: u32) -> Option<i64> {
        self.go(args, depth, &mut BTreeMap::new())
    }

    /// The value at `args`; what has been worked out on the way is remembered, so that a step over two earlier values does not branch for ever.
    fn go(&self, args: &[i64], depth: u32, memo: &mut BTreeMap<Vec<i64>, Option<i64>>) -> Option<i64> {
        if let Some(v) = self.table.borrow().get(args) {
            return Some(*v);
        }
        if depth == 0 {
            return None;
        }
        if let Some(done) = memo.get(args) {
            return *done;
        }
        let result = (|| {
            let mut vars = args.to_vec();
            for step in &self.steps {
                let next: Vec<i64> = step.iter().map(|g| g.eval(args)).collect::<Option<_>>()?;
                if next.iter().any(|&n| n < 0) || next == args {
                    return None;
                }
                vars.push(self.go(&next, depth - 1, memo)?);
            }
            self.e.eval(&vars)
        })();
        memo.insert(args.to_vec(), result);
        result
    }
}

impl PartialEq for FixDef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.table, &other.table) && self.arity == other.arity && self.steps == other.steps && self.e == other.e
    }
}

impl Eq for FixDef {}

impl std::fmt::Debug for FixDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FixDef({})", self)
    }
}

impl std::fmt::Display for FixDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let all = ["x", "y", "z", "w"];
        let args: Vec<&str> = all.iter().copied().take(self.arity).collect();
        let values = ["r", "s"];
        let mut with_values = args.clone();
        with_values.extend(values.iter().take(self.steps.len()));
        let calls: Vec<String> = self
            .steps
            .iter()
            .zip(values)
            .map(|(step, v)| format!("{v} = f({})", step.iter().map(|g| g.render(&args)).collect::<Vec<_>>().join(", ")))
            .collect();
        write!(f, "f({}) = {}, {}", args.join(", "), self.e.render(&with_values), calls.join(", "))
    }
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
            Expr::Fix(fix) => fix.apply(args.get(..fix.arity)?, FIX_DEPTH),
        }
    }

    /// Whether the program walks a chain of reductions anywhere, including inside the learned words it uses.
    pub fn has_fix(&self) -> bool {
        match self {
            Expr::Fix(_) => true,
            Expr::Var(_) | Expr::Const(_) => false,
            Expr::Bin(_, a, b) => a.has_fix() || b.has_fix(),
            Expr::Call(lib, xs) => lib.body.has_fix() || xs.iter().any(Expr::has_fix),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Expr::Bin(_, a, b) => 1 + a.size() + b.size(),
            Expr::Call(_, xs) => 1 + xs.iter().map(Expr::size).sum::<usize>(),
            Expr::Fix(fix) => 1 + fix.steps.iter().flatten().map(Expr::size).sum::<usize>() + fix.e.size(),
            _ => 1,
        }
    }
}

impl Expr {
    /// The expression written out, with the given names for its variables.
    pub fn render(&self, names: &[&str]) -> String {
        match self {
            Expr::Var(i) => names.get(*i).copied().unwrap_or("v").to_string(),
            Expr::Const(c) => c.to_string(),
            Expr::Bin(op, a, b) if matches!(op, Op::Add | Op::Sub | Op::Mul) => format!("({} {} {})", a.render(names), op.symbol(), b.render(names)),
            Expr::Bin(op, a, b) => format!("{}({}, {})", op.symbol(), a.render(names), b.render(names)),
            Expr::Call(lib, xs) => format!("{}({})", lib.name, xs.iter().map(|x| x.render(names)).collect::<Vec<_>>().join(", ")),
            Expr::Fix(fix) => format!("[{fix}]"),
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render(&["x", "y", "z", "w"]))
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
        // The step and the reduction (the step has one more variable, the value of the nearer point), and the base points the recurrence leaves.
        Expr::Fix(fix) => node + fix.steps.iter().flatten().map(|g| cost_of(g, arity, words)).sum::<Cost>() + cost_of(&fix.e, arity + fix.steps.len(), words) + fix.base_cost,
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
    search_inner(task, None, 0)
}

/// Like [`search`], modulo `m` if one is given, but up to `slack` of the data points may be got wrong: a rule that holds from some point on and has a
/// few base cases at the start still fits. Which points were missed is for the caller to find out; they have to be paid for.
pub fn search_with_slack(task: &Task, modulus: Option<i64>, slack: usize) -> Outcome {
    search_inner(task, modulus, slack)
}

/// Like [`search`], but the answers are those of the program *reduced modulo `m`*: a program fits if its value is never negative and leaves the
/// remainder the answer on division by `m`. The program found is the whole `mod(program, m)`. The modulus is nearly given by the data (every
/// answer is below it), and what is left to find is a program that only has to be right up to a multiple of `m`.
pub fn search_modular(task: &Task, m: i64) -> Outcome {
    search_inner(task, Some(m), 0)
}

fn search_inner(task: &Task, modulus: Option<i64>, slack: usize) -> Outcome {
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

    // What the reduction modulo m adds to a program: the `mod` node, and the node and the value of the constant.
    let wrap_cost = modulus.map_or(0, |m| 2 * node + bits::uint(m.unsigned_abs()));
    let wrap = |e: Expr| match modulus {
        Some(m) => Expr::Bin(Op::Mod, Box::new(e), Box::new(Expr::Const(m))),
        None => e,
    };
    let reduce = |v: i64| modulus.map_or(v, |m| v % m);
    let matches_answer = |v: i64, answer: i32| match modulus {
        Some(m) => v >= 0 && v % m == i64::from(answer),
        None => v == i64::from(answer),
    };
    let check_fit = |space: &Space, id: u32, size: usize, found: &mut Vec<Found>, first: &mut Option<usize>| {
        let row = space.row(id);
        let mut missed = 0;
        let fits = (0..n).all(|c| {
            missed += usize::from(!matches_answer(i64::from(row[c]), want[c]));
            missed <= slack
        });
        if fits {
            let probe_values = row[n..].iter().map(|&v| reduce(i64::from(v))).collect();
            found.push(Found { expr: wrap(space.expr(id, task.words)), cost: space.items[id as usize].cost + wrap_cost, probe_values });
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
        // The last level to be built only has to be checked against the data, and nearly every candidate fails at the first or second point: no
        // need to evaluate it everywhere, store it or tell it apart from the others. That level is the largest, so this is most of the saving.
        let last_level = match first_fit_size {
            Some(f) => size >= f + task.rival_size,
            None => size == task.max_size,
        };
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
                        let apply = |space: &Space, c: usize| {
                            let x = i64::from(space.row(a)[c]);
                            let y = if arity == 1 { 0 } else { i64::from(space.row(b)[c]) };
                            match fun {
                                Fun::Prim(op) => op.apply(x, y),
                                Fun::Word(w) => {
                                    let lib = &task.words[*w];
                                    lib.body.eval(&[x, y][..lib.arity])
                                }
                            }
                        };
                        if last_level {
                            let mut fits = true;
                            let mut missed = 0;
                            for c in 0..n {
                                effort.evaluations += 1;
                                if !apply(&space, c).is_some_and(|v| matches_answer(v, want[c])) {
                                    missed += 1;
                                    if missed > slack {
                                        fits = false;
                                        break;
                                    }
                                }
                            }
                            if !fits {
                                continue;
                            }
                            let mut probe_values = Vec::with_capacity(cols - n);
                            for c in n..cols {
                                effort.evaluations += 1;
                                match apply(&space, c) {
                                    Some(v) => probe_values.push(reduce(v)),
                                    None => {
                                        fits = false;
                                        break;
                                    }
                                }
                            }
                            if !fits {
                                continue;
                            }
                            let sub = space.items[a as usize].cost + if arity == 1 { 0 } else { space.items[b as usize].cost };
                            let (ea, eb) = (space.expr(a, task.words), space.expr(b, task.words));
                            let expr = match fun {
                                Fun::Prim(op) => Expr::Bin(*op, Box::new(ea), Box::new(eb)),
                                Fun::Word(w) => Expr::Call(task.words[*w].clone(), if arity == 1 { vec![ea] } else { vec![ea, eb] }),
                            };
                            found.push(Found { expr: wrap(expr), cost: node + sub + wrap_cost, probe_values });
                            first_fit_size.get_or_insert(size);
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

/// Programs one edit away from `best` that fit the data as well: a leaf or an operation changed, a constant swapped, or any part wrapped in an
/// operation with a constant (the way a ceiling or a wrap-around hides behind a rule). A rival differs from the best only where the data does
/// not say, which is exactly where doubt belongs, and finding them costs a few thousand evaluations instead of enumerating every larger program.
/// Each rival keeps the belief its cost allows.
pub fn rivals(best: &Expr, task: &Task) -> (Vec<Found>, Effort) {
    rivals_priced(best, task, None)
}

/// Like [`rivals`], but a program may get some data points wrong if it pays for each: `(price, cutoff)` gives the bits an exception costs at
/// every data point and the most a program may cost in all (its own bits plus its exceptions). A program that fits everything is the case where
/// nothing is paid.
pub fn rivals_priced(best: &Expr, task: &Task, priced: Option<(&[Cost], Cost)>) -> (Vec<Found>, Effort) {
    let n = task.inputs.len();
    let mut effort = Effort::default();
    // Constants for wrapping: the task's own, then the numbers just above and well above the largest value the best program takes on the data.
    let top = task.inputs.iter().filter_map(|p| best.eval(p)).max().unwrap_or(0).max(1);
    let mut wrap: Vec<i64> = task.consts.to_vec();
    for d in [0, 1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64] {
        wrap.push(top + d);
    }
    wrap.push(top * 2);
    wrap.push(top * 3);
    wrap.retain(|c| (0..LIMIT).contains(c));
    wrap.sort_unstable();
    wrap.dedup();
    let cost_best = cost_of(best, task.arity, task.words.len());
    let mut kept: Vec<Found> = Vec::new();
    for cand in variants(best, task, &wrap) {
        let own = cost_of(&cand, task.arity, task.words.len());
        let limit = priced.map_or(cost_best + 32 * bits::BIT, |(_, cutoff)| cutoff);
        if own > limit {
            continue;
        }
        // Cheapest test first: the data points, stopping at the first one it gets wrong (or, when exceptions are priced, as soon as it can no
        // longer afford them).
        let mut cost = own;
        let mut fits = true;
        for (i, (p, &t)) in task.inputs.iter().zip(task.target).take(n).enumerate() {
            effort.evaluations += cand.size() as u64;
            if cand.eval(p) != Some(t) {
                match priced {
                    Some((price, cutoff)) => {
                        cost += price[i];
                        if cost > cutoff {
                            fits = false;
                            break;
                        }
                    }
                    None => {
                        fits = false;
                        break;
                    }
                }
            }
        }
        if !fits {
            continue;
        }
        let mut probe_values = Vec::with_capacity(task.probes.len());
        let mut defined = true;
        for p in task.probes {
            effort.evaluations += cand.size() as u64;
            match cand.eval(p) {
                Some(v) => probe_values.push(v),
                None => {
                    defined = false;
                    break;
                }
            }
        }
        if defined {
            kept.push(Found { expr: cand, cost, probe_values });
        }
    }
    (kept, effort)
}

/// Every expression that differs from `e` by one edit.
fn variants(e: &Expr, task: &Task, wrap: &[i64]) -> Vec<Expr> {
    let mut out: Vec<Expr> = Vec::new();
    for op in OPS {
        for &c in wrap {
            out.push(Expr::Bin(op, Box::new(e.clone()), Box::new(Expr::Const(c))));
            if !op.commutative() {
                out.push(Expr::Bin(op, Box::new(Expr::Const(c)), Box::new(e.clone())));
            }
        }
    }
    match e {
        Expr::Var(v) => {
            out.extend((0..task.arity).filter(|u| u != v).map(Expr::Var));
            out.extend(task.consts.iter().map(|&c| Expr::Const(c)));
        }
        Expr::Const(k) => {
            out.extend(task.consts.iter().filter(|&c| c != k).map(|&c| Expr::Const(c)));
            out.extend((0..task.arity).map(Expr::Var));
        }
        Expr::Bin(op, a, b) => {
            out.extend(OPS.iter().filter(|o| *o != op).map(|&o| Expr::Bin(o, a.clone(), b.clone())));
            out.extend(variants(a, task, wrap).into_iter().map(|va| Expr::Bin(*op, Box::new(va), b.clone())));
            out.extend(variants(b, task, wrap).into_iter().map(|vb| Expr::Bin(*op, a.clone(), Box::new(vb))));
        }
        // A function defined by its own values is edited by wrapping, not inside.
        Expr::Fix(_) => {}
        Expr::Call(lib, xs) => {
            for i in 0..xs.len() {
                for vx in variants(&xs[i], task, wrap) {
                    let mut args = xs.clone();
                    args[i] = vx;
                    out.push(Expr::Call(lib.clone(), args));
                }
            }
        }
    }
    out
}

/// The search for the best program, then the programs one edit away from it.
pub fn search_with_rivals(task: &Task) -> Outcome {
    search_with_rivals_in(task, None)
}

/// [`search_with_rivals`], with the answers taken modulo `m` if one is given.
pub fn search_with_rivals_in(task: &Task, modulus: Option<i64>) -> Outcome {
    let mut out = search_inner(task, modulus, 0);
    if let Some(best) = out.found.first().map(|f| f.expr.clone()) {
        let (more, effort) = rivals(&best, task);
        out.effort.evaluations += effort.evaluations;
        out.found.extend(more);
        out.found.sort_by(|x, y| x.cost.cmp(&y.cost).then_with(|| x.expr.size().cmp(&y.expr.size())).then_with(|| x.expr.to_string().cmp(&y.expr.to_string())));
        let mut kept: Vec<Found> = Vec::new();
        for f in std::mem::take(&mut out.found) {
            if !kept.iter().any(|k| k.probe_values == f.probe_values) {
                kept.push(f);
            }
        }
        kept.truncate(64);
        out.found = kept;
    }
    out
}

/// Every behavior that `max_size` nodes can make over the starting operations, with its program, its cost and its values at the points (programs
/// that agree everywhere on the points are one). Used to propose the reductions of a recurrence.
pub fn pool(points: &[Vec<i64>], arity: usize, consts: &[i64], max_size: usize, max_items: usize) -> (Vec<(Expr, Cost, Vec<i64>)>, Effort) {
    let cols = points.len();
    let mut effort = Effort::default();
    let mut space = Space { cols, vals: Vec::new(), items: Vec::new(), seen: HashMap::default() };
    let node = node_cost(arity, 0);
    let blank = Item { kind: 0, op: Op::Add, word: 0, a: 0, b: 0, cost: node };
    let mut by_size: Vec<Vec<u32>> = vec![Vec::new(); max_size + 1];
    let mut row = vec![0i32; cols];
    for v in 0..arity {
        for (c, p) in points.iter().enumerate() {
            row[c] = p[v] as i32;
        }
        if let Some(id) = space.add(&row, Item { kind: 0, a: v as u32, ..blank }) {
            by_size[1].push(id);
        }
    }
    for &k in consts {
        row.iter_mut().for_each(|r| *r = k as i32);
        if let Some(id) = space.add(&row, Item { kind: 1, a: k as i32 as u32, cost: node + bits::uint(k.unsigned_abs()), ..blank }) {
            by_size[1].push(id);
        }
    }
    'sizes: for size in (3..=max_size).step_by(2) {
        for op in OPS {
            for left in (1..size - 1).step_by(2) {
                let right = size - 1 - left;
                if op.commutative() && left > right {
                    continue;
                }
                let (la, lb) = (by_size[left].clone(), by_size[right].clone());
                for (ia, &a) in la.iter().enumerate() {
                    for (ib, &b) in lb.iter().enumerate() {
                        if op.commutative() && left == right && ib < ia {
                            continue;
                        }
                        effort.evaluations += cols as u64;
                        let mut ok = true;
                        for c in 0..cols {
                            match op.apply(i64::from(space.row(a)[c]), i64::from(space.row(b)[c])) {
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
                        let cost = node + space.items[a as usize].cost + space.items[b as usize].cost;
                        if let Some(id) = space.add(&row, Item { kind: 2, op, a, b, cost, ..blank }) {
                            by_size[size].push(id);
                            if space.items.len() >= max_items {
                                break 'sizes;
                            }
                        }
                    }
                }
            }
        }
    }
    let out = (0..space.items.len() as u32).map(|i| (space.expr(i, &[]), space.items[i as usize].cost, space.row(i).iter().map(|&v| i64::from(v)).collect())).collect();
    (out, effort)
}

/// What the believable programs say at one point: the most believed answer, how much of the belief it carries, and the total belief (both in
/// the fixed-point units of [`bits::exp2_neg`]). Programs that are undefined at the point count toward the total but support no answer.
pub fn weigh(found: &[Found], at: &[i64]) -> Option<(i64, u64, u64)> {
    let best = found.first()?.cost;
    let mut votes: Vec<(i64, u64)> = Vec::new();
    let mut total = 0u64;
    for f in found {
        let w = bits::exp2_neg(f.cost - best);
        let value = f.expr.eval(at);
        // A recurrence whose chain does not reach a known value has nothing to say here: it abstains, it does not doubt.
        if value.is_none() && f.expr.has_fix() {
            continue;
        }
        total += w;
        if let Some(v) = value {
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
