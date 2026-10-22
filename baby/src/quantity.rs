// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! A unit that learns a rule instead of a table.
//!
//! Every network so far memorised sums: given "three plus two" it learned the pair, and "three plus one" was a stranger. A rule
//! needs something a table lacks: a notion of *quantity*. This unit has two parts, both learned from experience:
//!
//!   1. A number line, built the way a child gets one, by being taught to count: "what comes after three? four". Each word's value
//!      is just how many steps from zero. Words never used in a sum (six, seven...) are on the line too, so the rule can reach them.
//!   2. An operator: for every other word in the question, three learned numbers (a, b, c). The question's prediction is
//!      a*first + b*second + c, summed over its words. "plus" ends up (1, 1, 0) and "minus" (1, -1, 0), found by fitting a handful
//!      of examples, not by being told.
//!
//! The answer is the number word whose value is nearest the prediction; the distance is its (honest) uncertainty.

use crate::data::{tokenize, Example, NUM};
use std::collections::HashMap;

pub struct NumberLine {
    pub(crate) value: HashMap<String, f32>,
}

/// The largest number written in digits that counts as a number ("9999"). A longer string of digits is a code or a year, not a quantity.
const MAX_DIGITS: usize = 4;

/// The value of a word written in digits ("17"), if it is one.
pub fn digits_value(w: &str) -> Option<f32> {
    if (1..=MAX_DIGITS).contains(&w.len()) && w.bytes().all(|b| b.is_ascii_digit()) {
        w.parse::<u32>().ok().map(|v| v as f32)
    } else {
        None
    }
}

/// Is this word a number, in words or in digits?
pub fn is_number(w: &str) -> bool {
    NUM.contains(&w) || digits_value(w).is_some()
}

/// Does this question write any of its numbers in digits? Then it is answered in digits.
pub fn uses_digits(q: &str) -> bool {
    tokenize(q).iter().any(|w| digits_value(w).is_some())
}

/// The counting lessons: (word, the word that comes after it).
pub fn counting_lessons() -> Vec<(String, String)> {
    NUM.windows(2).map(|w| (w[0].to_string(), w[1].to_string())).collect()
}

impl NumberLine {
    /// Learn the line from "x comes after y" lessons, anchored at the word with nothing before it.
    pub fn from_counting(lessons: &[(String, String)]) -> NumberLine {
        let next: HashMap<&str, &str> = lessons.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let after: std::collections::HashSet<&str> = lessons.iter().map(|(_, b)| b.as_str()).collect();
        let mut value = HashMap::new();
        for (start, _) in lessons {
            if !after.contains(start.as_str()) {
                let mut word = start.as_str();
                let mut v = 0.0f32;
                loop {
                    value.insert(word.to_string(), v);
                    match next.get(word) {
                        Some(n) => {
                            word = n;
                            v += 1.0;
                        }
                        None => break,
                    }
                }
            }
        }
        NumberLine { value }
    }

    /// A number word's value, or a digit string's.
    pub fn value(&self, w: &str) -> Option<f32> {
        self.value.get(w).copied().or_else(|| digits_value(w))
    }

    /// Say a result as a number: in digits (when the question used them) if it lands on a whole number, else the nearest number word.
    /// The second part is how far off it was. A negative or oversized result is never said.
    pub fn say(&self, y: f32, digits: bool) -> Option<(String, f32)> {
        if digits {
            let r = y.round();
            if r < 0.0 || r > 9999.0 {
                return None;
            }
            Some(((r as u32).to_string(), (y - r).abs()))
        } else {
            self.nearest(y)
        }
    }

    /// The word whose value is closest, and how far off it is.
    pub fn nearest(&self, y: f32) -> Option<(String, f32)> {
        self.value
            .iter()
            .map(|(w, v)| (w.clone(), (v - y).abs()))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap().then_with(|| a.0.cmp(&b.0)))
    }
}

pub struct Quantity {
    pub line: NumberLine,
    /// (a, b, c) for each non-number word.
    pub op: HashMap<String, [f32; 3]>,
}

const SCALE: f32 = 10.0;

impl Quantity {
    /// The question's two numbers (0 when absent) and its other words; none if it contains no number at all.
    fn parse(&self, q: &str) -> Option<([f32; 2], usize, Vec<String>)> {
        let mut nums = Vec::new();
        let mut rest = Vec::new();
        for w in tokenize(q) {
            match self.line.value(&w) {
                Some(v) => nums.push(v),
                None => rest.push(w),
            }
        }
        if nums.is_empty() || nums.len() > 2 {
            return None;
        }
        Some(([nums[0], *nums.get(1).unwrap_or(&0.0)], nums.len(), rest))
    }

    fn predict(&self, nums: &[f32; 2], rest: &[String]) -> f32 {
        let f = [nums[0] / SCALE, nums[1] / SCALE, 1.0];
        let mut y = 0.0;
        for w in rest {
            if let Some(o) = self.op.get(w) {
                y += o[0] * f[0] + o[1] * f[1] + o[2] * f[2];
            }
        }
        y * SCALE
    }

    /// Fit from solved questions (every example whose question has a number and whose answer is a number word).
    pub fn fit(train: &[Example], line: NumberLine, epochs: usize, lr: f32) -> Quantity {
        Quantity::fit_penalized(train, line, epochs, lr, &|_| 0.0)
    }

    /// As `fit`, with a pull towards zero on chosen words. Examples alone cannot always say which of two words that always turn
    /// up together does the work ("what comes after four" / "what is after four"); a word that is common in everything else
    /// the baby has been taught is a poor suspect, so such words are pulled harder (`decay(word)` per update).
    pub fn fit_penalized(train: &[Example], line: NumberLine, epochs: usize, lr: f32, decay: &dyn Fn(&str) -> f32) -> Quantity {
        let mut q = Quantity { line, op: HashMap::new() };
        let mut data: Vec<([f32; 2], Vec<String>, f32)> = Vec::new();
        for e in train {
            if let (Some((nums, _, rest)), Some(y)) = (q.parse(&e.query), q.line.value(&e.answer)) {
                data.push((nums, rest, y));
            }
        }
        for (_, rest, _) in &data {
            for w in rest {
                q.op.entry(w.clone()).or_insert([0.0; 3]);
            }
        }
        for _ in 0..epochs {
            for (nums, rest, y) in &data {
                let err = q.predict(nums, rest) / SCALE - y / SCALE;
                let f = [nums[0] / SCALE, nums[1] / SCALE, 1.0];
                for w in rest {
                    let pull = (lr * decay(w)).min(0.5);
                    let o = q.op.get_mut(w).unwrap();
                    for j in 0..3 {
                        o[j] -= lr * 2.0 * err * f[j];
                        o[j] -= pull * o[j];
                    }
                }
            }
        }
        q
    }

    /// Fit the same rule by solving it exactly instead of nudging it thousands of times. The model is linear in the operator weights,
    /// so the best fit is the solution of one small system of equations (ridge regression); no learning rate, no number of passes,
    /// and the same answer every time. `ridge(word)` is how hard to pull a word's weights towards zero (common filler words are
    /// poor suspects). Returns the rule and the arithmetic operations spent.
    pub fn fit_ridge(train: &[Example], line: NumberLine, ridge: &dyn Fn(&str) -> f64) -> (Quantity, u64) {
        let mut q = Quantity { line, op: HashMap::new() };
        let mut data: Vec<([f32; 2], Vec<String>, f32)> = Vec::new();
        for e in train {
            if let (Some((nums, _, rest)), Some(y)) = (q.parse(&e.query), q.line.value(&e.answer)) {
                data.push((nums, rest, y));
            }
        }
        let mut words: Vec<String> = data.iter().flat_map(|d| d.1.iter().cloned()).collect();
        words.sort();
        words.dedup();
        let index: HashMap<&str, usize> = words.iter().enumerate().map(|(i, w)| (w.as_str(), i)).collect();
        let dim = 3 * words.len();
        let mut a = vec![vec![0.0f64; dim]; dim];
        let mut b = vec![0.0f64; dim];
        let mut ops = 0u64;
        for (nums, rest, y) in &data {
            let f = [nums[0] as f64 / SCALE as f64, nums[1] as f64 / SCALE as f64, 1.0];
            let target = *y as f64 / SCALE as f64;
            let mut x = vec![0.0f64; dim];
            for w in rest {
                let at = 3 * index[w.as_str()];
                for j in 0..3 {
                    x[at + j] += f[j];
                }
            }
            for r in 0..dim {
                if x[r] == 0.0 {
                    continue;
                }
                b[r] += x[r] * target;
                for c in 0..dim {
                    a[r][c] += x[r] * x[c];
                }
                ops += dim as u64 + 1;
            }
        }
        for (wi, w) in words.iter().enumerate() {
            for j in 0..3 {
                a[3 * wi + j][3 * wi + j] += 1e-6 + ridge(w);
            }
        }
        // Solve a theta = b by Gaussian elimination with partial pivoting.
        for col in 0..dim {
            let pivot = (col..dim).max_by(|&x, &y| a[x][col].abs().partial_cmp(&a[y][col].abs()).unwrap()).unwrap_or(col);
            a.swap(col, pivot);
            b.swap(col, pivot);
            let p = a[col][col];
            if p.abs() < 1e-12 {
                continue;
            }
            for r in col + 1..dim {
                let factor = a[r][col] / p;
                if factor == 0.0 {
                    continue;
                }
                for c in col..dim {
                    a[r][c] -= factor * a[col][c];
                }
                b[r] -= factor * b[col];
                ops += (dim - col) as u64 + 1;
            }
        }
        let mut theta = vec![0.0f64; dim];
        for r in (0..dim).rev() {
            let mut s = b[r];
            for c in r + 1..dim {
                s -= a[r][c] * theta[c];
            }
            theta[r] = if a[r][r].abs() < 1e-12 { 0.0 } else { s / a[r][r] };
            ops += (dim - r) as u64;
        }
        for (wi, w) in words.iter().enumerate() {
            q.op.insert(w.clone(), [theta[3 * wi] as f32, theta[3 * wi + 1] as f32, theta[3 * wi + 2] as f32]);
        }
        (q, ops)
    }

    /// Rough count of the arithmetic operations the iterative fit spends: every pass over every question touches each of its words.
    pub fn iterative_fit_ops(train: &[Example], line: &NumberLine, epochs: usize) -> u64 {
        let q = Quantity { line: NumberLine { value: line.value.clone() }, op: HashMap::new() };
        let per_pass: u64 = train
            .iter()
            .filter_map(|e| q.parse(&e.query).filter(|_| q.line.value(&e.answer).is_some()).map(|(_, _, rest)| rest.len() as u64))
            .map(|tokens| tokens * 14 + 8)
            .sum();
        per_pass * epochs as u64
    }

    /// Rebuild from saved operator weights (the number line comes from the counting lessons).
    pub fn from_ops(ops: Vec<(String, [f32; 3])>) -> Quantity {
        Quantity { line: NumberLine::from_counting(&counting_lessons()), op: ops.into_iter().collect() }
    }

    /// The weights of one word, if the rule has met it.
    pub fn op_of(&self, w: &str) -> Option<[f32; 3]> {
        self.op.get(w).copied()
    }

    /// Every word the rule has met in a solved question, with its weights (zeros included: knowing a word is harmless matters).
    pub fn all_ops(&self) -> Vec<(String, [f32; 3])> {
        let mut v: Vec<(String, [f32; 3])> = self.op.iter().map(|(w, o)| (w.clone(), *o)).collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    }

    /// The learned operator weights, in a stable order, keeping only words that do something.
    pub fn active_ops(&self) -> Vec<(String, [f32; 3])> {
        let mut v: Vec<(String, [f32; 3])> =
            self.op.iter().filter(|(_, o)| o.iter().any(|x| x.abs() > 0.05)).map(|(w, o)| (w.clone(), *o)).collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    }

    /// The answer word and how far the prediction sits from it (0 is certain). None if the question has no quantity in it, or
    /// no word in it that the rule has learned to treat as an operation (so "capital of three" is not its business).
    pub fn answer(&self, q: &str) -> Option<(String, f32)> {
        self.answer_ignoring(q, &|_| false)
    }

    /// As `answer`, but a word the rule never met in a solved question is let through if the caller says it is harmless filler
    /// (for instance because it is common in everything the baby has been taught). Any other unmet word makes it decline.
    pub fn answer_ignoring(&self, q: &str, filler: &dyn Fn(&str) -> bool) -> Option<(String, f32)> {
        let (nums, how_many, rest) = self.parse(q)?;
        // A word the rule never met in a solved question: it does not know what the question is asking.
        if rest.iter().any(|w| !self.op.contains_key(w) && !filler(w)) {
            return None;
        }
        let engaged: Vec<&[f32; 3]> = rest.iter().filter_map(|w| self.op.get(w)).filter(|o| o.iter().any(|x| x.abs() > 0.3)).collect();
        if engaged.is_empty() {
            return None;
        }
        // An operation that uses both numbers needs both. Never invent the missing one.
        if engaged.iter().any(|o| o[1].abs() > 0.3) && how_many < 2 {
            return None;
        }
        self.line.say(self.predict(&nums, &rest), uses_digits(q))
    }

    /// As `fit_ridge`, but a few wrong lessons do not bend the rule. Fit, find the lessons the rule misses by more than half a step, and refit
    /// without them, up to three times and never more than a tenth of the lessons. A lesson is only set aside if every other word in it is
    /// explained by other lessons: scattered mistakes qualify, but a second rule hiding in the unit (every "times" lesson in a unit of sums)
    /// does not, because nothing explains its operator word, so it stays and shows up as the unit no longer following one rule.
    pub fn fit_ridge_robust(train: &[Example], ridge: &dyn Fn(&str) -> f64) -> (Quantity, u64) {
        let mut keep: Vec<Example> = train.to_vec();
        let mut ops_total = 0u64;
        let allowed = train.len() / 10;
        let mut dropped = 0usize;
        let mut rounds = 0usize;
        loop {
            let (q, ops) = Quantity::fit_ridge(&keep, NumberLine::from_counting(&counting_lessons()), ridge);
            ops_total += ops;
            if dropped >= allowed || rounds >= 12 {
                return (q, ops_total);
            }
            rounds += 1;
            // how far off is each lesson, and what words does it say? (None: not a lesson the rule reads)
            let read: Vec<Option<(f32, Vec<String>)>> = keep
                .iter()
                .map(|e| match (q.parse(&e.query), q.line.value(&e.answer)) {
                    (Some((nums, _, rest)), Some(y)) => Some(((q.predict(&nums, &rest) - y).abs(), rest)),
                    _ => None,
                })
                .collect();
            // the median miss among the lessons that share each word
            let mut by_word: std::collections::HashMap<&str, Vec<f32>> = std::collections::HashMap::new();
            for r in read.iter().flatten() {
                for w in &r.1 {
                    by_word.entry(w.as_str()).or_default().push(r.0);
                }
            }
            let median = |w: &str| -> f32 {
                let v = by_word.get(w).cloned().unwrap_or_default();
                let mut v = v;
                v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                v.get(v.len() / 2).copied().unwrap_or(0.0)
            };
            // The worst lesson that is far off, and far off compared with its peers on every word it has (three times the median miss of
            // the lessons sharing the word, and at least half a step). Scattered mistakes are; every "times" lesson in a unit of sums is not,
            // because they all miss alike and so their median is as bad as they are.
            let mut bad: Vec<(usize, f32)> = read
                .iter()
                .enumerate()
                .filter_map(|(i, r)| r.as_ref().map(|r| (i, r)))
                .filter(|(_, (miss, words))| *miss > 0.5 && words.iter().all(|w| by_word[w.as_str()].len() >= 6 && *miss >= 3.0 * median(w)))
                .map(|(i, (miss, _))| (i, *miss))
                .collect();
            bad.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            bad.truncate(3.max(allowed / 12).min(allowed - dropped));
            if bad.is_empty() {
                return (q, ops_total);
            }
            let mut gone: Vec<usize> = bad.iter().map(|b| b.0).collect();
            gone.sort_unstable_by(|a, b| b.cmp(a));
            for i in gone {
                keep.remove(i);
                dropped += 1;
            }
        }
    }

    /// What the fitted rule says in the units of the answer, for every word that does anything: (per first number, per second number,
    /// constant). (The fit works on numbers divided by `SCALE`, so the constant is multiplied back.)
    pub fn terms(&self) -> Vec<[f32; 3]> {
        self.op.values().filter(|o| o.iter().any(|x| x.abs() > 0.03)).map(|o| [o[0], o[1], o[2] * SCALE]).collect()
    }

    pub fn operator(&self, word: &str) -> Option<[f32; 3]> {
        self.op.get(word).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{self, Family};

    fn sum_lessons(n: usize) -> Vec<Example> {
        let mut v = Vec::new();
        for a in 0..=5usize {
            for b in 0..=5usize {
                if v.len() < n {
                    v.push(Example { family: crate::data::Family::Color, template: 0, query: format!("what is {} plus {}", NUM[a], NUM[b]), answer: NUM[(a + b).min(10)].to_string() });
                }
            }
        }
        v
    }

    #[test]
    fn a_few_wrong_lessons_do_not_bend_the_rule_but_a_second_rule_hiding_in_the_unit_is_not_set_aside() {
        let ridge = |w: &str| if ["what", "is"].contains(&w) { 5.0 } else { 0.0 };
        let mut noisy = sum_lessons(36);
        // three lessons that are far out (a high leverage: wrong by seven or more)
        noisy[1].answer = "ten".into();
        noisy[2].answer = "ten".into();
        noisy[30].answer = "zero".into();
        let (plain, _) = Quantity::fit_ridge(&noisy, NumberLine::from_counting(&counting_lessons()), &ridge);
        let (robust, _) = Quantity::fit_ridge_robust(&noisy, &ridge);
        let clean = |q: &Quantity| sum_lessons(36).iter().filter(|e| q.answer(&e.query).map(|a| a.0) == Some(e.answer.clone())).count();
        assert!(clean(&robust) >= 35, "robust rule explains the clean lessons: {}", clean(&robust));
        assert!(clean(&robust) > clean(&plain), "and better than the plain fit ({} vs {})", clean(&robust), clean(&plain));
        // six multiplication lessons under "plus" words: their operator word is explained by nothing, so they stay (and the unit stays visibly polluted)
        let mut mixed = sum_lessons(36);
        for (a, b) in [(2, 3), (1, 5), (2, 5), (3, 3), (0, 4), (4, 2)] {
            mixed.push(Example { family: crate::data::Family::Color, template: 0, query: format!("what is {} times {}", NUM[a], NUM[b]), answer: NUM[(a * b).min(10)].to_string() });
        }
        let (robust, _) = Quantity::fit_ridge_robust(&mixed, &ridge);
        assert!(robust.operator("times").is_some_and(|o| o.iter().any(|x| x.abs() > 0.03)), "the times rule was kept, not discarded as noise");
    }

    #[test]
    fn solving_it_exactly_gives_the_same_rule_as_nudging_it_800_times_for_far_less_work() {
        let ds = data::build(1);
        let line = || NumberLine::from_counting(&counting_lessons());
        let iterative = Quantity::fit(&ds.train, line(), 800, 0.05);
        let (exact, ops) = Quantity::fit_ridge(&ds.train, line(), &|_| 0.0);
        let extra = data::extrapolation();
        let held: Vec<&Example> = ds.item_test.iter().chain(&extra).filter(|e| matches!(e.family, Family::Add | Family::Sub)).collect();
        assert!(held.len() > 50);
        for e in &held {
            assert_eq!(exact.answer(&e.query).map(|x| x.0), Some(e.answer.clone()), "{}", e.query);
            assert_eq!(iterative.answer(&e.query).map(|x| x.0), exact.answer(&e.query).map(|x| x.0));
        }
        let nudging = Quantity::iterative_fit_ops(&ds.train, &line(), 800);
        assert!(ops * 5 < nudging, "exact {ops} vs iterative {nudging}");
        let plus = exact.operator("plus").unwrap();
        assert!((plus[0] - 1.0).abs() < 0.02 && (plus[1] - 1.0).abs() < 0.02 && plus[2].abs() < 0.02, "{plus:?}");
    }

    #[test]
    fn counting_builds_a_number_line() {
        let line = NumberLine::from_counting(&counting_lessons());
        assert_eq!(line.value("zero"), Some(0.0));
        assert_eq!(line.value("seven"), Some(7.0));
        assert_eq!(line.nearest(7.2).unwrap().0, "seven");
        assert_eq!(line.value("banana"), None);
    }

    #[test]
    fn it_learns_plus_and_minus_from_examples_and_reaches_sums_it_never_saw() {
        let ds = data::build(1);
        let q = Quantity::fit(&ds.train, NumberLine::from_counting(&counting_lessons()), 800, 0.05);
        let right = |set: &[Example]| {
            let sums: Vec<&Example> = set.iter().filter(|e| matches!(e.family, Family::Add | Family::Sub)).collect();
            let ok = sums.iter().filter(|e| q.answer(&e.query).map_or(false, |(a, _)| a == e.answer)).count();
            (ok, sums.len())
        };
        let (ok, n) = right(&ds.item_test);
        assert_eq!(ok, n, "held-out sums");
        let extra = data::extrapolation();
        let (ok, n) = right(&extra);
        assert_eq!(ok, n, "sums beyond the taught range");
        let plus = q.operator("plus").unwrap();
        assert!((plus[0] - plus[1]).abs() < 0.3, "plus treats its two numbers alike: {plus:?}");
    }

    #[test]
    fn a_question_with_a_number_but_no_operation_is_left_alone() {
        let ds = data::build(1);
        let q = Quantity::fit(&ds.train, NumberLine::from_counting(&counting_lessons()), 800, 0.05);
        assert!(q.answer("capital of three").is_none());
        assert!(q.answer("what is sevn plus two").is_none(), "a missing operand must not be made up");
        assert!(q.answer("what is the sum of three and two").is_none(), "wording it never met");
        assert!(q.answer("what is three").is_none());
        assert_eq!(q.answer("seven plus two").map(|x| x.0), Some("nine".into()));
        let again = Quantity::from_ops(q.active_ops());
        assert_eq!(again.answer("eight minus five").map(|x| x.0), Some("three".into()));
    }

    #[test]
    fn questions_without_a_number_are_not_its_business() {
        let q = Quantity { line: NumberLine::from_counting(&counting_lessons()), op: HashMap::new() };
        assert!(q.answer("capital of france").is_none());
    }
}
