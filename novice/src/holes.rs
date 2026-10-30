// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! A constant that is found, not searched for. An enumeration that offers every constant as a leaf pays for the constants at every node; but a program with one
//! unknown constant beside parts that have none says what the constant must be, once those parts are known. For a part `E` (a program of up to five nodes with no
//! unknown) and the answers `y`:
//!
//! - `E mod c`: every point says `E - y` is a multiple of `c`, so `c` divides the gcd of those differences, and is larger than every answer;
//! - `E div c`: every point says `c * y <= E < c * y + c`, an interval for `c`;
//! - `E * c`, `E + c`, `E - c`, `c - E`: `c` is the same ratio or difference at every point;
//! - `max(E, c)`, `min(E, c)`: `c` is the answer where the answer is not `E`.
//!
//! The same holds one level up: beside a sibling `P` that has no unknown (`P + Q`, `P - Q`, `Q - P`, `P * Q`, `max(P, Q)`, `min(P, Q)`), the answers give
//! `Q`'s answers (all of them, or where `P` does not decide), and `Q = op(x, c)` is solved as above. Everything solved is checked against every point, so the
//! program returned is right on the data whatever the reasoning; what the reasoning saves is the enumeration of the constants.

use crate::bits::Cost;
use crate::fit::Fitted;
use crate::program::{self, Expr, Op};

/// Fewest points a constant is inferred from, and fewest points that must have said something about it.
const MIN_POINTS: usize = 10;
const MIN_KNOWN: usize = 6;
/// The most expressions returned, and the most constants kept for one part (the smallest ones that fit: where they differ beyond the data, they are rivals).
const MAX_FOUND: usize = 600;
const MOST_CANDIDATES: usize = 3;

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// How the unknown constant sits beside a part.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Hole {
    ModBy,
    DivBy,
    TimesBy,
    Plus,
    Minus,
    FromC,
    MaxWith,
    MinWith,
}

const ROOT_HOLES: [Hole; 8] = [Hole::ModBy, Hole::DivBy, Hole::TimesBy, Hole::Plus, Hole::Minus, Hole::FromC, Hole::MaxWith, Hole::MinWith];
const SIBLING_HOLES: [Hole; 3] = [Hole::ModBy, Hole::DivBy, Hole::TimesBy];

fn apply(hole: Hole, e: i64, c: i64) -> Option<i64> {
    match hole {
        Hole::ModBy => Op::Mod.apply(e, c),
        Hole::DivBy => Op::Div.apply(e, c),
        Hole::TimesBy => Op::Mul.apply(e, c),
        Hole::Plus => Op::Add.apply(e, c),
        Hole::Minus => Op::Sub.apply(e, c),
        Hole::FromC => Op::Sub.apply(c, e),
        Hole::MaxWith => Some(e.max(c)),
        Hole::MinWith => Some(e.min(c)),
    }
}

fn build(hole: Hole, e: &Expr, c: i64) -> Expr {
    let (e, k) = (Box::new(e.clone()), Box::new(Expr::Const(c)));
    match hole {
        Hole::ModBy => Expr::Bin(Op::Mod, e, k),
        Hole::DivBy => Expr::Bin(Op::Div, e, k),
        Hole::TimesBy => Expr::Bin(Op::Mul, e, k),
        Hole::Plus => Expr::Bin(Op::Add, e, k),
        Hole::Minus => Expr::Bin(Op::Sub, e, k),
        Hole::FromC => Expr::Bin(Op::Sub, k, e),
        Hole::MaxWith => Expr::Bin(Op::Max, e, k),
        Hole::MinWith => Expr::Bin(Op::Min, e, k),
    }
}

/// The constants that make `hole` applied to the values `e` give the answers `z` where they are known (`work` counts the points looked at).
fn candidates(hole: Hole, e: &[i64], z: &[Option<i64>], work: &mut u64) -> Vec<i64> {
    let known = z.iter().filter(|v| v.is_some()).count();
    if known < MIN_KNOWN {
        return Vec::new();
    }
    let at = |i: usize| z[i].map(|zi| (e[i], zi));
    match hole {
        Hole::ModBy => {
            let (mut g, mut top) = (0i64, 0i64);
            for i in 0..e.len() {
                let Some((ei, zi)) = at(i) else { continue };
                *work += 1;
                if ei < 0 || zi < 0 {
                    return Vec::new();
                }
                g = gcd(g, (ei - zi).abs());
                top = top.max(zi);
                // (the constant divides g and is above every answer: once g is no larger than an answer there is none)
                if g != 0 && g <= top {
                    return Vec::new();
                }
            }
            if g == 0 {
                return Vec::new();
            }
            let mut divisors: Vec<i64> = Vec::new();
            let mut d = 1i64;
            while d * d <= g {
                if g % d == 0 {
                    divisors.extend([d, g / d]);
                }
                d += 1;
            }
            divisors.sort_unstable();
            divisors.dedup();
            divisors.into_iter().filter(|&d| d > top && d >= 2).take(MOST_CANDIDATES).collect()
        }
        Hole::DivBy => {
            let (mut lo, mut hi) = (2i64, i64::MAX);
            for i in 0..e.len() {
                let Some((ei, zi)) = at(i) else { continue };
                *work += 1;
                if ei < 0 || zi < 0 {
                    return Vec::new();
                }
                lo = lo.max(ei / (zi + 1) + 1);
                if zi > 0 {
                    hi = hi.min(ei / zi);
                }
                if lo > hi {
                    return Vec::new();
                }
            }
            if hi == i64::MAX {
                return Vec::new();
            }
            (lo..=hi.min(lo + MOST_CANDIDATES as i64 - 1)).collect()
        }
        Hole::TimesBy => {
            let mut c: Option<i64> = None;
            for i in 0..e.len() {
                let Some((ei, zi)) = at(i) else { continue };
                *work += 1;
                match c {
                    None if ei != 0 => {
                        if zi % ei != 0 {
                            return Vec::new();
                        }
                        c = Some(zi / ei);
                    }
                    None => {
                        if zi != 0 {
                            return Vec::new();
                        }
                    }
                    Some(k) => {
                        if ei.checked_mul(k) != Some(zi) {
                            return Vec::new();
                        }
                    }
                }
            }
            c.filter(|&k| k >= 2).into_iter().collect()
        }
        Hole::Plus | Hole::Minus | Hole::FromC => {
            let mut c: Option<i64> = None;
            for i in 0..e.len() {
                let Some((ei, zi)) = at(i) else { continue };
                *work += 1;
                let k = match hole {
                    Hole::Plus => zi - ei,
                    Hole::Minus => ei - zi,
                    _ => zi + ei,
                };
                if *c.get_or_insert(k) != k {
                    return Vec::new();
                }
            }
            c.filter(|&k| k != 0).into_iter().collect()
        }
        Hole::MaxWith | Hole::MinWith => {
            let mut c: Option<i64> = None;
            for i in 0..e.len() {
                let Some((ei, zi)) = at(i) else { continue };
                *work += 1;
                let (decides, wrong) = if hole == Hole::MaxWith { (zi > ei, zi < ei) } else { (zi < ei, zi > ei) };
                if wrong {
                    return Vec::new();
                }
                if decides && *c.get_or_insert(zi) != zi {
                    return Vec::new();
                }
            }
            c.into_iter().collect()
        }
    }
}

/// How a part `P` with no unknown and a part with the unknown make the answers.
#[derive(Clone, Copy)]
enum Join {
    Add,
    PMinusQ,
    QMinusP,
    Mul,
    Max,
    Min,
}

const JOINS: [Join; 6] = [Join::Add, Join::PMinusQ, Join::QMinusP, Join::Mul, Join::Max, Join::Min];

/// What the unknown part must give at each point (None: not fixed by the data there), or None if no such part can make the answers.
fn needed(join: Join, p: &[i64], y: &[i64], work: &mut u64) -> Option<Vec<Option<i64>>> {
    let mut z = Vec::with_capacity(y.len());
    for i in 0..y.len() {
        *work += 1;
        z.push(match join {
            Join::Add => Some(y[i].checked_sub(p[i])?),
            Join::PMinusQ => Some(p[i].checked_sub(y[i])?),
            Join::QMinusP => Some(y[i].checked_add(p[i])?),
            Join::Mul => {
                if p[i] == 0 {
                    if y[i] != 0 {
                        return None;
                    }
                    None
                } else if y[i] % p[i] == 0 {
                    Some(y[i] / p[i])
                } else {
                    return None;
                }
            }
            Join::Max => match y[i].cmp(&p[i]) {
                std::cmp::Ordering::Greater => Some(y[i]),
                std::cmp::Ordering::Equal => None,
                std::cmp::Ordering::Less => return None,
            },
            Join::Min => match y[i].cmp(&p[i]) {
                std::cmp::Ordering::Less => Some(y[i]),
                std::cmp::Ordering::Equal => None,
                std::cmp::Ordering::Greater => return None,
            },
        });
    }
    Some(z)
}

fn join_values(join: Join, p: i64, q: i64) -> Option<i64> {
    match join {
        Join::Add => Op::Add.apply(p, q),
        Join::PMinusQ => Op::Sub.apply(p, q),
        Join::QMinusP => Op::Sub.apply(q, p),
        Join::Mul => Op::Mul.apply(p, q),
        Join::Max => Some(p.max(q)),
        Join::Min => Some(p.min(q)),
    }
}

fn join_expr(join: Join, p: &Expr, q: Expr) -> Expr {
    let p = Box::new(p.clone());
    let q = Box::new(q);
    match join {
        Join::Add => Expr::Bin(Op::Add, p, q),
        Join::PMinusQ => Expr::Bin(Op::Sub, p, q),
        Join::QMinusP => Expr::Bin(Op::Sub, q, p),
        Join::Mul => Expr::Bin(Op::Mul, p, q),
        Join::Max => Expr::Bin(Op::Max, p, q),
        Join::Min => Expr::Bin(Op::Min, p, q),
    }
}

/// The constants of the parts that stand alone (up to five nodes) and of the sibling beside a part with the unknown (up to three nodes).
pub const PART_CONSTS: [i64; 3] = [1, 2, 3];
pub const SIBLING_CONSTS: [i64; 11] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

/// Every program of the starting operations that gives the answers with one constant that the data fixes, seven nodes at most. The parts are made of the
/// variables and `PART_CONSTS` (up to five nodes) or `SIBLING_CONSTS` (a sibling of up to three nodes); the constant is found, whatever it is. `work` is
/// every point checked against every candidate, the building of the parts included.
pub fn find(inputs: &[Vec<i64>], target: &[i64], arity: usize) -> Fitted {
    if inputs.len() < MIN_POINTS || arity == 0 {
        return Fitted { exprs: Vec::new(), work: 0 };
    }
    let (parts, e1) = program::pool(inputs, arity, &PART_CONSTS, 5, 12_000);
    let (siblings, e2) = program::pool(inputs, arity, &SIBLING_CONSTS, 3, 4_000);
    let mut fitted = solve(inputs, target, arity, &parts, &siblings);
    fitted.work += e1.evaluations + e2.evaluations;
    fitted
}

/// [`find`] from pools that are already made (`program::pool`): the parts for the constant at the root, and the siblings.
pub fn solve(inputs: &[Vec<i64>], target: &[i64], arity: usize, pool: &[(Expr, Cost, Vec<i64>)], sibling_pool: &[(Expr, Cost, Vec<i64>)]) -> Fitted {
    let mut fitted = Fitted { exprs: Vec::new(), work: 0 };
    let n = inputs.len();
    let distinct: std::collections::BTreeSet<i64> = target.iter().copied().collect();
    if n < MIN_POINTS || distinct.len() < 4 || arity == 0 {
        return fitted;
    }
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let all: Vec<Option<i64>> = target.iter().map(|&t| Some(t)).collect();
    let mut work = 0u64;
    let mut found: Vec<Expr> = Vec::new();
    let mut keep = |e: Expr, found: &mut Vec<Expr>| {
        if found.len() < MAX_FOUND && seen.insert(e.to_string()) {
            found.push(e);
        }
    };
    // the constant at the root, beside any part of up to five nodes
    for (part, _, ev) in pool {
        if ev == target {
            continue;
        }
        for hole in ROOT_HOLES {
            for c in candidates(hole, ev, &all, &mut work) {
                work += n as u64;
                if (0..n).all(|i| apply(hole, ev[i], c) == Some(target[i])) {
                    keep(build(hole, part, c), &mut found);
                }
            }
        }
    }
    // the constant in a small part beside a sibling with none: the sibling and the answers say what the small part must give
    let siblings: Vec<&(Expr, Cost, Vec<i64>)> = sibling_pool.iter().filter(|(e, _, v)| e.size() <= 3 && v.iter().any(|x| *x != v[0])).collect();
    let variables: Vec<&(Expr, Cost, Vec<i64>)> = pool.iter().filter(|(e, _, _)| matches!(e, Expr::Var(_))).collect();
    for (p_expr, _, pv) in &siblings {
        for join in JOINS {
            let Some(z) = needed(join, pv, target, &mut work) else { continue };
            for (e_expr, _, ev) in &variables {
                for hole in SIBLING_HOLES {
                    for c in candidates(hole, ev, &z, &mut work) {
                        work += n as u64;
                        if (0..n).all(|i| apply(hole, ev[i], c).and_then(|q| join_values(join, pv[i], q)) == Some(target[i])) {
                            keep(join_expr(join, p_expr, build(hole, e_expr, c)), &mut found);
                        }
                    }
                }
            }
        }
    }
    found.retain(|e| inputs.iter().zip(target).all(|(p, &t)| e.eval(p) == Some(t)));
    fitted.work = work;
    fitted.exprs = found;
    fitted
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A reproducible spread of points (no randomness library: a mixing function).
    fn points(arity: usize, n: usize, top: i64) -> Vec<Vec<i64>> {
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = move || {
            state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            (z ^ (z >> 31)) as i64
        };
        let mut seen = std::collections::BTreeSet::new();
        let mut out = Vec::new();
        while out.len() < n {
            let p: Vec<i64> = (0..arity).map(|_| next().rem_euclid(top + 1)).collect();
            if seen.insert(p.clone()) {
                out.push(p);
            }
        }
        out
    }

    /// Whether `rule` is found: some expression returned gives every answer; and the work.
    fn finds(arity: usize, n: usize, top: i64, rule: impl Fn(&[i64]) -> i64) -> (bool, u64) {
        let inputs = points(arity, n, top);
        let target: Vec<i64> = inputs.iter().map(|p| rule(p)).collect();
        let fitted = find(&inputs, &target, arity);
        let right = fitted.exprs.iter().any(|e| inputs.iter().zip(&target).all(|(p, &t)| e.eval(p) == Some(t)));
        assert!(fitted.work < 800_000, "{} checks", fitted.work);
        (right, fitted.work)
    }

    #[test]
    fn a_modulus_is_found_from_the_differences_not_searched_for() {
        // a sum times a number reduced modulo a prime, and a product plus a number reduced modulo a prime (three numbers each)
        assert!(finds(3, 60, 30, |p| (p[0] + p[1]) * p[2] % 37).0, "(a + b) * c mod 37");
        assert!(finds(3, 90, 14, |p| (p[0] * p[1] + p[2]) % 19).0, "(a * b + c) mod 19");
        // a quadratic whose modulus lies well above the largest answer taught
        assert!(finds(1, 40, 60, |p| (p[0] * p[0] + 3) % 181).0, "(a * a + 3) mod 181");
    }

    #[test]
    fn a_divisor_a_factor_and_a_difference_are_found_from_the_data() {
        assert!(finds(2, 40, 60, |p| (p[0] * 3 + p[1]) / 7).0, "(a * 3 + b) div 7");
        assert!(finds(2, 40, 40, |p| p[0] * p[1] * 13).0, "a * b * 13");
        assert!(finds(2, 40, 40, |p| p[0] * p[1] + 37).0, "a * b + 37");
        assert!(finds(1, 30, 50, |p| p[0].max(17)).0, "max(a, 17)");
    }

    #[test]
    fn a_constant_beside_a_sibling_is_found_too() {
        // a remainder plus a half; five times a number plus a maximum; the larger of a quotient and a remainder
        assert!(finds(2, 60, 60, |p| p[0] % 11 + p[1] / 2).0, "a mod 11 + b div 2");
        assert!(finds(2, 40, 30, |p| 5 * p[0] + p[0].max(p[1])).0, "5 * a + max(a, b)");
        assert!(finds(1, 60, 90, |p| (p[0] % 6).max(p[0] / 4)).0, "max(a mod 6, a div 4)");
    }

    #[test]
    fn arbitrary_answers_are_not_fitted_and_the_work_is_small() {
        let inputs = points(2, 60, 60);
        let mut state = 7u64;
        let target: Vec<i64> = inputs
            .iter()
            .map(|_| {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                ((state >> 33) % 100) as i64
            })
            .collect();
        let fitted = find(&inputs, &target, 2);
        assert!(fitted.exprs.is_empty());
        assert!(fitted.work < 1_000_000, "{} checks", fitted.work);
    }
}
