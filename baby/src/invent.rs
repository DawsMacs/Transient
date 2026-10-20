// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DawsMacs
//! Can the baby invent a way to learn something nobody gave it a mechanism for?
//!
//! Nine ordinary kinds (sum, minus, after, double, half, times, apart, bigger, smaller) are taught first, in digits. Then come
//! "hidden" kinds that the mechanisms were not written for ("five times", "remainder by three", "take away, but not below zero",
//! "sum plus ten"...). Each is taught on small numbers only and tested on numbers it was never shown. Whether the baby worked a
//! kind out is judged the way everything is: right, refused, or WRONG, on questions it has not seen.
//!
//! A richer way of building things is only worth having if it does not start "explaining" what has no explanation, so a second set
//! of kinds must stay unexplained: arbitrary tables (a random code for every number) and structured things the language cannot
//! express (digit sums, primes). Those must end up memorised and must never be answered. The whole experiment reports both.

use crate::kinds::{Item, World};
use crate::rng::Rng;
use crate::scale::{fill, templ, word, BINARY, UNARY};
use crate::school::School;

#[derive(Clone)]
pub enum Gen {
    Fun(fn(i64, i64) -> Option<i64>),
    /// An arbitrary code for each number 0..=30: nothing to work out.
    Table(Vec<i64>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Role {
    /// Taught first; an ordinary kind.
    Base,
    /// Not given a mechanism: should be worked out, and reach numbers it was not shown.
    Hidden,
    /// Needs more than the base language can build in its size limit, but only things the baby has already learned (a constant it met
    /// in an earlier kind, a program it already built): should be worked out once it has that vocabulary, and not before.
    Stage2,
    /// Must stay unexplained.
    Unexplainable,
}

#[derive(Clone)]
pub struct Kind {
    pub name: &'static str,
    pub arity: usize,
    pub gen: Gen,
    pub role: Role,
    /// Largest number taught for a unary kind (the rest are "beyond").
    pub near: i64,
}

fn k(name: &'static str, arity: usize, f: fn(i64, i64) -> Option<i64>, role: Role) -> Kind {
    Kind { name, arity, gen: Gen::Fun(f), role, near: 15 }
}

fn nonneg(v: i64) -> Option<i64> {
    (v >= 0).then_some(v)
}

/// Teach the stage-2 kinds before the kinds that teach what they need (a control: the library should then not help).
/// Fraction of taught answers made wrong (set by `--noise`; thousandths).
pub static NOISE_PERMILLE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub static STAGE2_FIRST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn kinds(seed: u64) -> Vec<Kind> {
    let mut v = kinds_in_order(seed);
    if STAGE2_FIRST.load(std::sync::atomic::Ordering::Relaxed) {
        // stable: stage-2 kinds first, everything else in its order
        v.sort_by_key(|k| k.role != Role::Stage2);
    }
    v
}

fn kinds_in_order(seed: u64) -> Vec<Kind> {
    use Role::*;
    let mut v = vec![
        k("sum", 2, |a, b| Some(a + b), Base),
        k("minus", 2, |a, b| nonneg(a - b), Base),
        k("after", 1, |a, _| Some(a + 1), Base),
        k("double", 1, |a, _| Some(2 * a), Base),
        k("half", 1, |a, _| (a % 2 == 0).then_some(a / 2), Base),
        k("times", 2, |a, b| Some(a * b), Base),
        k("apart", 2, |a, b| Some((a - b).abs()), Base),
        k("bigger", 2, |a, b| (a != b).then_some(a.max(b)), Base),
        k("smaller", 2, |a, b| (a != b).then_some(a.min(b)), Base),
        // hidden, unary
        k("triple", 1, |a, _| Some(3 * a), Hidden),
        k("fivefold", 1, |a, _| Some(5 * a), Hidden),
        k("tenfold", 1, |a, _| Some(10 * a), Hidden),
        k("square", 1, |a, _| Some(a * a), Hidden),
        k("plusten", 1, |a, _| Some(a + 10), Hidden),
        k("modthree", 1, |a, _| Some(a % 3), Hidden),
        k("halfdown", 1, |a, _| Some(a / 2), Hidden),
        k("doubleone", 1, |a, _| Some(2 * a + 1), Hidden),
        // hidden, binary
        k("twicebig", 2, |a, b| Some(2 * a.max(b)), Hidden),
        k("takeaway", 2, |a, b| Some((a - b).max(0)), Hidden),
        k("prodone", 2, |a, b| Some(a * b + 1), Hidden),
        k("sumsq", 2, |a, b| Some(a * a + b * b), Hidden),
        k("avgdown", 2, |a, b| Some((a + b) / 2), Hidden),
        k("remainder", 2, |a, b| (b > 0).then(|| a % b), Hidden),
        k("sumten", 2, |a, b| Some(a + b + 10), Hidden),
        k("doublesum", 2, |a, b| Some(2 * (a + b)), Hidden),
        // stage 2: needs a constant (ten) or a whole earlier program as a piece
        k("tensdigit", 1, |a, _| Some(a / 10), Stage2),
        k("unitsdigit", 1, |a, _| Some(a % 10), Stage2),
        k("digitsum", 1, |a, _| Some(a / 10 + a % 10), Stage2),
        Kind { near: 25, ..k("digitsum_wide", 1, |a, _| Some(a / 10 + a % 10), Stage2) },
        k("tenplus", 2, |a, b| Some(10 * a + b), Stage2),
        k("sumsqone", 2, |a, b| Some(a * a + b * b + 1), Stage2),
        k("sumsqhalf", 2, |a, b| Some((a * a + b * b) / 2), Stage2),
        k("avgsq", 2, |a, b| Some(((a + b) / 2) * ((a + b) / 2)), Stage2),
        k("sqdiffsq", 2, |a, b| Some((a - b) * (a - b) + a * a), Stage2),
        // must stay unexplained
        k("prime", 1, |a, _| Some((a >= 2 && (2..a).all(|d| a % d != 0)) as i64), Unexplainable),
        k("xorfive", 1, |a, _| Some(a ^ 5), Unexplainable),
    ];
    let mut rng = Rng::new(seed.wrapping_add(555));
    for (i, name) in ["code_a", "code_b", "code_c", "code_d"].into_iter().enumerate() {
        let table: Vec<i64> = (0..=30).map(|_| rng.below(100) as i64).collect();
        let _ = i;
        v.push(Kind { name, arity: 1, gen: Gen::Table(table), role: Unexplainable, near: 15 });
    }
    v
}

fn eval(kind: &Kind, a: i64, b: i64) -> Option<i64> {
    match &kind.gen {
        Gen::Fun(f) => f(a, b),
        Gen::Table(t) => t.get(a as usize).copied(),
    }
}

pub struct Battery {
    pub names: Vec<String>,
    pub kinds: Vec<Kind>,
    pub world: World,
}

/// The lessons: for each kind, three phrasings of facts on small numbers are taught, the fourth is a rewording; a fifth of the small
/// facts are never taught; numbers above anything taught are the "beyond" questions.
pub fn battery(seed: u64) -> Battery {
    let ks = kinds(seed);
    let mut rng = Rng::new(seed.wrapping_add(9));
    let mut w = World { train: Vec::new(), reworded: Vec::new(), new_items: Vec::new(), beyond: Vec::new() };
    let mut names = Vec::new();
    let mut fact_id = 0usize;
    for (u, kind) in ks.iter().enumerate() {
        let attr = word(3_000_000 + u * 11 + 1, 4);
        names.push(format!("{}-{attr}", kind.name));
        let templates = templ(if kind.arity == 2 { BINARY } else { UNARY }, &attr);
        let (mut near, mut far) = (Vec::new(), Vec::new());
        if kind.arity == 1 {
            for a in 0..=30i64 {
                if let Some(y) = eval(kind, a, 0).filter(|y| *y <= 999) {
                    if a <= kind.near { near.push((vec![a], y)) } else { far.push((vec![a], y)) }
                }
            }
        } else {
            for a in 0..=30i64 {
                for b in 0..=30i64 {
                    if let Some(y) = eval(kind, a, b).filter(|y| *y <= 999) {
                        if a <= 12 && b <= 12 { near.push((vec![a, b], y)) } else { far.push((vec![a, b], y)) }
                    }
                }
            }
        }
        rng.shuffle(&mut near);
        rng.shuffle(&mut far);
        near.truncate(40);
        far.truncate(30);
        let held = if matches!(kind.gen, Gen::Table(_)) { 0 } else { ((near.len() as f32) * 0.2).round().max(1.0) as usize };
        for (i, (args, y)) in near.iter().enumerate() {
            let slots: Vec<String> = args.iter().map(|x| x.to_string()).collect();
            for (t, tpl) in templates.iter().enumerate() {
                let mut item = Item { unit: u, fact: fact_id, template: t, query: fill(tpl, "", &slots), answer: y.to_string() };
                let noise = NOISE_PERMILLE.load(std::sync::atomic::Ordering::Relaxed);
                if noise > 0 && i >= held && t < 3 && rng.below(1000) < noise {
                    item.answer = (y + 1 + rng.below(9) as i64).to_string();
                }
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
        for (args, y) in &far {
            let slots: Vec<String> = args.iter().map(|x| x.to_string()).collect();
            w.beyond.push(Item { unit: u, fact: fact_id, template: 0, query: fill(&templates[0], "", &slots), answer: y.to_string() });
            fact_id += 1;
        }
    }
    Battery { names, kinds: ks, world: w }
}

#[derive(Default, Clone, Copy)]
struct Score {
    right: usize,
    refused: usize,
    wrong: usize,
}

impl Score {
    fn n(&self) -> usize {
        self.right + self.refused + self.wrong
    }
    fn show(&self) -> String {
        format!("{:>3}/{:<3}/{:<3}", self.right, self.refused, self.wrong)
    }
}

fn score(school: &School, items: &[&Item]) -> Score {
    let mut s = Score::default();
    for it in items {
        let r = school.ask(&it.query);
        match r.answer {
            None => s.refused += 1,
            Some(a) if a == it.answer => s.right += 1,
            Some(a) => {
                s.wrong += 1;
                if std::env::var("INVENT_WRONG").is_ok() {
                    println!("   WRONG {:?} should be {}, said {a}: {}", it.query, it.answer, r.how);
                }
            }
        }
    }
    s
}

pub struct Outcome {
    pub hidden_solved: usize,
    pub hidden_total: usize,
    pub stage2_solved: usize,
    pub stage2_total: usize,
    pub invented_for_unexplainable: usize,
    pub unexplainable_total: usize,
    pub wrong_total: usize,
    /// (kind, role, worked out?) for every kind, so rates can be reported over many seeds.
    pub per_kind: Vec<(&'static str, Role, bool)>,
}

/// Teach everything, then report each kind. `quiet` suppresses the table.
pub fn run(seed: u64, quiet: bool) -> Outcome {
    let b = battery(seed);
    let school = School::from_items(&b.world.train, b.names.clone(), Some(&b.world.reworded), seed, true);
    if !quiet {
        println!("{:<10} {:<9} {:<9} {:>5}  {:<12} {:<12} {}", "kind", "role", "chose", "held", "never", "beyond", "(right/refused/WRONG)  how");
    }
    let mut out = Outcome { hidden_solved: 0, hidden_total: 0, stage2_solved: 0, stage2_total: 0, invented_for_unexplainable: 0, unexplainable_total: 0, wrong_total: 0, per_kind: Vec::new() };
    for (u, kind) in b.kinds.iter().enumerate() {
        let never: Vec<&Item> = b.world.new_items.iter().filter(|i| i.unit == u).collect();
        let beyond: Vec<&Item> = b.world.beyond.iter().filter(|i| i.unit == u).collect();
        let (sn, sb) = (score(&school, &never), score(&school, &beyond));
        let unit = &school.units[u];
        let how = unit.program().unwrap_or_default();
        out.wrong_total += sn.wrong + sb.wrong;
        out.per_kind.push((kind.name, kind.role.clone(), {
            let all = Score { right: sn.right + sb.right, refused: sn.refused + sb.refused, wrong: sn.wrong + sb.wrong };
            match kind.role {
                Role::Unexplainable => unit.chosen.name() != "remember",
                _ => all.wrong == 0 && all.right * 100 >= all.n() * 95,
            }
        }));
        match kind.role {
            Role::Hidden | Role::Stage2 => {
                if kind.role == Role::Stage2 {
                    out.stage2_total += 1;
                } else {
                    out.hidden_total += 1;
                }
                let all = Score { right: sn.right + sb.right, refused: sn.refused + sb.refused, wrong: sn.wrong + sb.wrong };
                if all.wrong == 0 && all.right * 100 >= all.n() * 95 {
                    if kind.role == Role::Stage2 {
                        out.stage2_solved += 1;
                    } else {
                        out.hidden_solved += 1;
                    }
                }
            }
            Role::Unexplainable => {
                out.unexplainable_total += 1;
                if unit.chosen.name() != "remember" {
                    out.invented_for_unexplainable += 1;
                }
            }
            Role::Base => {}
        }
        if !quiet {
            println!(
                "{:<10} {:<9} {:<9} {:>4.0}%  {}  {}  {}",
                kind.name,
                format!("{:?}", kind.role).to_lowercase(),
                unit.chosen.name(),
                100.0 * unit.best_evidence(),
                sn.show(),
                sb.show(),
                how
            );
        }
    }
    if !quiet {
        println!(
            "\nhidden kinds worked out (95% right on unseen numbers, 0 wrong): {} of {}\nstage-2 kinds worked out: {} of {}\nunexplainable kinds that got a mechanism: {} of {}\nWRONG answers in all: {}",
            out.hidden_solved, out.hidden_total, out.stage2_solved, out.stage2_total, out.invented_for_unexplainable, out.unexplainable_total, out.wrong_total
        );
    }
    out
}
