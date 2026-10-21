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
    /// Looks exactly like a rule or a selection on the taught numbers and changes regime outside them (a sum that saturates or wraps, a
    /// "first operand" that was only ever shown when it was the bigger): there is no competing explanation inside the baby's language to
    /// notice. It must never answer wrongly; whether it answers outside the taught region at all is the question.
    Deceptive,
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
    /// Taught only with the first operand bigger than the second (the rest are "beyond").
    pub ordered: bool,
}

fn k(name: &'static str, arity: usize, f: fn(i64, i64) -> Option<i64>, role: Role) -> Kind {
    Kind { name, arity, gen: Gen::Fun(f), role, near: 15, ordered: false }
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
        // deceptive: a rule or a selection on the taught numbers, something else beyond them
        k("capsum", 2, |a, b| Some((a + b).min(30)), Deceptive),
        k("wrapsum", 2, |a, b| Some((a + b) % 40), Deceptive),
        k("capdouble", 1, |a, _| Some((2 * a).min(30)), Deceptive),
        k("wraptriple", 1, |a, _| Some((3 * a) % 50), Deceptive),
        Kind { ordered: true, ..k("firstalways", 2, |a, _| Some(a), Deceptive) },
        // must stay unexplained
        k("prime", 1, |a, _| Some((a >= 2 && (2..a).all(|d| a % d != 0)) as i64), Unexplainable),
        k("xorfive", 1, |a, _| Some(a ^ 5), Unexplainable),
    ];
    let mut rng = Rng::new(seed.wrapping_add(555));
    for (i, name) in ["code_a", "code_b", "code_c", "code_d"].into_iter().enumerate() {
        let table: Vec<i64> = (0..=30).map(|_| rng.below(100) as i64).collect();
        let _ = i;
        v.push(Kind { name, arity: 1, gen: Gen::Table(table), role: Unexplainable, near: 15, ordered: false });
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
                        if a <= 12 && b <= 12 && (!kind.ordered || a > b) { near.push((vec![a, b], y)) } else { far.push((vec![a, b], y)) }
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
    /// Over the deceptive kinds: wrong answers, right answers, and questions asked.
    pub deceptive_wrong: usize,
    pub deceptive_right: usize,
    pub deceptive_total: usize,
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
    let mut out = Outcome { hidden_solved: 0, hidden_total: 0, stage2_solved: 0, stage2_total: 0, deceptive_wrong: 0, deceptive_right: 0, deceptive_total: 0, invented_for_unexplainable: 0, unexplainable_total: 0, wrong_total: 0, per_kind: Vec::new() };
    for (u, kind) in b.kinds.iter().enumerate() {
        let never: Vec<&Item> = b.world.new_items.iter().filter(|i| i.unit == u).collect();
        let beyond: Vec<&Item> = b.world.beyond.iter().filter(|i| i.unit == u).collect();
        let (sn, sb) = (score(&school, &never), score(&school, &beyond));
        let unit = &school.units[u];
        let how = unit.program().unwrap_or_default();
        if kind.role != Role::Deceptive {
            out.wrong_total += sn.wrong + sb.wrong;
        }
        out.per_kind.push((kind.name, kind.role.clone(), {
            let all = Score { right: sn.right + sb.right, refused: sn.refused + sb.refused, wrong: sn.wrong + sb.wrong };
            match kind.role {
                Role::Unexplainable => unit.chosen.name() != "remember",
                Role::Deceptive => all.wrong == 0,
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
            Role::Deceptive => {
                out.deceptive_wrong += sn.wrong + sb.wrong;
                out.deceptive_right += sn.right + sb.right;
                out.deceptive_total += sn.n() + sb.n();
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
            "\ndeceptive kinds (a rule or selection that changes regime outside the taught numbers): {} wrong answers, {} of {} unseen questions right\nhidden kinds worked out (95% right on unseen numbers, 0 wrong): {} of {}\nstage-2 kinds worked out: {} of {}\nunexplainable kinds that got a mechanism: {} of {}\nWRONG answers in all: {}",
            out.deceptive_wrong, out.deceptive_right, out.deceptive_total, out.hidden_solved, out.hidden_total, out.stage2_solved, out.stage2_total, out.invented_for_unexplainable, out.unexplainable_total, out.wrong_total
        );
    }
    out
}

/// What happened to one kind when the baby was allowed to ask.
pub struct ActiveResult {
    pub kind: &'static str,
    pub role: Role,
    /// Percent of unseen questions (never taught, and beyond the taught numbers) answered rightly: before any question, then after each.
    pub asking: Vec<f32>,
    /// The same with a teacher who gives random numbers instead.
    pub random: Vec<f32>,
    pub wrong: usize,
    /// How many questions it actually had to ask (it asks only when explanations compete).
    pub questions: usize,
}

fn coverage(school: &School, b: &Battery, u: usize) -> (f32, usize) {
    let never: Vec<&Item> = b.world.new_items.iter().filter(|i| i.unit == u).collect();
    let beyond: Vec<&Item> = b.world.beyond.iter().filter(|i| i.unit == u).collect();
    let (sn, sb) = (score(school, &never), score(school, &beyond));
    let n = (sn.n() + sb.n()).max(1);
    (100.0 * (sn.right + sb.right) as f32 / n as f32, sn.wrong + sb.wrong)
}

/// The numbers a question is about, read back from its digits.
fn numbers_in(q: &str) -> Vec<i64> {
    crate::data::tokenize(q).iter().filter_map(|w| w.parse().ok()).collect()
}

/// For the kinds that are not fully worked out by ordinary teaching: let the baby ask, one question at a time (`limit` at most), answer from
/// the truth, and record how much of the unseen it then gets right; then do the same with a teacher giving random numbers. Each teacher gets
/// its own school, taught once; every kind is its own unit, so they share it.
pub fn run_active(seed: u64, limit: usize) -> Vec<ActiveResult> {
    let b = battery(seed);
    let fresh = || School::from_items(&b.world.train, b.names.clone(), Some(&b.world.reworded), seed, true);
    let mut school = fresh();
    let mut school_random = fresh();
    let mut results = Vec::new();
    for (u, kind) in b.kinds.iter().enumerate() {
        let (start, w0) = coverage(&school, &b, u);
        if std::env::var("ACTIVE_ONLY").map_or(false, |o| o != kind.name) {
            continue;
        }
        if kind.role == Role::Base || (start >= 99.9 && w0 == 0 && kind.role != Role::Unexplainable) {
            continue;
        }
        let name = b.names[u].clone();
        if std::env::var("ACTIVE_DEBUG").is_ok() {
            eprintln!("START {} {}", kind.name, school.describe_unit_open(u));
        }
        let mut asking = vec![start];
        let mut wrong = w0;
        let mut questions = 0;
        for _ in 0..limit {
            let last = *asking.last().unwrap();
            let wonders = school.wonder_about(Some(&name));
            if std::env::var("ACTIVE_DEBUG").is_ok() {
                eprintln!("  wonders for {}: {:?}", kind.name, wonders.iter().map(|w| w.question.clone()).collect::<Vec<_>>());
            }
            let next = match wonders.into_iter().next() {
                Some(w) => {
                    let nums = numbers_in(&w.question);
                    match eval(kind, nums[0], *nums.get(1).unwrap_or(&0)) {
                        Some(truth) if { let r = school.learn_wondered(&w.question, &truth.to_string(), &name); if std::env::var("ACTIVE_DEBUG").is_ok() { eprintln!("  learn_wondered -> {:?}", r.as_ref().map(|l| l.surprised)); } r.is_ok() } => {
                            if std::env::var("ACTIVE_DEBUG").is_ok() {
                                eprintln!("  {} asked {:?} -> {truth}; unit now {} ({:.0}%) {}", kind.name, w.question, school.units[u].chosen.name(), 100.0 * school.units[u].best_evidence(), school.units[u].program().unwrap_or_default());
                                let never: Vec<&Item> = b.world.new_items.iter().chain(b.world.beyond.iter()).filter(|i| i.unit == u).collect();
                                for it in never.iter().filter(|i| school.ask(&i.query).answer.as_deref() != Some(i.answer.as_str())).take(4) {
                                    eprintln!("      still not right: {:?} want {} got {:?}: {}", it.query, it.answer, school.ask(&it.query).answer, school.ask(&it.query).how);
                                }
                            }
                            questions += 1;
                            let (c, w2) = coverage(&school, &b, u);
                            wrong = wrong.max(w2);
                            c
                        }
                        _ => last,
                    }
                }
                None => last,
            };
            asking.push(next);
        }
        // the random teacher
        let mut rng = Rng::new(seed.wrapping_add(1234 + u as u64));
        let template = b.world.train.iter().find(|i| i.unit == u).map(|i| i.query.clone()).unwrap_or_default();
        let mut tried: std::collections::HashSet<(i64, i64)> =
            b.world.train.iter().filter(|i| i.unit == u).map(|i| { let n = numbers_in(&i.query); (n[0], *n.get(1).unwrap_or(&0)) }).collect();
        let mut random = vec![coverage(&school_random, &b, u).0];
        for _ in 0..limit {
            let mut given = false;
            for _ in 0..50 {
                let a = rng.below(31) as i64;
                let c = if kind.arity == 2 { rng.below(31) as i64 } else { 0 };
                if !tried.insert((a, c)) {
                    continue;
                }
                if let Some(truth) = eval(kind, a, c) {
                    let toks = crate::data::tokenize(&template);
                    let mut nums = vec![a, c].into_iter();
                    let q: Vec<String> = toks.iter().map(|t| if t.parse::<i64>().is_ok() { nums.next().unwrap_or(0).to_string() } else { t.clone() }).collect();
                    let _ = school_random.learn_wondered(&q.join(" "), &truth.to_string(), &name);
                    given = true;
                    break;
                }
            }
            let _ = given;
            let (c, w2) = coverage(&school_random, &b, u);
            wrong = wrong.max(w2);
            random.push(c);
        }
        results.push(ActiveResult { kind: kind.name, role: kind.role.clone(), asking, random, wrong, questions });
    }
    results
}

/// What happened to one kind when its unit was allowed to check its own explanation.
pub struct CheckResult {
    pub kind: &'static str,
    pub asked: usize,
    pub refuted: usize,
    pub before: (f32, usize),
    pub after: (f32, usize),
    /// after the checks AND following up on what it then wonders about
    pub settled: (f32, usize),
}

/// For every kind whose unit holds a single program: let it ask `limit` self-checking questions, answer from the truth, and see whether the
/// explanation survived (and how accurate it is afterwards). A refuted explanation is replaced or dropped; a confirmed one costs a question.
pub fn run_check(seed: u64, limit: usize) -> Vec<CheckResult> {
    let b = battery(seed);
    let mut school = School::from_items(&b.world.train, b.names.clone(), Some(&b.world.reworded), seed, true);
    let mut results = Vec::new();
    for (u, kind) in b.kinds.iter().enumerate() {
        let name = b.names[u].clone();
        let checks: Vec<_> = school.check_myself(limit).into_iter().filter(|c| c.unit == name).collect();
        if checks.is_empty() {
            continue;
        }
        if std::env::var("CHECK_ONLY").map_or(false, |o| o != kind.name) {
            continue;
        }
        let before = coverage(&school, &b, u);
        if std::env::var("CHECK_DEBUG").is_ok() {
            eprintln!("BEFORE {} {} cov {:?}", kind.name, school.describe_unit_open(u), before);
        }
        let (mut asked, mut refuted) = (0, 0);
        for c in checks {
            let nums = numbers_in(&c.question);
            let Some(truth) = eval(kind, nums[0], *nums.get(1).unwrap_or(&0)) else { continue };
            let said = school.ask(&c.question).answer;
            if std::env::var("CHECK_DEBUG").is_ok() {
                eprintln!("   asks {:?} truth {truth} it said {:?}", c.question, said);
            }
            // not confirmed: it said something else, or declined at the very number it was asked about
            if said.as_deref() != Some(truth.to_string().as_str()) {
                refuted += 1;
            }
            if school.learn_wondered(&c.question, &truth.to_string(), &name).is_ok() {
                asked += 1;
            }
        }
        if std::env::var("CHECK_DEBUG").is_ok() {
            eprintln!("AFTER  {} {} cov {:?}", kind.name, school.describe_unit_open(u), coverage(&school, &b, u));
        }
        let after = coverage(&school, &b, u);
        for _ in 0..3 {
            let Some(w) = school.wonder_about(Some(&name)).into_iter().next() else { break };
            let nums = numbers_in(&w.question);
            let Some(truth) = eval(kind, nums[0], *nums.get(1).unwrap_or(&0)) else { break };
            let _ = school.learn_wondered(&w.question, &truth.to_string(), &name);
        }
        results.push(CheckResult { kind: kind.name, asked, refuted, before, after, settled: coverage(&school, &b, u) });
    }
    results
}

/// Unseen-question results over a set of kinds: right, refused and wrong.
#[derive(Default, Clone, Copy)]
pub struct Tally3 {
    pub right: usize,
    pub refused: usize,
    pub wrong: usize,
}

pub struct CarefulResult {
    /// For deceptive kinds and for genuine ones (hidden, stage-2 and base): permissive, then careful after 0, 1, 2, ... rounds of checks.
    pub deceptive: Vec<Tally3>,
    pub genuine: Vec<Tally3>,
    pub asked: usize,
}

fn tally(school: &School, b: &Battery, want: &dyn Fn(&Role) -> bool) -> Tally3 {
    let mut t = Tally3::default();
    for (u, kind) in b.kinds.iter().enumerate() {
        if !want(&kind.role) {
            continue;
        }
        let items: Vec<&Item> = b.world.new_items.iter().chain(b.world.beyond.iter()).filter(|i| i.unit == u).collect();
        let s = score(school, &items);
        t.right += s.right;
        t.refused += s.refused;
        t.wrong += s.wrong;
    }
    t
}

/// Permissive vs careful, and careful with `rounds` rounds of self-checks (`per_unit` questions each) answered from the truth.
pub fn run_careful(seed: u64, rounds: usize, per_unit: usize) -> CarefulResult {
    let b = battery(seed);
    let mut school = School::from_items(&b.world.train, b.names.clone(), Some(&b.world.reworded), seed, true);
    let deceptive = |r: &Role| *r == Role::Deceptive;
    let genuine = |r: &Role| matches!(r, Role::Base | Role::Hidden | Role::Stage2);
    let mut res = CarefulResult { deceptive: vec![tally(&school, &b, &deceptive)], genuine: vec![tally(&school, &b, &genuine)], asked: 0 };
    if std::env::var("CAREFUL_DEBUG").is_ok() {
        eprintln!("LIBRARY at start:\n{}", school.library_text());
    }
    school.set_careful(true);
    res.deceptive.push(tally(&school, &b, &deceptive));
    res.genuine.push(tally(&school, &b, &genuine));
    for _ in 0..rounds {
        for (u, kind) in b.kinds.iter().enumerate() {
            if kind.role == Role::Unexplainable || std::env::var("CAREFUL_ONLY").map_or(false, |o| o != kind.name) {
                continue;
            }
            let name = b.names[u].clone();
            let checks: Vec<_> = school.check_myself(12).into_iter().filter(|c| c.unit == name).collect();
            // only the edge questions: gaps are the business of the program checks
            for c in checks.into_iter().filter(|c| c.why.contains("past") || c.why.contains("other way") || c.why.contains("below")).take(per_unit) {
                let nums = numbers_in(&c.question);
                let Some(truth) = eval(kind, nums[0], *nums.get(1).unwrap_or(&0)) else { continue };
                if std::env::var("CAREFUL_DEBUG").is_ok() {
                    eprintln!("  round asks {:?} truth {truth} it says {:?}", c.question, school.ask(&c.question).answer);
                }
                if school.learn_wondered(&c.question, &truth.to_string(), &name).is_ok() {
                    res.asked += 1;
                    if std::env::var("CAREFUL_DEBUG").is_ok() {
                        eprintln!("      unit now {}", school.describe_unit_open(u));
                    }
                }
            }
        }
        res.deceptive.push(tally(&school, &b, &deceptive));
        res.genuine.push(tally(&school, &b, &genuine));
        if std::env::var("CAREFUL_DEBUG").is_ok() {
            eprintln!("LIBRARY after a round:\n{}", school.library_text());
        }
    }
    if std::env::var("CAREFUL_DEBUG").is_ok() {
        for (u, kind) in b.kinds.iter().enumerate().filter(|(_, k)| matches!(k.role, Role::Base | Role::Hidden | Role::Stage2)) {
            let items: Vec<&Item> = b.world.new_items.iter().chain(b.world.beyond.iter()).filter(|i| i.unit == u).collect();
            let refused: Vec<&&Item> = items.iter().filter(|i| school.ask(&i.query).answer.is_none()).collect();
            let wrong: Vec<&&Item> = items.iter().filter(|i| school.ask(&i.query).answer.map_or(false, |a| a != i.answer)).collect();
            if let Some(w) = wrong.first() {
                eprintln!("WRONG {:<12} {} wrong, e.g. {:?} want {} got {:?}: {}", kind.name, wrong.len(), w.query, w.answer, school.ask(&w.query).answer, school.ask(&w.query).how);
            }
            if let Some(first) = refused.first() {
                eprintln!("{:<12} refused {:>2} of {:>2}, e.g. {:?}: {}", kind.name, refused.len(), items.len(), first.query, school.ask(&first.query).how);
            }
        }
    }
    res
}
