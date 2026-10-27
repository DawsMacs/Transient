// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! Closed forms that are fitted instead of enumerated: the floor of a plane, `(m * x + n * y + c) div d`, and a sum of multiples reduced modulo a
//! constant, `(m * x + n * y + c) mod k`. Written as expressions of the language they take seven to nine nodes with constants no short list has,
//! so no enumeration reaches them; but each is a family with a few integers, and the integers are pinned by the data. A floor says, for every
//! point, `d * answer <= m * x + n * y + c < d * answer + d`: for a given `d` and slopes that is an interval for `c`, empty or not. The steps of
//! the answers, where they fall, are what reveals the slope as a ratio (a slope of 43/12 steps by 3 or 4, never by a fixed amount, which is why
//! the shape stage, which looks for a few places where the slope changes, gives such a kind nothing).
//!
//! The functions return every expression of the family that reproduces all the points (the cheapest to write first, a few per slope), so that
//! the learner can keep the ones that disagree beyond the data as rivals and state a doubt instead of a guess. They also report the work done, in the
//! same unit as the enumerations (one point checked against one candidate).

use crate::program::{Expr, Op};

/// The most expressions returned from one fit.
const MAX_FOUND: usize = 600;
/// Fewest points a fit is attempted on: a family with three or four integers is not pinned by fewer.
const MIN_FIT_POINTS: usize = 10;
/// Largest divisor tried, and the largest modulus.
const MAX_DIVISOR_UNARY: i64 = 128;
const MAX_DIVISOR_BINARY: i64 = 64;
const MAX_MODULUS: i64 = 200;

pub struct Fitted {
    pub exprs: Vec<Expr>,
    /// Points checked against candidates.
    pub work: u64,
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// `m * x + n * y + c` with the zero terms left out; none if there is nothing to write.
fn linear(coefs: &[i64], c: i64) -> Option<Expr> {
    let mut sum: Option<Expr> = None;
    for (v, &m) in coefs.iter().enumerate() {
        let term = match m {
            0 => continue,
            1 => Expr::Var(v),
            _ => Expr::Bin(Op::Mul, Box::new(Expr::Var(v)), Box::new(Expr::Const(m))),
        };
        sum = Some(match sum {
            None => term,
            Some(s) => Expr::Bin(Op::Add, Box::new(s), Box::new(term)),
        });
    }
    let sum = sum?;
    Some(if c == 0 { sum } else { Expr::Bin(Op::Add, Box::new(sum), Box::new(Expr::Const(c))) })
}

/// Whether the answers never fall as a variable rises, among points that agree on the other variable.
fn rises(inputs: &[Vec<i64>], target: &[i64]) -> bool {
    let arity = inputs[0].len();
    for v in 0..arity {
        let mut rows: Vec<(Vec<i64>, i64, i64)> = inputs.iter().zip(target).map(|(p, &t)| (p.iter().enumerate().filter(|(i, _)| *i != v).map(|(_, &x)| x).collect(), p[v], t)).collect();
        rows.sort();
        if rows.windows(2).any(|w| w[0].0 == w[1].0 && w[1].2 < w[0].2) {
            return false;
        }
    }
    true
}

fn spans(inputs: &[Vec<i64>]) -> Vec<i64> {
    (0..inputs[0].len()).map(|v| inputs.iter().map(|p| p[v]).max().unwrap_or(0) - inputs.iter().map(|p| p[v]).min().unwrap_or(0)).collect()
}

/// The slope along one variable, as a ratio, is pinned by every pair of points that differ only in it: the floor of a line lifts by its slope
/// times the distance, give or take less than 1. Returns the lowest and the highest the slope can be as `(numerator, denominator)`, or none if no
/// two points differ only in this variable.
fn slope_bracket(inputs: &[Vec<i64>], target: &[i64], v: usize) -> Option<((i64, i64), (i64, i64))> {
    let mut lo: (i64, i64) = (i64::MIN / 4, 1);
    let mut hi: (i64, i64) = (i64::MAX / 4, 1);
    let mut any = false;
    for (i, p) in inputs.iter().enumerate() {
        for (j, q) in inputs.iter().enumerate() {
            if q[v] > p[v] && (0..p.len()).all(|u| u == v || p[u] == q[u]) {
                any = true;
                let (dx, dt) = (q[v] - p[v], target[j] - target[i]);
                if (dt - 1) * lo.1 > lo.0 * dx {
                    lo = (dt - 1, dx);
                }
                if (dt + 1) * hi.1 < hi.0 * dx {
                    hi = (dt + 1, dx);
                }
            }
        }
    }
    any.then_some((lo, hi))
}

/// Every `(m * x + n * y + c) div d` (for one variable, `(m * x + c) div d`) that gives every answer, with `c >= 0` and the fraction in lowest terms.
/// For each divisor and slopes, `c` can be anything from `c_lo` to `c_hi`; the smallest `c` and the largest are both kept, since they can
/// disagree outside the data.
pub fn floors(inputs: &[Vec<i64>], target: &[i64]) -> Fitted {
    let mut out = Fitted { exprs: Vec::new(), work: 0 };
    let arity = inputs.first().map_or(0, Vec::len);
    if !(1..=2).contains(&arity) || inputs.len() < MIN_FIT_POINTS || target.iter().any(|&t| t < 0) || !rises(inputs, target) {
        return out;
    }
    let (tmin, tmax) = (target.iter().copied().min().unwrap_or(0), target.iter().copied().max().unwrap_or(0));
    if tmax - tmin < 3 {
        return out;
    }
    let span = spans(inputs);
    let max_d = if arity == 1 { MAX_DIVISOR_UNARY } else { MAX_DIVISOR_BINARY };
    // The slope along each variable is bracketed by the pairs of points that differ only in it (a variable that never varies has slope 0). If
    // the brackets cannot hold, nothing is a floor of a plane and nothing is tried: most kinds that are not floors fail here at once.
    let mut brackets: Vec<Option<((i64, i64), (i64, i64))>> = Vec::new();
    for v in 0..arity {
        if span[v] == 0 {
            brackets.push(None);
            continue;
        }
        match slope_bracket(inputs, target, v) {
            Some(b) if b.0 .0 * b.1 .1 <= b.1 .0 * b.0 .1 => brackets.push(Some(b)),
            _ => return out,
        }
    }
    // The numerators a divisor allows for each variable.
    let m_range = |d: i64, v: usize| -> (i64, i64) {
        match brackets[v] {
            None => (0, 0),
            Some((lo, hi)) => (((d * lo.0 + lo.1 - 1).div_euclid(lo.1)).max(0), (d * hi.0).div_euclid(hi.1)),
        }
    };
    for d in 1..=max_d {
        let ranges: Vec<(i64, i64)> = (0..arity).map(|v| m_range(d, v)).collect();
        let mut coefs = vec![0i64; arity];
        let visit = |coefs: &[i64], out: &mut Fitted| {
            if coefs.iter().all(|&m| m == 0) || coefs.iter().fold(d, |g, &m| gcd(g, m)) != 1 {
                return;
            }
            let (mut lo, mut hi) = (0i64, i64::MAX);
            for (p, &t) in inputs.iter().zip(target) {
                out.work += 1;
                let base: i64 = coefs.iter().zip(p).map(|(m, x)| m * x).sum();
                lo = lo.max(d * t - base);
                hi = hi.min(d * t + d - 1 - base);
                if lo > hi {
                    return;
                }
            }
            for c in if lo == hi { vec![lo] } else { vec![lo, hi] } {
                if let Some(sum) = linear(coefs, c) {
                    out.exprs.push(if d == 1 { sum } else { Expr::Bin(Op::Div, Box::new(sum), Box::new(Expr::Const(d))) });
                }
            }
        };
        if arity == 1 {
            for m in ranges[0].0..=ranges[0].1 {
                coefs[0] = m;
                visit(&coefs, &mut out);
            }
        } else {
            for m in ranges[0].0..=ranges[0].1 {
                for n in ranges[1].0..=ranges[1].1 {
                    coefs[0] = m;
                    coefs[1] = n;
                    visit(&coefs, &mut out);
                }
            }
        }
        if out.exprs.len() >= MAX_FOUND {
            break;
        }
    }
    out
}

/// Every `(m * x + n * y + c) mod k` that gives every answer, for the moduli just above the largest answer (as in the one-variable modular
/// stage): given the slopes, `c` is fixed by the first point. For one variable also `(a * x * x + b * x + c) mod k`, tried only if no line is.
pub fn modular(inputs: &[Vec<i64>], target: &[i64]) -> Fitted {
    let mut out = Fitted { exprs: Vec::new(), work: 0 };
    let arity = inputs.first().map_or(0, Vec::len);
    let top = target.iter().copied().max().unwrap_or(0);
    let distinct: std::collections::BTreeSet<i64> = target.iter().copied().collect();
    if !(1..=2).contains(&arity) || inputs.len() < MIN_FIT_POINTS || target.iter().any(|&t| t < 0) || top < 3 || top + 1 > MAX_MODULUS || distinct.len() < 4 {
        return out;
    }
    for k in top + 1..=(top + 3).min(MAX_MODULUS) {
        let try_coefs = |coefs: &[i64], out: &mut Fitted| {
            if coefs.iter().all(|&m| m == 0) {
                return;
            }
            let dot = |p: &[i64]| -> i64 { coefs.iter().zip(p).map(|(m, x)| m * x).sum() };
            let c = (target[0] - dot(&inputs[0])).rem_euclid(k);
            for (p, &t) in inputs.iter().zip(target) {
                out.work += 1;
                if (dot(p) + c).rem_euclid(k) != t {
                    return;
                }
            }
            if let Some(sum) = linear(coefs, c) {
                out.exprs.push(Expr::Bin(Op::Mod, Box::new(sum), Box::new(Expr::Const(k))));
            }
        };
        if arity == 1 {
            for m in 0..k {
                try_coefs(&[m], &mut out);
            }
        } else {
            for m in 0..k {
                for n in 0..k {
                    try_coefs(&[m, n], &mut out);
                }
            }
        }
        if out.exprs.len() >= MAX_FOUND {
            break;
        }
    }
    if arity == 1 && out.exprs.is_empty() {
        for k in quadratic_moduli(inputs, target, top) {
            for a in 1..k {
                for b in 0..k {
                    let value = |x: i64| a * x * x + b * x;
                    let c = (target[0] - value(inputs[0][0])).rem_euclid(k);
                    let mut fits = true;
                    for (p, &t) in inputs.iter().zip(target) {
                        out.work += 1;
                        if (value(p[0]) + c).rem_euclid(k) != t {
                            fits = false;
                            break;
                        }
                    }
                    if fits {
                        let square = Expr::Bin(Op::Mul, Box::new(Expr::Var(0)), Box::new(Expr::Var(0)));
                        let mut sum = if a == 1 { square } else { Expr::Bin(Op::Mul, Box::new(square), Box::new(Expr::Const(a))) };
                        if let Some(line) = linear(&[b], 0) {
                            sum = Expr::Bin(Op::Add, Box::new(sum), Box::new(line));
                        }
                        if c != 0 {
                            sum = Expr::Bin(Op::Add, Box::new(sum), Box::new(Expr::Const(c)));
                        }
                        out.exprs.push(Expr::Bin(Op::Mod, Box::new(sum), Box::new(Expr::Const(k))));
                    }
                }
            }
        }
    }
    out
}

/// The moduli a quadratic reduced modulo `k` could have: its second differences (over consecutive numbers) are all the same number modulo `k`,
/// so `k` divides every difference between two of them. The moduli are the divisors of the gcd of those that are above the largest answer.
fn quadratic_moduli(inputs: &[Vec<i64>], target: &[i64], top: i64) -> Vec<i64> {
    let at: std::collections::BTreeMap<i64, i64> = inputs.iter().map(|p| p[0]).zip(target.iter().copied()).collect();
    let seconds: Vec<i64> = at.keys().filter(|&&x| at.contains_key(&(x + 1)) && at.contains_key(&(x + 2))).map(|&x| at[&(x + 2)] - 2 * at[&(x + 1)] + at[&x]).collect();
    if seconds.len() < 4 {
        return Vec::new();
    }
    let g = seconds.iter().fold(0, |g, &s| gcd(g, (s - seconds[0]).abs()));
    if g == 0 {
        return Vec::new();
    }
    (top + 1..=MAX_MODULUS).filter(|k| g % k == 0).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_floor_of_a_line_is_recovered_with_its_ratio_in_lowest_terms() {
        let inputs: Vec<Vec<i64>> = (0..20).map(|x| vec![x]).collect();
        let target: Vec<i64> = (0..20).map(|x: i64| (43 * x + 65) / 12).collect();
        let f = floors(&inputs, &target);
        assert!(f.exprs.iter().any(|e| e.to_string().contains("43") && e.to_string().ends_with(", 12)")), "{:?}", f.exprs.iter().map(|e| e.to_string()).collect::<Vec<_>>());
        assert!(f.exprs.iter().all(|e| inputs.iter().zip(&target).all(|(p, &t)| e.eval(p) == Some(t))));
    }

    #[test]
    fn answers_that_fall_somewhere_are_not_a_floor() {
        let inputs: Vec<Vec<i64>> = (0..20).map(|x| vec![x]).collect();
        let target: Vec<i64> = (0..20).map(|x: i64| (x * 3) % 17).collect();
        assert!(floors(&inputs, &target).exprs.is_empty());
    }
}
